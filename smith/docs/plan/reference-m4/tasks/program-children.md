+++
id = "program-children"
part = "program-children"
kind = "feature"
needs = ["close-lower", "mcp-sub-agent", "events-offer"]
files = ["smith/crates/smith-agent-service/src/service/program.rs", "smith/crates/smith-agent-service/src/service/program/**", "smith/crates/smith-agent-service/src/service/stages.rs", "smith/crates/smith-agent-service/src/service/request.rs", "smith/crates/smith-domain/src/translate.rs", "smith/crates/smith-domain/src/tests/program.rs", "smith/tests/agent-process/src/program.rs", "smith/tests/agent-process/tests/suite/main.rs", "smith/tests/agent-process/tests/suite/program_children.rs", "smith/tests/agent-process/tests/suite/program_events.rs"]
effort = "high"
minutes = 180
+++
# A program run's sub-agent runs on a program of its own

## Goal

Make the story run: on a program backend, main asks `sub_agent` through the agent's MCP server,
and the run answers it with a child that is a `claude` process of its own, at the model and
effort of the child's LLM, inspect only; the child's answer is main's tool result, and the
child's program ends with its conversation. The halves exist on the trunk: `Lower::Close` /
`FromDomain::Close` (task `close-lower`), the program's `sub_agent` listing and decoding (task
`mcp-sub-agent`), an empty `withheld` (task `events-offer`). This task keeps a program backend
per child in the service (contract C3) and stops smith-domain withholding the family on a
program.

Acceptance:

- `smith-domain/src/translate.rs` no longer drops `agents` for a program conversation (about line
  88: `agents: families.agents && program.is_none()`): a program main whose charter grants
  sub-agents is offered `sub_agent`; a child is still never offered it.
- The service keeps one `smith_protocol_program::Backend` per live child conversation on the
  program, as C3 says: created at the child's first `Complete` (`prompt.asker == Child` and main's
  backend `holds` its grant), given its own token range and entropy and main's directory,
  workspace, mounts, grant, turn limit and run time; `Cancel`, `Yield` and `Close` reach the
  backend that keeps their owner; io, file and pipe events reach the backend whose tokens they
  carry; `Close` closes the child's backend, and it is dropped once `closed()`. The run's end
  closes every child backend with main's, and the service is closed only when all are.
- The service reserves io, file slots and pipes for main and `conversations - 1` children
  (`limits.domain.run` or wherever the agent's `conversations` lives; design: programs.md,
  5.10); a configuration whose children's token ranges do not fit `OWNER_NAMES` is refused as
  `ConfigError::Limits`.
- A child's completions whose grant main's backend does not hold go to the direct backend, as
  today.
- In the agent-process world (`smith/tests/agent-process`), a program run's main asks one
  inspect-only child and finishes with a report holding its answer; the tests below pass.
- fmt, clippy and the focused and readers' tests below are green.

## Inputs

- `smith/docs/plan/reference-m4/notes/common.md` (read it first) and
  `smith/docs/plan/reference-m4/notes/contracts.md`, C1 to C4.
- `smith/docs/design/protocol/programs.md`, section 2 and 5.10 (whole), 5.1 (the command), 5.7
  (a cancel), 5.8 (the session id).
- `smith/docs/design/domain/run.md`, 5.3.

## Owns and may change

- `smith/crates/smith-agent-service/src/service/program.rs` and `program/**` (its tests): the
  `Program` struct gains the child backends, `complete` routes by owner, `fire`, `deadline`,
  `closed`, `pending`, `file_event`, `pipe_event`, `io_event`, `down`, `renew`, `install` and
  `wait_for_close` cover every backend; `reserved` and `new` size for the children.
- `smith/crates/smith-agent-service/src/service/stages.rs` and `request.rs`: call sites only.
- `smith/crates/smith-domain/src/translate.rs`: the line above. `smith-domain/src/tests/program.rs`:
  `a_program_main_offers_no_sub_agent` becomes `a_program_main_offers_its_sub_agents`
  (`prompt.served` contains `Served::SubAgent`).
- `smith/tests/agent-process/src/program.rs`: the harness's fake `claude` and relay `instances`
  (about lines 636 and 645) raised so main and two children run at once (fake: 6, relay: 3,
  or what the children need, with a comment why), and helpers for a charter with a child model.
- `smith/tests/agent-process/tests/suite/program_children.rs` (new) and `main.rs` (its `mod`).
- `smith/tests/agent-process/tests/suite/program_events.rs`: `the_run_s_tools_withhold_no_family`'s
  second half: main's prompt now lists `sub_agent`; assert it does, renaming the test
  `the_run_s_tools_offer_sub_agents_and_withhold_no_family`.
