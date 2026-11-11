# temper's command lines

How temper's binaries meet their command lines, on the repository's
convention (`skein/docs/foundation/cli.md`) and skein's table
(`skein/docs/design/runtime/shell.md`, 14): each binary's package and
crate, its table, how its arguments reach a command, its help, its
exits, the environment it reads and passes down, and the reference
documents generated from its table. The sets are the draft's
(`../draft/cli.md`, section 4); this document gives `temper-box`'s,
`temper-engine`'s, `temper-worker`'s and `temper-agent`'s. `temper`, the
client, takes its in its own milestone (`../draft/cli.execution.md`,
C3).

## 1. In one page

- **A binary is a package** at `temper/bin/<binary>/`, whose `main.rs`
  installs fail-stop and calls its crate's `main`, handing it
  `temper-version`'s build identity, which no other package links (2).
  The box's crate is `temper-box-shell`, the engine's
  `temper-engine-shell`, the worker's `temper-worker-shell`, the agent's
  `temper-agent-shell` and the client's `temper-client-shell`.
- **Its crate holds its table,** a `skein_shell::Table`, and captures
  the process's arguments and environment once (3). Every other reader
  takes what the capture holds.
- **`Table::reach`** answers what the arguments reach
  (`skein/docs/design/runtime/shell.md`, 14.4): the page, the version,
  a command's page, a command, or no command (3).
- **Exits:** 0 done, 1 failed, 2 refused its arguments (4).
- **Help:** `-h`, `--help` and `help` print the table's page;
  `help pages`, `help NAME`, `help --search` and `help commands` are the
  pages, and `commands` is the binary's own table (5).
- **The tables** of `temper-box` (6) and of the deployment's three,
  `temper-engine`, `temper-worker` and `temper-agent` (7), and the
  variables each declares (8).
- **Generated:** each binary's `<binary>-args.md` and `<binary>-env.md`,
  and the tree's common environment document, beside this page, checked
  byte for byte (9).
- **The test set:** `temper-test-env`'s table of the `TEMPER_TEST_*`
  variables, through which temper's tests read the environment, and its
  generated `test-env` (11).

## 2. The binaries and their crates

| Binary | Package | Its crate | Its table |
|---|---|---|---|
| `temper-box` | `temper/bin/temper-box` | `temper-box-shell` (`temper/crates/temper-box-shell`) | `temper_box_shell::TABLE` |
| `temper-engine` | `temper/bin/temper-engine` | `temper-engine-shell` | `temper_engine_shell::ENGINE` |
| `temper-worker` | `temper/bin/temper-worker` | `temper-worker-shell` | `temper_worker_shell::WORKER` |
| `temper-agent` | `temper/bin/temper-agent` | `temper-agent-shell` (`temper/crates/temper-agent-shell`) | `temper_agent_shell::AGENT` |
| `temper-client` | `temper/bin/temper-client` | `temper-client-shell` (`temper/crates/temper-client-shell`) | `temper-shell`'s rows, `temper_client_shell::COMMANDS`, until `../draft/cli.execution.md`'s C3 |

- **`main.rs`** is the package's whole source, and hands its crate the
  build identity (`../release.md`, 3):

  ```rust
  //! The box's binary (temper/docs/design/cli/README.md, 2).
  fn main() -> std::process::ExitCode {
      skein_fail_stop::install();
      temper_box_shell::main(temper_version::VERSION, temper_version::COMMIT)
  }
  ```

  The client's hands its crate the line itself:
  `temper_client_shell::main(&temper_version::line(temper_client_shell::NAME))`.
  Its `[dependencies]` are its crate, `skein-fail-stop` and
  `temper-version`, which decide what it links
  (`skein/docs/foundation/cli.md`, 2). Only these packages depend on
  `temper-version`, so a new commit relinks the binaries and rebuilds no
  crate they link.
- **The libraries that held the names are renamed:** the whole
  product's library, `temper` at `temper/crates/temper`, is
  `temper-box-shell` at `temper/crates/temper-box-shell`, holding the
  box's table and everything it runs; the box's machine, `temper-box` at
  `temper/crates/temper-box`, is `temper-box-machine` at
  `temper/crates/temper-box-machine`. The engine's package,
  `temper/crates/temper-engine`, moves to `temper/bin/temper-engine`,
  and its table and dispatch move into `temper-engine-shell`.
