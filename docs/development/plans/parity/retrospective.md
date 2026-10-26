# Retrospective: parity

2026-10-15.

## 2. Outcome

The supervisor ran the README's "Done when" once, at 20:13 to 20:17, on
smith `main` at `9d1119b3` and skein at `ecccbac`. Its commands and
outputs are in `done.md`.

| # | Item | Result |
|---|---|---|
| 1 | Every step landed or dropped | Holds: 58 of 58 landed. The command's `grep -c` exits 1 because it counts 0 open steps. |
| 2 | smith is on par with Codex at the plan's final `main` | **Exits 1, short on RT alone,** which the item, as amended by ruling 146, allows: the plan ends with it failed, naming RT. The summary is the last closing run, `parity-87a8d1e8-20261015T170125Z`. No code commit of the plan landed after its smith arm. |
| 3 | The smoke passes on `main` | Holds: `suite smoke` passed, 1 attempt, 2.2 s. |
| 4 | The calibrations are in their task files | Holds: prints 4. They are `history`, `history/small-window`, `first-request` and `tool-output/background`. `tool-output/timeout` stays uncalibrated (ruling 145, `later.md`). |
| 5 | Both gates within their budgets | Holds. smith: 2,261 tests in 4.926 s, and 42 in 6.094 s, against 15 s and 60 s. skein's last gated landing is skein-ui's, 2,443 in 6.531 s. |
| 6 | Each format moved once | Holds: events 2, transcript 4, result 3, summary 3. |
| 7 | Each session's "Done when" | 11 sessions hold, and 12's is items 2 and 4. Two checks fail on later work, not on a regression: 06's "one role summary" finds two, the second 12.s1's (ruling 146); 09's "no `struct Job`" finds the held-job table that session 10 added (`jobs.rs:26`). Session 09's calls in flight became `Flight`. |

**The verdict across the plan's three `parity` runs** (statistic,
interval, and whether it holds). The margin is 1.20 for FT, FC and RT,
and 1.50 for RC.

| Criterion | Baseline, 12.1, `744e0820` | First closing, `b7b78fb1` | Last closing, `87a8d1e8` |
|---|---|---|---|
| O | 29 [0.88, 1.00], holds | 29 [0.82, 1.00], holds | 29 [0.88, 1.00], holds |
| FT | 0.995 [0.916, 1.085], holds | **1.193 [1.128, 1.260], fails** | 1.090 [1.021, 1.166], holds |
| FC | 0.486 [0.446, 0.530], holds | 0.598 [0.545, 0.654], holds | 0.532 [0.485, 0.584], holds |
| RT | **1.293 [0.909, 1.539], fails** | **1.156 [1.003, 1.581], fails** | **1.326 [1.031, 1.756], fails** |
| RC | 0.642 [0.321, 0.766], holds | 0.606 [0.504, 0.819], holds | 0.778 [0.549, 0.971], holds |

- **RT's upper bound never came below 1.54.** 11.3's own measure of it
  was 1.176 [0.729, 1.599].
- **`later.md`'s last row** carries RT to the next smith plan: a
  diagnosis, then a change that names its cause, then a `parity` run.
  In the first closing run, smith made 75.4 responses an attempt on
  `shell-cwd`, against Codex's 55.0.

## 5. What did not

7. **One design area took eight rulings and is still open.**
   - **What:** whose failure a calibration counts, what decides, and
     what is "not exercised":
     - ruling 112 at 18:53, ruled a plan defect, and 04.s1;
     - 118 at 20:05;
     - 119 and 121 at 20:34, the first design question, and 04.s2;
     - 123 and 124 at 21:11, a gap in 121's own answer, and 04.s3;
     - 126 at 21:51;
     - 145 the next day: `timeout` left uncalibrated, its
       classification defect in `later.md`.

     07.3 spent 59 attempts against the 28 it planned.
   - **Cause:** benchmarks.md 3.1 and 3.2 did not say whose failure a
     check's miss is. Ruling 112 read it as the plan's.
   - **First visible:** 18:51, 04.6's block. **Acted on:** the designer
     was asked at 20:18, 87 minutes later.
