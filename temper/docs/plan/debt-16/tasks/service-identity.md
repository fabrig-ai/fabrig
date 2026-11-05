+++
id = "service-identity"
part = "temper-shared"
kind = "feature"
needs = []
files = ["temper/crates/temper-service/src/assignment.rs"]
effort = "medium"
minutes = 15
+++

# temper: the service's run identity is jig's

## Goal

`temper-service/src/assignment.rs` hashes a run's identity by hand, in bytes `jig_protocol_charter::run_identity` already
makes. It's a D004 copy. The bytes stay, and a test now says so: nothing in temper pins them.

Acceptance:

- `rg -n 'fn identity' temper/crates/temper-service/src/assignment.rs` finds nothing.
- `a_run_identity_keeps_its_bytes` passes, and pins bytes computed by the old function.

## Inputs

- `temper/docs/plan/debt-16/notes/common.md`
- `jig/docs/development/debt/D004-ops-root-jig-wiring.md`

## Owns and may change

- **Owns:** `temper-service/src/assignment.rs`.
- **Stays, as temper's own:** `messages`, the charter, the items, `text_tail`, and `answered`: `jig/docs/design/domain/hosts.md`,
  12 keeps that open, with "it keeps its own" as its default.
- `jig-protocol-charter` is already a dependency. Drop `skein_io::digest` only if nothing else in the crate needs it.

## Steps

1. Before the change, add `a_run_identity_keeps_its_bytes` in this file's `tests` module, with a fixed deployment, task and
   conversation, and the old function's output as a literal. Run it.
2. Replace the call with `jig_protocol_charter::run_identity(&encoding.deployment, assignment.task,
   assignment.conversation)` and delete `identity`. If the bytes differ, stop and ask: the identity leaves this process.
3. Break by hand: swap task and conversation at the call site; see the test fail; restore.
4. `heavy cargo nextest run -p temper-service --lib`.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-service --all-targets -- -D
   warnings`; the citations check. Commit and push.

## Report

The common note's items, with the break above.
