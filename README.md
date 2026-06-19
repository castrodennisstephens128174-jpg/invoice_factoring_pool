# invoice_factoring_pool

## Project Title
invoice_factoring_pool

## Project Description
Small and medium businesses (SMBs) often have to wait 30, 60, or 90 days to be paid by their corporate debtors, which creates a painful cash-flow gap. Traditional invoice factoring is expensive, paper-heavy, and locked behind intermediaries. `invoice_factoring_pool` is a Soroban smart contract that lets an SMB list an invoice (amount, debtor, due date) into an on-chain pool, allows any investor to fund the invoice at a discount, and then distributes the debtor's eventual payment back to the investors proportionally. Every actor is a `stellar` account, every action is authorized via `require_auth`, and the contract itself never custodies real XLM — it is a transparent, auditable settlement layer for invoice financing on the Stellar network.

## Project Vision
Bring the $3 trillion global invoice-factoring market on-chain by making it cheap (sub-cent Stellar fees), transparent (every listing and contribution is visible on-ledger), and globally accessible (any investor with a Stellar wallet can participate, not just accredited institutions). The long-term goal is to give any business with verifiable receivables instant liquidity and give any saver an asset class that is short-duration, asset-backed, and yield-bearing by construction.

## Key Features
- **List an invoice (`list_invoice`)** — A business registers an invoice with a face value, a debtor address, and a due date. The invoice enters the pool and becomes visible to investors.
- **Fractional funding (`fund_invoice`)** — Multiple investors can each contribute any amount up to the remaining face value. Each contribution is recorded against the investor's address, so ownership shares are explicit on-chain.
- **Debtor settlement (`settle_invoice`)** — The named debtor authorizes a single payment equal to the funded amount, which marks the invoice as settled and unlocks payouts.
- **Proportional claim (`claim_payout`)** — Once settled, each investor pulls out their own share. Double-claiming is impossible because a claimed contribution is zeroed out on-chain.
- **Live funding progress (`funding_progress`)** — A read-only helper that returns the percentage funded (0-100), so a frontend can show a progress bar to the marketplace.
- **Enumerable marketplace (`list_invoices`, `invoice_count`)** — The contract keeps a running list of every invoice id, so an off-chain UI can build a marketplace page without an external indexer.

## Contract

- **Network:** Stellar Testnet (Public)
- **Scope:** finance dApp — see `contracts/invoice_factoring_pool/src/lib.rs` for the full invoice_factoring_pool business logic.
- **Functions exposed:** see `Key Features` above and the `pub fn` list in `lib.rs`.
- **Contract ID:** `CDCEECTOL62SZ4Z2MKDQLFDPGQP6RNOPRVYWWXIMGDD7FRAKNZJ5LSHE`
- **Explorer template:** `https://stellar.expert/explorer/testnet/tx/14fba9a3e32917f1ebb8d916650b7d1af4737f27b1d78579fe7ecf115271a4ad`



## Future Scope
- **Stable-settlement asset:** integrate a Stellar stablecoin (e.g. USDC on Stellar) as the settlement unit so that payouts are stable rather than XLM-denominated.
- **KYC / accredited-investor gating:** add an admin-managed allowlist of investor addresses so that the pool can be used in jurisdictions that require it.
- **Auto-default handling:** extend the contract with a `claim_default` flow that, after `due_date + grace_period`, returns the funded amount to investors (with optional penalty to the business) when the debtor never pays.
- **Multi-invoice bundles:** let a business list several invoices under one listing, so investors can fund a diversified basket in a single transaction.
- **Off-chain attestation oracle:** plug in a Stellar oracle (e.g. via the `stellar-expert` price feed pattern) to verify the debtor's credit rating and surface a risk score on the listing.
- **Frontend dApp:** a React/Freighter-based UI that lists open invoices, lets investors fund and claim, and lets debtors settle — turning the contract into a complete marketplace.

## Profile

- **Name:** <!-- Fill github name -->
- **Project:** `invoice_factoring_pool` (finance)
- **Built with:** Soroban SDK 25, Rust, Stellar Testnet
