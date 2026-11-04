# bugs-temper: standing rules

Every task of this plan names this note.

- Read `AGENTS.md`, then `temper/AGENTS.md`. Your task stays in the temper tree.
- **The request** is `temper/docs/plan/bugs-temper/inputs/bugs-temper.md`: three temper bugs and
  nothing else from the debt list. Low priority: fix exactly your bug, with no reshaping. Tests
  that pinned the defect change with it; every other test passes unchanged.
- **A test that fails on the old code:** every fix carries one. Your spec names it and how to see
  it fail. Your report quotes the failing line. A spec that departs from this says why.
- **Stay clear of debt-5,** which works beside this plan: temper-docs, pipeline-name comments,
  the client's interim account counts and `#[path]` fixtures.
- **No wire or stored format changes,** and no golden moves. If your change would move one, stop
  and ask.
- **Status files are the spec's.** This departs from the implementer's text ("Not yours: any
  `docs/design/`"), because the request has each bug's status line leave with its fix: a task
  edits exactly the status file its spec lists, as it lists, and no other design text.
- **A status file's form** is `DESIGN.md`, 2: its date line is today's (`date -u +%F`), and it
  holds only the gaps as they stand after your change, never history. Where an entry names
  several gaps, remove only the words for the one closed.
- **Citations.** Before pushing, run `python3 -I -B tools/citations/citations.py --base
  origin/main` from the repository root and fix what it lists in your files.
- **Comments say what is true now,** in the design's words: no "no longer", "was" or "formerly"
  for something gone.
- One commit per coherent step, each saying why. Push your branch.
