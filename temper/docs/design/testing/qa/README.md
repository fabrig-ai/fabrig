# Landed QA

Provisional, 2026-11-12, from `../../draft/landed-qa.md`, milestone Q1
of `../../draft/landed-qa.execution.md`. QA after landing, beside the
deployment: code watches main, keeps the ledger, keeps and deletes builds
and starts agents; agents triage each landing and, from Q3, run each
pass. This document is the tool, its home, its loop and its lines; its
parts are documents of their own beside it. Q1 designs landings, triage,
kept builds and the ledger that holds them; the setups, the drivers and
the passes come with Q2 and Q3, each adding its documents here.

## 1. In one page

- **One tool,** `temper-qa` (section 2), configured by the deployment in
  a home directory of its own (section 3). It builds nothing, takes none
  of the deployment's worker slots and runs on the machine that holds the
  places (`../../draft/landed-qa.md`, 2).
- **No coordinator** (section 4): one loop, `temper-qa run`, moves
  landings, cases and builds through their states on what it sees: main
  moving, a build kept, an agent's session ending, a bound passing. Each
  agent session is a leaf: it records through the ledger's commands and
  ends.
- **The ledger** (`ledger.md`): one JSON document per project, the only
  record of the pass's state, written under a lock by the loop and by
  the agents' commands alike. Its rules refuse what does not fit.
- **The watcher** (`landings.md`): main's ref read on every tick; every
  commit a move gained listed, so a goal's merge carried off the
  first-parent line is seen, and seen once.
- **Kept builds** (`builds.md`): the deployment builds the box and the
  two web bundles at a landed tip and hands them to `temper-qa build
  keep`, which checks every executable's version, writes the manifest
  and records the build. The loop deletes a build once no pass can need
  it.
- **Triage** (`triage.md`): one landing at a time, in main's order, an
  agent session at a lower effort, given the landing's draft, diff and
  the surfaces; it answers no QA or cases, and a case for a page a queued
  case covers joins it.
- **Its own faults are lines for its owner** (section 5), never findings
  against temper.

## 2. The tool

- **`temper-qa`,** its package `temper/bin/temper-qa/` and its crate
  `temper/crates/temper-qa-tool/`, laid out, named, answered and tested
  as `../tools.md` has every tool (sections 3, 4 and 6 there). Its line:
  `QA after landing: main's landings triaged into cases, and builds kept
  for them.`
- **Its commands,** each `Set::Internal`, Q1's:

  | Command | Whose | Designed in |
  |---|---|---|
  | `temper-qa run` | the deployment's service | section 4 |
  | `temper-qa build keep COMMIT DIR` | the deployment's after-landing work | `builds.md`, 3 |
  | `temper-qa triage none WHY` | a triage session's agent | `triage.md`, 4 |
  | `temper-qa triage case FILE` | a triage session's agent | `triage.md`, 4 |
  | `temper-qa show` | anyone | `ledger.md`, 6 |
  | `temper-qa lines` | anyone | `ledger.md`, 6 |

- **Its variables,** internal, read through its table
  (`../tools.md`, 4.3):
  - `TEMPER_QA_HOME`: its home (section 3), required by every command
    but the front's;
  - `TEMPER_QA_SESSION`: the agent session a command records for, set by
    the loop for the agent it starts (`triage.md`, 5), required by the
    session's commands.
