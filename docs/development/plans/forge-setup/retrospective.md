# Retrospective: forge-setup

2026-10-18.

## 2. Outcome

"Done when", item by item. The supervisor's sub-agent, on Sonnet, ran it
once, from 16:01 to 16:28, at temper `58c5321c1` and skein `f7a4e70`,
in fresh worktrees `wt/donewhen-<repository>`. It reran the
gate's items at 17:39 to 17:42, at temper `61bae3871` (`done.md`).

1. **Every step landed or dropped: passes.** `status.md` shows 44 of 44
   landed. The sub-agent read `status.md` rather than `plan status`.
2. **The plan's commits: passes.**
   - temper's `Step:` trailers list the 37 planned temper steps, and skein's
     list skein-01.1 to 01.3.
   - The four supplements add a line each.
3. **Each session file's "Done when": fails as written, in seven groups;
   holds after 14.s2, with six misses recorded by ruling 155.**
   - **The gate** (temper 01.7, 03.7, 06.5 and 10.1, with 07.1, 14.2 and
     15.4, which run the same test). `scripts/check.sh` exits 100 at
     `serve_prepares_once_on_the_owned_example`
     (`crates/temper/tests/startup.rs:328`) with "workers[0].memory_bytes:
     the worst case passes memory_bytes by 8 bytes". Every landing's gate
     had passed in its own, shorter worktree. After 14.s2 the rerun passed:
     1,833 tests and 50 fuzzy tests, 106 s.
   - **Five stale checks:**
     - temper 01.6 greps the plan's history for `actual`. 02.1 added three
       lines with it, and 02.2 removed them 25 minutes later. No later
       commit can make the item pass.
     - 03.1 counts `"17.0.0"` lines. A third, a test vector of 15.1's,
       sits in `crates/temper-forge-setup/src/tests/upgrade.rs:32`.
     - 04's `git diff $B --stat -- transcripts` expects nothing. Sessions
       01 and 02 added 120 `setup-*` captures there.
     - 08.4b's filter finds 1 test, not 4, because 15.1 renamed three.
     - 10.3d's `grep -c '"ci":true'` prints 0, because 14.1 moved the tier's
       CI rule into the owned example.
   - Not run, because they name reports: temper 02.5, the last paragraph of
     04, and 05.7.
4. **The box, on polling and with webhooks, end to end: passes.**
   `scripts/proof-one-process.sh` ran 19 tests, all passed, in 138.2 s,
   within 180 s. The hinted story alone took 84.4 s.
5. **The conformance's facts: passes.** The seven `fails` lines, and no
   `unchecked`.
6. **temper's skein holds `sha256::Hmac`: passes.** One revision,
   `8a51e10b8`, a descendant of skein-01.2.
7. **The test-time aims: temper's default suite fails; its fuzzy suite and
   skein pass.**
   - temper's default suite: the median of three is 5.991 s for 1,816
     tests, against 3.928 s for 1,306 at the reference `eab91b13d`. That is
     +2.06 s, against an aim of +1.0 s.
   - client-api's tests are inside that figure, and item 7's command cannot
     separate them (ruling 155).
   - temper's fuzzy suite: 21.977 s against 24.745 s.
   - skein: +0.30 s, against +0.5 s.
   - As written, the default run stopped at the gate's failure, so the
     sub-agent measured again with `--no-fail-fast`.
   - The three slowest default tests are not this plan's: they came from
     protocol's tests and from earlier plans.
8. **No budget raised: passes.** `.config/nextest.toml` has no
   `Plan: forge-setup` change, in temper or skein.
9. **These are the only checks.**

The four added steps came from the plan's own code or its "Done when", none
from a fault already on `main`:
- **temper-15.s1** (ruling 129): 07.4 drew Forgejo's `JWT_SECRET` as 64 hex
  digits, which Forgejo decodes as base64. A re-check against the real
  binary found it while 15.1 was being merged (ruling 128).
- **temper-08.s1** (ruling 133): three races in 08's process-test support,
  which read state before it was written, and failed 1 to 5% of gate runs.
  09.3's gate failed once on one of them. A $0.98 sub-agent diagnosis found
  the class, and the step's stress run passed 100 times in 100.
