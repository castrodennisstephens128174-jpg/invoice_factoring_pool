#![no_std]

use soroban_sdk::{contract, contractimpl, contracttype, Address, Env, Map, Symbol, Vec, symbol_short};

/// Represents a single invoice that has been listed into the factoring pool.
/// Stored under a composite key of `(INVOICE, invoice_id)` in persistent storage.
#[contracttype]
#[derive(Clone)]
pub struct Invoice {
    /// The small business (supplier) that listed the invoice.
    pub business: Address,
    /// The party that is obligated to pay the invoice when it falls due.
    pub debtor: Address,
    /// The total face value of the invoice in the smallest token unit.
    pub amount: u64,
    /// Unix timestamp (seconds) at which the invoice is due.
    pub due_date: u64,
    /// The amount that investors have currently committed to this invoice.
    pub funded_amount: u64,
    /// Whether the debtor has paid the invoice and unlocked investor payouts.
    pub settled: bool,
    /// Per-investor contributions. A value of `0` means the investor has already
    /// claimed (or never participated).
    pub contributors: Map<Address, u64>,
}

/// Symbol used as a tag for the running invoice counter.
const COUNTER: Symbol = symbol_short!("COUNTER");
/// Symbol used as a tag for the list of every invoice_id that has been listed.
const ID_LIST: Symbol = symbol_short!("ID_LIST");
/// Symbol used as a tag for the invoice record itself (composite key).
const INVOICE: Symbol = symbol_short!("INVOICE");

#[contract]
pub struct InvoiceFactoringPool;

#[contractimpl]
impl InvoiceFactoringPool {
    /// List a brand-new invoice into the factoring pool so that investors can fund it
    /// at a discount. The business (supplier) must authorize this call.
    ///
    /// `amount` is the face value the debtor will pay at `due_date`. The total amount
    /// funded by investors cannot exceed `amount`. Re-listing an existing `invoice_id`
    /// is rejected.
    pub fn list_invoice(
        env: Env,
        business: Address,
        invoice_id: Symbol,
        amount: u64,
        debtor: Address,
        due_date: u64,
    ) {
        business.require_auth();

        if amount == 0 {
            panic!("invoice amount must be positive");
        }
        if due_date <= env.ledger().timestamp() {
            panic!("due date must be in the future");
        }
        if env
            .storage()
            .persistent()
            .has(&(INVOICE, invoice_id.clone()))
        {
            panic!("invoice already exists");
        }

        let invoice = Invoice {
            business,
            debtor,
            amount,
            due_date,
            funded_amount: 0,
            settled: false,
            contributors: Map::new(&env),
        };

        env.storage()
            .persistent()
            .set(&(INVOICE, invoice_id.clone()), &invoice);

        // Track the invoice id in a contract-level list so it can be enumerated.
        let mut ids: Vec<Symbol> = env
            .storage()
            .instance()
            .get(&ID_LIST)
            .unwrap_or_else(|| Vec::new(&env));
        ids.push_back(invoice_id);
        env.storage().instance().set(&ID_LIST, &ids);

        // Bump the invoice counter.
        let count: u32 = env.storage().instance().get(&COUNTER).unwrap_or(0u32);
        env.storage().instance().set(&COUNTER, &(count + 1));
    }

