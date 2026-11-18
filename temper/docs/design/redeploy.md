# The redeploy

Provisional, 2026-11-13, revised 2026-11-16 for milestone R8 (who was
told the engine restarts, 4.2; each worker judged by its wait, 6.2) and
2026-11-18 for milestone C3 of `draft/checkpoints.execution.md` (the
check over a checkpoint while the old build serves, 4.1; the stop's
checkpoint as the backup and the verdict read again over it, 4.2; going
back by restore, 4.6).
How a deployment of temper takes a new build
while its work goes on, and how each redeploy is refereed afterwards:
`temper-redeploy`, an internal tool run by the deployments' operator
from a machine that reaches the deployment's hosts. It carries
`draft/redeploys.md`, section 7, milestone R6 of
`draft/redeploys.execution.md`. What it drives is designed elsewhere:
the engine's settled stop (`domain/root.md`, 8.2), a worker's drain
(`domain/worker/README.md`, section 2; jig's `domain/hosts/hub.md`, 7),
`upgrade --check` and its verdict on the workers (`protocol/binaries.md`,
section 2; `protocol/wires.md`, 10.2), a deployment's checkpoints and
their restore (`protocol/checkpoints.md`), the workers' grace
(`protocol/configuration/README.md`) and an agent pinned at its worker's
start (`domain/worker/README.md`, section 5). This document says what
the tool runs, in what order, what it keeps and how it judges.

## 1. In one page

- **A description of the deployment** (section 2), kept by its operator
  outside the repository: each part's host, reached by a command prefix
  such as `ssh`, and the command lines that stop, start, swap, check,
  take a checkpoint and read each part. The tool names no machine.
- **The build staged first** (section 3) on every host, beside what runs,
  each binary asked its `--version`.
- **The check while the old build serves** (4.1): a checkpoint of the
  serving deployment, taken through the engine's `ctl`, and the new
  build's `upgrade --check` over it, with its verdict on the workers. A
  check that refuses ends the redeploy with nothing stopped.
- **The engine alone, by default** (4.2, 4.3), when the check says the
  workers may stay: its settled stop, a checkpoint of the stopped
  deployment as the backup, the verdict's inputs read again over it, and
  the swap and the start the workers ride.
- **Workers in turn, when asked** (4.4): one at a time, drained, swapped,
  started, and waited for until it opened and its logins came, before
  the next.
- **Every part together** (4.5) when the check says a worker may not
  stay, or when asked: every worker drained to the old engine, which
  still serves, then the engine's settled stop and its checkpoint, every
  binary swapped and all started. A verdict that no longer says yes at
  the stop starts the old engine again first.
- **Nothing half done before the swap** (4.6): a failure until the new
  engine is installed starts again what was stopped, on its old build;
  after it, the tool stops where it stands and says how to go back: the
  new build's `upgrade --back` before the new generation's first commit,
  a restore of the stop's checkpoint after it.
- **A record** of the redeploy (section 5), written as it goes, which the
  referee reads.
- **The referee** (section 6): each worker's agreed versions; each
  worker's return against the wait it was given, the restart wait the
  engine told it or its grace; no try spent and no task held for a
  failure but a run's own since the redeploy began; a verdict.
- **Tested against a fake deployment** in the focused and fuzzy suites
  (section 8); running it against a deployment is a redeploy, the
  operator's to start.

## 2. The deployment's description

### 2.1 Its members

A JSON object, read whole before anything runs. Every member below is
required; a member it doesn't name, a missing one, a wrong type, an
empty string or list, a template lacking a placeholder it requires or
naming one it isn't given, and two workers of one name are refused,
naming the member's path (`workers[1].swap`), exit 1.

```json
{
  "source": "https://fabrig.ai/temper/v0",
  "engine": {
    "run": ["ssh", "-o", "BatchMode=yes", "operator@engine.example.com"],
    "binaries": "/usr/local/bin",
    "builds": "/var/lib/temper-builds",
    "stop": "sudo systemctl stop temper-engine",
    "start": "sudo systemctl start temper-engine",
    "log": "sudo journalctl -u temper-engine -o cat --since @{since}",
    "check": "sudo -u temper {binary} upgrade --check --config /var/lib/temper/config --data /var/lib/temper/data --cache /var/lib/temper/cache /var/lib/temper/data/deployment/checkpoints/{checkpoint}",
    "checkpoint": "sudo -u temper {binary} checkpoint --config /var/lib/temper/config --data /var/lib/temper/data --cache /var/lib/temper/cache",
    "verdict": "sudo -u temper {binary} upgrade --workers --config /var/lib/temper/config --data /var/lib/temper/data --cache /var/lib/temper/cache /var/lib/temper/data/deployment/checkpoints/{checkpoint}",
    "swap": "sudo install -m 0755 {from} {to}",
    "ctl": "sudo -u temper temper ctl /var/lib/temper/ctl"
  },
  "workers": [
    {
      "name": "w1",
      "run": ["ssh", "-o", "BatchMode=yes", "operator@worker.example.com"],
      "binaries": "/usr/local/bin",
      "builds": "/var/lib/temper-builds",
      "stop": "sudo systemctl stop temper-worker",
      "start": "sudo systemctl start temper-worker",
      "swap": "sudo install -m 0755 {from} {to}",
      "document": "sudo cat /var/lib/temper/config/temper.json"
    }
  ]
}
```

- **`source`:** where builds are fetched from, as `temper help server`'s
  "Before you start" fetches them: `SOURCE/COMMIT/SHA256SUMS` and
  `SOURCE/COMMIT/Linux-x86_64/NAME`, gzip without a suffix.
- **`run`:** the command prefix that runs one command line on the
  part's host; the tool appends the line as one more argument (2.2).
  `["sh", "-c"]` for a part on the operator's own machine.
- **`binaries`:** the directory the part's installed binaries are in:
  the engine's `temper-engine` and `temper` (the client, for `ctl`); a
  worker's `temper-worker` and `temper-agent`, which its document's
  `agent.program` names, so that the worker pins the new agent at its
  next start (`domain/worker/README.md`, section 5).
- **`builds`:** a directory the `run` account writes, holding each staged
  build as `BUILDS/COMMIT/NAME` (section 3).
- **`stop`:** stops the part and returns once it has exited: the
  engine's settled stop, or a worker's drain within its stop bound. A
  service manager's stop does, its stop timeout above the bound
  (`temper help server`).
- **`start`:** starts the part, returning once started.
- **`log`** (the engine's alone): prints the engine's stderr lines since
  `{since}`, oldest first, in any line format that keeps each line's
  text whole.
- **`check`** (the engine's): runs `{binary} upgrade --check` over the
  checkpoint `{checkpoint}` of the engine's deployment directory,
  `DEPLOYMENT/checkpoints/{checkpoint}`, as the deployment's own user,
  with the directories the engine's `serve` is given
  (`protocol/binaries.md`, section 2). It runs while the engine serves,
  holding the checkpoint's lock (jig's `protocol/store/checkpoints.md`,
  6).
- **`checkpoint`** (the engine's): runs `{binary} checkpoint` over the
  engine's stopped deployment, as `check`'s user with `serve`'s
  directories, which prints its one line (`protocol/checkpoints.md`,
  2.1).
- **`verdict`** (the engine's): runs `{binary} upgrade --workers` over the
  checkpoint `{checkpoint}`, as `check` does (`protocol/binaries.md`,
  section 2).
- **`swap`:** installs `{from}` as `{to}`.
- **`ctl`** (the engine's): `temper ctl DIR` for a ctl directory on the
  engine's host, its endpoint loopback (`client/ctl/README.md`), signed
  in as a person who owns the deployment, whose act takes its
  checkpoint while it serves (`protocol/checkpoints.md`, 3), and holds a
  role in each project the referee is to read (6.1).
- **`document`** (a worker's): prints the worker's configuration
  document, from which the tool reads its grace (4.1).
- **`workers`:** in the order workers go in turn (4.4), at least one, each
  `name` the worker's name as the engine's configuration and its opening
  give it.

### 2.2 Commands on a host

- **Placeholders** are `{since}` (the Unix time in whole seconds),
  `{binary}`, `{checkpoint}` (a checkpoint's name, `<20 digits>-<16
  hex>`, `protocol/checkpoints.md`, 2.4), `{from}` and `{to}`. `log`
  requires `{since}`; `check` and `verdict` both `{binary}` and
  `{checkpoint}`; `checkpoint` `{binary}`; `swap` both `{from}` and
  `{to}`; `stop`, `start`, `ctl` and `document` take none. A description
  of a build before checkpoints, with a `backup` member, is refused as
  any member 2.1 doesn't name. Each value goes in quoted for a POSIX
  shell, `'` around it and `'\''` for each quote within.
- **Lines the tool composes** besides the templates (a binary's
  `--version`, `mkdir -p`, `cp`, the build's fetch) quote every word the
  same way.
- **One command line per call:** the tool runs `RUN... LINE`, LINE the
  whole line as one argument, which `ssh` hands its remote shell and
  `sh -c` its own. It waits for the line's exit, keeping its stdout and
  stderr; a prefix that cannot be spawned is a failed call, naming it.
- **The tool's own environment** is handed to every child as it was
  captured (`testing/tools.md`, 4.3); it adds none.

## 3. The build staged

- **COMMIT** is the build's twelve lowercase hexadecimal digits
  (`release.md`, section 3); anything else is refused, exit 2.
- **For each part, each of its binaries:** `BUILDS/COMMIT/NAME --version`
  answering `NAME VERSION (COMMIT)` is staged. Otherwise the part's host
  fetches the build into `BUILDS/COMMIT/`, as `temper help server`'s
  "Before you start" does: `SHA256SUMS`, then each missing binary under
  `Linux-x86_64/`, checked with `sha256sum -c --ignore-missing
  SHA256SUMS`, unzipped to `BUILDS/COMMIT/NAME`, mode 0755; then asked
  `--version` again. A fetch, a check or an answer naming another commit
  refuses the stage, naming the part and the binary, exit 1.
- **`temper-redeploy stage DESCRIPTION COMMIT`** stages alone, ahead of a
  redeploy; `run` stages first in any case, so a staged build is left as
  it is. Nothing is stopped or swapped.

## 4. The redeploy

`temper-redeploy run DESCRIPTION COMMIT --record FILE [--workers |
--together]`: the engine alone by default (4.3); `--workers` goes on with
the workers in turn (4.4), for a change the workers run; `--together`
restarts every part together (4.5), for a redeploy its operator knows
breaks, such as one whose check on staging said no. The two flags
exclude each other.

### 4.1 Before anything stops

1. **The description read** (section 2), and FILE refused if it exists:
   one record per redeploy, never overwritten.
2. **The build staged** (section 3).
3. **Each part's build:** its installed binaries' `--version`, the
   engine's by `temper-engine`, a worker's by `temper-worker`, each the
   part's previous build in the record (section 5).
4. **Each worker's grace:** its `document`'s `workers` entry whose `name`
   is the worker's, its `graces.grace_ms`
   (`protocol/configuration/README.md`).
5. **The referee's snapshot** of the tasks (6.1).
6. **`began`:** the time the record starts its span from.
7. **The checkpoint,** while the old build serves: the engine's `ctl`
   with `deployment checkpoint` (`protocol/checkpoints.md`, 3), its
   answer's `name` printed and kept as the record's `checkpoint`. A
   serving build before milestone C1 of
   `draft/checkpoints.execution.md` has no such act, and its refusal
   fails the step: such a deployment is redeployed onto a build that has
   it by the tool of a build before this one.
8. **The check,** run once per redeploy, before anything stops: the
   engine's `check`, `{binary}` the staged
   `BUILDS/COMMIT/temper-engine`, `{checkpoint}` that name. It is the new
   build's full check of the state at the checkpoint's commit (jig's
   `protocol/store/opening.md`, 5): every record decoded, the jobs, the
   restores and the verdict on the workers. Each of its lines is printed
   and kept. It never writes its input, so neither the checkpoint nor the
   live store changes (`draft/checkpoints.md`, 5.2).
9. **What it says** (`protocol/wires.md`, 10.2), by its exit and its last
   line ending `workers may stay: `:
   - exit other than 0: a start would be refused, and the redeploy ends
     **refused**, exit 1, with nothing stopped;
   - exit 0, `workers may stay: yes`: as the mode asks, the engine alone
     (4.3), then with `--workers` the workers in turn (4.4), or with
     `--together` every part together (4.5);
   - exit 0, `workers may stay: no`, `workers may stay: unknown…`, or no
     such line: every part together (4.5), each worker drained while the
     old engine still serves.

A failure here refuses the redeploy with nothing stopped, exit 1. The
check's checkpoint stays until the store's retention removes it
(`protocol/checkpoints.md`, 4).

### 4.2 The settled stop and its checkpoint

- **The engine's `stop`** at the window's start (6.2). The workers keep
  their runs, their grace or the restart wait running from their links'
  loss.
- **Who was told** (`draft/redeploys.md`, 5.3, *Who will not ride it is
  named*): once the stop returns, the engine's `log` read with `{since}`
  one second before it began. A line holding `serve: worker NAME told
  the engine restarts: wait MS ms` says NAME was told MS; one holding
  `serve: worker NAME not told the engine restarts: ` says it was not,
  and keeps its grace (`protocol/wires.md`, section 4). A worker of the
  description with neither line was not told: its link was down, or the
  engine's build cannot say it, as the first redeploy onto the build
  that brings the word is stopped by one that cannot. Read at the first
  settled stop alone, the window's; each is printed and kept (section
  5). A log that cannot be read fails the step as any.
- **Its checkpoint,** at every settled stop, as the backup: the engine's
  `checkpoint`, `{binary}` the installed `BINARIES/temper-engine`. The
  installed build takes it, since the new build refuses a store it must
  upgrade before serving (`Unready::Upgrade`, jig's
  `protocol/store/checkpoints.md`, 5). A stopped deployment's checkpoint
  links every segment and copies only the secrets and the live tasks'
  transcripts and logs (`draft/checkpoints.md`, 4), so no deployment is
  copied whole. Its line is printed and kept; NAME, its word after
  `checkpoint: `, is kept as the record's `backup`, replacing an earlier
  stop's: the state the swap starts from, which going back after the new
  generation's first commit restores (4.6). A refusal fails the step.
- **The verdict read again,** when the engine goes alone (4.3): the
  engine's `verdict`, `{binary}` the staged
  `BUILDS/COMMIT/temper-engine`, `{checkpoint}` the stop's checkpoint.
  It reads the inputs the check's verdict used, the engine's kept wires,
  the configured workers and their kept openings, and judges them by
  the same rules, with no record walked and no job run
  (`protocol/binaries.md`, section 2; `protocol/wires.md`, 10.2). Its
  lines are printed and kept as the record's `verdict`:
  - exit 0, `workers may stay: yes`: the swap (4.3);
  - exit 0, any other verdict, or none: an input changed since the check,
    such as a worker's opening or the configuration. The engine started
    again on its old build, its openings waited for (4.7), then every
    part together (4.5), each worker draining to a live engine;
  - exit other than 0: the step fails (4.6).
- **Retention:** the store keeps its newest `store.checkpoints.keep`
  checkpoints, 2 by default, skipping one whose lock is held
  (`protocol/checkpoints.md`, 4). A redeploy makes two, the check's and
  the stop's, so the default keeps a redeploy's backup until the next
  one's stop.

### 4.3 The engine alone

1. The settled stop, its checkpoint and the verdict read again (4.2).
2. **The swap:** for each of the engine's binaries, the installed one
   copied to `BUILDS/PREVIOUS/NAME` unless there, PREVIOUS its build
   (4.1), so that going back is a redeploy of PREVIOUS; then the
   engine's `swap`, `{from}` the staged binary, `{to}` the installed
   path. A part already installed at COMMIT is swapped all the same.
3. **The start,** and every worker's opening waited for (4.7). The
   window ends at the first opening.
4. **With `--workers`,** the workers in turn (4.4).

### 4.4 Workers in turn

For each worker in the description's order, one at a time, the others
serving throughout (`draft/redeploys.md`, 7.2):

1. its `stop`: the drain;
2. its `temper-worker` and `temper-agent` swapped, as the engine's are
   (4.3, step 2);
3. its `start`;
4. its opening in the engine's log waited for (4.7), then its logins:
   the engine's `ctl` `account list --json`, polled each second, until
   the entry of `$.accounts.deployment_facts.program_hosts` whose
   `name` is the worker's reads `connected` true and `awaiting_logins`
   false (`client/ctl/README.md`), within the wait's bound.

Its outage runs from its `stop` to its logins. A worker that doesn't
open, is refused or doesn't log in within the bound ends the redeploy
**failed**; the workers after it are not touched.

### 4.5 Every part together

`draft/redeploys.md`, 7.3; `draft/checkpoints.md`, 6:

1. every worker's `stop` at once, the tool waiting for all: each drains
   to the engine, which serves;
2. the engine's settled stop and its checkpoint (4.2);
3. the engine's binaries swapped, then each worker's;
4. the engine's `start`, then every worker's;
5. every worker's opening waited for (4.7).

The check ran before anything stopped (4.1). Reached from a verdict that
changed at the stop (4.2), the old engine has been started again and its
openings waited for before step 1; its second settled stop takes a
second checkpoint, which becomes the backup, and who was told stays the
first stop's. The outage runs from the first worker's `stop` to the last
opening.

### 4.6 A step that fails

- **A step fails** when its command line exits other than 0, cannot be
  spawned, or its wait passes its bound.
- **Before anything stops** (4.1), a failure ends the redeploy
  **refused**, exit 1, with nothing to start again.
- **Until a binary is swapped,** a failure starts again every part the
  redeploy stopped, on its old build, waits for their openings, and
  ends **refused**, exit 1: nothing is half done.
- **Once one is,** a failure ends the redeploy **failed** where it
  stands, exit 1: its line names the step, the parts swapped, the backup
  (the last stop's checkpoint) and the previous build. Going back is the
  operator's, over the stopped deployment (`protocol/binaries.md`,
  section 2; `draft/checkpoints.md`, 6, step 6):
  - **before the new generation's first commit:** the new build's
    `upgrade --back`, then the previous build's binaries, kept as
    `BUILDS/PREVIOUS/NAME` (4.3), swapped back and started;
  - **after it:** the new build's `restore BACKUP`, which first takes a
    safety checkpoint of the state it replaces (`protocol/checkpoints.md`,
    2.5), then the previous build swapped back and started. The new
    build restores, since the safety checkpoint's open refuses a store
    ahead of the build that opens it (jig's
    `protocol/store/restoring.md`, 4.3). The generation restored holds
    the stop's state at the previous build's levels, which the previous
    build opens as it left it;
  - **where no level moved,** the previous build opening the store as it
    stands, `run` with that build is a redeploy as any.
- **The referee runs** after a refused or failed redeploy too, judging
  what it left.

### 4.7 Waiting for openings

- **The engine's `log`** read with `{since}` one second before the
  start, polled each second.
- **A line holding `serve: worker NAME opened: `** is NAME's opening, its
  time the poll's that first saw it, and the rest of the line its
  agreed versions, `WIRE VERSION` pairs separated by `, `, as the engine
  logs them (`protocol/wires.md`, section 4).
- **A line holding `serve: worker NAME refused at its opening: `** is
  NAME refused: its wait ends, the line kept, the step failed.
- **The bound:** the largest wait given among the workers waited for:
  for a worker told, the longer of its told wait and its grace; for any
  other, its grace (4.1, 4.2). A worker unopened by then fails the step,
  named. Openings of a name
  the description lacks are kept, never waited for.

## 5. The record

FILE, JSON, written whole after each step: a temporary file beside it
renamed over it. Its members, which the referee reads and a later
`referee` reads again:

| Member | What |
|---|---|
| `commit` | COMMIT |
| `mode` | `"engine"`, `"workers"` or `"together"`, as asked |
| `began` | 4.1, step 6, wall nanoseconds since the Unix epoch, as every time here |
| `previous` | each part's build before: `{"part": "engine" or a worker's name, "commit": …}` |
| `graces` | each worker's grace: `{"worker", "grace_ms"}` |
| `told` | each worker of the description, at the window's stop (4.2): `{"worker", "wait_ms"}`, `wait_ms` the wait the engine told it, or null when it was not told; null before that read, and absent from a record of a build before it, which a later `referee` reads as no worker told |
| `before` | the referee's snapshot (6.1): `projects`, the project numbers read; `cut`, whether ctl cut the memberships; `tasks`, each `{"task", "project", "label", "phase", "held"}`, `held` the hold's failures class or null |
| `lines` | every line the run printed, each `{"at", "line"}` |
| `checkpoint` | the name of the checkpoint the check read (4.1), or null before it was taken; absent from a record of a build before it, which a later `referee` reads as null |
| `check` | `{"code", "lines"}`, or null before it ran |
| `verdict` | the verdict read again at the stop (4.2), `{"code", "lines"}`, or null when not read; absent as `checkpoint` is |
| `backup` | the name of the last settled stop's checkpoint (4.2), or null before one |
| `window` | `{"from", "to", "worker"}`: the engine's stop to the first opening after the start the workers rode, or null |
| `outages` | each `{"what", "from", "to"}`: a worker's name in turn, or `"every part"` |
| `openings` | each `{"worker", "at", "line", "refused"}`, from the last start waited for |
| `outcome` | null while it runs, then `"done"`, `"refused"` or `"failed"` |
| `ended` | its time, or null |

## 6. The referee

`draft/redeploys.md`, 7.5. It runs at the end of every `run`, and alone
as `temper-redeploy referee DESCRIPTION RECORD`, which reads the tasks
afresh, so that a later run extends the span to its own read.

### 6.1 What it reads

Through the engine's `ctl` (2.1), each call with `--json` and stdin
closed, as the client's reads give them (`client/ctl/README.md`;
`temper-client-tools`' renderers):

- **The projects:** `me --json`, `$.projects[].project`, and `$.cut`.
- **The live tasks of each:** `task list --project P --json`, every page:
  the first's rows `$.tasks.rows[]`, a continued page's `$.live[]`, the
  next page's words `$.read_on`, null on the last. Of each row,
  `.chip.task`, `.chip.phase` and `.label` (2 a goal).
- **A held task's hold:** `task show T --json`, `$.task.hold`, its
  failures class `$.task.hold.reason.failures.class` where the reason
  is failures.
- **A task's run endings:** `task activity T --json`, newest page first
  and back by `$.before`, until a row older than `began`: each row's
  `.at`, and where `.event.owner.source.line.run_ended` is present its
  `attempt` and `end`, a failed end's class `end.failed.class`.

The snapshot before (4.1) is the projects and their live tasks, with each
held one's class. After, the referee reads them again, and the
activity of every task in either.

### 6.2 What it judges

1. **Openings:** each worker of the description has an opening in the
   record, unrefused; its agreed versions printed. A worker unopened or
   refused fails, named.
2. **Each worker's return,** when the workers rode an engine's restart
   (`draft/redeploys.md`, 7.5): from the window's start to the worker's
   opening in the record, against the wait it was given, the longer of
   its told wait and its grace for a worker told (4.2), its grace for
   any other; at or past it fails, named. A worker back within it ended
   no run for contact, which the referee cannot read otherwise: a run's
   end in the engine's activity keeps no cancel's reason (6.1). The
   window is printed, not judged. Outages are printed, not judged: a
   worker restarted in turn or every part together ends its runs
   interrupted, spending no try.
3. **Tries:** each run ended failed at or after `began`, of any class
   but the run's own (`run`), fails, named with its class: a redeploy
   can cause any of the others (lost, infrastructure, agent, transient,
   permanent, invalid), and the referee cannot tell which did. One of
   the run's own class is printed, not judged. An interruption, which
   spends no try, ends `refused` and is passed over.
4. **Holds:** each task held after that was not held before, its reason
   failures of any class but the run's own, fails, named; a goal is
   named as one. Other holds since are printed by their reason's name,
   not judged.
5. **What it could not read:** a ctl call failing fails, naming it;
   `cut` memberships are printed, the projects past them unread.
6. **The verdict:** pass with no failure, else fail with their count.

### 6.3 Its lines

The tool prints each line as it happens, to stdout, each starting
`temper-redeploy COMMAND: `, COMMAND `stage`, `run` or `referee`, as
temper's binaries do (`protocol/binaries.md`, section 2). Durations are
seconds to one decimal. The referee's, after that prefix:

```
worker NAME agreed WIRE VERSION, WIRE VERSION, …
worker NAME did not open
worker NAME refused: TEXT
window SECONDS s
worker NAME back in SECONDS s within its restart wait SECONDS s
worker NAME back in SECONDS s past its restart wait SECONDS s
worker NAME back in SECONDS s within its grace SECONDS s
worker NAME back in SECONDS s past its grace SECONDS s
WHAT was out SECONDS s
task|goal T attempt A failed: CLASS
task|goal T attempt A failed: run, its own
task|goal T held: failures CLASS
task|goal T held: REASON
read: P projects, T tasks, since YYYY-MM-DDTHH:MM:SSZ
memberships cut: projects past them not read
could not read: WHAT: WHY
pass
fail: N
```

`run`'s lines name each step as it ends: the builds before, the
check's checkpoint, the check's lines as it printed them and what it
decided, each stop with its time, each worker told or not at the
window's stop (`worker NAME told: restart wait SECONDS s`, or `worker
NAME keeps its grace: not told`), each stop's checkpoint, the verdict's
lines read again and what they decided, each swap with its builds, each
start, each opening with its time and agreed versions, each worker's
logins, and the outcome, `done`, `refused: WHY` or `failed at STEP:
WHY`, the last naming what is swapped, the backup and the previous
build.

### 6.4 Exits

`run`: 0 when done and the referee passes; 1 when refused, failed or the
referee fails. `referee`: 0 on pass, 1 on fail. `stage`: 0 or 1. Each
command's refused arguments: 2, in its table's words.

## 7. The tool

- **An internal binary,** `temper/bin/temper-redeploy/`, and its crate,
  `temper/crates/temper-redeploy-tool/`, as `testing/tools.md`, section 3
  lays every tool out, its line "a deployment's redeploy: the engine
  alone by default, workers in turn, every part together across a
  breaking change, and its referee." Its commands are internal:
  `stage`, `run` and `referee`, with section 4's flags.
- **Its seam** (crosses its builders): every host command and every
  clock read goes through one trait, so the tests run the whole of it
  against a fake (section 8):

  ```rust
  pub enum Part { Engine, Worker(usize) }   // a worker by its index in the description
  pub struct Output { pub code: Option<i32>, pub stdout: String, pub stderr: String }
  pub trait Hosts {
      fn run(&mut self, part: Part, line: &str) -> Output;
      fn run_each(&mut self, lines: &[(Part, String)]) -> Vec<Output>;  // at once, each waited for
      fn now(&mut self) -> u64;              // wall nanoseconds since the Unix epoch
      fn sleep(&mut self, nanoseconds: u64);
  }
  ```

  `Output.code` is None when the line could not be spawned or was
  killed, `stderr` saying why. The process implementation spawns
  `RUN... LINE` (2.2); `run_each` spawns every line, then waits for
  each.
- **Its modules,** each section's: the description (2), the quoting and
  the templates (2.2), staging (3), the redeploy's steps (4), the record
  (5) and the referee (6), with `cli` composing a command: `run` reads
  the description, stages, takes the referee's snapshot, sets `began`,
  redeploys, then referees, writing the record throughout; `referee`
  reads the description and the record and referees, writing nothing.
- **What the modules hand each other** (crosses its builders):

  ```rust
  // description: section 2, its members as fields of the same names
  pub struct Description { pub source: String, pub engine: Engine, pub workers: Vec<Worker> }
  pub fn read(text: &str) -> Result<Description, String>;
  // hosts: 2.2
  pub fn quote(word: &str) -> String;
  pub fn fill(template: &str, values: &[(&str, &str)]) -> String;
  // the engine's `ctl` with WORDS quoted, `--json` and stdin closed; its stdout on exit 0,
  // else why, naming the call
  pub fn ctl(description: &Description, words: &[&str], hosts: &mut dyn Hosts) -> Result<String, String>;
  // record: section 5, its members as fields of the same names
  pub struct Record { /* … */ pub before: Before, /* … */ }
  pub struct Before { pub projects: Vec<u64>, pub cut: bool, pub tasks: Vec<TaskBefore> }
  pub struct TaskBefore { pub task: u64, pub project: u64, pub label: u64, pub phase: String, pub held: Option<String> }
  pub struct Journal { pub record: Record, /* its file, if any, and its output */ }
  impl Journal {
      pub fn line(&mut self, at: u64, line: &str) -> Result<(), String>;  // printed, kept in `lines`, the file written
      pub fn save(&mut self) -> Result<(), String>;                       // the file written whole
  }
  // stage: section 3
  pub fn stage(description: &Description, commit: &str, journal: &mut Journal, hosts: &mut dyn Hosts) -> Result<(), String>;
  // redeploy: section 4, from 4.1's step 3
  pub enum Mode { Engine, Workers, Together }
  pub enum Outcome { Done, Refused, Failed }
  pub fn redeploy(description: &Description, commit: &str, mode: Mode, journal: &mut Journal, hosts: &mut dyn Hosts) -> Outcome;
  // referee: section 6
  pub fn before(description: &Description, hosts: &mut dyn Hosts) -> Result<Before, String>;
  pub fn judge(description: &Description, journal: &mut Journal, hosts: &mut dyn Hosts) -> bool;  // true on pass
  ```

  A `Journal` with no file, as `referee` alone runs it, keeps its lines
  in memory.
- **It reads no host but through `Hosts`,** writes no file but its
  record, and starts nothing on the operator's machine but `run`'s
  prefixes.

## 8. Testing

Focused and fuzzy alone (`draft/redeploys.md`, 10). No test reaches a
host: no `ssh`, no service manager, no network.

- **The fake deployment:** a `Hosts` whose engine and workers each keep
  whether they run, their installed and staged builds and the engine's
  log; that answers the description's templates and the lines the tool
  composes; whose engine's start logs each worker's opening, or its
  refusal, as a test scripts it; whose `check`, `verdict` and `ctl`
  answer what a test scripts, `ctl`'s `deployment checkpoint` among
  them; whose engine's `checkpoint` answers a new name in
  `protocol/checkpoints.md`, 2.1's line, refused while the engine runs;
  and whose clock moves only by `sleep`. A test's description uses
  templates the fake reads.
- **Focused,** a case each: the description's refusals and quoting, a
  `backup` member and a `check` without `{checkpoint}` among them; a
  staged build left, a missing one fetched, a wrong answer refused; the
  engine alone with workers that may stay: the checkpoint taken and the
  check run over it before any stop, the stop's checkpoint by the
  installed binary, the verdict read again over it, then the swap; the
  checkpoint's act refused, and a check refusing, each ending refused
  with nothing stopped or swapped; a check saying no and unknown, the
  workers drained while the old engine serves, no engine started again,
  then the engine's stop and its checkpoint; the verdict turning no at
  the stop, the old engine started again and its openings waited for,
  then every part together with a second checkpoint as the backup; the
  verdict or the stop's checkpoint failing, the engine started again on
  its old build and the redeploy refused; `--workers` one at a time,
  each waiting for its logins; `--together` after a yes; the record's
  `checkpoint`, `verdict` and `backup`, and a failed redeploy's line
  naming the backup; a refusal at the opening; a step failing before and
  after the swap; the
  stop's lines read (4.2): a worker told, one not told, one with no
  line, each kept and printed, and the openings' bound the largest wait
  given; and the referee: agreed versions, a worker back within and past
  its restart wait and within and past its grace, a record without
  `told` judged by the graces, a record without `checkpoint` and
  `verdict` read, a lost try failing and a run's own not, a
  hold since failing and one from before passed over, the task list's
  pages followed, a ctl call failing.
- **Fuzzy,** seeded, over deployments of one to four workers, each mode,
  each verdict at the check and at the stop, refusals, and steps failing
  at random: the check before any stop, over the checkpoint the act
  named; no stop's `checkpoint` while the engine runs; no swap without a
  checkpoint at the settled stop before it, the record's `backup` that
  checkpoint; no binary swapped
  on a running part; no two workers stopped at once in turn; no engine
  stop while a worker runs in every part together; a refusal leaves
  every part running its previous build, nothing swapped; a done
  redeploy leaves every part it restarted on COMMIT; the record's lines
  are the lines printed. The parsers: any opening line, and any told or
  not-told line, the engine's renderer makes
  (`temper_engine_shell::lines::note_line`) parses back to its worker
  and wires, or its worker and wait, and any word quoted reads back as
  itself.
- **The binary's table test,** and its reference documents, as
  `testing/tools.md`, sections 4.4 and 6.

## 9. Later and open questions

- **Going back by the tool.** Default: the operator's, by `upgrade
  --back` or a restore of the backup, then the previous build swapped
  back and started (4.6).
- **The switch adopting the check's rehearsed generation**
  (`draft/checkpoints.md`, 8), so that a start with a long job does none
  of it again. Default: the start runs its jobs over the stopped state.
- **A worker before the engine** (`draft/redeploys.md`, 7.2). Default:
  the engine always goes first.
- **Workers in turn because a coming build's floor would pass them**
  (`draft/redeploys.md`, 7.2): the operator asks it with `--workers`.
  Default: the tool judges only the build it deploys.
- **A failure the redeploy caused after the referee's read** is the next
  `referee`'s. Default: run it again later, as the drill's report.
