# The box

## 1. What it runs

`temper-box [DIRECTORIES] [--echo none|steps] [--front]` runs its steps, which
`temper-forge-setup`'s `kinds` and `temper-fake`'s fake slot give from
the document (`init.md`), each a command of the `temper-box`
binary, as a child process given the box's three directories as flags,
`--config C --data D --cache K`, each absolute (../configuration/directories.md
), written `DIRS` below, `N` a forge's number:

```
temper-box
│                                     the fake kind's group, the playground's fakes: asked (below)
├── temper-box fake llm DIRS              long (8.4)
├── temper-box forge prepare DIRS N       once: the fake forge's handover
├── temper-box fake forge DIRS N          long (8.4)
│                                     the owned kind's group, an owned Forgejo's: asked (below)
├── temper-box forge fetch DIRS N         once, when its binaries are fetched (6.6)
├── temper-box forge prepare DIRS N       once
├── temper-box forge run DIRS N forgejo   long: Forgejo's parent on Linux with fd 3 (6.4)
│   └── forgejo web                   via temper-box internal forge-child; execs without fd 3
├── forgejo-runner daemon             long: temper-box forge run DIRS N runner
├── temper-box forge source DIRS N        once, then again until the source serves, when the setup file's sign_in is set (`place.md`, 6)
└── temper-box internal serve DIRS --box  long: its own tree (../binaries.md); its setup
                                      host takes each handover and reconciles (`box-backends.md`, 1);
                                      its listeners from fd 3, then its pipe to the box (below)
```

- **With `--front`,** the box is the child of `temper box`'s TUI, its
  link and its lines on the descriptors its parent hands it, and nothing
  on its terminal (`box-front.md`, 1). It runs no TUI of its own.
