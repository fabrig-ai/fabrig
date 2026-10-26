# Retrospective: tools

2026-10-19.

## 2. Outcome

`done.md`'s "Done when, run 2026-10-19T11:52:47Z" holds each command's output. A Sonnet sub-agent of the supervisor ran them once, from `origin/main`, and ran item 13. `done.md`'s first part, at 11:48:42, lists only which steps cover each item.

| Item | Result |
|---|---|
| 2. skein's table takes a given key and refuses a held one | **Met.** It prints 2 and 0. |
| 3. jig sends under a name | **Met.** It prints 1. |
| 4. jig reads one turn, at one wire version | **Met.** One line, `crates/jig-client-protocol-wire/src/lib.rs:12`, `VERSION: u32 = 9`; `Turn { task, attempt, turn }` once. |
| 5. jig takes a deadline as a span | **Met.** `NewDeadline` and `tasks: 4, people: 4` once each. |
| 6. jig's budgets are 15 s and 60 s | **Met.** `15s 60s`. jig's `tech-debt.md` still says 30 s (section 9, item 12). |
| 7. No profile changed | **Met.** |
| 8. temper's tools exist as a library | **Met.** |
| 9. The usage lines of `close`, `decide` and `adopt` | **Met.** |
| 10. The naming checks | **Met.** |
| 11. One revision of each source; skein with given keys | **Met, by hand for hearth.** hearth's lock names `github.com/fabrig-dev/skein.git` since its `5efba43` at 11:45. The item's `forgetic` pattern skipped it. In the second loop `$s` was empty, so `git show ":crates/…"` read skein's index and passed. The supervisor then checked hearth's pin `49725eca` by hand: it has `ask_given`. |
| 12. temper holds jig's wire version 9 | **Met.** temper pins jig `eceadcaf`, jig-repin.s1's landing. |
| 13. The test-time aims | **Missed for temper fuzzy**, recorded in ruling 139. Medians of three: jig focused 5.404 s (aim 7), jig fuzzy 6.993 s (8), temper focused 3.501 s (5), temper fuzzy 9.210 s (8). One more temper fuzzy run at 11:57 took 7.860 s. Its three slowest tests are named in `done.md`. |
| 14. Nothing else | **Met.** |

What the plan leaves for later:
- **The next plan,** `temper ctl`'s host, the client stories, the fake forge's host, the real-loop world and the process tier. Its drafts are in `findings/next/`.
- **The design text of decisions M to Q,** `later.md` rows 40, 42 and 43 (section 6).
- **jig's test mends** that ruling 128 deferred, `later.md` row 41.
- **setup.md's figures for the owned example's memory,** `later.md` row 36 (ruling 109).

## 5. What did not

2. **Eight design questions came up during the run, and five never reached the design.**
   - **What they were.** Each was a mismatch between tools.md and jig's contract, which the types show:
     - J: `TaskQuery.order` has no `Option`, and tools.md names no default;
     - K: tools.md's 50 rows, against the wire's ceiling of 32 in jig's `CEILINGS`;
     - L: `Read::Tasks` answers two groups with two cursors, and tools.md prints one;
     - M: 13 of jig's 23 `ActKind`s have no tool;
     - N: the inbox's two groups, two cursors;
     - O: two first pages within 32 rows;
     - P: `Read::Activity` takes no row count, so a cut page cannot read on;
     - Q: `Outcome::Proposed` keeps no recipient.
   - **When.** J at 04:57 and L at 07:04, from ready checks; K at 05:31, from temper 02.1's block; M to Q from 08:58 to 10:01, five blocks in an hour as temper 03 wrote the documents.
   - **Who answered.** No designer was joined (`plan wake designer` refused at 05:39). The supervisor wrote each under "For the user" with its default, and the decision was each as the default. development.md 10.4 has the designer decide a design gap "always", and amend the design in the same landing.
   - **The design.** A designer session on another machine wrote J, K and L into tools.md at 08:41 (`dba3c3fcb`). M to Q are `later.md` rows for "the designer's next pass". tools.md now contradicts the code on five behaviours, and the next plan's planner is reading it.
   - forge-setup's proposal 9 (the planner traces the design first) was adopted. It did not reach the upstream contract's types and bounds, where all eight lay.
   - Proposals 2 and 3.
