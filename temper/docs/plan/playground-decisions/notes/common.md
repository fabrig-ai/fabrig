# playground-decisions: standing rules

Every task of this plan names this note.

- Read `AGENTS.md`, then the `AGENTS.md` of each tree your task touches. Your task stays in the
  trees and files its spec names; a reader file outside them is edited only as the spec says.
- **The design is already written** (`inputs/request.md`): the user decided five questions on
  2026-11-02 and the design states each decision. Your task brings the code to the sections its
  spec cites. Design documents' text is the designer's: you never edit it.
- **Status files close with the code.** This departs from the implementer's text ("Not yours: any
  `docs/design/`"): a task edits exactly the status files, `STATUS.md` lines and entries its spec
  lists, in the way it lists, and no other. A status file's form is `DESIGN.md`, 2: its date line
  is today's (`date -u +%F`), it holds only the gaps as they stand after your change, never
  history. Remove an entry your change closes; where an entry names several gaps, remove only the
  words for those closed, and leave the rest word for word. A status left with no gap says
  `No gap.` under its date line.
- **Drafts, `docs/help/` and the archived draft** stay untouched unless the spec names a file
  there and says how (a reference page regenerated from the table, never hand-written).
- **No budget, timeout or limit moves** unless the spec sketches it (`inputs/request.md`; skein's
  `docs/foundation/testing-strategy.md`, 8.1).
- **Goldens and generated pages** are regenerated with `SKEIN_CODEGEN_WRITE=1` on the test that
  owns them, never edited by hand; read the regenerated diff with `git diff --stat` and keep only
  what your change moves.
- **Schemas change in place,** at their current version, and only where the spec sketches it.
- **Old-form counts only fall:** `python3 -I -B <tree>/scripts/old_forms.py` for each tree you
  touch; `--write` only to lower a count for what you removed. A rising count is a question.
- **Tests:** after each change the narrowest test (`AGENTS.md`, 6); before the push, the focused
  tests of every package you changed and of its direct dependents (`heavy cargo nextest run -p
  ...`), and `heavy cargo check -p <pkg> --tests` for each cross-tree reader of a public item you
  changed. Check each run's count: a run that selected nothing proves nothing.
- **Before the push,** when your branch changed any Markdown: `python3 -I -B
  tools/citations/citations.py --base origin/main` from the repository root, and fix what it
  lists in your files.
- One commit per coherent step (the code and its tests; then the status text), each saying why.
  Push your branch.
