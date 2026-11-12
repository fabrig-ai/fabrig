# The command line

Provisional, 2026-11-09. How the repository's binaries meet the people
and the programs that run them: their environment variables, their
arguments, and how they're built. It is the repository's convention
beside the programming model, for every tree. Where a tree doesn't
follow it yet, a check's exception says so, and its exceptions only
shrink (section 9). The reasons are the user's, of 2026-11-08
(`temper/docs/design/draft/notes.md`, 13). skein's mechanism for it is
`skein-shell`'s table (`../design/runtime/shell.md`, 14).

## 1. In one page

- **One layout:** each binary is a package of its own at
  `<tree>/bin/<binary>/`. Its `main.rs`, at most 30 lines, calls into a
  crate, and its logic lives under `<tree>/crates/` (section 2).
- **No Cargo features:** `cargo build` at the root builds everything,
  and a binary links exactly its package's dependencies, whatever is
  built beside it. What a feature chose becomes a crate, a runtime
  setting or a fixtures crate (section 3).
- **Three sets:** public (stable, advertised by help), internal (hidden,
  may change without notice) and test (read by tests alone). A
  variable's name says its set; a command's or a flag's set is in its
  binary's table, and its name doesn't change with it (sections 4 and
  5).
- **One table per binary** declares every command, flag and environment
  variable it reads. Help and the reference documents are rendered from
  it, and code reads the process's arguments and environment only
  through it (section 6).
- **One help:** `-h`, `--help` and `help` print the same page of public
  commands, flags and variables, in the same format for every binary
  (section 7).
- **The reference:** `<tree>/docs/design/cli/`, generated from the
  tables and checked against them (section 8).
- **Checked by code:** the layout, `main.rs`'s size, no features,
  environment access, declared names, the generated documents and help
  (section 9).

## 2. Binaries

- **A binary is a package of its own** at `<tree>/bin/<binary>/`:
  - a `Cargo.toml` naming the package as the binary;
  - one binary target, at `main.rs` beside it;
  - no library.

  A library crate holding the name is renamed.
- **`main.rs` is plumbing, at most 30 lines:** it installs fail-stop
  (`programming-model.md`, 2) and calls one function of one crate under
  `<tree>/crates/`, which holds the binary's table and everything it
  does. With fail-stop, argument capture and error mapping in that
  crate, a `main.rs` is under 10 lines. 30 leaves room for a module
  comment and imports, not for logic.
- **Every binary follows this,** a product, a tool or a fixture. A
  tool's or a fixture's commands and variables are internal or test.
- **A binary's package's dependencies decide what it links,** and
  nothing else does (section 3).

## 3. No Cargo features

- **No workspace crate declares `[features]`.** No dependency on a
  workspace crate is `optional`, or sets `default-features` or
  `features`. An external crate's features are chosen once, in the
  root's `[workspace.dependencies]`.
- **`cargo build` at the root builds every member,** and once no tree
  has features left, each binary is the same, built alone or with the
  rest.
- **What a feature did goes to one of three places:**
  - **a crate,** when it chose code a binary links or doesn't: the
    binary that needs it depends on it;
  - **a runtime setting, internal,** when it switched code every build
    has;
  - **a fixtures crate beside the crate,** when it exposed fixtures to
    other crates' tests, which depend on it.
- **A fixtures crate** is the package `<crate>-fixtures`, in its tree's
  `testing/`, and holds what other crates' tests, the crate's own
  integration tests among them, read of the crate's fixtures. It is
  built on the crate's public items. Where a fixture needs what the
  crate keeps private, the crate makes public the least that serves,
  in every build, named for what it gives rather than for tests. A
  fixture no test outside the crate reads needs no fixtures crate, and
  one only re-exporting another crate's leaves its readers to take
  that crate's.
- **A crate's own step tests keep their fixtures:** step tests that
  read a crate depending on their own build their crate twice, so its
  types differ between the two. They keep, under `#[cfg(test)]`, the
  fixtures they read, and a fixture both they and other crates' tests
  read is held in each: the crate's copy for its step tests, the
  fixtures crate's for the rest.
- **A test's fake program** that a package's tests run is the fake's
  package's own binary, found where the build puts it beside the
  binary under test; the package under test takes the fake's crate as
  a dev-dependency alone. A run that hasn't built it fails, naming the
  build that makes it.