5. **Test shape took eight steer points and still slipped.**
   - **What.** The hourly checks found the same classes at 04:40, 05:45, 06:51, 07:56, 09:02, 10:07 and 11:13: a setup typed again, a row that carries a flag, a bare size, two behaviours in one test, a test found by position.
   - **What was done.** Three README rules by steer (rulings 106, 116, 120), and steers at 04:53, 05:53, 07:05, 08:18 and 10:20. Ruling 122 asked for a mend at the file's next edit. No increment edited it again, and ruling 128 withdrew the mend at 09:14.
   - **What is left.** jig's `tests/core/tests/deadline_spans.rs` and `tests/client-protocol/tests/deadline_spans.rs`, `read_activity.rs`, `requests.rs` and `job_stories.rs`; temper's `src/tests/jig/acts.rs` and `help.rs` (section 9).
   - **Cause.** render's proposal 6 (a rule sent twice becomes a gate check) was adopted, and no check was built. The supervisor wrote at 07:56 that only a check on the diff would serve: bare `from_nanos(` is in 89 jig test files, and test names with "and" in a third of them.
   - The new crate's tests carry `_and_` in 19 names; some are the plan's own (12.1's, kept by ruling 120).
   - Proposal 6.
7. **Another run's change broke the closing repin, and was seen 76 minutes before it was acted on.**
   - **What.** live-text's smith-01.1 (`f925b180`, 09:34) replaced channel v2 with v3 in place and added `Start.live`. jig-repin had landed at 08:29. `plan downstream` at 10:13 showed smith, jig and temper failing.
   - **When.** The 10:07 check alerted it, and left it for the repins under the note of 06:58. temper-repin met it at 11:25; ruling 138 added `jig-repin.s1` at 11:29. A step added at 10:20 could have run beside temper 04, off the chain, though live-text's later smith steps (10:28 to 11:13) might have needed it again.
   - **Cost.** About 20 minutes at the end of the chain.
8. **"Done when" item 11 passed without checking hearth.**
   - **Cause.** The GitHub organization was renamed during the run. hearth's `5efba43` moved its lock to `fabrig-dev` at 11:45. The item and the README's Pins rule grep `github.com/forgetic/`. The loop read an empty revision, and `git show ":path"` read skein's index instead, which passed.
   - **When.** The "Done when" run saw it at 11:52; the supervisor checked hearth's pin by hand at 11:59.
   - smith, jig and temper still name `forgetic`. When they move, the Pins rule and items 11 and 12 pass for all of them, checking nothing.
   - Proposal 7.
10. **The temper fuzzy aim was missed, by a test known slow since client-api.**
    - **What.** temper's fuzzy suite runs as long as `fuzzy_walking_commit_cuts_latency_replay_and_observation_pressure`, 7.86 s alone at the end. The suite's median at the base was 7.04 s on this machine, and the plan's aim, 8 s, left under 1 s for noise. Temper's fuzzy gate ran above 8 s on 8 of 18 landings.
    - **Cause.** client-api's retrospective (section 9, item 3) named it: `run_replayed` formats both worlds with `Debug` at up to 600 iterations (`tests/engine/domain/src/walking.rs:832-833`). No plan has a step for it.
    - **When.** First at temper-01.1's gate, 05:24 (9.481 s). Each check called the excess noise.
11. **A process-tier story failed once, and its cause was not sought.**
    - **What.** At 08:00, `a_change_lands_by_deliveries_with_every_poll_raised` panicked in the engine, at jig-domain-core `documents/small.rs:11`: "source text is UTF-8 at the engine entrance". It passed the next three runs.
    - The alert named forge-kit's range, and no one diagnosed it. A reachable `expect` at the engine's text entrance is a debt (section 9, item 8).
12. **Smaller things.**
    - **temper 04.4 landed twice.** 04.3 had missed its Shape's `skein_lib::Deadlines`. 04.4's own check found it, and a second commit landed at 11:29.
    - **temper 12.2 waited.** It started at 06:29. Ruling 117, at 06:32, made it need 03.2's golden helper, and it waited 2.5 hours for it; a second thread then reconciled it. The README's row for temper 12 still names no increment that needs more.
    - **The Claude login expired** at about 05:05 and stopped three re-checks for 14 minutes.
    - **Hourly-check sub-agents read `supervision.md` whole,** two of seven, through `cat`; one read the README whole in two ranges. client-api's proposal 6, which the review had sent, says no.
    - **Two Evidence lines lost a space** ("passed207 tests"), from 11:10 on.

## 6. Decisions taken during the run

| Decision | Ruling | Reached the design? |
|---|---|---|
| J: `tasks` without `--order` reads by number; `--ended` changes no order; `Changed` and `Ended` have no argv form | 110 | **Yes,** tools.md 5.2 (`dba3c3fcb`, 08:41). |
| K: the tools' rows are 32, jig's wire ceiling | 114 | **Yes,** tools.md 12 (`dba3c3fcb`). `done.md` calls it "decision L". |
| L: a `tasks` call reads one group, the live one or with `--ended` the ended one; jig's cursor | 121 | **Yes,** tools.md 5.2 (`dba3c3fcb`). `later.md` rows 37 to 39 still say "next pass" for J, K and L. |
| M: an offered act with no tool prints its name and ` (no tool)`, with no call | 127 | **No.** `later.md` row 40. Code: `jig/render/acts.rs:44-47`. |
| N: the inbox's printed cursor carries a group byte before jig's | 130 | **No.** `later.md` row 42. Code: `render/lists.rs:195-226`, `argv.rs:523-537`. |
| O: the inbox's first page prints needs first, and updates on it only when both fit in 32 rows | 133 | **No.** `later.md` row 42. Code: `render/lists.rs:150-193`. |
| P: a page cut by `--limit` reads on by replaying the page whole; the cursor stays jig's | 134 | **No.** `later.md` row 42. Code: `render/activity.rs:9-41`. |
| Q: a restored proposal's kept line prints its identity without a recipient | 136 | **No.** `later.md` row 43. Code: `render/outcome.rs:213-226`. |
| An inbox read cut before Updates digests the watch's first window from all the rows read | 137 | No new design: tools.md 5.4 answered it. |
| The owned example's `memory_bytes` rise by each refusal's N | 109 | **No.** setup.md 8.1's figures are the designer's, `later.md` row 36. |
| jig 05.2 and 05.3 may grow the store profile's value, chunk and page by measurement | 118 | Not design: a plan allowance. |
| Joint stories share the fixture; a setup written once; rows carry data; one behaviour per test; no lint exception for a plan's words | 106, 116, 120 | Not design: testing-strategy.md 10 says them. They are README rules. |
| The `deadline_spans.rs` mend at its next edit, then withdrawn | 122, 128 | Not design. `later.md` row 41. |
| Scratch worktrees take an `r` suffix | 111 | Not design: the supervisor's practice. |
| `jig-repin.s1`, past live-text's smith | 138 | Not design: a repin step. |
| temper's fuzzy aim missed | 139 | Not design: for the user (section 9, item 2). |

Decisions A to I were recorded at 03:25:57, before the claim. The review vetoed C's fallback and installed Forgejo 17 and its runner instead.

## 9. Debts left in the code

For jig's `docs/development/tech-debt.md`, and for temper and skein, which have none. "Listed" says whether a debt file or `later.md` already has the entry. They are ordered by what they cost the next plans.

1. **tools.md lacks decisions M to Q.**
   - The code (all under `crates/temper-client-tools/src/jig/`):
     - M: `render/acts.rs:44-47`;
     - N: `render/lists.rs:195-226`, `argv.rs:523-537`, `runner/reads.rs:248-266`;
     - O: `render/lists.rs:150-193`, `runner/reads.rs:152-163`;
     - P: `render/activity.rs:9-41`;
     - Q: `render/outcome.rs:213-226`.
   - tools.md has no text for any of them.
   - `later.md` rows 37 to 39 (J, K, L) are stale: those are in tools.md.
   - Listed in `later.md`, rows 40, 42 and 43.
2. **temper's fuzzy suite is one test.**
   - `fuzzy_walking_commit_cuts_latency_replay_and_observation_pressure` takes 7.86 s of about 8 s. `tests/engine/domain/src/walking.rs:817-840` (`run_replayed`) formats both worlds with `{:?}` over 64 seeds of up to 600 iterations.
   - The fix needs `PartialEq` on jig's `Core` and temper's `engine::Domain` (client-api's retrospective, section 9, item 3).
   - Any plan that grows the worlds misses temper's fuzzy aim. Listed: ruling 139 only.
3. **temper's hand-written JSON mirrors jig's contract.**
   - `jig/render/document/` holds 5,112 lines in 23 files, one `encode_*` per contract type or variant. `act_answers.rs` alone has 23 `unreachable!` arms ("dispatched outcome"), and the crate 55.
   - Each change to `jig_client::contract` needs a matching edit here, found only by the compiler's match arms.
   - Not listed.
4. **Tool positions are bare numbers in production code.**
   - 23 sites compare `call.tool` with a literal: `jig/render.rs:291`, `:308`; `render/activity.rs:18`, `:22`, `:34`, `:42`; `render/document/read_sets.rs:16`, `:75`, `:79`, `:84`; `render/turn.rs:37`, `:63`, `:93`, `:99`; `runner/acts.rs:28`, `:377`; `runner/reads.rs:152`; and others. Watch slots are bare too (`local == 5`, `pages.get(&1)`).
   - The next plan adds tools (`remove`, `order`, `check`, `resume`, `ctl`'s verbs), which moves every position silently.
   - Not listed.
5. **jig's test shape, deferred by ruling 128.**
   - `tests/core/tests/deadline_spans.rs`: a loop that branches on `expected` (`:67-76`), the signed-in opener three times (`:15-16`, `:33-34`, `:51`), three behaviours in `an_aged_span_is_beyond_its_accepter_and_passed_up` (`:78-103`), bare seeds, keys and `from_nanos(15_000_000_000)`.
   - `tests/client-protocol/tests/deadline_spans.rs:172-204`: `a_named_proposal_keeps_its_span_across_client_and_engine_restart`, three behaviours.
   - `crates/jig-domain-core/src/tests/read_activity.rs:145-162`: a `tiny` flag that switches the setup, and its opener four times.
   - `crates/jig-client-domain/src/tests/requests.rs:7-12`, `:163-168`, `:234`: the save-then-send destructuring three times.
   - `tests/client/src/joint/stories/job_stories.rs:6`, `:16`, `:27`: its own `call`, `result` and `turn` beside `fixture.rs`'s `turn`, and bare `256` at `:124-125` and `:133`.
   - Listed: `later.md` row 41 has the first three; `requests.rs` and `job_stories.rs` are in no list.
6. **temper's test shape in the new crate.**
   - `src/tests/jig/acts.rs`: one opener typed five times (`:156`, `:164`, `:172`, `:187`, `:224`); `chat_checks_both_sides_of_its_bounds` (`:64-90`) branches; `decide_sends_its_two_forms` and `proposal_names_preserve_task_or_party` send one call and assert one value.
   - `src/tests/help.rs`: bare table positions in `GOLDENS` (`:15-22`) and `USAGES` (`:54-60`); `each_derivation_names_every_tool` (`:61-108`), the plan's own, with four behaviours.
   - 19 test names with `_and_`.
   - Not listed.
7. **Figures and goldens refreshed by hand.**
   - `docs/examples/owned.json:192` and `:825` were refreshed in three commits of this run (`4ef11dee3`, `e3430f46f`, `8c02b3b9e`). The worker's grew by 366,843,064,659 at 01.1.
   - jig's small profile, `crates/jig-protocol-config/src/profile/small.rs:321`, `:329` and `:330`, and its copy in `testing/jig-test-service/src/tests.rs:25`.
   - 40 goldens in `crates/temper-client-tools/src/tests/golden/`, with no write mode.
   - Listed: `later.md` row 36 for the owned figures; client-api's retrospective, items 4 and 16, for the rest; no debt file.
8. **An engine panic on text that is not UTF-8.** `crates/jig-domain-core/src/documents/small.rs:11` calls `.expect("source text is UTF-8 at the engine entrance")`. A process story reached it once at 08:00 (`logs/20261019T080041-3833.log`, lines 285-286). Not listed.
9. **Two GitHub organizations.**
   - hearth's lock names `fabrig-dev`; smith's, jig's and temper's name `forgetic`, in their `Cargo.toml`s too (25, 45 and 90 places). Cargo treats them as two sources, so they must move together.
   - Still `forgetic`: this plan's README (Pins, items 11 and 12) and five session files; temper's `docs/development/local-jig.md:19`, `:53`, `:55`, `:57`, whose `[patch."https://github.com/forgetic/jig.git"]` stops matching once `Cargo.toml` moves; smith's `docs/development/workflow.md:34` and README.
   - Not listed.
10. **jig's `tech-debt.md` is stale.**
    - `:197-198`, `:217`: the focused budget is 15 s since jig 04.4, not 30 s.
    - `:507-511`: the smith goldens' pin is `ff623361`, not `a277e922`. `crates/jig-protocol-transcript/tests/data/README.md:2` says `a277e922` too.
    - `:526-528`: `v9.rs` has 106,639 lines.
    - `:289`: 281 `unreachable!`, not 278.
11. **Files near or past 800 lines.**
    - No hand-written file crossed 800 in this run.
    - jig's over-800 files grew: `jig-domain-core/src/tests.rs` 2,974 to 3,003, `page/routing.rs` 1,103 to 1,130, `tests/acts.rs` 975 to 1,002, the scripted engine's `tests.rs` 2,257 to 2,283, `documents/objects.rs` 930 to 955.
    - skein's `requests.rs` is at 743 and its test `request_table.rs` at 752.
    - Listed: jig's `tech-debt.md:374`, with old figures.
12. **Not code.**
    - 41 local `tools/*` branches remain: 19 in temper, 22 in jig.
    - temper has no `docs/development/tech-debt.md`; items 2 to 4, 6 and 7 need one.