- Neighbour edits allowed in `smith-protocol-program` for a child's backend only, each with a unit
  test in its `backend/*_tests.rs`: a child's prompt (asker `Child`, no `Served::Finish`) gives no
  `--json-schema` and lists only the prompt tool, which the code does when this was written
  (`builtins.rs` `result_contract`, `files.rs` `tools()`); if it does not, make it so.
- Pre-approved:
  - Token ranges: give child `k` (1-based) the base `FIRST` plus `k` times a stride that covers
    the backend's own tokens (`backend.rs` `token()`; seven today) and the MCP server's, within
    `OWNER_NAMES`. Entropy: sha256 of main's entropy, a fixed label and the child's owner token,
    as `Names::new` derives names (`smith-protocol-program/src/backend/start.rs`).
  - If a child's slot is free only after its backend's close settles, a child whose `Complete`
    finds no free slot fails that completion as a program that could not start
    (`ToDomain::Failed` with the backend's program failure), naming the bound in its detail, and
    a test shows it; say so.
  - If an existing test in `smith-domain`, `smith-agent-service` or `smith-agent-process-world`
    fails only because a program main is now offered `sub_agent` (its tool count, its system
    text's delegation section, a byte measure), update that expectation and list it; if its
    meaning changes otherwise, ask.
  - If `jig-channel-world` or `temper-real-loop-world` (step 7) fails, stop: commit and push what
    you have and ask, naming the test and the difference; those trees are not this plan's.

## Steps

1. Read the inputs, `service/program.rs` whole, and the agent-process harness
   (`smith/tests/agent-process/src/program.rs`: `Case`, `run`, the story fixture) and
   `tests/suite/program.rs`' `a_program_main_may_share_its_account_with_a_declared_child`.
2. Write `program_children.rs` first and run it red
   (`heavy cargo nextest run -p smith-agent-process-world --test suite -E 'test(/program_children/)'`):
   - `a_program_main_reads_its_sub_agents_answer`: a charter granting sub-agents whose `models`
     holds main's LLM again with `effort` `high` (main's has another or none); a story (C4) whose
     positional turns are main's: turn 1 `Step::CallChecked { tool: "sub_agent", input:
     {"brief":"CUE-ALPHA List what lib.rs exports.","tools":["inspect"],"llm":<that model>},
     contains: "exports run" }`, turn 2 `finish` with a report; and a `cued` script on
     `CUE-ALPHA` whose turn says `exports run` and ends. Assert: the run is accepted; one fake
     invocation is the child's `-p`, with `--model` the child's model, `--effort high`, `--tools
     Read,Glob,Grep`, `--permission-prompt-tool`, an `--mcp-config` and no `--json-schema`; the
     events hold a `conversation.opened` of kind `child` with its call and model and its
     `conversation.closed`; main's told `sub_agent` result holds `exports run` and the run's
     usage line (`Child usage:`).
   - `a_program_childs_program_ends_with_its_conversation`: the same run; the child's program
     exited (its observations end, or the harness's process table shows it gone) before main's
     second turn's `Complete`; and, where the harness shows the programs root, its private run
     directory is gone at the run's end (say in the report if it cannot).
   - `a_program_child_asking_to_write_is_refused`: main's call asks `"tools":["inspect","modify"]`;
     main's told result is invalid naming `tools`, and no child invocation happens.
3. Flip `translate.rs`; update the smith-domain test; run `heavy cargo nextest run -p smith-domain`.
4. The service's child backends, contract C3, until step 2's tests pass.
5. `the_run_s_tools_offer_sub_agents_and_withhold_no_family`; run `program_events`.
6. `cargo fmt`; `heavy cargo clippy -p smith-agent-service -p smith-domain -p smith-protocol-program
   -p smith-agent-process-world --all-targets -- -D warnings`.
7. Focused and readers' tests, one command at a time: `heavy cargo nextest run -p smith-domain`,
   `-p smith-protocol-program`, `-p smith-agent-service`, `-p smith-agent-process-world`,
   `-p smith-hosts-world`, `-p smith --test end_to_end`; then the other trees' program runs:
   `heavy cargo nextest run -p jig-channel-world -E 'test(/program/)'` and
   `heavy cargo nextest run -p temper-real-loop-world -E 'test(/program/)'`.
8. Commit (the flip, the service and the worlds may be separate commits), push.

## Report

The red runs of step 2, how the service keeps and routes child backends (tokens, entropy, slots,
close), every pre-approved option you took, every test expectation you updated and why, the test
runs (selected, passed), and implementation and check minutes apart.
