+++
id = "forge-fixtures"
part = "forge-fixtures"
kind = "feature"
needs = ["seam-shapes", "hand-leftovers"]
files = ["skein/crates/skein-forge-connection/src/fixtures.rs", "skein/crates/skein-forge-connection/src/lib.rs", "skein/crates/skein-forge-connection/src/tests/fixtures.rs", "skein/tests/forge/machine/src/hand.rs", "skein/tests/forge/machine/src/hand/**", "skein/tests/forge/machine/tests/suite/**", "skein/docs/development/debt/D050-two-forge-harnesses.md", "skein/docs/development/debt/README.md", "skein/docs/design/forge-connection.md"]
effort = "high"
minutes = 120
+++

# Two forge harnesses in one package: one fixtures module, and the twins cut

## Goal

D030 closed mostly by moving code: `hand` and the seeded `machine` tests sit over one component in one package, with near-twin tests, and `hand` and the crate's test fixtures hold the same items. Open skein's item D050 for it and pay it here: a seam-gated fixtures module both read, and the twins cut. The two harnesses stay: the one-harness clause was dropped on +131's evidence, and that stands.

Acceptance:

- `skein-forge-connection/src/fixtures.rs` exists, `#[cfg(any(test, feature = "seam"))] pub mod fixtures;` in `lib.rs`, and holds what the crate's `src/tests/fixtures.rs` and the world's `hand` both had.
- No fixture item is written twice across those two. On the cut's count that's eleven items, 59 lines: `ok`, `response`, `response_with_fields`, `user_ok`, `header`, `TOKEN_BOUND`, `filled`, `OTHER_FORGE`, `BRANCH`, `BRANCH_HEAD` and `github_settings`, with what those need.
- Every twin the steps below find is cut, and the rule stays pinned by the twin that's left. Your report holds one row per cut: the test cut, the test kept, and any assertion you carried across.
- `skein/docs/development/debt/README.md`'s highest ID is D050, whatever happens to the item. If D050 is taken on your base, take the next free ID and say so.
- The item is gone if this paid it in substance. Otherwise `skein/docs/development/debt/D050-two-forge-harnesses.md` stays, saying what's left.
- `skein/docs/design/forge-connection.md`, 10, *The seam* holds the sentence below.
- The package's test lines fall. The review reckons about -180 on fixtures and -300, rough, on twins; the cut's count finds only about -60 on fixtures, so judge the fixtures by the "no item twice" above and not by that number. Report the count.
- `skein-forge-connection`'s lib tests and `skein-forge-machine-world`'s, focused and fuzzy, pass.

## Inputs

- `skein/docs/design/forge-connection.md`, 10: *By hand*, *The seam*.
- `skein/docs/design/testing.md`, 6, on what a kit and a world hold.
- `skein/docs/development/debt/README.md`, the item's form.
- `temper/docs/plan/debt-26/inputs/debt24-review.md`, first review, finding 1.
- `temper/docs/plan/debt-26/notes/common.md`.

## Owns and may change

Owns the files in `files`, the one design sentence being a stated departure from the implementer's text. No `skein-forge-kit` change: it can't turn `seam` on, since temper's and other worlds read it and only the machine world may (`tests/boundary/tests/skein_worlds.rs` holds that). So the kit keeps its own `limits`, `shared` and `settings`, a copy forced by that rule, and you say so in the report.

Options, each pre-approved:

- **The module's reach.** `github_settings` needs `settings` and `app_access`: move those as well, and whatever else the eleven need. The crate's `limits`, `shared` and `settings` may move too, and `hand` then reads them from the module rather than from the kit, if `limits`' one comment and `shared`'s field order are all that differ. Leave a difference of value alone, and say so.
- **`PAGE_ROWS`** differs (11 lines against 4): leave both.
- **A step crate's module.** The module lives in a `no_std` crate, under the workspace's lints when clippy runs on `test` or `seam`. Write it to the subset: `#[must_use]` on what it returns, and nothing `std`.
- **What goes when a twin goes:** fixtures only that test read go with it (`EVICTION_*`, `ready_two`, `branch_answers`, if nothing else does). A file left with no test goes, with its `mod` line.

