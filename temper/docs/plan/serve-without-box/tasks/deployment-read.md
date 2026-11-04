+++
id = "deployment-read"
part = "fact"
kind = "feature"
needs = ["welcome-member", "deployment-wire", "init-box"]
files = ["temper/crates/temper-client-protocol-wire/schema/client-wire-v1.schema", "temper/crates/temper-client-protocol-wire/src/generated/v1.rs", "temper/crates/temper-client-protocol-wire/golden/**", "temper/crates/temper-client-protocol-wire/src/tests.rs", "temper/crates/temper-service/src/family.rs", "temper/crates/temper-service/src/clients.rs", "temper/crates/temper-service/src/system.rs", "temper/crates/temper-service/src/startup/checks.rs", "temper/crates/temper-service/src/tests/family.rs", "temper/crates/temper-service/src/tests/setup_github.rs", "temper/crates/temper-service/src/tests/welcome_policy.rs", "temper/crates/temper-service/src/clients/playground_tests.rs", "temper/crates/temper-engine-domain-forge/src/tests/playground.rs", "temper/docs/design/protocol/setup.status.md", "temper/docs/design/client/README.status.md"]
effort = "high"
minutes = 60
+++

# The engine answers the deployment's read, and offers the playground only with the welcome

## Goal

`DeploymentAnswer` gains `owned` (contracts K4) and `temper-service` answers the read from the
start's facts on every deployment. `StartPlayground` is *not offered* with no welcome.

Acceptance:

- The schema's record is K4's; generated code and goldens by the writer (`notes/common.md`).
- `Read::Deployment` answers a record, never empty, whether or not a playground or `demo` exists.
- `to_root` takes K4's `Deployment`. A test in `temper-service/src/tests/family.rs`,
  `the_deployment_read_answers_the_document`, runs the four values of `Deployment` and decodes
  each answer with `wire::decode_deployment_answer`.
- `welcome` is `settings::Engine.welcome`; `owned` is `machine.owned` present and the engine under
  the box (the value `under_box` takes in `system.rs`), by one small function a test covers at
  its four inputs, `deployment_owned_needs_the_document_and_the_box`.
- `playground_config(demo, welcome, deadline)` is `None` with no welcome, so that
  `temper-engine-domain-forge`'s `start` answers `NotOffered` at `playground.rs:87`. Tests:
  `playground_config_needs_the_welcome`, and in
  `temper-engine-domain-forge/src/tests/playground.rs`,
  `a_start_with_no_playground_config_is_not_offered`, which no test holds today.
- `heavy cargo nextest run -p temper-client-protocol-wire -p temper-codecs-world -p temper-service
  -p temper-engine-domain-forge`, then `-p temper --test suite`, `-p temper-playground-world` and
  `-p temper-one-process-world` pass: this changes what a shared item means, so its readers' tests
  run. `heavy cargo check -p temper-client-domain --tests`.
- `temper/docs/design/protocol/setup.status.md` loses "8.4: `StartPlayground` is offered wherever
  the document has `application.demo`, with a welcome or none."; in
  `temper/docs/design/client/README.status.md`, "4.6: the deployment's read, `deployment`: not in
  `temper-client-protocol-wire`'s schema, and neither served nor read." goes.

## Inputs

- `temper/docs/plan/serve-without-box/notes/common.md`; `notes/contracts.md`, K1, K2 and K4.
- `temper/docs/design/client/README.md`, 4.6 (The deployment's read).
- `temper/docs/design/protocol/setup.md`, 7.4 and 8.4 (Offered while no real forge is kept).
- `temper/docs/design/protocol/configuration.md`, 3 (`welcome`).

## Owns and may change

- The files of the header, for what acceptance names.
- `temper/crates/temper-service/src/family.rs`: the arm `deployment-wire` left, `w::Read::Deployment
  => None`, answers the record. 24 calls of `to_root` in tests gain the argument
  (`tests/family.rs`, `tests/setup_github.rs`); a local helper if it reads better, call sites only.
- `playground_config`'s callers: `system.rs:236`, `startup/checks.rs:668`,
  `tests/welcome_policy.rs:92`. `temper-engine-domain/src/domain.rs:333` derives
  `playground_provider` from the same config, harmless with no welcome: no edit.
- **Documents and fixtures that start a playground gain the welcome, one line each:**
  `welcome: true` beside `demo: Some(..)` in `temper-service/src/clients/playground_tests.rs` and
  its fixtures; `"welcome": true` in `temper/crates/temper/tests/support/configuration.rs`'s
  `demo_document`, `temper/tests/real-loop/src/fixtures.rs` and
  `temper/crates/temper-playground/src/documents.rs`, each only if a test there starts the
  playground. `init-box` gave the box's template, story A's document and the browser tier theirs.
- If `checks.rs:668` needs the document's `demo` checked with no welcome too, keep it checked:
  a deployment with `demo` and no welcome still has its provider to check. Pass `true` there and
  say so.
- The client's read is `client-reads-deployment`'s: no domain file here.
- Beside `debt-7`, whose hosts move `temper-service/src/clients.rs`, `system.rs` and
  `engine.rs`: keep to the lines K4 needs.

## Steps

1. The schema's field, the writer, `deployment_read_and_answer_round_trip` at both fields.
2. `Deployment` and its small function; `Clients` holds it; the arm answers it.

   ```rust
   w::Read::Deployment => {
       answer = Some(deployment_answer(deployment, limits));
       None
   }
   ```

3. `playground_config`, its callers; the fixtures that need the welcome.
4. The tests; the implementer's checks; commit and push.

## Report

As the implementer's text asks; list each fixture given the welcome.
