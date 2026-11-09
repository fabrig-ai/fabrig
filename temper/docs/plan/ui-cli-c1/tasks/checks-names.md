+++
id = "checks-names"
part = "checks"
kind = "feature"
needs = ["checks-layout", "render-help", "render-docs"]
files = ["tests/boundary/tests/cli_names.rs", "tests/boundary/Cargo.toml", ".config/gate/README.md", "skein/docs/design/shell.status.md"]
effort = "high"
minutes = 90
+++

## Goal

The command line's check 5 (`skein/docs/foundation/cli.md`, 9): every
`<TREE>_…` token in the tracked files outside a `docs/` directory is
declared in its tree's environment documents, in the set its name
says, and every declared variable is read or set somewhere. No tree has
those documents yet, so every token found is an exception naming the
milestone or round that removes it. A meeting test renders a table's
documents with `skein-shell` and reads them back with the check's
parser, so the two halves of `notes/contracts.md`, 5 agree.

Acceptance:

- `tests/boundary/tests/cli_names.rs` holds `every_name_is_declared`
  (check 5 over the trunk, green with its exceptions) and
  `rendered_documents_read_back` (the meeting test), and the rule's
  table test (Steps, 4).
- Tokens: in each file `git ls-files` lists that has no path component
  `docs` and isn't `cli_names.rs` itself, read as UTF-8 (a file that
  isn't is skipped, and the report counts them), each maximal run of
  `[A-Za-z0-9_]` that is `(SKEIN|SMITH|JIG|TEMPER|HEARTH)_` then one or
  more of `[A-Z0-9_]`.
- Declared: the entries of `<tree>/docs/design/cli/*-env.md` for the
  five trees, by contracts 5's rule. A finding is a token no document
  declares (`TOKEN`); one declared in a set its name doesn't say
  (`TOKEN: declared internal, named public`; `<TREE>_INTERNAL_` is
  internal, `<TREE>_TEST_` test, any other public), or under another
  tree's documents; and a declared variable no file outside the
  documents names (`TOKEN: declared, never read or set`).
- The exceptions: one per finding on the trunk, `until` by
  `notes/contracts.md`, 4 (temper's by the map, the other trees' as
  their rounds), sorted, with the rule of `checks-layout`'s `judged`:
  an unexcepted finding and a stale exception both fail.
- The meeting test builds, with made-up names (no tree's prefix), a
  small tree of two `skein_shell::Table`s and a `TestTable` (a public
  twin both read, a public variable one sets, an internal variable, a
  test variable), renders `environment_document`, `common_document` and
  `TestTable::document`, parses them with the check's parser and finds
  exactly those names with their sets.
- `.config/gate/README.md`'s item 10 names check 5 and `cli_names.rs`
  beside `cli.rs`.
- The **Not yet** entry 15 in `skein/docs/design/shell.status.md` (its
  three lines, from `15: no binary's table`) is removed, its lines alone:
  `table-types`, `render-help` and `render-docs` have built it. This
  departs from the implementer's text, which closes `docs/design/` to
  you: the entry is this task's.
- `heavy cargo nextest run -p fabrig-boundary` passes, well under a
  second for `every_name_is_declared`.

## Inputs

- `skein/docs/foundation/cli.md`, 5 (names and sets) and 9, check 5.
- `skein/docs/design/shell.md`, 15.3 (the documents' form).
- `temper/docs/plan/ui-cli-c1/notes/contracts.md`, 3, 4 and 5
  (binding), and `notes/common.md`.
- `tests/boundary/tests/cli.rs`: `checks-layout`'s `Exception` and
  `judged`; copy them, as each file there is its own crate.
- `tests/boundary/tests/outward.rs`: how a check here lists tracked
  files.

## Owns and may change

- Owns the files in `files`; `cli_names.rs` is new.
- `tests/boundary/Cargo.toml`: the dev-dependency `skein-shell.workspace
  = true`, for the meeting test.
- A token the map doesn't place: `cli C4` if only tests, tools or CI
  name it, else `cli C2`; the report names it.
- If a token on the trunk is plainly not a variable (a constant's name
  in code), except it as its tree's like any other and name it in the
  report; don't rename it.
- Not yours: any file that holds a token; any design document but the
  status entry above.

## Steps

1. **The parser:** a function from a document's text to its `(name,
   set)` entries, by contracts 5 alone.
2. **The tokens:** from `git ls-files` at the workspace root, the files
   read once each, a `BTreeSet` of tokens and, for the read-or-set rule,
   whether each appears outside the documents.
3. **Check 5** as the acceptance says, its findings judged against the
   exceptions.
4. **The rule's table test,** `names_are_judged`: rows over made-up
   inputs (documents' text and files' text, not the trunk) with the
   prefix `DEMO_` standing in for a tree, so that the test itself holds
   no tree's prefix outside its exceptions: a declared token, an
   undeclared one, a wrong set, a declared name read nowhere.
5. **The exceptions:** run with an empty table, then one per finding.
6. **The meeting test,** then the gate's README line and the status
   entry.
7. `cargo fmt`; `heavy cargo clippy -p fabrig-boundary --all-targets --
   -D warnings`; `heavy cargo nextest run -p fabrig-boundary`. Bite:
   remove one exception, then add a stale one, then render a document
   whose set line the parser misses; see each fail; restore. Commit,
   push.

## Report

The check, its exceptions' count by `until`, any token you placed by
judgement, the files skipped as not UTF-8, the meeting test, the tests
and their counts and times, the bites and the failures seen.
