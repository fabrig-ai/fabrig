# Citations

`citations.py` checks the citations the design relies on (AGENTS.md, 6): Markdown links and
`x.md` paths, sections ("`x.md`, 5.3", "section N.N") against the cited document's numbered
headings, and a status file's labels ("**Not yet:** 3.2:") against its document.

```sh
python3 -I -B tools/citations/citations.py --base origin/main   # the gate: nothing new broken
python3 -I -B tools/citations/citations.py --all                # every broken citation
python3 -I -B tools/citations/test_citations.py
```

- **The gate** (`--base`) fails when HEAD has a broken citation that the base lacks, counted per
  file and target with renames mapped. Removing or renumbering a section that another file cites
  fails too, at the citing file: whoever removes or moves a section fixes its citations.
- **The full run** (`--all`) lists every broken citation; the count is to stay at zero.
- Both read committed trees through `git archive`: commit before checking.
- **What is read:** every tree's `docs/`, the root `DESIGN.md`, Markdown under `tools/`, every
  `AGENTS.md` and `CLAUDE.md`, and the comments of `.rs .py .sh .toml .js` files.
  Paths may lead up any number of directories (`../../a.md`);
  Markdown links may name any tracked file, with a query or anchor ignored when resolving it.
  Sections after a path include possessives followed by a comma or "section"
  ("`x.md`'s, 6.2", "`x.md`'s section 15") and parentheses ("`x.md` (section 10 is ...)").
  An "its" before a path inherits the nearest explicit tree owner in the same sentence.
  A list item's bold heading path ("- **`x.md`:**", "6. **x.md:**") is also a target for bare
  sections throughout the item. Strings in code are test data, not citations. Left out:
  everything under a tree's `docs/plan/`, whatever its extension (a runner's records of a
  change: a task spec cites what its task deletes), and
  `docs/development/` files dated in their name, which describe the tree at their date.
- **Another repository's documents:** a citation of one ("the probe repository's
  `docs/a.md`") is refused: a finding (`outside`) even where a file of that name exists
  here.
- **Not citations:** dates or counts ("`x.md`, 21 places") after a path,
  a possessive followed by a bare number ("x.md's 32 bytes"), and RFC sections
  ("RFC 6749, section 5.1", "RFC 9500's `testRSA2048` (section 2.1)", with a code span or
  at most two words before the parenthesis). Bare sections after "its", "their", "that" or
  "this" are skipped regardless of case. A status file that says "By step" before its first
  numbered label has step labels: its labels are left unchecked, while its citations are checked.
- **`allow.txt`** excuses what must stay as it is, one line each: `FILE TARGET # reason`, FILE a
  glob and TARGET as the checker prints it, or `*` for every target.
- Exit 0 clean, 1 findings, 2 usage, a malformed `allow.txt` or a git failure.

## Moving documents and sections

`move.py` applies a map to the clean working tree, rewrites citations using the checker's
readings, and stages every change. Run it from the repository root:

```sh
python3 -I -B tools/citations/move.py MOVE_MAP --dry-run
python3 -I -B tools/citations/move.py MOVE_MAP
python3 -I -B tools/citations/test_move.py
```

The map has one move per line; blank lines and `#` comments are ignored. All sources refer
to the tree before the move, so swaps and nested moves apply together:

```text
skein/docs/design/forge-client.md -> skein/docs/design/forge/client.md
skein/docs/design/examples/ -> skein/docs/design/forge/examples/
skein/docs/design/forge.md 6 -> skein/docs/design/forge/markers.md 1
title skein/docs/design/forge/markers.md: Keys' markers
```

- **Documents** carry their status and execution files. **Directories** carry every tracked
  file, including assets. Moves use `git mv`.
- **Sections** carry their subsections and unnumbered headings. The most specific source
  wins; an explicit subsection move overrides its parent's move. The destination number
  sets the heading levels and subsection prefixes. Sections are inserted in number order;
  other numbers stay as they are. A document entry and a section entry for the same source
  document are refused.
- **New documents** made from sections require a `title` entry. Destinations must be under
  one of the five trees' `docs/`. Missing sources, duplicate destinations and occupied
  destinations that are not moved away are errors. All map errors are listed before any
  change; a git failure rolls back the operation. A tree with tracked changes is refused.
- **Citations** keep their path style where it resolves to the final target. Relative paths
  in moved text are rebased, section numbers follow their targets, and links follow heading
  anchors. Status labels are renumbered within their document. The source files and skipped
  plans and dated records are the same as the checker's.
- **By hand** lists citations split across documents, ambiguous section readings, status
  labels that need another document, allow-list globs, and moved names in code strings or
  other text outside the citations. These are left for a person. Exact allow-list paths and
  section targets are rewritten.
- **Dry runs** print the same move summary and By hand list and write nothing. Exit 0 means
  the map was applied (or planned); 2 means a map error or a git failure. The mover never
  commits. Review the staged changes and By hand list, repair those items, commit, then run
  the citation gate before pushing.
