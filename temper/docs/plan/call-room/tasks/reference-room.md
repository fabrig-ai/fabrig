+++
id = "reference-room"
part = "reference"
kind = "feature"
needs = ["large-argument", "smith-room", "jig-room"]
files = ["temper/docs/examples/reference/**", "temper/crates/temper-service/src/startup/checks.rs", "temper/crates/temper-service/src/startup.rs", "temper/crates/temper-service/src/tests/root_checks.rs", "temper/crates/temper-service/src/tests/checks.rs", "temper/crates/temper-service/src/tests/stock_brief.rs", "temper/crates/temper-service/src/tests/workers.rs", "temper/crates/temper/src/tests/reference_settings.rs", "temper/crates/temper/tests/support/owned.rs"]
effort = "high"
minutes = 180
+++

# The reference declares a call's room, and the start names what moves with it

## Goal

The reference takes a call of about 128 KB, by its own figures and not the stock's: its agent's `tool_payload` and its endpoints' `output`, its charters' models, and every limit that moves with them. This task writes those figures, picks them by the start's checks, and makes the start name each limit that falls short.

Acceptance:

- `temper/docs/examples/reference/room.json` exists, `limits.json` and `settings.json` take their share, and `README.md` gives the two commands, as `notes/contracts.md`, 4 has them.
- A test merges `limits.json` and `room.json` over `temper init`'s document, sets init's agent as `room.json` says, and passes `complete_profile_startup`, at 131,072 and 98,500. If the start's checks refuse those, or `smith-room`'s row was refused, the files hold 63,740 and 48,000, and the report says what refused.
- A test holds every figure in those files to `temper/docs/design/protocol/configuration.md`, 13.8's relations, computed from that agent's measured T and payload, so an edition that drifts is red.
- `each_turn_bound_accepts_equality_and_names_one_byte_short` has rows for both hubs' `hub.outcome_bytes` and for `forge.client.op_bytes`, each accepting equality and naming one byte short. The other limits of 13.8's table already have their rows (`root_checks.rs`, `hello_refused.rs`, `startup/calls_tests.rs`): say so in the report, or add the one that lacks it.
- A `change` input of the reference's payload decodes with its words whole, in `temper-engine-smith`'s tests; one past `HostInput::CAPACITY` is `Problem::TooLarge`.
- At the reference's limits a brief with a section of the pair's payload P, the largest spec one call carries, stays whole through both entrances, and one past the room is cut with its `[cut N bytes; whole: READ]` line: rows of `the_stock_brief_with_a_whole_spec_fits_both_receiving_bounds` and `review_fitting_uses_the_configured_assignment_envelope`. At 131,072 and 98,500, P is that 128 KB; at the fallback, see `## Answers`.
- `temper init`'s figures, `temper-service/src/stock.rs` and the T of 16,803,810 don't move, and no charter's words, spend, period or time does.
- Focused and fuzzy tests of each changed package and of the readers below pass.

## Inputs

- `temper/docs/plan/call-room/notes/common.md`; `temper/docs/plan/call-room/notes/contracts.md`, 1, 4 and 5.
- `temper/docs/design/protocol/configuration.md`, 13.8 and 13.9.
- `temper/docs/design/domain/agent.md`, 4.3, *The brief fits the room its charter leaves*.
- `smith-room`'s report gives T at each pair; its test is `forty_eight_calls_per_response_derive_with_two_models`.

## Owns and may change

Owns the header's files, and one test row in `temper/crates/temper-engine-smith/src/tests.rs`.

Readers (surveyed): these read `settings.json` or `limits.json` and pin no output or spend; run them, and edit only as said.

- `temper/crates/temper/src/tests/reference_settings.rs`: `the_reference_settings_fit_the_reference_profile` reads init's limits; it merges `limits.json` first, since template 4's report now takes the reference's `tasks.result_bytes`.
- `temper/crates/temper-service/src/tests/root_checks.rs`: `the_reference_profile_keeps_four_questions_per_change` and `shipped_payload_profiles_pass_complete_startup_checks` give the reference's document the reference's agent, and init's document init's.
- `temper/crates/temper/tests/support/owned.rs`, `bounded_reference`: overlays `room.json` too, the agent and each worker, so `the_reference_profile_passes_startup_admission` (`temper/crates/temper/tests/suite/startup.rs`) holds.
- `temper/tests/engine/domain/tests/suite/{configuring.rs,host_calls.rs,forge_routes/goal_branch.rs}`, `temper/crates/temper-init/src/tests/documents.rs`, `temper/crates/temper-client-tools/src/tests/jig/settings.rs`, `temper/crates/temper-service/src/tests/reference_landing.rs` and `temper/tools/stage-one`. A world whose own limits are below the reference's may set template 4's words to its own bound on the value it read, one line, and say so.

Options, each pre-approved:

- **A limit outside `notes/contracts.md`, 4 that the start's checks ask to move** moves with them, sized from 13.8's relations, in `limits.json` or `room.json`: name it in the report. A budget, a spend, a period or a time never does: that is a `Question:`.
- **The validity check that says only `engine.limits: past its bound`** (`temper-engine-domain/src/limits.rs`) stays as the rule. Name the relation first, in `checks.rs`, so that what a person reads is the named line.
- **A check's words:** "the agent's outcome (agent.json profile.declared.tool_payload)" for `hub.outcome_bytes`, and "tasks.result_bytes" for `forge.client.op_bytes`, through `short`.
- **`hub.charter_bytes`** gets no check: past it a brief is cut, not refused.
- **A `small` document whose agent's payload passes 4,096** now falls short of `hub.outcome_bytes`: set that document's override to the agent's outcome, one line, and name it. It's what the check is for.
- **`forge_component.body_bytes`** comes from `temper-service/src/forge.rs`'s `Unusable::BodyBytes`; init's 21,341 is 2 × 10,660 + 21. If the check wants another relation, take its figure and say so.

