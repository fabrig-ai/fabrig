# The redeploy

Provisional, 2026-11-13. How a deployment of temper takes a new build
while its work goes on, and how each redeploy is refereed afterwards:
`temper-redeploy`, an internal tool run by the deployments' operator
from a machine that reaches the deployment's hosts. It carries
`draft/redeploys.md`, section 7, milestone R6 of
`draft/redeploys.execution.md`. What it drives is designed elsewhere:
the engine's settled stop (`domain/root.md`, 8.2), a worker's drain
(`domain/worker/README.md`, section 2; jig's `domain/hosts/hub.md`, 7),
`upgrade --check` and its verdict on the workers (`protocol/binaries.md`,
section 2; `protocol/wires.md`, 10.2), the workers' grace
(`protocol/configuration/README.md`) and an agent pinned at its worker's
start (`domain/worker/README.md`, section 5). This document says what
the tool runs, in what order, what it keeps and how it judges.

## 1. In one page

- **A description of the deployment** (section 2), kept by its operator
  outside the repository: each part's host, reached by a command prefix
  such as `ssh`, and the command lines that stop, start, swap, check,
  back up and read each part. The tool names no machine.
- **The build staged first** (section 3) on every host, beside what runs,
  each binary asked its `--version`.
- **The engine alone, by default** (4.2, 4.3): its settled stop, the new
  build's `upgrade --check` over the stopped deployment, and, when it
  says the workers may stay, the swap and the start the workers ride.
- **Workers in turn, when asked** (4.4): one at a time, drained, swapped,
  started, and waited for until it opened and its logins came, before
  the next.
- **Every part together** (4.5) when the check says a worker may not
  stay, or when asked: the old engine started again first, so that every
  worker drains to a live engine, then the engine's settled stop, every
  binary swapped and all started.
- **Nothing half done before the swap** (4.6): a failure until the new
  engine is installed starts again what was stopped, on its old build;
  after it, the tool stops where it stands and says how to go back.
- **A record** of the redeploy (section 5), written as it goes, which the
  referee reads.
- **The referee** (section 6): each worker's agreed versions; the window
  against the workers' grace; no try spent and no task held for a
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
    "check": "sudo -u temper {binary} upgrade --check --config /var/lib/temper/config --data /var/lib/temper/data --cache /var/lib/temper/cache /var/lib/temper/data/deployment",
    "backup": "sudo -u temper cp -a /var/lib/temper/data/deployment /var/lib/temper/backups/{name}",
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
  engine's stopped deployment directory, as the deployment's own user,
  with the directories the engine's `serve` is given
  (`protocol/binaries.md`, section 2).
- **`backup`** (the engine's): copies the stopped deployment's data to a
  place named by `{name}` (`draft/upgrades.md`, 3.3).
- **`swap`:** installs `{from}` as `{to}`.
- **`ctl`** (the engine's): `temper ctl DIR` for a ctl directory on the
  engine's host, its endpoint loopback (`client/ctl/README.md`), signed
  in as a person holding a role in each project the referee is to read
  (6.1).
- **`document`** (a worker's): prints the worker's configuration
  document, from which the tool reads its grace (4.1).
- **`workers`:** in the order workers go in turn (4.4), at least one, each
  `name` the worker's name as the engine's configuration and its opening
  give it.

### 2.2 Commands on a host

- **Placeholders** are `{since}` (the Unix time in whole seconds),
  `{binary}`, `{name}`, `{from}` and `{to}`. `log` requires `{since}`,
  `check` `{binary}`, `backup` `{name}`, `swap` both `{from}` and `{to}`;
  `stop`, `start`, `ctl` and `document` take none. Each value goes in
  quoted for a POSIX shell, `'` around it and `'\''` for each quote
  within.
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

A failure here refuses the redeploy with nothing stopped, exit 1.

### 4.2 The settled stop and the check

- **The engine's `stop`** at the window's start (6.2). The workers keep
  their runs, their grace running from their links' loss.
- **The check:** the engine's `check`, `{binary}` the staged
  `BUILDS/COMMIT/temper-engine`, run once per redeploy, at its first
  settled stop. Each of its lines is printed and kept. It never writes
  its input (`protocol/binaries.md`, section 2).
- **What it says** (`protocol/wires.md`, 10.2), by its exit and its last
  line ending `workers may stay: `:
  - exit other than 0: a start would be refused. The engine started
    again on its old build, its openings waited for (4.7), and the
    redeploy ends **refused**, exit 1.
  - exit 0, `workers may stay: yes`: the engine alone (4.3).
  - exit 0, `workers may stay: no`, `workers may stay: unknown…`, or no
    such line: the engine started again on its old build, its openings
    waited for, then every part together (4.5).
- **A backup** before the swap when the check's lines say it patched (a
  line holding `upgrade: patched `): the engine's `backup`, `{name}`
  `COMMIT-YYYYMMDDTHHMMSSZ`, the time the backup's, UTC.

### 4.3 The engine alone

1. The settled stop and the check (4.2), then the backup if any.
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

`draft/redeploys.md`, 7.3:

1. every worker's `stop` at once, the tool waiting for all: each drains
   to a live engine;
2. the engine's `stop`;
3. the check, unless this redeploy ran it (4.2), its exit and backup as
   there; a refusal starts the engine and then every worker again on
   their old builds and ends **refused**;
4. the engine's binaries swapped, then each worker's;
5. the engine's `start`, then every worker's;
6. every worker's opening waited for (4.7).

The outage runs from the first worker's `stop` to the last opening.

### 4.6 A step that fails

- **A step fails** when its command line exits other than 0, cannot be
  spawned, or its wait passes its bound.
- **Until a binary is swapped,** a failure starts again every part the
  redeploy stopped, on its old build, waits for their openings, and
  ends **refused**, exit 1: nothing is half done.
- **Once one is,** a failure ends the redeploy **failed** where it
  stands, exit 1: its line names the step, the parts swapped, the backup
  and the previous build; going back is the operator's, by `run` with
  that build, or after the patched generation's first commit a restore
  of the backup (`draft/staged-upgrades.md`, 7, step 5).
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
- **The bound:** the largest grace among the workers waited for (4.1).
  A worker unopened by then fails the step, named. Openings of a name
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
| `before` | the referee's snapshot (6.1): `projects`, the project numbers read; `cut`, whether ctl cut the memberships; `tasks`, each `{"task", "project", "label", "phase", "held"}`, `held` the hold's failures class or null |
| `lines` | every line the run printed, each `{"at", "line"}` |
| `check` | `{"code", "lines"}`, or null before it ran |
| `backup` | its `{name}`, or null |
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
2. **The window,** when the workers rode an engine's restart: its
   length against the smallest grace among the workers; at or past it
   fails. Outages are printed, not judged: a worker restarted in turn or
   every part together ends its runs interrupted, spending no try.
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
window SECONDS s within NAME's grace SECONDS s
window SECONDS s past NAME's grace SECONDS s
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

`run`'s lines name each step as it ends: the builds before, each stop
with its time, the check's lines as it printed them, what it decided,
the backup, each swap with its builds, each start, each opening with
its time and agreed versions, each worker's logins, and the outcome,
`done`, `refused: WHY` or `failed at STEP: WHY`, the last naming what is
swapped, the backup and the previous build.

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
  refusal, as a test scripts it; whose `check` and `ctl` answer what a
  test scripts; and whose clock moves only by `sleep`. A test's
  description uses templates the fake reads.
- **Focused,** a case each: the description's refusals and quoting; a
  staged build left, a missing one fetched, a wrong answer refused; the
  engine alone with workers that may stay; a check refusing, the old
  engine started again and nothing swapped; a check saying no and
  unknown, the old engine started again, then the workers drained before
  the engine's stop; `--workers` one at a time, each waiting for its
  logins; `--together`; a patching check's backup before the swap; a
  refusal at the opening; a step failing before and after the swap; and
  the referee: agreed versions, a window within and past the grace, a
  lost try failing and a run's own not, a hold since failing and one
  from before passed over, the task list's pages followed, a ctl call
  failing.
- **Fuzzy,** seeded, over deployments of one to four workers, each mode,
  each verdict, refusals, and steps failing at random: no binary swapped
  on a running part; no two workers stopped at once in turn; no engine
  stop while a worker runs in every part together; a refusal leaves
  every part running its previous build, nothing swapped; a done
  redeploy leaves every part it restarted on COMMIT; the record's lines
  are the lines printed. The parsers: any opening line the engine's
  renderer makes (`temper_engine_shell::lines::note_line`) parses back to
  its worker and wires, and any word quoted reads back as itself.
- **The binary's table test,** and its reference documents, as
  `testing/tools.md`, sections 4.4 and 6.

## 9. Later and open questions

- **Going back by the tool.** Default: the operator's, by `run` with the
  previous build or a restore (4.6).
- **Knowing the verdict before the stop,** sparing the old engine's
  restart on a no: a worker would need to state its wires off its link.
  Default: the check after the settled stop (4.2).
- **A worker before the engine** (`draft/redeploys.md`, 7.2). Default:
  the engine always goes first.
- **Workers in turn because a coming build's floor would pass them**
  (`draft/redeploys.md`, 7.2): the operator asks it with `--workers`.
  Default: the tool judges only the build it deploys.
- **A failure the redeploy caused after the referee's read** is the next
  `referee`'s. Default: run it again later, as the drill's report.
