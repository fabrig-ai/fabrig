# client-binary: what every task keeps

Every task of this plan reads this note, `notes/contracts.md` at the sections
its spec names, fabrig's `AGENTS.md` and `temper/AGENTS.md`.

## The plan in a line

The deployment's client, `temper-client`, a package and binary of its own, runs
the TUI and `ctl` by the same code the box's `temper` runs; that code moves out
of the `temper` library into crates both binaries link: `temper-shell` (what
every binary of temper's starts from) and the client's shells
(`temper/docs/design/protocol/README.md`, sections 4 and 5;
`temper/docs/design/client/README.md`, section 2 and 6.11).

## Rules

- **A move is a move.** Code moved between crates keeps its body, its words
  and its tests' assertions; what changes is its path, its visibility, and the
  binary's name where a line names the binary (`name: &str`, `"temper"` from the
  box). **Every line the box's `temper` prints stays byte for byte**, its
  `--help`, each `COMMAND --help`, its refusals and
  `temper/docs/help/reference/commands.md` among them: its existing tests prove
  it, and none of their expected strings changes.
- **The box's readers don't move.** Where a moved item is read outside the
  module it leaves (another module of `temper`, `temper`'s tests, another
  package's tests such as `temper/tests/real-loop` or
  `temper/crates/temper/tests/**`), `temper` keeps the name they read, as a
  `pub use` (or `pub(crate) use`) or a one-line wrapper naming `"temper"`, so
  that the readers compile unchanged. `temper/src/lib.rs`' `run` dispatch is
  left as it is.
- **A moved test moves with what it tests** when it reads only moved items;
  a test that also reads what stays in `temper` (`Directories`,
  `configure_with`, `Command`, `COMMANDS`) stays in `temper` and reads the
  moved items by their new path or by `temper`'s kept name. Moved tests keep
  their names and assertions; no bite is needed for a move.
- **A new crate:** `#![forbid(unsafe_code)]` and a module doc citing its
  design section at its root; `python3 -I -B tools/lints/lints.py --set binary
  <its Cargo.toml>`; a `clippy.toml` copied byte for byte from
  `temper/crates/temper-client-shell-cli/clippy.toml` (the `ordinary`
  variant); the root `Cargo.toml` gains its member line beside its neighbours
  under `temper/crates/` and, for a library, its `[workspace.dependencies]`
  line beside them; `python3 -I -B tools/lints/lints.py` reports 0 findings.
  A fixture file under a `tests/fixtures/` directory needs its entry in
  `.config/fixture-readers.toml` (none is expected here).
- **Only the focused and fuzzy suites run.** A process-tier story is written
  and compiled (`heavy cargo nextest run -p temper --test process --no-run`),
  never run; nothing runs `tools/tiers/run`, a `<tier>-<tree>` profile, a
  browser, a walk, a render check or a throwaway driver.
- **Tests stay cheap:** a new case of one behaviour is a row of an existing
  table test. Budgets and timeouts are the user's.
- **Decide, don't ask.** A public interface may change freely before v0.1.0;
  a small edit a task needs just outside its list is made and named in the
  report. A `Question:` is only for what truly blocks the task.
- **Generated files:** `temper/docs/help/reference/commands.md` is rendered
  from `temper`'s table; a task whose change makes it stale regenerates it
  (`SKEIN_CODEGEN_WRITE=1 heavy cargo nextest run -p temper --lib -E
  'test(=tests::commands::the_page_commands_is_what_the_table_renders)'`). This
  plan expects it unchanged: a diff there is a red.
- **A failure "outside the change"** is first run once on the pristine base: if
  the base passes and the branch fails, it is the branch's.
- **Old forms:** `python3 -I -B temper/scripts/old_forms.py` after the change; a
  count that rises because code moved between temper's crates is lowered back
  by the moved form, never by `--write` raising it.
- **Status files:** a task edits `docs/design/` only where its spec says so,
  and then only the lines it names.

## Not yours, whatever the task

- `temper-service` and every engine, worker or smith crate (M2 splits the
  service beside this plan).
- The box's own files that the in-flight plan `tui-box-front` changes:
  `temper/crates/temper/src/box_command.rs`, `fake_backends.rs`, `tui_front.rs`
  (if present), `temper/crates/temper/tests/suite/box_kinds.rs`,
  `box_front.rs`, `temper/crates/temper/tests/suite/main.rs`; `temper-box`;
  `temper-client-tui`; and in `temper-client-shell-tui`, `host.rs`, `run.rs`,
  `limits.rs`, `tests.rs` and `journey/**`. A task that must change one of
  them for a compile says so in its report.
- `temper/docs/help/*.md` (the help pages; M5 names each command's binary).
- `.github/workflows/**` and any release script (M6).

## When `tui-box-front` has landed on your base

That plan adds `temper/crates/temper/src/tui_front.rs` (`temper internal tui
--box`, the TUI in front of the box) reading `crate::tui`'s helpers. If it is on
your base when your task moves what it reads, keep `temper`'s `tui` names it
reads (`pub(crate) use` of the moved items), and say so in the report;
`tui-start`'s spec says more.
