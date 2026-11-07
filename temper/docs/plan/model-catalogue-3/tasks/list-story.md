+++
id = "list-story"
part = "on"
kind = "feature"
needs = ["list-on", "models-read"]
files = ["temper/tests/client/tests/suite/models.rs", "temper/tests/client/src/**", "temper/tests/one-process/tests/suite/one_process.rs", "temper/tests/one-process/src/**"]
effort = "high"
minutes = 110
+++

# A person watches a new model arrive

## Goal

The request's story through temper's real roots, at the two levels that hold it cheaply: the client world, with typed answers, and one row of the one-process world, with the real worker, agent and fake provider. No new browser test.

Acceptance:

- In `temper/tests/client/tests/suite/models.rs`, beside `a_model_added_by_name_is_on_the_picker`, its `configuration` given a provider `list` and its `model_world`'s fake person pressing:
  - a key account's read is answered with a new name; Settings, Models says "Read from OpenAI" and the row "· new", off; All models tags it "new"; the picker doesn't offer it; a reload holds it all;
  - "Read again" is pressed, says "Reading…", and a read that fails says "Couldn't read OpenAI's list", keeping every row, with no revision made;
  - a read that drops a stock name narrows that account's offer, and a task holding it keeps its choice;
  - the person's switch turning the new model on survives another read, and a repeat read delivers no new `me`.

  Each is a table's row where its shape allows, and shares the world's setup.
- In `temper/tests/one-process/tests/suite/one_process.rs`, one row: a real worker, a hosted `agent-list` process and the fake's list, a key account added: one `Observation::List` with that key, its new name in `me`, and no attempt, spend or budget moved. If a key account isn't in that fixture, take the narrowest fixture beside it that has one, and say so.
- Fuzzy: `temper/tests/client/tests/suite/fuzzy/client.rs`'s referee still holds with these configurations left as they are, and with `keep` on: run it once, and don't add an action. A fuzzy budget that moves is yours by `common.md`.

## Inputs

- `temper/docs/plan/model-catalogue-3/notes/common.md`.
- `temper/docs/plan/model-catalogue-3/inputs/request.md`.
- `temper/docs/design/client/README.md`, 3.5; `jig/docs/design/domain/engine.md`, 17.12.

## Owns and may change

Owns the header's files.

- `temper/tests/engine/domain/src/protocol.rs:261,331`: the domain world's stand hears a read's `Held` and can answer it: a helper there, `answer_read`, in the stand's style; that file may change for it.
- `temper/tests/one-process/src/fixtures.rs:66,72,142,150` and `src/world.rs:1072,1117`: a provider's `keep`, and `Peer::set_list`.
- Any test oracle a list's correct behaviour trips is yours, by `common.md`.

Options, each pre-approved:

- A row that needs a clock moves it the way `a_switch_turned_off_and_back_holds_across_a_reload` does.
- If a one-process row would raise temper's focused suite past its 30 CPU seconds in the gate, keep it and take it out of `one_process.rs` for a row in `temper/tests/engine/domain`, no agent, and say so with the measurement: it isn't moved to a slow tier.

## Steps

Then `cargo fmt`; `heavy cargo clippy -p temper-client-world -p temper-one-process-world -p temper-engine-domain-world --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-client-world -E 'test(/model/)'`; `heavy cargo nextest run -p temper-one-process-world -E 'test(/one_process/)'`; `heavy cargo nextest run --profile fuzzy -p temper-client-world`. Commit and push, each commit with the trailer `Look-request: model-picker`, directly above the other trailers.

## Report

As the implementer's text says, with the suite's CPU figure beside its budget.
