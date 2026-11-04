+++
id = "temper-names"
part = "tidying"
kind = "feature"
needs = ["part-names"]
files = ["temper/crates/temper-service/src/worker.rs", "temper/crates/temper-service/src/accounts.rs", "temper/crates/temper-service/src/engine.rs", "temper/crates/temper-service/src/system.rs", "temper/crates/temper-service/src/forges.rs", "temper/crates/temper-service/src/forge/records.rs", "temper/crates/temper-service/src/forge/gone.rs", "temper/crates/temper-service/src/engine/diagnostics.rs", "temper/crates/temper-service/src/tests/**"]
effort = "medium"
minutes = 20
+++

# B7, temper: the accessor and the helper

## Goal

temper's half of `part-names`: `Hosted::mark_durable` at every site, and one helper for the four `SourceAnswered`
pushes in `worker.rs`. Behaviour stays unchanged.

Acceptance:

- `rg 'component\.mark_durable\(\)' temper/crates/temper-service/src` finds nothing. A site in a part that holds a
  `Hosted` by another name reads through it too; one that holds no `Hosted` keeps its call, and your report names it.
- `temper/crates/temper-service/src/worker.rs` has one helper for its four pushes, shaped as `part-names` shaped jig's.
- **Size cap:** net at most -6 lines.

## Inputs

- `temper/docs/plan/debt-10/notes/common.md`, `temper/docs/plan/debt-10/notes/contracts.md`, `AGENTS.md`, `temper/AGENTS.md`.
- jig's `jig/testing/jig-test-service/src/worker.rs`, the helper, on the trunk.

## Owns and may change

- Owns the files in the header, each edited at those lines alone. `debt-9/temper-wrong-kind` is changing
  `accounts.rs` and `engine.rs` beside this: if its work has landed, take it as it stands.

## Steps

1. The sites. The helper.
2. `heavy cargo nextest run -p temper-service --lib -E 'test(worker) | test(stop) | test(accounts)'`, then `heavy cargo
   check -p temper-service --tests`. fmt, clippy, old forms, citations. Commit, push.

## Report

As the implementer's text says, with the net lines.
