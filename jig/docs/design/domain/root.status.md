# root: status

2026-11-05. Where jig's code lags or departs from [root.md](root.md).

- **Not yet:** 2, 5, 6: `smith-inline-agent` composed in the engine's
  root. 3: temper's root lists `LinkUp`'s variants itself
  (`temper-engine-domain/src/admission.rs`, `worker_completes`), rather
  than asking `new_work`. 5: temper's root spells the job's assignment
  and a settled call's relay (`temper-engine-domain/src/link.rs`), which
  `jig-domain-hub-link` holds. The code's side of both is
  `jig/docs/development/debt/D004-ops-root-jig-wiring.md`.
- **Departs:** 3, 5, 8, 11: the test root is not in `ops`'s shape: its
  `Domain` keeps `now`, `store_answers`, `busy` and `public_reads` beside
  the journal between steps, its `release` steps released callbacks
  before draining `now`, its restart puts loads in `now`, and its
  `Now::RunPrepared`, `Now::RelayBusy`, `Now::DropCall` and held fleet
  events run otherwise than `ops`'s. Aligning them moves when an event
  runs, and what it keeps is the user's to decide (section 14; the code's
  side is `jig/docs/development/debt/D004-ops-root-jig-wiring.md`).
  11, 12: `ops`'s root reads a value it carries: it decodes a triage's
  delegate batch (`jig_protocol_tools::batch`, `translate.rs`) and appends
  the alert numbers to its words, rather than handing it out (the code's
  side is `jig/docs/development/debt/D004-ops-root-jig-wiring.md`).
