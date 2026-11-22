# The box's binary

This document holds the box binary's commands and standalone processes, split from [`binaries.md`](binaries.md).

## 1. The box's command table

The box's binary, `temper-box` (`bin/temper-box`, its code
`crates/temper-box-shell`), ordinary Rust: a shell, copied from jig's
reference binary, the whole product, for trying temper and for one
person's development on one machine; a deployment runs a binary per
process (1). Its table, its sets, its help and its exits are
`../cli/README.md`'s (6); its commands, `temper ctl`'s as
`../client/ctl/README.md` designs them:

```
public         temper-box [--config DIR] [--data DIR] [--cache DIR] [--echo none|steps] [--record]
               temper-box init [DIRECTORIES]
               temper-box serve [DIRECTORIES] [--record]
               temper-box program-login [DIRECTORIES] NAME
               temper-box help [NAME | pages | --search WORDS...]
               temper-box versions
               temper-box upgrade --check DIRECTORY | --workers DIRECTORY | --back
               temper-box checkpoint [DIRECTORIES] [--hold NAME -- COMMAND ARGS... | --remove NAME]
               temper-box restore NAME [DIRECTORIES]
internal       temper-box [DIRECTORIES] --fake-backends [--bundle DIR] [--record]
               temper-box [DIRECTORIES] [--fake-backends [--bundle DIR]] [--record] --front
               temper-box init [DIRECTORIES] --box
               temper-box forge fetch|prepare|sign-in|source|remove [DIRECTORIES] FORGE
               temper-box forge run [DIRECTORIES] FORGE forgejo|runner
               temper-box ctl DIRECTORY GROUP VERB [ARGUMENT...] [--json]
               temper-box ctl DIRECTORY --call FILE [--json]
               temper-box ctl help [GROUP [VERB]]
               temper-box open DIRECTORY LINK_OR_ADDRESS
spawned        temper-box internal agent AGENT.json
               temper-box internal serve [DIRECTORIES] --box [--record]
               temper-box internal forge-child --parent PID --signal kill|terminate -- PROGRAM ARGS...
test doubles   temper-box fake llm|forge|oauth|git ...
```

- **Every command is a command of the binary's table,**
  `skein_shell::Table` (`../cli/README.md`, 6), as ctl's tools are
  (`../client/ctl/tools.md`, 1): its words, its forms, its line, its
  arguments with a line each, and its notes. Its reach, `temper-box
  --help`, `temper-box COMMAND --help`, the reference documents and the
  page `commands` derive from it.
- **What temper spawns itself** is under `temper-box internal`, internal,
  absent from `temper-box --help` and from every page but `commands`; a
  person or an agent never runs it.
- **The TUI is the client's,** `temper`'s, below: on an engine,
  `temper [ADDRESS]`, and in front of a box, `temper box`, which runs
  `temper-box --front` as its child (`setup/box-front.md`, 1;
  `../client/tui/box.md`). The box runs no TUI of its own.
## 2. The box's commands

`DIRECTORIES` is `[--config DIR] [--data DIR] [--cache DIR]`, given
anywhere among the command's arguments, each with its variable and its
default; the document is `temper.json` in the configuration directory
(configuration/directories.md).

