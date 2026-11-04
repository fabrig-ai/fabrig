+++
id = "jig-ui-callers"
part = "jig-ui-callers"
kind = "feature"
needs = []
files = [
  "jig/crates/jig-client-view/src/**",
  "jig/testing/jig-client-test-view/src/connector_values.rs",
  "jig/testing/jig-client-test-kit/tests/suite/check.rs",
  "jig/testing/jig-client-test-domain/src/tests/root/navigation.rs",
  "jig/tests/client/src/observe.rs",
  "jig/tests/client/src/model.rs",
  "jig/tests/client/src/harness/world/words_tests.rs",
  "jig/tests/client/src/observe/link_tests.rs",
  "jig/tests/client/tests/suite/referee.rs",
  "jig/tests/ops/src/joint/fixture.rs",
  "jig/docs/development/debt/D022-ui-limits-literals.md",
  "jig/docs/development/debt/D023-view-tests-text-bytes.md",
  "jig/docs/development/debt/README.md",
]
effort = "medium"
minutes = 70
+++

# jig D022 and D023: callers of `Limits::EMPTY` and `Node::text_bytes`

## Goal

skein-ui gives `Limits::EMPTY` and a public `Node::text_bytes` (`skein/docs/design/ui.md`, 15).
jig's 25 `skein_ui::Limits` literals that spell `scene_marks: 0` move onto the constant, and the
view's tests read a node's bytes from skein-ui instead of copying its accounting. This pays
`jig/docs/development/debt/D022-ui-limits-literals.md` and
`jig/docs/development/debt/D023-view-tests-text-bytes.md` whole, on one branch, in two commits.

Acceptance:

- D022's Measure, `rg -U -c '(skein_ui::)?Limits \{[^{}]*?scene_marks: 0' -g '*.rs' jig`, prints
  nothing (25 at base), and `rg -n 'scene_marks: 0|scene_bytes: 0' jig/crates jig/testing
  jig/tests -g '*.rs'` prints nothing: the second search catches a literal with a nested
  brace, which the Measure skips.
- `rg -n 'fn text_bytes|tests::text_bytes' jig/crates/jig-client-view/src` prints nothing (the
  helper and its 34 uses in 16 files at base).
- **D023's Measure stays above zero, and that is expected:** `rg -l 'text_bytes'` also counts
  `skein_ui::Limits`'s field and `PhraseLimits`'s, in 38 files, and the method `Node::text_bytes`
  this task moves onto; `budget.rs` keeps a local of that name. These use the word for something
  else, so they stay; the search above is this item's. Report what the Measure reads.
- Every test the Steps run passes unchanged but for these lines, each count above zero.
- Both items' files and their rows are gone; `tools/citations/citations.py --all` reads 0;
  clippy is clean.

## Inputs

- `temper/docs/plan/debt-3/notes/common.md`.
- `jig/AGENTS.md`.
- `jig/docs/development/debt/D022-ui-limits-literals.md` and
  `jig/docs/development/debt/D023-view-tests-text-bytes.md`, whole.
- `skein/docs/design/ui.md`, 15 (`Limits::EMPTY`, `Node::text_bytes`);
  `skein/crates/skein-ui/src/limits.rs`; `skein/crates/skein-ui/src/tree.rs`, `Node::text_bytes`.

## Owns and may change

The literals (`rg -U -l '(skein_ui::)?Limits \{[^{}]*?scene_marks: 0' -g '*.rs' jig`), nothing else
of each file but the helper's uses below:

- production: `jig-client-view/src/sizes/limits.rs`, `src/tree/fragments/deployment.rs`,
  `src/tree/fragments/result.rs`;
- the view's tests: `src/tests/kit.rs`, and `src/tests/tree/fragments/` `authority.rs`,
  `amendment.rs`, `outcome.rs` (2), `person_task.rs`, `wake.rs`, `history.rs`, `policy_edit.rs`,
  `milestone.rs`, `chip.rs`;
- `jig/testing/jig-client-test-view/src/connector_values.rs`,
  `jig/testing/jig-client-test-kit/tests/suite/check.rs`,
  `jig/testing/jig-client-test-domain/src/tests/root/navigation.rs`;
- `jig/tests/client/src/observe.rs` (3), `src/model.rs`, `src/harness/world/words_tests.rs`,
  `src/observe/link_tests.rs`, `tests/suite/referee.rs`; `jig/tests/ops/src/joint/fixture.rs`.

The helper: `text_bytes` in `jig-client-view/src/tests.rs` goes, and each of its uses
(`rg -n 'tests::text_bytes' jig/crates/jig-client-view/src`) reads the method.

The items: both files (delete) and their rows in `jig/docs/development/debt/README.md`.

Nothing else changes. A `Limits` whose `scene_marks` is not a literal 0 (none in jig at base) and
every other `Limits` type (`PhraseLimits`, `view::Limits`, `client::Limits`) stay.

Pre-approved:

- **A field whose value is a literal `0` goes with the rest,** so a literal says only what it
  means; the base covers it (`Limits::EMPTY`, every field zero). A zero that carries a comment
  (`sizes/limits.rs`'s `patches: 0`) stays where it is, with its comment.
- **Computed fields stay as they are,** a value that happens to be zero at run time included,
  and the fields keep their order.
- The base is written as the literal names its type: `..skein_ui::Limits::EMPTY` beside
  `skein_ui::Limits {`, `..Limits::EMPTY` beside `Limits {`. No new import.
- **If a size test fails on `Node::text_bytes`,** the method also counts a description, a
  placeholder and an image address, which the helper did not. A node carrying those, read
  against a bound of the view's, means that bound or that test was short. Do not raise the bound
  or loosen the test: stop and ask, giving the test, the node and both counts. A test whose
  limits are built from the tree itself (`text_per_node: ...max()`) only tightens to the
  builder's own count, so it passes.

## Steps

1. D022. A literal becomes, for example:

   ```rust
   Builder::new(&skein_ui::Limits {
       nodes: nodes.checked_add(1).unwrap(),
       depth: 4,
       text_per_node: bytes,
       text_bytes: nodes.checked_add(1).unwrap().checked_mul(bytes).unwrap(),
       ..skein_ui::Limits::EMPTY
   })
   ```

   Run:
   - `heavy cargo nextest run -p jig-client-view`
   - `heavy cargo nextest run -p jig-client-world`
   - `heavy cargo nextest run -p jig-ops-world`
   - `heavy cargo nextest run -p jig-client-test-kit -p jig-client-test-domain -p jig-client-test-view`

   Commit: "jig's skein_ui::Limits literals sit on Limits::EMPTY (ui.md, 15)".
2. D023. `text_bytes(node)` reads `node.text_bytes()`, and `.map(crate::tests::text_bytes)`
   reads `.map(skein_ui::Node::text_bytes)`. The helper's definition in `tests.rs` goes.
   Run `heavy cargo nextest run -p jig-client-view`. Commit: "the view's tests read a node's
   bytes from skein-ui (ui.md, 15)".
3. `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `jig/`; `heavy cargo clippy -p
   jig-client-view -p jig-client-test-view -p jig-client-test-kit -p jig-client-test-domain -p
   jig-client-world -p jig-ops-world --all-targets -- -D warnings`.
4. Both items' files and rows; `python3 -I -B tools/citations/citations.py --base origin/main`
   and `--all` from the repository root; both acceptance searches. Commit: "pay jig D022 and
   D023: the items and their rows". Push.

## Report

As the implementer's text says, and also: what each acceptance search and each Measure read
after; the literals with a zero kept for its comment.
