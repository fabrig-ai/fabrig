# subagent-burst: plan

The request (`inputs/request.md`): a program-backed run survives a burst
of sub-agents. On a deployment at `94d1f6ce16`, a Claude Code main asking
five or six `sub_agent` calls in one message aborted its agent; the
agent's `claude` child and its relay were left running; and the run's
detail had lost the panic's message to the backtrace. Four things are
wanted, ranked, within four tasks: the failing test and its fix; the
panic's message kept; a program's children ending with their agent; the
design saying so. All four are in the plan, in four tasks; nothing is
dropped.

The cut wrote the design first (item 4):

- `smith/docs/design/domain/run.md`, 5.3, a new bullet *The room*, and
  `smith/docs/design/protocol/limits.md`, section 6, a `sub_agent` row:
  a call past the room is refused as too many (busy where the agent's
  tables are full), call by call, and the parent goes on.
- `skein/docs/design/shell.md`, section 6: a panic's report closes with
  its `panicked at` line and message after the backtrace; section 13: a
  child exec'd through `exec_dying_with_parent` is the exception to *A
  crash closes nothing*.
- `smith/docs/design/domain/host.md`, section 4: the kept tail therefore
  holds the panic's line and message.
- `smith/docs/design/protocol/programs.md`, sections 2 and 5.1 (*Started
  to die with its agent*), and `smith/docs/design/shell.md`, 2.2: the
  agent starts each program process through its own binary's
  `--program-child` mode, which execs it with skein's
  `exec_dying_with_parent` and Kill.
- Their status files' **Not yet** entries, and both trees' `STATUS.md`
  summaries; each task deletes the entries it closes.

## Decisions

- **The burst's cause is found by the test, not by the cut.** A
  read-only survey found none of the request's three candidates reached
  by a burst's width alone, and named a fourth suspect (the session's
  batch reservation in `smith-domain-session/src/session/history.rs`).
  The task's rows decide; its spec lists, in order, the axes that make
  the world closer to the deployment if the first rows pass (calls' MCP
  requests sent at once, children's timing, a direct endpoint on another
  provider), and what to do if nothing fails or the bound is skein's.
- **The rows run at a room of 5,** smith's `standard` profile's (main and
  four children), so N = 2, 4, 5, 6 are two, exactly the room, one past
  and two past, matching the deployment's five or six. The world sets
  its own declared room; no production figure moves.
- **The panic's message is kept by the reporter, not by every keeper.**
  The kept detail is re-cut by a tail at each layer:
  `smith-host-protocol`, `smith-host-domain`, `jig-domain-host`,
  `temper-worker-domain` and temper's serve log. Composing "the panic's
  head plus the tail" in each keeper would take a skein mechanism and a
  task in each of smith, jig and temper, past the four-task bound, and
  item 3 would drop. `skein-fail-stop` closing its report with the same
  line and message again puts them in every tail, in every tree, within
  each keeper's own bound, with no protocol or stored change: one skein
  task, and one smith row of the existing detail test.
- **Children die with their agent by `exec_dying_with_parent`,** which
  skein's shell design already supports (section 3) and temper's
  `internal forge-child` already uses. Ending the program's group when
  the host reaps the agent would need the host to learn the program's
  group, which io keeps from it: new reporting across the host channel.
  The mode's parent-death signal is Kill: the agent is gone, nothing
  waits for a graceful stop, and the program's spend must end. On macOS,
  which has no parent-death signal, the agent starts the program directly.
- **The simulated program world keeps the direct launch** (`parent:
  None`), as macOS does: its spawns are dispatched by the program's path,
  and the trampoline's argv is pinned by a unit table in
  `smith-protocol-program`. The real chain, agent through the mode to the
  fake and its relay, is proven by one row in the shipped product's
  `end_to_end` tests (focused), which kills the agent and checks that
  nothing remains.

## Parts and tasks

- **burst:** task `burst`, high, about 240 minutes, no needs. Eight rows
  in `program_children.rs` (N = 2, 4, 5, 6; program and direct
  children) at a room of 5, the harness to run them (room, instances, a
  direct endpoint), seen failing; the fix at the bound the first
  failing row names, with a row at that bound's tier.
- **detail:** task `report-end`, medium, about 40 minutes, no needs:
  `skein-fail-stop` closes its report with its line and message; its
  test world checks the end and a 1,024-byte tail. Task `detail-row`,
  medium, about 30 minutes, no needs (built to the contracts note): a row
  of `smith-host-protocol`'s kept-detail table, a report in that form
  with a 4 KiB backtrace kept at 1,024 bytes.
- **lifetime:** task `program-child`, high, about 180 minutes, no needs:
  the `--program-child` mode, the launches through it, unit tables for
  the launch and the mode's argv, and the end-to-end abort row.

Slices 1, 2 and 3 hold burst, detail and lifetime. No walk.

## Critical path

All four tasks start at the first push. The path is `burst`, about 240
implementer-minutes, its length unknown until its rows find the cause;
`program-child` runs beside it, about 180.

## Overlaps

- **Within the plan:** `burst` and `program-child` both touch
  `smith/tests/agent-process/src/program_world.rs` (`program-child` one
  line, `parent: None`, in the `ProgramBackend` literal; each spec names
  the lines the other keeps), and `burst`'s fix may land in
  `smith-agent-service/src/service/program.rs` or
  `smith-protocol-program/src/backend/start.rs`, which `program-child`
  edits; its spec keeps it off the struct, the settings literal and the
  launch. `smith/docs/design/STATUS.md`'s summary is edited by three
  tasks, each deleting its own phrase.
- **With the drafts beside this one:** none known. `look-round-20` and
  `tui-launchpad` change temper's client and `skein-ui-screen`; this plan
  changes none of their files. `ui-ask-m3a`, `reference-walks`,
  `v0.1.0-ladder`, `configuring-walks` and `post-landing-probes` list no
  files yet.
