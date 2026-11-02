# reference-m4: surveys as sub-agents on Claude Code

Milestone M4 of `temper/docs/design/draft/reference-case.execution.md`: on a program backend
(Claude Code), smith serves its `sub_agent` through the agent's own MCP server, each child a
program session of its own at the model and effort its LLM names, inspect-only children side by
side. smith alone; jig and temper change no code.

## The design this plan wrote

- `smith/docs/design/protocol/programs.md`: section 1 and 2 (sub-agents are smith's, each a
  program of its own), 5.10 (new: the served tool, inspect only, side by side, the child's
  command, its share and final turn, its end, its failures), 7 (the budget's overrun per live
  program; children read only), 8 (the fake's and the live suite's children), 9 (the open
  question on sub-agents settled and removed; one login's runs at once narrowed).
- `smith/docs/design/domain/run.md`, 5.3 (on a program backend); `protocol/events.md`, 3.3
  (`withheld` empty); `shell.md`, 5.2 (a sub-agent runs on a program only on main's account).
- Text only, to keep the trees agreeing: jig's `domain/accounts.md` and `domain/hosts.md`,
  temper's `domain/agent.md`.
- Statuses: Not yet entries in smith's `programs`, `run`, `events` and `shell` status files and
  `STATUS.md`; the drafts' Design lags for N10 removed from `reference-code-production.status.md`
  and `reference-case.status.md`.

## Decisions

- **Children on a program backend read only** (5.10). Claude Code decides alone which calls run
  together, so a writing child's lease (run.md, 5.3) would need the agent to hold the program's
  own calls at the prompt; the reference's surveys are inspect only (N10). A call asking modify
  or shell is invalid on its `tools` field. Writing children there are not designed.
- **Side by side by the domain, not the program:** the backend reports a message's calls as one
  completion, and the run opens its inspect-only children together whatever order the program
  sends their MCP requests in. No MCP annotation is needed.
- **One program `Backend` per child conversation,** kept by the service beside main's (the
  backend is already one lazy conversation, built from a token base and an entropy), and a new
  `Lower::Close` / `FromDomain::Close` so a child's program ends with its conversation.
- **A child runs on its LLM's grant:** main's program account gives a program child; an LLM on an
  endpoint with no program and a credential gives a direct child, as the service already routes.

## Parts and tasks

| Task | Part | Needs | Crates | Effort | Minutes |
|---|---|---|---|---|---|
| `close-lower` | close-lower | | smith-domain, smith-protocol-llm (arms in smith-agent-service) | high | 60 |
| `mcp-sub-agent` | mcp-sub-agent | | smith-protocol-program | high | 60 |
| `events-offer` | events-offer | | smith-protocol-events, one agent-process test | medium | 30 |
| `shell-reason` | shell-reason | | smith-local-shell | medium | 20 |
| `program-children` | program-children | close-lower, mcp-sub-agent, events-offer | smith-agent-service, smith-domain's `translate.rs`, the agent-process world | high | 180 |
| `side-by-side` | side-by-side | program-children | the agent-process world and its live probe; smith-protocol-program and the service for what its stories find | high | 120 |
| `exec-children` | exec-children | program-children | smith's `end_to_end` binary (test code only) | high | 75 |

Slices: 1 (the three program-side halves, each alone), 2 (`smith check`'s reason), 3 (one
program child, the meeting point), 4 (side by side, the final turn, a cancel), 5 (`smith exec`
on Claude Code with two inspect-only sub-agents side by side: the walk, `walk = true`).

All seven are released.

## Critical path

`close-lower` or `mcp-sub-agent` (about an hour each, side by side with `events-offer`), then
`program-children` (about three hours: the service's child backends, the switch that offers
`sub_agent` on a program, the first world story), then `side-by-side` and `exec-children` side by
side (about two hours). About six hours of implementers end to end.

## Overlaps with drafts beside this one

- live-smith-reds-2 (integration) touches `smith/tests/end-to-end/live.rs`, `support/live.rs`,
  `support/program_contract.rs`, `program_runs.rs` and `end_to_end.rs`. `exec-children` keeps to
  a new file, `program_children.rs`, and adds one `mod` line to `end_to_end.rs`: the one overlap
  left, a line beside the module list. The live check of two programs on one configuration
  directory goes into `smith/tests/agent-process/tests/live/program.rs` (`side-by-side`), which
  no draft beside this one is changing.
- reference-m8 (cut in flight) and the queue's drafts touch none of these files as far as the
  brief shows.

## Risks

- **The child's final turn on a program** reuses main's wind-down path (interrupt, note, refusals
  at the prompt); if that path is main's alone, `side-by-side` generalizes it (pre-approved).
- **The other trees' program runs** (`jig-channel-world`, `temper-real-loop-world`) now see
  `sub_agent` offered to a program main whose charter grants the family; none asserts it, but a
  byte measure could move. `program-children` runs them and asks if one goes red.
