# phase-case: standing rules

Every task of this plan names this note.

- The tree is temper; read `AGENTS.md`, then `temper/AGENTS.md`.
- Touch no `docs/design/` and no draft. What the code taught goes in the report.
- `temper/docs/help/` is the lead's: a task changes a help line only where its spec gives the
  line verbatim, and then runs the help tests (`temper-client-tools` lib, `temper-docs`).
- Goldens under `temper/crates/temper-client-tools/src/tests/golden/` are regenerated with
  `SKEIN_CODEGEN_WRITE=1` on the help tests, never edited by hand; the regenerated diff is
  read by `git diff` and holds only the lines the spec changes.
- `phase_word` stays the canonical lower-case spelling: refusals, the table and the argv
  echo print it.
- No new public items, no new crates, no dependency changes.
