+++
id = "dead-api"
part = "dead-api"
kind = "feature"
needs = []
files = ["skein/crates/skein-oauth/src/record.rs", "skein/crates/skein-oauth/src/lib.rs", "skein/crates/skein-fake-llm-protocol/src/provider.rs", "skein/crates/skein-llm-connection/src/component.rs", "skein/crates/skein-llm-connection/src/tests.rs", "skein/crates/skein-llm/src/openai.rs", "skein/crates/skein-llm/src/openai/response.rs", "skein/crates/skein-llm/src/openai/tests.rs", "skein/crates/skein-llm/README.md", "skein/crates/skein-forge/src/forgejo/client.rs", "skein/crates/skein-forge/src/forgejo/client/delivery.rs", "skein/crates/skein-forge/src/forgejo/server.rs", "skein/crates/skein-forge/src/forgejo/server/writes.rs", "skein/crates/skein-forge/src/forgejo/tests/fixtures.rs", "skein/crates/skein-ui-web/src/timer.rs", "skein/crates/skein-ui-web/src/tests/history.rs", "skein/docs/development/debt/D040-dead-public-api.md"]
effort = "medium"
minutes = 25
+++

# Delete skein's public items with no caller, or only a test's

## Goal

Pay skein's D040: each item below goes, or its one test folds it in. The removals are agreed: no tree calls them from shipping code.

Acceptance:

- `rg -n '\b(RECORD_VERSION|RFC3339_BYTES|head_bound|admit_with_reasoning_drop|encode_completion|body_first)\b' skein smith jig temper hearth -g '!**/docs/development/**' -g '!**/docs/plan/**'` finds nothing.
- `client::first`, `client::delivery::first` and `server::body_first` are gone from `skein-forge/src/forgejo`, and skein-ui-web's `timer.rs` defines no pure `Timer`; `timer::web::Timer` stays as it is.
- Each test that remains asserts what it asserted.
- `skein/docs/development/debt/D040-dead-public-api.md` is deleted.

## Inputs

- `skein/docs/development/debt/D040-dead-public-api.md`, whole.
- `skein/docs/design/ui-web.md`, 5.1, for the web timer that stays.
- `temper/docs/plan/debt-27/notes/common.md`.

## Owns and may change

Owns the files in the header. The cut's survey found no other reader in any tree; `temper/docs/design/client/tools.md`'s one mention of `RECORD_VERSION` is already reworded on the trunk. Already paid: `forgejo::time`, with its `RFC3339_BYTES`, has gone; the search above just checks it.

Options, each pre-approved:

- `admit_policy` may become `pub(crate)` for the test; if its parameter's other values read better through a small test helper, that helper lives in `tests.rs`.
- If a removal leaves an import or a `use` unused, remove that too.
- If a removed item turns out to have a caller the survey missed, keep the item, and report it under `## Left`.

## Steps

- **`skein-oauth`:** delete `RECORD_VERSION` and its export in `lib.rs`.
- **`skein-fake-llm-protocol`:** delete `provider::head_bound` and its two doc lines.
- **`skein-llm-connection`:** delete `Component::admit_with_reasoning_drop`. `tests.rs`'s one call becomes `pool.admit_policy(Token::new(7), 0, input, Some(value), Some(enabled))`. Delete `skein-llm/README.md`'s sentence naming it.
- **`skein-llm`:** delete `openai::encode_completion` and its re-export in `openai.rs`. `fake_completion_echo_exercises_large_event_without_keeping_request` calls `encode_peer_event(&Event::Completed { stop: Stop::EndTurn, usage: Usage::NONE }, Some(&request()), Echo::REQUEST, &LIMITS)`, as the deleted function did; its doc line says the decoder ignores the echo, so keep that as a comment on the test if the test doesn't already say it.
- **`skein-forge`:** delete the three functions and `server.rs`'s re-export of `body_first`. `forgejo/tests/fixtures.rs`'s three uses start from `tokenizer::Request::Next`.
- **`skein-ui-web`:** `timer.rs` keeps its first comment and `#[cfg(target_arch = "wasm32")] pub mod web;`. Delete `the_timer_reports_its_deadline_once` from `tests/history.rs`.

Then `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-oauth -p skein-fake-llm-protocol -p skein-llm-connection -p skein-llm -p skein-forge -p skein-ui-web --all-targets -- -D warnings`; `heavy cargo nextest run` on those packages; and `heavy cargo check --target wasm32-unknown-unknown -p skein-ui-web`, since `timer/web.rs` builds only there. Commit, push.

## Report

As your text says: the net lines, and any item kept.
