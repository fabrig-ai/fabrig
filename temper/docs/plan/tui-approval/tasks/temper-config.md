+++
id = "temper-config"
part = "temper-approval"
kind = "feature"
needs = ["config", "component", "http"]
files = ["temper/crates/temper-init/template/temper.json", "temper/crates/temper-init/template/bare.json", "temper/docs/examples/owned.json", "temper/crates/temper-init/src/tests/documents.rs", "temper/crates/temper-service/src/startup/members.rs", "temper/crates/temper-service/src/tests/approval.rs", "temper/crates/temper-service/src/tests.rs", "temper/crates/temper-service/src/tests/fixtures.rs", "temper/crates/temper-service/src/tests/checks.rs", "temper/docs/help/reference/configuration.md", "temper/docs/help/reference/configuration-engine.md"]
effort = "medium"
minutes = 60
+++

## Goal

temper enables a terminal's approval wherever it enables the printed link: `temper init`'s two documents, so every `temper box`'s, and the owned example set `"approval": true` beside `"link": true` in `engine.clients`, and temper's service hands it to jig's clients' component, so a box answers an ask with an approval (`temper/docs/design/protocol/configuration.md`, section 3, clients, *The printed link and a terminal's approval*; `temper/docs/design/protocol/setup.md`, 8.3, *Its sign-ins*; `temper/docs/plan/tui-approval/notes/contracts.md`, 4).

Acceptance:

- `temper/crates/temper-init/template/temper.json` and `bare.json`, and `temper/docs/examples/owned.json`, hold `"approval": true` right after `"link": true` in `engine.clients`; nothing else in them changes.
- A test in `temper-init`'s `documents.rs` holds that all three enable both, and failed before the documents changed.
- A test in `temper-service` holds that the service, given a document whose `engine.clients.approval` is `true`, answers `POST /jig/v1/approval/ask` with `asked` (a 32-byte secret, a code of `CODE_LETTERS` letters of `CODE_ALPHABET`, an expiry ten minutes past the wall), and with it absent or `false`, `refused`.
- A row holds that a clients' listener off loopback is refused at `application.clients.listener` with the approval enabled, as with the link.
- The clients' listener's member line names the approval beside the printed link, and the two configuration reference pages are regenerated.

## Inputs

- `temper/docs/design/protocol/configuration.md`, section 3, *`clients`* (the `listener` row and the bullet *The printed link and a terminal's approval*), and section 10's skeleton.
- `temper/docs/design/protocol/setup.md`, 8.3, *Its sign-ins*.
- `jig/docs/design/protocol/configuration.md`, section 3, the clients' row (`approval`).
- `temper/docs/plan/tui-approval/notes/contracts.md`, sections 2.1, 2.2, 3.2 (the ask's answers), 4 and 5 (binding).
- `temper/docs/plan/tui-approval/notes/common.md`.

## Owns and may change

- Owned: the three documents above; `temper/crates/temper-init/src/tests/documents.rs` (one test); `temper/crates/temper-service/src/startup/members.rs` (the `application-clients` `listener` row's `bound`, today `"socket address; loopback when printed links are enabled"`, to `"socket address; loopback when printed links or approvals are enabled"`, and nothing else); a new `temper/crates/temper-service/src/tests/approval.rs` with its `mod approval;` line in `src/tests.rs`; `src/tests/fixtures.rs`, for a start whose jig settings enable the approval (below); `src/tests/checks.rs`, one `Shape` and its row; the two generated pages `temper/docs/help/reference/configuration.md` and `configuration-engine.md`, regenerated, never edited by hand.
- Facts on the trunk, checked:
  - temper's service needs no code to pass the member on: `temper-service/src/clients.rs` builds the component's settings as `..config::clients::settings(config, ..)`, which the `config` task makes copy `Clients.approval` into `Settings.approval`.
  - temper refuses no member off loopback by itself: `startup::checks::standalone` (`temper-service/src/startup/checks.rs`, the `match clients.listener` after the standalone branch) refuses every clients' listener off loopback with `What::Loopback` at `application.clients.listener`, whichever of `link` and `approval` is on. That is the refusal contracts 4 asks of temper, and it already holds, so its row passes on arrival: commit it with the others and say so in the report. Add no second check.
  - `temper box` reads `temper init`'s document; it prints the link alone (`temper/crates/temper/src/box_command.rs`) and prints nothing for the approval. No box code changes.
- Readers, checked: `temper-init/src/tests/documents.rs` compares the generated documents with the owned example member by member, so changing all three together keeps it; `temper/crates/temper/src/tests/configuration.rs` renders the reference pages from init's document; other documents that set `"link": true` (`temper/crates/temper/tests/support/github/engine.json`, `tests/fixtures/cluster/engine.json`, `temper-playground`'s documents, `temper-service/src/tests/fixtures.rs`'s `DOCUMENT`) stay as they are: the approval absent is the approval off.
- Options:
  - The service test's start: if `fixtures.rs`'s `start_with_worker` can take the jig settings' `clients.approval` with a one-argument helper beside `started` (say `started_with_approval(approval: bool)`, setting `settings.jig.clients.approval` before the start), add that; if the start reads its settings elsewhere, change the nearest place and say so. Never change `DOCUMENT` for it.
  - If `configuration-engine.md` grows past `temper_docs::PAGE_BYTES` with the jig row, shorten the jig row's line in `jig-protocol-config`'s `members.rs` only if the `config` task left it longer than `link`'s; otherwise it's a question.

## Steps

1. **Tests first, red:**
   - `temper-init`, `documents.rs`: `the_documents_enable_the_printed_link_and_the_approval`, in the form of `the_box_runs_four_agents_and_keeps_eight_streams_and_four_calls_beside_them`: for `fixtures::OWNED`, `template::TEMPER` and `template::BARE`, `value["engine"]["clients"]["link"]` and `["approval"]` are both `true`. Red until step 2.
   - `temper-service`, `tests/approval.rs`: `an_ask_is_answered_as_the_document_enables_the_approval`, rows `(Option<bool>, expected)`: `Some(true)` asked, `None` and `Some(false)` refused. Each row starts the service (`fixtures::listening` as `server.rs`'s `a_post_under_jig_reaches_the_component_as_its_message` does), POSTs `/jig/v1/approval/ask` through `fixtures::http::Peer` with the encoded `w::ApprovalAsk` (empty), pumps, and decodes the body as `w::ApprovalAskAnswer`: for asked, the secret's length 32, `ApprovalCode::from_indexes` or `parse` of the code some, and `expires` ten minutes (600 s in nanoseconds) past the service's wall at the ask. It passes once `config`, `component` and `http` have landed, with no temper code: say so in the report; it is the proof the service passes the member on. The secret is never printed by the test, even in an assertion's message.
   - `temper-service`, `tests/checks.rs`: a row of `standalone_shapes`, a `Shape::ClientRemoteApproval` beside `Shape::ClientRemote` in `apply_shape` (the remote listener `10.0.0.1:7100`, and `engine.clients.approval = true`, `engine.clients.link = false`), expecting the same `What::Loopback` at `application.clients.listener`; the table's array length grows by one. Green on arrival (above).
   Commit them; the report says what the `documents.rs` test failed on.
2. Add `"approval": true` after `"link": true` in the three documents, keeping their indentation and order. Change the listener row's `bound` line in `members.rs`.
3. Regenerate the reference pages: `SKEIN_CODEGEN_WRITE=1 TEMPER_BUNDLE=none heavy cargo nextest run -p temper --lib -E 'test(=tests::configuration::the_pages_configuration_are_what_the_tables_render)'`, then the same without `SKEIN_CODEGEN_WRITE` to see it pass; read the diff: the listener's line, and `approval`'s row saying init writes `true`.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-init -p temper-service --all-targets -- -D warnings`; focused tests: `heavy cargo nextest run -p temper-init`, `TEMPER_BUNDLE=none heavy cargo nextest run -p temper-service`, `TEMPER_BUNDLE=none heavy cargo nextest run -p temper --lib`.
5. Commit and push.

## Report

As the implementer's text says, and: the fixture added for the approval's start; the red test and what it failed on; the two tests green on arrival and why; the reference pages' diff, one line each.
