# ui-cli-c1: the plan

Milestone C1 of `temper/docs/design/draft/cli.execution.md`: the command
line's conventions (`cli.md`, 3) and their mechanism (8.1 to 8.3). Low
priority. C1 adds and moves nothing: no binary moves, nothing is
renamed, no feature goes. What temper and the other trees don't follow
yet is an exception or a scoped `expect` naming the milestone or round
that removes it.

Design written by this cut:

- `skein/docs/foundation/cli.md`, new: the conventions, from the draft's
  section 3, renumbered (2 to 9), with section 10, Later.
- `skein/docs/design/shell.md`: 6.5 revised (a setting's set, its
  declared twin, a group's line), section 15 new (a binary's table: 15.1
  entries and sets, 15.2 help, 15.3 the reference documents), sections 1
  and 14 a line each. Its status: **Not yet** 6.5 and 15.
- `tools/lints/design.md`, 4: the environment's functions in every
  variant. Its status: **Not yet** 4.
- `skein/docs/design/STATUS.md`: the foundation paragraph names
  `cli.md`'s exceptions and where its gaps are kept.
- The draft's status, `cli.status.md`: the lag "3: no page" closed; two
  **Design departs** (below). `cli.execution.md`: C1 in flight.

## Decided

- **The page is the cut's,** written whole with the first push, so every
  task cites its sections from the trunk.
- **Check 3 holds section 3's whole rule:** no `[features]`, and no
  `optional`, `default-features` or `features` on any member's
  dependency, external crates' included, nor on a workspace crate's
  entry in the root. An external crate's features are chosen in the
  root alone. The draft's "a workspace dependency" is read as a
  member's dependency: the reading that checks 3.2 entire.
- **Check 4 bans `skein_shell::arguments` and `environment` too,**
  `allow-invalid` for crates that don't link the shell: they read the
  process's for any caller, and temper calls `environment()` deep in
  seven places. A **Design departs** entry in `cli.status.md`.
- **The help example wraps at 80 columns,** as the draft's own rule
  asks, where its example runs past. A **Design departs** entry.
- **Exceptions only shrink, by code:** an exception that matches nothing
  fails its check (`foundation/cli.md`, 9), so the plan that brings a
  thing into line removes its exception.
- **An `expect`'s reason starts with what the read is:** `capture`,
  `build script`, `cli C2` to `cli C8`, or `skein's round`, `jig's
  round`, `hearth's round` (`notes/contracts.md`, 2). temper's existing
  `std::env::var` expectations get the prefix; the other trees' stay as
  they are.
- **No binary gets a table in C1,** and dispatch isn't skein-shell's yet:
  C2 builds the first tables and their checks 6 and 7, on the types and
  rendering this plan lands. The table names a delegated grammar's row
  (`Reach::RunAs`) so C2 needs no new type for "run as `git`".
- **Checks 6 and 7 are each binary's,** in its crate's tests (the page,
  9): with no table in C1, there's nothing for them to check yet.
- **Made-up names in skein-shell's tests** (`DEMO_*`), so check 5 needs
  no exception for them.

## Interface changes in `skein-shell`, for hearth/core

- `Setting` gains `set: Set`, `twin: Option<&'static str>` and `notes`;
  `Settings` gains `about`. A setting's variable is its declared twin,
  no longer `<TOOL>_<NAME>` derived; one with no twin reads none.
  `resolve`, `help`, `Refusal` keep their signatures; their words name
  the twin only where there is one. Readers today: temper's five tables
  alone (`temper-shell`'s directories, `temper`'s echo and three fakes).
- New, portable: `Set`, `Table`, `Reach`, `Command`, `Argument`,
  `Switch`, `Variable` (with `value`, a lookup by entry), `Way`,
  `TestTable`, `HOME`; `Base::variable` public, answering a `Variable`.
  Then `Table`'s help, command page and version line, and the reference
  documents (`shell.md`, 15.2 and 15.3).
- Clippy: every variant bans `std::env`'s eight functions and
  `skein_shell::arguments` and `environment`; hearth's reads carry
  `hearth's round` expectations (`hearth/crates/hearth/src/main.rs`
  among them).

## Exceptions each later milestone removes

By `notes/contracts.md`, 4; each check's table holds the exact list.

- **C2, temper/ui:** binaries `temper` and `temper-engine` (checks 1,
  2); reads in what they link; `TEMPER_CONFIG`, `_DATA`, `_CACHE`,
  `_JOB`, the internal renames of the draft's 4.6 and the five dropped
  twins (check 5).
- **C3, temper/tui:** binary `temper-client`; reads in the client's own
  crates.
- **C4, temper/tiers:** every read in a test; the test set's names and
  `TEMPER_BENIGN`, `TEMPER_TEST_CLAUDE`.
- **C5, temper/migration and temper/tui:** every temper feature, its
  optional dependencies and their `features`, and temper manifests'
  external crates' `features`.
- **C6, temper/migration:** binary `temper-agent`; its reads.
- **C7, ops:** none: the site holds nothing these checks read.
- **C8, temper/ui:** the tools under `temper/tools/` and the fixture
  binaries under `temper/tests/`; their reads.

## Parts and tasks

| Part | Task | Crates (files) | Needs | Minutes |
|---|---|---|---|---|
| `shell-table` | `table-types` | `skein-shell` (`table.rs`, `table/`, `settings.rs`, `lib.rs`); temper's five `Settings` | | 100 |
| `checks` | `checks-layout` | `fabrig-boundary` (`tests/cli.rs`); the root's `syn` | | 120 |
| `env-access` | `env-ban` | `tools/lints/clippy/`, every `clippy.toml` copy; skein's reads | | 90 |

Released with the first push. Later batches: `render-help` and
`render-docs` (`shell-table`), `checks-names` (`checks`), and
`env-jig-hearth`, `env-temper-src`, `env-temper-tests` (`env-access`).

## Critical path

`table-types` → `render-docs` → `checks-names`, about five hours;
`env-ban` → `env-temper-src` beside it.

## Overlaps

- `skein-shell`'s `settings.rs`: `table-types` and `env-ban` run
  together; `env-ban` adds lines only above `arguments` and
  `environment`, which `table-types` leaves as they are.
- Every `clippy.toml` (`env-ban`, by `lints.py --write`): no draft
  beside this one changes a variant.
- The env sweeps touch many files across the trees with attribute lines
  alone; `merge-job-runs-2` works in temper's worker and engine crates,
  where a read there gains one line above its item.
- Once `env-ban` merges, jig's, hearth's and temper's clippy fails on
  this plan's trunk until their sweep tasks merge.
