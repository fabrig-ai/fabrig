# Citations

`citations.py` checks the citations the design relies on (AGENTS.md, 6): Markdown links and
`x.md` paths, sections ("`x.md`, 5.3", "section N.N") against the cited document's numbered
headings, and a status file's labels ("**Not yet:** 3.2:") against its document. It also holds
design documents under the size limit and each tree's `STATUS.md` to its fixed page.
`status.py`, beside it, rolls up the status files and prints that page (below).

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
  Opening parentheses keep an explicit owner only directly after its possessive
  ("skein's (`forge/README.md`, section 5)"); intervening words end that ownership at "(".
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
  under the limit. Its line reads `PATH: size N bytes, over the 65,536-byte limit: split it
  along its concepts, each piece under 40 KB (DESIGN.md, 5); don't cut text to fit`. Nothing
  fails at the 48 KB mark or at an opening past 8 KB: `status.py` lists those files.
- **The fixed page:** in the trees `sized.txt` lists, a tree's `docs/design/STATUS.md` whose
  text isn't, byte for byte, the page `status.py --page TREE` prints fails `--all`, and fails
  the gate whatever the base's page holds, with the line `PATH: STATUS.md is a fixed page
  (DESIGN.md, 4): put the gap in its document's status file`. Only those five paths: the
  mockups' `STATUS.md` (`temper/docs/design/client/web/mockups/`) keeps its own rules, and a
  tree with no page fails nothing here. The checker takes the page from `status.py`'s
  `page(tree)`, the file beside it loaded by path, since `-I` keeps the script's directory off
  `sys.path`.
- **`allow.txt`** excuses what must stay as it is, one line each: `FILE TARGET # reason`, FILE a
  glob and TARGET as the checker prints it, or `*` for every target.
- Exit 0 clean, 1 findings, 2 usage, a malformed `allow.txt` or a git failure.

## The status roll-up and the page

`status.py` reads the status files of a committed tree and rolls them up, and prints each
tree's fixed `docs/design/STATUS.md` (`temper/docs/design/draft/design-upkeep.md`, 3.1 and
3.2). It writes nothing, and no gate runs the roll-up.

```sh
python3 -I -B tools/citations/status.py [--head REV] [TREE...]   # the roll-up, every tree when none is named
python3 -I -B tools/citations/status.py --page TREE              # the tree's fixed page
python3 -I -B tools/citations/test_status.py
```

- **What it reads:** every tracked `*.status.md` under each named tree's `docs/design/`, at
  REV (`HEAD` by default), through `git archive` as the checker does. A status file under
  `docs/design/draft/`, `draft/archive/` included, is a draft's; any other is a design
  document's. `--page` reads nothing and needs no repository.
- **The opening:** the first line that isn't blank is a heading (`# `), and the next paragraph
  starts with a date, `YYYY-MM-DD` (`DESIGN.md`, 2). The heading and that paragraph open the
  file; its text is what follows them, and its date is that one.
- **Labels:** `**Not yet:**`, `**Departs:**`, `**Design lags:**`, `**Design departs:**` and
  `**Open:**`, each at the start of a line, after any indent and a list marker `- `. A label's
  text runs from it to the next label or to the end of the file, its bullets included. Other
  bold words (`**Later:**`) are text. Text before the first label holds no entry and leaves
  the file read. A design document's status may use any of the five: goals record there where
  the design departs from a request or a mockup, as **Design departs**.
- **Entries:** in a label's text, each section label counts one entry: a section number, or a
  list of them as the checker reads a status label (`3.2:`, `3.2 and 4:`, `5, 6:`), with its
  colon, at the start of the text, at the start of a line or list item, or after a sentence's
  end (`. `). A text with no section label counts one entry, and none if it reads "none",
  whatever its case, a final full stop aside.
- **In flight:** a **Not yet** entry whose text, to the next section label or the end of its
  label's text, ends with a tag `[in flight: TEXT]`, as `tools/close/README.md` reads one: after
  the tag only sentence punctuation and spaces.
- **Done:** it has labels and none has an entry, or it has none and its text is empty or
  reads "No gap." alone. An empty text is what `close` leaves when it settles a file's last
  entry, and it reads as done. A draft's status is also done when no label but **Design
  departs** has an entry, so when its **Design lags** and **Open** read none: departures
  record what the design does instead, and nothing closes them, since goals leave a draft's
  text as it is (`temper/docs/design/draft/design-upkeep.md`, 2.5 and 3.2). A design
  document's status is done only as the first sentence says, whatever its labels.
- **Archived:** a status file under `docs/design/draft/archive/` counts no entry, neither in
  the open entries nor as a line of a status file with an open entry: its draft has gone.
  It is still read, and listed as unread when it is in neither form.
- **Unread:** a file without the opening, or with no label and any text but "No gap.". It is
  listed as unread, never as done, and counts no entry.
