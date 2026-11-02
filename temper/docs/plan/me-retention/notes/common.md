# me-retention: standing rules

Every task of this plan names this note.

- The tree is temper; read `AGENTS.md`, then `temper/AGENTS.md`.
- Touch no `docs/design/`, no `docs/help/` and no draft. What the code taught goes in the
  report.
- Goldens under `temper/crates/temper-client-tools/src/tests/golden/` are regenerated with
  `SKEIN_CODEGEN_WRITE=1` on the test that owns them, never edited by hand; the regenerated
  diff is read by `git diff` and holds only the lines the spec changes.
- `--json` is not changed: its records keep their numbers as they are.
- No new public items, no new crates, no dependency changes.
