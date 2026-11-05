+++
id = "decided-tool"
part = "decided-tool"
kind = "feature"
needs = ["away-forge"]
files = [
  "temper/crates/temper-engine-smith/src/calls.rs",
  "temper/crates/temper-engine-smith/src/answers.rs",
  "temper/crates/temper-engine-smith/src/tests/**",
  "temper/crates/temper-engine-domain/src/boundary.rs",
  "temper/crates/temper-engine-domain/src/translate.rs",
  "temper/crates/temper-engine-domain/src/store.rs",
  "temper/crates/temper-engine-domain/src/route/tools.rs",
  "temper/crates/temper-engine-domain/src/route/forge_route.rs",
  "temper/crates/temper-engine-domain/src/route/decided.rs",
  "temper/crates/temper-engine-domain-forge/src/boundary.rs",
  "temper/crates/temper-engine-domain-forge/src/domain.rs",
  "temper/crates/temper-engine-domain-forge/src/lib.rs",
  "temper/crates/temper-protocol-store/schema/store-v11.schema",
  "temper/crates/temper-protocol-store/src/generated/**",
  "temper/crates/temper-protocol-store/golden/**",
  "temper/tests/engine/domain/tests/suite/forge_routes/decided.rs",
  "temper/tests/engine/domain/tests/suite/forge_routes.rs",
]
effort = "high"
minutes = 90
+++

# The host tool `decided`

## Goal

A goal says what it would decide and hears, at once, whether it may: `temper/docs/design/domain/agent.md`, 4.3;
`temper/docs/design/domain/forge.md`, 5.9. The charter's words are `decided-charter`'s; this task adds the tool, and so moves the
host part's figures.

Acceptance:

- **Declared and decoded** in `temper-engine-smith/src/calls.rs`: name `decided`, schema `{ question, options, took, why, kind }`,
  `additionalProperties` false, all required, as `goal`'s spec is. Decoding refuses, in words, fewer than 2 or more than 6 options,
  `took` outside them, a `kind` outside the five, and a text past `forge::DECIDED_*`.
- **`Tool::Decided(forge::DecidedCall)`**, kind `ToolKind::Control`, so an advisor is answered beyond authority.
- **The caller's person,** in the root: a task with `tracked` some, whose requester is `Party::Person(p)`, or a task with
  `served_chat` some whose chat's requester is `Party::Person(p)`, and whose spec's words hold no line equal to `Asks: always`.
  Then `Work::Forge(forge::Event::Decided { .. })`, its project from `domain.core.tasks.delegation`. Any other caller is decided
  ask at once.
- **`forge::Request::Decided`** decides the call in the same decision: `CallAnswer::Decided { took }`, kept as the connector's named
  call, `forge::CallPayload::Decided { took }`, and in the store's `CallPayload`.
- **Rendered,** `temper-engine-smith/src/answers.rs`: "Decided: carry on with TOOK." or "Ask: your person is here or this kind waits
  for them. Ask with message, as a question."
- **Named and decided once:** a replay of the key answers what it answered, after the switch has moved, and writes no second row.
- The host part's figures, and each thing that lists host tools, move with the tool (below).
- `temper/docs/design/domain/agent.status.md` and `domain/forge.status.md` lose what this closes.

## Inputs

- `temper/docs/plan/ui-away/notes/common.md`, `notes/contracts.md`.
- `temper/docs/design/domain/agent.md`, 4.3; `temper/docs/design/domain/forge.md`, 5.9.
- The models: `goal` in `calls.rs` (decode) and `temper-engine-domain/src/route/goal.rs` (`row.tracked`); `forge::Request::Adopted`'s
  `reply_to`; `forge::Request::Read` and `read_answer` in `route/forge_route.rs`; `decide_call`; `keep_connector_answer` and
  `call_payload_answer` in `translate.rs`.

## Owns and may change

Owns, as `files` lists them. Status lines as above. Reader and neighbour files, each edited for the tool or for its figure alone:

- **`Tool` arms:** `temper-engine-domain/src/translate.rs` (`checked_call`, `call_needs_input`, `call_shape`, `tool_kind`,
  `is_core_read`), and `route/tools.rs`' `relay_payload`.
- **`CallAnswer` arms:** `src/store.rs` (`core_part`, `call_answer_bytes`), `translate.rs` (`keep_connector_answer`,
  `call_payload_answer`), `temper-engine-smith/src/answers.rs` (`answer`, and `tool_family`, whose wildcard won't force it).
- **`CallPayload` arms:** `temper-engine-domain-forge/src/lib.rs`' `row_bytes` (the words' length), `domain.rs`' `named_effect` and
  `named_outcome`, `temper/tests/codecs`.
