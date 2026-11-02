# status-gaps: standing rules

Every task of this plan names this note.

- Read `AGENTS.md`, then the `AGENTS.md` of your task's tree. Your task stays in its tree, but
  for the files its spec names in another one, each edited only as the spec says.
- **The status files are the spec** (`inputs/request.md`). Each task closes the gaps its spec
  names, and updates beside the code the design text and status entries the change makes true.
  This departs from the implementer's text ("Not yours: any `docs/design/`"): a task edits
  exactly the design documents, status files and `STATUS.md` its spec lists, in the way it
  lists, and no other. Drafts and `docs/help/` stay untouched unless the spec names them.
- **A status file's form** is `AGENTS.md`, 9: its date line is today's (`date -u +%F`), it holds
  only the gaps as they stand after your change, never history. Remove an entry the change
  closes; where an entry names several gaps, remove only the words for those closed. A status
  with no gap left says `No gap.` under its date line (as `smith/docs/design/domain/session.status.md`
  does). The tree's `docs/design/STATUS.md` summary paragraph drops the sentence or clause about
  the gaps closed, and its index lists only documents that exist.
- **A removed section leaves its number unused:** never renumber the sections after it. Whoever
  removes a document or a section fixes every citation to it in the repository
  (`rg -n '<name>.md' --glob '!temper/docs/plan/**'`), outside `docs/plan/` and outside dated
  reports in `docs/development/`, which stay as written.
- **Design text says what is true now,** in the design's words: no "no longer", "was", "removed",
  "formerly". A sentence about something removed is deleted, not reworded into its history.
- **Old-form counts only fall.** If your removal lowers a count of your tree's
  `scripts/old-forms.txt`, run `python3 -I -B scripts/old_forms.py --write` from the tree's root
  and commit the lowered file; never raise a count. If a count would rise, stop and ask.
- **Goldens** are regenerated with `SKEIN_CODEGEN_WRITE=1` on the test that owns them, never
  edited by hand; read the regenerated diff with `git diff --stat` and keep only what your
  change moves.
- **Removing, not replacing:** no new public items, crates or dependencies unless the spec
  sketches them. A test that exercised only the removed form is deleted with it; a test that
  exercised something else beside it keeps its other assertions.
- **Cross-tree readers:** after a public item is removed, `heavy cargo check -p <pkg> --tests`
  for each package in another tree that named it (the spec lists those it found).
- One commit per coherent step (code and its tests; then the design and status text), each
  saying why. Push your branch.
