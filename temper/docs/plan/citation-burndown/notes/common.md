# Every citation-burndown task

- Read `AGENTS.md` (1, 2, 6) and `DESIGN.md`, 4: surviving section numbers never change, and
  whoever moves or removes a section fixes every citation to it.
- **The checker** is `tools/citations/citations.py`; its README says what it reads. It checks
  committed trees (`git archive`): commit first, then run
  `python3 -I -B tools/citations/citations.py --all` from the slot's root. It and
  `tools/citations/test_citations.py` build nothing: run them directly, never through heavy.
- **Every finding has one owner,** listed in
  `temper/docs/plan/citation-burndown/notes/findings.md`. Fix only your own. Another task's
  finding stays on your branch, untouched: its task fixes it. Your acceptance is that your own
  findings are gone and `--all` prints none that the list lacks.
- **Departures from your implementer's text,** for the docs tasks: editing `docs/design/` and
  drafts is the work, text only, on the lines the spec names. A docs task changes no Rust, so
  it runs no `cargo fmt` and no clippy; it runs the tests that read the file it changes, which
  its spec names (none, where no test reads it).
- **Text fixes stay narrow:** change a citation's words only (the file it names, its section),
  never what the sentence says, and never a surviving section's number.
- **Not yours, any task:** `temper/docs/design/qa.md`, the pointer the v0.1.0 ladder's frozen kit
  still cites (it stays until the kit's next edition drops the path); anything under
  `temper/docs/plan/v0.1.0-ladder/`; `tools/citations/allow.txt` (the `allow-and-zero` task's);
  `tools/citations/*.py` and its README (the `checker-readings` task's).
- **Your report** lists the findings your branch fixed (`file:line`, target, as the checker
  printed them), and the `--all` count on your branch before your change and after it.