- **The agent's package splits:** `temper/crates/temper-agent`, a
  library and a binary in one package, becomes the binary's package
  `temper-agent` at `temper/bin/temper-agent` and its crate
  `temper-agent-shell` at `temper/crates/temper-agent-shell`, which holds
  the library's code, its table and everything the binary runs. No other
  crate names the old library. The worker's package is already at
  `temper/bin/temper-worker`, its crate `temper-worker-shell`.
- **The client's package splits the same way:**
  `temper/crates/temper-client`, a library and a binary in one package,
  becomes the binary's package `temper-client` at
  `temper/bin/temper-client` and its crate `temper-client-shell` at
  `temper/crates/temper-client-shell`, which holds the library's code
  and the capture the old `main.rs` made (3). Its suite moves to the
  binary's package, as the agent's did. The binary's file and target
  keep the name `temper-client` until C3 names it `temper`.
- **Tests that run a binary live in its package,** under its `tests/`,
  since Cargo gives `CARGO_BIN_EXE_<name>` to its own package's tests
  alone: the whole product's integration tests, with their `[[test]]`
  targets, fixtures and support, move from `temper/crates/temper/tests/`
  to `temper/bin/temper-box/tests/`, the engine's from
  `temper/crates/temper-engine/tests/` to `temper/bin/temper-engine/tests/`,
  and the agent's suite from `temper/crates/temper-agent/tests/` to
  `temper/bin/temper-agent/tests/`.
  They are dev-dependencies' users, which link nothing into the binary.
  Unit tests stay in their crate.
- **A crate's features stay its own** until `../draft/cli.execution.md`'s
  C5: `temper-box-shell` keeps the whole product's `record`, and the
  checks' feature exceptions that named `temper/crates/temper/Cargo.toml`
  name `temper/crates/temper-box-shell/Cargo.toml`, the same exceptions
  under the path they moved to.

## 3. Its `main`: the capture and the reach

Each binary's crate has `pub fn main(version: &'static str, commit:
&'static str) -> std::process::ExitCode`, given `temper-version`'s
`VERSION` and `COMMIT` by its binary's package (2):