## Steps

`room.json` and `limits.json` as `notes/contracts.md`, 4 gives them, each figure from 13.8: T from the reference's agent, measured with `fixtures::parsed_agent(fixtures::stock_configuration(payload, output))`, its `limits.bodies.turn_body`. `settings.json`: every charter model's `"output"`, and template 4's `contract.report.words`. Add a section to `README.md`, *A call's room*, after the document's limits and before the settings' order, saying the agent and the workers are set before the first start and the settings after it, since the settings' models fit no agent smaller. Leave its `cat > reference-authority.json` recipe as it is: tests split on it.

The start, in `checks::turns`, which holds the engine, each worker and the agent:

```rust
short(problems, engine_path.member(b"hub.outcome_bytes"), engine.hub.outcome_bytes,
      b"the agent's outcome (agent.json profile.declared.tool_payload)", Some(agent.host_limits.outcome_bytes));
// and each worker's, as hub.turn_bytes is
```

The tests, in `temper-service/src/tests/root_checks.rs`:

```rust
fn reference_document(generated: &[u8]) -> Box<[u8]>;         // limits.json, then room.json's worker over each worker
fn reference_agent() -> crate::settings::Agent;               // init's, as room.json's agent says

#[test]
fn the_reference_room_starts_and_holds_its_relations() { /* complete_profile_startup; then each figure against T and P */ }
```

Say in that test's comment what it costs in CPU, measured. A row or test that holds the figures must fail on a figure moved by hand: break one, see it, report it.

This changes what a shared file means, so run its readers' tests: `heavy cargo nextest run -p temper-service -p temper-engine-smith -p temper-init -p temper-client-tools -p temper-engine-domain-world -p temper-stage-one`; `-p temper --lib`, and `-p temper --test suite -E 'test(the_reference_profile_passes_startup_admission)'`, which the focused profile runs. Compile any other story `bounded_reference` feeds, with `heavy cargo check -p temper --tests`. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-service -p temper-engine-smith -p temper --all-targets -- -D warnings`. Commit, push.

## Report

Which figures the reference took and why, and T for them. Each limit that moved, with its value, and any not named in `notes/contracts.md`. What the start names for each limit in 13.8's table, as a list. The test's cost. Any heap ceiling that moved, before and after. Anything the reference now can't do on a stock agent, in a line for the design.

## Answers

2026-11-07. **Q: at the fallback, smith's charter allowance is 2 × 63,740 = 127,480 bytes, so a 131,072-byte section can't stay whole. Does the brief acceptance bend, or must smith admit a larger charter?** The acceptance follows the pair; smith's derivation doesn't move.

- **The fallback stands.** `smith-room` measured the refusal (`turn-in-transcript`, `session.said_text` 1,245,440 past `transcript.said_text` 1,048,576), so `room.json` holds 63,740 and 48,000. The request ties 128 KB to the larger pair and 64 KB to this one, and asks of a brief that it hold a whole spec and be cut past its room. A spec reaches the store through one call, so at this pair it stops at about P, and none of the pipeline's 1,406 passed 64 KB (`goal-as-designer.md`, 10). The cut's 128 KB was the larger pair's P.
- **The receiving rows,** in `the_stock_brief_with_a_whole_spec_fits_both_receiving_bounds` or a table beside it in `stock_brief.rs`, on `fixtures::stock_configuration(63_740, 48_000)`, the coding charter and its writable workspace. Measure the room in the test, from `run::charter_room` and `run::brief_room` against the charter with an empty section, never as a literal:
  - a section of 63,740 stays whole;
  - a section at the measured room stays whole;
  - a section one byte past it is cut, with its `[cut N bytes; whole: READ]` line;
  - a section of 131,072 is cut, with that line. The asking run's probe kept 90,190 bytes here: that's a check on the measurement, not a figure to pin.
- **The envelope's row,** in `review_fitting_uses_the_configured_assignment_envelope`: a bound of 262,144 keeps a section of 63,740 whole, and one past that bound's room is cut. This entrance's room passes the receiving one, and that is fine: the worker's fit is the one that binds.
- **The existing assertions stay as they are:** these are new rows. `hub.charter_bytes` stays 262,144, as `jig-room` set it and the request asks.
- **`notes/contracts.md`, 4's figures are the larger pair's.** `room.json` takes the fallback's by 13.8's relations: `hub.event_bytes` is 63,738, the payload less two, and the zeros follow T = 18,431,842, read from the measurement rather than copied. Only the pair's figures change; `limits.json`'s and `settings.json`'s other shares stay as the spec has them.
- **If a charter the reference gives a `change`'s spec leaves less than 63,740 of room,** with the reference's own instructions, that's a `Question:` with the measurement: a whole spec is what the request asks for. The asking run's probe says it shouldn't happen, leaving about 90 KB on the stock's coding charter.
- **Report, under what the reference can't do:** a section past the measured room, whatever `hub.charter_bytes` allows, is cut at the worker, and why: the receiving allowance, 2 × P, binds before 262,144. `room-close` writes that line into 13.8.
