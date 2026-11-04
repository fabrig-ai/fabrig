# bugs-skein: standing rules

Every task of this plan names this note.

- Read `AGENTS.md` and `skein/AGENTS.md`. The plan's source is
  `skein/docs/plan/bugs-skein/inputs/bugs-skein.md`: low priority, small tasks, no reshaping.
- **Only the bug your spec names.** Nothing else from `skein/docs/development/debt/` or its
  reviews, however close: a neighbouring bug goes in your report, under `## Left`.
  `skein-kv/src/store.rs` is another plan's (debt-4); this plan does not touch it.
- **A fix carries a test that fails on the old code.** Write the test first and run it against
  the unfixed code; your report quotes the failure it showed. Where the old code cannot show a
  failure on this machine, your spec says what stands in for it.
- **The design is written.** `## Inputs` names the sections; read those, never a whole design.
  The one `docs/design/` edit a task makes is the removal of its own **Not yet** entry from the
  status file its spec names, leaving `No gap.` if it was the last, with the date moved to the
  day you make it (a stated departure from the implementer's text, whose "Not yours" otherwise
  holds). Where the code teaches that the design is wrong, say so in the report; never change
  the design to fit.
- **Tests that read Markdown:** `.config/gate/read-by-tests.txt` lists them;
  `skein/docs/design/forge-github.md` is among them. No task here edits that file.
