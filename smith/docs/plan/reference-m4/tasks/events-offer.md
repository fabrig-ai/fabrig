+++
id = "events-offer"
part = "events-offer"
kind = "feature"
needs = []
files = ["smith/crates/smith-protocol-events/src/events.rs", "smith/crates/smith-protocol-events/src/events/program_tests.rs", "smith/tests/agent-process/tests/suite/program_events.rs"]
effort = "medium"
minutes = 30
+++
# `run.started` withholds no family on a program backend

## Goal

`run.started`'s `tools.withheld` lists the granted families a run does not offer; on a program
backend that was `sub_agents`, and now none is withheld on either backend
(`smith/docs/design/protocol/events.md`, 3.3). Stop pushing `Family::SubAgents` there and bring
the two tests that assert it to the new meaning. The field stays, always empty for now: it is a
stored record's field.

Acceptance:

- `Producer::starting` (`smith-protocol-events/src/events.rs`, about line 186) leaves
  `withheld` empty whatever the backend; nothing else of `run.started` changes.
- `smith-protocol-events`' `a_program_run_withholds_sub_agents` becomes
  `a_program_run_withholds_no_family`, asserting an empty `withheld` for each of its four cases.
- `smith-agent-process-world`'s `the_run_s_tools_withhold_sub_agents`
  (`tests/suite/program_events.rs`, about line 116) becomes
  `the_run_s_tools_withhold_no_family`: `withheld` is empty. Its second half, that no response's
  prompt lists a `sub_agent` tool, stays as it is: the program's prompt does not offer it until
  task `program-children`, which changes that half.
- fmt, clippy and the focused tests below are green.

## Inputs

- `smith/docs/plan/reference-m4/notes/common.md` (read it first).
- `smith/docs/design/protocol/events.md`, 3.3 (`run.started`'s `tools`).
- `smith/docs/design/protocol/programs.md`, section 2 (sub-agents on a program backend).

## Owns and may change

- `smith/crates/smith-protocol-events/src/events.rs`: the `withheld` push in `starting`, and the
  `programs` field of the producer only if nothing else reads it once the push is gone (it is
  also read at about line 383 for a model's backend; keep it then).
- `smith/crates/smith-protocol-events/src/events/program_tests.rs`: the test above.
- `smith/tests/agent-process/tests/suite/program_events.rs`: the test above, its `withheld`
  assertion and its name only.
- Pre-approved: if a `with_capacity(1)` list is now always empty, keep the list and its type
  (`smith_events::Tools.withheld` does not change) and build it empty.

## Steps

1. Change the two tests' assertions first; run them red:
   `heavy cargo nextest run -p smith-protocol-events --lib -E 'test(/withholds/)'` and
   `heavy cargo nextest run -p smith-agent-process-world --test suite -E 'test(/the_run_s_tools_withhold/)'`.
2. Remove the push; rerun both green.
3. `cargo fmt`; `heavy cargo clippy -p smith-protocol-events -p smith-agent-process-world
   --all-targets -- -D warnings`.
4. Focused tests: `heavy cargo nextest run -p smith-protocol-events`, then
   `heavy cargo nextest run -p smith-agent-process-world`.
5. Commit, push.

## Report

The red runs, the change, the test runs (selected, passed), and implementation and check minutes
apart.
