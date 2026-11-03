# fab3-fixes: standing rules

Every task of this plan names this note.

- Read `AGENTS.md`, then the `AGENTS.md` of your task's tree. Your task stays in its tree, but
  for the files its spec names in another one, each edited only as the spec says.
- **The request** is `temper/docs/plan/fab3-fixes/inputs/request.md`: three findings of the
  acceptance review of forgejo-connect and withdrawn-forms, one task each, none reading another.
  Do only your finding; what you notice beyond it goes in your report's `## Left`.
- **Design and status files:** a task edits exactly the design documents, status files,
  `STATUS.md` and `AGENTS.md` lines its spec lists, in the way it lists, and no other. This
  departs from the implementer's text ("Not yours: any `docs/design/`") only where a spec says
  so. Drafts and `docs/help/` stay untouched.
- **A status file's form** is `DESIGN.md`, 2: its date line is today's (`date -u +%F`), it holds
  the gaps as they stand after your change, never history.
- **Design text says what is true now,** in the design's words: no "no longer", "was",
  "removed", "formerly", "withdrawn" for something gone. A sentence about something gone is
  deleted, not reworded into its history.
- **No probe of a real service:** no test resolves a real host's name, opens a connection or
  reads a login; a configuration that names a real address is checked by the parser that does
  neither.
- **Old-form counts only fall.** Run `python3 -I -B scripts/old_forms.py` from your tree's root
  before you push; if your change lowers a count, run it with `--write` and commit the lowered
  `scripts/old-forms.txt`; if a count would rise, fix it before you push.
- **Citations:** before pushing a change to Markdown, run
  `python3 -I -B tools/citations/citations.py --base origin/main` from the repository root and
  fix what it lists in the files you own.
- One commit per coherent step (the code and its tests; then any text), each saying why. Push
  your branch.
