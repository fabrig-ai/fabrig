# live-smith-reds-2: the plan

Three `live-smith` stories still red after plan live-smith-reds landed (request:
`inputs/request.md`), read from the after-landing logs of main 46e6e70d9b17 on hetzner8
(`~/src/fabrig/fab/after-landing/46e6e70d9b17-20261102T170025232765Z/`: the whole run
`live-smith.log`, the reruns `live-smith-1..3.log`). The CLIs there: Claude Code 2.1.312,
codex-cli 0.162.1. In the same run every other live story passed, the ten agent-process probes on
2.1.312 among them.

| Test (`smith::live`) | Where it failed | Cause | Task |
|---|---|---|---|
| `claude_delivers_a_commit_in_place` | `live.rs:569` both runs, after an accepted, delivered run | the story's list requires `finish` in the first prompt's offer; a result-contract run is never offered `finish` (programs.md, 5.5); not CLI drift: smith's init check accepted 2.1.312's built-ins | `claude-tools` |
| `a_run_refreshes_its_grant_at_the_issuer` | `live.rs:45` both runs: generation 7 before and after, run completed | being surveyed | (next batch) |
| `a_chat_crosses_from_chatgpt_to_claude_then_back` | whole run: leg 2 (Claude) `live.rs:475`, `failure.class=Agent ended=Signal(9)`, turns 0, no `run.started`; rerun: leg 1 (ChatGPT) `live.rs:531`, `write` Written but `repo/first.txt` missing | being surveyed | (next batch) |

## Parts and tasks

- **claude-tools** (slice 1, needs nothing): a shared check, `support/program_contract.rs`,
  of the first prompt's offered tools (the six built-ins and `permission_prompt`, no `finish`, no
  `sub_agent`), naming the departing tool; the live story uses it; an offline fake-backed twin in
  `end_to_end` (`program_runs.rs`) runs it on the live arguments.

## Critical path

So far: `claude-tools` (60 min), alone.

## Overlaps with drafts beside this one

None so far: `claude-tools` owns smith's end-to-end live harness files only. `sign-in-wait`
(queue) owns `smith-local-shell`'s `credentials/product.rs`, `credentials/login/**` and
`shell.md`; `citation-burndown` and `v0.1.0-ladder` touch no smith test file this plan names.
