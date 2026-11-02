# live-smith-reds: the plan

Six standing reds of `live-smith` (request: `inputs/request.md`), read from the after-landing logs
of main da9027dbf5ae on hetzner8 (`~/src/fabrig/fab/after-landing/da9027dbf5ae-20261102T102506619199Z/`,
the whole run `live-smith.log` and the reruns `live-smith-<n>.log`). The CLI there is Claude Code
2.1.312 (codex-cli 0.162.1); the programs status says the live suite last passed on 2.1.311.

| Test | Where it failed (both runs unless said) | Cause | Task |
|---|---|---|---|
| `smith-agent-process-world::live program::a_live_server_with_a_non_object_schema_lists_no_tools` | `program.rs:388`: init's server entry gained `"source":"dynamic"` | CLI drift (2.1.312) | `claude-shapes` |
| `smith-agent-process-world::live program::a_live_thinking_turn_has_the_fakes_shape` | `program.rs:478`: a `thinking_delta`'s `estimated_tokens` is not a number | CLI drift: 2.1.312 coalesces deltas, `estimated_tokens: null` | `claude-shapes` |
| `smith::live a_run_refreshes_its_grant_at_the_issuer` | `live.rs:34`: generation did not rise; the run itself passed | the story's lead `34_560_000` means 400 days in seconds, but deployment durations are milliseconds: 9.6 hours | `refresh-lead` |
| `smith::live claude_delivers_a_commit_in_place` | `live.rs:344`, "run reports the expected resume state", after exit 0 | on a `program` account the run's detail records are the spawned agent's, in `<trace>.agent`; the live arguments name no trace and the harness reads only the product's stream | `claude-trace` |
| `smith::live a_chat_crosses_from_chatgpt_to_claude_then_back` | whole run `live.rs:334` (exec failed, 22 s); rerun `live.rs:375` (a leg's file content, 16 s; by elimination leg 1, ChatGPT) | the Claude leg has `claude-trace`'s cause; the rest is unknown until the live report names it; `crossing-program` pins the leg's `resumed` and the summary session's init | `claude-trace`, `crossing-program`, `live-report` |
| `smith::live a_second_run_resumes_the_chat_from_its_files` | `live.rs:334`, exec failed (~6 s for both runs) | unknown from code: run 1 parking, the contract changing on resume, or the provider refusing replayed reasoning; `resume-twin` runs the latter two offline | `resume-twin`, `live-report` |

The request's table attributes some of these lines differently; the logs above are the record.
Earlier after-landing runs (11-01) failed every live story at setup (`$SMITH_LIVE` unset), so this
is the first run with lent logins: none of the six has been green in the pair.

## Parts and tasks

- **claude-shapes** (slice 1, landed): the fake `claude` sends 2.1.312's init `source` and a
  coalesced `thinking_delta` with `estimated_tokens: null`; the two agent-process probes expect
  them.
- **live-report** (slice 2): every failing `smith::live` assertion carries a bounded report of
  typed facts (leg, exit, the run's typed end and failure, resume state, file relation,
  generations), built by `support/live_report.rs` to `notes/contracts.md` and checked offline in
  `live_configuration`. This is the request's "first make the failure say why".
- **refresh-lead** (slice 3, needs `live-report`): the refresh story's lead becomes
  `REFRESH_LEAD_MS` (400 days in milliseconds) in `support/live.rs`; an offline test resolves the
  story's settings and sees 400 days (red at 9.6 hours).
- **claude-trace** (slice 4, needs `refresh-lead`): each live invocation gets its own trace path;
  a `program` account's observer reads `<trace>.agent` as smith-bench does (`agent_trace`,
  `events()`); offline, `live_configuration` checks the arguments and the observer, and
  `end_to_end` pins that a program exec's `.agent` trace holds `run.started` (resumed false), the
  prompt and `finish`.
- **crossing-program** (slice 5, needs `claude-shapes`): the agent-process world asserts
  `run.started.resumed` on a fresh (false) and a carried-over (true) program run; a unit test of
  `compact::checked_init`; a live probe of the summary session's init on 2.1.312. A red is fixed in
  the one smith crate that sets `resumed`.
- **resume-twin** (slice 6, needs nothing): in the local-process world, a report chat resumed as a
  delivered change, and a resumed chat replaying its reasoning item verbatim. A red is fixed in the
  crate that holds it; green says the cause is live-only, for the live report to name.

## Critical path

`live-report` (90 min) → `refresh-lead` (40) → `claude-trace` (100): about 4 hours of
implementer time, the longest chain, all in the `smith` package's live harness, serialized because
one writer holds `smith/tests/end-to-end/live.rs` at a time. `crossing-program` (after the landed
`claude-shapes`) and `resume-twin` run beside it. The live check of every fix is the after-landing
pair on a main holding it, not an implementer's run.

## After landing

The acceptance is the after-landing pair. What this plan cannot settle from code: the chat
crossing's ChatGPT leg file content and its 22 s exec failure, and the resume story's failing run.
With `live-report` landed, their next red says why (exit, typed end and failure, file relation), so
a follow-up starts from a cause. If `resume-twin` or `crossing-program` is red offline, its task
fixes it here.

## Overlaps with drafts beside this one

None found: this plan owns smith's fake `claude`, the agent-process and local-process worlds, the
end-to-end live harness and two smith-protocol-program test files; `sign-in-wait` changes smith's
login shell (`smith-local-shell` credentials/login), the others temper. If `crossing-program` and
`resume-twin` both find a smith cause, they could meet in one domain crate; each is told to fix in
one crate and ask beyond it.

## For the designers

- `smith/docs/design/protocol/events.md`, 2: "It holds the run's records whichever kind of agent
  the local host runs" reads against "Where it goes" (the spawned agent's own trace) and against
  the code and its offline tests, which keep detail records in the agent's trace. `claude-trace`
  follows the code; the sentence wants its designer.
- `smith/docs/design/protocol/programs.status.md` says the live suite passes against Claude Code
  2.1.311; after this plan lands and the pair is green it is 2.1.312.