- **temper-14.s1** (ruling 147): 07.4's `CreatePerson` stored Forgejo's
  whole line ("generated random password is '…'") as the password. 03.2
  had recorded that line. The fixtures used a bare password, so only 14.2's
  first sign-in by web found it, 37 hours after the code landed.
- **temper-14.s2** (ruling 156): the owned example's `memory_bytes`,
  measured with each string a copy changes at its bound.

## 5. What did not

2. **Another run's landings broke this run's process tier, unseen.**
   - **Cause.**
     - The process tier runs "where a Proof line or 'Done when' says,
       never in the gate" (README). The pinned Forgejo and runner are on
       x1, and client-api's landings recorded their process proofs as
       "unavailable".
     - During the idle, the quiet checks ran nothing. So a story turned red
       on `main` only when this run's next step ran the tier.
   - **The landing story,
     `a_persons_change_lands_on_the_forge_and_is_found_after_a_restart`.**
     - It passed at `d4fe894a9` (02:29 on 10-17) and failed at `64f610ae6`
       (08:56 on 10-18).
     - The supervisor's bisect named three client-api commits: `fe8fcdf6b`
       (temper-03.5, 10:36 on 10-17), `e461dfd8d` (04.3, 05:38 on 10-18) and
       `3f8b3df6f` (05.1, 07:44).
     - client-api's `fe4978f22` fixed it at 13:31. It was red for at least
       4.5 hours, and perhaps 27.
   - **The serve stories** (the signed 204, the unknown 404, the drain).
     client-api's `3262df8f9` broke them at 13:10:53, and `46796e8df` fixed
     them at 14:42:28. The supervisor found the fix at 15:37: its wait had
     watched the bisected commit's files.
   - **First visible:** 09:22, ruling 139, from 11.1's ready check.
   - **Acted on.**
     - The bisect began at 09:19, was cut off by the network loss, was
       redone from 11:18 to 11:54, and went to client-api through the
       review at 11:55.
     - The review asked client-api, at 14:03, to run the whole tier before
       each remaining landing. client-api's gate ran it from 14:59.
   - **Cost.**
     - 11.1 was held from 11:20:58 to 15:45:31, across two blocked threads
       and two fresh restarts.
     - Five rulings: 139, 141, 152, 153 and 154.
     - From 09:00 to 14:05, 83 gate-lane tasks, 48 of them failing.
   - **This run did it too.** 14.1 dropped the tier's `root.brief.gather_ms`
     from `forgejo::configuration` while running only the box stories. The
     12:37 check found it, and 11.1 restored it at 15:45, so it was off
     `main` for 3 hours. Rulings 148 and 149 followed.
   - Proposal 1.
3. **"Done when" failed seven ways, after every step had landed.**
   - **Five stale checks** (section 2, item 3).
     - Cause: each item pins a state that a later session's Files change. It
       reads the history (01.6), counts a file (03.1), diffs a directory
       others write to (04), or names tests and members others rename or
       move (08.4b, 10.3d).
     - The README forgave only growing counts ("read as 'at least'").
     - The plan's reviews had flagged the class (literal-A 9, literal-B 1).
     - Ruling 128 knew of 08.2's split at 01:14 on 10-17, and said "Done
       when's counts unchanged".
     - First visible: when 02.2 (01:41), 15.1 (01:15) and 14.1 (12:36)
       landed, and on reading the plan.
     - Acted on: 16:35 on 10-18, ruling 155. Proposal 4.
   - **The gate in a longer path** (item 4 below). Proposal 5.
   - **The default suite's aim.**
     - Cause: client-api's tests ran in the same suite, and item 7 measures
       the workspace. On the first night, when only this plan landed, its
       382 default tests added about 0.75 s.
     - Single gates swung 1.2 to 1.65 s at the same test count, more than
       the aim.
     - The kit's pulse prints the budgets, not the aims. `171ca57`, which
       names landings past an aim, is not in the kit.
     - First visible: 03:49 on 10-17, with one gate at 5.9 s. At the 11:19
       check on 10-18, +1.85 s, and no steer.
     - Acted on: 16:35, recorded for the user by ruling 155. Proposal 6.
