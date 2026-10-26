# Retrospective: client-api

2026-10-18.

## 2. Outcome

`done.md`'s "Done when, run 2026-10-18T23:14:55Z" holds each command's output. A Sonnet sub-agent of the supervisor ran them once, from `origin/main`, and ran item 11.

| Item | Result |
|---|---|
| 2. jig's old client crates gone | **Met.** `ls-tree` prints nothing. |
| 3. jig's ten new client crates | **Met.** It prints 10. |
| 4. The naming checks | **Met.** `PENDING: [&str; 0]`; `client_crates_carry_the_client_prefix` passes in jig and in temper. |
| 5. No shape of work on the wire | **Met.** The grep prints nothing. |
| 6. jig's component owns no listener | **Met.** |
| 7. jig's gate has the browser suite and the wasm32 builds | **Met.** Four lines of `scripts/check.sh`. |
| 8. temper's client is its own | **Met.** No `jig-client-view`, `jig-domain-client` or `jig-view` in temper's manifests. temper's `Cargo.lock` still reaches `jig-client-view` through `jig-client-world` (section 9, item 16). |
| 9. temper pins jig's last landing | **Met.** `bbea94081`, which is jig-22.s1's. |
| 10. The plan's Step trailers | **Met.** All 137 planned steps are listed. Other lines: the 22 supplements, and `temper-06.s3`, which ruling 335 counts as 06.3; one empty line each in jig and temper, from a commit with no Step trailer; and four `temper-NN.N-repin` names, from ungated repin commits within steps. |
| 11. The test-time aims | **Missed**, recorded in ruling 402 with the three slowest tests of each suite. Medians of three: jig focused 14.283 s against 13.3, fuzzy 26.789 s against 24.91; temper focused 6.476 s against 6.0, fuzzy 23.885 s against 19.36. Every run was within its budget. |
| 12. No budget raised | **Met, as ruling 401 amended it at 22:53**, 21 minutes before the run. jig shows the browser profile's lines and ruling 275's focused 15 s to 30 s. temper shows ruling 326's scheduling override, which the item had not allowed for. |
| 13. Each store family moved once | **Met.** It prints 3. |
| 14. One definition of the wire's version | **Met.** One line, `crates/jig-client-protocol-wire/src/lib.rs:12`, `VERSION: u32 = 8`. |
| 15. The client's transitional pieces gone | **Met.** Both commands print nothing. |
| 16. No other aim | **Met.** |

What the plan names but leaves for later:
- **The browser's back and forward.** Ruling 393 deferred them to temper's later.md, since skein's pin was capped. The browser tier ends at 13 stories.
- **The brief's 64-digit commit spelling.** Ruling 399 deferred it to temper's later.md and STATUS.md.
- **The rows `later.md` already held.**

The six steps of jig's STATUS.md, section 3, are on `main`.

## 5. What did not

10. **The run broke temper's process tier for 27 hours, and another run found it.**
    - **The cause.** Ruling 163 (18:36 on 10-16) had the landing story `#[ignore]`d until temper 04.4.
      - temper 03.5 (`fe8fcdf6b`, 10:36 on 10-17) landed with "Proof: … unavailable under ruling 163; joint fallback passed".
      - 04.3 (`e461dfd8d`, 05:38 on 10-18) and 05.1 (`3f8b3df6f`, 07:44) broke the story further. Their process Proofs ran a smoke test.
      - temper-01.1 and 06.1 also landed with "unavailable" process Proofs.
    - **First visible:** forge-setup's check, at 11:55 on 10-18, on the shared `main`. forge-setup's retrospective puts the story red from 08:56 at the latest, possibly from 10:36 on 10-17.
    - **Acted on:**
      - Ruling 347 at 12:17 added temper-05.s1, which landed at 13:48.
      - 04.s1's `3262df8f9` (13:10) then broke the signed delivery. Ruling 367 added 05.s2, which landed at 14:42.
      - 13 rulings, from 339 to 372, dealt with these regressions, and temper-04.s1 alone blocked seven times. Ruling 369 made the whole tier run before every temper landing.
    - **The cost to forge-setup.** It waited on these fixes for 4 h 25 min.
    - Proposal 7.
