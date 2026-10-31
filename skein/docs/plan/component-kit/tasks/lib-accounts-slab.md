+++
id = "lib-accounts-slab"
part = "lib-sites"
kind = "migration"
repo = "fabrig"
needs = ["ratchets", "kit-slab", "lib-accounts"]
files = ["skein/crates/skein-oauth-accounts/src/**", "skein/scripts/old-forms.txt"]
effort = "high"
minutes = 40
+++
# skein-oauth-accounts walks its slab, not a list over it

Read first: `skein/docs/plan/component-kit/notes/common.md`, `skein/docs/plan/component-kit/notes/lib-common.md`, `skein/docs/plan/component-kit/notes/decisions.md`.

Draft, 3.4, and the review's 1d: without slab iteration, oauth-accounts
hand-rolls a linked list over its slab with nine `while let` walks.

## What changes

- Find the nine walks (`rg -n 'while let' src/`) and the links they
  follow. Each becomes a `for` over `Slab::iter`, filtered as the walk
  was. The links, and the code that keeps them, go where no other reader
  needs them.
- Where a walk depends on the list's order (insertion order, not slot
  order) and a test pins it, keep that walk and name it with the test.

## Acceptance

- No `while let` walk over the slab's links remains, but those named.
- `heavy cargo nextest run -p skein-oauth-accounts -p
  <tests/oauth-accounts' package>`, default then fuzzy, with counts;
  report the lines removed and any state digest that changed (a digest
  that hashed the links).
