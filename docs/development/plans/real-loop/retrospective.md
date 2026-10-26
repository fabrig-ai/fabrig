# Retrospective: real-loop

2026-10-20.

## 2. Outcome

`done.md` holds each command's output: the Codex helper's run at temper `9f444b30c` (15:47:40), then the supervisor's run of item 5's second part at `8f2228ce8` (16:03:56).

| Item | Result |
|---|---|
| 2. Each session's own "Done when" | **Met but for three commands, on other work's lines or the plan's own literals** (ruling 150). |
| | Session 13, second command: `cue: Some(` was never written by any commit (`git log -S` finds none); the cue is an assignment, `setup.checks.cue = Some(…)`. 13.5 renamed `@coding Small fix` to `@coding Fix`, "to keep the coding brief within the title bound". |
| | Session 14, seventh and eighth commands: each found two lines, not one. The second lines are browser-1's: its `[profile.browser]` timeout (`00e77f2cd`) and its `skein-ui-bundle` opt-level (`7719ba16a`). |
| | The other 33 session commands passed. |
| 3. No test this plan adds is named with `_and_` | **Met for this plan; the command failed** (ruling 150). It printed ten names, all from the accounts-2 owners' commits (`3ceceb8ff`, `a438acc2c`, `2a529b4f2`, `2a343d21e`), in directories this plan shares. None of this run's 113 test functions has `_and_`. |
| 4. The process tier within 3 minutes | **Met.** 24 passed in 107.8 s (base 152.9 s). The tier fell from 152–159 s on 10-19 to 105–109 s on 10-20. It fell while the run was paused and landed nothing, so other work's landings caused it. |
| 5. The test-time aims | **Missed by 0.04 s for the other worlds** (ruling 151, for the user). The rest was met. |
| | Suites, medians of three: focused 14.44 s (aim 37 s), fuzzy 19.65 s (aim 22 s); the base was 9.90 s and 11.86 s. |
| | Worlds below the domain, medians of the last three of four idle runs: the other worlds 4.04 s against 4 s; the real loop 3.23 s against 6 s; no story past 1 s. |
| | By world, from the base (ruling 112) to the end: client 0.92–0.99 s → 1.62 s (this run's 14 stories, 0.47 s); one-process 1.54–1.67 s → 1.80 s; forge protocol 0.61–0.67 s → 0.31 s (forge-kit); terminal 0.16 s → 0.17 s; git 0.15–0.18 s → 0.14 s; reconcile 0.26–0.30 s → gone to skein. |
| | The slowest of the other worlds: the one-process adoption story, 0.899 s, 22% of the sum. 13.s1 had cut it from 1.008 s to 0.724 s; it rose again during the run, for a cause not traced. |
| | 13.5's landing story, 4.1 s, went to `fuzzy_landing.rs`, as the README's rule says. |
| | The first measurement could not run: forge-kit's temper-07.1 (`4b074713d`) had removed `temper-forge-setup-reconcile-world` during the run. Ruling 150 dropped it from the list. |
| 6. Nothing else | **Met.** |

What the plan leaves for later:
- **The other worlds' 0.04 s,** with the adoption story at its cause (ruling 151): work for temper's owner.
- **testing.md 6.1 still names reconcile's world** among the 4 s worlds: `later.md`'s last row.
- **The rows the plan added to `later.md`:** an existing-only private open in skein's io (ruling 102), and jig's reference service freeing a stream at a peer's end (ruling 123).
- **The real loop's next stories,** about fifteen `later.md` rows, each one smoke path.

## 5. What did not

7. **Test shape took six rulings, and the gate saw only part of it.**
   - **What.** The rulings, each from an hourly check:
     - 117, from the check of 14:50: act once and assert after;
     - 126, from 15:46: four points;
     - 133, from 16:52: named positions, with 09.s2's gate;
     - 137, from 17:57: named counts, and no `_` slice patterns;
     - 142, from 12:27 on 10-20: with 13.s2's gate;
     - 146, from 13:33: a module of its own, and a named deadline.

     About 15 steer points. Assertions on the way were sent three times before 13.s2's gate checked them, and bare positions twice before 09.s2's.
   - **What is left.**
     - Three `_` slice patterns over calls (`tests/real-loop/tests/lost.rs:30`; `crates/temper/tests/process.rs:61`, `:103`).
     - At least eight expectations computed by the code under test, two tables that branch on a row, five one-sided bounds and five assertions before the act. Most are in `crates/temper-client-shell-cli/src/tests.rs`, whose tests mostly landed before ruling 117 (15:06).
     - The gate's checks cover only `tests/real-loop/tests`.
   - **When.** First at the 14:50 check.
   - Proposal 2.
8. **Two defects of `temper ctl`'s host showed after its landing, and others fixed them.**
   - **What.** A sign-in answered `Again` was fatal: 13.3's process proof failed on it at 11:22, and after-landing on 13.2's commit at 11:16. `e9bb42960` fixed it at 11:20, outside the ledger. Many calls exhausted its sockets: after-landing panicked on "bounded sockets" at 12:24. `5761ae077` replaced the run's in-memory transport with jig's `Transport` at 14:54.
   - **Cause.** 05.1 to 05.6 proved the host by unit tests, which neither answer a sign-in NotReady nor make many calls. The process tier reached `temper ctl` only from 10.1 (15:55), and the real loop only from 08.3 (16:05).
   - **When.** 11:16 on 10-20, by the after-landing tier: skein's `23e80d5` ran it after merging, and temper's script arrived at 11:09 (`4833827a6`).
11. **Smaller things.**
    - **Release.** The supervisor's ready-check worktrees `wt/temper-09r` and `-10r` were dirty at the release, which removed them by force (ruling 152).
    - **Compactions and the watch.** After four of them the supervisor found its watch not running, though the notes put several stops down to events that had come in. After the fifth, 06.2's block of 15:09:06 went unanswered for 17 minutes (ruling 121).
    - **A lost report.** One Codex hourly report was lost: `-o` resolves against the shell's working directory, not `-C`'s (`rl-hourly-1333`). The supervisor recovered it from the log (proposal 6).
    - **A duplicate tier run.** The supervisor's tier run of 14:42 on `57b9c692e` repeated after-landing's run of the same commit, which had ended at 14:42:19.
    - **The claim came before the kit.** Tools' proposals 6 and 8 landed in skein at 14:04, after the claim, and reached the run with the kit at 15:06. live-text's decision said real-loop would be claimed once they were applied. The claim's `plan check` (`2bd29bd`) did not yet refuse a claim with an adopted proposal unapplied; `4134983` added that at 14:18.
    - **Queue reads.** The coordinator ran `pueue status --json` unfiltered, and authors ran `pueue log`, while queued tasks still carried their callers' environment (ruling 144, until `a46635c8`). No read of another process's environment was found.
    - **Records.** `supervision.md` calls 13.s1's commit "13.1" (14:50 check). `decisions.md`'s "For the designer" still lists three points that landed in `116d0a645` before the claim. Ruling 150 says "36 ledger commits"; there are 35.

## 6. Decisions taken during the run

| Decision | Ruling | Reached the design? |
|---|---|---|
| The worlds' sum below the domain is measured idle, and is the worlds' guide, never a landing's | 109 (interim), 112 | **Yes,** testing.md 6.1 (`2f97ab8bb`). |
| The real loop counted apart, at a standing 6 s, each story at 1 s; the others keep 4 s (the user's L) | 136 | **Yes,** testing.md 6.1 (`bab11df83`). |
| A peer's end ends its unfinished call; the server frees the stream | 118 (interim), 123 | **Yes,** jig `protocol/client.md` 17.4 (`bfc3a82fc`); temper `client/README.md` 8 and tools.md 8.2 (`200916de5`). jig's reference service is a `later.md` row. |
| A call that `--call` reads gives no FILE argument; a FILE is read as given, under a root opened at the start | 111, 119 (interim), 123 | **Yes,** tools.md 8.1 (`200916de5`). |
| The real loop is smoke; its corner cases are tested below (the user's decision) | 130, 132 | **Yes,** testing.md 2.4, tools.md 11.1 and 11.3 (`2f71813ad`). |
| DIRECTORY's lock ends at the `Close` of its descriptor, before the call exits | 135 | **Yes,** tools.md 8.3 (`6a287983a`). |
| An absent DIRECTORY is opened before its private open; the window in which another process removes and remakes it is accepted | 102 | **No.** A `later.md` row (an existing-only private open in skein's io); no comment at `directory.rs:227`, no STATUS.md row. |
| A wake that overtakes a fresh-read chain is kept and replayed; an overtaken chain sends no decision | 145, 148 | **No.** domain/forge.md 5.3 states freshness; the rule is in `decisions.md` and the files of 13.s3 and 13.s4. |
| reconcile's world leaves the worlds below the domain | 150 | **No.** testing.md 6.1 still names it; `later.md`'s last row. |
| Owner work resumes behind a refused input | 131 | Already in the design (domain/root.md 4): a code defect. |
| Two exits after one commit have no order | 134 | Not design: a test's assertion. |
| The other worlds' 4.04 s | 151 | Not design: a measured miss, for the user and temper's owner. |
| Test shape: act once; independent expectations; named positions, counts and sizes; no `_` slice patterns; shared setup | 117, 126, 133, 137, 142, 146 | Not design: testing-strategy.md 10 says them. README rules, and two gate checks. |
| Diagnosis budgets | 110, 125, 129, 140, 143, 149 | Not design: process. |
| The coordinator checks no tests; kit moves; no other process's environment | 116, 120, 141, 144, 147 | Not design: process. |

For the user A to K were decided before the claim, at 13:40:09 (`e0bc5ebb9`) and in the plan's landing. L was the user's, at 17:32:40 (`08e91e24a`), as the review amended it.

## 9. Debts left in the code

For temper, which has no `docs/development/tech-debt.md`, and for its owner. "Listed" says whether `later.md`, a STATUS.md or a debt file already has it. Ordered by what they cost the next work.

1. **The other worlds below the domain are 0.04 s past their 4 s.**
   - `tests/one-process`'s `an_adopted_repository_gets_its_change_pushed_through_git` takes 0.899 s, 22% of the sum. 13.s1 had cut it to 0.724 s, and it rose again during the run; forge-kit's changes to `tests/one-process/src/world.rs` (`4e16e4eba`, `20d88b6bb`, `5dec8f6f0`) are candidates, not traced.
   - Next: `a_person_signs_in_by_the_printed_link_and_a_chat_is_answered` 0.264 s, and `a_signal_mid_run_drains_and_the_next_start_settles_the_claims` 0.253 s.
   - Listed: ruling 151; `later.md`'s 4 s row, whose wording ("if temper 06 … lands") is now past.
2. **The landing story costs a quarter of the fuzzy sum.** `tests/real-loop/tests/fuzzy_landing.rs:8`, `a_chat_lands_its_change_on_the_forge`, takes 4.1 to 4.6 s: four agent lifetimes, seven LLM turns, 23 hosted git calls. Its world's `tests/real-loop/src/landing.rs:72-107` sets 33 limit overrides, 2 of them explained, so each new forge story retunes them by trial. Not listed.
3. **testing.md 6.1 names reconcile's world** among the 4 s worlds, though it is skein's since `4b074713d`. Listed: `later.md`'s last row.
4. **Test shape left in this run's tests.**
   - `_` slice patterns over calls: `tests/real-loop/tests/lost.rs:30` (with a bare `2` at `:36`); `crates/temper/tests/process.rs:61`, `:103`.
   - Expectations computed by the code under test, in `crates/temper-client-shell-cli/src/tests.rs`:
     - `help_opens_nothing` at `:520-548`, which retypes `host/input.rs`'s help render;
     - `:608-614` (`root::next_deadline`), `:184-193` (`* 2 + 5`, as `host.rs:697`) and `:648-665` (the backoff modelled in the test);
     - `crates/temper/tests/process.rs:104-111`, which takes `first_sent` from the output under test.
   - Tables that branch on their row: `tests.rs:11-23`, `:477-493`.
   - One-sided bounds: `tests.rs:108`, `:191`, `:292`, `:304`, `:333`.
   - Assertions before the act: `tests.rs:136-145`, `:224-244`, `:345-355`; `tests/real-loop/tests/tools.rs:28-34`.
   - A port literal the "Done when" grep cannot see: `tests/real-loop/tests/landing.rs:13`, `([127, 0, 0, 1], 41_234)`, never bound.
   - `scripts/check.sh:9-17` checks only `tests/real-loop/tests`, and has no rule for slice patterns.
   - Not listed.
5. **Two call drivers, and their copies.**
   - `crates/temper/tests/support/ctl.rs` (532 lines: `Calls`, `Step`, `Call`) and `tests/real-loop/src/calls.rs` (640 lines: `Caller`, `Step`, `Argv`) are parallel vocabularies. Each defines `OUTPUT: usize = 65_536`, and the literal recurs about 12 times unnamed.
   - The adopt argv is typed in six places, the chat argv in two.
   - `tests/client`'s `call` helper is in `agent.rs:17` and `agent_recovery.rs:40`, and inlined at `agent_turn.rs:27-28`.
   - The standalone tree fixture is in three places: `crates/temper/src/tests.rs:127`, `tests/support/mod.rs:37`, and `tests/real-loop/src/fixtures.rs:83` (the last on purpose).
   - Not listed.
6. **A dev-dependency cycle.** `crates/temper/Cargo.toml:43` dev-depends on `../../tests/real-loop`, which depends on `temper`, since 13.3 shared `landing.rs` with the process tier (`48543c5ac`). `temper`'s tests build about 33 more crates. The line is not `workspace = true`, unlike its siblings. Not listed.
7. **Files near or past 800 lines.**
   - `tests/client/src/fixtures.rs`: 527 → 762 lines (+226 from this run). Every client story edits it.
   - `crates/temper-service/src/tests/fixtures.rs`: 841 → 840, already past; 09.s1 had to put its 35-line setup in `tests/engine.rs:171`.
   - `crates/temper-engine-domain-forge/src/domain.rs`: 3,594 lines, +24 from this run. Listed in protocol's `later.md`.
   - `tests/real-loop/src/calls.rs`: 640 lines, and about fifteen `later.md` rows will add to it.
8. **Decisions in force that the design does not say** (section 6):
   - the two-open window of ruling 102 (`crates/temper-client-shell-cli/src/directory.rs:227` has no comment on it);
   - the overtaken chain's rule of 13.s3 and 13.s4.

   Listed: `later.md` for the first only.
9. **Magic numbers in `temper ctl`'s host.**
   - `host.rs:94` (`* 3`), `:96` (`16`), `:106` (`+ 8192`), `:697` (`* 2 + 5`), and `limits.rs:77`, `:80`, with no comment.
   - The 30 s deadline is written again beside `limits::DEADLINE`: "within 30 s" at `host.rs:485`, and `Duration::from_secs(30)` at `directory.rs:61`.
   - Not listed.
10. **Stale rows.**
    - temper's `docs/design/STATUS.md` still names real-loop as the owner of delivered work: lines 15, 32, 33, 42 and 45.
    - The dropped 09.4 has no `later.md` row; ruling 130 names the lower tests that cover it.
11. **Not code.**
    - 39 local `real-loop/*` branches in temper. 37 are merged. Two are not:
      - `real-loop/temper-09-4-unread` (`9b9cba32a`), the only copy of the dropped 09.4's 247 lines;
      - `real-loop/temper-10-2-signals` (`ed5325404`), which ruling 128 left unlanded.
    - The designers' worktrees `temper/.claude/worktrees/real-loop-{lock,smoke,sums}`, all merged, and the planner's `tmp/planning/real-loop`.
    - The run's directory, 174 MB, keeps the two scratch patches of ruling 152.
