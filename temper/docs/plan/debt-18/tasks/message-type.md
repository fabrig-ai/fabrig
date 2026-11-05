+++
id = "message-type"
part = "tasks-operation"
kind = "feature"
needs = ["amend-move"]
files = ["jig/crates/jig-domain-tasks/src/**", "jig/crates/jig-domain-core/src/routing/task.rs", "jig/crates/jig-domain-core/src/routing/sibling_routes.rs", "jig/crates/jig-domain-core/src/routing/run/calls.rs", "jig/crates/jig-domain-core/src/routing/effect/procedure.rs", "jig/docs/development/debt/D012-tasks-operation-vocabulary.md", "jig/docs/development/debt/D039-large-files.md"]
effort = "high"
minutes = 55
+++

# jig: a message to a task has one type, and D012 is paid

## Goal

The last of three steps of `jig/docs/development/debt/D012-tasks-operation-vocabulary.md`. `Message` becomes a public type
(`temper/docs/plan/debt-18/notes/contracts.md`, 2), `Operation`, `Event::Message` and `Event::AuthorizedMessage` carry it, and `ProcedureMessage`
goes. That pays the item, so it's deleted.

Acceptance:

- `rg -n 'struct ProcedureMessage' jig temper -g '*.rs'` finds nothing, and `Operation::Message` is a tuple variant. `Ask::ProcedureMessage` is
  a connector's own, and stays.
- A cancel, release, move, amend and message each have one type in `jig-domain-tasks`, as **Paid when** asks: your report lists each with
  where it's used.
- Every test passes, its assertions unchanged, the composed readers named below included.
- `authorized` stays as the contract lists it: by hand, make `Event::AuthorizedMessage` pass `false`, see a test go red, and restore.
- The item is deleted, and `jig/docs/development/debt/D039-large-files.md` no longer cites it (its two mentions; the file stays open).
- The branch's net lines rise by at most 40: this step removes only two fields per site, and a literal `Word` nests a level deeper.
  **The three steps together must fall.** Your report gives their sum from this plan's base: if it doesn't fall, stop and ask, with the figures.

## Inputs

- `temper/docs/plan/debt-18/notes/common.md`
- `temper/docs/plan/debt-18/notes/contracts.md`, 2: **what differs between entrances stays**
- `jig/docs/development/debt/D012-tasks-operation-vocabulary.md`
- `jig/docs/design/domain/people.md`, 5.1

## Owns and may change

- **Owns:** the files in `files`. `inbox::Message` gives way to the public type plus `project`, `number` and `authorized`.
- **Reader files, spelling only:** tests in `jig-domain-tasks` and `jig-domain-core`; `jig/tests/tasks/**`, including `src/world.rs`'s or-patterns; `jig/tests/core/**`.
- **`routing/task.rs`, about line 226,** may turn a message into `PersonQuestion`, and that stays. `Event::PersonQuestion` and its
  neighbours in the contract's list keep their flat fields. temper's `Event::Message` hits are smith's and the hub's, not this one.

## Steps

1. The type; `Operation`; `compound.rs`, `procedure.rs`, `inbox.rs`; `Event`; `domain.rs`; `routing/{task,sibling_routes}.rs`, `run/calls.rs`, `effect/procedure.rs`; the readers, by compiler.

2. Break the rule by hand, as the acceptance says. Delete the item, and fix D039.

3. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`; `heavy cargo clippy -p jig-domain-tasks -p jig-domain-core -p jig-tasks-world -p jig-core-world
   --all-targets -- -D warnings`; `heavy cargo nextest run -p jig-domain-tasks -p jig-domain-core -p jig-protocol-tools -p jig-tasks-world -p jig-core-world
   -p jig-test-service -p temper-engine-domain -p temper-engine-domain-world -p jig-conformance-world -p temper-conformance-world -p jig-one-process-world -p
   temper-one-process-world`: the last four go through the composed path, without an edit. `heavy cargo check --workspace --all-targets`. **This departs
   from the implementer's text** as `cancel-release` did, for the same reason. The citations check. Commit and push.

## Report

The common note's items, the three steps' net lines and their sum, production and tests apart, what you broke and what failed.