4. **The owned example: three design rounds and two supplements in the last
   session.**
   - **Cause.**
     - Decision 5's limits were checked by arithmetic. No one started the
       example at the base: it did not exist. The design's example could not
       start, whatever its grants (ruling 142).
     - 14.1's Shape said "one run reports every problem: at most three
       reruns". `startup.rs` returns after each of its five stages (ruling
       143), so each run shows one stage, one problem deeper.
     - `small` did not fit smith's agent bounds. Four of its production
       constants changed to fit the example (ruling 146).
     - Ruling 143's "the least each check accepts" made the numbers depend
       on the scratch directory under the checkout. The worker counts its
       agent's program and its configuration path, and the engine its
       token and git base.
   - **First visible.**
     - 09:22, 14.1's ready check: "eight reruns found no passing document".
     - The path, at 12:32: 14.1's first box proof needed 32 bytes more,
       which its report of 12:37:18 gives as "an additional 32 bytes for
       its longer scratch path".
     - The path again at 13:48, in client-api's `acc7496e2`: "32 for the
       box's longer scratch".
   - **Acted on.** 11:42 (ruling 142), 12:03 (143), 12:28 (146), and 16:48
     (156, after the gate failed). Five commits of two runs set or raised the
     numbers between 12:36 and 16:56, three of them for a checkout's path.
   - **Cost.**
     - The fifth question waited 2 h 19 min, across the network loss.
     - 14.1 was open from 09:22 to 12:36, over four threads, and two of them
       landed nothing.
     - 14.s1 and 14.s2.
   - Proposal 5.
5. **The guide script's runs: three, then six, then ten.**
   - **Cause.**
     - 14.2's Proof bounded the runs of a first script against Forgejo's
       web pages, which no one had probed. Each run met a new cause:
       - the version's build suffix;
       - no `_csrf` on `/user/login`;
       - a redirect of `/user/change_password`;
       - a 303 that becomes a 404;
       - the script's own CI branch, which adoption refuses.
     - Run 4 failed because the coordinator's `cargo clean`, at 13:09:10
       when the turn ended, had removed the binary the checkpoint said was
       built.
   - **First visible:** 12:41. **Acted on:** 13:10 (ruling 150) and 13:18
     (151). 14.2 landed at 13:20, after eight runs of 2 to 42 s.
   - The supervisor's 13:17 note: "a first script against a pinned binary's
     pages meets one new cause per run".
12. **Smaller faults.**
    - **Test-shape rules at the third sighting.** Ruling 140 made two README
      rules from sightings at 23:03, 02:28 and 11:19. That was 36.5 hours
      after the first, across the idle. development.md 14.4 says the second
      sighting.
    - **A stale premise.** Ruling 148 declined render's proposal 6, a
      test-shape check in the gate, because the workspace `AGENTS.md` says
      "temper has no `scripts/check.sh` yet". temper has had one since
      `796d50279`, on 10-13. Ruling 152 corrected the premise 61 minutes
      later, and added no check (proposal 11).
    - **`rm -rf target`.** It was refused in 25 of the first 28 author
      threads, each time with the `plan log` of its report in the same
      command. The card prescribed it; `b30cc95`'s `cargo clean` came with
      the 11:34 steer.
    - **The network loss**, 09:30 to 11:00 on 10-18.
      - No task ended for 1 h 43 min.
      - The supervisor's requests failed from 09:41, and it resumed only at
        the message at 11:18, about 18 minutes after the network
        returned.
      - The designer and two sub-agents were cut off and restarted.
    - **Local times.** `uptime -s` prints x1's local time.
      - The coordinator first wrote the reboot an hour late.
      - The review recorded a second reboot "at 12:50:56Z". The boot list
        has one boot, at 11:51:14.
    - **`done.md`.** The coordinator's "Supervisor final verification:
      Pending" and "Rulings for these: not yet" stayed after the supervisor's
      sections answered them.
    - **Disk.** Starts held below 40 GB at 01:13 on 10-17 (2 minutes), at
      08:58 on 10-18 (29 GB), and at 11:36 (36 GB). The review found
      something building in temper's primary checkout, and freed 9.7 GB.

## 6. Decisions taken during the run

56 rulings, 101 to 156. By kind:
- 19 plan defects and 1 plan question;
- 14 ready checks and re-checks against `main`, which client-api kept
  moving;