8. **Memory sat at its bound, and the plan said it needed no change.**
   - **What:**
     - smith's worst case at startup (`service.worst_case`) passed the
       shipped limit (`declared.memory`) eight times: in 02.1, 08.3,
       07.2, 04.6, `main` itself, 10.2, 10.4 and a dependents' run.
     - Six rulings followed (107, 110, 114, 115, 125, 134). README rule 8
       was amended three times.
     - Each fix removed an over-counted term, proven by a heap test. The
       margin left is 28,783 bytes of 2 GiB (10.4).
   - **Cause:** decisions.md 7.4: "Memory needs no change". No command
     checked it at planning, and no test held the margin.
   - **First visible:** 12:00, 02.1's second smoke. **Acted on:** each
     time, by a ruling; 08.s1 at 19:32.
14. **temper broke on this plan's changes, and that went nowhere.**
    - **What:**
      - From 17:44 every check found temper failing against smith's
        `main` with 10 errors in its own code: `smith_transcript::v3`,
        `Request::Job` and `Turn`'s `jobs`. Each called it "for the
        user, not a steer".
      - It is not in "For the user", and temper's `STATUS.md` has no row
        for it. `later.md` has one.
      - The plan's "Repin: none" rests on decisions.md 8: the user
        reserved jig and temper for their own plans.
    - **Cause:** skein-ui's proposal 8, adopted in `e3b148e`, sends such
      work to the downstream `STATUS.md`. It was not in force here.
    - **First visible:** 17:44. **Acted on:** not; section 9 has the row
      for temper.
## 6. Decisions taken during the run

Rulings 101 to 146, eleven steers, and two decisions of the user's. The
supervisor marked each decision in force in `done.md` with its design
commit, or "not in the design yet".

| Rulings | What | Reached the design | Proposal |
|---|---|---|---|
| 107, 110, 114, 115, 125, 134 | Memory at its bound: setup refusals, a branch's against `main`'s, and terms over-counted | 110 rests on limits.md 2.2 and 3.5, with no design change. 125 is not in the design yet, and contradicts benchmarks.md 4.2, "A red smoke stops the line" | 10 |
| 112, 118, 119, 121, 123, 124, 126, 145 | Whose failure a calibration counts; what is not exercised; `history` version 3; effort and token caps | 121 and 124 yes: smith `62376cb8`, `f242ecaa`. 145's defect is in `later.md` | 9 |
| 113, 116, 117, 120, 122, 127, 128, 130 | Session 10's job sketch: Files, numbering, clamps, the lifecycle across 10.2 and 10.6 | The session files only. 120, derived literals that follow a changed layout, is not in the design yet | — |
| 129, 131, 133, 135, 136 | The verdict's pipeline run live: compaction caps, a retried failure, the setup deadline, the cost, the baseline standing | 131 yes: smith `452c9967` | 2 |
| 139, 140, 141, 146 | The role: its count, its binary arms, 06.2's stop, and the closing runs | 146 yes: smith `7c345bd0`, benchmarks.md 16.5 and shell.md 6.6 | 3, 4 |
| 101, 102, 104, 105, 106, 137, 142, 144 | Prices at the resolved window; a lint exception; Files; a drift test's ceiling; ready checks; two panics found live (11.s1, 11.s2) | Not design matters | — |
| 132 | The flaky resume test retries once | Not a design matter; `later.md` has the test | Section 9 |
| 138 | No coordinator waker; a log by its queue time | Not in the design yet | 5 |
| 143 | The user doubled every cap: 680 attempts and $100 | Not a design matter | — |
| Steers 17:44, 18:44, 00:31 | One behaviour per test; named constants and fields | In development.md 5.6 and 14.3 before the run; named struct fields are not | 12 |
| Steers 23:33, 00:31 | Yield on a gate-lane run, with its task id; a one-attempt smoke waits | Close to `main`'s the implementation rules, not the same | — |
| Steer 06:33 | A new increment past 150K goes to a fresh thread | In part: `main`'s 7.3 holds it for a resumed session only | 6 |
| Steers 20:44, 22:29, 07:32, 08:15 | 10.1's world proof with 10.6; hold 12.1; 11.s1 first; the caps | The plan only | — |