    /// Fund a portion of a listed invoice. An investor may call this multiple times
    /// for the same invoice; each contribution is added to the investor's share and
    /// the invoice's running `funded_amount`.
    ///
    /// Returns the new total amount funded for the invoice after this contribution.
    /// Funding more than the face value is rejected.
    pub fn fund_invoice(
        env: Env,
        investor: Address,
        invoice_id: Symbol,
        amount: u64,
    ) -> u64 {
        investor.require_auth();

        if amount == 0 {
            panic!("funding amount must be positive");
        }

        let key = (INVOICE, invoice_id);
        let mut invoice: Invoice = env
            .storage()
            .persistent()
            .get(&key)
            .expect("invoice not found");

        if invoice.settled {
            panic!("invoice already settled");
        }
        if env.ledger().timestamp() > invoice.due_date {
            panic!("invoice past due date");
        }

        let new_funded = invoice
            .funded_amount
            .checked_add(amount)
            .expect("funding overflow");
        if new_funded > invoice.amount {
            panic!("funding exceeds invoice amount");
        }

        let prior = invoice
            .contributors
            .get(investor.clone())
            .unwrap_or(0u64);
        let new_share = prior
            .checked_add(amount)
            .expect("contributor share overflow");
        invoice.contributors.set(investor, new_share);
        invoice.funded_amount = new_funded;

        env.storage().persistent().set(&key, &invoice);
        new_funded
    }

    /// Settle the invoice as the debtor. The `amount` paid must equal the currently
    /// funded amount; the pool records the settlement and unlocks investor payouts.
    /// The actual transfer of funds into the pool is performed by the caller (this
    /// contract never moves XLM or any other asset on its own).
    pub fn settle_invoice(env: Env, debtor: Address, invoice_id: Symbol, amount: u64) {
        debtor.require_auth();

        let key = (INVOICE, invoice_id);
        let mut invoice: Invoice = env
            .storage()
            .persistent()
            .get(&key)
            .expect("invoice not found");

        if invoice.settled {
            panic!("invoice already settled");
        }
        if invoice.debtor != debtor {
            panic!("only the named debtor can settle");
        }
        if invoice.funded_amount == 0 {
            panic!("cannot settle an unfunded invoice");
        }
        if amount != invoice.funded_amount {
            panic!("settlement amount must equal funded amount");
        }

        invoice.settled = true;
        env.storage().persistent().set(&key, &invoice);
    }

    /// Claim an investor's share from a settled invoice. Because the settlement amount
    /// is required to equal the funded amount, each investor's payout equals their
    /// recorded contribution. The contribution is zeroed out so that a claim cannot
    /// be replayed.
    ///
    /// Returns the payout amount that should be released to the investor off-chain.
    pub fn claim_payout(env: Env, investor: Address, invoice_id: Symbol) -> u64 {
        investor.require_auth();

        let key = (INVOICE, invoice_id);
        let mut invoice: Invoice = env
            .storage()
            .persistent()
            .get(&key)
            .expect("invoice not found");

        if !invoice.settled {
            panic!("invoice not yet settled");
        }

        let share = invoice
            .contributors
            .get(investor.clone())
            .unwrap_or(0u64);
        if share == 0 {
            panic!("no contribution to claim");
        }

        // Mark this contribution as claimed by zeroing it out.
        invoice.contributors.set(investor, 0u64);
        env.storage().persistent().set(&key, &invoice);

        share
    }

    /// Returns the funding progress of an invoice as a percentage in the range 0-100.
    /// Useful for frontends showing a progress bar while an invoice is being filled.
    pub fn funding_progress(env: Env, invoice_id: Symbol) -> u32 {
        let key = (INVOICE, invoice_id);
        let invoice: Invoice = env
            .storage()
            .persistent()
            .get(&key)
            .expect("invoice not found");

        if invoice.amount == 0 {
            return 0;
        }

        (invoice.funded_amount as u128 * 100u128 / invoice.amount as u128) as u32
    }

    /// Returns the total number of invoices that have ever been listed, whether or
    /// not they have been settled.
    pub fn invoice_count(env: Env) -> u32 {
        env.storage().instance().get(&COUNTER).unwrap_or(0u32)
    }

    /// Returns the list of invoice IDs that have been listed into the pool. Useful
    /// for an off-chain indexer to enumerate and display the marketplace.
    pub fn list_invoices(env: Env) -> Vec<Symbol> {
        env.storage()
            .instance()
            .get(&ID_LIST)
            .unwrap_or_else(|| Vec::new(&env))
    }
}