11. **btrfs ran out of metadata room.**
    - **What failed.** At 04:04 on 10-18, `/srv` had 1 MiB unallocated, its metadata at 3.80 of 4.32 GiB, while `df` showed 98 GiB free.
      - mold's linking died with Bus errors (SIGBUS) and "Disk full?".
      - pueued and Codex's daemon died, at 03:47:45 and about 04:13:42.
      - Four author threads were lost: 161 requests and 15.8M tokens, redone.
      - pueued, restarted by hand, lost the cgroup the process tier needs for three hours.
    - **Acted on, by hand:**
      - the supervisor's three balances (04:13 to 04:16), then a balance below 5 GiB unallocated at each disk check (ruling 309);
      - a dozen more balances, 04:40 to 12:36;
      - the trigger raised to 10 GiB (ruling 331), and `-dusage=75` when data was 90% used (ruling 334).
    - **The gap.** development.md 11.4 bounds disk and memory, not a filesystem's unallocated space, and `plan check` does not read it.
    - Proposal 8.
12. **The test-time aims were missed, and jig's focused budget was raised.**
    - **jig.**
      - The focused suite went from 9.9 s to 14.3 s (+44%) for 702 more tests (+20%).
      - Five land gates timed out at 15 s from 17:01 to 17:09 on 10-17, every test passing. Two of them coincided with the check's own un-niced pulse and a downstream run.
      - Ruling 275 raised the focused `global-timeout` to 30 s for the rest of the run, by the user's rule of 2026-10-13. It is still 30 s on `main`, and "For the user" F has no answer (section 6).
    - **The repair step.** jig-14.s2 (ruling 269) was added on ruling 267's premise, which ruling 268 reversed. It had no measured cause, and it left the suite 0.56 s slower.
    - **temper.** Ruling 326 made temper's aims guides. It also made "Done when" item 12 false, until ruling 401 amended the item 21 minutes before the end.
    - **SLOW flags.** jig's "Done when" runs flagged 20 to 22 focused tests SLOW with the machine alone. Ruling 400 made a flag under a test's period report-only.
    - No new proposal. testing-strategy.md 8 ("a slow test is understood, never hidden", `b4c6241`) landed at the run's end, and the pass after the run is the debt of section 9.
## 6. Decisions taken during the run

302 rulings, 101 to 402, one commit each in temper, 08:09 on 10-16 to 23:14 on 10-18.