1. **The capture,** one function, the only place the binary reads the
   process: argv's first word, the arguments after it, the environment
   (`skein_shell::arguments` and `skein_shell::environment`, with
   `std::env::args_os` for argv's first word) and the working directory,
   under one `#[expect(clippy::disallowed_methods, reason = "the binary's
   capture: ...")]` (`skein/docs/foundation/cli.md`, 9, check 4).
2. **The reach,** `TABLE.reach(run_as, &arguments)`
   (`skein/docs/design/runtime/shell.md`, 14.4):
   - `Help`: `TABLE.help()` on stdout, exit 0;
   - `Version`: the version line of the table with `main`'s identity,
     `Table { version, commit, ..TABLE }.version_line()`, on stdout,
     exit 0;
   - `CommandHelp(command)`: `TABLE.command_help(command)` on stdout,
     exit 0;
   - `Unknown(line)`: the line on stderr, exit 2;
   - `Command(command, rest)`: the command's code, matched on its
     `reach`, given `rest` and what the capture holds.
3. **Passing it down:** a function that reads a variable takes the
   captured environment, `&[(Box<[u8]>, Box<[u8]>)]` as
   `skein_shell::environment` returns it, or the value looked up from
   it, and looks a variable up by its declared `Variable`
   (`Variable::value`), never by a string. A function that hands a child
   the environment hands it what the capture holds. A helper of another
   tree's that reads the process itself, smith's relay and program child,
   stays as it is until its tree's round.

**The build identity is `main`'s alone.** A crate's table names none:
its `version` and `commit` are empty (`""`), and the crate links no
`temper-version`. Every place the crate prints its version line, a
pure `configure` that answers `--version` among them, takes the
identity `main` was handed, so `--version` prints `temper-box VERSION
(COMMIT)` as `../release.md`, section 3, words it. The client's crate
has `pub fn main(line: &str) -> std::process::ExitCode` instead,
given `temper_version::line(NAME)` whole, and prints it, a newline
after it, for `--version`.

## 4. Exits and refusals

- **0** done; **1** failed; **2** refused its arguments
  (`skein/docs/foundation/cli.md`, 5). A delegated grammar keeps its
  program's: fake git's, the Claude Code launcher's, smith's relay's and
  program child's.
- **No command:** `Reached::Unknown`'s line, exit 2. A bare form that
  takes no positional argument answers a word that doesn't begin with
  `-` with `TABLE.unknown(arguments)`, exit 2:
  `temper-box frobnicate` says `temper-box: no command frobnicate;
  temper-box --help lists them`.
- **A bare form that takes one positional argument,** the agent's
  `AGENT.json` (7.3), answers no argument, or a first argument that
  begins with `-`, with `TABLE.unknown(arguments)`, exit 2, and more
  arguments than its form takes as a refusal of its arguments (below):
  `temper-agent` alone says `temper-agent: no command; temper-agent
  --help lists them`, and `temper-agent --frob` says `temper-agent: no
  command --frob; temper-agent --help lists them`.
- **A command's arguments refused** (a missing or extra argument, an
  unknown flag, a flag's bad value, a path past its bound): its first
  line `BINARY WORDS: WHY`, then `usage: BINARY WORDS FORM` for each of
  its forms, as `temper_shell::table::refused` renders them (5), exit 2.
- **A failure** once the arguments are taken (a document refused, a
  directory that already holds `temper.json`, a start that fails): its
  line, exit 1, as today.
- **Another binary's document** is refused before any other check, exit
  1, in `../protocol/binaries.md`, 2's words, the standalone shape's
  hint naming `temper-box serve`.

## 5. Help

- **`-h`, `--help` and `help`,** alone, print `TABLE.help()`
  (`skein/docs/design/runtime/shell.md`, 14.2).
- **`COMMAND -h` or `COMMAND --help`** print `TABLE.command_help`; an
  internal command's page says it's internal.
- **`help` with words** (`../protocol/documentation.md`, 4):
  - `help pages`: the pages' index, the page `index`;
  - `help commands`: the binary's own table, as
    `temper_shell::table::commands_page(&TABLE)` renders it;
  - `help NAME`: the page NAME; `help --search WORDS...`: the search.
- **`--version`:** `TABLE.version_line()`; no `version` command.

`temper-shell`'s `table` module holds what every temper binary on
skein's table shares, beside the `commands` module the client keeps
until C3:

```rust
/// `help`'s row: public, its form `[NAME | pages | --search WORDS...]`.
pub const HELP: skein_shell::Command;
/// `help`'s words after `help`: `pages`, `commands`, a NAME or a search,
/// printed for `table`: Ok for stdout, exit 0; Err(line) for stderr,
/// exit 1 for a page or a search that answers nothing, 2 for words
/// `help` doesn't take.
pub fn help(table: &skein_shell::Table, words: &[Box<[u8]>]) -> Result<String, (u8, String)>;
/// A command's arguments refused: `BINARY WORDS: WHY`, then its usage lines.
pub fn refused(table: &skein_shell::Table, command: &skein_shell::Command, why: &str) -> String;
/// The page `commands` for `table`: `../protocol/documentation.md`, 8.
pub fn commands_page(table: &skein_shell::Table) -> String;
/// The compiled page `commands`, from several binaries' tables in the
/// order given: `../protocol/documentation.md`, 8.1.
pub fn commands_reference(tables: &[&skein_shell::Table]) -> String;
```

`temper_shell::help::request` takes `pages` as the index's word, so
`help pages` prints the index in every binary.

## 6. `temper-box`'s table

The whole product, for trying temper and for one person's development
on one machine (`../draft/cli.md`, 4.2). Its line is `temper on this
machine, as one box.`; its last, `temper-box COMMAND --help prints one
command; temper-box help pages lists the pages.`

| Reached by | Set | Forms and what |
|---|---|---|
| the bare form | public | `[--config DIR] [--data DIR] [--cache DIR] [--echo none\|steps]`: run the deployment as one box, its setup's steps and `temper-box serve`, until stopped: `../protocol/setup/box.md` |
| `init` | public | `[DIRECTORIES]`; its switch `--box`, internal |
| `serve` | public | `[DIRECTORIES]`: the deployment in one process |
| `program-login` | public | `[DIRECTORIES] NAME` |
| `help` | public | `temper_shell::table::HELP` (5) |
| `versions` | public | `temper-engine-shell`'s `VERSIONS` (7.1) |
| `upgrade` | public | `../protocol/binaries.md`'s, once it's built |
| `ctl`, `open` | internal | the tests' |
| `forge fetch`, `prepare`, `run`, `remove`, `source`, `sign-in` | internal | `temper-engine-shell`'s (7) |
| `internal agent`, `internal agent-list`, `internal serve`, `internal forge-child`, `internal tui` | internal | spawned |
| `fake llm`, `fake forge`, `fake forge check`, `fake oauth`, `fake git` | internal | the test doubles (`../testing/fakes.md`) |
| run as `git`; run as `claude`; `--mcp-relay`; `--program-child`; `--list` | internal | the pre-table modes, rows that match before the others: fake git, the Claude Code launcher, smith's relay, smith's program child, and smith's list read's `--list internal agent-list AGENT.json` |

- **The bare form is today's `temper box`:** its internal switches
  `--fake-backends` and `--no-tui` and its internal setting `--bundle
  BUNDLE` give today's other forms. In a terminal, with `--echo none`
  and no `--no-tui`, it runs the TUI in front, signed in, by `internal
  tui` (`../protocol/setup/box.md`, 6). C3 moves that front into the
  client's `temper box` and removes `--no-tui` and `internal tui`
  (`../draft/cli.md`, section 5). `--echo` is a flag of every form, the
  fake backends' among them.
- **No `box` words and no TUI:** `temper box` is the client's
  (`../draft/cli.md`, 4.1 and 5), and the TUI on an engine is
  `temper-client [ADDRESS]`'s until C3 names it `temper`. A caller that
  ran the TUI through the whole product runs `temper-client`.
- **The settings:** the three directories, `temper_shell::directories`'
  group, with their twins `TEMPER_CONFIG`, `TEMPER_DATA` and
  `TEMPER_CACHE`; `--echo`, with no twin; the fakes' `--document`,
  `--ready`, `--seed` and `--observations`, internal, with no twins
  (`../draft/cli.md`, 4.6, *Twins dropped*).
- **Rows two tables share** are one constant both hold: `help`, the
  forge's commands, `internal forge-child` and `versions`,
  `temper-engine-shell`'s (7.1); `program-login`,
  `temper-worker-shell`'s `PROGRAM_LOGIN` (7.2); and the pre-table
  modes `--mcp-relay`, `--program-child` and run as `claude`,
  `temper-agent-shell`'s `RELAY`, `PROGRAM_CHILD` and `CLAUDE` (7.3).
  Each is run by the same code as in its own binary, under the box's
  name: `versions` by `temper_engine_shell::versions`, and `internal
  agent` and `internal agent-list` by `temper-agent-shell`'s `run` and
  `list` (7.3).

## 7. The deployment's tables

The three binaries a deployment at scale runs, each on its own host
(`../draft/cli.md`, 4.3 to 4.5; `../protocol/binaries.md`, 2, 3 and
5). Each is built as 2 to 5 say, as the box's is.

### 7.1 `temper-engine`

The engine's host's binary (`../protocol/binaries.md`, 2; `../draft/cli.md`,
4.3). Its line is `the deployment's engine, with the owned forge's
commands.`

| Reached by | Set | Forms and what |
|---|---|---|
| `serve` | public | `[DIRECTORIES]`: an engine alone's document, until stopped |
| `versions` | public | what the binary speaks and keeps, one line each |
| `help` | public | `temper_shell::table::HELP` (5) |
| `forge fetch`, `prepare`, `run`, `remove`, `source`, `sign-in` | internal | the owned forge's commands |
| `internal forge-child` | internal | spawned |

`upgrade` joins it, public, once it's built. The forge's commands and
`internal forge-child` are `temper-engine-shell`'s constants, which the
box's table holds too:

```rust
// temper_engine_shell::commands
pub const SERVE: skein_shell::Command;      // the engine's own
pub const VERSIONS: skein_shell::Command;
pub const FETCH: skein_shell::Command;      // `forge fetch`, and so on
pub const PREPARE: skein_shell::Command;
pub const RUN: skein_shell::Command;
pub const REMOVE: skein_shell::Command;
pub const SOURCE: skein_shell::Command;
pub const SIGN_IN: skein_shell::Command;
pub const FORGE_CHILD: skein_shell::Command;
// temper_engine_shell
pub const ENGINE: skein_shell::Table;
pub fn main() -> std::process::ExitCode;
/// `versions`' lines (`../protocol/binaries.md`, 2), each ending in a
/// newline, which `temper-engine versions` and `temper-box versions` print.
pub fn versions() -> String;
```

### 7.2 `temper-worker`

Each worker's host's binary (`../protocol/binaries.md`, 5;
`../draft/cli.md`, 4.4). Its line is `the deployment's worker: a worker
alone, and its programs' sign-in.`; its last, `temper-worker COMMAND
--help prints one command; temper-worker help pages lists the pages.`

| Reached by | Set | Forms and what |
|---|---|---|
| `serve` | public | `[DIRECTORIES]`: a worker alone's document, until stopped |
| `program-login` | public | `[DIRECTORIES] NAME`: a program source signed in on this host |
| `help` | public | `temper_shell::table::HELP` (5) |

- **No bare form and nothing internal:** `temper-worker` alone is
  `Reached::Unknown`'s line, exit 2 (4).
- **Its settings** are the three directories, the table's group
  (`temper_shell::directories::SETTINGS`), as the engine's table holds
  them.
- **`program-login` is one constant both tables hold,**
  `temper-worker-shell`'s, worded as the box's row is now, and run by
  the same code under each binary's name (`../protocol/binaries.md`,
  5). `serve` is the worker's own, saying it serves a worker alone's
  document and naming the binaries that serve the other shapes.
