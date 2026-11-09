# ui-cli-c1: contracts

The shapes that cross this plan's tasks. Binding: a task builds its half
to them, and a change to one is a question.

## 1. skein-shell's table (`skein/docs/design/shell.md`, 15.1)

Portable, beside `settings.rs` (no `cfg(unix)`; it builds for
`wasm32-unknown-unknown`). Module layout, created by `table-types`:

- `skein/crates/skein-shell/src/table.rs`: the types below, and
  `mod help;` and `mod reference;` declared in it.
- `src/table/help.rs`: `render-help`'s, holding `impl Table` blocks.
  `table-types` creates it with its module doc alone.
- `src/table/reference.rs`: `render-docs`'s, holding `impl Table` and
  `impl TestTable` blocks. `table-types` creates it with its module doc
  alone.
- `src/table/demo.rs`, `#[cfg(test)]`: the demo table both render
  tasks test with (section 1.3), `table-types`'s.

`lib.rs` re-exports, from `table-types` on: `Set`, `Table`, `Reach`,
`Command`, `Argument`, `Switch`, `Variable`, `Way`, `TestTable`, `HOME`.
The render tasks add methods, so they add no export.

### 1.1 The types

Every type derives `Clone, Copy, PartialEq, Eq, Debug`.

```rust
/// Who an entry is for (skein/docs/foundation/cli.md, 4).
pub enum Set { Public, Internal, Test }

/// A binary's table (shell.md, 15.1).
pub struct Table {
    /// The binary's name: `temper-box`.
    pub binary: &'static str,
    /// One line: what the binary is. Help's first line is `BINARY: LINE`.
    pub line: &'static str,
    /// The tree's build identity, as it words it: `0.1.0`, `0123456789ab`.
    pub version: &'static str,
    pub commit: &'static str,
    /// In help's order.
    pub commands: &'static [Command],
    /// The groups of settings every command takes: a deployment's directories.
    pub settings: &'static [&'static Settings],
    /// The variables it reads or sets beside its settings' twins.
    pub variables: &'static [&'static Variable],
    /// Help's last line: where to read more.
    pub more: &'static str,
}

/// How a command is reached.
pub enum Reach {
    /// Its words after the binary's name; none for the bare binary's form.
    Words(&'static [&'static str]),
    /// The name the binary runs as, for a delegated grammar: `git`.
    RunAs(&'static str),
}

pub struct Command {
    pub reach: Reach,
    pub set: Set,
    /// Each form's arguments, as usage shows them after the name: `[DIRECTORIES] NAME`.
    pub forms: &'static [&'static str],
    pub line: &'static str,
    pub arguments: &'static [Argument],
    /// The groups of settings it takes beside the table's.
    pub settings: &'static [&'static Settings],
    pub switches: &'static [Switch],
    /// The variables it reads or sets beside its settings' twins.
    pub variables: &'static [&'static Variable],
    pub notes: &'static str,
}

pub struct Argument { pub shown: &'static str, pub line: &'static str }

/// A flag with no value: `--front`.
pub struct Switch { pub name: &'static str, pub set: Set, pub line: &'static str, pub notes: &'static str }

/// Whether a binary reads a variable or sets it for the programs it runs.
pub enum Way { Reads, Sets }

pub struct Variable {
    /// `TEMPER_DATA`, `HOME`.
    pub name: &'static str,
    pub set: Set,
    /// What its value is: `DIR`, `1`, `none|steps`.
    pub form: &'static str,
    pub line: &'static str,
    /// What holds when it's absent or empty.
    pub absent: &'static str,
    pub notes: &'static str,
    pub way: Way,
}

impl Variable {
    /// Its bytes in the captured environment, the pairs `environment`
    /// returns; an empty value is unset.
    #[must_use]
    pub fn value<'a>(&self, environment: &'a [(Box<[u8]>, Box<[u8]>)]) -> Option<&'a [u8]>;
}

/// A tree's test set (skein/docs/foundation/cli.md, 8): its variables, all `Set::Test`.
pub struct TestTable { pub tree: &'static str, pub variables: &'static [Variable] }

/// The host's home directory, read by a setting's `Fallback::Base`.
pub const HOME: Variable;
```

`Base` gains `pub const fn variable(self) -> &'static Variable`: its
XDG variable's entry (`XDG_CONFIG_HOME`, `XDG_DATA_HOME`,
`XDG_STATE_HOME`, `XDG_CACHE_HOME`), each `Set::Public`, `Way::Reads`.
`HOME` and these four are constants in `table.rs`; their lines say what
the kit does with them.

### 1.2 Settings (`shell.md`, 6.5)

