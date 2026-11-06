+++
id = "offer-surface"
part = "offer-surface"
kind = "interface"
needs = ["effort-config"]
files = [
  "jig/crates/jig-client-protocol-wire/schema/**",
  "jig/crates/jig-client-protocol-wire/src/generated/**",
  "jig/crates/jig-client-protocol-wire/golden/**",
  "jig/crates/jig-client-domain/src/contract/llm.rs",
  "jig/crates/jig-client-domain/src/contract/me.rs",
  "jig/crates/jig-client-protocol-link/src/translate/llm.rs",
  "jig/crates/jig-client-protocol-link/src/translate/documents/frame.rs",
  "jig/crates/jig-client-protocol-link/src/tests/llm_accounts.rs",
  "jig/crates/jig-protocol-engine-clients/src/translate/llm.rs",
  "jig/crates/jig-protocol-engine-clients/src/translate/owner.rs",
  "jig/crates/jig-protocol-config/src/llm.rs",
  "jig/crates/jig-protocol-config/src/clients.rs",
  "jig/crates/jig-protocol-config/src/tests/llm.rs",
  "jig/crates/jig-client/src/shapes.rs",
  "jig/crates/jig-client/src/tests/shapes.rs",
  "jig/testing/jig-client-scripted-engine/src/codec/values/llm.rs",
  "jig/testing/jig-client-scripted-engine/src/codec/values/me.rs",
  "jig/tests/client-wire/tests/fuzzy_client_wire/documents.rs",
]
effort = "high"
minutes = 110
+++

# `me`'s surface for the offer, with nothing yet filling it

## Goal

Later tasks fill the offer in the core and draw it in the clients, each on its own. This task only carries the
shapes: a model's `shown` and `picker` in configuration; `MeModel`'s `effort`, `shown`, `accounts` and `picker`;
`MeProject`'s `defaults`. **No behaviour changes:** `models` still lists what the party's authority allows,
`accounts` and `defaults` go empty.

Acceptance:

1. `engine.llm.models[].shown` and `picker` parse as the contracts note says; `shown` empty is `What::Form`, over 256
   bytes `What::Bound` (whatever `description`'s check reports for the same faults).
2. A `MeModel` and a `MeDefault` cross the engine's translation, the wire and the link whole, `None` and empty
   included (`tests/llm_accounts.rs`, beside `me_with_model`). The link refuses as malformed: an `effort` not among
   `efforts`, an `effort` where `efforts` is empty, none where it isn't, and an empty `shown`.
3. The engine's `me` fills `effort` by `Model::default_effort`, `shown` and `picker` from configuration.
4. `jig_client::shapes::resolved_default` finds a project's template's preview, and `shapes::chat`'s `Create` still
   carries the template's own `llm`, with a `defaults` row present (`tests/shapes.rs`).
5. `check_settings` refuses `accounts.accounts` past `me_model_accounts` and `core.templates` past
   `me_project_defaults`, as a row of its table.

## Inputs

- `temper/docs/plan/model-catalogue-1/notes/common.md`, `temper/docs/plan/model-catalogue-1/notes/contracts.md`
  (*A configured model*, *`me`*).
- `jig/docs/design/client/domain/contract.md`, 5.1; `jig/docs/design/protocol/client.md`, 3 (the paragraph naming
  `resolved_default`); `jig/docs/design/protocol/configuration.md`, 3.

## Owns and may change

- Owns the files in its `files`. The schema changes in place at v19 (`jig/docs/design/protocol/client.md`, section 9).
- **The required field's other lines, in any tree, each edited only for the field:**
  - `c::MeModel` literals: `temper/crates/temper-client-view/src/tests/{choice,home,fixtures}.rs`,
    `temper-client-domain/src/tests/pages/{choice,chats}.rs`, `temper-client-tools/src/tests/fixtures/choices.rs`,
    `temper-client-tools/src/tests/jig/accounts.rs`, `temper-client-shell-terminal/src/tests/fixtures/chat.rs`,
    `temper/tools/gallery/src/worlds.rs`. Default values: `effort` the first of `efforts`, `shown: None`,
    `accounts` one account where the fixture has one, or empty, `picker: true`.
  - `MeProject` literals, `defaults` empty: `jig/testing/jig-client-test-records/src/owner.rs`,
    `jig/tests/client-wire/tests/suite/set_visibility.rs`, `jig/tests/ops/tests/suite/client_stories/orders.rs`,
    `temper-client-tools/src/tests/fixtures.rs`, `temper-client-view/src/tests/fixtures.rs`,
    `temper-client-view/tests/suite/{task_pages,change_page}.rs`, `temper-client-domain/src/tests/fixtures/chats.rs`,
    `temper-client-domain/tests/suite/task_pages.rs`, `temper-client-shell-cli/src/tests/fixtures/recovery.rs`.
  - `jig_protocol_engine_clients::Model` literals: `jig/testing/jig-client-protocol-kit/src/fixtures.rs`,
    `jig/testing/jig-client-kit/src/joint/component.rs`, `jig/testing/jig-engine-clients-kit/src/fixtures/receiving.rs`,
    `jig/crates/jig-protocol-engine-clients/src/tests/fixtures.rs`, `jig/tests/engine-clients/tests/suite/requests.rs`,
    `temper-client-tools/src/tests/fixtures/task_text.rs`, `temper/crates/temper/src/tests/settings_fields.rs`,
    `temper/tests/client/src/component.rs`, `temper/tests/engine/domain/tests/suite/root_routes/open_chat.rs`,
    `temper/tests/engine/domain/src/tier_task_show.rs`.
  - The compiler finds any this list misses: the same edit is pre-approved.
- **Don't put the preview on `Template`:** `w::Template` and `c::Template` are also a setting's value, so a field
  there would move every settings path and golden. `crate::Template` goes through the store's codec.
- `temper/docs/design` isn't yours. Removes `jig/docs/design/protocol/configuration.status.md`'s **Not yet** 3 about
  `shown` and `picker`, and from `jig/docs/design/client/domain/testing.status.md`'s 4 the shapes test this writes. The
  contract's status entry closes with `offer-engine`.

## Steps

1. The schema: `MeModel`, `MeDefault` and `MeProject`, in the contracts note's order. Regenerate, never by hand:
   `SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p jig-client-wire-world -p jig-store-codecs-world -E 'test(drift::)'`.
   It rewrites `generated/v19.rs` and the goldens, and removes stale ones. Add a `check!` for `MeDefault` in
   `jig/tests/client-wire/tests/fuzzy_client_wire/documents.rs`.
2. `contract/llm.rs`, `contract/me.rs`; the link's `translate/llm.rs`, its frame.
3. `jig-protocol-config`: `ParsedModel`, `Llm.shown`, `Llm.pickers`; `clients.rs` fills `Model`. The engine's
   `translate/llm.rs`: `models` fills `effort`, `shown` and `picker`, with `accounts` empty; `owner.rs`'s project row
   has `defaults` empty. `fits` bounds them. `offer-engine` fills these two from the core.
4. `shapes::resolved_default`. The tests above. Break each rule once to see it bite.
5. Narrow tests, then the implementer's text's closing checks over the packages you changed, with
   `heavy cargo check --tests` naming `-p temper-client-view -p temper-client-domain -p temper-client-tools
   -p temper-client-shell-terminal -p temper-client-shell-cli -p temper-client-world -p temper-engine-domain-world
   -p temper -p temper-client-gallery`. Commit and push.

## Report

As the implementer's text says; the goldens that moved, and any literal beyond this list.
