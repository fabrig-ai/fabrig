# debt-4: standing rules

Every task of this plan names this note.

- Read `AGENTS.md`, then the `AGENTS.md` of your task's tree. Your task stays in its tree, but
  for the files its spec names in another one, each edited only as the spec says.
- **The request** is `temper/docs/plan/debt-4/inputs/request.md`: four independent parts, one
  per tree, each paying the items of the tree's `docs/development/debt/` it names. Read your
  item's file whole first, and the design text it cites. Your spec says which of the item's
  words the cut read, and how, where the item leaves a choice.
- **Pay exactly your item,** or the share of it your spec gives you. Make the change so the
  item's **Paid when** holds (as your spec reads it), and nothing else. Behaviour stays
  unchanged unless the spec says otherwise: tests that pinned removed code go with it; every
  other test passes unchanged, but for the literals and arms the compiler asks for because of
  a type the spec changes.
- **Keep new code in proportion.** A state enum replaces its flags and the transitions it
  needs, not a layer around them; a branch that adds more lines than it removes says why in
  its report.
- **No wire or stored format changes,** and no public item is removed or changes its
  signature unless the spec sketches it; additions the spec sketches are fine. If your change
  would move a wire shape, a schema, a golden file or a settings document's accepted shape,
  stop and ask.
- **The item's file and its row.** The task the spec names as the item's last deletes the
  item's file and its row in that tree's `docs/development/debt/README.md`, in the same branch
  (the README's rule: git keeps the history), and fixes every citation of the file the
  deletion breaks. A task whose spec says the item stays open leaves the file.
- **Status files and design text are the spec's.** This departs from the implementer's text
  ("Not yours: any `docs/design/`"): a task edits exactly the design documents, status files
  and `STATUS.md` its spec lists, in the way it lists, and no other. Drafts stay untouched.
- **A status file's form** is `DESIGN.md`, 2: its date line is today's (`date -u +%F`), and it
  holds only the gaps as they stand after your change, never history. Remove an entry the
  change closes; where an entry names several gaps, remove only the words for those closed. A
  status with no gap left says `No gap.` under its date line.
- **Citations.** Before pushing, run `python3 -I -B tools/citations/citations.py --base
  origin/main` from the repository root and fix what it lists in your files; `--all` reads 0
  at this plan's base and must stay at 0.
- **Comments and design text say what is true now,** in the design's words: no "no longer",
  "was", "removed", "formerly" for something gone, and no debt item's ID in code (an item's
  file goes when it is paid). A sentence about something removed is deleted, not reworded
  into its history.
- **Old-form counts only fall.** If your change lowers a count of your tree's
  `scripts/old-forms.txt`, run `python3 -I -B scripts/old_forms.py --write` from the tree's
  root and commit the lowered file; never raise a count. If a count would rise, stop and ask.
- **Goldens** are regenerated with `SKEIN_CODEGEN_WRITE=1` on the test that owns them, never
  edited by hand; your spec names any it expects, and none means none moves.
- **Acceptance searches are narrow:** code that legitimately uses the same word for something
  else is exempted, not changed; your spec names the exemptions it found, and your report
  names any other you meet.
- **Markdown read by tests:** `.config/gate/read-by-tests.txt` lists it. A task that edits such
  a file runs the tests that read it, as its spec names.
- One commit per coherent step (the code and its tests; then the design, status and debt
  text), each saying why. Push your branch.
