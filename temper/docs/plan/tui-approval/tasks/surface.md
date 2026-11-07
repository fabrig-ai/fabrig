+++
id = "surface"
part = "jig-approval"
kind = "interface"
needs = ["wire"]
files = ["jig/crates/jig-protocol-engine-clients/src/boundary.rs", "jig/crates/jig-protocol-engine-clients/src/routes.rs", "jig/crates/jig-protocol-engine-clients/src/component.rs", "jig/crates/jig-protocol-engine-clients/src/watches.rs", "jig/crates/jig-protocol-engine-clients/src/tests/limits.rs", "jig/crates/jig-protocol-engine-clients/src/tests/approval_surface.rs"]
effort = "high"
minutes = 75
+++

## Goal

The clients' component's surface for a sign-in approved by a browser, small and whole, so that the engine's body (`component`), the HTTP mapping (`http`), configuration (`config`) and the client (`client`) all build on it at once: the four messages, the exchange's `browser`, `Settings.approval`, and the four routes answered as where approval is not enabled. Its readers' compiler-forced fields go with it, in every tree.

Acceptance:

- `Message::{ApprovalAsk, ApprovalRead, ApprovalDecide, SignInApproval}(Box<[u8]>)`, `FromTransport::Exchange { exchange, credential, browser, message }` and `Settings { .., approval: bool }` exist, as `temper/docs/plan/tui-approval/notes/contracts.md`, 3.1 says.
- Each new message reaches a route of its own and is answered with its record of 2.1 as where approval isn't enabled (3.2): an ask `refused`; a read, a decision and a take `unknown`; while the engine stops, each `not_ready`. This holds whatever `Settings.approval` says until the `component` task builds the approval.
- Every literal `Settings` in the workspace has `approval: false`; every construction of `FromTransport::Exchange` has `browser: false`, every pattern on it `browser` or `..`. The workspace builds, and every package you touched passes its focused tests.

## Inputs

- `temper/docs/plan/tui-approval/notes/contracts.md`, sections 1, 2.1 and 3 (binding).
- `temper/docs/plan/tui-approval/notes/common.md`.
- `jig/docs/design/protocol/client.md`, 4.2 and section 8, *A sign-in approved by a browser*.

## Owns and may change

