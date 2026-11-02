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

| Task | Part | Needs | Crates | Effort |
|---|---|---|---|---|
| `close-lower` | close-lower | | smith-domain, smith-protocol-llm (arms in smith-agent-service) | high |
| `mcp-sub-agent` | mcp-sub-agent | | smith-protocol-program | high |
| `events-offer` | events-offer | | smith-protocol-events, one agent-process test | medium |
| `shell-reason` | shell-reason | | smith-local-shell | medium |

Released so far: the four above, all from the trunk's base.

## Critical path

`close-lower` and `mcp-sub-agent`, then the service's child backends with the switch that offers
`sub_agent` on a program, then the worlds' stories (being written).

## Overlaps with drafts beside this one

- live-smith-reds-2 (integration) touches `smith/tests/end-to-end/live.rs`, `support/live.rs`,
  `support/program_contract.rs`, `program_runs.rs` and `end_to_end.rs`: no task released so far
  touches them.
