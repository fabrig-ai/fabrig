# hearth-speed-2: rules every repair task names

Beside `notes/common.md`, which holds. The repairs' sources are `figures.md`, which the `profile` task wrote, and a survey of the code at this cut: a spec's estimates are judgements from reading, and a measurement beats them.

## The start check

Before any change: the method's figure at the base commit and at your start, twice each, on your machine. Compare the lower of each side. **If the ratio is at most 85%, the aim is met:** your first line is `Done: the aim is already met at <ratio>`, with the figures in the report. You change nothing and push nothing: return to your branch, delete `.measure.py`, and leave `git status` clean, and fab collects the run as one with no commit. That's the request's "stop once the aim is met", and it's why the later tasks can stand in the plan. The same holds when the profile rules out every option you have.

## The profile decides the order

Read `figures.md`: `## What it found` and your family's `## Profile` section.

- A spec's options are in its own order, until the profile says otherwise: do them largest share first.
- **An option the profile shows under 1% of its family's CPU isn't done.** Say so in the report, with the figure. If that's every option, your first line says it and you change nothing.
- **A cost the profile shows over 3% of its family, in files you own, that the spec doesn't list, is yours,** pre-approved on the same terms.
- **A cost outside your files** goes under `## Left`, with its share.
- **Where `figures.md` says `unknown`,** take callgrind on the test you change (`hearth/docs/plan/hearth-speed-2/tasks/profile.md`, "How to profile").

## What a repair keeps

- **The same results:** every state digest, golden, `fixtures::CUTS` and recorded digest is unchanged. If `git status` shows one changed, that's a finding.
- **The same draws:** an `Rng` is drawn the same number of times in the same order, so a seed replays what it replayed.
- **The same tests:** `heavy cargo nextest list` for your package lists what it listed, plus any test your spec adds.
- **Nothing shared that can change.** A value computed once is never written after it's made. Each seed takes its own copy or borrows that value. A fake's or a world's state is never shared across seeds.
- **Memory tests read peaks.** A value computed lazily must not be first built inside a meter's span, or the peak moves for a reason that isn't the code under test. A repair stays out of what a meter measures unless it only makes that cheaper; the spec says where.
- **An oracle stays independent:** it may compute once what it computed twice, and never shares code with what it checks.
- **Step crates keep their rules** (`skein/docs/foundation/programming-model.md`, 10).
- **Code comments say what's true now,** with no history (`AGENTS.md`, 2).

## What goes where

- **`figures.md` isn't yours:** tasks that run side by side would meet in it. `last-round` and `close` write in it, and they run alone. Your figures go in your commits and report.
- **Measured means `Ir` or CPU.** A repair whose saving is lost in the suite's noise lands on its test's `Ir`, quoted before and after, and on the family's figure showing no rise. One with no fall in `Ir` doesn't land.
- **The tests you run** are the end of `notes/common.md`, and each spec's `## Steps` name the packages that read what it changes. `fuzzy` runs alone for a package only where the spec says so.
- **Your report** gives what each repair saved, the method's figure for the whole suite on both sides, the machine, and `## Left`.