- **A step's start.** On Linux the box spawns each step through `temper-box
  internal forge-child --parent BOX --signal terminate -- temper-box ARGS`, BOX
  its own process id (`owned.md`, 4). That process execs the step, keeping its
  descriptors, its environment and its process group, so nothing else of
  this section sees the difference. A box that ends in any way, killed
  among them, leaves each step a `SIGTERM`, which it takes as on any stop.
  **On macOS** there is no parent-death signal (`owned.md`, 4): the box spawns each
  step directly, so a box killed outright leaves them running, and its
  next start refuses on the ports they hold.
- **An empty configuration directory,** absent or with no entry, is
  written first by `temper-box init --box` (`init.md`, 5), in the box's own process, before
  the document is read; the box says so in its own log, `First run:
  wrote the configuration, <path>` (`box-supervision.md`, 1), and goes on to its steps.
  Otherwise the box
  never writes the document: a configuration directory with entries but
  no document is refused as every command refuses it.
- **The box knows kinds, not forges.** It has a group for each kind:
  the owned kind's, an owned Forgejo's fetch, preparation, Forgejo and
  runner (`temper-forge-setup`'s steps); and the fake kind's, the
  playground's fake LLM, a fake forge's preparation and the fake forge
  (`temper-fake`'s), in every box, the fake LLM on the listener the box
  holds from its start for the scripted provider
  (`../configuration/providers.md`, 4). Each
  group's place and setup are its forge's setup file's (`place.md`, 3). Each is
  dormant until serve asks for it for a forge, and then runs for that
  forge alone, its steps' forge number that forge's: a box runs at most
  one group of each kind. It reads no store and names no forge of its
  own: it stays wiring only.
- **A forge's group, asked.** Under the box, serve writes the forge's
  setup file (`place.md`, 3), with the port it chose, the box writes the
  group's kind to `<data>/box/groups/N`, private, and serve asks `start N` on the pipe:
  at an owned forge's set up (`owned.md`, 5) and at the playground's start (`playground.md`).
  The box builds the group's steps by the kind, opens the group's
  listener at its setup file's port (`place.md`, 5), and runs the steps
  in order; `temper-box forge fetch`, `prepare`, `run` and `remove`
  read forge N's setup from its setup file (`place.md`, 3).
  Once the group's long steps have started and, for an owned Forgejo,
  Forgejo answers, the box writes its record, `<data>/box/started/N`, one empty file, and
  answers `started N`. `box`, `box/groups` and `box/started` are made as
  needed, 0o700, their files 0o600. A record that cannot be written is
  named on the box's stderr, and the group answers `failed N record not
  written` and is dormant again.
- **At a later start,** the box lists `box/groups`, at most `box.groups`
  files (../configuration/limits.md), and runs each group whose
  `started` record exists from its file, in its place, as an unasked
  group, before `temper-box serve`, or just after it when the box signs its
  person in through that forge: a later preparation's source commands
  and Forgejo, loading its source, fetch discovery from temper
  (`../openid-provider.md`, 9), and serve takes the handover later for it (`owned.md`, 5). A
  group with a file and no record stays dormant until asked. A file it cannot read, or a record it cannot look
  at, is named on its stderr, and its group stays dormant; of two
  started groups of one kind, the one of the higher number runs, and the
  other is named and stays dormant.
- **A newer build at a later start.** A stopped box's directories start
  with another build installed at the same paths, the files of
  `temper-box`, `fake-claude` beside it and the bundle's directory
  replaced while it was stopped. Every program the box runs is its own
  executable, run again for its steps and its agents' host, or a path
  its configuration names, never a copy in its data but one: its agents
  run the worker's pin of that executable, copied into the worker's
  `agent/` afresh as the worker starts, the previous copy removed first
  (`../../domain/worker/agent-processes.md`, 1), so a start pins the build then
  installed. So the engine, its agents, its adapters and the served
  bundle all run the new build, and nothing the old. Its forges, an owned Forgejo or the fake forges with
  their state (`box-backends.md`, 1), its repositories, pull requests, checks and tasks come
  back as they were; a store the build cannot read refuses serve's start by
  name (`../store-restart.md`, 1), a step's failure as `box-supervision.md`, 1 has it. The process
  tier proves it (`../../testing/process-tier.md`).
- **A forge's group, removed.** At a forge's removal
  (`../../domain/forge/adoption-connection.md`, 1) serve asks `remove N`, and again at each of
  its starts until it is answered `removed N`. The box stops the group's
  steps as it stops a group (`box-supervision.md`, 1), then runs its remover, `temper-box forge
  remove DIRS N`, a once step, which removes the forge's directory,
  `<data>/forge-N` or `<data>/playground-N`, and refuses any path
  outside the data directory or within the cache, so the cache, and
  Forgejo's fetched binaries in it, stay (`owned.md`, 6). The box then removes
  `box/groups/N` and `box/started/N`, and answers `removed N`. For a
  forge with no group's file it knows, it removes those files alone. A
  removal that fails answers `failed N WHY`, `WHY` one of `remove:
  exited <code>`, `remove: did not start` and `remove: files not
  removed`, and the group is dormant again, to be asked again.
- **The listeners are the box's,** every step's, from the box's start,
  or a group's first ask (`place.md`, 5), to its end: so that the port a step
  reports is its own from before it starts, and a second first run's
  init finds the first one's ports taken (`init.md`, 5). Each step names its
  listeners, the socket addresses the document gives it (3): `temper-box
  serve` the clients' and the webhooks' listeners, the fake LLM and the
  fake forge their ports, and, on Linux, `temper-box forge run N forgejo` the
  owned Forgejo's port, its setup file's (`place.md`, 3; `owned.md`, 4). On a
  first run the box keeps the sockets `temper-box init` chose (`init.md`, 5); on a later
  start it opens each at the document's address before any step, with
  skein's `open_listener`, and a port taken refuses its start, naming
  the port and the document, as any start is refused. A box started with
  listeners of its own (`LISTEN_FDS`, systemd's convention, as `temper-box
  internal serve`, `temper-box fake` and `temper-box forge run` read it: skein's
  `runtime/shell-startup.md`, 1.6) takes each one given for a step's address, matched by
  its address, and opens only the rest, so a person's socket activation
  and a test's held port reach the box's steps as they reach serve; one
  given for no step's address refuses its start, naming it. It holds each
  one, never accepting, and gives it to its step at every spawn, a
  restart's among them, as the descriptors 3, 4, … in the step's order,
  with `LISTEN_FDS` set to their count in the step's environment and
  dropped from its own: systemd's convention, which skein's shell reads
  (skein's `runtime/shell-startup.md`, 1.6) and Forgejo reads as well (`owned.md`, 4). The step's
  own `Listen` for that address then takes the socket it was given
  (skein's `runtime/io/sockets.md`, 2), so nothing above io knows it was given one. A
  step with no listener is given none, and no `LISTEN_FDS`. On Linux,
  `temper-box forge run N forgejo` is given the owned Forgejo's port and
  stays Forgejo's parent (`owned.md`, 4); on macOS it is given none, and a fresh
  box closes the reservation `temper-box init` made for its port, so
  Forgejo binds it itself. A listener at port 0 is not held: its step binds it, as without the box, since a
  port the kernel chooses can't collide. A held one could not be claimed
  either, since the step's `Listen` names port 0, not the port chosen.
  It changes at each restart, as it does without the box. `LISTEN_FDS`
  counts only the sockets given.
- **The pipe,** two of them, made again at each spawn of `temper-box
  internal serve`, at the two descriptors after its listeners: 3 + *n*,
  serve to box, and 4 + *n*, box to serve, *n* its listeners' count, so
  fds 5 and 6 under the default document's two. `--box` tells `temper-box
  internal serve` to adopt them, and a serve given `--box` whose
  descriptor 3 + *n* or 4 + *n* is not an open pipe refuses its start,
  naming the fd, *n* being the `LISTEN_FDS` it was given, 0 without it
  (a check of serve's own descriptors' list, `/proc/self/fd` on Linux and
  `/dev/fd` on macOS, `../../testing/macos.md`, 3, made on both systems
  before serve reads its standard input or its configuration);
  without `--box`, as under every other arrangement (4), serve has no
  pipe and leaves those descriptors alone. One line each, ending in a line
  feed, `FORGE` the forge's number:

  | Line | From | Says |
  |---|---|---|
  | `start FORGE` | serve, once it has written the group's file: at an owned forge's set up (`owned.md`, 5) and at the playground's start (`playground.md`) | start the forge's group, by its file |
  | `step FORGE STEP` | the box | the group's step `fetch`, `prepare`, `run` or `source` is starting, for the client's row (`../../client/forge/deployment.md`) |
  | `fetched FORGE BINARY DONE TOTAL` | the box | how far the group's `fetch` step has gone, one for each line it says (`owned.md`, 6): `BINARY` `forgejo` or `runner`, `DONE` its bytes written, `TOTAL` its size or `none`. Between `step FORGE fetch` and the next step alone (`box-supervision.md`, 1), and at most 64 bytes, `box.line_bytes`'s least, whatever its numbers |
  | `source FORGE added`, `source FORGE failed WHY` | the box | how the group's `source` step ended (`../openid-provider.md`, 9), before its `started`: `WHY` as `failed`'s, the step's own words or `source exited N`, `source did not start`, within the line's bound, holding no secret. A `start` for a group already standing says its last such line again, before its `started`. A retried step's ending is said again, after the `started`, only when it differs from the last one said, so `source FORGE added` follows a `failed` once the source serves |
  | `started FORGE` | the box | the group's long steps are started, an owned Forgejo answers, and its record is written; the engine then takes the handover and reconciles (`owned.md`, 5) |
  | `failed FORGE WHY` | the box | the group stopped: `WHY` names the step by its name, as its log file and the box's terminal name it (`box-supervision.md`, 1), such as `prepare-3 exited 1`: `<step> exited <code>`, `<step> killed by signal <n>`, `<step> did not start`, `record not written`, or a removal's `remove: exited <code>`, `remove: did not start` or `remove: files not removed`, cut to fit the line's bound, and holds no secret; a step's own lines are in its log, the last of them under the terminal's `failed` line (`box-supervision.md`, 1) |
  | `stop FORGE` | serve | stop the forge's group, its file and record kept |
  | `stopped FORGE` | the box | the group's steps are closed, as the box's own stop closes them (`box-supervision.md`, 1), and the group is dormant again; its file and record stay |
  | `remove FORGE` | serve, at the forge's removal (`../../domain/forge/adoption-connection.md`, 1), and at each of its starts until answered | stop the forge's group and remove its directory, its file and its record (above) |
  | `removed FORGE` | the box | the group's steps are closed, and its directory, its file and its record removed; the cache is kept |

  One ask at a time: a `start` for another forge while a group starts,
  or while a failed group's steps close, is answered `failed FORGE busy`;
  one for the forge whose group is starting, or whose failed steps
  close, by that group's own lines, its `failed` once they have closed;
  one for a forge with no group, `failed FORGE not asked`; one for a
  started group, `started FORGE`; one while the box stops,
  `failed FORGE stopping`. A `stop` is an ask as a `start` is, one at a
  time: for a started group, the box closes its steps, then answers
  `stopped FORGE`; for a dormant one, `stopped FORGE` at once; for a
  group starting, or whose failed steps close, `failed FORGE busy`; for
  a forge with no group, `failed FORGE not asked`; and while the box
  stops, `failed FORGE stopping`. A `remove` is an ask as a `stop` is,
  answered `removed FORGE` where a `stop` is answered `stopped FORGE`,
  once its remover has run and its files are removed; for a forge with
  no group's file the box knows, it removes the files and answers at
  once. While a group's steps close at its `stop` or `remove`, or its
  remover runs, a `start`, a `stop` or a `remove` for any forge is
  answered `failed FORGE busy`. An
  answer to a `temper-box serve` that has exited is dropped: its restart
  finds the handover at its start (`owned.md`, 5). Each line is at most
  `box.line_bytes` (../configuration/limits.md), which both sides
  read within; another line, or a longer one, is named on its reader's
  stderr and ignored. serve's side, its asks, its reads and the states
  they tell the engine's root, is the routing's box link (../forge/engine.md,
  8).
- **The first link's secret** (`box-supervision.md`, 1) reaches `temper-box internal serve`'s
  first start on its standard input, a pipe of its own: the box writes
  the secret's 32 bytes, nothing around them, and closes it; with
  `--front`, the printed link's 32 bytes and then the first local
  link's 32, whose link the box hands its parent (`box-front.md`, 1). `--box` tells
  serve to read its standard input to its end before it starts anything,
  and it gives what it read to jig's sign-in component, the first 32 bytes
  as the printed link's `link_secret` and the next 32 as the local link's
  `local_secret` (jig's `protocol/client/sign-in.md`, *A secret given*). No argument and no
  environment entry carries it, and no log holds it. Every later start
  has no such pipe, its standard input is the null device, and its
  component draws its own first secret, so a restart never replays a
  spent one. The box gives the secret at serve's first spawn whatever
  becomes of that start. Serve refuses any length but none, 32 or 64 bytes,
  `--box: stdin: expected the first link's secret`. Linux and macOS read it
  alike, as the pipe's own descriptors are made alike: skein's `Spawn` makes
  a pipe at a child's descriptor 0 as at any other, and fills what no pipe
  takes from `/dev/null`.
