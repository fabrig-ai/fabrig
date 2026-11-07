+++
id = "model-act"
part = "act"
kind = "interface"
needs = ["catalogue-core", "catalogue-clients"]
files = ["jig/crates/jig-client-domain/src/contract/settings.rs", "jig/crates/jig-client-domain/src/contract/llm.rs", "jig/crates/jig-client-domain/src/contract/me.rs", "jig/crates/jig-client-protocol-wire/schema/**", "jig/crates/jig-client-protocol-wire/src/generated/**", "jig/crates/jig-client-protocol-wire/golden/**", "jig/crates/jig-client-protocol-link/src/translate/**", "jig/crates/jig-protocol-engine-clients/src/translate/**", "jig/crates/jig-protocol-engine-clients/src/request.rs", "jig/crates/jig-domain-core/src/settings/catalogue.rs", "jig/crates/jig-domain-core/src/routing/settings/dispatch.rs", "jig/tests/core/tests/suite/model_act.rs"]
effort = "high"
minutes = 150
+++

# An owner turns a model on or off and adds one by name, and `me` says so

## Goal

`ChangeModel`, the one request both acts take, so that a client holds no model whole and a switch can't go stale; and what the page draws from `me`: a model's number, where it came from, whether its prices are estimated, and how many a project's chats left out. The core makes the new model, by `new_model`, which `model-catalogue-3` will call for a provider's list.

Acceptance:

- `turning_a_model_off_keeps_the_rest_of_its_value`: `Picker { on: false }` makes a revision whose model differs in `picker` alone, whichever revision the client saw, a task keeping its model, and `me` says `picker: false`. Turning it off twice is `empty`.
- `adding_a_model_by_name_makes_a_persons_model_on_the_picker`: `Add` takes the highest free number, its provider's dearest stock row's endpoint, dialect, program, window, output and prices, no efforts, `estimated: true`, `source: Person`, `picker: true`, and is in the offer of an account of that provider, and no other's.
- `a_name_out_of_shape_is_refused_as_shape`; `a_name_the_catalogue_holds_is_refused_as_shape`; `adding_a_withdrawn_name_brings_it_back`; `a_full_catalogue_refuses_as_count`; `a_provider_with_no_stock_takes_its_endpoint_and_the_dearest_figures`; `an_unknown_provider_is_a_reference`.
- `a_party_who_isnt_a_deployment_owner_is_not_theirs`.
- A `MeModel` carries `number`, `source`, `estimated`, and a `MeProject` `left_out`, across engine, wire and link, in `jig-protocol-engine-clients/src/tests/model_offers.rs`.
- `client-v19`'s schema changes in place, goldens regenerated; `heavy cargo check --workspace --tests` compiles; focused tests of each package touched pass.

## Inputs

- `jig/docs/design/client/domain/contract.md`, 3.2, *`ChangeModel`*, and 5.1.
- `jig/docs/design/domain/engine.md`, 17.11, *An owner's act*.
- `temper/docs/plan/model-catalogue-2/notes/contracts.md`, 3 and 4.
- `temper/docs/plan/model-catalogue-2/notes/common.md`.

## Owns and may change

Owns the header's files. Readers: `jig-domain-people/src/**` for the request's admission as `ChangeSettings`'; `jig/testing/jig-client-scripted-engine/src/codec/values/{settings,llm}.rs`, `jig/testing/jig-client-test-records/src/settings.rs`; `temper/tools/gallery/src/{worlds,samples}.rs`, `temper-client-tools/src/tests/fixtures/*.rs`, for a literal's new fields; `temper-client-view`'s tests' fixtures likewise.

Options, each pre-approved:

- `Add` is a change whose item is a `Set`: no new `Edit`, no new answer. Its `Outcome` is `SettingsChanged`.
- The refusal words stay `ChangeSettings`' (`jig-client-view/src/words/phrases/english.rs`).

## Steps

As `contracts.md`, 3 and 4. `new_model`'s dearest is by `runs::state::most_costly`, ties to the lowest number. Each test breaks its rule once by hand.

`cargo fmt`; each tree's `old_forms.py`; `heavy cargo clippy` on the changed packages `--all-targets -- -D warnings`; `heavy cargo nextest run` on them and their direct dependents. Commit and push.

## Report

What `me`'s size moved to on the wire, and which pins moved with it.

## Answers

2026-11-07 (temper/ui). Yes: run the committed recorder, temper-client-journeys again NAME, for chat-draft, project-landing and welcome-playground, through the build wrapper, then the three focused checks. temper/ui owns the journeys and rules: that tool regenerates what the gate's focused replay checks read, so it's generation and neither a walk nor a render check, even though it drives a browser. The user's rule stands for anything else: no throwaway driver, walk or comparison. Retry on 'a key mark caught a moving page', at most three attempts in all; another refusal is a new question. Read the print diff for what you'd expect from this wire change.