- **Its variables,** as 8 says, each as its code reads it: `TEMPER_JOB`,
  which a worker sets for a job's scripts;
  `TEMPER_INTERNAL_WORKSPACE_REUSE` and `TEMPER_INTERNAL_DUMP_LLM`, which
  a worker alone's start reads; the compile-time `TEMPER_INTERNAL_COMMIT`;
  serve's host's internal variables, as far as a worker alone's run
  reads them; and the host's, by the draft's list.

```rust
// temper_worker_shell
pub const WORKER: skein_shell::Table;
pub const PROGRAM_LOGIN: skein_shell::Command;   // the box's table holds it too
pub const INTERNAL_WORKSPACE_REUSE: skein_shell::Variable;
pub fn main() -> std::process::ExitCode;
```

### 7.3 `temper-agent`

The agent's binary, which a worker starts for each run and a person
never does (`../protocol/binaries.md`, 3; `../draft/cli.md`, 4.5). Every
mode is internal; its public entries are `help`, `-h`, `--help` and
`--version` alone, as every binary's. Its line is `the deployment's
agent, which its worker runs for each run.`; its last, `temper-agent
COMMAND --help prints one command; temper-agent help pages lists the
pages.`

| Reached by | Set | Forms and what |
|---|---|---|
| `help` | public | `temper_shell::table::HELP` (5) |
| the bare form | internal | `AGENT.json`: one agent for a worker's run, on smith's agent configuration |
| `--list` | internal | `AGENT.json`: one endpoint's model list, on smith's agent configuration |
| `--mcp-relay` | internal | `127.0.0.1:PORT`: smith's relay of a program's MCP connection to its agent |
| `--program-child` | internal | `PARENT PROGRAM ARGUMENTS...`: smith's program child, which ends with its parent |
| run as `claude` | internal | `ARGS...`: Claude Code's launcher, `temper-launcher`'s, its grammar delegated |

- **The lines and arguments are today's rows'** (`../protocol/binaries.md`,
  3), the relay's argument naming `SMITH_MCP_SECRET` as smith's.
- **The bare form takes exactly one argument,** the document's path, as
  4 says; a document named as a word, `help`, is given by a path holding
  a slash, `./help`.
- **The delegated modes:** reached by `Table::reach`, as the box's are
  (6). The relay and the program child are smith's `relay::main` and
  `program_child::main`, which read the process themselves until
  smith's round (3), so `--mcp-relay --help` and `--program-child
  --help` print their row's page, as the box's do; run as `claude`, the
  launcher takes every argument and the captured environment.
- **Its variables:** `TEMPER_INTERNAL_DUMP_LLM`, which the run looks up
  in the capture; the compile-time `TEMPER_INTERNAL_COMMIT`; and
  `SSL_CERT_FILE` on run as `claude`'s row, which the launcher reads,
  as on the box's. smith's own stay smith's (8).

```rust
// temper_agent_shell
pub const AGENT: skein_shell::Table;
pub fn main() -> std::process::ExitCode;
/// The rows the box's table holds too, as its pre-table modes (6).
pub const RELAY: skein_shell::Command;
pub const PROGRAM_CHILD: skein_shell::Command;
pub const CLAUDE: skein_shell::Command;
/// One run of AGENT.json, and one endpoint's list
/// (`../protocol/binaries.md`, 3): Ok the exit status; Err the lines for
/// stderr, each after `WORDS: `, `temper-agent` or `temper-agent --list`
/// in this binary, the box's `internal agent` and `internal agent-list`
/// words in the box's.
pub fn run(
    words: &str,
    configuration: &std::path::Path,
    environment: &[temper_shell::directories::EnvironmentEntry],
    working: &Result<std::path::PathBuf, String>,
) -> Result<u8, String>;
pub fn list(words: &str, configuration: &std::path::Path) -> Result<u8, String>;
```

## 8. The environment

- **Each table declares** every variable its binary's code reads, or
  sets under temper's prefix, in the set its name says
  (`skein/docs/foundation/cli.md`, 4 and 5): the directories' twins
  through their group; the host's, by the draft's list
  (`../draft/cli.md`, 4.6), as the code reads them; and temper's own. A
  variable's `Variable` constant lives in the crate that reads it; one
  that several of temper's crates read, the host's among them, lives in
  `temper-shell`'s `environment` module; `HOME` and the XDG base
  directories are skein's (`skein/docs/design/runtime/shell.md`, 14.1).
- **smith's names stay smith's:** `SMITH_MCP_SECRET`, which smith's relay
  reads, is declared in no temper table until smith's round.
- **Renamed** (`../draft/cli.md`, 4.6), each with every reader and
  setter, in code, scripts, `tools/`, CI and documents:

  | Today | After | Read by |
  |---|---|---|
  | `TEMPER_BUNDLE` | `TEMPER_INTERNAL_BUNDLE` | the bundle's build script |
  | `TEMPER_COMMIT`, `TEMPER_CLIENT_COMMIT` | `TEMPER_INTERNAL_COMMIT` | `temper-version`, from its build script alone, into its `COMMIT`, which each binary's package hands its crate (2 and 3) |
  | `TEMPER_DUMP_LLM` | `TEMPER_INTERNAL_DUMP_LLM` | the service and its agents |
  | `TEMPER_FAKE_BOX_PORTS` | `TEMPER_INTERNAL_FAKE_BOX_PORTS` | the box with fake backends |
  | `TEMPER_FORGE_NO_REGISTRATION` | `TEMPER_INTERNAL_FORGE_NO_REGISTRATION` | the forge's sign-in |
  | `TEMPER_R8_CUT_AT_LISTEN`, `TEMPER_R8_WAIT_CHECK` | `TEMPER_INTERNAL_CUT_AT_LISTEN`, `TEMPER_INTERNAL_WAIT_CHECK` | serve's host |
  | `TEMPER_WORKSPACE_REUSE` | `TEMPER_INTERNAL_WORKSPACE_REUSE` | the worker, in the box and in `temper-worker` |

  A compile-time name (`env!`) is declared as internal in each table
  whose binary links its crate, as is the bundle's build variable.
  `TEMPER_INTERNAL_COMMIT`'s `Variable` is
  `temper_shell::environment::INTERNAL_COMMIT`, since every table
  declares it and no table's crate links `temper-version`; its words are
  as they were, so the generated documents (9) don't change.
- **Empty is unset** for each of them (`skein/docs/foundation/cli.md`,
  5), switches included: `TEMPER_INTERNAL_WORKSPACE_REUSE` set to any
  nonempty value turns reuse on, and an empty one leaves it off, as
  `Variable::value` reads it.
- **Twins dropped:** `TEMPER_ECHO`, `TEMPER_SEED`, `TEMPER_READY`,
  `TEMPER_OBSERVATIONS` and `TEMPER_DOCUMENT`. A test that set one
  passes the flag; a flag's precedence over its twin is skein's to test
  (`skein-shell`'s settings tests).
- **`TEMPER_JOB`,** which the worker sets for a job's scripts, is
  declared public, `Sets`, in the table of each binary that runs a
  worker, the box's and `temper-worker`'s: one constant,
  `temper_shell::environment::JOB`, both tables hold.

## 9. The generated documents

Beside this page, rendered by skein (`skein/docs/design/runtime/shell.md`,
14.3):

- each binary's arguments document, `<binary>-args.md` for each of the
  four binaries, each checked byte for byte against its table in its
  crate's tests;
- each binary's environment document, `<binary>-env.md`, and the tree's
  common environment document, `common-env` (`skein/docs/foundation/cli.md`,
  8), which each depend on every table of the tree, checked in
  `temper-box-shell`'s tests from `temper_box_shell::TREE`, the tree's
  tables in this order: `TABLE`, `temper_engine_shell::ENGINE`,
  `temper_worker_shell::WORKER` and `temper_agent_shell::AGENT`, and
  C3's client as it's built. `temper-box-shell` depends on
  `temper-agent-shell` for it, as it does on the engine's and the
  worker's crates already;
- the test set's document, `test-env`, from
  `temper_test_env::TESTS`, checked in `temper-test-env`'s tests (11.1).

`SKEIN_CODEGEN_WRITE=1` rewrites them, as it does the help's generated
pages. A change to an entry changes its documents in the same commit.
The help page `commands` (`../protocol/documentation.md`, 8) is
`commands_page(&TABLE)`, checked the same way.

## 10. Tests

- **Each binary's step tests,** one test binary in its package
  (`tests/suite/`), run the built binary over a table of cases, with a
  cleared environment and the three directories given: `-h`, `--help`
  and `help` print one page, listing exactly the table's public entries;
  an internal command's `--help`, where it has one, says it's internal;
  `--version` prints its name, the workspace's version and a commit of
  twelve hexadecimal digits, `-dirty` or `unknown`; an unknown command
  is refused, exit 2; another binary's document is refused, exit 1, as
  today, by each binary that reads one. A row that checks the version
  line whole reads `temper-version`'s `VERSION` and `COMMIT`, a
  dependency of its package, where the identity enters; no crate's unit
  test names `temper-version`, and a crate's own tests of a `configure`
  that answers `--version` give it an identity of their own, such as
  `("0.0.0", "0123456789ab")`.
- **Dispatch and parsing** stay unit tests of the crate, on its
  `configure` without a process.
- **The agent's suite** keeps a row per mode as well
  (`../protocol/binaries.md`, 3): the relay's and the program child's
  usage and refusals, a run and a list on a missing document and on a
  loopback plaintext one, and the launcher by a link named `claude`.
- **A story in a slower tier** that a rename touches is changed and
  compiled, not run (`AGENTS.md`, 5).

## 11. The test set

What temper's tests, their harnesses, its tools and its scripts read
beside the binaries' tables: the test set (`skein/docs/foundation/cli.md`,
4), named `TEMPER_TEST_*` (5), declared in one test table and read
through it (6 and 8).

### 11.1 Its table

- **The crate:** `temper-test-env`, at `temper/testing/temper-test-env`,
  in the harness profile (`tools/lints/design.md`, 2.2), whose one
  dependency is `skein-shell`. It's a crate of its own, not a module of
  `temper-fixtures`, because the unit tests of temper's crates read the
  set as the worlds do, and `temper-fixtures` brings jig's client with
  it. A test crate takes it as a dev-dependency; a test-support crate
  that reads the set, as `temper-fixtures` does, as a dependency.
- **The table:** `temper_test_env::TESTS`, a `skein_shell::TestTable`
  (`skein/docs/design/runtime/shell.md`, 14.1) whose tree is `temper`,
  holding every variable of 11.3 in the order of their names. Each is a
  `pub const` `Variable` of its own, named for it without
  `TEMPER_TEST_`: `TEMPER_TEST_LIVE_STORE` is
  `temper_test_env::LIVE_STORE`. Each is `Set::Test` and `Way::Reads`,
  since a test, a harness, a tool or a script reads it; its form,
  absent and notes are as its readers take it, and its line says what
  it's for and who sets it.
- **Each is read:** by a test or a harness through 11.2, by one of
  temper's tools, or by a script. One that nothing reads leaves the
  table, as `TEMPER_TEST_CLAUDE` does (11.3).
- **Its document:** `test-env`, beside this page, is
  `TESTS.document()` (`skein/docs/design/runtime/shell.md`, 14.3),
  checked byte for byte in `temper-test-env`'s tests and rewritten when
  11.2's `rewrite()` is on, as section 9's documents are. Check 5
  (`skein/docs/foundation/cli.md`, 9) reads it with the others.

### 11.2 Reading it

`temper-test-env` is the one place temper's test code reads the
process's environment. It reads it through `skein_shell::environment`,
under one scoped
`#[expect(clippy::disallowed_methods, reason = "a test harness: ...")]`
(`skein/docs/foundation/cli.md`, 9, check 4), and offers:

```rust
/// A declared variable's value in the process's environment: none
/// when it's absent or empty.
pub fn value(variable: &skein_shell::Variable) -> Option<std::ffi::OsString>;

/// The process's environment, as `skein_shell::environment` returns it,
/// for a test that hands a child the environment it was given.
pub fn environment() -> Box<[(Box<[u8]>, Box<[u8]>)]>;

/// Whether the rewrite switch is on: `SKEIN_CODEGEN_WRITE` is `1`.
pub fn rewrite() -> bool;
```

- **A lookup names a declared entry,** never a string: a variable of
  `TESTS`; a host's variable by `temper_shell::environment`'s constant,
  as `PATH` and `TERM`; an internal one by its crate's, as
  `temper_service_shell::WAIT_CHECK`; or one of three this crate
  declares outside `TESTS`, since no table of temper's may hold them:
  - `TMPDIR`, the host's, which no binary of temper reads;
  - `SKEIN_CODEGEN_WRITE` and `SKEIN_TEST_CHROMIUM`, skein's, until
    skein's round declares them in skein's test table and this crate
    names those instead.
- **Empty is unset** (`skein/docs/foundation/cli.md`, 5): a variable
  that was on by being set at all, as `TEMPER_RATE` was, is on only
  when it's nonempty.
- **`rewrite()`** is the rewrite switch section 9 names. Every checked
  document and golden of temper's tests asks it, and `1` alone turns it
  on, as most of them took it already.
