+++
id = "temper-account-counts"
part = "temper-items"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-domain/src/limits.rs",
  "temper/crates/temper-client-view/src/accounts.rs",
  "temper/crates/temper-client-view/src/pages/accounts.rs",
  "temper/crates/temper-client-view/src/pages/accounts/rows.rs",
  "temper/crates/temper-client-view/src/root.rs",
  "temper/crates/temper-client-view/src/tests/accounts.rs",
  "temper/docs/development/debt/D009-interim-account-counts.md",
  "temper/docs/development/debt/README.md",
]
effort = "medium"
minutes = 35
+++

# temper D009: the client reads jig's account counts

## Goal

temper's client has one set of account counts, `jig_client::AccountCounts`. `AccountLimits` goes. The one count jig lacks, how many uses an account's row names, becomes `temper_client_domain::limits::Limits::row_uses`, with a comment that says why it is temper's. This task closes the item.

Acceptance:

- `rg -n 'AccountLimits' temper --type rust` reads nothing.
- `Limits` has `forge`, `client` and `row_uses`, and `row_uses`'s comment says why jig does not hold it.
- The view's account pages read `accounts`, `sources` and `sign_ins` from `limits.client.domain.accounts`, and `Bindings` keeps a `jig_client::AccountCounts`.
- Each shell and fixture sets the three counts once; only `row_uses: 8` sits beside them.
- `temper/docs/development/debt/D009-interim-account-counts.md` and its row are gone.
- `heavy cargo nextest run -p temper-client-domain -p temper-client-view -p temper-client-tools -p temper-client-shell-terminal -p temper-client-shell-web` passes, its count above zero, and no assertion of an existing test changes.
- `heavy cargo check --tests -p temper-client-world -p temper-client-gallery` passes; clippy passes on the five packages above.

## Inputs

- `temper/docs/plan/debt-5/notes/common.md`, `AGENTS.md`, `temper/AGENTS.md`.
- `temper/docs/development/debt/D009-interim-account-counts.md`, whole. Its line that `binding_capacity` reads `uses` is old: the function reads `sources` and `accounts`.
- `jig/docs/design/client/domain/client.md`, 14, for what `AccountCounts` holds.
- `jig-client-domain/src/limits.rs`: the counts are `jig_client::DomainLimits::accounts`, which `temper_client_domain::limits::Limits` reaches as `limits.client.domain.accounts`.

Every shell and every literal at base sets `accounts: 16, sources: 8, sign_ins: 4` in both places, so no value changes. The comment on `AccountLimits` is also a site of D008: `temper-pipeline-names` leaves this file to you, and the comment goes with the type.

## Owns and may change

- Owns: `temper-client-domain/src/limits.rs`; `temper-client-view/src/accounts.rs`, `src/pages/accounts.rs`, `src/pages/accounts/rows.rs`, `src/root.rs`, `src/tests/accounts.rs`.
- Each file below changes one literal's line, and the comment above it where one says "sources/uses": `temper-client-shell-web/src/limits.rs`, `temper-client-tools/src/limits.rs`, `temper-client-shell-terminal/src/limits.rs`, `temper/tests/client/src/fixtures.rs`, `temper-client-view/src/tests/fixtures.rs`, `temper-client-view/tests/suite/{task_pages,change_page}.rs`, `temper-client-domain/src/tests/fixtures.rs`, `temper-client-domain/tests/suite/{task_pages,change}.rs`.
- No jig file changes. `temper/docs/design/` names none of these items, so no design text changes.
- Pre-approved: `src/tests/accounts.rs`'s `bounded_view` writes `domain.client.domain.accounts`, and its one test passes `accounts: 0`. If `crate::Limits::of` or `View::new` refuses that value or sizes something by it, compute `bounds` from the unchanged limits and write the counts after, so the test still draws no row. Say so in the report.

## Steps

1. `temper-client-domain/src/limits.rs`:

   ```rust
   pub struct Limits {
       pub forge: forge::Limits,
       pub client: jig_client::Limits,
       /// The uses one account row names before it says how many more. jig's
       /// `AccountCounts` bounds what the client receives; what a row draws is temper's.
       pub row_uses: u32,
   }
   ```

   `AccountLimits` and its comment go.

2. `temper-client-view/src/accounts.rs`:

   ```rust
   pub(crate) struct Bindings { entries, next, revision, counts: jig_client::AccountCounts, row_uses: u32 }
   pub fn new(counts: jig_client::AccountCounts, row_uses: u32, providers: u32) -> Self
   pub const fn counts(&self) -> jig_client::AccountCounts
   pub const fn row_uses(&self) -> u32
   pub(crate) fn binding_capacity(counts: &jig_client::AccountCounts, providers: u32) -> u32
   ```

   `src/root.rs`: `Bindings::new(domain.client.domain.accounts, domain.row_uses, domain.client.domain.rows)`. `src/pages/accounts.rs` and `src/pages/accounts/rows.rs` take `jig_client::AccountCounts` where they took `AccountLimits`; `rows.rs`'s `pays` call reads `bindings.row_uses()`.

3. Each literal `accounts: ..::AccountLimits { accounts: 16, sources: 8, uses: 8, sign_ins: 4 }` becomes `row_uses: 8`. The shells' comment above it becomes one line about what remains, such as `// An account's row names eight uses.`

4. `src/tests/accounts.rs`: the counts come from `fixtures::limits().client.domain.accounts`, and `d::limits::AccountLimits` becomes `jig_client::AccountCounts`.

5. The item's file and its row; citations. Run what the acceptance names; commit (`D009: temper's client reads jig's account counts`, saying why) and push.

## Report

The lines added and removed, and whether the pre-approved case arose.
