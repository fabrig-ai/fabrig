+++
id = "lib-accounts-slab"
part = "lib-sites"
kind = "migration"
needs = ["lib-oauth", "kit-slab"]
files = ["skein/crates/skein-oauth-accounts/src/component.rs", "skein/crates/skein-oauth-accounts/src/exchange.rs"]
effort = "high"
minutes = 40
+++
# oauth-accounts walks its exchanges with the slab's cursor

## Goal

Draft, 3.4 (slab iteration); the review's 1d: oauth-accounts keeps a
linked list over its `exchanges` slab (`first_exchange`, each
`Exchange::next`) and walks it with nine `while let` loops, because lib
had no slab iteration. It walks the slab with `Slab::first` and
`Slab::after` instead, and the list goes.

Acceptance:

- `first_exchange` and `Exchange::next` are gone; every walk is `let mut
  next = self.exchanges.first(); while let Some(id) = next { next =
  self.exchanges.after(id); ... }`; `insert_exchange` and
  `retire_exchange` no longer link or unlink (`component.rs` ~995-1030).
- Behaviour holds: `heavy cargo nextest run -p skein-oauth-accounts -p
  skein-oauth-accounts-world` and `heavy cargo nextest run -p
  skein-oauth-accounts-world --profile fuzzy` pass (counts named);
  clippy clean.
- The report gives the lines removed and the walks moved (path:line).

## Inputs

- `skein/docs/plan/component-kit-2/notes/common.md`, `notes/lib.md`.
- `skein/docs/design/oauth.md`, 6.5 (the exchanges and their reclaim):
  whether it orders the exchanges.
- `Slab::first`, `Slab::after` (`skein/crates/skein-lib/src/slab.rs`) on
  your base: live entities in slot order, retired ones skipped.
- The walks: `src/component.rs:543`, `:1001`, `:1020`, `:1034`, `:1136`,
  `:1171` and the rest found by `rg 'first_exchange|\.next\b'
  crates/skein-oauth-accounts/src`.

## Owns and may change

- `src/component.rs` and `src/exchange.rs` (where `Exchange` holds
  `next`, if it is there). Nothing else.

Pre-approved:

- **Order.** The list kept insertion order; the cursor walks slot order,
  which differs once a slot is reused. If oauth.md does not order the
  exchanges and a test's expectation changes only in the order two
  exchanges' events go up, update the expectation and name the test. If
  oauth.md orders them (oldest first, say), or a test pins the order as
  a rule, stop with a `Question:` and leave the list.
- **Digests.** A state digest that changes because the `next` links are
  no longer state is expected: update a pinned digest and name it.

## Steps

1. Replace each walk; drop the links; keep each loop's body as it is.
2. Checks of `common.md` (no count changes); commit; push.

## Report

Per `common.md`, with the order question's outcome.
