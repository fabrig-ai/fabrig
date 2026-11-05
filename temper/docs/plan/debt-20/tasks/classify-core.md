+++
id = "classify-core"
part = "classifiers"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-core/src/documents.rs", "jig/crates/jig-domain-core/src/acts.rs", "jig/crates/jig-domain-core/src/routing/view/routes.rs", "jig/crates/jig-domain-core/src/routing/view/marks.rs", "jig/crates/jig-domain-core/src/acts/objects/decisions.rs", "jig/crates/jig-domain-core/src/acts/objects/task.rs"]
effort = "high"
minutes = 80
+++

# jig: the core asks `Subject` and `Act` their questions

## Goal

Part of `jig/docs/development/debt/D015-exhaustive-lists.md`. Four files of the core hold 464 continuation lines, mostly the same partitions of `Subject` and `Act`. These become methods, and their callers match the answer.

| Type: question | Lists today (lines) | The method | Removes |
|---|---|---|---|
| `Subject`: which singled-out subject | `routes.rs` (8) (128) | `Subject::shape(&self) -> SubjectShape`, `{ Application { .. }, Run(u64), Retained(u64), Chips, Other }`; `Subject::chips(&self) -> Option<&List<u64>>`, in `documents.rs` | ~99 |
| `Subject`: does this change touch it | `marks.rs` (5) (79) | `Subject::touched_by(&self, touch: Touch) -> bool`; the `touch_*` functions stay as thin wrappers | ~68 |
| `Act`: which decision | `decisions.rs` (4) (84) | `Act::decision(self) -> Option<Decision>`, in `acts.rs` | ~63 |
| `Act`: re-narrowing | `task.rs` (21, one `unreachable!`) | split the arm, no classifier | ~21 |

Acceptance:

- `rg -c '^\s*\| ' jig/crates/jig-domain-core/src/routing/view/routes.rs jig/crates/jig-domain-core/src/routing/view/marks.rs jig/crates/jig-domain-core/src/acts/objects/decisions.rs jig/crates/jig-domain-core/src/acts/objects/task.rs | awk -F: '{s+=$2} END {print s}'` reads at most 250 (464 at base). The crate's own total moves with other plans, so this is the check.
- The note's table tests cover all of `Subject` for `shape` and `touched_by`, and all of `Act` for `decision`. They pass before and after.
- `task.rs`'s `unreachable!` re-narrowing is gone.
- `heavy cargo nextest run -p jig-domain-core --lib` and `heavy cargo nextest run -p jig-core-world --test suite` pass, with no assertion changed. `heavy cargo check -p jig-ops-domain -p temper-engine-domain --tests` compiles.
- By hand, per the note.
- The branch's net production lines fall by at least 100 (estimate: -200).

## Inputs

- `temper/docs/plan/debt-20/notes/common.md`
- `temper/docs/plan/debt-20/notes/classifiers.md`: the rule, pin first, the run
- `jig/docs/development/debt/D015-exhaustive-lists.md`
- `skein/docs/foundation/programming-model.md`, 10

## Owns and may change

- **Owns:** the files in `files`. A new enum's export is one line of `lib.rs`. Tests go in `src/tests/**`, in a new file if you like. The item stays: `classify-close` deletes it.
- **A second writer in this crate, by design:** `store-copy` and `store-remaps` work in `settings/`, and you stay out. Neither builds on the other.
- **Stay off** `reads.rs`, `reads/**`, `vocabulary.rs`, `stored.rs`, `effects.rs`, `routing/mod.rs`, `routing/effect/**`, `routing/run/**`, `runs/**`, `admission/**`, `documents/small.rs`, `documents/project_note.rs`, `documents/task_tree.rs`: other plans are changing them.
- **What stays, as routing:** `routing/stopping.rs`, `routing/flight.rs`, `routing/children.rs`, `acts/objects.rs`, `documents/offers.rs`, `routing/task/{done,refused}.rs`, and `routing/task/save.rs` (its five `tasks::Stored` lists are five different sets, each one question asked once).
- **All 51 `touch_*` call sites stay as they are,** through the wrappers: `tests/start.rs` names them.

## Steps

1. The table tests against today's code. Commit.
2. `Subject::shape` and `chips`; `touched_by`; `Act::decision`; `task.rs`'s split, a commit each.
3. By hand, per the note.
4. The note's run. Commit and push.

## Report

The note's items.