**Decided by the user during the run:**
- leave to delete the old benchmark's `target/` directories (17:31,
  ruling 109);
- the caps doubled (08:13, ruling 143).

**Still with the user:** "For the user" holds the compaction
threshold's evidence (12.2): keep 4/5, or change it. Nothing waits on
it.

## 9. Debts left in the code

For smith's `tech-debt.md`, and for skein's (its rows are for the
designer).

**smith:**
- **Memory's margin is 28,783 bytes of 2 GiB** between
  `service.worst_case` and `declared.memory` (10.4, `log.md` line 3813).
  The next term added will refuse at setup. No test holds the margin
  (proposal 10).
- **Six placeholder arms** in `crates/smith-domain-tools/src/flight.rs`
  (lines 430, 451, 482, 537, 582 and 611):
  `unreachable!("the tools issue no yield or job act yet")`, though the
  tools now issue job acts through the job table. An arm of the same
  words, in `smith-inline-agent`'s `translate.rs:513` (from 09.2),
  panicked in 11.3's live run, and 11.s1 replaced it (ruling 142).
- **48 `unreachable!` arms added in the run,** 33 of them outside tests:
  - `smith-domain-tools`: `flight.rs` 8, `jobs.rs` 4, `kit.rs` 3,
    `authority.rs` 1;
  - `smith-domain-run`: `land.rs` 4, `run/jobs.rs` 4, `agent.rs` 2;
  - `smith-agent-service`'s `calls/jobs.rs` 2,
    `smith-protocol-machine`'s `process/jobs.rs` 2,
    `smith-protocol-llm`, `smith-domain` and `benchmarks/src` 1 each.

  Each is a state the types allow.
- **Files past 800 lines grew:** `benchmarks/src/probe.rs` 634 → 823,
  `benchmarks/src/batch.rs` 954 → 1,004,
  `crates/smith-domain-session/src/session.rs` 3,578 → 3,630, and
  `crates/smith-agent-service/src/service.rs` 2,905 → 2,920.
- **Tests:**
  - `tests/machine/tests/jobs.rs:132`,
    `a_cancelled_held_job_answers_cancelled`, tests two outcomes (the
    17:44 steer).
  - Helpers defined again: `run` in 19 files, `finish` in 12, `read` in
    14. A 25-line setup is copied into 5 files, and a fifth copy of the
    Start…Started sequence.
  - Bare numbers: a 4096 in place of `TASK_ROLE_BOUND`, and 55 and 91
    (the 06:26 and 08:26 checks).
- **Two tuples that dodge clippy's argument limit,** and 04.1's branch
  on a probe's name (the 00:24 and 04:24 checks).
- **The design:** benchmarks.md 4.2's "A red smoke stops the line"
  against README rule 8 as amended by ruling 125.
- **From `later.md`:**
  - RT on `shell-cwd`;
  - `tool-output/timeout`'s calibration and its classification defect;
  - a `finish` without `fields`;
  - `calibrate`'s rewrite of the whole `task.toml`;
  - the flaky `a_second_run_resumes_the_chat_from_its_files`;
  - six task files to recalibrate before `suite smoke` is relied on.

**Workspace:**
- **temper fails against smith's `main`** with 10 errors in its own
  code: `smith_transcript::v3`, `Request::Job` and `Turn`'s `jobs`.
  - temper's `STATUS.md` has no row for it. skein-ui's 8 sends it there
    as "Not planned": the transcript's version 4, and the tools domain's
    new `Call` and `Outcome` variants, for temper's next repin.
  - jig, not on this machine, takes `smith-domain-tools` the same way.
- **temper's checkout here** is 268 commits behind origin.
- **The stashes** smith `96cdd17c` and temper `697a9805` hold only
  `plan downstream`'s regenerated `Cargo.lock`s. Copies are in
  `logs/coordinator/`.
