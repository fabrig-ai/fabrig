+++
id = "render-help"
part = "shell-table"
kind = "feature"
needs = ["table-types"]
files = ["skein/crates/skein-shell/src/table/help.rs", "skein/crates/skein-shell/src/table/help/**"]
effort = "high"
minutes = 90
+++

## Goal

A binary's table renders its help page, a command's page and its
version line, in the one format of `skein/docs/design/shell.md`, 15.2:
what `-h`, `--help`, `help`, `COMMAND --help` and `--version` print for
every binary of every tree (`skein/docs/foundation/cli.md`, 7). Pure
text from the table; no binary calls it yet (C2's).

Acceptance:

- `table/help.rs` holds, in an `impl Table` block:

  ```rust
  impl Table {
      /// The page `-h`, `--help` and `help` print (shell.md, 15.2).
      #[must_use]
      pub fn help(&self) -> String;
      /// What `COMMAND -h` and `--help` print for one of its commands.
      #[must_use]
      pub fn command_help(&self, command: &Command) -> String;
      /// `BINARY VERSION (COMMIT)` and a newline.
      #[must_use]
      pub fn version_line(&self) -> String;
  }
  ```

  each following 15.2 to the letter: its parts, their order, the
  two-column layout at columns 2 and 37, a first column of more than 33
  characters on its own row, wrapping at spaces to 80 columns with
  continuations at column 37 (two columns) or at the line's own indent
  (one column).
- The demo table's page (`table/demo.rs`'s `DEMO`) equals an expected
  text, written in the test by hand from 15.2's rules, never pasted from
  the output: it follows `skein/docs/foundation/cli.md`, 7's example with
  the demo's names. It lists exactly the public entries: no `forge`, no
  `git`, no `--front`, no `DEMO_INTERNAL_DUMP`.
- `command_help` for a public command with arguments, settings and a
  twin (`program-login` or the bare form), for the internal `forge`
  (its first line `internal: not part of demo-box's public command line;
  it may change without notice.`) and for `Reach::RunAs("git")` (usage
  `  git ARGUMENT...`) each equal an expected text.
- `version_line` gives `demo-box 0.1.0 (0123456789ab)\n`.
- A table test holds the layout rules alone, on made-up lines: a first
  column of 33 and 34 characters, a second column that wraps once and
  twice, a one-column line that wraps, and a word longer than the room,
  which stays whole on its own line. Every line of every expected text
  above is at most 80 columns, asserted.
- The **Not yet** entry 15 in `skein/docs/design/shell.status.md` stays:
  `checks-names` removes it once this task and `render-docs` have both
  merged.
- `heavy cargo clippy -p skein-shell --all-targets -- -D warnings` and its
  wasm32 clippy pass; skein-shell's focused tests pass.

## Inputs

- `skein/docs/design/shell.md`, 15.1 and 15.2 (the format; binding).
- `skein/docs/foundation/cli.md`, 4 (sets: help lists the public only)
  and 7 (help, and the example page).
- `temper/docs/plan/ui-cli-c1/notes/contracts.md`, 1 (the types and the
  demo table), and `notes/common.md`.
- `skein/crates/skein-shell/src/table.rs` and `table/demo.rs`, as
  `table-types` left them.

## Owns and may change

- Owns `table/help.rs` and its tests' module (`table/help/tests.rs`, if
  you put them there).
- `table/demo.rs` is `table-types`'s fixture, which `render-docs` reads
  in parallel: read it, never edit it. If a field the page needs is
  empty there, write your expected text from it as it is, and say so.
- A default's words are a private helper of `help.rs`, a `match` on
  `Fallback`: `Refuse`, required; `Text(text)`, the text;
  `Base(base)`, `$` and `base.variable().name`, `/` and the tool's name
  (`$XDG_CONFIG_HOME/demo`), as `settings.rs`'s `help` words them. Don't
  edit `settings.rs`: `render-docs` keeps its own copy.
- skein-shell is on the `ring` profile: no indexing, no unchecked
  arithmetic, no `as`; widths count `chars()`.
- Not yours: `table/reference.rs` (`render-docs`, in parallel), the
  types in `table.rs`, `lib.rs`.

## Steps

1. **A line-layout helper,** private to `help.rs`: one function for a
   two-column row and one for a wrapped one-column line, each pushing
   into a `String`. Test them first (the layout table test).
2. **`help`:** the six parts of 15.2, each left out when it holds
   nothing but the flags part, which always has `-h, --help` and
   `--version`. The environment part: a group's public twins on one
   row, `As --config, --data and --cache.` (`As --x.` for one, `--x and
   --y` for two); then each public variable the table or a public
   command reads, once.
3. **`command_help`:** 15.2's command page. A public command's page
   lists its public arguments, settings, switches and variables; an
   internal command's lists all of its own. A setting's second column
   is its about, then `Default: DEFAULT.` or `Required.`, then
   `Environment: TWIN.` where it has one.
4. **`version_line`.**
5. **The expected texts,** in the tests, by hand. Then run, and where
   output and expectation differ, decide from 15.2 which is wrong.
6. `cargo fmt`; the two clippy runs; `heavy cargo nextest run -p
   skein-shell`. Bite: show an internal command on the page, and break
   the 33-character rule; see each test fail for it; restore. Commit,
   push.

## Report

The functions, the tests and their counts, any empty field of
`demo.rs` you met, the bites and the failures seen, and any reading of
15.2 you had to choose.
