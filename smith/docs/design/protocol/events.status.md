# events: status

2026-11-09. Where smith's code lags or departs from [events.md](events.md).

- **Not yet:** 3, 4 (domain/run.md, 8.5): the `tools` record has no
  `save`, and no call event names a save's delivery.
- **Not yet:** 3.2: no producer writes `containment: "trees"`. The
  vocabulary has it (`smith-events`' `Containment::Trees`), and every
  process writes `groups`, since commands run in process groups
  (agent.md, section 3).
- **Departs:** 6: `smith-events/src/read.rs`, `tokenize`, drives a
  tokenizer over one whole line by hand, with its own step bound, where
  `skein/docs/design/json.md`, 5.3 gives one whole buffer to `parse` or a
  `Feed`, and the user's request of 11-04 asked that no step crate
  outside skein-json do so. No reason is recorded, and no debt item
  holds it.
