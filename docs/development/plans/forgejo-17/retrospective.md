# Retrospective: forgejo-17

2026-10-16.

## 2. Outcome

The supervisor ran the README's "Done when" once, from 16:03 to 16:12,
in `~/src/rust/temper` at `origin/main` `294de1303`. Its commands and
outputs are in `done.md`.

| # | Item | Result |
|---|---|---|
| 2 | The pin | Holds: `binary.rs` names 17.0.0 and 13.2.0. Nothing under `crates`, `testing`, `tools` or `tests` names 16.0.5 or 12.13.2. |
| 3 | The conformance's facts are v17's | Holds: the header names `17.0.0` and `v13.2.0`. The four `fails` are the base's, and there are no others. |
| 4 | The 11 facts of `forge.md`, section 13 | Holds: prints 11. The waiting job's fact is in `later.md`. |
| 5 | The component's facts | Holds: no `fails` or `unchecked`, and 40 `holds component-`. The 40th is 04.2's. |
| 6 | git's facts are v17's | Holds: both facts hold on 17.0.0, with git 2.53.0. |
| 7 | The answer and the brief name the failed step | Holds: `api.rs:156` has `step: Option<Step>`, and `Job log tail` is gone. |
| 8 | `STORE_VERSION` did not move | Holds: `JobAnswer` changed in place. |
| 9 | The process tier on v17 | Holds: 6 tests run, 6 passed, in 42 s. It needed temper-01.s1. |
| 10 | The plan's commits | Holds: the 9 planned steps and `temper-01.s1`. skein-00.s1 landed in skein (`267730eb2`). The empty line is the rulings' commits. |
| 11 | The test-time aims | Holds: default −0.054 s, fuzzy +0.441 s, against the reference at `fb381950e`. |
| 12 | No budget raised | Holds: `.config/nextest.toml` is untouched. |
| 13 | Nothing else checked | Holds. |

The reference for item 11 was taken at the parent of `b6a69057c`, the
designer's answers to the planning questions. Those answers carry `Plan:
forgejo-17`, so they are the plan's first commit by the README's
command. They hold no code, so the reference is the base the item
meant.

## 5. What did not

1. **The process tier was never run at the base.**
   - **Cause:** at 12:21 the planner judged that the process tier "can't
     be the proof here", having no CI story.
     - It prescribed the tier anyway, as decision 10 and "Done when"
       item 9.
     - Writer 01 reported at 12:56:47 that the planner should run it at
       the base. The planner went on "as scoped".
     - After the user's 13:01 message, the planner dropped it, with the
       sketches' compile.
     - 5.4 asks for every command the plan prescribes, at the base.
       f6f01026f was in the base, and the run with v16.0.5 was one
       command away.
   - **First visible:** 14:10:01, 01.1's proof. The author reported
     34 s after the failure.
   - **Acted on:** 14:21 (ruling 101) and 14:50 (ruling 104). The
     supplements landed at 15:09 and 15:22.
2. **The sketches were not compiled under clippy.**
   - **Cause:** at 13:18:58 the planner left the compile to the ready
     checks, "as 5.4 allows". 5.4 defers only sketches that name code
     an earlier session lands. Sessions 01, 02 and 05 named the base's.
     - Of the writers, one compiled with pedantic warnings only, one
       with plain `rustc`, and three not at all.
     - No transcript of the planning reads `clippy.toml`'s
       `disallowed-macros`.
   - **Cost:** rulings 102, 105 and 106 corrected:
     - `.unwrap()` and `format!`;
     - an `#[expect]` never met;
     - functions at the 100-line limit.

     The authors still met these in their first clippy runs:
     - functions at 105 and 107 of 100;
     - `matches!`, `Option::map`, `vec!`, `slice::concat` and
       `copy_from_slice`, all disallowed;
     - `arithmetic_side_effects`.

     That was about 28 failed build rounds in all.
   - **First visible:** 14:18, the first ready check.
   - **Acted on:** at each ready check.
11. **Smaller things.**
    - Steer 1 let 02.1 change three lines outside its Files with no
      landed ruling (supervisor.md, section 3). The text sits in
      `log.md` alone.
    - `rm -rf target` was refused by Codex's sandbox in four of five
      sessions of the first set. `rm -r` and `shutil.rmtree` worked.
    - skein 00 queried the code graph by the workspace `AGENTS.md`'s
      example name, `srv-data-git-runner-src-skein`, which was not
      there.
    - The designer's `plan join --help` wrote a role named `--help`
      into `claude.tsv`, and removed it by hand. protocol's proposal 12,
      in the kit, has `plan set` refuse a step that starts with `-`;
      `plan join` has the same gap.
    - The designer found that `plan wake supervisor` refuses a busy
      supervisor. It sent its message with SendMessage instead.