- **A group's failure** before `started` answers `failed`, and the group
  is dormant again, to be asked again; the box runs on. Once started,
  its long steps are restarted, and given up on, as `box-supervision.md`, 1 says.
- **A first run's steps** (`init.md`, 6): `temper-box serve` alone runs at the
  start; the fake kind's group is dormant until the person tries the
  playground (`playground.md`), and the owned kind's until the person sets up an
  owned Forgejo (`owned.md`, 5). No step is a link: serve's links are the
  person's.

## 3. What it names

- **Steps only:** each step's name, its arguments and whether it is
  once, long or link; and which of them are a kind's dormant group,
  bound to a forge by its number alone at its ask. Its policy is a small
  machine of its own (steps, exits, timers and the pipe's lines in;
  spawns, signals, closes, the pipe's lines and the groups' files and
  records out), hosted on skein-io. With `--front` (`box-front.md`, 1) the host writes
  the link and the lines on its parent's pipes beside the machine, and
  reads its standard input's end.
- **The steps in groups:** `Steps { always, asked }`, `always` the steps
  run in order, and `asked` a list of `Group { kind, forge, before,
  steps, remover, recorded, label }`, each a kind's group, dormant until
  asked for a forge (1). Its steps name the forge by a placeholder,
  `FORGE_ARGUMENT`, which the box replaces with the forge's number when
  it binds the group to it, its steps' names taking the number too;
  `remover` is the once step its removal runs (1); and its `started`
  writes the box's record. `label: Box<[u8]>` is the words the terminal
  names it by (`box-supervision.md`, 1), owned opaque bytes passed from the setup/fake
  constructor into the box's layout and written verbatim but for its
  `FORGE_ARGUMENT`, as a step's name is: `Playground` for the fake kind's,
  `Forge N (Forgejo)` for the owned kind's, N the forge's number. The box never
  infers a label from arguments; it adds no label field to a wire reply
  or stored record. `before` is its place in the order:
  the index in `always` it runs just before (its length puts it last).
  Each step carries its `Stage`, whose word its `step` lines carry, and
  its listeners, the socket addresses the box holds and gives it (1). The
  machine's step tests drive a group from dormant to `started`, and to
  `failed`, a started one to `stopped`, and one to `removed`, by lines
  and exits alone (`testing.md`, 1).