- 8 design answers;
- 7 process;
- 3 interims, 2 test-shape rules, 1 unblock;
- 1 "Done when" record.

| Decision | Rulings | Reached the design? |
|---|---|---|
| A cancelled job's delivery is recorded through the API's cancel | 103, 104 | Yes, `fbe1ceaed` (forge.md 5, 13) |
| A pull's check at its push's delivery varies; the fake always checks | 111 | Yes, `edcd1954b` (forge.md 9, 13) |
| Reconcile never edits an existing bot | 118 | Yes, `231582c05` (setup.md 1, 5, 8) |
| A prepared forge takes a runner added or removed; the runner's lock | 123 | Yes, `13e35d82e` (setup.md 6.3, 6.4, 13.1); `done.md` omits it |
| The owned example's engine limits, stated whole | 142 | Yes, `a975025ad` (configuration.md 7, 10) |
| The hinted story proves CI's end delivery, with the stall raised; long CI deferred | 144 | Yes, `3a323b3b2` (setup.md 10, 13.4; testing.md 2.6; `later.md`) |
| `small` gives four constants; the example's worker limits | 145, 146 | Yes, `6545cb474` (configuration.md 7, 10) |
| The owned example's `memory_bytes` at the bounds of what a copy changes | 156 | Yes, `13154c9dd` (configuration.md 7, 10) |
| Forgejo's answers: a nameless team edit, explicit `none`, a narrowing hook keeps `workflow_job_*` | 119, 127, 131 | No: the conformance's `facts.txt` holds them |
| `JWT_SECRET` as URL base64 of 32 bytes | 128, 129 | No |
| `person_password` holds the password alone; forms post `_csrf` when a page has one | 147 | No |
| The hook secret's bound wired in 05.3 | 134, 135 | No: the plan's |
| 06.1 lands with 06.2 | 115 | The plan's |
| Size tolerance 2.6 times, then 3.5 (codec 4.5), increments in progress included | 102, 125, 126 | The plan's README. the implementation rules at `main` now lets an increment land past its Size |
| Byte-stable recordings; the fixture's HTTP 500 retried | 114, 121, 124 | The plan's README |
| No rerun limit; each number the least its check accepts | 143 | The plan's; replaced for `memory_bytes` by 156 |
| The guide script's run bound, 3 to 6 to 10 | 150, 151 | The plan's |
| One behaviour per test with a literal; named sizes and both sides of a bound; a setup written once; a changed support function's users run | 117, 140, 148, 149 | The plan's README; testing-strategy.md 10 at `main` says some |
| The author runs a renamed Proof; the coordinator checks with git | 137 | development.md 2.3 and 7.3 already say it |
| Regressions of another run's on `main`: hold, permit named reds, unblock | 139, 141, 152, 153, 154 | No: in no document (proposal 1) |
| The six "Done when" misses, each with the command that shows the property | 155 | The plan's (proposal 4) |

**The reversals.**
- **104 over 103's interim**, 4 minutes later: the designer found an API
  route that cancels a run.
- **135 over 134's `Refused::words()`**, 3 minutes later: 10.1 had landed.
- **More replacements, without a `Reverses:` line:** 123 over 120, 125 over
  102, 126 over the 00:30 steer, 129 over 128's parts, 146 over 145, 149
  over 148, 152 over 139 and 141, 154 over 152's list, and 156 over 143
  (whose unblock says "Reverses: nothing"). Proposal 12.

**For the user.**
- decisions.md lists ten numbered decisions, and one unnumbered, "for the
  user", none with a `Decision:` line. The run built each as written.
  kit-batch's `5f26abb` refuses such a plan at `plan check`, but the run's
  kit lacked it.
- Decision 13, the test-time aims, is not among them. Its miss, item 7, is
  recorded for the user by ruling 155.

## 9. Debts left in the code

temper has no `docs/development/tech-debt.md`. These belong in one, with
what was seen, why it matters and what pays it back. Paths are at temper
`main`.

