# Landed QA

Provisional, 2026-11-12, revised 2026-11-16, from
`../../draft/archive/landed-qa.md`, milestones Q1, Q3, Q4, Q6, Q7 and Q8 of
`../../draft/archive/landed-qa.execution.md`. QA after landing, beside
the deployment: code watches main, keeps the ledger, keeps and deletes
builds, restores setups, files findings and starts agents; agents triage
each landing and run each pass. This document is the tool, its home, its
loop and its lines; its parts are documents of their own beside it.
Landings, triage, kept builds and the ledger are Q1's; places and their
copies (`places.md`), passes (`passes.md`) and findings with their goal
(`findings.md`) are Q3's, on Q2's drive session and recipes
(`../../client/drive.md`, `../../client/recipe.md`); the TUI's cases, the
setups' catalogue with its twins (`setups.md`), places' own ports and
the figures bounds are tuned from are Q4's; the `ctl` surface, run
first, and places of a deployment's kind (`deployment.md`), with the
first deployment setup, are Q6's; the web client and the TUI at a
deployment's place, with a deployment setup of web and tui lines, are
Q7's; the two kinds of place as one framework (`kinds.md`) are Q8's.

## 1. In one page

- **One tool,** `temper-qa` (section 2), configured by the deployment in
  a home directory of its own (section 3). It builds nothing, takes none
  of the deployment's worker slots and runs on the machine that holds the
  places (`../../draft/archive/landed-qa.md`, 2).
- **No coordinator** (section 4): one loop, `temper-qa run`, moves
  landings, cases, passes and builds through their states on what it
  sees: main moving, a build kept, an agent's session ending, a goal's
  landing, a bound passing. Each agent session is a leaf: it records
  through the ledger's commands and ends.
- **The ledger** (`ledger.md`): one JSON document per project, the only
  record of the pass's state, written under a lock by the loop and by
  the agents' commands alike. Its rules refuse what does not fit.
- **The watcher** (`landings.md`): main's ref read on every tick; every
  commit a move gained listed, so a goal's merge carried off the
  first-parent line is seen, and seen once.
- **Kept builds** (`builds.md`): the deployment builds the box, its
  client, the drive tool, the deployment's engine, worker and agent, and
  the web bundle at a landed tip and
  hands them to `temper-qa build keep`, which checks every temper
  executable's version, writes the manifest and records the build. The
  loop deletes a build once no pass can need it.
- **Triage** (`triage.md`): one landing at a time, in main's order, an
  agent session at a lower effort, given the landing's draft, diff and
  the surfaces, ctl's first; it answers no QA, cases or a gap, and a
  case for a page or ctl group a queued case covers joins it.
- **Places and copies** (`places.md`): a place is a fixed directory with
  its own `bin/`, where one box runs, or one deployment's processes
  (`deployment.md`), the two kinds behind one interface, differing only
  by their values (`kinds.md`); a pass installs its build there,
  restores each case's setup from its copy, made again from its recipe
  when the build refuses it, and drives the box a line at a time through
  a holder the agent's one-shot commands speak to.
- **Passes** (`passes.md`): a pass takes the newest reached kept build
  holding queued cases and every case it holds, in one step; one agent
  session covers them all, recording each outcome and finding as it
  goes, its ctl cases first in a session with no browser; with failed
  cases it files its request and waits, and resumes on a build holding
  its goal's landing.
- **Findings** (`findings.md`): each with its setup and the lines that
  reproduce it, linked to an open one it matches; a pass's open findings
  go back as one request, put on main by the deployment's command, and
  one goal started on it; two goals at most, then handed on.
- **Gaps** (`gaps.md`): what QA should judge and cannot yet, recorded by
  triage or a pass, filed as documents with no goal, and closed by a
  landing carrying `QA-gap: N`.
- **Its own faults are lines for its owner** (section 5), never findings
  against temper.

## 2. The tool

