# hearth-speed-2: the plan

hearth's focused suite under its 30 CPU s, aiming for 27 on the queue's figure, by repairs the profile shows and with no build setting changed. The design is `hearth/docs/design/domain/testing.md`, 9.1 and 11; the rule is `skein/docs/foundation/testing-strategy.md`, 8. The standing rules, the aim, the base commit and the method are in `notes/common.md`.

## Parts and tasks so far

| Part | Tasks | Needs |
|---|---|---|
| `profile` | `profile` | nothing |

The repair tasks and the closing task follow in the next batch. The critical path as it stands is `profile`, about 90 implementer-minutes.

## What was decided

- **The profile goes first and alone,** because the request says what's in it decides the rest, and `hearth-speed` found no profiler and went without one.
- **The method is written out in `notes/common.md`, with its driver,** because `testing.status.md`, 9.1 no longer holds it: the commit that took the `opt-level` lines out cut that text. It's `tools/tiers/design.md`, 7, 7.3 and 10.2 with a caller of its own, as `hearth-speed` measured.
- **The base is `39b490f138`,** the main this plan started from, which every slot can reach. The trunk differs from it only in this plan's files.