- **Cargo's directory** is `env!("CARGO_MANIFEST_DIR")`, at compile
  time, not a read of the process.
- **No other test code reads the process,** so no `expect` whose reason
  starts `cli C4` is left. A tool's own reads stay its own until
  `../draft/cli.execution.md`'s C8 gives it a table: the git transcripts
  tool reads `TEMPER_TEST_FORGEJO` under its own `expect`.
- **Setting one for a child** stays the child's command's
  (`std::process::Command::env`), naming the constant's `name`.

### 11.3 The names

The draft's renames (`../draft/cli.md`, 4.6): each name below takes
`TEMPER_TEST_` before the rest of its name, unchanged, so a name is
found by its old one: `TEMPER_BOX_PARENT_TEST` becomes
`TEMPER_TEST_BOX_PARENT_TEST`. Every reader and setter changes with it:
code, scripts, `tools/tiers/`, CI, `.config/gate/` and the design
documents.

| Today | Read by |
|---|---|
| `TEMPER_BOX_PARENT_TEST` | `temper-box`'s process tests |
| `TEMPER_E4_CAPTURE`, `TEMPER_E4_LIFE`, `TEMPER_E4_RESTART` | `temper-box`'s Forgejo-elsewhere support |
| `TEMPER_FORGE_PARENT_HELPER` | `temper-box`'s startup suite |
| `TEMPER_FORGEJO`, `TEMPER_FORGEJO_RUNNER` | `temper-box`'s owned-forge support, the checkout world's real delivery and the git transcripts tool; CI sets them |
| `TEMPER_LIVE_ACCOUNT`, `TEMPER_LIVE_CLAUDE`, `TEMPER_LIVE_KEEP`, `TEMPER_LIVE_NOTICE`, `TEMPER_LIVE_STORE` | the live tier's stories and `temper/scripts/proof-live.sh` |
| `TEMPER_LIVE_GITHUB_FIXTURES`, `TEMPER_LIVE_GITHUB_KEEP`, `TEMPER_LIVE_GITHUB_OWNER` | the live GitHub story and `temper/scripts/proof-live-github.sh` |
| `TEMPER_MANAGED_CLIENT`, `TEMPER_MANAGED_EVIDENCE`, `TEMPER_MANAGED_MODE`, `TEMPER_MANAGED_PERSON`, `TEMPER_MANAGED_PREFIX`, `TEMPER_MANAGED_SCRATCH`, `TEMPER_MANAGED_UPSTREAM` | the managed sitting's relay and fake client, which the sitting and `temper/scripts/proof-live-github-managed.py` set |
| `TEMPER_RATE` | the forge protocol's rate suite |
| `TEMPER_RECORD_RELEASE` | `temper-client-view`'s confirm tests |
| `TEMPER_SITTING_WAIT_SECS` | `temper/scripts/proof-live-github-managed.py` |
| `TEMPER_WRAPPER` | temper's proof scripts and the real loop's fixture scripts |

- **Kept:** `TEMPER_TEST_BUNDLE` and `TEMPER_TEST_PLAYGROUND`, which
  `tools/tiers/` sets for the browser tier and the browser world reads.
- **Gone:** `TEMPER_TEST_CLAUDE`, which `tools/tiers/` and CI set and
  nothing reads. The process tier still builds `fake-claude`, which
  temper's tests run as a sibling binary, and names it in no variable.
- **`TEMPER_BENIGN`,** a made-up name in the program login's tests'
  data, is `BENIGN_SETTING`, outside every tree's prefix
  (`skein/docs/foundation/cli.md`, 9, check 5).
- **Nothing stored** names any of them, so no stored form changes.

## 12. Open questions

None now; `../draft/cli.status.md` keeps what's open.