- **What it prints,** for each tree in the order named, or `temper`, `smith`, `skein`, `jig`,
  `hearth` when none is named, with a blank line between trees. Paths are from the repository
  root and sorted bytewise; a list with nothing in it prints `- none`; counts are plain and
  bytes take thousands separators.

  ```
  # temper

  Open entries: 812 Not yet (14 in flight), 181 Departs, 97 Design lags, 256 Design departs, 108 Open.

  Status files with an open entry:
  - temper/docs/design/domain/landing.status.md, 2026-11-20: 3 Not yet (1 in flight), 1 Departs

  Over the mark, 49,152 bytes, largest first:
  - temper/docs/design/domain/landing.md: 65,526 bytes

  Openings over 8,192 bytes, largest first:
  - temper/docs/design/testing/process-tier.md: 46,132 bytes

  Drafts done, outside draft/archive/:
  - temper/docs/design/draft/archive/flake-regate.md

  Unread status files:
  - temper/docs/design/draft/archive/cli-c8-tools.status.md
  ```

  - **Open entries:** the tree's entries by label, every label shown, zeros too, with the
    **Not yet** entries in flight; none from `draft/archive/`.
  - **A status file's line:** each file with an entry, outside `draft/archive/`, with its date
    and its labels that have entries, in the order above; "(N in flight)" only where N isn't 0.
  - **Over the mark:** every Markdown file under the tree's `docs/design/` over 49,152 bytes
    (`DESIGN.md`, 5), status files and `draft/archive/` included, largest first, then by path.
  - **Openings over 8,192 bytes:** every Markdown file under the tree's `docs/design/` whose
    opening is over 8,192 bytes, giving the opening's bytes, largest opening first, then by
    path (`DESIGN.md`, 5; `design-upkeep.md`, 4.1). A file's opening is its bytes before the
    first line that starts with `## ` and a digit, a numbered section's heading, in a code
    fence or not; all its bytes where it has no such line. Status files, notes files and
    execution files (`*.status.md`, `*.notes.md`, `*.execution.md`) are left out, not read
    for an opening; `draft/archive/` is included, as over the mark.
  - **Drafts done:** each draft whose status file is done, outside `draft/archive/`, named by
    the draft's own path (`DESIGN.md`, 4); a draft's notes file and execution file are never drafts.
  - **Unread status files.**
- **The page,** printed by `--page TREE` and returned by `page(tree)`, TREE one of the five:
  the text below, less the two spaces that indent it here, with `TREE` replaced by the tree's
  name, ending in one newline. It changes only with this rule (`DESIGN.md`, 4): its summary of
  what's due stays as written, though the roll-up also lists the long openings and counts a
  draft with only departures left as done.

  ```
  # Status

  The status of each of TREE's design documents and drafts is beside it,
  as `<name>.status.md`: where the code lags or departs from a design
  document (**Not yet**, **Departs**), and where the design lags or
  departs from a draft (**Design lags**, **Design departs**, **Open**),
  as `DESIGN.md`, 2, says. A gap goes in its document's status file,
  never on this page.

  To roll them up, from the repository root:

      python3 -I -B tools/citations/status.py TREE

  It prints the open entries by label, a line for each status file with
  an open entry, and what's due: the files over 48 KB, largest first, the
  drafts whose status records no gap, and the status files it can't read.

  This page is fixed: `python3 -I -B tools/citations/status.py --page TREE`
  prints it, and the citation checker refuses any other text
  (`DESIGN.md`, 4).
  ```
- **Exit** 0 when it printed, 2 for a usage error (a tree that isn't one of the five, `--page`
  without exactly one tree) or a git failure. It uses Python's standard library alone.
- **Its tests,** `test_status.py`, are a table of small committed trees, each with its expected
  output, as `test_citations.py` builds its repositories.

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
skein/docs/design/forge/README.md 5 -> skein/docs/design/forge/markers.md 1
title skein/docs/design/forge/markers.md: Keys' markers
skein/docs/design/forge/README.md 6 -> skein/docs/design/forge/decisions.md
title skein/docs/design/forge/decisions.md: Decisions
skein/docs/design/forge.md -> skein/docs/design/forge/README.md
renumber skein/docs/design/forge.md
renumber skein/docs/design/forge/markers.md
heading skein/docs/design/forge/markers.md 1: The markers
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
  It can also name a created document: numbering applies after all sections arrive.
  A promoted section and explicitly numbered sections can share a created document.
- **Headings** can be retitled with `heading PATH N: TEXT`. The path must be a created
  document and N its number after section placement, before renumbering. Links' anchors
  follow both the title and the final number. Every text file written ends with one newline.
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
- **Ambiguous bare sections** choose the sentence's antecedent when its path is possessive
  and at most two words intervene before the section reference; otherwise they choose the
  citing document. The report lists each decision whose candidates map differently, and
  the final check verifies the chosen reading. Other ambiguities remain listed for review.
- **Code strings and other tracked text** follow the same path and section map when they
  contain a citation-shaped path followed by section numbers. The mover reads physical
  literals without evaluating escapes or interpolation, and reports each changed line under
  **Rewrote in code strings**, with its Cargo package when present. Message expectations
  and text fixtures follow the same rules. Run `cargo test -p` for each affected package.
  Runtime paths, interpolated paths and document paths used to read files remain listed
  under **By hand** when they are not citation-shaped.
- **Status entries** follow their labels into created documents. Each created document gets
  a dated UTC status header, and `No gap.` when it receives no entry. An emptied source
  status gets `No gap.` too. Split entries copy their full text, keeping only each
  destination's labels; whole-document gaps follow every extracted part. Each moved or
  split entry is listed once for review and trimming.
- **By hand** lists ambiguous sections whose candidates need different rewrites, grammar
  that cannot be split mechanically, allow-list globs, and moved names outside citations.
  A shadowed suffix name without sections keeps its prose token and is listed with its
  new resolution. With sections it gains an explicit owner or resolving root path.
  Backticked link labels, including status-index document names,
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
