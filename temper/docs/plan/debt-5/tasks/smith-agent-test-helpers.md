+++
id = "smith-agent-test-helpers"
part = "smith-lifecycles"
kind = "feature"
needs = []
files = [
  "smith/crates/smith-agent-service/src/service/tests.rs",
  "smith/crates/smith-agent-service/src/service/tests/lifecycles.rs",
  "smith/crates/smith-agent-service/src/service/tests/setup.rs",
]
effort = "medium"
minutes = 20
+++

# smith review, finding 2: the service's lifecycle tests share `decoded_start` and `assert_cancel`

## Goal

`tests.rs` and `tests/lifecycles.rs` share one `decoded_start` and one `assert_cancel`. No test's meaning changes. No debt item closes.

Acceptance:

- `rg -n 'fn decoded_start' smith/crates/smith-agent-service/src` reads one definition, in `service/tests/setup.rs`.
- `assert_cancel` serves the five sites below.
- `heavy cargo nextest run -p smith-agent-service --lib -E 'test(service::tests::)'` passes, with the same count as at base.
- `heavy cargo clippy -p smith-agent-service --all-targets -- -D warnings` passes.
- The two test files lose about 20 lines between them.

## Inputs

- `temper/docs/plan/debt-5/notes/common.md`, `temper/docs/plan/debt-5/notes/smith-review.md`, `AGENTS.md`, `smith/AGENTS.md`.
- `temper/docs/plan/debt-5/inputs/smith-lifecycles-review.md`, finding 2.

## Owns and may change

- Owns: the three test files in the header.
- The five sites lose their own panic messages; the helper's message names the run.

## Steps

1. `fn decoded_start(mounts: Box<[Mount]>) -> Box<DecodedStart>` in `service/tests/setup.rs`, replacing the two near `tests.rs:532` and `tests/lifecycles.rs:131`.
2. `fn assert_cancel(events, run)` beside it, for `tests.rs` near 179 and 189, and `tests/lifecycles.rs` near 76, 96 and 112.
3. `cargo fmt`, the acceptance's commands; commit (`smith-agent-service's tests share decoded_start and assert_cancel`) and push.

## Report

The lines removed.