```rust
pub struct Settings {
    /// The tool's name, the last part of an XDG default: `temper`.
    pub tool: &'static str,
    /// One line for the group's flags in help: `The deployment's three directories.`
    pub about: &'static str,
    pub table: &'static [Setting],
}

pub struct Setting {
    pub name: &'static str,
    pub value: &'static str,
    pub about: &'static str,
    pub fallback: Fallback,
    /// New: its set.
    pub set: Set,
    /// New: its environment twin, declared, never derived: `Some("TEMPER_CONFIG")`.
    pub twin: Option<&'static str>,
    /// New: its notes, for the reference documents; empty for none.
    pub notes: &'static str,
}
```

`resolve`, `help`, `Refusal` and the rest keep their signatures. A
setting with no twin reads no variable; a refusal and `help` name the
twin only where there is one.

### 1.3 The demo table (`src/table/demo.rs`)

Made-up names, no tree's prefix. `table-types` writes it; the render
tasks only read it. It holds at least:

- `DEMO: Table`: binary `demo-box`, line `demo on this machine, as one
  box.`, version `0.1.0`, commit `0123456789ab`, more `demo-box COMMAND
  --help prints one command; demo-box help pages lists the pages.`;
- the table's group `DIRECTORIES: Settings` (tool `demo`, about `The
  deployment's three directories.`): `config`, `data`, `cache`, each
  `DIR`, public, twins `DEMO_CONFIG`, `DEMO_DATA`, `DEMO_CACHE`, XDG
  defaults;
- commands, in this order: the bare form (`Reach::Words(&[])`, public,
  form `[--config DIR] [--data DIR] [--cache DIR] [--echo none|steps]`,
  with an `ECHO` group: `echo`, `none|steps`, public, no twin, default
  `none`, and the internal switch `front`); `init`, `serve` and
  `program-login` (public, forms `[DIRECTORIES]`,
  `[DIRECTORIES]`, `[DIRECTORIES] NAME`, the last with the argument
  `NAME`), with the lines of `skein/docs/foundation/cli.md`, 7's
  example; `help` (public, form `[NAME | pages | --search WORDS...]`,
  line `Print this, a page, the pages, or a search.`); `forge`
  (internal, form `COMMAND...`, notes); and `Reach::RunAs("git")`
  (internal, form `ARGUMENT...`);
- variables: `DEMO_JOB` (public, `Way::Sets`, on `serve`) and
  `DEMO_INTERNAL_DUMP` (internal, `Way::Reads`, on the table);
- `DEMO_TESTS: TestTable`, tree `demo`, with `DEMO_TEST_RATE` and
  `DEMO_TEST_WRAPPER`;
- `OTHER: Table`, binary `demo-engine`, taking the same `DIRECTORIES`
  group, with one public command `serve` and no other variable: so
  that the three twins are read by two tables, and `DEMO_JOB` and
  `DEMO_INTERNAL_DUMP` by one.

## 2. The environment's reads (`skein/docs/foundation/cli.md`, 6 and 9, check 4)

Every variant of `tools/lints/clippy/` lists, in `disallowed-methods`,
with the one reason `"a binary's crate reads its process's arguments and
environment once, through its table (skein/docs/foundation/cli.md, 6)"`:

```toml
{ path = "std::env::args", reason = "..." },
{ path = "std::env::args_os", reason = "..." },
{ path = "std::env::var", reason = "..." },
{ path = "std::env::var_os", reason = "..." },
{ path = "std::env::vars", reason = "..." },
{ path = "std::env::vars_os", reason = "..." },
{ path = "std::env::set_var", reason = "..." },
{ path = "std::env::remove_var", reason = "..." },
{ path = "skein_shell::arguments", reason = "...", allow-invalid = true },
{ path = "skein_shell::environment", reason = "...", allow-invalid = true },
```

Each read is a scoped `#[expect(clippy::disallowed_methods, reason =
"...")]` on the smallest item that holds it (a function, an impl, a
`const`), or a test file's `#![expect(...)]` where four or more of its
items read. Its reason starts with one of these, then `: ` and why:

| Prefix | For |
|---|---|
| `capture` | the one place a binary's crate reads its process's arguments or environment, and `skein_shell::arguments` and `environment` themselves |
| `build script` | a build script reading Cargo's variables or its own |
| `cli C2` … `cli C8` | a temper read the milestone removes, by section 4's map |
| `skein's round`, `jig's round`, `hearth's round` | another tree's read, until its own round |

An item that already has a `disallowed_methods` expectation for
`std::env::var` keeps it; in temper its reason is rewritten to start
with the prefix, and in the other trees it stays as it is.

## 3. The checks' exceptions (`tests/boundary/`)