- Owned, in `jig/crates/jig-protocol-engine-clients/src/`: `boundary.rs` (the variants, the field, `Settings.approval` with a doc line), `routes.rs` (`Route`'s four new variants, `message`, `stopped`, and the answers' encoders: `approval_ask_answer(w::ApprovalAskAnswer)`, `approval_state(w::ApprovalState)`, `approval_decided(w::ApprovalDecided)`, `approval_sign_in(w::ApprovalSignInAnswer)`, each `-> Box<[u8]>`, beside `signed_in`), `component.rs` (the field taken in `transport`, the four routes answered in `body` before any credential lookup, and their arms in every match on `Route`), `watches.rs` (the arms), `tests/limits.rs` (the four messages as rows of `MESSAGES`), a new `tests/approval_surface.rs` and its `mod` line in `tests.rs`.
- Readers, each edited only for the compiler: `approval: false` in every literal `Settings` (today `jig/crates/jig-protocol-config/src/clients.rs:119`, `jig/crates/jig-protocol-engine-clients/src/tests/fixtures.rs:41`, `jig/testing/jig-client-kit/src/joint/component.rs:67`, `jig/testing/jig-client-protocol-kit/src/fixtures.rs:65`, `jig/testing/jig-engine-clients-kit/src/fixtures.rs:31` and `:231` (there, `settings.approval`), `temper/crates/temper-playground/src/documents.rs:530`, `temper/crates/temper-service/src/clients.rs:213`, `temper/crates/temper-service/src/tests/clients_unwatch.rs:54`, `temper/crates/temper-service/src/tests/fixtures.rs:494`, `temper/crates/temper/tests/support/owned.rs:975` and `:1048`, `temper/tests/client/src/component.rs:69`, `temper/tests/one-process/src/fixtures.rs:109`; `rg 'link_secret:'` finds them); `browser: false` in every `FromTransport::Exchange` built (`jig/crates/jig-protocol-engine-clients-http/src/call.rs`, `jig/testing/jig-client-kit/src/joint/component.rs`, `temper/tests/client/src/component.rs`, the component's own `tests/fixtures.rs` and `tests/closing.rs`), and `..` or `browser: _` in every pattern (`jig/crates/jig-protocol-engine-clients-http/src/tests.rs` and `tests/public.rs`, `jig/tests/engine-clients/tests/suite/provider.rs` and `public_http.rs`, `temper/crates/temper-service/src/tests/clients_unwatch.rs`; `rg 'FromTransport::Exchange'` finds them).
- Not yours: the approval itself (`component`), the HTTP routes and the cookie's `browser: true` (`http`), the configuration member (`config`), the client (`client`). Don't add an `approval.rs` module: `component` does.
- Options:
  - If a reader builds `Settings` by a helper rather than a literal, leave it; the compiler says which need the field.
  - If `routes::stopped` would answer a new route `None` by its shape, answer `not_ready` as the link's sign-in does instead: contracts 3.2.

## Steps

1. Add the variants, the field and `Settings.approval` (`/// Whether the component holds sign-ins approved by a browser (protocol/client.md, 8).`). The `browser` field's doc: `/// Whether the transport carried the credential itself, as a browser's cookie (protocol/client.md, 17.3).`
2. In `routes.rs`, `Route::{ApprovalAsk, ApprovalRead, ApprovalDecide, ApprovalTake}`, `message` mapping each new `Message`, `stopped` answering each `not_ready` in its own record, and the four encoders. In `component.rs`, `transport` takes `browser` and passes it to `receive` (store it on the exchange's admission if that's the natural place; `component` reads it), and `body` answers the four routes at once, before the credential lookup, as the not-enabled answers above. Every other `match` on `Route` lists them.
3. **Tests first, red, then the code:** `tests/approval_surface.rs`, one table, `each_approval_route_is_answered_as_not_enabled`, rows `(&str, fn(Box<[u8]>) -> Message, Box<[u8]>, Box<[u8]>)`: the message, a well-formed record of its kind (an empty `ApprovalAsk`; `ApprovalRead` and `ApprovalDecide` with code `*b"WDJBMJHT"`; `ApprovalSignIn` with 32 bytes of 7), and its expected answer's bytes; with `Settings { approval: false, .. }` and with `approval: true`, each answered as not enabled, nothing on `out` (no core event, no secrets lookup), no pending exchange. A second table, `each_approval_route_is_not_ready_while_stopping`, rows as the first, after the component's stop as `tests/closing.rs` brings it about. Commit the variants and the tests first, the four routes left unanswered in `body`, so the tables fail on the missing answer; then commit the answers. The report says what each failed on.
4. The readers: let the compiler list them (`heavy cargo check --workspace --tests` once is allowed for this, then each package by name), and edit each only as above.
5. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py` and `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p jig-protocol-engine-clients -p jig-protocol-engine-clients-http -p jig-protocol-config -p jig-engine-clients-kit -p jig-client-kit -p jig-client-protocol-kit -p temper-service -p temper-playground -p temper-client-world --all-targets -- -D warnings` (add any other package you touched); focused tests: `heavy cargo nextest run -p jig-protocol-engine-clients`, `-p jig-protocol-engine-clients-http`, `-p jig-engine-clients-world`, `-p jig-protocol-config`, `-p temper-service`, `-p temper-client-world`; `heavy cargo check -p temper -p temper-one-process-world --tests` for the rest.
6. Commit and push.

## Report

As the implementer's text says, and: each reader file edited, one line each; where `browser` is kept on an admitted exchange; what the red tests failed on.
