+++
id = "shell-crate"
part = "shared-shell"
kind = "migration"
needs = []
files = ["temper/crates/temper-shell/**", "temper/crates/temper/src/commands.rs", "temper/crates/temper/src/directories.rs", "temper/crates/temper/src/directories/**", "temper/crates/temper/src/help.rs", "temper/crates/temper/src/link_file.rs", "temper/crates/temper/src/tests/link_file.rs", "temper/crates/temper/src/tests/program_environment.rs", "temper/crates/temper/src/tests.rs", "temper/crates/temper/Cargo.toml", "temper/docs/design/protocol/README.status.md"]
effort = "high"
minutes = 90
+++

## Goal

The parts every binary of temper shares move out of the box's `temper`
library into a new crate, `temper-shell`: the commands' table and its
renderings, now under any binary's name; the directories; `help`; and
the engine's link file (`temper/docs/design/protocol/README.md`, 4.1 and
section 5). `temper-engine` will render its own table with them; the box's
`temper` keeps every path, word and test it has.

Acceptance:

- `temper/crates/temper-shell`: ordinary Rust on the `binary` lint
  profile with the `ordinary` `clippy.toml` copy (as
  `temper-forge-setup-shell` has them), on `skein-shell`, `skein-io`, `skein-lib`,
  `jig-protocol-config` and `temper-docs` alone; its crate doc cites
  `protocol/README.md`, 4.1. Its modules and names are
  `notes/contracts.md`, 4's: `commands` (`Who`, `Row`, `Argument`,
  `Table` and its renderings, `DIRECTORIES`, `HELP`), `directories`,
  `help` and `link_file`.
- **The renderings take the binary's name.** `Table`'s methods render
  what `temper/src/commands.rs`'s functions of the same names render
  today, with `self.binary` wherever they name the binary: the usage
  line, a command's words (`temper forge run`), `temper --help`,
  "the temper binary", "Spawned by temper", and `unknown`'s
  `temper: no command …; temper --help lists them`. "temper's pages" and
  the reference page's title name the product, not the binary, and stay
  as they are. With `binary` `"temper"` and `temper`'s rows, every
  rendering is byte for byte today's: the box's own tests, among them
  `the_page_commands_is_what_the_table_renders`, pass unchanged, and
  `temper/docs/help/reference/commands.md` doesn't change.
- **One new test in `temper-shell`** (`src/tests.rs` or a module of it):
  a table of `(binary, words) → unknown(...)` and `(binary, row) →
  forms(...)` rows over a two-row table of its own, the binary
  `temper-engine` among them, so that the name is held. Break the name
  once (hard-code `temper`), see its rows fail, restore.
- **`temper` keeps every path** (`notes/contracts.md`, 4): `temper::directories`
  is a module re-exporting `temper_shell::directories::*` and holding
  `program_environment` as a free function (its two callers, `lib.rs`'s
  `prepare_with_limits` and `src/tests/program_environment.rs`, call it
  so); `temper::help` and `temper::link_file` are `pub use` of the
  shell's; `temper::commands` keeps `COMMANDS` and every row, takes
  `Who`, `Row`, `Argument`, `DIRECTORIES` and `HELP` from the shell (its
  first row is `HELP`), holds `pub const TABLE`, and keeps `find`,
  `unknown`, `forms`, `refused`, `help`, `row` and `reference` as one-line
  wrappers over it. No file outside `temper/src` and the new crate
  changes for this.
- **The tests move with their code:** `directories/tests.rs` and
  `src/tests/link_file.rs` move into `temper-shell`, unchanged but for
  paths; a case among them that reads `program_environment` or `printed`
  stays in `temper`'s tests. `src/tests/commands.rs` and
  `src/tests/help.rs` stay: they read the box's table, its parser, its
  host's lines and `ctl`, and hold the pages against them.
- The status entry closes: in `temper/docs/design/protocol/README.status.md`,
  the **Not yet** 4.1, 5 entry "`temper-shell`: the commands' table and its
  renderings, …", its lines alone. This departs from the implementer's
  text, which closes `docs/design/` to you: the entry is this task's.

