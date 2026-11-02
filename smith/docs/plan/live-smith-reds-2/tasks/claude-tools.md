+++
id = "claude-tools"
part = "claude-tools"
kind = "feature"
needs = []
files = ["smith/tests/end-to-end/support/program_contract.rs", "smith/tests/end-to-end/program_runs.rs", "smith/tests/end-to-end/live.rs"]
effort = "high"
minutes = 60
+++
# The Claude story checks the tools a result-contract run is offered

## Goal

`smith::live claude_delivers_a_commit_in_place` fails on Claude Code 2.1.312 (main 46e6e70d9,
hetzner8) after a run that completed, passed its checks and delivered:

```
panicked at smith/tests/end-to-end/live.rs:569:9:
story=claude_delivers_a_commit_in_place account=claude invocation=1 step=Exec
failed=init provides a required built in or smith tool
exit=Code(0) forced=No live_pids=0  error=none  end=Completed
session=1 run.completed status=Accepted turns=4
session=1 run.started resumed=false messages=1 backend=ClaudeCode
session=1 tool.completed tool=Bash verdict=Result delivery=none   (twice)
session=1 check.completed exit=Code(0) passed=true
session=1 tool.completed tool=finish verdict=Result delivery=Delivered
```

The cause is the story's list, not Claude Code and not smith. `program_contract` (live.rs) requires
the first `ResponseStarted.prompt.tools` to hold `Read, Glob, Grep, Write, Edit, Bash,
permission_prompt, finish`. That list is smith's own offer (`Backend::offer_prompt`,
`smith/crates/smith-protocol-program/src/backend/measures.rs`: the MCP server's tools under
smith's names, then `permission_prompt`, then the granted built-ins), not the init's. Every
`smith exec` run has a result contract, so it is never offered `finish` over MCP
(`backend/files.rs`, `ToolKind::Finish if self.result_contract()`): it ends through
`StructuredOutput`, which the agent records as a `finish` completion. The design says so
(smith/docs/design/protocol/programs.md, 5.5: "Such a run is not offered `finish` over MCP, so the
model has one way to end, never two"), and the agent-process world already pins it
(`smith/tests/agent-process/tests/suite/program_events.rs`, the first prompt's tools hold no
`finish`). smith's own init check (`builtins::init_error`) accepted 2.1.312's init, so that init
listed each granted built-in exactly once: no CLI drift here.

Make the story check what the design requires, through one shared check that an offline,
fake-backed twin also runs, and make the check's failure name the tool.

Acceptance:

- A new support module holds the check; `program_contract` in `live.rs` uses it, its failure
  text naming the departing tool, and requires `finish` to be absent from the offer.
- An offline twin in the `end_to_end` binary runs a fake-backed `smith exec` on a `program`
  account granted inspect, modify and shell, reads its agent trace, and passes the shared check;
  run with the story's old eight names it is red on `finish` (the red before the fix).
- A pure test of the check: each departure is named.
- fmt, clippy and the focused tests below are green.

## Inputs

- `smith/docs/plan/live-smith-reds-2/notes/common.md` (read it first).
- `smith/docs/design/protocol/programs.md`, 5.3 (the built-ins per family and the agent's MCP
  server's tools), 5.5 (`system/init` lists no permission prompt; a result-contract run is not
  offered `finish`), 8 (the live suite checks what the fake assumes).
- `smith/docs/design/protocol/events.md`, 3.11 (`tools`: the names of the tools offered).
- `smith/docs/design/testing.md`, 2.3 ("A failure says why, in typed facts").

## Owns and may change

- `smith/tests/end-to-end/support/program_contract.rs` (new).
- `smith/tests/end-to-end/support/mod.rs`: one line, `pub mod program_contract;`, after
  `pub mod live_run;`, and nothing else.
- `smith/tests/end-to-end/live.rs`: the function `program_contract` only.
- `smith/tests/end-to-end/program_runs.rs`: new tests, and new helpers beside `signed_in` if the
  twin needs them; existing tests unchanged.
- Pre-approved:
  - If `support::live::arguments("claude", ...)` cannot drive the fake (a setting the fake-backed
    product refuses), build the twin's arguments as `a_program_exec_traces_its_run_to_the_agent_file`
    does, adding `-C <scratch>/repo` and the live arguments' `workspace.directories` entry
    (`repo`, writable, git false) and `workspace.contract.form='report'`; say so.
  - If the twin's run is granted other families than inspect, modify and shell, give it the
    settings the live arguments give and say which; if it still differs, stop and ask.
  - If the `end_to_end` binary already has the feature on without `--features end-to-end`, drop
    the flag from the commands below.

## Steps

1. Read the inputs, `live.rs`'s `program_contract`, `program_runs.rs` (`signed_in`, `outcome`,
   `a_program_exec_traces_its_run_to_the_agent_file`) and `program_runs/records.rs` (`agent`).
