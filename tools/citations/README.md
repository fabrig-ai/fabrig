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
- **The full run** (`--all`) lists every broken citation; the count is to reach zero.
- Both read committed trees through `git archive`: commit before checking.
- **What is read:** every tree's `docs/` (but `docs/development/` files dated in their name, which
  describe the tree at their date), every `AGENTS.md` and `CLAUDE.md`, and the comments of
  `.rs .py .sh .toml .js` files. Strings in code are test data, not citations.
- **Not citations:** another repository's documents ("hearth's `domain/store.md`", "the agents
  repository's `docs/design/fab.md`"), dates after a path, and RFC sections.
- **`allow.txt`** excuses what must stay as it is, one line each: `FILE TARGET # reason`, FILE a
  glob and TARGET as the checker prints it, or `*` for every target.
- Exit 0 clean, 1 findings, 2 usage, a malformed `allow.txt` or a git failure.
