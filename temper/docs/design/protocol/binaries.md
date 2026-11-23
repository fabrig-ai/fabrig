# The processes and the binaries

## 6. The box's command table

Moved to [`binaries-box.md`](binaries-box.md), section 1.

## 7. The client's binary

Moved to [`binaries-client.md`](binaries-client.md), section 1.

## 8. The box's commands

Moved to [`binaries-box.md`](binaries-box.md), section 2.

## 9. Standalone processes

Moved to [`binaries-box.md`](binaries-box.md), section 3.

## 1. The deployment's binaries

A deployment's processes each run a binary of their own
(`../draft/deployment-binaries.md`, section 4): the engine's,
`temper-engine` (2), the agent's, `temper-agent` (3), the worker's,
`temper-worker` (5), and the client's, each written here by its own
milestone
(`../draft/deployment-binaries.execution.md`). The box is `temper-box`,
which links every one of them and keeps every command above. What holds
of each:

- **Its own table,** commands as above, from which its reach, its
  `--help`, its `COMMAND --help` and its usage lines derive, rendered
  under its own name: `skein_shell::Table` for `temper-box`,
  `temper-engine`, `temper-worker`, `temper-agent` and the client's
  `temper` (`../cli/README.md`). A command two binaries
  share, the same words run by the same code, is one constant both
  tables hold. `help`, the pages, is every binary's.
- **`--version`** prints `NAME VERSION (COMMIT)` before any command is
  read (`../release.md`, section 3).
- **A binary refuses a document that isn't its own,** before any other
  check, by the document's nearest shape (`cluster.md`, 1), naming the binary that
  runs it; the standalone shape is `temper-box serve`'s.
- **What it links is held by a test,** one table test with a row per
  binary's package, which reads the package's closure as `cargo tree`
  gives it and fails naming each crate its row excludes, and each named
  exception it no longer finds, so that the list only shrinks (4). An
  exception is a cut that costs more than it gives.
