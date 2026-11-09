+++
id = "render-docs"
part = "shell-table"
kind = "feature"
needs = ["table-types"]
files = ["skein/crates/skein-shell/src/table/reference.rs", "skein/crates/skein-shell/src/table/reference/**"]
effort = "high"
minutes = 100
+++

## Goal

A tree's tables render the reference documents of
`skein/docs/foundation/cli.md`, 8, in the form of
`skein/docs/design/shell.md`, 15.3: each binary's arguments and
environment, the tree's common environment and its test set. Pure text;
no tree writes a document yet (C2's). The environment documents are
what check 5 reads (`checks-names`), so their entries' form is a
contract (`notes/contracts.md`, 5).

Acceptance:

- `table/reference.rs` holds:

  ```rust
  impl Table {
      /// `<binary>-args.md`: its commands and flags, public then internal.
      #[must_use]
      pub fn arguments_document(&self) -> String;
      /// `<binary>-env.md`: the variables this binary alone reads, or sets,
      /// of the tree whose tables are `tree` (this one among them).
      #[must_use]
      pub fn environment_document(&self, tree: &[&Table]) -> String;
      /// `common-env.md`: the variables two or more of `tables` read.
      #[must_use]
      pub fn common_document(tree: &str, tables: &[&Table]) -> String;
  }
  impl TestTable {
      /// `test-env.md`.
      #[must_use]
      pub fn document(&self) -> String;
  }
  ```

  each following 15.3: titles, the generated paragraph, `## Public`,
  `## Internal` (and `## Test` after them when a binary's table holds a
  test entry), and the entries' headings and list items.
- A binary's variables are, once each by name, in this order: the
  twins of the table's groups; the table's variables; each command's
  groups' twins and variables, in the commands' order; then the host's
  variables its settings' `Fallback::Base` read (the XDG one, then
  `HOME`). A twin's entry takes its set, form and default from its
  setting, its line from the setting's about, and `**Twin of:**
  `--name``.
- With the demo tables (`DEMO`, `OTHER`, `DEMO_TESTS`): `DEMO`'s
  environment document holds `DEMO_JOB` and `DEMO_INTERNAL_DUMP` and no
  twin; the common document holds `DEMO_CONFIG`, `DEMO_DATA`,
  `DEMO_CACHE`, the XDG variables and `HOME`, each `**Read by:**
  demo-box, demo-engine`; the test document holds `DEMO_TEST_RATE` and
  `DEMO_TEST_WRAPPER`. Each of the four documents equals an expected
  text written in the test by hand from 15.3.
- Every entry of an environment document is a `` ### `NAME` `` line
  whose list's first item is `- **Set:** public`, `internal` or `test`
  (contracts 5): a test parses the three environment documents by that
  rule alone and finds exactly the demo's variables with their sets.
- The **Not yet** entry 15 in `shell.status.md` stays (`checks-names`
  removes it).
- `heavy cargo clippy -p skein-shell --all-targets -- -D warnings` and its
  wasm32 clippy pass; skein-shell's focused tests pass.

## Inputs

- `skein/docs/design/shell.md`, 15.1 and 15.3 (the form; binding).
- `skein/docs/foundation/cli.md`, 4 and 8.
- `temper/docs/plan/ui-cli-c1/notes/contracts.md`, 1 and 5, and
  `notes/common.md`.
- `skein/crates/skein-shell/src/table.rs` and `table/demo.rs`, as
  `table-types` left them.

## Owns and may change

- Owns `table/reference.rs` and its tests' module.
- `table/demo.rs` is `table-types`'s fixture, which `render-help` reads
  in parallel: read it, never edit it. Write expected texts from it as
  it is, and name any empty field you met.
- A default's words are a private helper of `reference.rs`, a `match`
  on `Fallback` worded as `settings.rs`'s `help` words it; don't edit
  `settings.rs`.
- The generated paragraph, word for word: `Generated from BINARY's
  table (skein/docs/design/shell.md, 15.3): a test checks it byte for
  byte, and the tree's rewrite switch rewrites it.` (`the tree's tables`
  for the common document, `the tree's test table` for the test one),
  with the path in a code span.
- The arguments document's sections: in each set, the commands of that
  set, then every flag of that set once by name: the table's groups'
  settings, then each command's settings and switches. A command's
  `**Flags:**` names its flags of its own set.
- skein-shell is on the `ring` profile: no indexing, no unchecked
  arithmetic, no `as`.
- Not yours: `table/help.rs` (`render-help`, in parallel), `table.rs`'s
  types, `lib.rs`.

## Steps

1. **An entry writer** for a variable, and one for a command and a
   flag, each pushing Markdown into a `String`; the documents call them.
2. **The variables of a table,** once by name, in the order above, each
   with its way; a twin built from its setting.
3. **The four documents.** For `environment_document`, a variable
   leaves this binary's document when two or more of `tree`'s tables
   read it; it's the common document's then.
4. **Tests:** the four expected texts, by hand; the parse of contracts
   5 over the three environment documents.
5. `cargo fmt`; the two clippy runs; `heavy cargo nextest run -p
   skein-shell`. Bite: put an internal variable under `## Public`, and
   keep a common variable in `DEMO`'s own document; see the tests fail
   for each; restore. Commit, push.

## Report

The functions, the tests and their counts, any empty field of
`demo.rs` you met, the bites and the failures seen, and any reading of
15.3 you had to choose.