- **The list of tools:** `temper-engine-smith/src/tests/reference.rs` (`TOOLS`, 41, and its order), `temper/docs/help/reference/host-tools.md`
  (its generator; add `decided`'s heading as that test asks), `temper/docs/help/task.md`, `chat.md`, `configure.md`, and
  `temper-docs/src/tests/rules.rs` if it pins them; shorten rather than raise a page's bound.
- **The figures,** every charter's row, since every charter declares it: `temper-service/src/tests/stock_brief.rs`,
  `src/limits.rs` (`brief_bytes`), `src/tests/limits.rs`, `temper/docs/design/protocol/configuration.md` (its 359,205),
  `configuration.status.md`. Fixtures measured by `temper/crates/temper/tests/suite/startup.rs`:
  `temper/crates/temper/tests/fixtures/cluster/engine.json`, `w1.json`, `w2.json`,
  `temper/crates/temper/tests/support/github/engine.json`, `w1.json`; `temper-init/template/{bare,temper}.json`,
  `temper/docs/examples/owned.json`, `temper/docs/help/reference/configuration{,-engine}.md` and
  `temper/tests/real-loop/tests/suite/template_bounds.rs` only if the turn's bound moves. Each takes what its test measures: a
  fixture's measure of a stock's size, not a budget.

Options, each pre-approved:

- **Where the route lives:** `route/decided.rs`, called from `relay_payload`'s arm, as `route/goal.rs` is.
- **`key`:** carried as a `NamedCall` by `named_call(key)` and back, or the event carries `CallKey`'s fields if the forge can't name
  it; say which.
- **The queue:** that `Work::Forge` and its `Request::Decided` drain in one decision is what `Work`'s loop in `route.rs` reads as. A test
  holds it (`decided_answers_in_its_decision`). If it doesn't hold, stop at the first `Request::Decided` and decide there, and say so.

## Steps

1. The store's `CallDecided` and regeneration, then `CallPayload::Decided` and `CallAnswer::Decided`.
2. `calls.rs` and `answers.rs`; `temper-engine-smith/src/tests`: `decided_decodes`, `decided_refuses_past_its_bounds`,
   `decided_refuses_a_took_outside_its_options`, `its_answers_read_as_the_design_says`.
3. `route/decided.rs`, the arms. In `temper/tests/engine/domain/tests/suite/forge_routes/decided.rs`, copying `goal_questions.rs`'
   `start`: `a_goal_decides_while_its_person_is_away`, `a_chats_goal_finds_the_chats_person`, `its_person_here_it_is_told_to_ask`,
   `a_kind_not_ticked_is_told_to_ask`, `other_is_told_to_ask`, `past_until_it_is_told_to_ask`,
   `a_replay_answers_the_same_after_the_switch_moved`, `a_chat_is_told_to_ask`, `asks_always_is_told_to_ask`,
   `an_advisor_is_beyond_authority`, `decided_answers_in_its_decision`. Set the switch by the forge's event directly: `away-route` may
   not have merged.
4. The figures. `heavy cargo nextest run -p temper-service -p temper-engine-smith -p temper-engine-domain`, and `-p temper
   --test suite -E 'test(startup)'` for the fixtures.
5. The check of the implementer's text, `temper-fake` and `temper-service` among the dependents; citations; push.

## Report

As `notes/common.md`, Your report: each figure, old and new.

## Rebase (2026-11-05, step merge, foreman ui-away/decided-tool/foreman-1)

rebase the three conflicting files, preserve both features, and regenerate Story A.
The merge failed on `temper/docs/design/domain/forge.status.md`, `temper/docs/help/task.md`, and `temper/tests/playground/tests/fixtures/story-a.log`.
In `forge.status.md`, remove the entire Away/Overturn/decided gap entry: `5d6561d3b7` closes the routes, and this branch closes the tool and named answer. Keep unrelated gaps.
In `task.md`, retain `433364d842`’s cancellation explanation and page condensation, then add `decided` beside `goal` in the host-tool list. Keep the page within its existing byte bound without losing facts.
Regenerate `story-a.log` from the combined code; do not select either recording or splice hashes. Integration’s `8181ff01e3` regenerated it for other combined changes.
Its generator is `temper/tests/playground/tests/suite/walk.rs::story_a_walks_from_the_welcome_to_the_csv_change_landed`, enabled by `SKEIN_CODEGEN_WRITE`.
Through heavy, run that test in package `temper-playground-world`, binary `suite`, with generation enabled; then run `story_a_kept_log_replays_natively` without generation. Report the regenerated fixture.
Run the focused `temper-docs` page checks and the spec’s relevant validation after resolving the conflicts.
