# root: status

2026-11-28. Where jig's code lags or departs from [root.md](root.md).

- **Not yet:** 8, 12: `ops`'s root still serves the start's phases, in `restart.rs` [in flight: change 22494]
- **Not yet:** 8: the test root still serves the start's phases [in flight: change 22499]
- **Not yet:** 8: temper's root still serves the start's phases, in `restart.rs` [in flight: change 22501]

- **Departs:** 3, 5, 11: the test root is not in `ops`'s shape: its
  `Domain` keeps `now`, `store_answers`, `busy` and `public_reads` beside
  the journal between steps, its `release` steps released callbacks
  before draining `now`, and its
  `Now::RunPrepared`, `Now::RelayBusy`, `Now::DropCall` and held fleet
  events run otherwise than `ops`'s. Aligning them moves when an event
  runs, and what it keeps is the user's to decide, and they deferred it on
  2026-11-06 (section 14; the code's side is
  `jig/docs/development/debt/D004-ops-root-jig-wiring.md`).
  11, 12: `ops`'s root passes a triage's delegate batch through
  `jig-ops-protocol-observability` in the step, rather than handing it
  out, because a hand-out would commit the connector's rows and the
  procedure step apart (the code's side is
  `jig/docs/development/debt/D004-ops-root-jig-wiring.md`).
  3: the user's request of 2026-11-05 has temper's consumers use `Class`,
  but `temper-engine-domain/src/admission.rs`, `completes`, keeps its yes
  or no, as the section allows, reading `LinkUp`'s `new_work`, because
  `Class` would be a new dependency and would change the public
  `completes`, whose callers read a `bool`.
