# Plainer panels: standing rules

Every task of this plan holds to these.

- **Nothing else changes** (the request, How). `--json` and its digest, the
  browser and terminal views, `WaitsOn`, the wire and the engine's documents
  stay as they are. Every fact these tasks print already reaches the client.
- **The lowest tier that shows it:** `temper-client-tools`' lib tests read
  `task show` as they do today, with the fixtures in
  `temper/crates/temper-client-tools/src/tests/fixtures.rs` and its
  `fixtures/`. No new fake and no new harness.
- **The form of a hold is the user's.** The request's `Left open` gives
  `held` alone or `held: REASON` to the person who wrote the draft. No role
  chooses it, and no default stands in for it. Only `held-waiting` reaches it.
- **Shapes** that cross a task are in
  `temper/docs/plan/plainer-panels/notes/contracts.md`.
