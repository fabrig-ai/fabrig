# live-smith-reds: the plan

Six standing reds of `live-smith` (request: `inputs/request.md`), read from the after-landing logs
of main da9027dbf5ae on hetzner8 (`~/src/fabrig/fab/after-landing/da9027dbf5ae-20261102T102506619199Z/`,
the whole run `live-smith.log` and the reruns `live-smith-<n>.log`). The CLI there is Claude Code
2.1.312 (codex-cli 0.162.1); the programs status says the live suite last passed on 2.1.311.

| Test | Where it failed (both runs unless said) | Reading |
|---|---|---|
| `smith-agent-process-world::live program::a_live_server_with_a_non_object_schema_lists_no_tools` | `program.rs:388`, init's server entry gained `"source":"dynamic"` | CLI drift (2.1.312) |
| `smith-agent-process-world::live program::a_live_thinking_turn_has_the_fakes_shape` | `program.rs:478`, a `thinking_delta`'s `estimated_tokens` is not a number | CLI drift: 2.1.312 coalesces deltas, `estimated_tokens: null` |
| `smith::live claude_delivers_a_commit_in_place` | `live.rs:344`, "run reports the expected resume state", after exit 0 | under survey |
| `smith::live a_chat_crosses_from_chatgpt_to_claude_then_back` | whole run `live.rs:334` (exec failed, 22 s); rerun `live.rs:375` (a leg's file content, 16 s) | under survey |
| `smith::live a_second_run_resumes_the_chat_from_its_files` | `live.rs:334`, exec failed (~6 s) | under survey |
| `smith::live a_run_refreshes_its_grant_at_the_issuer` | `live.rs:34`, generation did not rise; the run itself passed | the story's lead `34_560_000` means 400 days in seconds, but deployment durations are milliseconds: 9.6 hours |

The request's table attributes some of these lines differently; the logs above are the record.
Earlier after-landing runs (11-01) failed every live story at setup (`$SMITH_LIVE` unset), so this
is the first run with lent logins: none of the six has been green in the pair.

## Parts

- **claude-shapes** (slice 1, released): the fake `claude` sends 2.1.312's init `source` and
  coalesced `thinking_delta` with `estimated_tokens: null`; the two agent-process probes expect
  them. Task `claude-shapes`.
- **live-report** (slice 2, released): every failing `smith::live` assertion carries a bounded
  report of typed facts (leg, exit, the run's typed end and failure, resume state, file relation,
  generations), built by `support/live_report.rs` to `notes/contracts.md` and checked offline in
  `live_configuration`. Task `live-report`. The request's "first make the failure say why".
- **refresh-lead** (slice 3, released, needs `live-report`): the refresh story's lead becomes
  `REFRESH_LEAD_MS` (400 days in milliseconds) in `support/live.rs`; an offline test in
  `live_configuration` resolves the story's settings and sees 400 days (red at 9.6 hours).
- The causes of the three remaining end-to-end reds follow the surveys.

## Critical path

`claude-shapes` and `live-report` run side by side; neither needs the other. A cause task that
edits `smith/tests/end-to-end/live.rs` needs `live-report`: `refresh-lead` does. The live check of every fix is the after-landing
pair on a main holding it, not an implementer's run.

## Overlaps with drafts beside this one

None so far: `claude-shapes` owns smith's fake `claude`, the agent-process live probes and two
smith-protocol-program test files; `live-report` owns `smith/tests/end-to-end/live.rs` and its
offline controls; `sign-in-wait` changes smith's login shell, the others temper.
