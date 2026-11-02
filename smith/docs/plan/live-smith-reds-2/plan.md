# live-smith-reds-2: the plan

Three `live-smith` stories still red after plan live-smith-reds landed (request:
`inputs/request.md`), read from the after-landing logs of main 46e6e70d9b17 on hetzner8
(`~/src/fabrig/fab/after-landing/46e6e70d9b17-20261102T170025232765Z/`: the whole run
`live-smith.log`, the reruns `live-smith-1..3.log`), and for the refresh story from every
after-landing run of 11-01 and 11-02. The CLIs there: Claude Code 2.1.312, codex-cli 0.162.1. In
the same run every other live story passed, the ten agent-process probes on 2.1.312 among them.

| Test (`smith::live`) | Where it failed | Cause | Tasks |
|---|---|---|---|
| `claude_delivers_a_commit_in_place` | `live.rs:569` both runs, after an accepted, delivered run | the story's list requires `finish` in the first prompt's offer; a result-contract run is never offered `finish` (programs.md, 5.5). Not CLI drift: smith's own init check accepted 2.1.312's six built-ins | `claude-tools` (landed) |
| `a_chat_crosses_from_chatgpt_to_claude_then_back`, leg 1 | rerun: `live.rs:531`, `write` Written but `first.txt` missing | the prompt says "in repository repo"; one directory hides its name (domain/tools.md, 2), so `repo/first.txt` resolves to `repo/repo/first.txt`: the story's words, not the model or smith | `crossing-paths` |
| the same, leg 2 (Claude) | whole run: `live.rs:475`, `failure.class=Agent ended=Signal(9)`, turns 0, no `run.started`, about 17 s in | unknown: smith's spawned agent killed before admission, by the host (a channel fault, then SIGKILL) or from outside; the host's line that says which is dropped by the report | `report-lines` (says which next time), `crossing-twin` (the three legs offline through the product; a smith red is fixed there) |
| `a_run_refreshes_its_grant_at_the_issuer` | `live.rs:45` at every run since 11-02 10:49 (passed once, 10:49); generation equal before and after, while it rose about hourly between runs (3 at 11:51 to 7 at 16:30) | not found in code: the product refreshes at once on the 400-day lead unless the record holds an earliest refresh time ahead (skein oauth.md, 2) or no refresh token; the hourly rises point at the first | `refresh-twin` (the product path offline, both cases), `refresh-facts` (the live record's refresh facts in the failure) |

## Parts and tasks

- **claude-tools** (slice 1, landed): `support/program_contract.rs` checks the first prompt's
  offered tools (the six built-ins and `permission_prompt`, no `finish`, no `sub_agent`), naming
  the departing tool; the live story uses it; `program_runs.rs` holds its fake-backed twin.
- **report-lines** (slice 2, needs nothing): `live_report.rs` keeps the host's agent-failure line
  cut before its detail, and names an unknown notice by its kind (`not_contained`, today's
  `notice kind=unknown`).
- **crossing** (slice 3):
  - `crossing-paths` (needs `claude-tools`, which also edited `live.rs`): `CHANGE` and the
    crossing's prompt name the working directory; a missing file's report says when it was found
    under `repo/`; the tools world pins the prefixed write's resolution.
  - `crossing-twin` (needs nothing): `chat_crossing.rs` in `end_to_end`, three `smith exec --chat`
    legs Direct, program, Direct on the fake provider and the fake `claude`; a red is fixed in the
    one crate it points to.
- **refresh** (slice 4):
  - `refresh-twin` (needs nothing): in `end_to_end`'s `credentials.rs`, an exec on a raised lead
    refreshes a seeded current record at once (generation 1 to 2); with an earliest refresh time
    ahead it keeps it.
  - `refresh-facts` (needs nothing): `smith_local_shell::credentials::refresh_facts` (in
    `sign_in.rs`) and the story's failure line with the record's generation, refresh token, expiry
    and earliest refresh time before and after its run.

## Design written by this cut

- `smith/docs/design/testing.md`, 2.3: "Paths as the run tells them", "What the program run is
  offered", and in "A failure says why" a missing file found elsewhere, an unknown notice's kind,
  the refresh story's record facts and the agent-failure line without its detail; 8: "The refresh
  story and an earliest refresh time".
- `smith/docs/design/testing.status.md`, 2.3: the three reds as they stand at 46e6e70d9b17.

## Critical path

`crossing-twin` (120 min) alone; beside it `refresh-twin` (90), `refresh-facts` (75),
`report-lines` (45), and `crossing-paths` (50, after the landed `claude-tools`). Five tasks start
at once. `live.rs` has three writers, each on its own function: `claude-tools`'s
`program_contract` (landed), `crossing-paths`' `CHANGE`, crossing story and `assert_exec` file
loop, `refresh-facts`' refresh story body.

## After landing

The acceptance is the after-landing pair on a main holding these tasks:

- `claude_delivers_a_commit_in_place` should pass; a red now names the tool.
- The crossing: leg 1 should pass; if leg 2 is red again, its report says whether the host killed
  the agent (`; killed` and its fault) or not (outside smith: say so and leave the test), and
  `crossing-twin`'s result says whether smith shows it offline.
- The refresh story: if `refresh-twin` is green and the live facts show an earliest refresh time
  ahead, the story's expectation meets the issuer's rule (testing.md, 8): a follow-up decides
  what it checks then. If they show no refresh token, the shelf's record is the cause (outside
  smith). If they show neither, the product path differs live from the twin: a follow-up starts
  from the facts.

## Overlaps with drafts beside this one

- `sign-in-wait` (queue) owns `smith-local-shell`'s `credentials/product.rs`,
  `credentials/login/**`, `credentials/tests*`, `smith-local-shell/tests/**` and `shell.md`.
  `refresh-facts` adds its reader to `credentials/sign_in.rs` and one re-export in
  `credentials.rs`, outside those; `refresh-twin` may fix in smith-local-shell only outside them.
  The remaining risk is a neighbour edit of `credentials.rs`'s export lines by both.
- `citation-burndown` and `v0.1.0-ladder` touch no file this plan names.
