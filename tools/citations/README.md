# Citations

`citations.py` checks the citations the design relies on (AGENTS.md, 9): Markdown links and
`x.md` paths, sections ("`x.md`, 5.3", "section 5.3") against the cited document's numbered
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
- **What is read:** every tree's `docs/`, every `AGENTS.md` and `CLAUDE.md`, and the comments of
  `.rs .py .sh .toml .js` files. Paths may lead up any number of directories (`../../a.md`);
  Markdown links may name any tracked file, with a query or anchor ignored when resolving it.
  Sections after a path include possessives followed by a comma or "section"
  ("`x.md`'s, 6.2", "`x.md`'s section 15") and parentheses ("`x.md` (section 10 is ...)").
  An "its" before a path inherits the nearest explicit tree owner in the same sentence.
  A list item's bold heading path ("- **`x.md`:**", "6. **x.md:**") is also a target for bare
  sections throughout the item. Strings in code are test data, not citations. Left out:
  `docs/plan/`, fab's records of a change (a task spec cites what its task deletes), and
  `docs/development/` files dated in their name, which describe the tree at their date.
- **Not citations:** another repository's documents ("hearth's `domain/store.md`", "the agents
  repository's `docs/design/fab.md`"), dates or counts ("`x.md`, 21 places") after a path,
  a possessive followed by a bare number ("x.md's 32 bytes"), and RFC sections
  ("RFC 6749, section 5.1", "RFC 9500's `testRSA2048` (section 2.1)", with a code span or
  at most two words before the parenthesis). Bare sections after "its", "their", "that" or
  "this" are skipped regardless of case. A status file that says "By step" before its first
  numbered label has step labels: its labels are left unchecked, while its citations are checked.
- **`allow.txt`** excuses what must stay as it is, one line each: `FILE TARGET # reason`, FILE a
  glob and TARGET as the checker prints it, or `*` for every target.
- Exit 0 clean, 1 findings, 2 usage, a malformed `allow.txt` or a git failure.
