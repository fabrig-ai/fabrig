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
  Opening parentheses keep an explicit owner ("skein's (`forge.md`, section 6)").
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
- **The size limit:** in the trees `sized.txt` lists, a Markdown file under `docs/design/` over
  64 KiB (`LIMIT`) fails `--all`, and fails the gate when it crossed the limit or grew while
  over it, renames mapped (`DESIGN.md`, 5). A tree is listed once its layout keeps every file
  under the limit.
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
skein/docs/design/forge.md 7 -> skein/docs/design/forge/decisions.md
title skein/docs/design/forge/decisions.md: Decisions
skein/docs/design/forge.md -> skein/docs/design/forge/README.md
renumber skein/docs/design/forge.md
```

- **Documents** carry their status and execution files. **Directories** carry every tracked
  file, including assets. Moves use `git mv`.
- **Sections** carry their subsections and unnumbered headings. The most specific source
  wins; an explicit subsection move overrides its parent's move. The destination number
  sets the heading levels and subsection prefixes. Sections are inserted in number order;
  other numbers stay as they are. A section destination without a number makes that section
  the whole created document: its opening text becomes the preamble, its own heading goes,
  and its subsections are promoted to sections. Citations of the section become document
  citations. This form requires a created destination and its title.
- **A whole tree fits one map.** Section entries apply before a document entry for the same
  source; the rest of the document and its companions then follow the document entry.
  A `renumber` entry, naming the source before the move, numbers its remaining top-level
  sections consecutively, with their subsections, citations and status labels following.
- **New documents** made from sections require a `title` entry. Destinations must be under
  one of the five trees' `docs/`. Missing sources, duplicate destinations and occupied
  destinations that are not moved away are errors. All map errors are listed before any
  change; an exception during application rolls back the operation. Document destinations
  must end in `.md`. A tree with tracked changes is refused. A map comment begins at the
  start of a line or after whitespace, so a title can contain `C#`.
- **Citations** keep their path style where it resolves to the final target. Relative paths
  in moved text are rebased, section numbers follow their targets, and links follow heading
  anchors. The fallback order is a relative path within the same tree, a code comment's
  tree-relative path, then a root path. Cross-tree owner forms keep their owner; other
  cross-tree paths use the root. Ranges (`to`, `–`, `-`) expand from the source headings,
  including their middle members. Lists and ranges split by destination in their original
  order, then compress within each destination without adding headings.
- **Own dotted numbers** in design Markdown also follow their original headings: `(6.7)`,
  `6.6 to 6.9`, `12.2 and 12.8`, and `4.6's`. Literals, link targets, URLs, versions, dates
  and quantities are excluded. A number that crosses documents gains a path; one that
  stays local keeps its bare form. Single bare numbers still require `section`. The summary
  counts rewritten bare numbers separately. Status prose uses its companion document's
  headings, and capitalized section references follow the map too. These extra readings
  do not change the gate.
- **Status entries** follow their labels into created documents. Each created document gets
  a dated UTC status header, and `No gap.` when it receives no entry. An emptied source
  status gets `No gap.` too. Split entries copy their full text, keeping only each
  destination's labels; whole-document gaps follow every extracted part. Each moved or
  split entry is listed once for review and trimming.
- **By hand** lists ambiguous sections whose candidates need different rewrites, grammar
  that cannot be split mechanically, allow-list globs, and moved names outside citations.
  A shadowed suffix name whose target did not move keeps its prose token and is listed
  with its new resolution. Backticked link labels, including status-index document names,
  follow their links. Reports name final files and lines, with code and configuration
  first. Plans and dated development records stay unchanged and unlisted. Exact allow-list
  paths and section targets are rewritten.
- **The final check** re-reads citations in the resulting tree with the checker's reader,
  and checks each original region against its mapped headings or document. Bare-number
  readings count too. An ambiguous reading may keep a permitted original candidate, but
  may not gain a different heading. A mismatch appears under **Wrong**, showing its final
  text, reading and expected reading. Exit 1 leaves the changes applied and staged: repair
  them before committing. Exit 0 means the map was applied and the readings agree; 2 means
  a map error or a git failure. The mover never commits.
- **Dry runs** perform the same final check in a temporary copy and print the same report,
  without changing the working tree or index. Review the staged changes and By hand list,
  repair those items, commit, then run the citation gate before pushing.
