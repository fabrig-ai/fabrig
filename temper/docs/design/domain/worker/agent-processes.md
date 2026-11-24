# The worker's agent processes

This document holds the worker's agent processes, split from [`README.md`](README.md).

## 1. Agent processes

The agent child is smith's host domain, `smith-host-domain` (smith's
`domain/host.md`, section 4): jig's agent processes (jig's
`domain/hosts/README.md`, 5.1). One process per run, whose process tree is the
run's containment boundary (`README.md`, section 8); one channel over its pipes,
whose rules smith's host domain enforces; a watchdog on progress and a
bound on wall time; cancel, then kill; the slot back only once the
process has exited and its tree is empty. temper's choices:

- **Agent processes, always.** Every run is a process of its own, also
  when the engine and the worker share a process
  (`../../protocol/service-process.md`, section 1); `smith-inline-agent` is not used
  (`../../protocol/llm.md`, section 7).
- **temper's binary is the agent:** the box's `temper-box internal agent`,
  or a deployment's `temper-agent` (`../../protocol/binaries.md`, 3, and `../../protocol/service-routing.md`, 2
  ), smith's agent shell. temper builds no code into it.
- **Pinned at the worker's start,** so an agent is always its worker's
  build and smith's channel never spans two (`../../draft/archive/redeploys.md`,
  4.4). As it starts, before its loop, the worker writes the agent's
  configuration from its engine's figures, checks it and takes its
  bounds from it (`../../protocol/configuration/worker.md`, section 2;
  `../../protocol/configuration/figures.md`, section 6), then writes
  those same bytes, and a copy of its derived agent program,
  into `agent/` under its root directory (`../../protocol/git.md`,
  section 2), afresh at each start, the previous copies removed first.
  The program's copy keeps its file name and is executable by the
  worker's user alone; the configuration's is private. Every run's agent
  and every model list's launch spawns the copy until the worker exits:
  the program is the copy's path, and each argument equal to the
  configured configuration's path names the copy's instead, so the box's
  `internal agent AGENT.json` and a deployment's `AGENT.json` both read
  the pinned file. A redeploy that replaces the program or edits the
  configuration changes what the next start pins, never a running
  worker's agents. What the configuration names in turn, a program
  source's executable and directory, trust files, a trace's path, stays
  where it is: those are the host's, not the build's. A start that
  cannot pin, the program unreadable or the root full, is refused,
  naming the file.
- **An agent goes with its worker** (`../../draft/archive/redeploys.md`, 4.4).
  A worker's stop ends within its stop bound, every agent's process
  group killed at it at the latest (`README.md`, section 2). A worker that dies
  closes its agents' pipes, and each agent, hearing its channel end,
  cancels its run, ending its commands' groups, then exits (smith's
  `domain/host.md`, section 4). What an agent still running would leave,
  a wedged one or a command outside its groups, a deployment's service
  manager ends with the worker's control group
  (`../../protocol/service-routing.md`, section 2); a tree contained by io, proved
  empty, is smith's to give (smith's `domain/host.md`, section 4, whose
  status records it).
- **No credentials in its environment.** The tokens its LLM calls use
  come as grants on its channel (`../../protocol/credentials-llm-accounts.md`,
  section 1); git's never reach it.
- **The repository's checks are a long operation** the run reports, so
  the watchdog stretches to their span: a two-hour test suite is not a
  hang.
- **A check's build directory is held for the whole check.** A
  repository's check builds and then runs what it built, after cargo has
  exited. A deployment whose cargo on the agent's `PATH` lends a build
  directory only while a cargo command runs could reclaim it between the
  two, and a test's executable would vanish under the check. The worker
  gives a check nothing but the agent's `environment`
  (`../../protocol/configuration/worker.md`, section 2), so the hold is the
  check's and that environment's: one hold for the check, taken by its
  outermost entry point from before its first build to its last test,
  released at its end or by its process's death. fabrig's own check takes
  the hold its environment names in `TIERS_HOLD` (`tools/tiers/design.md`,
  section 3; `tools/pre-pr/design.md`, section 4), and a deployment that
  lends build directories names its hold there.
