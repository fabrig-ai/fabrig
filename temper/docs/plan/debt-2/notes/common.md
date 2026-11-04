# debt-2: standing rules

Every task of this plan names this note.

- Read `AGENTS.md`, then the `AGENTS.md` of your task's tree. Your task stays in its tree, but
  for the files its spec names in another one, each edited only as the spec says.
- **The request** is `temper/docs/plan/debt-2/inputs/request.md`: four independent parts, one
  per tree, each paying one item of the tree's `docs/development/debt/`. Read your item's file
  whole first, and the design text it cites. Your spec says which of the item's words the cut
  read, and how, where the item leaves a choice.
- **Pay exactly your item,** or the share of it your spec gives you. Make the change so the
  item's **Paid when** holds (as your spec reads it), and nothing else. Behaviour stays
  unchanged unless the spec says otherwise: tests that pinned removed code go with it; every
  other test passes unchanged, but for the literals and arms the compiler asks for because of
  a type the spec changes.
- **No wire or stored format changes,** and no public item is removed or changes its
  signature unless the spec sketches it. If your change would move a wire shape, a schema, a
  golden file or a settings document's accepted shape beyond what the spec says, stop and ask.
- **The item's file and its row.** The task the spec names as the item's last deletes the
  item's file and its row in that tree's `docs/development/debt/README.md`, in the same branch
  (the README's rule: git keeps the history), and fixes every citation of the file the
  deletion breaks. A task whose spec says the item stays open edits the item's file as the
  spec says instead, or leaves it.
- **Status files and design text are the spec's.** This departs from the implementer's text
  ("Not yours: any `docs/design/`"): a task edits exactly the design documents, status files
  and `STATUS.md` its spec lists, in the way it lists, and no other. Drafts stay untouched;
  `docs/help/` only where a spec names a page its test writes.
- **A status file's form** is `DESIGN.md`, 2: its date line is today's (`date -u +%F`), and it
  holds only the gaps as they stand after your change, never history. Remove an entry the
  change closes; where an entry names several gaps, remove only the words for those closed. A
  status with no gap left says `No gap.` under its date line. The tree's `docs/design/STATUS.md`
  summary drops the sentence or clause about the gap closed, only where the spec says so.
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
- **Goldens and written pages** are regenerated with `SKEIN_CODEGEN_WRITE=1` on the test that
  owns them, never edited by hand; read the regenerated diff with `git diff --stat` and keep
  only what your change moves.
- **Removing, not replacing:** no new public item, crate or dependency unless the spec
  sketches it. A test that exercised only removed code is deleted with it; a test that
  exercised something else beside it keeps its other assertions.
- **Acceptance searches are narrow:** code that legitimately uses the same word for something
  else (a test that checks an old member is refused, say) is exempted, not changed; your spec
  names the exemptions it found, and your report names any other you meet.
- **Cross-tree readers:** where your branch changes a public item that another tree names, it
  carries that tree's lines, each edited only for that item (a literal's field, an arm, an
  import); your spec lists those found. Then run `heavy cargo check -p <pkg> --tests` for each
  package in another tree that names it.
- **Markdown read by tests:** `.config/gate/read-by-tests.txt` lists it (`smith/docs/design/**`
  among them). A task that edits such a file runs the tests that read it, as its spec names.
- One commit per coherent step (the code and its tests; then the design, status and debt
  text), each saying why. Push your branch.
