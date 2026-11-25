# records: status

2026-11-25. Where jig's code lags or departs from [records.md](records.md).

- **Not yet:** 1: the hello lists live attempts and those with retained
  answers, but omits an attempt with only retained turns or parts.
- **Not yet:** 1: a call replayed after a redial carries the original
  frame's deadline duration, not the time left at replay.
- **Not yet:** 1: `FactValue` does not carry a tool begun with its call's
  name, tool and input. `CountKind.tool_started` is only a count; the
  inline worker gap remains `owners.status.md`, 1.4.
- **Not yet:** 2 and 5: the engine fits a start to the room at its own
  layout before converting it, and fits nothing again after: a start
  that would fit at the worker's layout once cut is refused as invalid,
  and one that grows past the room is refused without the cuts that
  would fit it.
- **Not yet:** 3: on redial, the application's worker emits its hello and
  replays its retained answers in one step, before forwarding the hub's
  `draining` notice.

