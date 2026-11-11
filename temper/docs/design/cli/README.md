# temper's command lines

How temper's binaries meet their command lines, on the repository's
convention (`skein/docs/foundation/cli.md`) and skein's table
(`skein/docs/design/runtime/shell.md`, 14): each binary's package and
crate, its table, how its arguments reach a command, its help, its
exits, the environment it reads and passes down, and the reference
documents generated from its table. The sets are the draft's
(`../draft/cli.md`, section 4); this document gives `temper-box`'s and
`temper-engine`'s. `temper`, the client, and `temper-worker` and
`temper-agent` take theirs in their own milestones
(`../draft/cli.execution.md`, C3 and C6).

## 1. In one page

- **A binary is a package** at `temper/bin/<binary>/`, whose `main.rs`
  installs fail-stop and calls its crate's `main` (2). The box's crate
  is `temper-box-shell`, the engine's `temper-engine-shell`.
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
- **The tables** of `temper-box` (6) and `temper-engine` (7), and the
  variables each declares (8).
- **Generated:** each binary's `<binary>-args.md` and `<binary>-env.md`,
  and the tree's common environment document, beside this page, checked
  byte for byte (9).

## 2. The binaries and their crates

| Binary | Package | Its crate | Its table |
|---|---|---|---|
| `temper-box` | `temper/bin/temper-box` | `temper-box-shell` (`temper/crates/temper-box-shell`) | `temper_box_shell::TABLE` |
| `temper-engine` | `temper/bin/temper-engine` | `temper-engine-shell` | `temper_engine_shell::ENGINE` |

- **`main.rs`** is the package's whole source:

  ```rust
  //! The box's binary (temper/docs/design/cli/README.md, 2).
  fn main() -> std::process::ExitCode {
      skein_fail_stop::install();
      temper_box_shell::main()
  }
  ```

  Its `[dependencies]` are its crate and `skein-fail-stop`, which decide
  what it links (`skein/docs/foundation/cli.md`, 2).
- **The libraries that held the names are renamed:** the whole
  product's library, `temper` at `temper/crates/temper`, is
  `temper-box-shell` at `temper/crates/temper-box-shell`, holding the
  box's table and everything it runs; the box's machine, `temper-box` at
  `temper/crates/temper-box`, is `temper-box-machine` at
  `temper/crates/temper-box-machine`. The engine's package,
  `temper/crates/temper-engine`, moves to `temper/bin/temper-engine`,
  and its table and dispatch move into `temper-engine-shell`.
- **Tests that run a binary live in its package,** under its `tests/`,
  since Cargo gives `CARGO_BIN_EXE_<name>` to its own package's tests
  alone: the whole product's integration tests, with their `[[test]]`
  targets, fixtures and support, move from `temper/crates/temper/tests/`
  to `temper/bin/temper-box/tests/`, and the engine's from
  `temper/crates/temper-engine/tests/` to `temper/bin/temper-engine/tests/`.
  They are dev-dependencies' users, which link nothing into the binary.
  Unit tests stay in their crate.
- **A crate's features stay its own** until `../draft/cli.execution.md`'s
  C5: `temper-box-shell` keeps the whole product's `record`, and the
  checks' feature exceptions that named `temper/crates/temper/Cargo.toml`
  name `temper/crates/temper-box-shell/Cargo.toml`, the same exceptions
  under the path they moved to.

## 3. Its `main`: the capture and the reach

Each binary's crate has `pub fn main() -> std::process::ExitCode`:

1. **The capture,** one function, the only place the binary reads the
   process: argv's first word, the arguments after it, the environment
   (`skein_shell::arguments` and `skein_shell::environment`, with
   `std::env::args_os` for argv's first word) and the working directory,
   under one `#[expect(clippy::disallowed_methods, reason = "the binary's
   capture: ...")]` (`skein/docs/foundation/cli.md`, 9, check 4).
2. **The reach,** `TABLE.reach(run_as, &arguments)`
   (`skein/docs/design/runtime/shell.md`, 14.4):
   - `Help`: `TABLE.help()` on stdout, exit 0;
   - `Version`: `TABLE.version_line()` on stdout, exit 0;
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

The version and commit in each table are `temper_version::VERSION` and
`temper_version::COMMIT`, so `--version` prints `temper-box VERSION
(COMMIT)` as `../release.md`, section 3, words it.

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
skein's table shares, beside the `commands` module the client and the
worker keep until C3 and C6:

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
| `versions`, `upgrade` | public | `../protocol/binaries.md`'s, once they're built for the box |
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
  forge's commands and `internal forge-child`. The box's
  `program-login` is its own constant, worded as
  `temper-worker-shell`'s row, which the worker keeps until C6.

## 7. `temper-engine`'s table

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
  | `TEMPER_COMMIT`, `TEMPER_CLIENT_COMMIT` | `TEMPER_INTERNAL_COMMIT` | `temper-version`, from its build script alone; `temper-client` reads `temper-version`'s |
  | `TEMPER_DUMP_LLM` | `TEMPER_INTERNAL_DUMP_LLM` | the service and its agents |
  | `TEMPER_INTERNAL_FAKE_BOX_PORTS` | `TEMPER_INTERNAL_FAKE_BOX_PORTS` | the box with fake backends |
  | `TEMPER_INTERNAL_FORGE_NO_REGISTRATION` | `TEMPER_INTERNAL_FORGE_NO_REGISTRATION` | the forge's sign-in |
  | `TEMPER_INTERNAL_CUT_AT_LISTEN`, `TEMPER_INTERNAL_WAIT_CHECK` | `TEMPER_INTERNAL_CUT_AT_LISTEN`, `TEMPER_INTERNAL_WAIT_CHECK` | serve's host |
  | `TEMPER_WORKSPACE_REUSE` | `TEMPER_INTERNAL_WORKSPACE_REUSE` | the worker |

  A compile-time name (`env!`) is declared as internal in each table
  whose binary links its crate, as is the bundle's build variable.
- **Twins dropped:** `TEMPER_ECHO`, `TEMPER_SEED`, `TEMPER_READY`,
  `TEMPER_OBSERVATIONS` and `TEMPER_DOCUMENT`. A test that set one
  passes the flag; a flag's precedence over its twin is skein's to test
  (`skein-shell`'s settings tests).
- **`TEMPER_JOB`,** which the worker sets for a job's scripts, is
  declared public, `Sets`, in the table of each binary that runs a
  worker.

## 9. The generated documents

Beside this page, rendered by skein (`skein/docs/design/runtime/shell.md`,
14.3):

- each binary's arguments document, `<binary>-args.md` for `temper-box`
  and for `temper-engine`, each checked byte for byte against its table
  in its crate's tests;
- each binary's environment document, `<binary>-env.md`, and the tree's
  common environment document, `common-env` (`skein/docs/foundation/cli.md`,
  8), which each depend on every table of the tree, checked in
  `temper-box-shell`'s tests from `temper_box_shell::TREE`, the tree's
  tables: `TABLE` and `temper_engine_shell::ENGINE` now, and C3's and
  C6's as they're built.

`SKEIN_CODEGEN_WRITE=1` rewrites them, as it does the help's generated
pages. A change to an entry changes its documents in the same commit.
The help page `commands` (`../protocol/documentation.md`, 8) is
`commands_page(&TABLE)`, checked the same way.

## 10. Tests

- **Each binary's step tests,** one test binary in its package
  (`tests/suite/`), run the built binary over a table of cases, with a
  cleared environment and the three directories given: `-h`, `--help`
  and `help` print one page, listing exactly the table's public entries;
  an internal command's `--help` says it's internal; `--version` prints
  its name, the workspace's version and a commit of twelve hexadecimal
  digits, `-dirty` or `unknown`; an unknown command is refused, exit 2;
  another binary's document is refused, exit 1, as today.
- **Dispatch and parsing** stay unit tests of the crate, on its
  `configure` without a process.
- **A story in a slower tier** that a rename touches is changed and
  compiled, not run (`AGENTS.md`, 5).

## 11. Open questions

None now; `../draft/cli.status.md` keeps what's open.