- **`temper-qa`,** its package `temper/bin/temper-qa/` and its crate
  `temper/crates/temper-qa-tool/`, laid out, named, answered and tested
  as `../tools.md` has every tool (sections 3, 4 and 6 there). Its line:
  `QA after landing: main's landings triaged into cases, and passes run
  on builds kept for them.`
- **Its commands,** each `Set::Internal`:

  | Command | Whose | Designed in |
  |---|---|---|
  | `temper-qa run` | the deployment's service | section 4 |
  | `temper-qa wake` | the deployment's after-landing work | section 4 |
  | `temper-qa build keep COMMIT DIR` | the deployment's after-landing work | `builds.md`, 3 |
  | `temper-qa triage none WHY` | a triage session's agent | `triage.md`, 4 |
  | `temper-qa triage gap KIND WHY` | a triage session's agent | `gaps.md`, 2 |
  | `temper-qa pass gap CASE KIND WHY` | a pass session's agent | `gaps.md`, 3 |
  | `temper-qa gaps [--open]` | anyone | `gaps.md`, 5 |
  | `temper-qa triage case FILE` | a triage session's agent | `triage.md`, 4 |
  | `temper-qa pass setup NAME` | a pass session's agent | `places.md`, 4 |
  | `temper-qa pass open [--ctl]` | a pass session's agent | `places.md`, 5 |
  | `temper-qa pass line LINE` | a pass session's agent | `places.md`, 5 |
  | `temper-qa pass close` | a pass session's agent | `places.md`, 5 |
  | `temper-qa pass recipe NAME FILE` | a pass session's agent | `places.md`, 6 |
  | `temper-qa pass outcome CASE RESULT NOTES [--finding N]...` | a pass session's agent | `passes.md`, 5 |
  | `temper-qa pass finding FILE` | a pass session's agent | `findings.md`, 2 |
  | `temper-qa pass check FINDING RESULT NOTES` | a resumed pass session's agent | `passes.md`, 7 |
  | `temper-qa pass note WORDS` | a pass session's agent | `passes.md`, 5 |
  | `temper-qa pass hold` | `pass open`, as its holder | `places.md`, 5 |
  | `temper-qa show` | anyone | `ledger.md`, 6 |
  | `temper-qa lines` | anyone | `ledger.md`, 6 |
  | `temper-qa report PASS` | anyone | `passes.md`, 8 |
| `temper-qa tune` | anyone | `ledger.md`, 6 |

- **Its variables,** internal, read through its table
  (`../tools.md`, 4.3):
  - `TEMPER_INTERNAL_QA_HOME`: its home (section 3), required by every
    command but the front's;
  - `TEMPER_INTERNAL_QA_SESSION`: the agent session a command records
    for, set by the loop for the agent it starts (`triage.md`, 5;
    `passes.md`, 4), required by the session's commands, `triage` and
    `pass` alike;
  - `TEMPER_INTERNAL_QA_AREA`: set, never read, for `request.command`
    (`findings.md`, 5).