2. `smith/tests/end-to-end/support/program_contract.rs`:

   ```rust
   //! The live Claude story's check of what a program run's first prompt offers
   //! (smith/docs/design/protocol/programs.md, 5.3 and 5.5).

   /// What a program run granted inspect, modify and shell offers in its first prompt, under smith's
   /// names: the six built-ins and the permission prompt.
   pub const OFFERED: [&str; 7] = ["Read", "Glob", "Grep", "Write", "Edit", "Bash", "permission_prompt"];

   /// The first way `tools` departs from what a result-contract run on those families is offered, in
   /// fixed words naming the tool: a name of `OFFERED` missing, `finish` offered, or `sub_agent`
   /// offered. `None` when it departs in none.
   #[must_use]
   pub fn offered_tools_fault(tools: &[&[u8]]) -> Option<String>;
   ```

   The words are fixed, e.g. `the first prompt offers no Glob`, `the first prompt offers finish
   under a result contract`, `the first prompt offers sub_agent`; a name the list holds but these
   rules do not mention (`remove`, a host tool) is no fault.
3. `program_runs.rs`, the twin first, run red:
   - `a_program_exec_offers_the_live_claude_story_s_tools`: a scratch whose user layer has the
     `program` account `claude` on the fake (`[programs] claude_code=<fake-claude>`,
     `[accounts.claude] provider='anthropic' source='program'`), `smith login claude` against it,
     a fake story of one turn ending in a report `Step::Outcome`; then `smith exec` with
     `support::live::arguments("claude", scratch.path(), false, false)` and
     `support::live::with_trace`. Assert exit 0; from `records::agent(&trace)`: `run.started`'s
     `tools.families` is `[Inspect, Modify, Shell]`, and the first `ResponseStarted` whose
     `prompt.tools` is non-empty passes `offered_tools_fault` (the failure message is the fault).
   - Before writing `offered_tools_fault`'s body, run the twin once with a body that requires the
     story's old eight names: it is red on `finish`. Note it for the report.
   - `the_offered_tools_check_names_each_departure`: `OFFERED` plus `remove` passes; each name of
     `OFFERED` dropped in turn is named; `finish` added is named; `sub_agent` added is named.
   `heavy cargo nextest run -p smith --features end-to-end --test end_to_end -E 'test(/offer/)'`.
4. `live.rs`, `program_contract`: keep the admission, backend and families assertions; take the
   first non-empty offer as today, its failure labelled `the first prompt's offered tools`; replace
   the name loop and the `sub_agent` assertion with one `offered_tools_fault` call whose fault is
   the report's failed text (`report(&fault)`). Check with `heavy cargo check -p smith --tests`;
   never run the `live` binary.
5. `cargo fmt`; `heavy cargo clippy -p smith --features end-to-end --all-targets -- -D warnings`.
6. `heavy cargo nextest run -p smith --features end-to-end --test end_to_end -E 'test(/program_runs/)'`,
   then `heavy cargo nextest run -p smith --test live_configuration`.
7. Commit (the check and twin, then the story), push.

## Report

The red run of step 3 (test, failed text), the files changed, which pre-approved option you used,
the test runs (selected, passed), and implementation and check minutes apart.
