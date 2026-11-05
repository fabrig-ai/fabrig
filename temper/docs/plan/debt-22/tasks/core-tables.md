+++
id = "core-tables"
part = "spot-review"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-core/src/tests/classify_subjects.rs", "jig/crates/jig-domain-core/src/tests/acts/classifiers.rs", "jig/crates/jig-domain-core/src/settings/tests.rs"]
effort = "high"
minutes = 60
+++

# jig-domain-core: the subject table loses its derived columns, and its always-true assert bites

## Goal

Findings 1 (`Row`, the act table's `None` rows, `history_copy`'s test) and 2 (third bullet) of `temper/docs/plan/debt-22/inputs/debt20-review.md`. Net test lines fall by about 160.

Acceptance:

- `classify_subjects.rs`'s `Row` is `{ subject, shape, marks }`. `subject_partitions_preserve_watch_routing` reads the retained task, the application tuple and the chips from `row.shape` in one local `match`, and asserts all it does today.
- `assert!(refused.pop().is_some(), ...)` goes: the popped request is asserted to be `Request::Now` holding `Now::WatchRefused` with this `watcher` and `WatchRefusal::Busy`, and nothing is left after it. Dropping the `WatchRefused` push in `routing/view/routes.rs`'s `refused_subject` goes red; swapping its two pushes goes red.
- `every_act_keeps_its_decision_admission` (`tests/acts/classifiers.rs`): its nineteen identical rows `(act, None, no, no, no, no)` become one loop over a list of those acts, sharing one local check with the rows that differ. Every act it covers today stays covered. Making `crate::acts::escalation` answer `Ok` for `Act::Message` goes red.
- `history_copy_orders_held_and_history_with_the_revision_number` goes, after checking it's redundant: swapping `history_copy`'s two keys must go red in `jig-protocol-store`'s `tests/commits.rs`. If it doesn't, the test stays, and the report says why.
- `heavy cargo nextest run -p jig-domain-core --lib` passes.

## Inputs

- `temper/docs/plan/debt-22/notes/common.md`
- `temper/docs/plan/debt-22/inputs/debt20-review.md`, findings 1, 2
- `jig/crates/jig-domain-core/src/routing/view/routes.rs`, `refused_subject`, `retained_subject`, `application_subject`

## Owns and may change

- **Owns:** the files in the header, only the named tests in each.
- **Production:** none changes. If a derived column turns out not to follow from `shape`, keep that column for that row's kind and say so in the report.

## Steps

1. `Row`, then the assertion; the act table; the removed test.
2. Break each rule by hand as Acceptance lists, with `heavy cargo nextest run -p jig-protocol-store --lib` for `history_copy`; restore.
3. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-domain-core --all-targets -- -D warnings`; the tests above; the citations check. Commit and push.

## Report

The common note's items; each rule broken and what failed.