- **Its answers:** the front's and refused arguments as `../tools.md`,
  4.2 has them, exit 2; a command that ran, exit 0; a refusal of the
  ledger's (`ledger.md`, 5), a bad configuration or a failure, `temper-qa:
  WHY` on stderr, exit 1, with nothing recorded. The place's commands
  add their own statuses (`places.md`, 4 and 5).
- **Its reference documents,** its arguments' and its environment's in
  `../../cli/`, rendered from its table (`../tools.md`, 4.4).

## 3. Its home and configuration

### 3.1 The home

`TEMPER_INTERNAL_QA_HOME` names a directory the deployment makes and owns:

```
qa.json            its configuration (3.2), the deployment's
ledger.json        the ledger (ledger.md, 2)
ledger.lock        the ledger's lock (ledger.md, 3)
run.lock           held by the one running loop (section 4)
run.sock           the loop's private wake socket (section 4)
builds/COMMIT/     each kept build, its files and its manifest (builds.md), unless qa.json names another directory
sessions/N/        each agent session's brief, diff and output (triage.md, 2; passes.md, 4)
trees/triage/      triage's kept tree (triage.md, 2)
trees/PLACE/       each place's kept pass tree, numbered in configuration order (passes.md, 4)
passes/P/          each pass's setups, reports and requests (passes.md, 2)
gaps/requests/N/   each gaps request, numbered by its first gap (gaps.md, 6)
evidence/F/        each finding's evidence (findings.md, 2)
places/1/          the place, unless qa.json names others (places.md, 2)
```

### 3.2 `qa.json`

One JSON object, read at every command's start; a key it does not know
is refused by name, a missing required key named, and a relative path
taken from the home.

```json
{
  "owner": "AREA",
  "git": { "directory": "PATH", "branch": "main", "fetch": "REMOTE" },
  "agent": { "command": ["PROGRAM", "ARG", "{effort}"], "triage": "EFFORT", "pass": "EFFORT" },
  "bounds": { "poll": 60, "triage": 900, "build_wait": 21600, "pass": 7200 },
  "builds": "builds",
  "places": ["places/1"],
  "surfaces": ["ctl", "web", "tui"],
  "areas": { "ctl": "AREA", "web": "AREA", "tui": "AREA" },
  "gaps": { "fake": "AREA" },
  "request": { "command": ["PROGRAM", "ARG"], "state": ["PROGRAM", "ARG"], "area": "AREA" },
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
  session's effort; `triage`, required, triage's effort, and `pass`,
  required, a pass's, each as the command takes it
  (`../../draft/archive/landed-qa.md`, 2).
- **`bounds`,** each in seconds: `poll`, the fetch interval when
  `git.fetch` is set, 60 when absent; without a remote it sets no
  periodic wake; `triage`, a triage session's time, 900 when absent;
  `build_wait`, how long a case may wait for a kept build that holds its
  landings before it is a line, 21600 when absent (`ledger.md`, 4.7);
  `pass`, a pass session's time, 7200 when absent (`passes.md`, 4).
- **`builds`:** where kept builds are, `builds` when absent.
- **`places`:** the places, at least one, `["places/1"]` when absent
  (`places.md`, 2): each its directory, or `{"directory", "kind",
  "ports"}`: `kind`, `box` or `deployment`, `box` when absent
  (`places.md`, 2; `deployment.md`); `ports`, the nine ports its boxes
  or its deployment's processes bind, as
  `TEMPER_INTERNAL_FAKE_BOX_PORTS` lists them
  (`../../protocol/setup/box-backends.md`, 1; `deployment.md`, 2), each nonzero
  and distinct, and distinct across places, required for a
  deployment's place: `qa.json: place PLACE is a deployment's and names
  no ports`. Directories are distinct. A place's name in
  the ledger is its directory as written here. More than one place
  needs each its ports (`places.md`, 2); the deployment adds one when
  `temper-qa tune` suggests it (`ledger.md`, 6).
- **`surfaces`:** the surfaces a pass takes cases of, each `ctl`, `web`
  or `tui`, `["ctl", "web", "tui"]` when absent; a case of another
  surface stays queued (`passes.md`, 3). Triage records cases of the
  configured surfaces alone, ctl's first (`triage.md`, 3–4).
- **`areas`:** each surface's area, `ctl`, `web` and `tui` each
  optional; a finding on a surface with none takes `owner`
  (`findings.md`, 2).
- **`gaps`,** optional: `{"fake": "AREA"}` names the owner of fake
  gaps; absent, `owner` takes them too. Ctl gaps take `areas.ctl`,
  else `owner`; QA gaps take `owner` (`gaps.md`, 4).
- **`request`,** required: `command`, required, the deployment's command
  that puts a request on main and starts its goal (`findings.md`, 5);
  `state`, the command that says whether a goal has ended without
  landing, absent when the deployment says nothing of it
  (`findings.md`, 6); `area`, the area each request's goal is started
  under, `owner` when absent.
- **`notify`:** a command each new line is given to (section 5), none
  when absent.

The tokens of a session are bounded by the agent's command line itself,
which the deployment writes: the tool bounds its time alone, and records
the tokens the command reports (`triage.md`, 5).

## 4. The loop

`temper-qa run` is the deployment's service: one per home, holding
`run.lock` for its life, so a second is refused, `another temper-qa run
holds HOME`. At its start it ends every session the ledger still has
running as lost (`triage.md`, 6; `passes.md`, 6), its process gone with
the loop that started it. Kept trees stay; session-local trees left by an
older home are removed and their worktree registrations pruned.

- **The wake socket:** while holding `run.lock`, the loop removes any old
  `run.sock` in its home and binds a Unix stream socket there, 0o600.
  Every command that saves a ledger transaction connects after the save
  and lock release, then closes; it sends no bytes and waits for no
  answer. A missing socket or refused connection is no error: no loop
  may be running. A wake failure never undoes a saved command.
  `temper-qa wake` does only that connection, without a session or a
  ledger write, so the deployment can announce its own move of main's
  ref (section 7).
- **A tick,** at start, on a wake or a deadline, takes these steps in
  order, each under the ledger's lock and saved before the next:
  1. **Watch:** main's ref read; landings recorded; kept builds the head
     now holds given their landings and order (`landings.md`, 3 to 5).
  2. **Triage:** a session whose agent has ended, or passed its bound,
     ended (`triage.md`, 6); with none running, a session started for
     the next landing (`triage.md`, 2).
  3. **Passes:** a pass session whose agent has ended, or passed its
     bound, ended (`passes.md`, 6); then, for each free place, a waiting
     pass resumed where it may (`passes.md`, 7), else a pass started
     (`passes.md`, 3).
  4. **Requests:** each request not yet filed given to `request.command`
     (`findings.md`, 5); every open gap not yet filed together as one
     request with no `goal` (`gaps.md`, 6), never starting work;
     each waiting pass's goal asked of
     `request.state` (`findings.md`, 6).
  5. **Waiting cases:** each case past `bounds.build_wait` with no kept
     build for it becomes a line (`ledger.md`, 4.7).
  6. **Builds:** each build the ledger's rule lets go deleted
     (`builds.md`, 5).
  7. **Lines:** each new line given to `notify` (section 5).
- **Between ticks** the loop waits on one channel, fed by the socket's
  blocking accepts and process exits. A burst coalesces to one pending
  wake, hence one tick. The loop's own saves need no socket poke. A
  command's saved records are read on the next tick; nothing waits on
  an agent's words: an agent's end is its process's exit.
- **Exits:** every started agent and notification process has a thread
  blocking on its exit, publishing the result before waking the loop.
  The loop collects those results without probing the processes. At a
  session's bound it still signals the process group, then kills it ten
  seconds later. A bounded request command also waits for its exit
  through a blocking waiter, with a channel timeout for its bound.
- **The wait's timeout** is the nearest known deadline: a session's
  bound, a pending kill, an unreported case's `build_wait`, a busy
  place's retry, a waiting goal's state check, or a failed step's retry
  and hour line. Place retries and goal checks are due in 60 seconds
  while that wait exists. `git.fetch` adds a fetch deadline every
  `bounds.poll` seconds; other wakes read the fetched ref without
  fetching again or postponing that deadline. Without `git.fetch` and
  without work with a deadline, the wait has no timeout.
- **A failed step** (git refusing, a directory not removed, a request's
  command failing) is said on the loop's stderr and tried again at the
  next wake, or after 60 seconds; one that has failed for an hour is a
  line, once, until it next succeeds.
- **A pass's end** is saved before its holder is sent the existing end
  request through the place's `drive.sock` (`places.md`, 5). The loop
  opens that connection before reusing the place, so a delayed helper
  cannot end a later pass's holder. A thread drains that reply and waits for the place's lock to release, then
  wakes the loop; the loop keeps running while the drive stops. The
  holder's ledger watch remains a 60-second fallback.
- **Its world:** the clock and event wait, git, the agent's processes,
  the request's commands and the home's files, each behind a face of
  the crate's (section 6). This std-based tool uses blocking socket and
  child waiter threads, rather than the service programming model's
  ring: std and `skein-shell` provide these events without another
  dependency. Those threads report results; the loop alone advances
  the ledger.

## 5. Lines for the owner

- **A line** is one sentence about the pass's own state, for the area
  `owner` names, never a finding against temper
  (`../../draft/archive/landed-qa.md`, 2): a landing whose triage recorded
  nothing twice (`triage.md`, 6), a case waiting past its bound for a
  build (`ledger.md`, 4.7), a step failing for an hour (section 4), a
  pass session that recorded nothing and a case blocked twice
  (`ledger.md`, 4.12). A finding handed on after two goals is a line for
  its own area (`ledger.md`, 4.14).
- **Kept in the ledger** (`ledger.md`, 2.6), each once, with its time
  and area, and printed by `temper-qa lines`.
- **Given to `notify`** where `qa.json` names one: the command runs once
  per new line, with `AREA: WORDS` and a newline on its stdin, and the
  line is marked given whatever it exits. The deployment's command
  carries it to the area: a message, a chat, a mail. A command that
  fails is said on the loop's stderr when its exit event arrives, never
  a line.

## 6. Its tests

Each failing first, in the crate's three test targets and its binary's
table test (`../tools.md`, 6). What replays runs in the focused and fuzzy
suites; only the production faces' own checks need the real loop
(`tools/tiers/layers.md`, 6.1).

- **Its world behind faces:** the tool reaches the machine through its
  faces alone (`faces`): git (`GitFace`); its children (`Processes`: the
  agents, the request's and `notify`'s commands, `tar`, the drive tool's
  `recipe` commands, an executable's `--version`, the holder and its drive
  session, and the loop's end of a holder); the clock (`ClockFace`: the
  loop's waits, the holder's and its clients' polls); the drive tool and
  the archive (`DriveFace`, `ArchiveFace`, `places.md`, 8); and every path
  it keeps (`FilesFace`: the home's documents and locks, sessions, builds,
  trees and places, their modes and links, the build files' checksums,
  and the Unix sockets among them, `run.sock` and a place's
  `drive.sock`). The binary's `main` makes the production faces once,
  beside its captured arguments and environment
  (`../tools.md`, 4.3), and hands them to the command entry, which runs
  every command over the faces it is given; nothing else makes one. So
  every rule below runs over a world in memory, and only the production
  faces need the kernel.
- **Three targets:** `step`, values and pure functions: the ledger,
  configuration, parser and reference checks, the seeded ledger sequences
  included (fuzzy), the command table's answers and `tune`'s figures;
  `domain`, the QA runner's world: the home in memory, the loop, the
  commands through the command entry, builds, installs, places and the
  holder; `suite`, in the real loop, the production faces' witnesses
  alone. Shared fixtures and the world compile once in the crate's
  library, its `testing` module.
- **The world** (`domain`): a files face holding a tree of files,
  directories, links, modes, devices, locks and sockets, with the
  failures a test injects by path; git of commits in memory whose
  worktrees are directories of that tree; agents, request commands,
  `tar`, the drive tool and the executables a build keeps as programs the
  test scripts, each seeing the tree as it stands when it runs; a holder
  and a drive session as scripted streams; a clock the test moves, which
  a sleep advances. A test's home is a path of the tree with its own
  `qa.json`.
- **The ledger's rules over values** (`ledger.md`, 4): built in memory,
  driven through its operations, its records checked; the deletion
  rule's and the passes' invariants under random sequences of events, in
  the fuzzy suite: a case in one pass at a time, a held build never
  deleted.
- **The command table** (`step`): each help page, the front's refusals
  and each command's argument refusals, with their words and exits
  (section 2), answered before any effect.
- **The commands through the command entry** (`domain`), in the world's
  home: the agents' commands and their refusals, each refusal leaving
  the ledger's bytes as they were, `show`, `lines`, `report`, `gaps` and
  `wake`, a finding's evidence copied and its limits.
- **The watcher and the loop over the world:** an agent that records
  through the ledger's commands or ends having recorded nothing, builds
  as directories of the tree, and request commands that answer as the
  test says, seeing the files the loop wrote for them.
- **`build keep`** on a staging directory of the tree whose executables
  answer their version as the test scripts it.
- **The place's commands** over the drive tool's and `tar`'s scripted
  programs and the archive's fake (`places.md`, 8), so that a copy's
  restore, its making again, a walk's log and the archive's set-aside
  paths are checked with no box, a place's ports passed to each walk
  and session among them.
- **The holder's two halves:** its clients' `pass open`, `pass line` and
  `pass close` against a scripted holder, and the holder against
  scripted connections and a scripted drive session, so that its lines,
  answers, exits, kept lines and the drive's arguments are checked with
  no socket and no child (`places.md`, 5).
- **`tune`** (`step`) over ledgers the test builds at a given time: each
  figure, and each suggestion made or withheld at its threshold
  (`ledger.md`, 6); and in the world a wait for a place opened and closed
  by the loop's third step (`ledger.md`, 4.17).
- **The `ctl` surface and the places' kinds:** a `ctl` case recorded
  against ctl's groups and listed first in triage's and a pass's
  briefs; a pass whose ctl cases run first opening its session with
  `--ctl`, no browser started; a case's kind its setup's, and a pass at
  a place of one kind taking cases of that kind alone; a web or tui case
  on a deployment's setup recorded as on a box's, and a deployment's
  pass briefed to run its ctl cases first in a ctl session and then its
  web and tui cases in a full one (`passes.md`, 4); triage's catalogue
  of the kinds the places run alone (`kinds.md`, 6).
- **The production faces** (`suite`, the real loop), each where only the
  kernel shows it: a file name that is not UTF-8, on Linux alone
  (`../macos.md`); a staged build's executables run for their versions
  and the build published; a saved command's poke, `run.sock` replacing
  a stale path under any umask, and a child's exit waking the loop; the
  holder tightening a place's modes before it starts, and ending its
  drive and freeing the place on the ledger's watch; a restore private
  under permissive umasks; the system's `tar` saving and restoring a
  copy without its pinned agent; a bounded command's process group
  killed at its bound; a pass's end reaching its holder through the
  socket, captured before the path can be reused.
- **The binary's table test** (end to end): `--help`, `--version` and
  a command the table lacks (`../tools.md`, 6), and one shipped path
  for each boundary a command crosses: `build keep` publishing a stage,
  `triage case -` from stdin, `wake`, `triage gap`, a holder's `pass
  open`, `pass line` and `pass close`, a box failing a line with status
  4, `pass setup` and `pass recipe` with their walks' exits, `pass
  finding` with its evidence, and `tune`.

None starts a browser, a box, a real agent or the network; none runs
`git` but the loop's real face, which the walk after landing exercises
(`../../draft/archive/landed-qa.execution.md`, 1).

## 7. Setting it up

The deployment's, on the machine that holds the places:

- a home with its `qa.json`, a repository of the project for `git`,
  an agent command line, with its token bound, for `agent`, and the
  request's commands for `request` (`findings.md`, 5 and 6);
- `temper-qa run` as a service, with `TEMPER_INTERNAL_QA_HOME`, and
  `SKEIN_TEST_CHROMIUM` naming the Chromium the drive session takes
  (`../../client/drive.md`, 2.1) for a full session at a place of
  either kind, which a ctl session never starts;
- a deployment's place, where its cases are wanted, with its own nine
  ports in `qa.json` (3.2);
- its after-landing work calling `temper-qa wake` after moving main's
  ref, then building at each landed tip and calling `temper-qa build
  keep` (`builds.md`, 2), whose saved transaction wakes the loop too.

## 8. Later

- **Edge cases, the help as a surface, and real backends**
  (`../../draft/archive/landed-qa.md`, 12).

## 9. Open questions

- **A line's medium:** the deployment's `notify` command carries it
  today; temper's routing by area (`../../draft/standing-roles.md`,
  section 4) may take it once built.
