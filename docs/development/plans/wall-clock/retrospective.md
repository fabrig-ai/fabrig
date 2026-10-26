# Retrospective: wall-clock

2026-10-17.

## 2. Outcome

The supervisor ran the README's "Done when" once, from 17:40 to 17:43,
on smith `main` at `92f9494d` (the smoke's arm at `7ef5790a`, which adds
only `STATUS.md`'s `done`). Its commands and outputs are in `done.md`.

| # | Item | Result |
|---|---|---|
| 1 | Every step landed or dropped | Holds: 20 landed; 11.2 dropped, "dropped: 11.1 on par". |
| 2 | smith is on par with Codex at the plan's final `main` | Holds: exit 0, offline, on `parity-a87ba689-20261017T123849Z`. No code commit of the plan landed after its smith arm. |
| 3 | The smoke passes on `main` | Holds: `build HEAD` at `7ef5790a`; `smoke-tools` passed, 1 attempt, 4.3 s, $0.0004. |
| 4 | Each change to what the model is sent has its report | Holds: four reports, none missing or refused. 09.1's and 10.1's print RC's stop, kept by rulings 125 and 128. |
| 5 | Each format moved once | Holds: events 3, transcript 5, charter 2, channel 2, result 4, summary 4, baseline 1. |
| 6 | Both gates within their budgets | Holds: 2,356 tests in 4.992 s and 43 in 5.204 s, against 15 s and 60 s. |
| 7 | Each session's "Done when" | Holds for sessions 01 to 11. |
| 8 | Only the change steps changed the prompt's files | Holds: 03.1, 04.1, and 07.1 to 10.1. |
| 9 | The Don'ts a command can read | Holds: no wall-clock commit in skein, jig or temper; the profile's quantities untouched; no other probe's words in the prompt. |

**The verdict** (statistic, interval, and whether it holds). The margin
is 1.20 for FT, FC and RT, and 1.50 for RC.

| Criterion | Baseline, 06.1, `parity-c10edbe5` | Closing, 11.1, `parity-a87ba689` |
|---|---|---|
| O | 34 [0.763, 0.981], holds | 34 [0.726, 0.967], holds |
| FT | **1.178 [1.110, 1.251], fails** | 1.067 [1.000, 1.135], holds |
| FC | 0.969 [0.883, 1.060], holds | 0.825 [0.763, 0.890], holds |
| RT | **1.397 [1.149, 1.636], fails** | 1.033 [0.892, 1.180], holds |
| RC | **1.407 [1.087, 1.734], fails** | 1.174 [1.042, 1.444], holds |

**`shell-cwd` by arm,** ten attempts each, from the two summaries'
attempts (`responses`, `task_wall_ms`, `priced_usd`):

| | smith, baseline | smith, closing | Codex, baseline | Codex, closing |
|---|---|---|---|---|
| Responses an attempt, mean | 70.1 | 54.2 | 45.5 | 48.3 |
| Task wall, median | 620.0 s | 609.7 s | 443.6 s | 590.4 s |
| Seconds a response, mean | 8.9 | 11.5 | 10.1 | 12.3 |
| Spend, median | $1.552 | $1.412 | $1.103 | $1.203 |
| Passed | 10 | 10 | 7 | 6 |

- **What smith changed:** its responses fell by 23%, and the ratio to
  Codex's from 1.54 to 1.12. That is what causes 1, 3 and 4 aimed at.
- **What the closing run changed:** each response took 22% to 30%
  longer, for both arms. smith's time held at about 610 s, and Codex's
  rose by a third. RT reads the two arms of one run, so it followed the
  response ratio.
- **The fixtures moved the same way:** smith's median fixture wall went
  from 83.6 s to 80.4 s, and Codex's from 72.9 s to 76.3 s.
- **The pass was close.** RT's upper bound, 1.180, is 0.02 below its
  margin. By `rt-power.md`, at ten attempts a smith with Codex's true
  time passes in 60% of runs, and one 5% faster in 84%. The plan
  allowed two closing runs (11.2), and needed one.

**The four change runs,** each smith against its parent, ten `shell-cwd`
attempts an arm. A projection carries the step's ratio onto the
baseline's.

| Step | RT ratio | RC ratio | RT projection | RC projection | Stop |
|---|---|---|---|---|---|
| 07.1, the window's check (cause 2) | 0.815 [0.650, 0.989] | 0.926 [0.698, 1.115] | 1.138 | 1.303 | none |
| 08.1, batched change (causes 1, 3) | 1.014 [0.719, 1.496] | 1.021 [0.839, 1.265] | 1.154 | 1.331 | none |
| 09.1, polls (cause 4) | 1.149 [0.801, 1.358] | 1.393 [0.892, 1.664] | 1.326 | **1.853** | RC, kept (ruling 125) |
| 10.1, ranged reads | 1.142 [0.880, 1.324] | 0.983 [0.876, 1.350] | 1.513 | **1.821** | RC, kept (ruling 128) |

The steps' own RT ratios multiply to 1.083. The closing run measured RT
at 1.033 against the baseline's 1.397.

## 5. What did not

1. **The four change runs took a third of the run, and decided nothing
   the verdict read.**
   - **What:**
     - Rule 11 ran 07.1, 08.1, 09.1 and 10.1 one at a time, each with a
       full `parity-change` against its parent: 9.3 h of the gate lane,
       164 attempts and $69.85. Each step's landing waited on its run.
     - Their RT intervals were 0.34 to 0.78 wide. `causes.md` put what
       each removed at 4% (cause 4) to 22% (causes 1 and 3) of smith's
       time. Three of the four intervals included no change: 08.1's, for
       the largest effect, ran from 0.72 to 1.50. Only 07.1's excluded
       it, at 0.815 [0.650, 0.989].
     - Two runs stopped on RC's projection (1.853 and 1.821, against the
       baseline's 1.734). The line stood 15 and 13 minutes, and two
       designer sessions cost $8.26. Both answers called the move noise
       and kept the step.
     - The projection multiplies each step's ratio onto the baseline's,
       and each parent arm measures again the code the step before it
       measured (`later.md`'s row, from the designer's answers). At 10.1
       it put RT at 1.513 and RC at 1.821. The closing run measured
       1.033 and 1.174.
     - What did move was the counts each step reported under rule 13:
       smith's responses on `shell-cwd` fell from 70.1 to 54.2 an
       attempt.
   - **Cause:**
     - Parity's proposal 4, deferred to this design, became `benchmarks.md`
       16.6: a change to what the model is sent runs `parity-change`
       against its parent, and stops when a projection passes the
       baseline's interval.
     - The design checked RT's power for the verdict (`rt-power.md`), but
       not for a change run, whose two arms are both smith and whose
       effects are a fraction of the verdict's margin.
   - **First visible:** 06:22, 08.1's RT interval [0.719, 1.496]; 09:17,
     09.1's stop. **Acted on:** the designer kept both stops (09:36,
     12:28), and left the projection's revision to `benchmarks.md` 16.6,
     before the next plan whose aim is a parity verdict (`later.md`).