Each check keeps its exceptions in its source, as a constant table:

```rust
/// What a check excepts, and the round or milestone that removes it.
struct Exception { what: &'static str, until: &'static str }
```

- `until` is `cli C2` to `cli C8`, by section 4's map, or `skein's
  round`, `smith's round`, `jig's round`, `hearth's round`.
- A finding no exception names fails the check, all of them in one
  message; an exception that names no finding fails it too, saying to
  remove it (`skein/docs/foundation/cli.md`, 9: exceptions only shrink).
- `what` is, for checks 1 and 2, `PACKAGE TARGET` as `cargo metadata`
  names them (`temper temper`); for check 3,
  `MANIFEST: [features]` or `MANIFEST: DEPENDENCY` with the manifest's
  path from the root and the dependency's key
  (`temper/crates/temper-service/Cargo.toml: smith-channel`), and
  `Cargo.toml: KEY` for the root's `[workspace.dependencies]`; for
  check 5, the token (`TEMPER_DUMP_LLM`).

## 4. Which milestone removes what, in temper

From `temper/docs/design/draft/cli.execution.md`, 1.

| Milestone | Binaries (checks 1, 2) | Features (check 3) | Reads (check 4) | Names (check 5) |
|---|---|---|---|---|
| `cli C2` | `temper temper`, `temper-engine temper-engine` | | a read in a crate the `temper` or `temper-engine` binary links, a test's excepted | `TEMPER_CONFIG`, `TEMPER_DATA`, `TEMPER_CACHE`, `TEMPER_JOB`; `TEMPER_BUNDLE`, `TEMPER_COMMIT`, `TEMPER_CLIENT_COMMIT`, `TEMPER_DUMP_LLM`, `TEMPER_FAKE_BOX_PORTS`, `TEMPER_FORGE_NO_REGISTRATION`, `TEMPER_R8_CUT_AT_LISTEN`, `TEMPER_R8_WAIT_CHECK`, `TEMPER_WORKSPACE_REUSE`; `TEMPER_ECHO`, `TEMPER_SEED`, `TEMPER_READY`, `TEMPER_OBSERVATIONS`, `TEMPER_DOCUMENT` |
| `cli C3` | `temper-client temper-client` | | a read in a `temper-client*` crate the `temper-client` binary links and neither C2's binary does | |
| `cli C4` | | | every read in a test: an integration test, a `src/tests` module or `#[cfg(test)]` item, `temper/tests/**`, `temper/testing/**` | every other `TEMPER_*` token: `TEMPER_E4_*`, `TEMPER_LIVE_*`, `TEMPER_MANAGED_*`, `TEMPER_BOX_PARENT_TEST`, `TEMPER_FORGE_PARENT_HELPER`, `TEMPER_FORGEJO`, `TEMPER_FORGEJO_RUNNER`, `TEMPER_RATE`, `TEMPER_RECORD_RELEASE`, `TEMPER_SITTING_WAIT_SECS`, `TEMPER_WRAPPER`, `TEMPER_TEST_BUNDLE`, `TEMPER_TEST_PLAYGROUND`, `TEMPER_TEST_CLAUDE`, `TEMPER_BENIGN` |
| `cli C5` | | every temper finding: the five crates' `[features]`, their `optional` dependencies, the `features` and `default-features` that name them, and temper manifests' external crates' `features` and `default-features` | | |
| `cli C6` | `temper-agent temper-agent` | | a read in `temper-agent` | |
| `cli C8` | every other temper target: the tools under `temper/tools/` and the fixtures under `temper/tests/` | | a read in a tool under `temper/tools/` or a fixture binary | |

A build script's read takes the `build script` prefix, whatever it
reads. A `TEMPER_*` token found on the trunk that this map doesn't name
goes to `cli C4` if only tests, tools or CI read or set it, else `cli
C2`; the task's report names it.

The other trees, whole, are their rounds': smith's (checks 1, 2, 3 and
5; check 4 holds no smith crate), skein's, jig's and hearth's.

## 5. The environment documents (`skein/docs/design/shell.md`, 15.3)

What `render-docs` writes and `checks-names` reads: in
`<tree>/docs/design/cli/`, a file whose name ends `-env.md` holds
variables, each an entry like:

```markdown
### `DEMO_INTERNAL_DUMP`

- **Set:** internal
- **Reads**
- **Form:** 1
- **Absent:** nothing is dumped.
- Dump each model request and reply to standard error.
- **Notes:** For a developer.
```

A line `` ### `NAME` `` starts an entry; the first line after it that
starts `- **Set:** ` gives its set, `public`, `internal` or `test`.
Nothing else of the document is the check's.