- **`temper-box serve`:** reads the configuration (configuration/README.md,
  section 2) and runs the services its document's shape names, in one
  process on skein's `Host` and `drive` (jig's `protocol/service.md`,
  sections 4 and 10): standalone, the engine's and the worker's, the
  store in the deployment's directory; an engine alone, the engine's
  with its store and no worker; a worker alone, the worker's, with no
  store (`cluster.md`, 1).
  - Its stdout carries only sign-in links, one per line: each the
    secret jig's clients' component gives, in jig's default mapping's
    link route under temper's prefix (jig's `protocol/client/sign-in.md`
    ; `../client/README.md`, section 4); its stderr carries startup
    notes, the lines of `cluster.md`, 3 (each TCP listener once bound, and a worker
    alone's link at each opening) and the reason of a failure. A worker
    alone prints no sign-in link.
  - `--record`, off by default, a public switch its help names: the
    page it serves records in every person's browser from its load,
    with nothing added to its address, by the page's `temper-record`
    meta (`sign-in.md`, 7; `../client/web/README.md`, 2.1 and 2.2), so
    that a person can record a journey on the deployment and save it
    with Alt+Shift+S (`../client/web/journeys.md`, 3.1). Its help line:
    "Have every page served record a journey in its browser, saved by
    Alt+Shift+S." A worker alone serves no page, and takes it
    as it is, changing nothing. `temper-box` takes it on each of its
    forms, the fake backends' and `--front` among them (so `temper box
    --record` passes it), and gives it to its `internal serve --box`
    step; the engine's `temper-engine serve` takes it as the box's
    `serve` does (2).
  - `--box`, given by `temper-box` alone, as `temper-box internal serve
    --box`, adopts the box's pipe, the two fds after the listeners the
    box gives it (setup/box.md, 1). Given listeners, `LISTEN_FDS` of them
    from fd 3, its clients' and webhooks' listeners take them by their
    addresses (skein's `runtime/shell.md`, 6.6).
  - A first termination signal settles the engine's forge writes, then
    drains it (`../domain/root-restart.md`, 1.2; jig's `protocol/service.md`,
    section 5), and a worker alone's runs its worker's ending (jig's
    `protocol/service.md`, section 6); a second turns either into an
    abort, which keeps the reason of a failure that started the drain
    (a lost webhooks listener's, forge/engine.md, 3). A write the settling
    left without its outcome is printed as the drained engine ends, one
    line each, and the exit is still 0:
    `temper-box serve: stopped before task {task}'s forge write {entry} settled; the next start settles it`.
  - A start refused at a step of its restart (jig's `domain/engine/restart.md`
    ; `../domain/root-restart.md`, 1.1), which only a store that does
    not decode, a live set past the limits or a link that cannot be
    made can be, prints the refusal
    in the restart's own words, `temper-box serve: ` then the step and what
    it refused (`restore tasks: live tasks past limits.tasks`), and
    exits 1. The catch-all `unknown reference` is left for a refusal nothing
    names. The refused process ends whole: it closes every listener it
    holds, the box's given ones too, whether or not a component took
    one, and its ended root settles each worker's pending release at
    once, so neither a listener nor a worker keeps it open.
  - Under the box, a refused start is a long step's exit like any other
    (setup/box-supervision.md, 1): restarted after its backoff, its refusal printed
    at each exit, the box giving up, exit 1, past `box.restarts` within
    `box.window_ms`. No refusal is told apart as lasting. The box reads
    no step's reasons, and a retry is a restart, which makes nothing
    twice; a cause repaired during the backoff (a setting, the disk)
    lets the next attempt start.
  - A link a start cannot make (a writer's bug, jig's
    `domain/engine/restart.md`) is such a refusal, naming the record
    and what it links to: `temper-box serve: restart refused during PHASE:
    RECORD links missing LINKED`.
- **`temper-box init`:** writes a deployment that starts, the default
  document among it, into the three directories, refusing when the
  configuration directory holds a document (setup/init.md, 2).
- **`temper-box internal agent AGENT.json`:** smith's agent shell,
  `smith_agent_shell::run_decoded`, on smith's agent configuration as
  temper reads it, a `"fetched"` program naming its launcher
  (configuration/worker.md, section 2; setup/owned.md, 6). The worker's protocol layer spawns it,
  one process per run; a person never does. `temper-agent` runs the
  same modes as a binary of its own (3): this command and `internal
  agent-list` call `temper-agent-shell`'s run and list, under the box's
  words.
- **smith's agent modes,** `temper-box --mcp-relay 127.0.0.1:PORT` and
  `temper-box --program-child PARENT PROGRAM ARGUMENTS...`: temper's binary
  is its agents' own executable, so the agent shell starts each program
  process through it, its status check and each print run, and the
  program starts its MCP relay through it (smith's
  `protocol/programs.md`, 5.1 and 5.3). `main` gives either flag, as
  the first argument, to smith's agent shell,
  `smith_agent_shell::relay::main` and
  `smith_agent_shell::program_child::main`, before it reads the
  binary's table, as `smith-agent` does. Both are internal rows of the table
  (`../cli/README.md`, 6), absent from public `temper-box --help`; a person never runs them.
- **`temper [ADDRESS]`:** a person's full-screen terminal client, the
  TUI (`../client/tui/README.md`, 1). Bare `temper`, with stdin and
  stdout both terminals, runs it at the address it last signed in to,
  else at `http://127.0.0.1:8367`; with either not a terminal, it prints
  `temper: no command; temper --help lists them` and exits 1, as before
  the TUI. ADDRESS is the engine's clients' listener on loopback; the
  sign-in link is pasted on the TUI's screen, never given. Its
  directory is its own, under the person's state directory
  (`../client/tui/README.md`, 2). It reads no configuration file.
- **`temper-box open DIRECTORY LINK_OR_ADDRESS`:** a person's terminal client
  (`../client/line.md`, 1). It reads no configuration file.
- **`temper ctl DIRECTORY ...`:** the agents' tools, one call per
  process, over the same native transport as `temper-box open`
  (`../client/ctl/README.md`, section 6). It reads no configuration file.
  Their rows, and run as `claude`'s, are `temper-agent-shell`'s
  constants, which the agent's table holds too (3).
- **`temper-box versions`:** what this binary speaks and keeps, one line
  each: every owner's explicit level (`floor.md`, 3), its
  families' and the store's versions, and its wires, as jig's store
  keeps them (jig's `protocol/store/records.md`, 3). It reads no
  configuration and no store. Its row and its lines are
  `temper-engine versions`', the same constant run by the same code (2).
- **`temper-box upgrade --check DIRECTORY`:** a rehearsal (jig's
  `protocol/store/opening.md`, 5) over DIRECTORY, a stopped deployment's
  data directory, a checkpoint of a deployment
  (`DEPLOYMENT/checkpoints/NAME`, `checkpoints.md`), or a copy of one
  such as last night's backup, which it never writes and whose lock it
  holds while it reads, so that a checkpoint it reads is not removed
  under it (jig's `protocol/store/checkpoints.md`, 6). Over a
  checkpoint it runs while the engine serves the deployment, the
  checkpoint an owner's act took (`checkpoints.md`, 3), so that a
  redeploy checks the new build before anything stops: nothing before
  the rehearsal takes the deployment's lock or writes under `--data`. It
  works in a scratch under the cache directory, `upgrade-SEED`, SEED
  sixteen lowercase hex digits from a fresh seed, with the
  configuration `temper-box serve` would read, and holds it under io's
  lock (the `flock` jig's store takes on a root, jig's
  `protocol/store/opening.md`, 1.1) from making it until it removes it.
  It makes the scratch as `upgrade-SEED.new`, a name the sweep below
  never matches, takes its lock, then renames it `upgrade-SEED`: the
  lock moves with the directory, so no rehearsal sees a scratch of this
  build unlocked. Once its rehearsal ends, passed, refused or failed,
  it removes its scratch whole, files left by a close that could not
  empty it included, while it still holds the lock; one it cannot
  remove stays, named in a line after the rehearsal's others, `scratch
  not removed: NAME: errno N`, and its exit is the rehearsal's.
  Before making it, it sweeps the cache directory: it removes, whole,
  each directory named `upgrade-SEED` whose modification time is an
  hour or more before its clock's reading and whose lock it then takes,
  a scratch left by a rehearsal killed partway, whose links would
  otherwise keep a checkpoint's files on disk past its retention. It
  reads the time before it takes the lock, and leaves each whose lock
  is held, a rehearsal's in use, and each modified within the hour or
  after its clock's reading, which a rehearsal by a build before this
  rule may still be filling: such a build makes `upgrade-SEED` itself
  and takes its lock only at its store's open, after the whole copy,
  and a removal that stopped partway would leave it working over
  missing files. It leaves `upgrade-SEED.new` too: one left by a
  rehearsal killed before its rename is empty. A leftover whose open,
  time or removal fails, or whose lock fails other than by being held,
  but for one gone since it listed the cache directory, stays, named in
  a line before the rehearsal's report, `scratch left behind not
  removed: NAME: errno N`, and the rehearsal goes on. In these lines,
  and in `scratch: errno N`, the failure to make or lock a scratch that
  ends the rehearsal, an error that carries no errno is named by its
  kind, `kind KIND`, KIND the name of its `std::io::ErrorKind` variant
  (`kind Other`). The open
  registers layouts, runs jobs and checks as a start would, restores,
  and runs on to the opening with no network (jig's
  `protocol/store/opening.md`, 5, *On to the opening*): the switch and
  the wires inside the scratch, the runs adopted with no worker, the
  calendar given its periods, decisions opened, and the forge's
  catching up begun with every forge call answered `Unavailable` at
  once, its connection closed as the check prepares
  (`../domain/forge/catching-up.md`); it ends at the opening. After the floor check and before
  its plan it decodes every record by its registry layout, or by the
  current text for a family outside the registry (jig's
  `protocol/store/opening.md`, 5, "Every record decoded"); the first
  that does not decode is printed, `record not decodable: {family}
  {version}, {map} key {hex}` (a frame: `{family} {version},
  transcript {name} at {offset}`), and the rehearsal ends there, exit
  1; else `decoded {records} records`. It prints a line per `Event::Ran`
  job as a start's (below), ending `; time unavailable`, and the
  `whole rehearsal/tool time: {nanoseconds} ns`. It prints the refusal
  a start would meet, in a start's words, or `registry commit, jobs and
  restores pass; opened in {milliseconds} ms`, the time from the
  store's open to the opening on the check's clock; then, for each
  forge kept and not ended, by number, `forge {n} would catch up:
  {words}`, its `CatchingUp` at the opening in
  `../domain/forge/catching-up.md`, 5's words after "Catching up: ",
  or `forge {n} would catch up: nothing` where every count is zero; then
  whether each worker may stay, by the opening the copy keeps of it,
  read before the check's own wires commit,
  and whether the workers may: yes when every worker the configuration
  names may, no naming each that may not and why, unknown when
  the copy keeps no wires (`wires.md`, 10.2; `../draft/upgrades.md`,
  section 3.1; `../draft/redeploys.md`, 5.1). Exit 0 when the
  start opens, 1 when a start would be refused before it.
- **`temper-box upgrade --workers DIRECTORY`:** `--check`'s verdict on
  the workers alone, over the checkpoint taken at the engine's settled
  stop (`../domain/root-restart.md`, 1.2; `checkpoints.md`, 2.1), and over the
  check's checkpoint while the engine serves, so that a build that
  cannot read the verdict at the stop is found before anything stops: a
  rehearsal over DIRECTORY, as `--check`'s, of jig's reach `Openings`
  (jig's `protocol/store/opening.md`, 5), which makes the active
  generation's maps in its scratch without their task logs, linking
  the snapshot and every segment but the newest and copying the rest,
  recovers them and reads the
  wires and the workers' openings they keep, and ends there: no record
  decoded, no level checked, no plan, job or restore run. It prints
  how many files its rehearsal linked and how many it copied, from
  `Rehearsing` (jig's `protocol/store/opening.md`, 5), a link the file
  system refused counted as a copy:

  ```
  temper-box upgrade: rehearsal linked {linked} files, copied {copied}
  ```

  then `--check`'s lines from its verdict's alone (`wires.md`, 10.2,
  steps 1 to 4), in their words and order, and exits 0 once they are
  printed, whatever the verdict. The count is the first line that
  starts so; no later line carries one. A refusal before
  the rows are read (DIRECTORY in use, an unfinished restore, an
  unreadable `active`, a copy or a recovery failing) is printed as
  `--check` prints a refusal, exit 1.
  It never writes DIRECTORY.
- **`temper-box upgrade --back`:** makes the previous generation active
  again (jig's `protocol/store/opening.md`, 5), for a binary that upgraded and
  failed before its first commit of its own: the deployment stopped,
  and the old binary started after it. Refused, changing nothing, once
  the upgraded generation has committed, with no previous generation, or
  while a process holds the deployment, each in words that say so; past
  the first commit, going back is restoring a backup
  (`../draft/upgrades.md`, 3.2).
- **`temper-box checkpoint`:** a checkpoint of the stopped deployment,
  its state at its last commit in `checkpoints/`, mostly by hard links;
  `--hold NAME -- COMMAND` runs a command under that checkpoint's lock,
  and `--remove NAME` removes it (`checkpoints.md`, 2). While the engine
  serves, an owner takes one with `temper ctl DIR deployment checkpoint`
  (`checkpoints.md`, 3).
- **`temper-box restore NAME`:** makes the checkpoint NAME's state the
  stopped deployment's, after a safety checkpoint of the state it
  replaces, crash-safe: a start after a cut finishes it
  (`checkpoints.md`, 2.5). Undoing it is restoring the safety
  checkpoint, and `upgrade --back` refuses after it with no previous
  generation.
- **A start that runs a job at open** prints a line per `Event::Ran`
  job as it ends,
  `temper-box serve: ran {owner} {job}: {rows} rows in {commits} commits`,
  and, for a job that counts (jig's `protocol/store/jobs.md`, 3), `; `
  then each of its counts as `{name} {count}`, joined by `, `, in the
  job's order, zeros included:
  `temper-box serve: ran temper deleted-repository-forge-rows: 812 rows in 2 commits; dropped 5, BranchHead 1, PullState 2, Ci 0, Client 0, Receipt 2, Tier 0, Verification 0, LandingLog 0, Application 0, Names 0`
  (`store-versions.md`, 1). `upgrade --check`'s line is the same with `upgrade`
  for `serve`, then `; time unavailable`.
- **`temper-box program-login NAME`:** a program's login for
  the program source NAME, on this host: it checks the document and the
  agent's configuration, makes the source's configuration directory, and
  replaces itself by the program's own sign-in (credentials-llm-accounts.md, section
  1). The engine need not run.
- **The forge's setup** (setup/README.md), FORGE a forge's number, as its
  record has it (forge/engine.md, 1) and the Forges page shows it; reconcile
  is the engine setup host's plan (forge/engine.md, 6). The box knows kinds,
  not forges: `fetch`, `prepare` and `run` read the forge's kind and
  setup entry from its group's file, `<data>/box/groups/FORGE`, which
  `temper-box serve` writes as it asks for the group (setup/box.md, 1), a
  forge with none taking the machine's `owned` part (setup/box.md, 4), and
  each reads and checks the whole document as `temper-box serve` does
  (configuration/startup.md, section 1):
  - `temper-box forge prepare`: an owned forge's preparation, before its
    Forgejo runs; it ends when done (setup/owned.md, 3);
  - `temper-box forge run`: checks the binary and the preparation, takes
    the forge's lock, and replaces itself by `forgejo web` or
    `forgejo-runner daemon`, whose process a host supervises (setup/owned.md,
    4);
  - `temper-box forge remove`: an owned or fake forge's directory under
    data removed, `forge-N` or the playground's, by the kind its group's
    file names: the box's remover step, run once a removed forge's group
    has stopped (setup/box.md, 1; forge/engine.md, 3 and 8). It refuses a
    path outside data or one within the cache, which it keeps,
    Forgejo's binaries with it, and a directory already gone is no
    failure; it reads no document and touches no store, so a forge is
    removed from the client, never by this command alone;
  - `temper-box`: its steps as child processes, `temper-box serve` and the
    groups it has started for forges, each by its kind (setup/box.md, 1),
    in order, restarted when they exit and stopped in reverse order; its
    stdout is `temper-box serve`'s sign-in links, each line whole; its
    stderr, a terminal's, the box's own lines alone, each step's lines
    in its log under the data directory, and on stderr too, prefixed by
    the step's name, with `--echo steps` (setup/box-supervision.md, 1).
- **Exit status:** 0 after a drained shutdown, a worker alone's ending
  or a finished command; 1 on
  any failure (a refused start, a usage error, a failed commit, an
  abort, a failed step of the setup, the box giving up), the reason on
  stderr. smith's agent shell exits 0 or 1 the same way; `temper ctl`'s
  statuses are `../client/ctl/results.md`, 3; `temper-box forge run`'s is Forgejo's or
  the runner's own, and `temper-box program-login`'s the program's sign-in's
  once it has replaced itself.

## 3. Standalone processes

Standalone, one person's machine:

```
temper-box serve                           one process, one loop: temper-service, both halves composed (services.md, 6)
├── the engine's service               section 6
│   ├── listens: the in-process name "workers"; the clients' listener and, when configured, the webhooks' listener, TCP on loopback
│   └── connects: each kept forge's API (HTTP; TLS unless the forge is on loopback)
├── the worker's service               section 7
│   └── dials: the in-process name "workers"
├── temper-box internal agent AGENT.json   a child process per run, in a process group of its own
└── git ...                              a child process per git invocation, in a process group of its own (git.md, section 7)
temper-box open DIRECTORY LINK_OR_ADDRESS  a process of its own, reaching the engine on loopback
temper ctl DIRECTORY GROUP VERB ...    a process per call, reaching the engine on loopback
temper-box program-login NAME              run by hand on a host, replaced by the program's sign-in
temper-box versions                        prints this binary's levels, versions and wires
temper-box upgrade --check DIRECTORY       rehearses an upgrade over a copy; switches nothing
temper-box upgrade --workers DIRECTORY     judges the workers alone over links and small copies; switches nothing
temper-box upgrade --back                  makes the previous generation active again, before the upgraded one's first commit
temper-box checkpoint                      takes a checkpoint of the stopped deployment; --hold runs a command under one's lock, --remove removes one
temper-box restore NAME                    restores a checkpoint over the stopped deployment, after a safety checkpoint of it
```

A box runs `temper-box serve` as one of its steps, after the forge's setup's
commands, each a child of its own (setup/box.md, 1). The cluster's
processes are `cluster.md`, 2's.

## 4. Open questions

None.
