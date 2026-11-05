# D022: The oauth client and accounts keep lifecycles in flags and parallel lists

- **Kind:** complexity
- **Where:** `skein-oauth/src/machine.rs`, `skein-oauth-accounts/src/component.rs`, `skein-oauth-accounts/src/exchange.rs`
- **Seen:** 2026-10-31 at `4a6683c72`, in `reviews/2026-10-31-skein.md` (theme T4, the oauth row) and `reviews/2026-10-31-skein-areas.md` (A10)
- **Seen:** 2026-10-31 at `337102750`, in `reviews/2026-10-31-skein-first.md` (theme 2)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** medium
- **Size:** M
- **Status:** open
- **Paid when:** the client's `Phase` holds its exchange instead of a separate `Option<Exchange>`; the accounts component holds one entry per account, not parallel lists; account states hold their data instead of flags (`skein/docs/foundation/programming-model.md`, 5.4).

## What

- `skein-oauth`'s `Client` keeps a `phase: Phase` beside
  `active: Option<Exchange>` (`machine.rs`), and decides by
  `self.phase != Phase::Idle` comparisons.
- `skein-oauth-accounts`' `Component` keeps per-account state in
  parallel lists (`accounts`, `names`, `sources`, `bindings`, `keepers`
  in `component.rs`); the review counted 124 reads across five lists, a
  `State::Record` with five flags and a 19-field `Exchange`.

## Why it matters

The same shape: positions instead of entries, and flags whose
meaning depends on a state the type does not hold. The phase
comparisons are where the review found `SignIn`, `Refresh` and
`Revoke` dropped outside `Idle` with no terminal.

## What pays it back

The client's exchange inside its phase; one `AccountEntry` per account;
record states as variants. One crate per task.
