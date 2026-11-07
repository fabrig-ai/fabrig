+++
id = "view-words"
part = "view"
kind = "feature"
needs = []
files = [
  "temper/crates/temper-client-view/src/words.rs",
  "temper/crates/temper-client-view/src/limits.rs",
  "temper/crates/temper-client-view/src/tests/fixtures.rs",
  "temper/crates/temper-client-view/src/tests/fixtures/**",
  "skein/crates/skein-ui/src/phrases.rs",
  "skein/crates/skein-ui/src/tests/phrases.rs",
  "temper/docs/plan/limits-fixtures/figures/view-words.md",
]
effort = "high"
minutes = 100
+++

# view-words: what `temper-client-view`'s tests pay for words

## Goal

`temper-client-view`'s 1,427 tests outside `tests::limits` read 22.7 CPU s at a 13 ms median, over an 8 ms floor, where `temper-client-domain`'s floor is 2.6 ms. Find what every test pays, which reading suggests is the phrasebook, and fix it at the cause. A person's browser builds the same book, so prefer the product fix.

**Acceptance:**

- `figures/view-words.md` gives `Ir` before and after, by test name, for the binary's cheapest test and a median test outside `tests::limits`, each with what it built before its first assertion.
- The binary's cheapest test that uses `fixtures::words()` has `Ir` before and after too, because that is the fixture's own cost.
- The causes table has every cause over the stop rule: fixed, or with what fixing it would take.
- A product change has its own focused test in `skein-ui` or `temper-client-view`. What a phrase says, and every refusal `Phrasebook::load` gives, stays the same.
- Every test of `temper-client-view` and `skein-ui` passes, and its readers below compile and pass. The declared `PHRASES`, `SEGMENTS` and `ENGLISH_BYTES` stay as they are, unless their own test says otherwise.

## Inputs

- `temper/docs/plan/limits-fixtures/notes/common.md`: the rules, how to profile, your figures
- `temper/docs/plan/limits-fixtures/inputs/request.md`, items 1 and 3 and `For every task of this plan`
- `skein/docs/foundation/testing-strategy.md`, 8 and 8.1
- `skein/docs/design/ui.md`, its phrasebook section (find it with `grep -n Phrasebook`)
- `skein/docs/foundation/programming-model.md`, 10, for `skein-ui` and `temper-client-view`: both are step crates

## Owns and may change

- **Owns:** in `temper/crates/temper-client-view/src/words.rs`, `Words::english` and `english_language` only (the words themselves are other plans': don't move or change them); `src/limits.rs`; `src/tests/fixtures.rs` and `src/tests/fixtures/**`; `skein/crates/skein-ui/src/phrases.rs` and `src/tests/phrases.rs`; `figures/view-words.md`.
- **Leave to `tests::limits`' owner:** `src/tests/limits.rs` is another plan's.
- **May change, where the profile puts the cause:** `jig/crates/jig-client/src/words/english.rs` and `temper/crates/temper-client-domain/src/words.rs` and `src/forge/words.rs`, where they build their parts of the book, without changing any word.
- **Readers of `skein_ui::phrases`:** call sites only, if a public type's form must change: `jig/crates/jig-client/src/**`, `jig/crates/jig-client-view/src/**`, `jig/examples/ops/client-view/src/**`, `jig/testing/jig-client-test-view/src/**`, `jig/testing/jig-client-kit/src/**`, `skein/examples/ui/view/src/**`, `temper/crates/temper-client-domain/src/**`.
- **Options, each pre-approved:**
  - **Keep `Segment::Text(Box<[u8]>)` if you can.** It has 986 uses across three trees and they'd all change. Look first at what `Phrasebook::load` does with what it's given, and at whether the book is built or cloned more than it need be.
  - **If `fixtures::words()`'s deep clone is the cost** (`tests/fixtures.rs:59`, a `OnceLock` and a `clone` per call, 279 calls in these tests), make that clone cheap or unneeded. `Rc` and `Arc` are disallowed by `skein/clippy.toml` and `temper/clippy.toml` ("one owner"), so first see whether `Words` can lend the book, or whether a test can hold one `Words`. A scoped `#[expect]` with its reason is allowed only in a test fixture, never in a step crate.
  - **If the sorts and checks of `load` are the cost** (it sorts shapes and phrases with derived `Ord`, then checks them), a cheaper form that makes the same checks and gives the same refusals is yours to choose: for example, sorting by id alone.
  - **If `View::new` or `view::Limits::of` is the cost,** (`src/limits.rs:17`), it's worth the same care: it runs in every client world too.
  - **If the cost is in the binary and not in any test** (relocations, as `notes/common.md` says), a line in the causes table with its share, and leave it.
  - **If `temper-docs` or `temper-fixtures` shows up,** say so: read why, and fix it if a test has no need of it. `temper-fixtures` is a neighbour you may change, for what this crate's tests call.

## Steps

**From reading, not found (`notes/common.md`):**

- `Words::english` (`src/words.rs:4336`) builds about 1,887 phrases and 3,316 segments through `english_language` (`:4211`), each with its own allocations, adds jig's and the domain's words, and hands them to `Phrasebook::load` (`skein/crates/skein-ui/src/phrases.rs:224`), which sorts and checks them. In an unoptimized build that's a lot of small steps.
- `tests/fixtures.rs` keeps one `Words` per process and clones it deeply for each `words()`: a `Phrasebook` holds boxed shapes, phrases, forms and segments.
- A test that makes a `View` also runs `view::Limits::of` (`src/limits.rs:17`).

**The work:**

1. Choose the tests and take the before figures, at your branch's start. Use `-p temper-client-view --lib`, leaving out `tests::limits::`.
2. Read the inclusive annotations. Say how much is the loader, `english`, `load`, the clones and `Limits::of`.
3. Fix the largest cause. Run the narrowest test (`heavy cargo nextest run -p temper-client-view --lib -E 'test(=NAME)'`, or `-p skein-ui`), and read its `Ir` again.
4. Repeat until the stop rule (`notes/common.md`).
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py` and `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-ui -p temper-client-view --all-targets -- -D warnings`, naming any other package you changed.
6. `heavy cargo nextest run -p temper-client-view -p skein-ui`. A change in `skein-ui` or in `Words` also runs its readers: `heavy cargo nextest run -p temper-client-world -p jig-client-view -p jig-client`, and `heavy cargo check -p temper-client-shell-terminal -p temper-client-shell-web -p temper-client-gallery -p temper-playground-world -p jig-client-test-view -p jig-client-kit --tests`.
7. Write the figures, commit and push.

## Report

The counts before and after, the causes fixed and not fixed, whether a public type changed, and which design section it touches, if any: `docs/design` isn't yours, so `close-figures` writes that.