- **No other switch chooses what's built:** no custom `--cfg` and no
  `RUSTFLAGS` in the repository. The one exception is development
  speed: a declared internal variable may leave out a step of a build
  that nothing else needs, as temper's leaves out the browser's bundle.
  A build without it builds everything.
- **Size:** a feature that becomes a crate leaves a binary linking what
  it linked, so most change nothing. One that becomes a runtime setting
  compiles its code into every build: the change that removes such a
  feature reports each binary's size before and after.

## 4. Three sets

| Set | For | Help | Reference | Changes |
|---|---|---|---|---|
| Public | people and their scripts | listed | its binary's documents, first | as the design changes: compatible once a tree's releases are |
| Internal | the binaries' own processes, tests and developers | not listed, though an internal command answers its own `--help`, headed "internal" | the same documents, after "Internal" | any time, with its readers and setters in the same change |
| Test | tests, harnesses and tools | none | the test set's document (section 8) | any time |

- **The host's variables** that a binary reads, like `HOME`, `PATH`
  and `XDG_*`, aren't ours to name. The common document (section 8)
  lists each and what it does: they're public in effect.
- **Variables a binary sets for the programs it runs:**
  - those with the tree's prefix are declared in that binary's table,
    in their set;
  - others' names, like `GIT_*` or `LC_ALL`, stay with the protocol
    that sets them.

## 5. Names

- **Environment variables:** `<TREE>_<NAME>` is public,
  `<TREE>_INTERNAL_<NAME>` internal and `<TREE>_TEST_<NAME>` test.
  `<TREE>` is the tree's name in capitals: `SKEIN`, `SMITH`, `JIG`,
  `TEMPER`, `HEARTH`. A variable's name says its set wherever a person
  meets it: in a shell, a unit or a script.
- **Commands and flags** keep their names when their set changes, since
  the table holds the set.
  - Words are lower case, with hyphens.
  - Flags are long, `--name VALUE` or `--name`, and `-h` is the only
    short one.
- **A flag has an environment twin only where the table declares one,**
  named by the flag's set: `--data` and `TEMPER_DATA`.
  - Twins are for what holds across a host or a session, like a
    deployment's directories. An invocation's choice stays a flag.
  - A flag wins over its twin.
- **Empty is unset,** for every variable, switches included.
- **Exit codes:** 0, done; 1, failed; 2, refused its arguments, after
  saying what was wrong and how to get help.
- **A delegated grammar stays its program's:** a binary acting as
  another program, as a fake git or a launcher, takes that program's
  arguments and exit codes. These conventions are for a binary's own
  grammar.

## 6. One table per binary

- **The binary's crate holds one table:**
  - its commands, with their positional arguments and flags;
  - the environment variables it reads, or sets under its tree's
    prefix.

  Each entry has its name, its set, its form, one line, what holds when
  it's absent, and notes.
- **The table's types and their rendering are skein's,** in
  `skein-shell` beside its settings (`../design/runtime/shell.md`, 6.5 and 14),
  so every tree's binaries share one format. A tree's rows are its own.
- **Code reads the process's arguments and environment once,** in the
  binary's crate, through the table.
  - `std::env`'s `args`, `args_os`, `var`, `var_os`, `vars`, `vars_os`,
    `set_var` and `remove_var` are disallowed everywhere else, and so
    are `skein_shell::arguments` and `skein_shell::environment`, which
    read them for a binary's crate.
  - A lookup in the captured environment names a declared entry, not a
    string.
  - A helper another tree gives the binary takes the captured arguments
    and environment rather than reading the process's.
- **Two other places read the environment directly,** each read under a
  scoped `expect`, naming declared variables:
  - a build script, reading Cargo's variables and its own build-time
    ones;
  - a test harness, reading the test set through its tree's test table
    (section 8).

  A name a build script hands the code at compile time (`env!`) is
  declared like any other, as internal.

## 7. Help

- **`<binary> -h`, `--help` and `help` print the same page,** in this
  order:
  1. one line: what the binary is;
  2. usage: its public forms;
  3. commands: each public command's form and one line;
  4. flags: the public flags, then `-h, --help` and `--version`;
  5. environment: the public variables it reads, each with its twin;
  6. one line on where to read more.
- **`<binary> COMMAND -h` or `--help`** prints that command: its forms,
  arguments, flags with their defaults, its variables and notes.
- **Pages,** for a binary that has them:
  - `<binary> help NAME` prints a page;
  - `help pages` prints their index;
  - `help --search WORDS...` searches;
  - `help commands` prints the binary's own table, not another's.
