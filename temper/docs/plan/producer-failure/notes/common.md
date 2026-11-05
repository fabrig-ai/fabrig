# producer-failure: standing rules

- Reproduce first: each task's first commit, or its first run, shows the failing
  test before the fix. The report says what the test showed. Where a cause in the
  request or in the spec turns out wrong, say so in the report.
- fabrig/ui has priority: no file under `temper/crates/temper-client-*`,
  `temper/tools/gallery`, `temper/tools/journeys`, or `temper/docs/design/client/`.
- No protocol or store change: no schema, no generated file, no golden. A task
  that finds it needs one stops and asks.
- `repair-green` also changes `temper-engine-domain-forge-change` and the forge
  world. Its trunk may land first: fab merges; keep edits small and local, add no
  reordering or reformatting of untouched code.
- Budgets and timeouts are the user's
  (`skein/docs/foundation/testing-strategy.md`, 8.1).
- Design is written by this cut. Implementers leave `docs/design/` and
  `docs/help/` alone, and report what the code taught.
