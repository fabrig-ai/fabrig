+++
id = "amend-move"
part = "tasks-operation"
kind = "feature"
needs = ["cancel-release"]
files = ["jig/crates/jig-domain-tasks/src/**", "jig/crates/jig-domain-core/src/routing/task.rs", "jig/crates/jig-domain-core/src/routing/party/controls.rs", "jig/crates/jig-domain-core/src/routing/party/decisions.rs", "jig/crates/jig-domain-core/src/routing/run/calls.rs", "jig/crates/jig-domain-core/src/routing/effect/procedure.rs"]
effort = "high"
minutes = 80
+++

# jig: an amend and a move of a task each have one type

## Goal

The second of three steps of `jig/docs/development/debt/D012-tasks-operation-vocabulary.md`. `Amend` and `Move` become public types
(`temper/docs/plan/debt-18/notes/contracts.md`, 2), `Operation` wraps them, `Event::Amend` and `Event::Move` carry them, and
`ProcedureAmendment` goes. No rule changes.

Acceptance:

- `rg -n 'ProcedureAmendment' jig temper -g '*.rs'` finds nothing, and `Operation::Amend` and `Operation::Move` are tuple variants.
- Every test passes, its assertions unchanged, `jig/tests/tasks/tests/suite/memory.rs` and `jig/tests/tasks/tests/suite/fuzzy/amendments.rs` included.
- `heavy cargo check --workspace --all-targets` compiles.
- By hand: `funder` is dropped on `Event::Amend`'s path (a test goes red), and restored.
- The branch's net lines rise by at most 10 (estimate: -40, give or take 40; rustfmt decides).

## Inputs

- `temper/docs/plan/debt-18/notes/common.md`
- `temper/docs/plan/debt-18/notes/contracts.md`, 2: **what differs between entrances stays**
- `jig/docs/development/debt/D012-tasks-operation-vocabulary.md`
- `jig/docs/design/domain/tasks.md`, 7

## Owns and may change

- **Owns:** the files in `files`. `control::Amend` gives way to the public type plus `places`, `funder` and the rest as arguments.
  `moving::Move<'a>` borrows its reason and may stay private, with a new name if it clashes.
- **Reader files, spelling only:** tests in `jig-domain-tasks` and `jig-domain-core`; `jig/tests/tasks/**`, including `src/world.rs`; `jig/tests/core/**`.
  temper reads neither. The comment at `proposals.rs`, about line 84, is reworded; `ProposalAction` doesn't change, since it's stored.
- **`src/world.rs`'s or-patterns** (about lines 487-603) bind fields across variants, and nesting breaks them. Rewrite the patterns, not
  what they read. The `unreachable!` re-matches in `routing/task.rs` stay as they are, at their new spelling.
- **The new `Box`:** `Amend.amendment` is boxed, one allocation on `Event::Amend`'s path, as `Operation::Amend` has today. If a memory
  ceiling trips, stop and ask, with the measurement: no budget moves.

## Steps

1. The types; `Operation`; `compound.rs`, `procedure.rs`; `Event`; `domain.rs`; `routing/task.rs` and the party and run routes; the readers, by compiler.

2. Break the rule by hand, as the acceptance says.

3. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-domain-tasks -p jig-domain-core -p jig-tasks-world -p jig-core-world
   --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-domain-tasks -p jig-domain-core -p jig-protocol-tools -p jig-tasks-world -p jig-core-world
   -p jig-test-service -p temper-engine-domain -p temper-engine-domain-world`; `heavy cargo check --workspace --all-targets`. **This departs from the
   implementer's text** as `cancel-release` did, for the same reason. The citations check. Commit and push.

## Report

The common note's items, the net lines of the step, what you broke and what failed.