8. **Claude Code cost the same per hour as in parity, and its sub-agents
   read the most.**
   - **What:**
     - The supervisor cost $95.44: its nine sessions $36.72, and 20
       sub-agents $58.72 (8 on Opus $21.27, 12 on Sonnet $37.45 at Opus's
       rates). That is $3.3 an hour, against parity's $3.4.
     - **By purpose:** 8 ready checks $25.54, 3 re-checks $7.02, 9 hourly
       checks $26.16.
     - **Two sub-agents ended with no report,** stopped at a handover
       after 17 and 20 requests; the first is item 4's re-check.
     - **The Sonnet sub-agents ran larger contexts than the Opus ones:**
       peaks of 87K to 246K (median 169K) against 125K to 160K (median
       137K), and $3.12 each at Opus's rates against $2.66.
     - **The checks paid off:** all nine sub-agent checks led to a steer or
       a ruling. The 19 quiet checks cost $3.00 in all (parity's proposal
       13).
   - **Cause:** the metric prices Sonnet as Opus, so it cannot show the
     saving the rule aimed at. A sub-agent's brief bounds its
     report, not its reading.
   - **First visible:** in this retrospective. **Acted on:** not.
10. **A premise the plan checked by a command was wrong again, and three
    Shapes were.**
    - **What:**
      - **Ruling 107 (plan defect):** "`memory` stays 2 GiB while the
        larger of the two worst cases the profile's memory tests print
        … fits it". `planning-commands.md` measured that quantity, as
        parity's proposal 10 asks. But those totals miss the `exec`
        faces, and the largest startup worst case was 2,193,357,213
        bytes, over 2 GiB. Ruling 108 met it again in 04.1's smoke.
      - **Rulings 109 and 118 (plan defects):** `AttemptRow` lacked the
        fields 02.2's Shape read, and 08.1's transcript Shape missed the
        channel's codec.
    - **Cause:** the command checked the premise's own words, not the
      bound the smoke enforces (`service.worst_case` against
      `declared.memory`).
    - **First visible:** 13:28. **Acted on:** rulings 107 and 108 at
      13:30 and 13:34.
