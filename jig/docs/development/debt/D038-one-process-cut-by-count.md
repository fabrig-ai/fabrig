# D038: The one-process world places its power-loss cut by a step count

- **Kind:** tests
- **Where:** `tests/one-process/tests/suite/one_process.rs`, `tests/one-process/src/referee.rs`
- **Seen:** 2026-10-18 at `86971be`, in jig's tech-debt list ("Fixtures that move with the code": the one-process world's power-loss cut, then 128)
- **Seen:** 2026-10-31 at `c30cf6a03`, in the jig code-quality review (A11 F17: now 195)
- **Checked:** 2026-11-03 at `41d63da8c3`
- **Value:** low (the cut stays where the story needs it)
- **Size:** S
- **Status:** open
- **Paid when:** the power-loss story cuts on an event (as the signal stories do), not on a numbered submission.

## What

`const BEFORE_AGENT_OUTPUT_CLOSE: u32 = 195` (`one_process.rs`, line 42) is the 195th
submission, passed to `crash(skein_world::Cut::PowerLoss, …)`. It was 128 on 10-18: any
change in how many steps a run takes moves it.

## Why it matters

The test can pass while cutting somewhere else than its name says.

## What pays it back

Place the cut on the event the name describes, as the referee's signal stories already
place theirs.
