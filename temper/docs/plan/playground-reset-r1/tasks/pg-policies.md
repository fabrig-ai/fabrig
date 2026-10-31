+++
id = "pg-policies"
part = "playground"
kind = "feature"
needs = []
files = ["temper/crates/temper-service/src/stock.rs", "temper/crates/temper-service/src/startup.rs", "temper/crates/temper-service/src/startup/checks.rs", "temper/crates/temper-service/src/tests/stock.rs"]
effort = "high"
minutes = 70
+++

# The scripted model, the playground's alone: two stock policies

## Goal

The playground's project allows only the scripted model, and no other
project allows it (`temper/docs/design/draft/playground-reset.md`, 6.1,
"The scripted model, the playground's alone"). temper's stock gives one
policy, 1, whose ceiling allows every model. When the document names a
scripted provider (today `application.demo.llm.provider`), the stock
gives two policies: 1, every configured model but the scripted
provider's, and 2, the playground's, the scripted provider's models
alone. The template's seeded project 1, today the playground's, takes
policy 2 until a later task of this plan makes the playground's project
at its start.

Acceptance:

- `temper_service::stock::defaults_with(llm, scripted: Option<u32>) ->
  Defaults`: with `None`, exactly today's `defaults(llm)` (one policy,
  `Models::ALL`); `defaults(llm)` stays and calls it with `None`, so
  its many readers outside the crate are untouched.
- With `Some(provider)`: policy 1's `ceiling.models` and its role 0's
  `authority.models` are `Models::Named(mask)`, `mask` the bits of every
  model of `llm.models` (configuration's order, bit `i` model `i`) that
  is not one of that provider's `models`; policy 2 is policy 1's value
  with both `models` `Models::Named(mask of that provider's models)`,
  everything else alike (grants, spend, roles, permissions). The rules'
  ceiling stays `Models::ALL`. A provider number not in `llm.providers`
  is treated as `None` (startup refuses such a document elsewhere).
- `pub const PLAYGROUND_POLICY: u32 = 2;` in `stock.rs`, the number later
  tasks make the playground's project with.
- Startup passes the scripted provider: `engine_with` reads
  `application.demo.llm.provider` from the same document (a u32, or
  `None` when `application.demo` is null, absent, or the member is not a
  u32: the application's own parse reports that) and gives it to
  `Given::defaults`; `checks::stock` checks the floors of the same value,
  so `authority.projects` must hold 2 policies when a scripted provider is
  named (the small profile's `authority.projects` is 2: it fits).
- `Given::Own` is unchanged: a world that passes its own defaults gets
  them as given.
- The template's `engine.projects[0].policy` becomes 2
  (`temper/crates/temper-init/template/temper.json`, line 168) so the
  playground, which today chats in project 1, keeps its scripted model;
  `temper-init`'s test that pins it (`src/tests/documents.rs`, near line
  478, `"policy"`) says 2.

## Inputs

- `temper/docs/plan/playground-reset-r1/notes/common.md`.
- The draft, 6.1 ("The scripted model, the playground's alone") and 6.3
  ("A real account alone does not end it").
- `jig/docs/design/domain/authority.md`, sections 3 and 6 (models as a
  mask); `jig/crates/jig-domain-tasks/src/llm.rs`, `default_llm` (main is
  the first charter model within the authority's models: why the
  charters need no change).
- `temper/docs/design/protocol/configuration.md`, 13.4 (the stock).

## Owns and may change

- Owns `temper-service`'s `stock.rs`, `startup.rs` (`engine_with` and the
  `Given::defaults` call only), `startup/checks.rs` (`stock` only) and
  `tests/stock.rs`.
- Neighbour edits allowed: the template's one `"policy"` value and the
  one assertion in `temper-init/src/tests/documents.rs` that reads it.
  Another task of this plan may be changing other lines of the template
  at the same time; touch no other line of it.
- If `Given::defaults`'s new parameter forces call-site edits elsewhere
  in `temper-service` (`startup.rs`'s other callers), make them, passing
  the same pre-read provider or `None` where no document is at hand.
- If a reader outside `temper-service` calls `Given::defaults` (it is
  `pub(crate)` at the trunk's base), stop with a `Question:`.

## Steps

1. `stock.rs`: split `checked(llm)` into `checked(llm, scripted)`; build
   the masks from `llm.providers`' `models` names matched against
   `llm.models`' names (index = bit). Insert policy 2 only when
   `scripted` names a provider. Keep the `#[expect]`s as they are.
2. `startup.rs`: a small reader `fn scripted_provider(document) ->
   Option<u32>` that walks `application` → `demo` → `llm` → `provider`
   with the `jig_protocol_config::read` helpers already used there;
   nothing it finds is reported (the application's parse reports its
   problems).
3. Tests in `src/tests/stock.rs` (the crate's unit tests, on the
   `generated()` document already there): with the template's document,
   policy 1's masks exclude model `demo`'s bit and include
   `gpt-6-luna`'s, policy 2's are `demo`'s bit alone, the rules' ceiling
   is ALL, and `floors` passes at the template's limits; with
   `application.demo` null (`single_model()` or `without_projects()`),
   one policy, ALL, as today; `defaults(llm)` equals
   `defaults_with(llm, None)`.
4. A change of meaning: the stock's policy 1 now narrows models when a
   document names a playground. Run `heavy cargo nextest run -p
   temper-service`, `-p temper-init`, and the process tier's first run,
   `heavy cargo nextest run -p temper --test process -E
   'test(=a_first_run_answers_on_the_demo)'`, which still answers on the
   scripted model in project 1, now of policy 2.
5. `cargo fmt --all`; `heavy cargo clippy -p temper-service -p
   temper-init --all-targets -- -D warnings`; commit and push.

## Report

As the implementer text says; name the masks the template's document
gives, and any test outside `temper-service` that read policy 1's models.