## Steps

The module's reader lines:

```rust
// skein-forge-connection/src/lib.rs
/// What the crate's tests and the machine world's `hand` both read (forge-connection.md, 10).
#[cfg(any(test, feature = "seam"))]
pub mod fixtures;
// skein-forge-connection/src/tests/fixtures.rs
pub use crate::fixtures::{BRANCH, BRANCH_HEAD, OTHER_FORGE, TOKEN_BOUND, filled, github_settings, header, ok, response, response_with_fields, user_ok};
// skein/tests/forge/machine/src/hand.rs
pub use skein_forge_connection::fixtures::{ /* the same */ };
```

The sentence in `forge-connection.md`, 10, *The seam*, at its end: `The fixtures the crate's tests and the machine world's hand both read are the crate's fixtures module, built for either.`

**Finding twins.** Walk `tests/suite/machine.rs` and `tests/suite/github_*.rs` against `tests/suite/hand/forgejo/*` and `tests/suite/hand/github/*`. Two tests are twins when they drive the same rule to the same outcome, once you ignore harness bookkeeping: `script_done`, `close`, `requests().len()`, connection and delivery counts that only restate the script, `h.answers.len()`, and `assert_eq!` on a harness's own setup. The review's two to start from, both checked by the cut:

- `machine.rs`'s `a_start_doubles_its_backoff_to_the_cap`, and `hand/forgejo/connection.rs`'s `start_backoff_doubles_to_its_maximum`. `hand` also asserts `Start::Waiting`'s `until` and `next`.
- `an_idle_connection_is_reassigned_to_another_forge`, in `machine.rs` and in `hand/forgejo/connection.rs`.

**Which goes.** Keep the seeded test, which survives seeds and runs the whole stack, unless the rule lives in an interleaving only a hand can set up (a connect completing when the test says, a fault in literal bytes): then keep `hand`'s. An assertion the cut twin holds and the kept one doesn't, bookkeeping aside, moves across if the kept harness can see it; if it can't, the two aren't twins and both stay. Don't merge the harnesses, and don't turn cases into new tables unless that cuts lines. Merely alike isn't twin: leave what pins something else, and list the closest calls in the report.

**The item.** Write it in the README's form with:

- `# D050: Two harnesses over forge-connection sit in one package, with near-twin tests`
- **Kind:** tests. **Where:** `tests/forge/machine/src/hand.rs`, `tests/forge/machine/tests/suite`. **Seen:** 2026-11-06 at `683f4e8b92`, in the spot review of debt-24 (finding 1). **Checked:** 2026-11-06. **Value:** low. **Size:** M. **Status:** open.
- **Paid when:** no fixture is written twice across the crate's test fixtures and `hand`, and no `hand` test repeats a seeded test's rule, comparing without harness bookkeeping. Two harnesses stay.

Delete it if both hold when you've done. If what's left is real (a family of twins you couldn't cut, a fixture a compile refuses), keep the file, with **What** naming what's left and why. Cite the review by name and merge, never by a path under `temper/docs/plan/`.

Tests: `heavy cargo nextest run -p skein-forge-connection --lib`, `heavy cargo nextest run -p skein-forge-machine-world`, and `heavy cargo nextest run -p skein-forge-machine-world --profile fuzzy`. Check the counts: they fall by exactly the tests you cut. Then fmt, `python3 -I -B scripts/old_forms.py` from `skein/`, `heavy cargo clippy -p skein-forge-connection --all-targets -- -D warnings`, the same with `--features seam`, `heavy cargo clippy -p skein-forge-machine-world --all-targets -- -D warnings`, and the debt and citations tools. Commit, push.

## Report

As your text says: the table of cuts, the closest calls you left, lines before and after by file group, whether the item went, and what the kit keeps.