1. **The owned example's numbers are kept by hand.**
   - A landing that raises a worst case raises `engine.memory_bytes` or
     `workers[0].memory_bytes` in `docs/examples/owned.json` by the N of
     T3's refusal (14-example.md, the block of 16:47:23).
   - Two runs edited them five times in four hours, and client-api holds its
     edits until 14.s2 (its ruling 377).
   - A test that derives them would end it. Medium.
2. **`small`'s worst cases are about 71.5 TiB (engine) and 1.8 TiB
   (worker)** at their bounds, so `memory_bytes` bounds nothing a person
   has (`later.md`). The designer's. Medium.
3. **A change's CI is read only at its `stall`, 30 s as shipped, when no
   hint arrives.** A box whose CI takes longer meets Stalled (ruling 144,
   `later.md`). It is due before dogfood. Large.
4. **Three gaps of the example's limits** (`later.md`, `6545cb474`):
   - the worker's check names one of two limits;
   - a start that passes, but whose hello never arrives, when
     `forge.resources_per_task` is below `checkout.repositories`;
   - the example's agent is not sized for a person's work.

   Medium.
5. **Four `small` constants changed to fit the example** (ruling 146,
   `crates/temper-service/src/limits.rs`):
   - `checkout.repositories` 3 to 2;
   - `forge.resources_per_task` 4 to 2;
   - `checkout.message_bytes` 256 to 4096;
   - `forge.name_bytes` 256 to 64.

   Whether they suit a person is open. The designer's. Medium.
6. **The guide's walk lives in the run's directory.** 14.2's Proof runs
   `logs/temper-14.2/follow.sh`. Once the run's directory goes, nothing
   checks the setup guide against the pinned Forgejo again. Medium.
7. **Two tests assert the whole usage text:**
   `sign_in_without_two_arguments_prints_its_usage` and
   `forge_without_its_arguments_prints_its_usage`
   (`crates/temper/src/tests.rs:125`, `:127`). Each failed a gate when a
   sibling session's command grew the usage under a clean rebase. Small.
8. **Files this plan brought near or past the 800-line aim:**

   | File | At the base | Now |
   |---|---|---|
   | `testing/temper-fake-forge-domain/src/api.rs` | 794 | 800 |
   | `crates/temper-protocol-store/src/tests/fixtures.rs` | 797 | 798 |
   | `crates/temper-engine-protocol-forge/src/reads.rs` | 758 | 793 |
   | `testing/temper-fake-forge-protocol/src/response.rs` | 741 | 789 |

   `crates/temper-service/src/tests/fixtures.rs` is at 840, mostly from
   client-api. Small.
9. **A version named in a test:**
   `crates/temper-forge-setup/src/tests/upgrade.rs:32` names `b"17.0.0"`.
   code-organization.md's "Don't" forbids a version outside the pin, and
   ruling 140's literal rule asks for one. The two rules disagree. Small.
10. **Forgejo behaviour held only as facts:** a nameless team edit answers
    200 and keeps the name; a narrowing hook PATCH keeps the
    `workflow_job_*` events; `/user/login` and
    `/user/settings/change_password` hold no `_csrf` (rulings 119, 131,
    147). The design says nothing of them. Small.

skein's, from this run:
- **The browser test on x1.**
  - `chromium_survives_a_page_that_reloads_itself` timed out once on x1
    (22:43 on 10-16), and passed on the retry.
  - With `a_page_of_another_origin_is_opened_by_its_step` filtered, x1's
    skein gate is not `scripts/check.sh` as written.
- **jig and temper do not build against skein's `main`:** the hourly
  downstream builds failed six times on 10-18, on `skein_io`'s new `Locked`
  and `Busy`. Not this plan's change.

The kit's, for skein:
- `plan ruling`'s switch of the docs branch (proposal 10).
- **the command log:**
  - It has no header, no kind, and no time alone. A gate's "ran" includes
    its pause.
  - pueue reuses ids (id 45 is in 234 rows).
  - Three finished tasks have no row: 37, 43 and 44.
- **Local times:** `uptime -s`, pueue's timestamps and file times give x1's
  local time where the documents expect UTC.
- **`plan idle`** ran in a script after the script's `context.md` write had
  failed (11:36:01). `context.md`'s 200-character lines ran words together.
- **`plan status`** prints "last check: Reasoning".
- **The pulse's "binary file matches"** for goldens, carried from earlier
  retrospectives.
