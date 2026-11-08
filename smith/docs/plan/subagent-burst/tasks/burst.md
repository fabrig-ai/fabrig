+++
id = "burst"
part = "burst"
kind = "feature"
needs = []
files = ["smith/tests/agent-process/tests/suite/program_children.rs", "smith/tests/agent-process/src/program_world.rs", "smith/tests/agent-process/src/program_world/**", "smith/tests/fake-claude/src/print.rs", "smith/tests/fake-claude/src/story.rs", "smith/docs/design/domain/run.status.md", "smith/docs/design/protocol/limits.status.md", "smith/docs/design/STATUS.md"]
effort = "high"
minutes = 240
+++

# A burst of sub-agents past a run's room never aborts the agent

## Goal

On a deployment, a program-backed main (Claude Code, sub-agents on
another provider's model) asked five or six `sub_agent` calls in one
message, at a room of five conversations, and the agent process aborted
on its main thread within about 15 s of the last call, four times; fewer
calls at once lived. Write the failing rows first, in the program world
below the real loop; let the first failing row name the bound that
panics; fix that bound so a call past the room is answered as
`smith/docs/design/domain/run.md`, 5.3 (*The room*) says; and say which
bound it was.

Acceptance:

- **The rows** of `notes/contracts.md`, section 1, in
  `smith/tests/agent-process/tests/suite/program_children.rs`: one table
  test over N in 2, 4, 5, 6 and the child's path (program, direct),
  eight rows, at a room of 5. It takes in
  `a_program_main_reads_two_children_from_one_message` as its (2,
  program) row, keeping that test's assertions for every row; each row
  asserts the outcome the contracts give: no abort, `min(N, 4)` children
  opened, answered and closed, `N - 4` results `refused reason=too-many`,
  main accepted, directories removed.
- **Seen failing first:** the report names the first row that failed on
  the base, the panic's message and location, and the bound it was:
  one of the request's three candidates
  (`smith-domain-run/src/run/child.rs`'s `run.children.insert`,
  `smith-agent-service/src/service/down.rs`'s `owners.insert(Owner::Llm)`,
  `smith-protocol-program/src/backend/start.rs`'s `self.calls.push`) or
  another.
- **The fix** at that bound, in its crate, answers the call as the
  contracts say, or makes the burst fit, without raising the room or any
  production figure; and a row at the lowest tier that holds that bound
  (the bound's own crate's table, where one exists) fails without the
  fix and passes with it.
- The **Not yet** entries `5.3: the room …`
  (`smith/docs/design/domain/run.status.md`) and `6: a `sub_agent` call
  past a run's `conversations` …`
  (`smith/docs/design/protocol/limits.status.md`) are gone, and the
  phrase `a burst of sub-agents past a run's room answered call by call
  (`domain/run.md`, `protocol/limits.md`),` from
  `smith/docs/design/STATUS.md`'s summary.
- Clippy clean, fmt clean, the focused tests below green.

## Inputs

- `smith/docs/plan/subagent-burst/notes/common.md` and `notes/contracts.md`,
  section 1, which binds the rows and their outcome.
- `smith/docs/plan/subagent-burst/inputs/request.md`, item 1.
- `smith/docs/design/domain/run.md`, 5.3, the bullets *The room* (this
  cut's), *On a program backend* and *The call*.
- `smith/docs/design/protocol/limits.md`, 2.2 (`conversations`), 3.1
  (the pool's `calls` and `connections`), 3.8 (routes) and section 6 (the
  `sub_agent` row, this cut's).
- `smith/docs/design/protocol/programs.md`, 5.10 (*Side by side*, *Each
  child a program of its own*, *A child on a credential*).
- What a read-only survey found, by reading alone (no build): none of
  the three candidates is reached by a burst's width alone. Admission
  checks `means.conversations >= run_conversations` before inserting the
  child, and the run's count stays main plus its retained children on
  every path (`run.rs` 995 and 1033, `run/jobs.rs` 61); routes are io's
  operations plus 64, and a direct child opens at most one pooled
  connection; the MCP server holds at most `calls_per_response` calls
  and pauses reading at it. Its next suspect is the session's batch
  reservation, `smith-domain-session/src/session/history.rs`, the
  assertion `each scheduled result has a minimum answer` and the
  `expect("provider admission left each call an answer")` before it,
  which divide what is left of the history by the batch's width. It also
  found that the fake's `Step::Together` emits one assistant message but
  sends its calls' MCP requests one at a time, each after the previous
  answer (`smith/tests/fake-claude/src/print.rs`), that `child_charter`'s
  `"llm":"fake"` is the program path (endpoint 0 names the program), and
  that the world hosts 4 relays and 8 fake invocations. None of this is
  proven; the rows decide.

Departures from the implementer's text, and why:

- **Status files:** you delete the three pieces named above, since this
  task closes them; touch no other design text.
- **The fix's files are found, not listed:** the crate and file of the
  bound your row reaches are yours, with their unit or domain tables, as
  below.

## Owns and may change

Owns:

- `smith/tests/agent-process/tests/suite/program_children.rs`: the table
  and its helpers.
- `smith/tests/agent-process/src/program_world.rs` (and submodules
  beside it): a declared room per run (5 for the rows; the world's other
  tests keep theirs), relay and fake instances sized from the room (main
  and each child a status check and a print run, a relay each), and a
  second LLM endpoint that names no program, served by the world's fake
  LLM and granted a credential by the scripted host, for direct
  children. The agent-process world's direct runs show how a fake LLM
  endpoint and a grant are set (`smith/tests/agent-process/src/world.rs`,
  `grants.rs`, `fake.rs`).
- `smith/tests/fake-claude/src/print.rs`, and the `Step` enum in
  `src/story.rs` (one variant, if needed; nothing else of that file).
- **The fix:** the file of the bound your first failing row reaches, in
  any smith step crate, and that crate's tests for the row at its tier.

Reader and neighbour files (edits allowed as stated):

- Other tests of `smith-agent-process-world` that the harness change
  makes name a new field or argument: the call or literal only.
- `smith/tests/agent-process/src/lib.rs`: a `mod` or `pub use` line.

Shared with task `program-child`, built beside you:

- In `program_world.rs`, `program-child` adds `parent: None,` after
  `effective_user: 1000,` in the `config.program = Some(…)` literal: do
  not edit that literal's `relay`, `entropy` or `effective_user` lines;
  if direct children need another `endpoints` value, edit that line
  alone.
- If your fix is in `smith-agent-service/src/service/program.rs` or
  `smith-protocol-program/src/backend/start.rs`, keep it away from
  `ProgramBackend`'s fields, `settings()`'s `backend::Settings` literal,
  the status check's `Launch` and `launch_spec`, which `program-child`
  edits; say so in the report.

Pre-approved options, in order, while no row fails:

1. The rows as written: program and direct children, the program
   asking the message's calls one at a time.
2. **All at once:** if `smith-protocol-program`'s MCP server marks
   `sub_agent` read-only, or if the rows as written all pass, add a
   `Step` variant whose calls' MCP requests are all sent before any
   answer is read, answers matched by id, as a program runs calls it
   judges safe together; run the rows with it, and keep it for the rows
   if it is what fails.
3. **Children's timing:** children that take several turns (a `read`,
   then their text), and one that answers before the program asks the
   message's last call, so refusals are answered while children run.
4. **The deployment's shape:** the direct children's endpoint on a
   provider other than main's, as the deployment's sub-agents ran on
   another provider's model.

If the run admits a message's calls one RPC at a time rather than at
the message (5.10's *Side by side*), and a child answers between two
calls, a row's counts may differ from the contracts': script the
children to answer only after the message's last call is asked, and say
so.

If no row fails after these: land the eight rows green (they pin the
room's answers), keep the three status pieces, and report `Done:` with a
first line that begins `no row reproduced the abort`, with what you
tried under `## Left`.

If the bound is in skein (a skein crate panics): stop with a
`Question:`, the bound and a sketch of skein's fix, the passing rows
pushed and the failing row described, not pushed.

## Steps

1. Read the inputs, the test file, `program_world.rs`'s `run`,
   `host_programs` and `child_charter`, and the fake's `Together` in
   `print.rs`.
2. Build the harness and the table; run `heavy cargo nextest run -p
   smith-agent-process-world --test suite -E
   'test(/program_children::/)'` (check the count). Read the failing
   row's log: the panic's message and location name the bound.
3. Write the row at the bound's tier and see it fail; fix the bound;
   run both green. Break the fix by hand once and see both fail.
4. Delete the status pieces.
5. Once, at the end: `cargo fmt`; `python3 -I -B smith/scripts/old_forms.py`;
   `heavy cargo clippy -p smith-agent-process-world -p smith-fake-claude
   -p <the fix's crate> --all-targets -- -D warnings`; `heavy cargo
   nextest run -p smith-agent-process-world -p smith-fake-claude -p <the
   fix's crate>`, and the fix's crate's direct dependents' focused tests
   if the fix changes what a shared item means (an answer it returns, an
   order it keeps); `python3 -I -B tools/citations/citations.py --base
   origin/main`. Commit the rows and the fix apart, each saying why (the
   fix's commit naming the bound and the panic), then push.

## Report

As the implementer's text says, and: the first failing row, the panic's
message and location, the bound and why a burst reached it, the fix,
the row at the bound's tier, what each row costs in CPU (the eight
together), the options above you took, and your implementation and
check minutes apart.