- **The binary hands it the steps.** `temper-box` resolves its three
  directories, runs `temper-box init --box` when the configuration directory is
  empty (1), checks the document as `init.md` says, asks
  `temper-forge-setup` for its kinds' steps, `temper-box serve` and the
  owned kind's group, and `temper-fake` for the fake kind's group (`playground.md`),
  and gives them to `temper-box-machine`, each asked group apart.
- **Welcome peers first:** the fake-backend configuration supplies an
  always-running prefix before the ordinary steps. Every asked group's
  insertion point, the playground's included, follows that prefix.
  An empty prefix leaves the ordinary order unchanged. Stopping or
  removing an asked group never stops these peers.
- **Checked:** a boundary test fails the gate when `temper-box-machine` depends
  on a crate named `temper-*` but `temper-forge-setup`, or names an item
  of it but `steps`, `Steps`, `Group`, `Stage`, `Step` and its `Kind`,
  `FORGE_ARGUMENT`, and of its kinds and group's files `BoxKind`,
  `group_path`, `group_file` and `parse_group_file`.

## 4. Other arrangements of the same commands

None has the box's pipe: `temper-box serve` runs without `--box`, and runs
no owned forge. An owned forge's group is the box's: `SetUp` of an
owned forge is refused *not under the box* (`owned.md`, 5). GitHub and a Forgejo
elsewhere connect as under the box.

