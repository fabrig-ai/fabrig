+++
id = "constants"
part = "constants"
kind = "migration"
needs = ["uncap"]
files = ["hearth/crates/hearth-domain-build/src/**", "hearth/crates/hearth-domain-builder/src/**", "hearth/crates/hearth-domain-render/src/lib.rs", "hearth/crates/hearth-domain-store/src/lib.rs", "hearth/crates/hearth-domain-tree/src/**", "hearth/crates/hearth-protocol-builder/src/lib.rs", "hearth/tests/builder/src/protocol.rs", "hearth/tests/tree/**", "hearth/scripts/old-forms.txt"]
effort = "medium"
minutes = 25
+++

# The constants go

## Goal

Nothing in the service reads them now, so the domains' `MAX_OUT` and `hearth-protocol-builder`'s `MaxOut` and `MAX_OUT_*` go. The queues inside the domain's root that sit between its own children are uncapped. This finishes D001's first clause: no hearth step crate defines a `MaxOut` or a `MAX_OUT`.

Acceptance:

- `git grep -nE 'MaxOut|MAX_OUT|max_out' -- 'hearth/crates/*' 'hearth/tests/*'` prints nothing.
- `hearth/scripts/old-forms.txt`: `max_out 0` and `worst_case 0`, with `--write`.
- `Builder`'s `build_output`, `store_output` and `output` are `Queue::uncapped()`, and `store_forward` loops `0..builder.store_output.len()`.
- Every test of each package you changed and its direct dependents passes, unchanged but for the lines below: `hearth-domain-build`, `hearth-domain-builder`, `hearth-domain-render`, `hearth-domain-store`, `hearth-domain-tree`, `hearth-protocol-builder`, `hearth-service-builder`, `hearth-builder-world`, `hearth-tree-world`.

## Inputs

- `hearth/docs/plan/hearth-memory-6/notes/common.md`
- `hearth/docs/plan/hearth-memory-6/inputs/request.md`, 3
- `skein/docs/foundation/programming-model.md`, 6.3
- `hearth/docs/design/domain/render.md`, 8.4

## Owns and may change

Every use, from a survey of the trunk:

| Where | What's there | What it becomes |
|---|---|---|
| `hearth-domain-build/src/lib.rs` | `MAX_OUT` | goes |
| `hearth-domain-build/src/tests.rs`, twice | `Queue::with_capacity(crate::MAX_OUT)` | `Queue::with_capacity(1)` |
| `hearth-domain-builder/src/lib.rs` | `MAX_OUT`, `MAX_OUT_RESUME` and their comments | go |
| the same, `Builder::new` | `build_output`, `store_output`, `output` at 1, 2 and 2 | `Queue::uncapped()` |
| the same, `store_forward` | `0..store::MAX_OUT` | `0..builder.store_output.len()` |
| `hearth-domain-builder/src/tests/fixtures.rs` | `0..crate::store::MAX_OUT` | `0..builder.store_output.len()` |
| `hearth-domain-render/src/lib.rs` | `MAX_OUT`, `MAX_OUT_RESUME` | go |
| `hearth-domain-store/src/lib.rs` | `MAX_OUT` | goes |
| `hearth-domain-tree/src/lib.rs` | `MAX_OUT` and its comment | go |
| `hearth-domain-tree/src/tests/fake.rs`, `merge.rs`, `walk.rs` | `Queue::with_capacity(crate::MAX_OUT)` | `Queue::with_capacity(1)` |
| `hearth-protocol-builder/src/lib.rs` | `MaxOut`, `MAX_OUT_UP`, `MAX_OUT_DOWN`, `MAX_OUT_RESUME` | go |
| `hearth/tests/tree/src/helper.rs` and `tests/suite/memory.rs` | the import, and `Queue::with_capacity(MAX_OUT)` | the import goes; `Queue::with_capacity(1)` |
| `hearth/tests/builder/src/protocol.rs` | the world's four queues at `PROTOCOL_QUEUE`; `self.calls.room()`, `self.asked.room()` and `self.told.room()`; `self.answers.room() >= builder::MAX_OUT_RESUME.max(builder::MAX_OUT)` | `calls`, `answers`, `asked` and `told` are `Queue::uncapped()`, and those conditions go. `PROTOCOL_QUEUE` stays as `schedule_events`' pace |

- **A test driver's queue at 1 stays at 1,** as `hearth-domain-builder/src/tests.rs` already has it: a second output panics, and what the driver asserts stands.
- **`store_forward`'s new bound keeps the order.** A step made inside the loop forwards its own outputs before it returns, so items still leave first in, first out, and `store_one` takes no action on an empty queue.
- **`Layer` in `hearth-protocol-builder/src/tests/fixtures.rs`** stays at capacity 1.
- **No behaviour moves.** A test that fails is a finding (`notes/common.md`). A ceiling moved names its peak: `store_output` and `output` take a little more on their first push.
- **If another use has appeared** since the survey, change it in the same way, and name it in the report.

## Steps

1. Work down the table. `cargo fmt` from the root.
2. `python3 -I -B scripts/old_forms.py --write` from `hearth/`.
3. `heavy cargo clippy` naming every package you changed, `--all-targets -- -D warnings`.
4. The tests of the packages named above, and `heavy cargo check -p hearth -p hearth-conformance-world --tests`.
5. Commit and push. No new test: nothing here changes what the code does, and `orders` and the worlds hold it.

## Report

The count before and after, any use beyond the table, every ceiling moved, what ran and how many tests.
