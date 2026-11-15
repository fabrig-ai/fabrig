# wires: status

2026-11-15. Where temper's code lags or departs from [wires.md](wires.md).

- **Not yet:** 6 and 11: `application` hands over its `Kept` and makes no bridge; it builds the bridge and every kept version's bounds at each call. [in flight: change 3668]
- **Not yet:** 11: the one-process world's mismatch story. The world composes the real local worker and has no seam to make it answer an assignment with a version refusal, so the focused tests of jig's `protocol/channel/wires.md`, 7 cover the path alone.
- **Not yet:** 11: a run adopted across an engine restart with its worker kept, sent a message, a call's answer and a renewed identity after it: the one-process world restarts its engine and worker together (`../testing/README.md`, 2.5).