`temper-box forge fetch`, `prepare` and `run` are commands of their own
too, and a forge the box did not run has no group's file (1). Its setup
file, in `<data>/forge-N`, is written by the first `forge prepare N`
from the flags a person gives (`place.md`, 4), so that a host's own
units run them with no file written by hand. A forge with no setup file
refuses every other command, naming the file and `forge prepare`'s
flags. `temper-box forge remove` reads the setup file alone, and
refuses a forge with none: it removes a directory by its kind, which it
does not guess.

The welcome is a setting (../configuration/settings-kinds.md, 1, kind 20),
shown by default under the box: `--box`, which only the box gives
`temper-box internal serve`, tells its default as it tells serve to
take its pipe. A unit running `temper box` shows the welcome as any box
does; one running `temper-box serve` shows it only once an owner sets
it.

- **systemd,** a unit for `temper box`, whose own stop stops its tree
  (`box-supervision.md`, 1), or one for `temper-box serve` alone; `temper-box serve`'s
  unit keeps systemd's default kill of its control group, which ends a
  crashed service's whole tree before its restart (skein's `runtime/shell.md`,
  12);
- **A host's own units,** `temper help server`'s walk: a unit running
  `temper-box forge fetch`, `prepare` and `run`, beside one for `temper-box
  serve`; a deployment's engine host runs the same units on
  `temper-engine`, its document an engine alone's (../binaries.md, 2). No owned forge's record is made there, so nothing takes the
  handover (`owned.md`, 5), and a start on a new store lists no forge. A person
  connects that Forgejo as its admin (`reconcile.md`, 5), the handover file
  `admin_token` the token, the connect's `--api` and `--git` the
  setup file's `listen` and `remote`, and `--web` its `public`
  (`../openid-provider.md`, 7). It is a managed forge from then on: `TakeOver`
  never runs, so the file stays in the forge's directory for the same
  connect on a store started afresh, where the forge is numbered 1
  again;
- **the process tier** (`testing.md`, section 4), which runs the commands itself,
  with no group's file, as a host's own units do;
- **a container image** whose entry point is `temper box`, every process
  of its tree ending with the container.

## 7. Open questions

- **A box killed outright on macOS** (`box.md`, 1): its steps have no parent-death
  signal, so they can keep running and hold the next start's ports. A
  lifetime guard on macOS needs a mechanism of its own; until one is
  designed, the box's ordered stop, including a hangup, is its cleanup.

- **A launchd agent for the box** on macOS, as `box.md`, 4 arranges systemd
  (`../../draft/macos.md`, section 7).