- **Code two binaries share is a crate each links,** never one binary's
  package another links: `temper-shell` (the commands' table and its
  renderings, the directories, `help`, the engine's link file) for every
  binary; `temper-service-shell` (serve's run and its host, the files a
  document names, `jig_directories`) for every binary that serves, each
  binary's shell giving it the service it drives and that service's
  lines (`services.md`, 7); `temper-engine-shell` for the engine's
  commands and an engine's start, and `temper-worker-shell` for the
  worker's half of a start and `program-login`, each of which the box
  links too, composing the standalone start from both.
- **A binary links its package's dependencies,** and no Cargo feature
  chooses what (`../draft/cli.md`, 3.2): the engine's service, the
  worker's and the one process composing them are crates of their own
  (`services.md`), `temper-engine` linking the engine's,
  `temper-worker` the worker's and `temper-box` the composition, so a
  binary built alone (`cargo build -p PACKAGE --bin NAME`, the release
  job's) links what it links built with the workspace.

## 2. `temper-engine`

The engine's host's binary, `bin/temper-engine`, its code
`crates/temper-engine-shell`, ordinary Rust: the engine and its web
client, with the owned forge's commands. Its table and sets are
`../cli/README.md`'s (7):

```
documentation  temper-engine help [NAME | pages | --search WORDS...]
deployment     temper-engine serve [DIRECTORIES] [--record]
               temper-engine versions
               temper-engine upgrade --check DIRECTORY | --workers DIRECTORY | --back
               temper-engine checkpoint [DIRECTORIES] [--hold NAME -- COMMAND ARGS... | --remove NAME]
               temper-engine restore NAME [DIRECTORIES]
               temper-engine forge fetch|prepare|sign-in|source|remove [DIRECTORIES] FORGE
               temper-engine forge run [DIRECTORIES] FORGE forgejo|runner
spawned        temper-engine internal forge-child --parent PID --signal kill|terminate -- PROGRAM ARGS...
               temper-engine --version
```

- **Each command is the box's of the same words,** run by the same
  code, `temper-engine-shell`'s, with the same exits, but for what
  follows. Its `help` and its `forge` and `internal forge-child` rows
  are the constants the box's table holds too; its `serve` row is its
  own, saying it serves an engine alone's document.
- **Its lines name it:** serve's and the forge's lines, refusals and
  hints say `temper-engine serve: …`, `temper-engine forge VERB: …` and
  `run temper-engine forge prepare …` where the box's say `temper-box`.
- **An engine alone's document only.** `serve` and each `forge` command
  that reads the document (all but `remove`) read its nearest shape
  first (`cluster.md`, 1) and, for any other, print one line and exit 1, before any
  other check, `COMMAND` the command's words:
  - a worker alone's:
    `temper-engine COMMAND: temper.json holds a worker alone; temper-worker serve runs it`;
  - standalone's:
    `temper-engine COMMAND: temper.json holds an engine and its worker in one process; temper-box serve runs it`.

  A document whose shape can't be read, not a JSON object, goes on to
  the checks `temper-box serve` makes, and is refused in their words.
- **`versions`** prints, one line each, in this order, and exits 0,
  reading no configuration and no store:
  - `level OWNER LEVEL` for `skein`, `smith`, `jig` and `temper`, the
    binary's `OwnerSet.level` of each (jig's `protocol/store/opening-owners.md`, 1),
    from the sets it hands its store (`floor.md`, 3): skein 0, smith 0,
    jig 6 and temper 14;
  - `store VERSION`, jig's store's component version, then `family
    NAME VERSION` for jig's families, `core`, `tasks`, `people`,
    `notes`, `accounts` and `settings`, and `temper`'s, `STORE_VERSION`
    (`README.md`, section 7; jig's `format::current`);
  - for each wire of `temper_engine_service::wires()`, in its order
    (`wires.md`, section 10): `wire NAME LOWEST HIGHEST`, LOWEST its
    floor, then `layout NAME VERSION FINGERPRINT` for each version from
    LOWEST to HIGHEST, FINGERPRINT its layout's 64 lowercase hex digits;
  - `bound stop SECONDS`, the settled stop's bound at the stock limits
    (`../domain/root-restart.md`, 1.2), `bound stop 602`, then `bound report
    SECONDS`, a wait's report bound (`service-process.md`, 1.3), `bound report
    30`, each in whole seconds, rounded up
    (`configuration/derived.md`, 5.6). A deployment whose limits it
    changed has its own stop bound, which `versions`, reading no
    settings, does not print. `--version` stays its one line, which
    tools read whole.
- **`upgrade`**, **`checkpoint`** and **`restore`** are `binaries.md`'s
  `temper-box upgrade`, `temper-box checkpoint` and `temper-box
  restore`, on this binary.
- **No command is an unknown command:** `temper-engine: no command WORDS;
  temper-engine --help lists them`, the words given as far as a command
  begins with them, and exit 2 (`../cli/README.md`, 4).
- **The bundle is in it,** as in the box's (`../client/web/bundle.md`,
  1): of a deployment's binaries, only the engine's serves the web
  client.
- **What it links** (1): the engine's service, `temper-engine-service`
  (`services.md`, 4); its row in 4 excludes the composition,
  `temper-service`, the
  worker's domain and service, `temper-worker-`, `jig-protocol-worker-agents`,
  `jig-protocol-worker-link` and smith's host half, `smith-host-`, and
  smith's agent: `smith-agent-`, `smith-local-`, `smith-inline-agent`,
  `smith-domain`, `smith-domain-`, `smith-protocol-channel`,
  `smith-protocol-events`, `smith-protocol-llm`,
  `smith-protocol-machine`, `smith-protocol-program` and
  `smith-events`. Its exceptions:
  - `smith-domain`, `smith-domain-run`, `smith-domain-session` and
    `smith-domain-tools`: smith's agent domain, sans-io, whose
    vocabulary jig's charter (`jig-protocol-charter`) and the engine's
    translation (`temper-engine-smith`) speak, the charter's run,
    `Start`, `Event` and `Transcript`, with no io, process or LLM of its
    own. Cutting them would move that vocabulary into a smith crate of
    its own, a change of smith's that gives the engine no io or process
    less.

  smith's codecs and shared translation, `smith-charter`,
  `smith-channel`, `smith-transcript` and `smith-protocol-translate`,
  and the channel's shared wires, `jig-protocol-worker-wire` and
  `temper-protocol-worker-wire`, are the engine's as much as the
  worker's, and are not excluded.
- **Its package** holds its `main` alone; its table, its dispatch and
  its commands' code are `temper-engine-shell`'s and `temper-shell`'s,
  `--version`'s `temper-version`'s (`components.md`). Its step tests, one
  test binary in its package, run the built binary.

## 3. The agent's binary, `temper-agent`

A deployment runs smith's agent from a binary of its own, `temper-agent`
(`bin/temper-agent`, its crate `crates/temper-agent-shell`;
`../draft/deployment-binaries.md`, sections 4
and 9, item 3), ordinary Rust, one process per run, which its worker starts
and a person never does. It holds every mode in which `temper-box` runs
smith's agent, run by the same code, and nothing else of `temper-box`'s.
`temper-box` keeps those modes, as `temper-box internal agent` and by the same
flags, until a worker names `temper-agent` as its agent's program
(setup/init.md, 2, as `../draft/deployment-binaries.execution.md`'s M3
makes it), so the box and its tests don't change. Its table is
skein's, `AGENT` (`../cli/README.md`, 7.3):

```
public    temper-agent help [NAME | pages | --search WORDS...]
internal  temper-agent AGENT.json
          temper-agent --list AGENT.json
          temper-agent --mcp-relay 127.0.0.1:PORT
          temper-agent --program-child PARENT PROGRAM ARGUMENTS...
          run as claude, with Claude Code's arguments
```

- **Its table** holds `help`, `temper_shell::table::HELP`, public, and
  five commands of its own, each internal, their notes empty; the four
  reached by their words read:

  | Command | Reached by | `forms` | `line` | `arguments` |
  |---|---|---|---|---|
  | the run | the bare form | `AGENT.json` | Run one agent for a worker's run, on smith's agent configuration. | `AGENT.json`: The agent's configuration, as the worker's agent.configuration names it. |
  | the list | `--list` | `AGENT.json` | Read one endpoint's model list, on smith's agent configuration. | `AGENT.json`, as the run's |
  | the relay | `--mcp-relay` | `127.0.0.1:PORT` | Relay a program's MCP connection to its agent. | `PORT`: The agent's relay listener on loopback; the secret is in SMITH_MCP_SECRET. |
  | the program child | `--program-child` | `PARENT PROGRAM ARGUMENTS...` | Run a program as the agent's child, which ends with its parent. | `PARENT`: The agent's process id. `PROGRAM`: The program's path, with its `ARGUMENTS`. |

  The run's and the list's lines and argument are `temper-box`'s
  `internal agent` and `internal agent-list` rows' words. The relay,
  the program child and the fifth, run as `claude`, are public
  constants of `temper-agent-shell`, `RELAY`, `PROGRAM_CHILD` and
  `CLAUDE`, which the box's table holds as its rows of those words
  (`../cli/README.md`, 6), so the box's lines for them are these.
- **The run and the list are the box's too:** `temper-agent-shell`'s
  `run` and `list`, public, each given the words its lines start with,
  `temper-agent` and `temper-agent --list` here, the box's `internal
  agent` and `internal agent-list` words there, so that both binaries
  run one copy of each (`../cli/README.md`, 7.3).
- **Its arguments,** captured once `main` has installed skein's
  fail-stop hook, as every binary's `main` does first, reach a command
  by `Table::reach` (skein's `runtime/shell-table.md`, 1.4;
  `../cli/README.md`, 3):
  1. run by a name whose file name is `claude`: Claude Code's launcher,
     `temper-launcher`'s, its arguments unchanged (setup/owned.md, 6),
     given the captured environment;
  2. `-h`, `--help` or `help` alone: the table's page, exit 0;
     `--version` alone: `temper-agent VERSION (COMMIT)`, exit 0,
     VERSION and COMMIT `temper-version`'s, as every binary's (1;
     `../release.md`, section 3);
  3. by its words: `help` and its words (`documentation.md`, 4),
     `--list AGENT.json`, `--mcp-relay` and `--program-child`, whose
     code is smith's `relay::main` and `program_child::main`, reading
     every argument themselves (smith's `protocol/programs.md`, 5.1 and
     5.3); a command's words then `-h` or `--help` alone print its page,
     which says it's internal;
  4. else the bare form, its one argument AGENT.json. A document named
     as a command's word is given by a path holding a slash, `./help`;
     a worker gives an absolute path.
- **One run:** AGENT.json read as `temper-box` reads an agent's document,
  its bytes with a `"fetched"` program's path replaced by its launcher
  (setup/owned.md, 6; `temper-launcher`'s), then smith's decode; then
  smith's agent shell, `run_dumping` into `TEMPER_INTERNAL_DUMP_LLM`'s directory
  when the captured environment's variable names one (`../draft/dogfood-hacks.md`, 2.13),
  `run_decoded` otherwise, what `temper-box internal agent` does. A
  document that doesn't read prints `smith: ` and the reason, then each
  line of the reason after `temper-agent: `; a run that fails prints
  each line of its reason after `temper-agent: `. A `TEMPER_INTERNAL_DUMP_LLM`
  that can't be made absolute prints `temper-agent: TEMPER_INTERNAL_DUMP_LLM: `
  and why, and the run goes on without its dumps.
- **The list mode:** the same read, then smith's `list::run_decoded`,
  what `temper-box internal agent-list` does; a failure prints each line of
  its reason after `temper-agent --list: `. smith's host half puts
  `--list` before its list launch's arguments (smith's
  `protocol/agent.md`, 6.1), so a worker that names `temper-agent` with
  the arguments `AGENT.json` for its runs launches `temper-agent --list
  AGENT.json` for a list, with no word of temper's.
- **Refusals,** exit 2, stdout empty, in skein's words for the
  binary's name (`../cli/README.md`, 4): no argument, `temper-agent: no
  command; temper-agent --help lists them`; a first argument beginning
  with `-` that no command's words begin, `--version` or `--help` with
  more arguments among them, `temper-agent: no command ARGUMENT;
  temper-agent --help lists them`; a command's form with other
  arguments, `temper-agent: invalid arguments` or `temper-agent --list:
  invalid arguments`, then its usage lines.
- **Its exit status:** smith's agent shell's code, a signal's 1
  (`binaries.md`); the launcher's, the build's once it has replaced itself,
  or 1 with `temper-agent: claude: ` and the reason when it can't; the
  relay's and the program child's, smith's.
- **It links** its crate, `temper-agent-shell`, `temper-shell`, smith's agent shell and what that links,
  `temper-launcher` for the launcher and the substitution, with the
  fetch's host and machine beneath it, `temper-fetch-shell` and
  `temper-fetch`, `temper-version`, and skein's crates: no other temper
  crate and no jig crate (`../draft/deployment-binaries.md`, section
  4). `temper-shell` brings `temper-docs`, the pages its `help` row
  answers from, and no jig crate: jig's form of the three directories
  is its callers' (`components.md`). 4's test holds it there.
- **Its file** is the Cargo target `temper-agent`, built alone as the
  release job builds it (`cargo build -p temper-agent --bin
  temper-agent`), with no Cargo feature of its own: the package at
  `bin/temper-agent`, its `main.rs` installing fail-stop and calling
  `temper-agent-shell`'s `main` (`../cli/README.md`, 2).
- **Its tests:** its crate's parser's table, a row per form and
  per refusal, as `temper`'s; and one table in its package's suite, a row
  per mode, each running the built `temper-agent` with an empty
  environment and stdin closed and comparing its exit, stdout and
  stderr: `--version`, `--help` and `--list --help`; the refusals; the
  relay's usage alone, and its row's page before `--help`; the program child's refusal
  and both statuses, as `bin/temper-box/tests/suite/agent_modes.rs` has
  them; a run and a list refused for a missing document; a run and a
  list started on a loopback plaintext document, ending as `temper-box
  internal agent` and `temper-box internal agent-list` end with stdin
  closed; and Claude Code's launcher by a link named `claude`, reaching
  a build placed in the cache with its arguments unchanged,
  `DISABLE_AUTOUPDATER=1` and `DISABLE_UPDATES=1`, and refusing a build
  of another size.

## 4. Each binary's closure

The dependency rule (`../draft/deployment-binaries.md`, sections 4 and
10) is one table test, `tests/boundary/tests/binaries.rs`, in the
repository's boundary tests: a row per deployment binary's package,
saying, tree by tree, which of the workspace's crates its closure may
hold, and its exceptions, each with its reason.

- **A row's closure** is `cargo tree --locked --offline -p PACKAGE -e
  normal --prefix none --format {p}` at the workspace's root: the
  package's normal dependencies, with that package's features alone. A
  crate's tree is the first directory of its manifest's path under the
  root (`skein`, `smith`, `jig`, `temper`); a crate from outside the
  workspace has none, and no row excludes it.
- **A row's rules,** one for each tree it names, each of two kinds:
  `only` names the crates of that tree the closure may hold, `not` the
  crates of it the closure may not. A crate is named by its name, or by
  a prefix ending in `-`. A tree the row doesn't name may be held whole.
- **An exception** names one crate, by its name, that a rule excludes
  and that a cut would cost more than it gives, with its reason.
- **The test fails,** naming the package and each finding, when:
  - the closure reads empty, or lacks the package itself;
  - the closure holds a crate a rule excludes that is no exception: the
    finding names the crate and its path, as `cargo tree -p PACKAGE -e
    normal -i CRATE` gives it;
  - an exception names a crate the closure no longer holds, or one no
    rule excludes: it is dropped, so that the list only shrinks.
- **The rules' matching** is held without cargo by a table test beside
  it, over closures the test writes: a name and a prefix, `only` and
  `not`, a tree no rule names, an exception kept, one gone and one no
  rule excludes, an empty closure and one lacking its package.

| Package | Rules | Exceptions |
|---|---|---|
| `temper` | temper only `temper`, `temper-client-shell`, `temper-client-domain`, `temper-client-protocol-wire`, `temper-client-shell-cli`, `temper-client-shell-terminal`, `temper-client-shell-tui`, `temper-client-redaction`, `temper-client-tools`, `temper-charter-set`, `temper-client-tui`, `temper-client-view`, `temper-shell`, `temper-docs`, `temper-version`, `temper-fetch` and `temper-fetch-shell`; jig only `jig-client`, `jig-client-domain`, `jig-client-protocol-http`, `jig-client-protocol-link`, `jig-client-protocol-wire`, `jig-client-shell-native` and `jig-client-view`; smith only none | `temper-settings-form`, `temper-protocol-records`, `temper-protocol-reaches`, `jig-protocol-transcript`, `smith-transcript` |
| `temper-engine` | temper not `temper-worker-` and `temper-service`; jig not `jig-protocol-worker-agents` and `jig-protocol-worker-link`; smith not `smith-host-`, `smith-agent-`, `smith-local-`, `smith-inline-agent`, `smith-domain`, `smith-domain-`, `smith-protocol-channel`, `smith-protocol-events`, `smith-protocol-llm`, `smith-protocol-machine`, `smith-protocol-program` and `smith-events` | `smith-domain`, `smith-domain-run`, `smith-domain-session`, `smith-domain-tools` (2) |
| `temper-worker` | temper not `temper-engine-`, `temper-box-`, `temper-fake`, `temper-fake-`, `temper-init`, `temper-client-bundle`, `temper-client-shell-web`, `temper-forge-setup`, `temper-forge-setup-shell`, `temper-protocol-records-forge`, `temper-agent-shell` and `temper-service`; smith not `smith-agent-shell`, `smith-agent-service`, `smith-local-` and `smith-inline-agent` | none (5) |
| `temper-agent` | temper only `temper-agent`, `temper-agent-shell`, `temper-shell`, `temper-docs`, `temper-launcher`, `temper-fetch`, `temper-fetch-shell` and `temper-version`; jig only none | none |

- **The client assembles charter overlays** (`../client/ctl/overlays.md`, 2)
  through `temper-charter-set`, the sans-io file assembly shared with
  startup. It depends on skein-lib alone, and carries no engine or
  worker code into the client.
- **The client's exceptions:** `temper-settings-form`, the settings
  form and its words, `no_std` and neither the client's nor the
  engine's, which ctl's tools (`temper-client-tools`) render and the
  engine decodes; `temper-protocol-records`, the forges' pasted bundle's
  codec, which the client encodes and the engine decodes, without the
  records' reading at start, `temper-protocol-records-forge`
  (`../client/README.status.md`, 2);
  `temper-protocol-reaches`, what reaches a person, `no_std`, whose
  subjects the client's forge pages and the engine share; and
  `jig-protocol-transcript` with `smith-transcript` beneath it, jig's
  bounded translation of smith's turns into its client's vocabulary,
  which `jig-client-protocol-link` reads, a cut of jig's. Each is
  sans-io vocabulary two sides speak; a crate of its own for either
  side would remove no io or process from the client.
- **The client links no `jig-protocol-accounts`:** its CLI shell's
  manifest names none, since no code of the client uses it, and with it
  go jig's engine protocol, its store and its domain crates.
- **The services' crates** (`services.md`): the engine's row keeps out
  the worker's service and the composition, `temper-service`; the
  worker's keeps out the engine's service, the forge's setup and its
  records' reading at start, and the composition, with no exception.

smith's crates and skein's are the agent's to hold: smith's agent and
what it rests on. The box links every crate the four hold but their
packages, and has no row: it is the whole product.

## 5. The worker's binary, `temper-worker`

Each worker's host's binary, `bin/temper-worker`: a worker alone, its
programs' sign-in and the pages:

```
public  temper-worker help [NAME | pages | --search WORDS...]
        temper-worker serve [DIRECTORIES]
        temper-worker program-login [DIRECTORIES] NAME
        temper-worker --version
```

- **Each command is the box's of the same words,** run by the same
  code, `temper-worker-shell`'s and `temper-service-shell`'s, with the
  same exits, but for what follows. Its table is skein's, `WORKER`
  (`../cli/README.md`, 7.2): its `help` and `program-login` commands
  are the constants the box's table holds too,
  `temper_shell::table::HELP` and `temper-worker-shell`'s
  `PROGRAM_LOGIN`; its `serve` is its own, saying it serves a worker
  alone's document.
- **Its lines name it:** serve's refusals and endings say
  `temper-worker serve: …` and program-login's say
  `temper-worker program-login: …`, where the box's say `temper-box`. The
  lines a worker alone prints (`cluster.md`, 3) are the same.
- **A worker alone's document only.** `serve` and `program-login` read
  its nearest shape first (`cluster.md`, 1) and, for any other, print one line and
  exit 1, before any other check, `COMMAND` the command's words:
  - an engine alone's:
    `temper-worker COMMAND: temper.json holds an engine alone; temper-engine serve runs it`;
  - standalone's:
    `temper-worker COMMAND: temper.json holds an engine and its worker in one process; temper-box serve runs it`.

  A document whose shape can't be read, not a JSON object, goes on to
  the checks a worker alone's start makes, and is refused in their
  words.
- **No command is an unknown command:** `temper-worker: no command
  WORDS; temper-worker --help lists them`, and exit 2, as 2 says it; a
  command's arguments refused exit 2 too (`../cli/README.md`, 4).
- **Its `serve`** is the box's `temper-box serve` on a worker alone: the
  worker's part read and checked, its agent's document read by
  `smith-agent-document`, its root, program sources and the launcher's
  links made (setup/owned.md, 6), its agent's program and document
  pinned in its root (`../domain/worker/agent-processes.md`, section 1), its link
  dialled, and its loop, ending and
  exit its worker's (`cluster.md`, 1). It takes no `--box`, and has no bundle, no
  fakes and no store.
- **Its `program-login`** is the box's on a worker alone (`cluster.md`, 1;
  credentials-llm-accounts.md, section 1): it reads the worker's part alone, makes
  the source's directory and replaces itself by the program's sign-in.
- **What it links** (1): `temper-worker-shell`,
  `temper-service-shell`, `temper-shell` and `temper-version`, and the
  worker's service, `temper-worker-service`, on `temper-service-core`
  (`services.md`, 5), so its closure holds none of the engine's: no
  engine's domain or service, no `temper-engine-smith`, no forge's setup
  or its records' reading at start, no `temper-box-machine`, and no
  composition, `temper-service`; its row in 4 has no exception. It links
  no `temper-engine-shell`, `temper-client-bundle`, `temper-fake`,
  `temper-forge-setup-shell`, smith's agent shell or smith's agent
  service.
- **Its package,** `bin/temper-worker`, has skein's layout
  (`skein/docs/foundation/cli.md`, section 2): a `main.rs` that installs
  fail-stop and calls `temper-worker-shell`'s `main`, which holds the
  table, its capture, its reach and the dispatch (`../cli/README.md`,
  3); `--version`'s line is the table's, from `temper-version`. Its
  step tests, one test binary of the package, run the built binary
  (`../cli/README.md`, 10).