- **`--version`** prints the binary's name, version and commit, on
  every binary. It has no `version` command beside it.
- **The page:** lines fit 80 columns. Help goes to stdout, exits 0, and
  reads no directory and no configuration.

For example, once temper's tables are built, `temper-box --help` prints:

```
temper-box: temper on this machine, as one box.

usage:
  temper-box [--config DIR] [--data DIR] [--cache DIR] [--echo none|steps]
  temper-box COMMAND [ARGUMENT...]

commands:
  init [DIRECTORIES]                 Write a deployment that starts with
                                     temper's defaults.
  serve [DIRECTORIES]                Run a deployment in one process, until
                                     stopped.
  program-login [DIRECTORIES] NAME   Sign a program source in on this host.
  help [NAME | pages | --search WORDS...]
                                     Print this, a page, the pages, or a search.

flags:
  --config DIR, --data DIR, --cache DIR
                                     The deployment's three directories.
  -h, --help                         Print this.
  --version                          Print the name, version and commit.

environment:
  TEMPER_CONFIG, TEMPER_DATA, TEMPER_CACHE
                                     As --config, --data and --cache.

temper-box COMMAND --help prints one command; temper-box help pages lists the
pages.
```

## 8. The reference documents

- **Under `<tree>/docs/design/cli/`:**
  - for each binary, `<binary>-args.md` (its commands, arguments and
    flags) and `<binary>-env.md` (the variables it alone reads or sets);
  - `<tree>/docs/design/cli/common-env.md`, for the variables two or
    more of the tree's binaries read, the host's among them;
  - `<tree>/docs/design/cli/test-env.md`, for the test set, from the tree's test table: one
    table in its test-support crate, through which tests read them.
- **Each is generated from the tables:** public entries first, then
  "Internal", each with its set, form, default, line and notes. A test
  checks each byte for byte, and the tree's rewrite switch rewrites
  them, as it does the tree's other generated pages. A change to an
  entry changes its document in the same commit.
- **They're the contract** for what they list. Design documents cite
  them rather than restate the names.

## 9. The checks

Each runs in the focused suite, in `tests/boundary/` unless it says
otherwise.

1. **Layout:** from `cargo metadata`, every binary target is
   `<tree>/bin/<name>/main.rs`, in a package named `<name>` with no
   library and no other binary.
2. **`main.rs`:** every binary target's source is at most 30 lines, the
   same files `fail_stop.rs` reads.
3. **No features:** every workspace manifest, read as `direction.rs`
   reads them: no `[features]` table, no `optional`, `default-features`
   or `features` on a member's dependency, and none on a workspace
   crate's entry in the root's `[workspace.dependencies]`. An external
   crate's entry there may choose its features.
4. **Environment access:** clippy's `disallowed-methods` in every
   variant of `tools/lints/` lists section 6's functions
   (`../../../tools/lints/design.md`, 4). Each read says why with a
   scoped `expect` whose reason starts with what it is: the binary's
   capture, a build script, a test harness, or the round or milestone
   that removes it. smith's own clippy configurations, outside
   `tools/lints/`, take the rule in smith's round.
5. **Declared names:** every token `<TREE>_[A-Z0-9_]+` in the tracked
   files outside a `docs/` directory (code, scripts, manifests and CI)
   is declared in a table or in the test set's document, in the set its
   name says. Every declared variable is read or set somewhere.
   Made-up names in tests' data take no tree's prefix, so the check
   needs no exclusion for them.
6. **Documents:** each binary's generated documents match its table, in
   the tests of the crate that holds it.
7. **Help,** in each binary's step tests:
   - `-h`, `--help` and `help` print the same page, listing exactly the
     public entries;
   - an internal command's `--help` says it's internal;
   - `--version` prints the name, version and commit.

Checks 1, 2, 3 and 5 carry exceptions in their source, each with the
round or milestone that removes it:

- the other trees' binaries and names, until their own rounds;
- temper's own, until the milestone of
  `temper/docs/design/draft/cli.execution.md` that brings each into line
  lands and removes its exception.

A check's exceptions only shrink: one that no longer matches anything
fails its check, so the change that brings a thing into line removes
its exception.

## 10. Later

- smith's, skein's, jig's and hearth's binaries and names, each in its
  tree's own round of these conventions.
- Shell completions, and `NO_COLOR`.