## Inputs

- `temper/docs/design/protocol/README.md`, 4.1 (its table, the rows two
  tables share, the shared crates) and section 5, the row
  `crates/temper-shell`.
- `temper/docs/plan/ui-engine-split/notes/contracts.md`, 4.
- `tools/lints/design.md`, sections 3 and 4.
- `temper/docs/plan/ui-engine-split/notes/common.md`.

## Owns and may change

- Owns the files in `files`; in `temper/src/lib.rs`, the `mod` and `use`
  lines of `directories`, `help`, `link_file` and `commands` alone, and
  the one call of `program_environment` in `prepare_with_limits`.
- The root `Cargo.toml`: the member line `"temper/crates/temper-shell",`
  right after `"temper/crates/temper-init",`, and the dependency line
  `temper-shell = { path = "temper/crates/temper-shell" }` right after
  `temper-client-view = { … }`; nothing else there. `Cargo.lock` as cargo
  writes it.
- Readers: a file under `temper/crates/temper/` whose `crate::…` path to
  a moved item no longer resolves (an item that was `pub(crate)` and is
  now the shell's `pub`): that path alone, named in the report.
- Pre-approved: an item the moved code calls in `temper`'s `lib.rs`
  (`path`, `errno`, and the like): if it is under ten lines, the shell
  takes a private copy; otherwise it moves into the shell and `lib.rs`
  calls it there. Say which in the report.
- Overlaps: the plan `tui-box-front` is changing `temper/src/lib.rs`,
  `commands.rs` and `src/tests/commands.rs`, and `client-binary` will
  move `link_file`. Keep `commands.rs`'s rows as they stand, edit only
  its types, its `DIRECTORIES` const, its first row and its functions;
  touch `lib.rs` only as above.
- Not yours: `printed.rs` and `Directories::program_environment`'s body
  (it moves as a function, unchanged); the forge's commands, serve and
  the host (tasks `engine-forge` and `engine-serve`).

## Steps

1. The manifest, `python3 -I -B tools/lints/lints.py --set binary
   temper/crates/temper-shell`, the `ordinary` `clippy.toml` copy.
2. `temper-shell`'s new test against a stub `Table`; see it fail; then
   the moves and the renderings. `heavy cargo check -p temper-shell -p
   temper --tests` while errors last.
3. `heavy cargo nextest run -p temper-shell`; `heavy cargo nextest run -p
   temper --lib -E 'test(/commands|help|program_environment/)'`; read the
   counts. Break the name and restore as above.
4. The status entry.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `python3 -I
   -B tools/lints/lints.py`; `heavy cargo clippy -p temper-shell -p temper
   --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-shell
   -p temper`; `heavy cargo check -p temper-real-loop-world -p
   temper-playground -p temper-client-world --tests`.
6. `python3 -I -B tools/citations/citations.py --base origin/main`.
   Commit and push.

## Report

As the implementer's text says, with the helpers copied or moved and any
test that stayed in `temper`.

## Answers

2026-11-08. *May `temper-shell` depend on `skein-io` for `errno`, or must
skein-shell expose the conversion first?* Depend on `skein-io` directly:
acceptance, `notes/contracts.md`, 4 and `protocol/README.md`, section 5's
row now list it. It adds no crate to the graph (`skein-shell` is on it
already), and `errno`'s numbers are Linux's, a diagnostic of temper's, not
a generic mechanism for skein. Move `errno` (30 lines) and `unread`, which
calls it, into `temper-shell`'s `directories` together: `unread` as `pub fn
unread(why: skein_shell::Unread) -> String`, `errno` private beside it,
both unchanged in body. `lib.rs` drops its two and takes `unread` with one
`use temper_shell::directories::unread;` line, so every `super::unread` in
`forge.rs`, `open.rs`, `fake.rs` and `directories.rs` resolves unchanged;
that line and the removal are pre-approved under "it moves into the shell".
`Error` then leaves `lib.rs`'s `use skein_io::kernel::{…}` line, also
pre-approved. Say so in the report. Continue from `2a2bfe657d`.