- **Its build places too, and its bound counts only them.** A deployment
  whose cargo admits builds to a limited number of places, shared by every
  workspace on the machine, makes a check wait for one per cargo command,
  behind other runs' builds and tests, and that wait counted against the
  checks' bound. Its hold takes the check's place once, before its first
  build, and keeps it to the check's end, so that the check's cargo
  commands wait for none; while the hold waits it says so, the check tells
  smith, and the wait is left out of the bound (smith's
  `smith/docs/design/domain/run-delivery.md`, 1.1), whose figure stays the
  deployment's. A check queued behind others is bounded by its call's
  deadline and the run's end alone until it holds its places.
- **The base's build has a target of its own.** A deployment's cargo
  that gives each workspace one target, by the command's path, would
  give a check's base worktree (`<git dir>/temper-pre-pr/base`) the
  head's target, and cargo's freshness by file times can then take one
  tree's outputs for the other's. The check sets `CARGO_TARGET_DIR` for
  every program it starts there (`tools/pre-pr/design.md`, section 5),
  and such a cargo keeps that setting, or keys the worktree to a target
  of its own.
- **What a test writes reaches the checkout.** A run's commands and
  checks run in place, in the workspace's checkout (smith's
  `protocol/agent.md`, section 2), and its delivery takes that tree, so a
  file a test writes under its package, a recorded fixture re-recorded by
  `SKEIN_CODEGEN_WRITE=1` among them, is in the change as any edit is,
  once the test finds its package when it runs
  (skein/docs/foundation/testing-strategy.md, section 8, *A test reads its
  paths when it runs*). A path compiled in names the checkout the
  executable was built from, which a build cache shared between
  checkouts, or a target directory kept apart from the checkout, can make
  another one's: the file goes there, and never reaches the change. A
  merge job re-records the recordings it knows in its own job's checkout
  and delivers them itself (`../../protocol/tooling.md`, 4.1).
- **Its limits** are smith's host limits, and the bounds the hello
  states, from the agent's configuration (`../../protocol/configuration/worker.md`,
  section 2). The one number that isn't the agent's is how many agents
  the host holds at once: a run is one agent process, so that number is
  the worker's slots, set where the worker composes its limits, and
  nothing in the agent's configuration raises it. Each of the host's
  other limits is one agent's, whatever the slots; an agent process
  holds one run.
- **Its reply bound,** in the hello's bounds: the agent's
  `host_reply_text`, its `agent.json` `profile.declared.tool_payload`,
  the same figure as the worker's `Limits::reply_bytes` (`hosted-runs.md`, 1.3), so that
  the engine sizes a task read and renders an answer for this worker's
  runs within what its agent takes (`../routing.md`, 2; jig's
  `domain/hosts/README.md`, section 2). Every temper worker names it, the
  engine's in-process local agent among them.
- **What rises with its slots:** the worker's `hub.slots`, the engine's
  `fleet.slots` and the checkout's `checkout.workspaces` are each at
  least the slots, and `git_component.operations` at least the
  workspaces. A start refuses a worker past any of them it can see,
  naming the limit to raise. An engine alone can't see a worker's slots:
  it uses them up to `fleet.slots` and turns away a hello listing more
  runs than that (jig's `domain/engine/fleet.md`).
- **Its program endpoints,** in the hello's bounds: the endpoint numbers
  the agent's configuration serves by a program, its `program_endpoints`,
  as jig's reference launcher declares them (jig's
  `domain/hosts/charters.md`, 1). temper's stock agent serves none, an empty
  list that proves no endpoint is a program; a chat whose ordinary
  source is one is refused before it starts, `HostRefused` in its
  status (`../agent/charter.md`, 5).

## 2. Open questions

- **A worker past what no document raises.** A standalone start counts its
  slots and git operations against the process's sockets, a constant
  (`../../protocol/configuration/limits.md`), and the link's caps are `u32`s
  that grow with the slots, so past them a start's refusal names a limit no
  one can raise, or none. A refusal saying what to lower, a key for the
  sockets or wider caps would close it. Default: as they stand, until a
  deployment asks for more than they hold. The user's to decide.

- **What a deployment's operators may read of a run's prompts.** A run's
  prompts hold repository text and anything an agent read, and a real
  design of keeping them starts from who may read them
  (`../../draft/dogfood-hacks.md`, 2.12 and 2.13). Default: nothing past what an
  agent's document asks its trace to keep.