## 6. Decisions taken during the run

Rulings 101 to 128, ten steers, and the two messages. The
supervisor marked each decision in force in `done.md` with its design
commit, or "not in the design yet".

| Rulings | What | Reached the design | Proposal |
|---|---|---|---|
| 101, 104, 106, 110, 111, 112, 116, 117, 119, 121, 123, 127 | Ready checks and re-checks: Shapes, Files, lines and commands against `main` | The session files only; not design matters | — |
| 102, 105, 107, 108 | The session's line as a named field; `memory` raised to the next 64 MiB multiple above the largest startup worst case | Not design matters; 107's value is in the code and the plan | — |
| 109, 118 | Plan defects: `AttemptRow`'s fields; the allowed names in the session's record | The plan only | — |
| 113 | A request never answered (`maybe_sent`) reruns the suite from the start; README rule 9 amended | The plan only; the design question is in section 9 | — |
| 114, 115 | The baseline's misses: 07 to 10 aim at FT, RT and RC; no step added | `65a54642`, plan files only, "No design text changes" | — |
| 120 | A poll waits as asked only beside `job` calls alone; the task role counted as written in the host part | Yes: `2c213b1b` (`session.md` 5, 10; `tools.md` 5; `llm.md` 3; `limits.md` 4; `shell.md` 6.6) | — |
| 125 | 09.1 kept: RC's move is noise | Yes: `7f3d4f4b` (`benchmarks.md` 16.6) | 1 |
| 128 | 10.1 kept: its RC projection carries 09.1's ratio | Not in the design yet: `db860283` changed the plan and `later.md` (the revision of 16.6) | 1 |
| 103, 122, 126 | Test debts deferred to `later.md` rows | Not design matters | 5 |
| 117, 124 | README rule 18 grows: shared setups, then any fixture the code holds | Not design matters | 5 |
| Steer 03:29, point 1 | A re-check never holds a start (`ce02728`'s 4) | In development.md on `main` | 4 |
| Steers 13:12, 14:07, 15:08, 16:33, 03:29, 06:46, 07:56 | Test shape: named sizes, records, setup, bounds, rows, fixtures; the summary copied into `benchmarks/summaries/` | Not in the design yet (process) | 5 |
| Steers 03:54, 12:27 | Hold 08.1's live run for the answer; a dropped need counts as met | The plan only | — |

**Decided in the decision during the run:**
- At 16:25 on 10-16, every decision reserved for the user (`decisions.md`
  11.1 to 11.5, 12.5.1 to 12.5.3), and that questions for the user go
  under "For the user";
- At 17:51, sub-agents on Sonnet and the supervisor's handover at 150K.

**Still with the designer** (`later.md`): the projection's revision in
`benchmarks.md` 16.6; FT's output in writing responses; where
`CARGO_BUILD_JOBS=2` comes from; the `edit-format` experiment; the
`history` recalibrations. Nothing waits on the user.

## 9. Debts left in the code

For smith's `tech-debt.md`, for skein's kit, and for the designer.

**smith:**
- **44 panics added outside tests:**
  - 42 `.expect(`: `smith-domain-session` 16 (`session.rs` 10,
    `batching.rs` 6), `smith-protocol-llm/src/guard.rs` 8, `benchmarks/src`
    8, `smith-domain-tools/src/jobs.rs` 3, and 2 each in
    `smith-domain-run`, `smith-domain` and `smith-protocol-channel`.
  - 2 `unreachable!` in `smith-protocol-machine/src/process/jobs.rs:43`
    and `:57`, each a state the types allow.
  - `smith-local-shell/src/settings/resolve.rs:88` expects "fits four
    kibibytes", but `TASK_ROLE_BOUND` is gone and nothing enforces it.
    `role.rs:28` asserts a bare `<= 4096`.
- **Files past 800 lines grew:**
  - `benchmarks/src/probe.rs` 823 → 967, `summary.rs` 822 → 871,
    `batch.rs` 1,004 → 1,022 and `run.rs` 1,223 → 1,231;
  - `crates/smith-domain-session/src/session.rs` 3,630 → 3,665;
  - `benchmarks/tests/codex_adapter.rs` crossed 800 (809).
- **Two tuples that dodge clippy's argument limit:** `probe.rs:638`
  (`selection`) and `probe.rs:400` (`arms`).
- **Untested branches in `guard.rs`:** `from > messages.len()`
  (`:23-25`), a measurement's error (`:63-66`), and the diagnostic's clip
  (`:104-106`).
- **Tests:**
  - `crates/smith-domain-session/src/tests/batched_change.rs:110-162`
    holds six unlike rows in one test. It branches on their labels
    (`:139`, `:156`, `:157`) and leaves tickets 9, 10 and 11 bare.
  - `a_compaction_carries_no_guard` (`src/tests.rs:3033`) first asserts
    that an ordinary prompt carries one.
  - Copies:
    - 14 `llm::Prompt` literals in 10 files;
    - the `Window::fresh` setup 4 times in `tests/window.rs`;
    - `parity.rs`'s wall loop 6 times, with `CHANGE_WALL_MS` defined 6
      times;
    - the observer's `Limits` setup 5 times;
    - the `JobAct::Hasten` arm in three world job tables;
    - `tests/session/src/world/llm.rs:26-31`, which types the guard's
      diagnostic again.
  - `str::replace` on recorded streams, with no count:
    `benchmarks/src/tests/comparison.rs:235` and
    `benchmarks/tests/summary_resources.rs:62-63`.
- **Memory's margin:**
  - 21,077,507 bytes under the new 2,214,592,512 at 03.2, less 04.1's
    205,248. No test holds it.
  - `settings/tests.rs:498` is still named `…_fit_two_gibibytes`.
- **Unchanged from parity's list:**
  - the six placeholder arms in `flight.rs`;
  - `tests/machine/tests/jobs.rs:132`, two outcomes in one test;
  - the flaky `a_second_run_resumes_the_chat_from_its_files` (140 passes
    here, no failure);
  - `tool-output/timeout`'s classification;
  - `calibrate`'s rewrite of the whole file.
- **From `later.md`:**
  - `history` and `history/small-window` recalibrated;
  - 08.1's one-sided bounds and its digits written by hand;
  - `memory_returns.rs`'s sizes;
  - 09.1's typed `edit()` and `read()`.

**For the designer:**
- **`benchmarks.md` 16.6's projection** counts each landed code twice
  (`later.md`), and is proposal 1's subject.
- **A suite stopped on one unanswered request** reruns whole:
  - 06.1's cost 3.04 h and $8.81 at attempt 53 of 68.
  - Whether a run may complete only the attempts it lacks is
    `benchmarks.md` 8.2's and 16.5's question.
- **The arms' spread:**
  - Codex's `shell-cwd` median went from 443.6 s at the baseline to
    590.4 s at the close, and each arm's seconds a response rose 22% to
    30%.
  - A verdict across runs reads within-run ratios, and those carry
    Codex's spread.
- **smith's `STATUS.md`, section 2,** still names "the wall-clock plan"
  as what builds ten rows that it has built.
- **jig's and temper's `STATUS.md`** lack `later.md`'s repin row:
  transcript 5, events 3, `Purpose::Ordinary { guard }`, `memory`.

**Workspace:**
- **jig and temper fail against smith's `main`,** with 4 and 5 errors in
  their own code, until their repins take this plan's formats.
- **smith's skein pins** were repinned by the render plan (`b415f737`)
  three minutes after the run.
