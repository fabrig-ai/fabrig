# bugs-jig: standing rules

Every task of this plan names this note.

- **The plan:** the code's bugs the debt backfill routed to jig (`inputs/bugs-jig.md`): small
  fixes, no reshaping, nothing else from the debt list. A neighbour you notice goes in your
  report, never in your branch.
- **The tree:** read `AGENTS.md`, then `jig/AGENTS.md`. A task's files are in its spec; a
  reader's test that turns red past them is reported under `## Left`, unless the spec's options
  say otherwise.
- **The test comes first:** every fix carries a test that fails on the old code. Write it,
  run it and see it fail for the reason the spec gives, then fix. The report says how it failed.
- **Design:** the sections each spec cites stand on your base; never edit a design document.
  This departs from the implementer's text in one way only: a spec may give you its own line in a
  `.status.md`, to remove when the code matches, and its dated first line to set to the day you
  push (`date -u +%F`). Leave every other entry as it is. If that leaves a **Not yet** line
  empty, remove the line; if no gap is left, say so as `DESIGN.md`, 2 asks.
- **Comments and names** use the design's words, say what is true now, and cite a design section
  at module or type level only.
- **The code's rules:** `skein/docs/foundation/programming-model.md`, section 1 and section 10 for
  step code; test kits and worlds are ordinary Rust under their tree's rules.
- **Citations:** a spec that touches a status file runs
  `python3 -I -B tools/citations/citations.py --base origin/main` before it pushes.