## 6. Decisions taken during the run

| Decision | What | In the design? |
|---|---|---|
| The six planning questions (`b6a69057c`, 14:17:51) | a step's long name carried empty; a job's log within `answer_bytes` less its name; one `STORE_VERSION` move shared by concurrent plans; the gather's cut of a CI section; a log read through bounded by `body_bytes_in`; a job's steps as streamed rows | Yes: `forge.md` 3.2 and 3.3, `store/README.md` 4, `domain/engine.md` 9 |
| Ruling 101 | 01.1's process proof without the two `temper open` tests | Plan only; ended by temper-01.s1 |
| Ruling 102 | 02's Shapes against `main` | Plan only (corrections) |
| Ruling 103 | 02.1's Size at most 370 | Plan only; landed at 352 |
| Ruling 104 | A print's room from its own tree: skein-00.s1 and temper-01.s1 | Yes: skein `054c06e1` (`ui-text.md` 3, 4, 8), temper `551b1d2d` (`client.md` 5.4, 6.8, 10); both `STATUS.md` files name it |
| Rulings 105 to 108 | 03's, 04's and 05's Shapes against `main` | Plan only. 105 noted that translate's read by id drops the attempt. 106 and 107 made 04.1 assert the attempt asked, which no world contradicts (section 9) |
| Ruling 109 | Remove `plan downstream`'s worktrees for the release | Process: proposal 6 |
| Steer 1 | A replaced phrase reads as English; three doc comments corrected | Not in any document. A taste rule for review guidance; one occurrence, not proposed |
| 04.1, in its commit | The raw-room test at 4,096 bytes, not the shipped 65,536 the Test shape named; the scripted peer could not deliver it | No: a test's choice (section 9) |
| 04.2, in its commit | A body of unknown length keeps `Raw`'s one-byte demand; the Shape's larger demand lost a short final chunk | No (section 9) |
| The designer's "Seen, not decided" | temper's client checks no configured memory at start (programming-model.md 6.3); jig's envelope's worst case is past any machine | No: in no `STATUS.md` row (section 9) |

## 9. Debts left in the code

temper keeps no `docs/development/tech-debt.md` yet, as protocol's
retrospective found. These are for one, ordered by what they cost the
next plan.

- **A body of unknown length is read a byte per demand.**
  - In `crates/temper-engine-protocol-forge/src/connection.rs:513-525`,
    `raw_demand` asks `Read::Fill(1)` for a `Raw` or `Tail` body without
    `Content-Length`.
  - 04.2 found that a larger demand lost a short final chunk. Its test
    `a_chunked_tail_read_keeps_the_last_bytes_across_pieces` read
    nothing where it should have read `[99, 100, 101]`.
  - The cause is not found, in temper's connection or in skein's HTTP
    machine. A demand can lose bytes at a body's end.
  - What one-byte demands cost on a long chunked log is not measured.
- **A stale job attempt asserts, rather than answering `MissingJob`.**
  - translate's read by job id drops the attempt (ruling 105).
  - 04.1's `job_answer` asserts that the fake's job is the attempt asked
    (rulings 106, 107). This holds only because no world sets
    `reruns`.
  - Re-runs, `later.md`'s first row, need translate to carry the
    attempt.
- **The client checks no configured memory at start.**
  - This is the designer's "Seen, not decided" (programming-model.md
    6.3). jig's envelope also puts its worst case past any machine.
  - It is in no `STATUS.md` row: a question for the designer first.
- **Two doc comments deleted to fit a Size:** `Range::Head` and
  `Range::Tail`, in `crates/temper-engine-protocol-forge/src/plan.rs:6-9`
  (04.2).
- **04.1's raw-room test runs at 4,096 bytes,** not the shipped 65,536
  its Test shape named, because the scripted peer could not deliver
  them.
- **Over-long tests and files:**
  - test functions over 40 lines:
    - three in 04.1 (52, 43 and 41 lines);
    - 04.2's suffix-range table (48);
    - the system story
      `a_smith_change_with_failed_ci_is_repaired_reviewed_and_landed`
      (120);
  - `tests/engine/domain/tests/forge_routes.rs` (2,875 lines) and
    `tests/engine/forge/client/tests/simulation.rs` (858) are past 800;
  - `testing/temper-fake-forge-domain/src/api.rs` is at 794.
- **The fake forge's Actions answers are not compared with v17's
  recordings** (`later.md`). The fake's jobs, steps and ranges follow
  the facts by hand.
