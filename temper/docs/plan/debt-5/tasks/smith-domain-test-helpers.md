+++
id = "smith-domain-test-helpers"
part = "smith-lifecycles"
kind = "feature"
needs = []
files = ["smith/crates/smith-local-domain/src/parent_tests.rs"]
effort = "medium"
minutes = 25
+++

# smith review, finding 2: the delivery tests share their setup

## Goal

`smith-local-domain/src/parent_tests.rs`'s delivery tests share three helpers, so each test shows only what differs. No assertion's meaning changes. No debt item closes.

Acceptance:

- `call_delivery`, `saved_intent` and `assert_store_failure` exist and serve the sites below.
- The `#[expect(clippy::too_many_lines, ..)]` near line 895 is gone.
- `heavy cargo nextest run -p smith-local-domain --lib` passes with the same test count as at base; `heavy cargo clippy -p smith-local-domain --all-targets -- -D warnings` passes.
- The file loses about 80 lines.

## Inputs

- `temper/docs/plan/debt-5/notes/common.md`, `temper/docs/plan/debt-5/notes/smith-review.md`, `AGENTS.md`, `smith/AGENTS.md`.
- `temper/docs/plan/debt-5/inputs/smith-lifecycles-review.md`, finding 2.

## Owns and may change

- Owns: `smith-local-domain/src/parent_tests.rs`, and nothing else.
- The `Called` block near line 476 uses its own `deadline`. It stays as it is, unless `call_delivery` can take the difference with no new parameter.

## Steps

1. `fn call_delivery(domain, env, out)`: the identical 24-line `Called` block, at its four sites (near lines 205, 900, 1011, 1056).
2. `fn saved_intent(out) -> Box<DeliveryRecord>`: the four sites near lines 230, 935, 1037, 1082.
3. `fn assert_store_failure(out, turn)`: the three six-line sites near lines 1041, 1091, 1137.
4. Remove the `too_many_lines` expectation: unfulfilled, it fails the build.
5. `cargo fmt`, the acceptance's commands; commit (`smith-local-domain's delivery tests share their setup`) and push.

## Report

The lines removed, and whether the `Called` block near line 476 joined.
