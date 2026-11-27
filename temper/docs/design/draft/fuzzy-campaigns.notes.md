# fuzzy-campaigns: notes

The reasons behind [fuzzy-campaigns.md](fuzzy-campaigns.md), by the draft's sections. The survey they cite read main at c8df34634f on 2026-11-24, statically and through git history, and ran no test. Its counts are a static reading's: it counted test functions under the fuzzy profile's selection, and calls written directly in their bodies. They are given as orders of size, not as figures a script reproduces.

## 1. Why the gate keeps its seeds

- **A fresh seed's red is no head's.** It finds a fault the gate's seeds didn't reach, old or recent, and a gate that turned red on it would hold a change that may have broken nothing. The gate's verdict is the head's alone because its seeds and its kernel are its own (`skein/docs/foundation/testing-strategy.md`, 8).
- **The figures stay readable.** The gate judges instructions against main's ledger (`tools/tiers/budgets.md`, 8). A seed that changed each run would move a sweep's work with the seed, and a layer's growth would no longer be the change's.
- **The user's word of 11-20** (`test-budgets.md`, 4): fixed at the gate; a sweep of fresh seeds after landing.

## 2. Why the fuzzy suite stays at the gate in full

- **Its yield:** the survey found twelve product bugs caught while sweeps were written. It also found four later failures in others' changes, caught by fixed seeds at the gate. That is a lower bound, since a goal repairs its own red in its change, and its commit seldom says a fuzzy seed caught it:
  - 4e033f4d08, jig: a product fault, a repeated act's kept answer left pending;
  - 2dbdf9d2f6, jig: a cost regression, timers invalidating frames hundreds of times;
  - eeb74e0181, temper: a product fault, a landing row decoded with both holds, and a faulty referee beside it;
  - 030ac25aa4, temper: a product fault, a landing's cleanup restoring a departed member's prefix.
- **Fixed seeds still explore** wherever a change moves the code beneath them: the same seed reaches other states.
- **Its cost:** about 574 fuzzy functions at the gate, and replaying ceilings of 754.7 fuzzy CPU s against 1351.3 focused (`tools/tiers/budgets.toml`, at c8df34634f, its moves applied and the repository's checks counted in the focused figure). The survey found little to drop: two tests registered twice (`fuzzy-campaigns.md`, 3), and a few fixed checks that could share a table with a focused one.

## 3. Why the seam is at the case loop

- **The case loop is outside every draw,** so a case's seed moves every draw beneath it, however each world derives them. Many draw from `Rng::new(seed)`, a fixture picked by `seed % n`, or a seed of their own. The survey found 35 gate bodies that call `Streams::new`, `Sweep::new` or `sweep::each_seed` directly, so a seam in `Streams` alone would miss most of the suite.
- **A new function, not a changed one.** `each_seed`'s callers include a helper serving focused tests (hearth's render properties). An environment-driven `each_seed` would move those, while a sweep that opts in by calling `cases` moves alone.
- **The case knows its test,** so a failing case can name itself, and the campaign can write the command that replays it. A seam inside a world's draws can't.
- **A check that returns its finding** fits jig's core sweeps, which return theirs today. A panic hook names a case that panics without catching it, which skein's programming model forbids.

## 4. Why rounds

- **A round is the unit a runner schedules, stops and reads.** A campaign cut short has its finished rounds whole, and a round is replayed by its seed and commit.
- **A test's case count stays its own,** set for the gate. Only the sweeps that ask for more in a campaign take more (`fuzzy-campaigns.md`, 6). No per-test knob lives in the runner.
- **Its cost is measured,** not forecast. A round runs a process per test, and its fresh draws do other work than the gate's seeds, so the gate's figures are a comparison.

## 5. Why a campaign's finding isn't filed by its runner yet

- **A landing tier's red finding** is blamed on the landing that caused it, by bisection over the tier's range (`temper/docs/design/domain/red-findings.md`, 3), and cleared by a run of its tier in which none of its ids failed again (6 there). A campaign's next round draws other cases, so its test passes, and the finding would clear unrepaired.
- **An ordinary red finding** is filed anew each time. Only flakes are matched and counted by signature (`temper/docs/design/domain/findings.md`, 4). A runner filing each round's cases would file one fault many times.
- **So, by hand first:** the campaign's report groups its cases by signature, and the user routes them. A finding filed from it follows findings' rules, closed once its fixing landing is cleared (`findings.md`, 7). A source of its own, with matching and closing rules, comes with the runner.

## 6. Why a generated agent's case is pinned as its capture

A seed replays to the same case only while the code that draws from it stands still. A world's draws change rarely. A generator's menu and weights change whenever the class it draws grows (`fuzzy-campaigns.md`, 8), and each change gives its pinned seeds other agents. The capture of its answers, keyed by request rather than by draw, is what the case was, and it stays a scenario whatever the generator becomes.

## 7. 11-26's crash

- **What happened:** the agent shell panicked in three runs on dev.fabrig.ai, in `smith-domain-session`'s accounting, on a call's result with no room reserved for it.
- **The trigger,** after a resume: the model batched a poll of a job from before the opening, an edit and a read. The tools answered the stale poll invalid at their entrance, which failed the batch's write. The session then widened the batch to the read without reserving its room.
- **The fix,** a0f6f9d91b: a batch's calls are fixed when it starts. Its unit test, `an_unknown_job_does_not_widen_its_read_batch`, names a job never started, with no park or resume. So the shape needs only a call answered invalid at the tools' entrance, before a write, with a read past the batch.
- **Why smith's session world didn't draw it:** it draws batches of offered tools over 1000 seeds, but its menu's five argument bodies hold neither `job` nor `yield_seconds`. A body with a field missing doesn't decode, and the session forecasts such a call when it reserves the batch's room, so the call never reaches the tools' entrance. Only an unknown job, or a yielding shell while the jobs are full, is answered there (`smith-domain-tools`' `jobs.rs` and `kit.rs`). No seed of that world could reach the shape. That is why `fuzzy-campaigns.md`, 8 asks a repair to widen its world, and why the misbehaving agents draft starts with those two bodies. Other worlds weren't surveyed for it.
