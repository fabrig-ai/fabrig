+++
id = "table-types"
part = "shell-table"
kind = "interface"
needs = []
files = ["skein/crates/skein-shell/src/table.rs", "skein/crates/skein-shell/src/table/**", "skein/crates/skein-shell/src/settings.rs", "skein/crates/skein-shell/src/settings/tests.rs", "skein/crates/skein-shell/src/lib.rs", "skein/docs/design/shell.status.md"]
effort = "high"
minutes = 100
+++

## Goal

`skein-shell` gets a binary's table: its types and sets, the host's
variables it reads itself, and a lookup that names a declared variable
(`skein/docs/design/shell.md`, 15.1). A setting gets its set, its notes
and a declared twin in place of the one derived from the tool's name,
and a group of settings its own line (6.5). Readers keep their
behaviour: every twin declared is the one derived today. Rendering is
the render tasks'; this task adds the surface they build on.

Acceptance:

- `table.rs` holds `notes/contracts.md`, 1.1's types, field for field,
  `Variable::value`, `HOME` and `Base::variable`; `help.rs` and
  `reference.rs` exist with their module docs alone; `demo.rs` holds 1.3's
  demo tables. `lib.rs` exports 1.1's names.
- `Setting` has `set`, `twin` and `notes`, `Settings` has `about`
  (contracts, 1.2). `resolve` reads a setting's declared twin, and no
  variable when it has none; `help` and `Refusal`'s words name the twin
  only where there is one. Nothing derives a variable's name any more.
- skein-shell's settings tests hold the rule with made-up names (`tool:
  "demo"`, twins `DEMO_*`): no `TEMPER_*` token is left in
  `skein/crates/skein-shell/` (`rg 'TEMPER_' skein/crates/skein-shell`
  prints nothing). A new row holds a setting with no twin: its variable
  set in the environment is ignored, and its refusal names no variable.
- `Variable::value` is held by a table test: set, empty (unset), absent,
  and a name that differs only in case.
- temper's five `Settings` constants declare what was derived: the
  directories (`temper-shell`'s `directories.rs`) public, twins
  `TEMPER_CONFIG`, `TEMPER_DATA`, `TEMPER_CACHE`; `box_command.rs`'s
  `ECHO_SETTINGS` public, twin `TEMPER_ECHO`; `fake.rs`'s three tables
  internal, each setting's twin `TEMPER_` and its name in capitals
  (`TEMPER_DOCUMENT`, `TEMPER_READY`, `TEMPER_SEED`, `TEMPER_OBSERVATIONS`,
  in all three where each has the setting). Their tests pass unchanged.
- The **Not yet** entry 6.5 in `skein/docs/design/shell.status.md` (the
  two lines from `6.5: a setting derives its twin`) is removed, those
  lines alone. This departs from the implementer's text, which closes
  `docs/design/` to you: the entry is this task's. Leave the entry 15
  standing: the render tasks build the rest of it.
- `heavy cargo clippy -p skein-shell --all-targets -- -D warnings` and
  `heavy cargo clippy --target wasm32-unknown-unknown -p skein-shell --
  -D warnings` pass; skein-shell's focused tests, and `temper-shell`'s
  and `temper`'s that read settings, pass.

## Inputs

- `skein/docs/design/shell.md`, 6.5 and 15.1 (the rule), 14 (the
  portable face: the table is portable).
- `skein/docs/foundation/cli.md`, 4 to 6: sets, names, the table.
- `temper/docs/plan/ui-cli-c1/notes/contracts.md`, 1 (binding), and
  `notes/common.md`.

## Owns and may change

- Owns the files in `files`.
- Reader files, the new fields alone (`set`, `twin`, `notes`, `about`)
  and their imports: `temper/crates/temper-shell/src/directories.rs`,
  `temper/crates/temper/src/box_command.rs`,
  `temper/crates/temper/src/fake.rs`. `about` for the directories is
  `The deployment's three directories.`; for the echo `How the box shows
  its steps.`; for each of `fake.rs`'s, one line saying which fake it
  configures. Notes stay empty.
- If a temper test or golden prints `help`'s output (a command's
  `--help`, `docs/help/reference/commands.md`) and changes, the twins
  you declared differ from the derived ones: fix the twin, never the
  golden.
- **Not yours,** and left exactly as they are: in `settings.rs`, the
  lines from `/// The process's arguments after its program name` down
  to `environment`'s closing brace (`env-ban` adds expectations there in
  parallel); `listeners.rs`, `listener_tests.rs`, `readiness.rs`; any
  `clippy.toml`. Add no call to `std::env`: a lookup takes the captured
  pairs.
- `Base`'s private `variable(self) -> &'static str` becomes the public
  `variable(self) -> &'static Variable` (contracts, 1.1); its callers in
  `settings.rs` read `.name` or call `value`.

## Steps

1. **`table.rs`,** portable, with the module doc
   `//! A binary's table (shell.md, 15; skein/docs/foundation/cli.md, 6).`
   and contracts 1.1's types. `Variable::value` walks the pairs in their
   order and answers the first whose key is the name's bytes, `None` when
   it's empty:

   ```rust
   impl Variable {
       #[must_use]
       pub fn value<'a>(&self, environment: &'a [(Box<[u8]>, Box<[u8]>)]) -> Option<&'a [u8]> { .. }
   }
   ```

   `settings.rs`'s private `lookup` then serves only the twin's name;
   make it one function both use if that's simpler.
2. **`help.rs` and `reference.rs`:** `//! A table's help, command pages
   and version line (shell.md, 15.2).` and `//! A table's reference
   documents (shell.md, 15.3).`, nothing else. `table.rs` declares them,
   and `#[cfg(test)] mod demo;`.
3. **`demo.rs`:** contracts 1.3, `pub(crate)` constants. Its lines
   follow `skein/docs/foundation/cli.md`, 7's example. Give each entry
   a line and, where one makes sense, notes, so the render tasks see
   every field filled somewhere.
4. **Settings:** contracts 1.2. In `resolve` and `fallback`, the twin
   is `setting.twin`; `variable(tool, name)` goes. `Refusal`'s display:
   `--NAME VALUE is not given, and TWIN is not set` with a twin,
   `--NAME VALUE is not given` without; the same for `NoBase`'s words.
   `help`'s line: `  --NAME VALUE (TWIN; DEFAULT)` with a twin, `  --NAME
   VALUE (DEFAULT)` without, then the about line as now.
5. **The settings tests** (`settings/tests.rs`): `tool: "demo"`, twins
   `DEMO_CONFIG` and so on, every expected text with them. One new row
   per rule above (no twin: ignored variable, refusal's words, `help`'s
   line), in the existing table tests where one fits.
6. **`Variable::value`'s table test,** in `table.rs`'s tests or beside
   the demo: the four rows of the acceptance.
7. **Readers:** the three temper files, the new fields only.
8. `cargo fmt`; `heavy cargo clippy -p skein-shell -p temper-shell -p
   temper --all-targets -- -D warnings`; the wasm32 clippy above;
   `heavy cargo nextest run -p skein-shell`; `heavy cargo nextest run -p
   temper-shell`; `heavy cargo nextest run -p temper --lib`. Break one
   rule by hand (derive the twin again, or read a twin-less setting's
   variable) and see its row fail; restore. Commit, push.

## Report

What you built, the twins declared in `fake.rs` (each with the derived
name it replaces), the tests and their counts, the rule you broke and
the failure seen, and anything under `## Left`.
