+++
id = "claude-trace"
part = "claude-trace"
kind = "feature"
needs = ["refresh-lead"]
files = ["smith/tests/end-to-end/live.rs", "smith/tests/end-to-end/support/live.rs", "smith/tests/end-to-end/live_configuration.rs", "smith/tests/end-to-end/program_runs.rs", "smith/tests/end-to-end/program_runs/**"]
effort = "high"
minutes = 100
+++
# The live stories read a Claude run's records from its agent's trace

## Goal

`smith::live claude_delivers_a_commit_in_place` fails at "run reports the expected resume state"
after a completed, exit-0 run, and every Claude leg of
`a_chat_crosses_from_chatgpt_to_claude_then_back` must fail the same way. The cause is in the
live harness, not the product: on a `program` account `smith exec` spawns its agent, and the
product's `--json` stream holds only the host's records (`session.started`, notices,
`run.completed`, `session.ended`); the run's detail records (`run.started`, `response.started`,
`tool.completed`, `check.completed`, ...) are the spawned agent's, written to the configured
trace path with `.agent` appended (smith/docs/design/protocol/events.md, 2, "Where it goes").
The live arguments name no trace path, so no agent trace exists, and `live.rs` reads only the
product's stream. Two offline tests already hold the split
(`smith/tests/end-to-end/program_runs/records.rs` `answer`, "detailed records belong to the child
process"; `smith/tests/local-process/tests/suite/events.rs`), and smith-bench reads program runs
this way (`smith/benchmarks/src/run/traces.rs`; `Observing::agent_trace` and `events()` in
`smith/benchmarks/src/agent/smith/observe.rs`; `Prepared::program_source` in `prepare.rs`).

Make the live harness do as smith-bench does: give each invocation its own trace path, mark a
`program` account's observer as a program source, feed `<trace>.agent` to it after the exec
settles, and take the run's detail records from `observing.events()`, the host's end and exit from
`observing.observer`.

Acceptance:

- `smith/tests/end-to-end/support/live.rs` gains, beside `with_refresh_lead`:
  ```rust
  /// Insert `-c trace.path=<path>` before the trailing prompt argument `-`.
  pub fn with_trace(arguments: &mut Vec<std::ffi::OsString>, path: &std::path::Path);

  /// The rendered settings an observer needs to tell a program account (smith-bench's `Prepared::rendered`).
  #[must_use]
  pub fn rendered(account: &str) -> serde_json::Value;
  ```
  `rendered("claude")` names the `anthropic` endpoint's account `claude` with source `program`;
  `rendered("chatgpt")` its `openai` account `chatgpt` with source `sign_in`; the shape is the
  one `Prepared::program_source` reads.
- `live.rs` `run`: the trace path is `<scratch>/trace-<invocation>.jsonl` (the `Leg`'s invocation
  from `live-report`), spliced with `with_trace` for every account; `observer` sets
  `Prepared { rendered: live::rendered(account), trace: Some(path), .. }`; after the exec settles,
  for a program source, read `<path>.agent` bounded (at most the stdout bound the exec uses) and
  pass it to `Observing::agent_trace`, its read error as `Err` text. `assert_exec` takes every
  detail record (`RunStarted`, `ResponseStarted`, `ToolCompleted`, `CheckCompleted`, and
  `program_contract`'s) from `observing.events().sessions()`, and classification and
  `finish()` from both observers as smith-bench's `health` does (the host's for exit and end).
  The `live-report` report lists the agent trace's read error when there is one.
- Offline tests:
  - `live_configuration`: `the_claude_live_story_reads_its_agent_trace`: `live::arguments("claude", ..)`
    with `with_trace` parses (as `assert_account` does) and resolves with a trace path, and an
    `Observing` built from `Prepared { rendered: live::rendered("claude"), .. }` has
    `prepared.program_source()` true, and from `live::rendered("chatgpt")` false. Red before
    (no `with_trace`/`rendered`), green after.
  - `end_to_end` (`smith/tests/end-to-end/program_runs.rs`):
    `a_program_exec_traces_its_run_to_the_agent_file`: on the fake program (`signed_in` with an
    `outcome()` turn, as `a_program_turn_exit_keeps_the_run_failure` sets up),
    `exec --json --chat chat -c trace.path=<file> -c trace.capture='everything' "Ask first"`
    exits 0; `<file>.agent` holds a `run.started` with `resumed == false`, a `response.started`
    whose prompt holds the task text, and the `finish` tool's completion; the product's stdout
    holds none of them (`records::answer` still passes). This pins what the live harness reads.
- fmt, clippy and the focused tests below are green; the `live` binary compiles.

## Inputs

- `smith/docs/plan/live-smith-reds/notes/common.md`; `notes/contracts.md` (the live report,
  landed with `live-report`).
- `smith/docs/design/protocol/events.md`, 2 ("Who writes it", "Where it goes");
  `smith/docs/design/testing.md`, 2.3 (the Claude story).
- The smith-bench code named in Goal, and `smith/tests/end-to-end/program_runs.rs`,
  `program_runs/records.rs` for the offline world.

## Owns and may change

- `smith/tests/end-to-end/live.rs`, `support/live.rs` (the two functions; `arguments` itself does
  not change), `live_configuration.rs` (the new test and helpers),
  `program_runs.rs` and `program_runs/**` (the new test and a reader for the `.agent` file beside
  `records::read`).
- Not `smith/benchmarks`, not the product: if `Observing` cannot be given an agent trace without
  a smith-bench change, stop and ask.
- Pre-approved:
  - If the trace path must be given as `--trace <path>` rather than `-c trace.path=`, use the flag
    and say why; the check step must still pass with it (`smith check` writes no execution state).
  - If a detail record `assert_exec` checks (the `finish` delivery, `check.completed`) is in
    neither stream for a program run in the offline test, keep the live assertion on the stream
    the offline test finds it in, and report which.
  - If the fake program cannot run a `--deliver` change, the offline test stays a report-form run
    as specified; do not add a fake story for delivery.
  - If `observing.events()` for an inline (ChatGPT) run is the product's observer (it is when
    `program_source()` is false), ChatGPT stories need no other change: say so.

## Steps

1. Write the `live_configuration` test; run it red
   (`heavy cargo nextest run -p smith --test live_configuration -E 'test(=the_claude_live_story_reads_its_agent_trace)'`).
2. Write the `end_to_end` test; run it
   (`heavy cargo nextest run -p smith --test end_to_end -E 'test(=program_runs::a_program_exec_traces_its_run_to_the_agent_file)'`;
   check the module path with `--list` if the filter selects nothing). It describes the product
   as built: if it is red, stop and ask with what it found.
3. Add `with_trace` and `rendered`; wire `live.rs`; step 1's test green;
   `heavy cargo check -p smith --tests`.
4. `cargo fmt`; `heavy cargo clippy -p smith --all-targets -- -D warnings`;
   `heavy cargo nextest run -p smith --test live_configuration`;
   `heavy cargo nextest run -p smith --test end_to_end`.
5. Commit, push.

## Report

The red of step 1, step 2's result, which stream each live assertion now reads, any
pre-approved option used, the tests run (selected, passed), and implementation and check minutes
apart. Note for the lead: `events.md` 2's sentence "It holds the run's records whichever kind of
agent the local host runs" reads against "Where it goes"; say what the offline test showed.
