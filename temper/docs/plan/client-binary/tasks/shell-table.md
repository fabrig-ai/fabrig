+++
id = "shell-table"
part = "table"
kind = "interface"
needs = ["shell-local"]
files = [
  "temper/crates/temper-shell/**",
  "temper/crates/temper/src/commands.rs",
  "temper/crates/temper/src/help.rs",
  "Cargo.lock",
]
effort = "high"
minutes = 90
+++

# `temper-shell`: a binary's table, the shared rows, and `help`'s answers

## Goal

Move the table's types and how a binary's table prints (`--help`, `COMMAND
--help`, the usage and the refusals), the three rows more than one binary holds
(`help`, `[ADDRESS]`, `ctl`) and `help`'s answers out of the `temper` library into
`temper-shell`, each printing for the binary's name it is given
(`temper/docs/design/protocol/README.md`, section 4, *Every command is a row* and
*The deployment's client*; section 5, the row `crates/temper-shell`). The box's
`temper` builds its table on them and prints every byte as today; the client's
table (`client-table`) and later binaries' build on the same.

Acceptance:

1. **`temper_shell::commands`, `rows` and `help`** as `notes/contracts.md`,
   sections 4 and 5: types and `help`'s `Request` and `request` moved verbatim
   from `temper/crates/temper/src/commands.rs` and `help.rs`; each function
   printing a line about the binary takes `name: &str` where today's names
   `temper` (`command`, `forms`, `refused`, `unknown`, `help`, `row`, and
   `help::answer`'s refusals); `find` and `unknown` take the table. The rows
   `HELP`, `TUI` and `CTL` are today's rows, word for word. `temper-shell` gains
   `temper-docs`; module docs cite protocol/README.md, section 4, and
   documentation.md, section 4.
2. **`temper` builds on them:** `commands.rs` as contracts, section 4, says (the
   types re-exported; `DIRECTORIES` and `FORGE` its own; `COMMANDS` with the three
   shared rows in their places, its length unchanged; `find`, `unknown`, `forms`,
   `refused`, `help`, `row` one-line wrappers with today's signatures and
   visibility; `reference()` its own, on `temper_shell::commands::command`);
   `help.rs` as section 5 says. No other file of `temper` changes, and no other
   package.
3. **Byte for byte:** `temper`'s tests pass unchanged, among them
   `tests::commands::every_row_parses_from_an_example`, `each_row_answers_its_help`,
   `the_page_commands_is_what_the_table_renders` (with no regeneration:
   `temper/docs/help/reference/commands.md` doesn't change),
   `tests::tui::bare_temper_and_temper_address_parse_and_refuse` and
   `tests::help::…`.
4. **A table prints for its binary's name,** each test seen failing first
   (`temper/crates/temper-shell/src/commands/tests.rs`, `help/tests.rs`):
   - `a_table_prints_for_its_binarys_name`, a table over `[rows::HELP, rows::TUI,
     rows::CTL]` and the name `temper-client`: `help`'s first two lines
     (`usage: temper-client COMMAND [ARGUMENT...]`, `temper-client help prints the
     index of temper's pages; temper-client COMMAND --help prints one command.`);
     `row(TUI)` starting `usage: temper-client [ADDRESS]\n`; `unknown` for no word
     (`temper-client: no command; temper-client --help lists them\n`) and for
     `box serve` (`temper-client: no command box; …`); `refused` for `TUI` and
     `invalid arguments`; `find` giving `TUI` for no word and for
     `http://127.0.0.1:8367`, `CTL` for `ctl DIR me`, none for `box`.
   - `help_names_its_binary_in_its_refusals`: `answer("temper-client", …)` for a
     page that isn't (`temper-client: no page nope; temper-client help lists
     them\n`) and a search that matches nothing.

## Inputs

- `temper/docs/plan/client-binary/notes/common.md`; `notes/contracts.md`,
  sections 1, 4 and 5.
- Design: `temper/docs/design/protocol/README.md`, section 4 (*Every command is a
  row of the binary's table*, *The deployment's client*) and section 5;
  `temper/docs/design/protocol/documentation.md`, section 4.
- The code: `temper/crates/temper/src/commands.rs` (all of it: `Who`, `Row`,
  `Argument`, `COMMANDS`, `find`, `begins`, `unknown`, `command`, `forms`,
  `refused`, `help`, `row`, `reference`); `help.rs`; their readers in `temper`:
  `lib.rs` (`commands::find`, `unknown`, `refused`, `row`, `help::request`,
  `answer`), `main.rs` (`commands::help()`), `tui.rs` (`commands::unknown`),
  `src/tests/commands.rs`, `src/tests/help.rs`, `src/tests/tui.rs`; the
  `temper-shell` crate as `shell-local` left it.

## Owns and may change

Owns `temper-shell`'s new modules, its `lib.rs` lines and its `Cargo.toml`'s
`temper-docs` line; `temper`'s `commands.rs` and `help.rs`. In `commands.rs`,
leave every row but the three shared ones as it is: the in-flight `tui-box-front`
adds a row (`internal tui`) and changes `box`'s; a merge there is theirs or the
catch-up's.

Not yours: `lib.rs`, `main.rs`, `tui.rs` of `temper` (they read the wrappers
unchanged), `temper/docs/help/**`.

Options, each decided:

- **`find`'s `'static`:** `temper`'s wrapper passes `&COMMANDS`, promoted as
  today's loop is; if the compiler refuses it, make `COMMANDS` a `static` and
  say so.
- **`begins`** stays private to `temper_shell::commands`.
- **A row's settings** (`&directories::SETTINGS` and the box's flag tables) stay
  in `temper`'s rows; the three shared rows hold none.

## Steps

1. The failing tests of acceptance 4 against the new signatures, then the move.
   `heavy cargo check -p temper-shell -p temper --tests` until it builds.
2. `heavy cargo nextest run -p temper-shell`; `heavy cargo nextest run -p temper
   --lib -E 'test(/tests::(commands|help|tui)::/)'`.
3. Once: `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-shell -p temper --all-targets -- -D warnings`; `heavy cargo
   nextest run -p temper-shell -p temper`; `heavy cargo check -p
   temper-real-loop-world --tests`; `git diff --exit-code
   temper/docs/help/reference/commands.md`.
4. Commit ("temper-shell: a binary's table and help, printed for its name; the
   box's table on them") and push.

## Report

`Done:`, `Failed:` or `Question:` first, then the files, the runs and their
counts, the rule each new test bit on, anything left.