- **Its answers:** the front's and refused arguments as `../tools.md`,
  4.2 has them, exit 2; a command that ran, exit 0; a refusal of the
  ledger's (`ledger.md`, 5), a bad configuration or a failure, `temper-qa:
  WHY` on stderr, exit 1, with nothing recorded.
- **Its reference documents,** its arguments' and its environment's in
  `../../cli/`, rendered from its table (`../tools.md`, 4.4).

## 3. Its home and configuration

### 3.1 The home

`TEMPER_QA_HOME` names a directory the deployment makes and owns:

```
qa.json            its configuration (3.2), the deployment's
ledger.json        the ledger (ledger.md, 2)
ledger.lock        the ledger's lock (ledger.md, 3)
run.lock           held by the one running loop (section 4)
builds/COMMIT/     each kept build, its files and its manifest (builds.md), unless qa.json names another directory
sessions/N/        each agent session's brief, output and, while it runs, its tree (triage.md, 2)
```

### 3.2 `qa.json`

One JSON object, read at every command's start; a key it does not know
is refused by name, a missing required key named, and a relative path
taken from the home.

```json
{
  "owner": "AREA",
  "git": { "directory": "PATH", "branch": "main", "fetch": "REMOTE" },
  "agent": { "command": ["PROGRAM", "ARG", "{effort}"], "triage": "EFFORT" },
  "bounds": { "poll": 60, "triage": 900, "build_wait": 21600 },
  "builds": "builds",
  "notify": ["PROGRAM", "ARG"]
}
```

- **`owner`,** required: the area that runs the pass, which its own
  faults' lines name (section 5).
- **`git`:** `directory`, required, a git repository of the project,
  bare or not; `branch`, main's name, `main` when absent; `fetch`, a
  remote of that repository, absent when the repository is main's own
  (`landings.md`, 2).
- **`agent`:** `command`, required, the coding agent's command line as
  an argument list, in which every `{effort}` is replaced by the
  session's effort; `triage`, required, triage's effort, as the command
  takes it (`../../draft/landed-qa.md`, 2). Q3 adds the pass's.
- **`bounds`,** each in seconds: `poll`, between two ticks of the loop,
  60 when absent; `triage`, a triage session's time, 900 when absent;
  `build_wait`, how long a case may wait for a kept build that holds its
  landings before it is a line, 21600 when absent (`ledger.md`, 4.7).
- **`builds`:** where kept builds are, `builds` when absent.
- **`notify`:** a command each new line is given to (section 5), none
  when absent.

The tokens of a session are bounded by the agent's command line itself,
which the deployment writes: the tool bounds its time alone, and records
the tokens the command reports (`triage.md`, 5).

## 4. The loop

`temper-qa run` is the deployment's service: one per home, holding
`run.lock` for its life, so a second is refused, `another temper-qa run
holds HOME`. At its start it ends every session the ledger still has
running as lost (`triage.md`, 6), its process gone with the loop that
started it, and prunes their trees.

- **A tick,** every `bounds.poll` seconds, and at once when the session
  it started ends, takes these steps in order, each under the ledger's
  lock and saved before the next:
  1. **Watch:** main's ref read; landings recorded; kept builds the head
     now holds given their landings and order (`landings.md`, 3 to 5).
  2. **Triage:** a session whose agent has ended, or passed its bound,
     ended (`triage.md`, 6); with none running, a session started for
     the next landing (`triage.md`, 2).
  3. **Waiting cases:** each case past `bounds.build_wait` with no kept
     build for it becomes a line (`ledger.md`, 4.7).
  4. **Builds:** each build the ledger's rule lets go deleted
     (`builds.md`, 5).
  5. **Lines:** each new line given to `notify` (section 5).
- **Between ticks** the agents' commands take the lock and record; the
  loop reads what they recorded at its next step. Nothing the loop does
  waits on an agent's words: an agent's end is its process's exit.
- **A failed step** (git refusing, a directory not removed) is said on
  the loop's stderr and tried again at the next tick; one that has
  failed for an hour is a line, once, until it next succeeds.
- **Its world:** the clock, git, the agent's processes and the home's
  files, each behind a face of the crate's (section 6).

## 5. Lines for the owner

- **A line** is one sentence about the pass's own state, for the area
  `owner` names, never a finding against temper
  (`../../draft/landed-qa.md`, 2). Q1's: a landing whose triage recorded
  nothing twice (`triage.md`, 6), a case waiting past its bound for a
  build (`ledger.md`, 4.7) and a step failing for an hour (section 4).
  Q3 adds a case blocked twice.
- **Kept in the ledger** (`ledger.md`, 2.6), each once, with its time
  and area, and printed by `temper-qa lines`.
- **Given to `notify`** where `qa.json` names one: the command runs once
  per new line, with `AREA: WORDS` and a newline on its stdin, and the
  line is marked given whatever it exits. The deployment's command
  carries it to the area: a message, a chat, a mail. A command that
  fails is said on the loop's stderr, never a line.

## 6. Its tests

All in the focused and fuzzy suites, in the crate's `tests/suite/` and
its binary's table test (`../tools.md`, 6), each failing first:

- **The ledger's rules over values** (`ledger.md`, 4): built in memory,
  driven through its operations, its records checked; the deletion
  rule's invariants under random sequences of events, in the fuzzy
  suite.
- **The commands through the binary,** in a scratch home with its own
  `qa.json`: the agents' commands and their refusals, `show` and
  `lines`.
- **The watcher and the loop over a fake world:** a git of commits in
  memory, an agent that records through the ledger's operations or ends
  having recorded nothing, a clock the test moves, builds as directories
  in the scratch home.
- **`build keep`** on a staging directory in the scratch home whose
  executables are small scripts printing a version.

None starts a browser, a box, a real agent or the network; none runs
`git` but the loop's real face, which the walk after landing exercises
(`../../draft/landed-qa.execution.md`, 1).

## 7. Setting it up

The deployment's, after Q1 lands, on the machine that holds the places:

- a home with its `qa.json`, a repository of the project for `git`,
  and an agent command line, with its token bound, for `agent`;
- `temper-qa run` as a service, with `TEMPER_QA_HOME`;
- its after-landing work building at each landed tip and calling `temper-qa
  build keep` (`builds.md`, 2).

## 8. Later

- **Passes, setups, drivers and findings:** Q2 and Q3
  (`../../draft/landed-qa.execution.md`), each with its documents here
  and its records in the ledger.
- **More places** when passes queue (`../../draft/landed-qa.md`, 12).

## 9. Open questions

- **A line's medium:** the deployment's `notify` command carries it
  today; temper's routing by area (`../../draft/standing-roles.md`,
  section 4) may take it once built.
