# Every outward-citations task

- Read `AGENTS.md` (1, 2, 6) and `DESIGN.md`, 4: surviving section numbers never change.
- **The checker** is `tools/citations/citations.py`; its README says what it reads. It checks
  committed trees (`git archive`): commit first, then run
  `python3 -I -B tools/citations/citations.py --all` from the slot's root. It and
  `tools/citations/test_citations.py` build nothing: run them directly, never through heavy.
- **Every finding has one owner,** listed in
  `temper/docs/plan/outward-citations/notes/findings.md`. Fix only your own; another task's
  finding stays on your branch untouched, and its task fixes it.
- **No Rust changes in this plan,** so no task builds: no `heavy`, no clippy, no nextest. A
  `.rs` file is changed in its comments only, checked by `cargo fmt --check -p <package>`
  (which builds nothing); a `.sh` file in its comments only, checked by `sh -n <file>`.
- **Wording, never behaviour,** outside `tools/citations/`: a changed line is a comment or
  prose, and says what is true now in fabrig's own words (`AGENTS.md`, 2). Where a sentence
  cites a document kept outside this repository, the citation is dropped or the sentence says
  the fact itself; it is never re-pointed at another repository, and never names one.
- **`tools/citations/allow.txt`** is `strict-checker`'s alone.
- **The outward check:** if `tests/boundary/tests/outward.rs` exists at your base (a parallel
  plan's), its inventory only falls on your branch: run its test and say so in the report.
- **Your report** lists the findings your branch fixed (`file:line`, target, as the checker
  printed them), and the `--all` count on your branch before your change and after it.