| Kind | Rulings | Reached the design? |
|---|---|---|
| Plan defect: Shapes, Files, Size, stale facts, lints, order | 130 | Plan only. Their Shapes changed session files. |
| Process: bounded diagnosis, steer plumbing, proof discipline | 33 | No design text, by kind. |
| Steer made a README rule | 32 | Plan only (section 5, item 4). |
| Design gap answered by the designer | 29, with 9 interim rulings | **Yes, every one** (the table below). |
| Budget and test time | 20 | No design text. Ruling 275's budget is in later.md. |
| Cross-session scope | 12 | Plan only. |
| Infrastructure | 10 | No design text: 309 and 334 (btrfs), 324 (pueued's cgroup), 315 (the merge's ref lock), 295 (the kit), 296 (skein's cap). |
| Supplementary step added | 9 | The steps added by design rulings are counted there. |
| Step reordered | 6 | Plan only. |
| Regression of a landed step | 6 | Plan only. |
| "Done when" amended | 4 | Plan only: 271, 383, 401, 402. |
| Other | 2 | 154, 293. |

The design answers. Each names its commits, all on `main`:

| Ruling | Question | jig | temper | skein |
|---|---|---|---|---|
| 106 | What temper's census does with `EndedRead` rows | `9a24ded8a` | `97315175` | |
| 125 | A tracked conversation; a second grace | `a70de2906` | `ed6593899` | |
| 134 | How the host bounds the terminal's lines | | `3d695094` | `d047b45a` |
| 143 | The terminal's worst case; a path's segments; templates as JSON | `7efaeef4d`, `413726589` | `0625a5fa` | `8907ea3a` |
| 150 | What a run watch holds | `bab2e89cb` | `9ae5fca0` | |
| 165 | A forget lost in a crash | `134cbc189` | `4f86717c` | |
| 166 | A mark on a position not yet durable | `09a600c43` | `f92fddd0` | |
| 170 | A parked `NotStored` | `3b8bf1263` | `b694e960` | `1f71424d` |
| 180 | The owner's versions | `b09e2239e` | `21c0ec9a` | |
| 184 | A compound's outcomes, numbered | `a20cf96c4` | `90ed12e0` | |
| 190 | The funder of an accepted amendment | `1ee6c02ac` | `cf9493c5` | |
| 194 | What the bridge tells; the wall, `SignIn`, `Fact` | `64e1ab93b` | `8a14b8ed` | |
| 210 | Words that release a task | `5a9a97485` | `3cd97ab2` | |
| 224 | jig 14.4 and 14.5 against `main`'s people types | `b58a49c39` | `bc7c1376` | |
| 235 | Who keeps a read's page | `de4f80564` | `892f4e88` | |
| 246 | A start before the sign-in | `8c832e517` | `c3d0c0b9` | |
| 251 | What a cancel counts | `360aabcc6` | `63803dd15` | |
| 253 | A sign-in page after the engine ended the sign-in | `bbb6303aa` | `5b2faba9d` | |
| 262 | Whose project a role executor is | `ad3b26d04` | `ec988d053` | |
| 274 | The chip's status; sources from `me` | `6155e83e0` | `d55ffe2b2` | |
| 303 | The board's move once `Prioritise` goes | `ad465272b` | `baa38bb0a` | |
| 316 | Whether a watch holds its room until it ends | `544a57091`, `cd2cac35f` | `bf33cf174` | |
| 319 | Whether a reader lacking Amend loses a whole document | `35ab18732` | `405924109` | |
| 321 | temper's out-of-date `main`; an unnamed project's title | | `872fda7aa` | |
| 373 | One stream per client | `e444994d4` | `8e2e88622` | |
| 374 | A click on a button's text child | | `f5d63eaee` | `f7a4e7022` |
| 376 | Where a chat's waiting proposals show | `3150da02d` | `ffa4c1a20` | |
| 394 | Whose renderers temper's cards use | `2bd1a5cbb` | `076c659a5` | |
| 399 | Whether a byte parameter is text | `479a8f42d` | `e1e56152b` | |

**Not in the design yet:**
- **The server's eight connections.** Ruling 362 set them, and no design text names a count.
- **The browser's back and forward.** Ruling 393 deferred them, as a later.md row.
- **The brief's commit spelling.** Ruling 399 deferred the 64-digit spelling, in later.md and STATUS.md.
- **The busy answer to a reader past its bound.** Ruling 319 deferred it to later.md.
- **The fuzzy walking sweep.** Ruling 122 left its cost unrepaired.
- **The test-cost pass that restores 15 s.** Ruling 275 left it as a later.md row.

**"For the user"** (`decisions.md`, from line 3251):
- **A to D.** temper's scope, the naming check's home, the plan's home, and the two items of jig's step 5 that wait. The review read them and vetoed none, so the defaults stood.
- **E.** The process tier's cgroup (ruling 111). It was done at 14:58 on 10-16, lost at the hand restart of 04:17 on 10-18, and restored at 07:17 (ruling 324).
- **F.** jig's focused budget at 30 s (ruling 275). **It has no recorded answer from the user.** Ruling 275 and later.md say 15 s comes back after the test-cost pass that follows the run, once the median of three gate-lane runs is at most 12 s. "Done when" measured 14.283 s.

## 9. Debts left in the code

For jig's `docs/development/tech-debt.md`, and for temper, which has none yet. "Listed" says whether jig's file already has the entry. They are ordered by what they cost the next plans.

1. **skein's pins are frozen.**
   - jig pins skein `0f093bb` and temper `8a51e10b`, under the `2733882` cap (rulings 154, 296). skein's `main` is 89 commits past it.
   - hearth's `skein_io` changes (`089ed83`: `Locked`, `Busy`, `Lock`, `ListPage`) gave 15 errors in jig and 19 in temper at the downstream check of 23:22 on 10-17.
   - The browser's back and forward wait on skein too (ruling 393), as does a click on the nearest control (`skein-ui-web/src/page/web.rs:76`). Until then jig and temper name their controls and hold no node in them (ruling 374).
   - Needs a jig repin and a temper repin, first in the next plans. Not listed.
2. **jig's focused budget is at 30 s.**
   - `.config/nextest.toml:10` (ruling 275); its header and `workflow.md` still say 15 s.
   - The median is 14.283 s against the base's 10.33 s. 15 s comes back once a gate-lane median is at most 12 s, so at least 2.3 s (16%) must go.
   - The slowest focused tests under load are protocol-world stories at 0.88 s, 0.31 s alone.
   - Listed (section 4), without these figures.
3. **The test-time aims missed** (ruling 402), with their slowest tests:
   - **jig fuzzy:**
     - `random_people_over_bytes_keep_the_referee_s_rules`: 13.7 s under load, 4.968 s alone, within 1% of its 5 s period;
     - `every_shared_story_survives_every_commit_and_seeded_random_cuts`: 9.7 s, 3.3 s alone;
     - `p_story_survives_seeded_commit_crashes_and_random_presses`: 8.4 s, 4.08 s alone.
   - **temper focused:** `an_adopted_repository_gets_its_change_pushed_through_git`, 3.171 s, 0.947 s alone.
   - **temper fuzzy:** `fuzzy_walking_commit_cuts_latency_replay_and_observation_pressure`, 24.69 s, which is the whole suite's median.
     - It formats both worlds with `Debug` at up to 600 iterations (`tests/engine/domain/src/walking.rs`, `run_replayed`; ruling 122).
     - A fix needs `PartialEq` on jig's `Core` and temper's `engine::Domain`.
   - Not listed, except the conformance cuts in general.
4. **Worst-case literals refreshed by hand** (rulings 110, 112, 354, 356, 363, 398).
   - **jig.**
     - `crates/jig-protocol-config/src/tests/domain.rs:195` pins the small profile's memory sum: 11,900,377,351 at the base, 18,576,825,183 now (+56%).
     - `ENGINE_SMALL` went from 16 to 32 GiB.
     - `testing/jig-test-service/src/tests.rs:22` pins store operations.
   - **temper.** `docs/examples/owned.json`'s `engine.memory_bytes` is exact, so each worst-case growth refused the owned startup and needed a commit of its own.
   - **Needs:**
     - a write mode, like the drift tests' (ruling 131);
     - a ruling on whether "small" may need 18.6 GB.
   - Not listed.
5. **SLOW flags are noise.** The "Done when" runs flagged 20 to 22 focused and 13 to 14 fuzzy jig tests SLOW with the machine alone. jig-22.s1's 26 flags are in `logs/jig-22-s1/slow-workflow.md`. Needs periods set from times alone, or the flag taken from a serial run. Partly listed.
6. **Bounds the client meets as silent cuts.** All are later.md rows, and need the design first (contract.md, 5.3 and 6). Not listed.
   - `documents/small.rs:133` fills `OfferedCharter.goals` to `documents.lines`, while the wire allows 4, so the frame is refused `Bound` (ruling 227).
   - `acts/bounds.rs:47` cuts `me`'s creating offer, or refuses `Limit { Authority }`.
   - `reads.rs:407` and `502` answer `Missing::TooLarge` as `Why::Busy`, which the client reads as "retry later".
7. **A widening's funder.** `jig-domain-tasks/src/control.rs` (`can_resize`, `resize`) resizes the task's present funder, while `acts/bounds.rs:358` counts the amender's pool. jig-03.5 passes `funder: None` on direct widenings (ruling 190). Needs a jig plan on `domain/authority.md` 7 and `tasks.md` 7. Not listed.
8. **Board moves while a goal is closing.** `page/project/moving.rs` renumbers every line in one compound. The goals query includes closing goals, so one closing goal refuses the whole compound. Not listed.
9. **Point loads before durable commits.** `Now::HistoricalProposal`, `HistoricalEscalation`, `HistoricalPersonTask` and `PartyInputs` (`jig-domain-core/src/routing/mod.rs:382`, `556`) go out as `Request::Now`, against engine.md 4.2. Not listed.
10. **A placement race.** `routing/run_preparation.rs:185` picks `fleet.free_kind` for two claims prepared in one commit, so one waits while a worker is free (ruling 129). Not listed.
11. **Decided proposals are never forgotten.** `forget.rs` has `Owner::of_row` and `of_result` only (rulings 175, 177), so decided proposals grow the store. Not listed.
12. **temper's brief spells a SHA-1 commit as 64 digits.** `temper-engine-domain-forge/src/brief.rs` (`append_hex`) disagrees with `route/forge_route.rs`'s `commit_text`, which trims the padding (ruling 399). Fix before an agent resolves or gates on a SHA-1 repository.
13. **A second kind of saved record.** The stream-name record (key `b"stream"`) is special-cased in `jig-client-protocol-link/src/store.rs:176`, and skipped by three checkers:
    - `tests/client-protocol/src/client.rs:130` (ruling 375);
    - temper's `tests/one-process/src/referee/client/cache.rs:143` (ruling 397);
    - `jig-client-test-shell-web/src/records.rs:30`.
    A third kind will need the same skips. Not listed.
14. **The listener's stopgaps.**
    - `loopback: true` is hard-coded (`temper-service/src/clients.rs:640`, `jig-test-service/src/clients.rs:703`), and the cookie has no `Secure`.
    - `jig-test-service` builds its components at `Time::ZERO` (`clients.rs:137`, `144`); ruling 325 fixed only temper's copy.
    - The server's 8 connections (ruling 362) are in no design text.
    - Not listed.
15. **Files past 800 lines.** jig has 66, from 65 at the base; temper 17, from 15.
    - Crossed during the run: `jig-domain-tasks/src/control.rs` (644 to 883), `testing/jig-test-connector/src/boundary.rs` (747 to 892), temper's `temper-service/src/tests/fixtures.rs` (753 to 840) and `tests/system/src/composed.rs` (795 to 819).
    - Needs the consolidation pass (code-organization.md, 7).
    - Partly listed (jig's section 7).
16. **Two vocabularies for jig's executors and outcomes.**
    - temper keeps its own phrases until `jig-client::words` says them (temper-07.s1).
    - temper's `Cargo.lock` still reaches `jig-client-view` through `jig-client-world`.
    - `temper-client-view/src/cards.rs:165`, `238` and `cards/proposal.rs:88`, `167` take parameters nothing reads.
    - 14 terminal goldens were recorded by hand (ruling 396), with no write mode.
    - Not listed.
17. **Terminal text cut at 128 bytes.** `temper-client-shell-terminal/src/limits.rs:55`: the view's `compose` overflows skein-ui's `u32` `text_bytes` at 256 (rulings 160, 163). Needs jig's view.md 10 and skein's ui.md 15 before temper's pages grow. Partly listed (jig's section 10).
18. **Wire fields nothing fills.** `translate/refusals.rs:85` sends `executor`, `needed`, `asked`, `latest`, `reason`, `party` and `how` as `None`. `jig-protocol-transcript/src/lib.rs:159` builds `Call.objects` and `Call.parts` empty. Listed for `refusals.rs` (section 3).
19. **Lints and words.** jig's `manual_map` allowances went from 191 to 233, 52 of them in `examples/ops`. The forbidden word `actual` is still in 839 places, mostly `tests/client/src/joint`. Comments cite rulings in `client-v8-part1.schema:49`, `56` and temper's `pages/inbox.rs:1`. Partly listed (jig's section 7).
20. **Not code.**
    - 172 local `client-api/*` branches remain (jig 122, temper 43, skein 7).
    - The drafts `temper-07-4-renderers-draft` and `temper-07-4-removal-draft` must never land (ruling 394).
    - Six dirty worktrees sit in the run's `preserved-worktrees/`.
    - The process tier needs pueued in its cgroup, which a reboot loses.
