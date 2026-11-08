+++
id = "shell-local"
part = "local"
kind = "interface"
needs = []
files = [
  "temper/crates/temper-shell/**",
  "temper/crates/temper/src/directories.rs",
  "temper/crates/temper/src/directories/tests.rs",
  "temper/crates/temper/src/link_file.rs",
  "temper/crates/temper/src/lib.rs",
  "temper/crates/temper/src/tests.rs",
  "temper/crates/temper/src/tests/link_file.rs",
  "temper/crates/temper/Cargo.toml",
  "Cargo.toml",
  "Cargo.lock",
]
effort = "high"
minutes = 75
+++

# `temper-shell`: the three directories and the engine's link file

## Goal

Create `temper-shell`, the crate every binary of temper's starts from, with its
first two modules, moved out of the `temper` library: `directories` (the three
directories' table, their resolution, the private directories) and `link_file`
(the engine's link at rest, which the engine writes and the TUI reads). The box's
`temper` reads them from there and prints and does exactly what it does today.
The TUI's start (task `tui-start`) and the deployment's client build on this.

Acceptance:

1. **The crate** `temper/crates/temper-shell` as `notes/contracts.md`, section 1,
   with `pub mod directories;` and `pub mod link_file;`, each as sections 2 and 3
   give them, bodies moved verbatim from `temper/crates/temper/src/directories.rs`
   and `link_file.rs`. Its root doc: "What every binary of temper's starts from
   (temper/docs/design/protocol/README.md, section 5)."; each module keeps its
   design citation.
2. **`temper` reads them:** `temper/crates/temper/src/link_file.rs` is gone and
   `lib.rs` says `pub use temper_shell::link_file;` in place of `pub mod
   link_file;`; `directories.rs` keeps `Directories` with every method and the
   names section 2 lists. No other file of `temper`, and no other package, needs
   an edit to compile: `crate::link_file::…`, `temper::link_file::path`,
   `directories::{SETTINGS, resolve, Refusal, make_private, private_root}` read
   on as before.
3. **The tests move with what they test** (`notes/common.md`): `src/tests/link_file.rs`
   becomes `temper/crates/temper-shell/src/link_file/tests.rs` (declared
   `#[cfg(test)] mod tests;` in `link_file.rs`), its `deployment` row reading
   `crate::directories::resolve`'s `Paths`; the two settings tests of
   `temper/crates/temper/src/directories/tests.rs`
   (`settings_resolve_with_their_origins_and_keep_the_remaining_arguments`,
   `settings_refusals_name_the_flag_and_its_reason`) move to
   `temper-shell/src/directories/tests.rs` if they read only `resolve` and
   `Refusal`; every test reading `Directories` stays. Each moved test passes in
   `temper-shell` under its own name; `temper`'s remaining tests pass unchanged.
4. **Nothing else moves:** `Directories`' methods, `printed`, `init`, `serve`'s
   use of the link file and every line `temper` prints are as they were.

## Inputs

- `temper/docs/plan/client-binary/notes/common.md`, all of it;
  `notes/contracts.md`, sections 1 to 3.
- Design: `temper/docs/design/protocol/README.md`, section 5 (the row
  `crates/temper-shell`); `temper/docs/design/client/README.md`, 6.12 (*The
  engine's link, kept at rest*: one function, `temper-shell`'s
  `link_file::path`); `temper/docs/design/protocol/configuration.md`, section 12.
- The code: `temper/crates/temper/src/directories.rs` (`SETTINGS`, `Directories`,
  `Refusal`, `resolve`, `private_root`, `make_private`; its `tests` module),
  `link_file.rs`, `lib.rs` (`pub mod link_file;`, `use directories::{Directories,
  private_root};`, `serve`'s `link_file::path` and `forget`), `host.rs`
  (`crate::link_file::keep`), `tui.rs` (`link_file::deployment`, `path`, `read`;
  `directories::make_private`), `src/tests/link_file.rs`, `src/tests.rs`.
- Readers outside `temper/src` that must compile unchanged:
  `temper/crates/temper/tests/fake_backends/box_link.rs` (`temper::link_file::path`),
  `temper/tests/real-loop/src/service.rs` and `cluster.rs`,
  `temper/crates/temper/tests/support/configuration.rs`,
  `temper/crates/temper/tests/suite/startup.rs`, `temper/crates/temper/tests/process.rs`
  (`temper::directories::…`).

## Owns and may change

Owns `temper/crates/temper-shell/**` (new), `temper`'s `directories.rs` and its
`directories/tests.rs`, the removal of `link_file.rs` and `src/tests/link_file.rs`,
the `mod link_file;` line of `src/tests.rs`, and in `lib.rs` the one line `pub mod
link_file;`. `temper/crates/temper/Cargo.toml` gains `temper-shell.workspace =
true`; the root `Cargo.toml` gains the member and the workspace dependency
(`notes/common.md`, *A new crate*); `Cargo.lock` follows.

Not yours: `tui.rs`, `open.rs`, `ctl.rs`, `commands.rs`, `help.rs` (later tasks
move them); the rest of `lib.rs` (the in-flight `tui-box-front` changes it).

Options, each decided:

- **`Directories` from `Paths`:** `impl From<temper_shell::directories::Paths> for
  Directories`, fields by name; if a field differs, stop: the contract holds the
  fields as today's.
- **`skein-shell`'s names:** if `skein_shell::Origin` or `Refusal` are reached
  through another path today (`settings::`), keep the shell's module using the
  same `use` as `directories.rs` does now.
- **A moved test needing `temper`'s scratch helper** (`directories::tests::scratch`):
  use `skein_scratch::Scratch` in `temper-shell`, as `link_file`'s tests do; if
  that changes what the test checks, leave the test in `temper` and say so.

## Steps

1. Create the crate (`notes/common.md`, *A new crate*): `Cargo.toml` with
   `skein-lib`, `skein-shell`, dev `skein-scratch`; `clippy.toml`; `src/lib.rs`.
2. Move `directories`' and `link_file`'s items (contracts 2 and 3); in `temper`,
   `directories.rs`' re-exports and `resolve` wrapper, `lib.rs`' `pub use`.
   `heavy cargo check -p temper-shell -p temper --tests` until it builds.
3. Move the tests (acceptance 3). `heavy cargo nextest run -p temper-shell`, then
   `heavy cargo nextest run -p temper --lib -E 'test(/directories|tui/)'`.
4. Once: `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`;
   `python3 -I -B tools/lints/lints.py`; `heavy cargo clippy -p temper-shell -p
   temper --all-targets -- -D warnings`; `heavy cargo nextest run -p temper-shell
   -p temper`; `heavy cargo check -p temper-real-loop-world --tests`.
5. Commit ("temper-shell: the three directories and the engine's link file, out
   of temper") and push.

## Report

`Done:`, `Failed:` or `Question:` first, then: the crate's files; which tests
moved and their counts in each package before and after; the `cargo tree -p
temper-shell -e normal --prefix none` closure (skein's crates alone expected); the
runs and their counts; anything left.
