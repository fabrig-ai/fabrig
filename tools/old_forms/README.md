# Old forms

Each tree's `scripts/old_forms.py` supplies its forms and exemptions to
`old_forms.py` here, loaded by absolute path with `runpy`. A landing check
runs it from each tree's root, in Python's isolated mode:

```sh
cd smith
python3 -I -B scripts/old_forms.py
python3 -I -B scripts/old_forms.py --write
```

The counter reads tracked Rust files below the current tree, omitting
`generated` path components and the tree's `EXEMPT_DIRECTORIES`. Code forms
retain string contents, with ordinary literal escapes replaced by separators.
`SYNTAX_FORMS` instead mask literals, so a string describing an attribute,
loop or struct literal does not count as code. Nested block comments, line
comments, doc comments, raw strings, escaped strings, characters and lifetimes
are distinguished by the shared scanner.

`COMMENT_FORMS` runs separately on comments and doc comments. Its rows use
`comment:` in the same baseline file. Every tree counts `actual`/`actually`
and `genuine`/`genuinely` in comments. Smith also counts the history words,
contract stamps, `Lower test of`, and missing Markdown citations. Smith's
code word forms split snake case, camel case and uppercase words, including
words in strings; the same word splitter handles its comment word forms.

A baseline can append new forms with `--write`; existing counts can only
fall. Existing rows must be a prefix of the configured order, without
negative counts or duplicates. Comments, including `# Raised:`, survive a
rewrite. A failed write leaves the file unchanged.

The shared configuration supports `MARKERS` prefilters, `WORDS` identifier
prefilters, per-form `EXCLUDE` prefixes and `INCLUDE` path expressions, and
`EXEMPT` exact names or name-to-qualifier maps. Comment keys in those options
use their `comment:` prefix. Regex forms count matches; computed forms receive
the selected text, relative Rust path, tree root and repository root.

Smith's structural forms have these scopes:

- `full_limits` counts literals whose type's final name is `Limits`, including
  qualified types, without a struct update at the literal's outer level.
  Nested expressions do not end the literal. Declarations, impls, return-type
  function bodies and simple destructuring patterns are excluded, and so is
  `file_layer::Limits`, which `skein/docs/design/runtime/io/files.md`, has a reader write out.
  Smith's `testing/` fixture definitions are excluded: a whole literal belongs
  there. Readers and the crate's `#[cfg(test)]` copies remain counted
  (`skein/docs/foundation/cli.md`, 3; the user, 2026-11-12).
- `bare_wire_kind` counts two-digit hex literals in `kind:` fields or as the
  first argument to `frame_writer`, `write_record`, `push_record` or
  `encode_record`. It excludes arbitrary byte arrays and wider wire kinds.
- `test_windows` counts calls in `tests/`, `benchmarks/`, `tests.rs`,
  `*_tests.rs`, and inline `#[cfg(test)]` or `#[test]` items.
- `domain_while` and `domain_loop` count production code in the seven domain
  crates; those test files and inline test items are excluded.
- `comment:contract_stamp` counts standalone `/// Contract:` lines, ignoring
  case and spacing. It includes stamps on fields and types; `//!` module
  citations and prose mentioning a contract do not count.
- `comment:dead_citation` checks each Markdown path occurrence in comments.
  Paths resolve from the repository, tree, source directory or tree design
  directory. Bare document names also resolve against tracked Markdown files
  anywhere in the repository. URLs are excluded; repeated missing citations
  count separately. This checks file existence, not section numbers.

Run the shared regression suite and the retained entry suites one at a time:

```sh
python3 -I -B tools/old_forms/test_old_forms.py
python3 -I -B smith/scripts/test_old_forms.py
python3 -I -B jig/scripts/test_old_forms.py
python3 -I -B temper/scripts/test_old_forms.py
```
