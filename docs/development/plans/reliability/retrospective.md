# Retrospective: reliability

2026-10-14.

## 2. Outcome

The plan's "Done when", on `main` at `a8f2b7e3` (smith) and `bec13dcc`
(skein):

| # | Item | Result |
|---|---|---|
| 1 | Every step landed or dropped | **Holds.** `plan status` shows 108 of 108 landed. Its "Ledger and origin/main disagree" lines name smith-12.1 (a blank line cut its `Plan:` trailer off the trailer block) and smith-10.s1 (its commit says `Step: smith-10.10`): rulings 13.28 and 13.76. |
| 2 | Each session's own "Done when" | **Holds, as ruled.** done.md maps it to final reports; nobody ran the commands (section 5, item 11). Every `rg` and `git` item, run here once, holds except smith 08's `rg -n 'window: 1,' …/responses.rs`. It prints line 1113, a `cfg(test)` fixture, which ruling 13.82 accepts. The `cargo nextest` items name tests in the suite that the last gate on `main`'s tree passed. |
| 3 | Session 13 ran its seven steps | **Holds as written, not as meant.** Both commands print 7. Four `Calibrated:` lines say "none" (13.2, 13.4, 13.6, 13.7). 13.1, 13.3 and 13.5 calibrated part of their probes. done.md names only 13.6 and 13.7 as not calibrated. |
| 4 | The smoke suite passes on `main` | **Does not hold.** 12.3's run of `smoke` had 4 attempts pass and 2 `first-request` attempts fail. The unnumbered ruling of 21:23:26Z classed them as the known uncalibrated configuration, "gating nothing". Not run again here: it is live. |
| 5 | Both gates within budget | **Holds, but its command no longer shows it.** In both repositories the command prints `none (Markdown only)`, since the newest commit is Markdown. The plan's last gates: smith `20ab17e4`, 2,000 tests in 4.211 s and 38 in 9.792 s; skein `7efc3072`, 2,072 in 4.275 s and 99 in 10.903 s. The run's worst: 9.933 s focused, 19.841 s fuzzy. |
| 6 | smith's report is published | **Holds.** `docs/development/benchmarks/smith-codex-2026-10-14.md`, from `5f4db659`, with its erratum. |

Against the README's purpose:
- **"Implement the spikes' fixes cleanly":** done as planned. The
  code-review agent finds the plan's "Don't" list kept in most places,
  and broken in four kinds (section 9): parallel types, second
  mechanisms, several behaviours under one test name, and new production
  lint exceptions.
- **"A harness that shows each fix works against a real provider":**
  - The harness exists and ran 660 attempts.
  - Of the 51 increment ids that session 13 was to prove, 22 are proven.
  - Of session 00's probes, `credentials` calibrated 10 of 10;
    `follow-up` and `history` did not.
  - The others stand on their deterministic tests, as decisions.md 12.4
    allows.
- **The comparison against Codex** is named "not an aim". The report's
  findings lead it:
  - 5 of 10 compaction attempts failed;
  - every fixture attempt failed the protected-tests scope check, for
    both agents;
  - smith reused the cache on the fixtures at about 0.2 times Codex's
    rate.

## 5. What did not

6. **Answers that rested on unchecked premises.**
   - **11 answers were reversed,** 10 of them on a number or a premise
     nobody checked:
     - one line allowed in a reader that needed two;
     - two signals in flight, which io.md 3.4 forbids;
     - an archive named without checking it had its helpers;
     - instructions given to 05.s1, which had already landed;
     - a misread HTTP 400;
     - "settings already check effort", which stood 4 h 41 min;
     - a `smith check` JSON field that does not exist, which stood 73 min;
     - the wrong owner;
     - a probe that had not run.
   - **Commands quoted.** 23 of 156 unblocks quote a counting command.
   - **The designer** was quick and wide twice:
     - wake 22 changed six design files in four minutes, and needed three
       more wakes to correct (23 to 25);
     - wake 9 inferred a cause that wake 11 then measured and corrected.
   - **Cause:** 10.4's "check an answer against the code before sending it"
     is a sentence that nothing checks, and the answers went out in about a
     minute.
   - **First visible:** 13:50 on 10-12, the first reversal. The supervisor
     wrote the lesson down at 18:29 and again at 21:45 on 10-13.
7. **Gates that failed after the author's own checks.**
   - **70 of 189 gates failed:**
     - 54 on tests;
     - 13 on clippy;
     - 3 on compile errors.
   - **The tests.** Most failures were expected values in end-to-end,
     fuzzy and smith-bench tests that no focused run of the increment
     included. `smith::end_to_end a_first_run_signs_in_and_ends_with_a_commit_in_place`
     failed four sessions' gates, and `legacy_recordings_terminals_and_emission_metadata`
     four.
   - **The clippy failures.**
     - In w1–w35, 10 of the 12 came after an edit that followed the last
       passing clippy: a conflict's resolution, a gate's fix, or a fixture.
     - Meanwhile clippy ran about 4 to 5 times per landing, up to 20 (w61,
       10.s4).
   - **Cost.** The 22 failed lands of w36–w69 cost 49.1M and 151 minutes.
     The 47 failed gates of w1–w35 took 44 minutes of the exclusive gate
     lane.
   - **Cause:**
     - The focused run before merging covers the crates the author
       names, not the crates that depend on them.
     - 8.3 forbids whole suites outside the gate, though the focused suite
       runs in 4–10 s.
   - **First visible:** 13:32 on 10-12. Not acted on.
10. **Proving fell short, and the record says less than it seems.**
    - **Session 13.** Four of its seven steps calibrated nothing. Done
      when 3 counts `Calibrated:` lines, so "none" passes it.
    - **Calibration is recorded only in commit bodies.**
      - Only `credentials` has a `[calibration]` block in its task file,
        and its `failed_before` is "none: no runnable control was named".
      - `calibration_suite(_probe)` ignores its argument.
    - **A diagnostic step that blocked at every run.** 06.s2, a hunt for a
      silence:
      - it raised 14 blocks in 105 minutes (21:25–23:10Z), one for each
        diagnostic run or world-library allowance (160, 172, 173, 183,
        191, 192 lines);
      - README rule 11 counts runs per failure, and the step had no budget
        of its own.
    - **The supervisor never totalled the spend** for the README's "$60"
      report. It stayed far below, but unchecked.
    - **First visible:** 16:56 on 10-12 (13.1's partial calibration), and
      21:25 (06.s2). Ruled one by one.
13. **One provider error, a missed push, and a supervisor fixing primary
    checkouts.**
    - **The crash.**
      - At 06:01:17Z on 10-13 the coordinator's turn, 16.7 hours long,
        ended with "Selected model is at capacity". The ring failed ("no
        turn in progress").
      - The supervisor's push to the user came back "Not sent — this
        terminal is active". It was the only push of the run.
      - The supervisor ran `plan resume` itself at 06:11:57, which 10.5
        does not say it may.
      - The coordinator logged the cause as "the user's request".
    - **The check that followed.** `plan check` then failed:
      - skein's primary checkout was one commit behind, from carbon-forge;
      - smith's held the untracked `.codebase-memory/`, and with it `main`'s
        gate failed.
      - The supervisor repaired both in the primary checkouts, by hand: a
        `git pull --ff-only`, and a line in `.git/info/exclude`.
    - **The supervisor's weekly limit** stopped it for 29.5 minutes at
      20:32 on 10-13. Its entry says "my watch ended".
    - **Visible and acted on:** all were visible at once and acted on within
      minutes. The causes recorded are wrong.
14. **Another machine on the same repositories.**
    - **carbon-forge's landings.** It landed 19 smith commits and 47 skein
      commits during the run, for jig's `jig-protocol` and `test-budgets`
      plans. Two consequences:
      - its skein change broke smith against skein's main for an hour
        (item 5);
      - the same rule landed twice: skein `codec.md` (`58cc905`) and smith
        `limits.md` rule 6 (`4c7b15e6`), "one piece bounded by its sum".
    - **Step numbers.** Its step numbers (`smith-01.1`, `smith-02.1`) clash
      with this plan's in blame, and only the `Plan:` trailer tells them
      apart.
    - **Gate times there.** Its smith gates ran the focused suite in 16.5 to
      24.1 s, over 15 s, on four commits. Budgets are the reference
      machine's (11.5), but nothing names the machine in the trailer.
    - **Visible and acted on:** visible in `plan downstream`, unread.

## 6. Decisions taken during the run

decisions.md section 13 holds 133 rulings. About 49 more unblocks rule
something that section 13 does not cite; about 20 of those carry a
designer's answer into the session files. The 26 design answers took 32
design commits: 26 in smith, 6 in skein.

| Rulings | What | In the design | Proposal |
|---|---|---|---|
| 13.1, .3, .4, .8, .19, .20, .24, .32, .36, .40, .43, .111 (README rule 8) | The edits a change in place forces outside Files | The plan's. development.md 8.2 has a narrower rule | 4 |
| 13.9, .46, .105 (rule 9); 11 splits; exceptions 13.15, .26, .29, .80, .96, .102 | Size as a guide up to 700; past it, a split | The plan's | 1 |
| 13.21 (rule 10) | A landed step checked on its own branch | The plan's; development.md 7.3 | 15 |
| 13.25, .38 (rule 11) | Diagnostics never committed; three runs per failure | The plan's | 13 |
| 13.45 (rule 12); 13.28 (Pass on) | The Pulse words grepped before merging; trailers in the last paragraph | The plan's | 15 |
| 13.50, 13.67 (rule 3) | A live batch stopped by the provider runs once more; the `EINVAL` retry, until skein-07.s2 | The plan's | — |
| 13.41, .44 | Codex from a private copy of 0.160.0 | The README's "Live services"; not in benchmarks.md | 19 |
| The 26 answers | Compaction, limits, the product's commands, sign-in, the terminal | In the design (`0f1d72ba` … `dafea731`; skein `bdbe1fd` … `ed8d235`). done.md names 11 | — |
| 13.14, .91, .98, .99 | The harness: a build lock, the baseline selector, a refused start counts no request, a 4xx before output charges nothing | Not in benchmarks.md yet | 19 |
| 13.116 | Effort as text of at most 32 bytes | Not in limits.md; the literal is in five places (section 9) | 19 |
| 13.39 | io drops a second child signal in flight | A skein question, not in io.md | 19 |
| 13.132 | A large effect is disjoint ranges over three attempts | Not in benchmarks.md, whose 3.1 and 11.5 name no threshold | 19 |
| 13.131 | Audits of every grader disagreement; session 12's line at 149 attempts | The plan's; its README still says 126 | — |
| Steer of 15:46:42Z on 10-13 | A fix after the smoke takes one more smoke | A Pass-on only | — |
| 21:23:26Z, unnumbered | The smoke's two `first-request` failures gate nothing | Not in section 13 | — |
| 00:52:04Z on 10-14, unnumbered | The compaction threshold goes to the user, recommending 80% | Not in the design, nor in "For the user" | user |

**Still with the user** (decisions.md section 10, and the run's ends):
- **Borrowing another CLI's login (7.9):** never answered. The run worked
  read-only, under README rule 7.
- **The smoke before merging (12.3):** open; the run worked that way.
- **The skein pin tags (12.2).** Unpublished: smith commits from 10-09 to
  10-12 pin skein revisions that exist only in this machine's skein clone.
  The push is `git -C ~/src/rust/skein push origin
  'refs/tags/smith-pin/*'`, from this machine.
- **The working tier, `gpt-6-astra` at high (12.17):** never answered.
  Session 12 spent $7.82 on it.
- **Local `sub_agents` on by default (07.13):** put to the user at
  18:36:41Z on 10-12, recommending off. Unanswered. The benchmark arms ran
  with it on (section 9).
- **The compaction threshold and contents (4.7):** routed at 00:52:04Z on
  10-14, recommending 80%. It is not in section 10.
- **temper:** 83 errors against smith's `main`, from before the run, wait
  for jig's and temper's own plans.

## 9. Debts left in the code

For smith's `docs/development/tech-debt.md`, beyond what it already lists.
Paths and lines are at smith `a8f2b7e3` and skein `bec13dcc`; each debt
names the step that left it. Ordered by what they cost the next plan.

- **Compaction answered with a call or with prose** (08.6, 08.s1).
  - The summary request keeps the conversation's tools and tool choice
    (`crates/smith-domain-session/src/compaction.rs:107`, `:256`).
  - A summary that calls a tool ends as `NoSummary::Malformed` (`:323-331`).
  - 5 of 10 attempts in 12.6 failed so.
  - The default threshold, 4/5 (`smith-agent-service/src/profile/quantities.rs:184`),
    waits for the user.
- **The fixtures' protected `tests/`.**
  - `benchmarks/tasks/fixtures/{01-backoff,02-line-decoder,03-planner}/task.toml:9`
    protect `tests/`, while their prompts ask for regression tests.
  - All 30 attempts of 12.4 failed the scope check, for both agents.
  - The comparison cannot pass until the tasks get a new version.
- **Calibration lives in commit bodies.**
  - Only `benchmarks/tasks/probes/credentials/task.toml:48` has a
    `[calibration]` block, with no failing control.
  - `benchmarks/src/fix.rs:14` `calibration_suite(_probe)` ignores its
    argument (00.29).
  - The probes suite runs calibrated and uncalibrated probes alike.
- **smith-bench keeps four smith faces, and renders the product's settings
  through the old template.**
  - `benchmarks/src/agent/smith/events.rs:54-63`.
  - `prepare.rs:126-169`.
  - `settings.rs:98-172`, converted by `product_document`, `:207-279`
    (10.15, 10.16), through a hand-kept list of field names.
  - The pins `benchmarks/agents/smith/{smoke,first-request}.json` keep the
    old shape.
- **Benchmark arms run with sub-agents on** (`benchmarks/src/agent/smith/settings.rs:164`,
  `:270`). The product's default is off (`smith-local-shell/src/settings/layers.rs:11`).
- **Three price catalogs that disagree, up to twelve times** for one model:
  - `benchmarks/agents/prices.toml:2-10` (12.9);
  - `crates/smith-local-shell/src/presets.rs:109-120` (10.13);
  - the pins.
- **Parallel types and second mechanisms.**
  - Three account-source enums: `smith-local-shell/src/credentials.rs:79`
    (10.19); `smith-local-service/src/credentials.rs:22`, `:59` (10.7).
  - The effective-limit projection written twice:
    `smith-agent-service/src/profile/effective/channel.rs:7-174` (04.21) and
    `smith-local-service/src/profile/effective/channel.rs:9-163` (04.22).
  - Startup paths resolved three times: `compose.rs:408-413`,
    `compose/files.rs:43-45`, `credentials/sign_in.rs:172-174`.
  - The teardown bound derived again in smith-bench, `settings.rs:175-189`.
  - Operator words outside `smith_events::words`:
    `smith-local-shell/src/front/interactive.rs:172-230` (10.24).
- **The binary parses its command line three times**
  (`src/main.rs:8-16`: `credentials`, `compose`, `check`), in an order
  that decides what runs.
- **A String-typed face beside the typed refusal.**
  `crates/smith-agent-shell/src/config.rs:231`, `:246`, `:304`, `:876-891`
  return `Result<_, String>`. `config/refusal.rs:1-23` keeps it "until that
  face is removed", and no step removes it.
- **A known notice sent as an unknown one.** `NoticeKind::Unknown(b"not_contained")`
  (`smith-agent-service/src/events.rs:222`, 03.10), matched on its bytes in
  `front/interactive.rs:177-180`.
- **Literal bounds repeated.**
  - Effort, 32 bytes, in five places: `smith-domain-run/src/charter.rs:420`,
    `smith-domain-session/src/compaction.rs:38`, and three more.
  - Model name, 128 bytes, at `compaction.rs:37` and `session.rs:2005`.
  - The codec ceilings own both.
- **Hotspots grown in small steps.**
  - `smith-local-service/src/process.rs`: 918 → 1,420 lines.
  - `service.rs`: 846 → 1,339.
  - `smith-agent-service/src/service.rs`: 2,589 → 2,905.
  - `responses.rs`: 1,534 → 1,852, and its `fact` at `:500` grew from 135
    lines to 235.
  - Crossed 800 during the run: `benchmarks/src/batch.rs` (954),
    `smith-local-shell/src/local_host.rs` (870),
    `smith-protocol-channel/src/translate.rs` (871).
  - New over 800: `smith-local-protocol/src/store/machine.rs` (818).
  - 50 new production functions over 60 lines, most just under clippy's
    100.
- **New lint exceptions in production.**
  - `wildcard_enum_match_arm`: `smith-local-service/src/credentials.rs:441`
    and `credentials/{private.rs:106,transport.rs:88}` (10.6, 10.7).
  - `manual_map`, module-wide: `smith-local-service/src/stream.rs:5` (03.7),
    and `translate.rs:573`.
  - `disallowed_methods`: `smith-events/src/words.rs:12`.
- **Panics on a wrong variant.**
  - 46 `unreachable!` in production, e.g. 8 in `store/machine.rs:318-633`
    (10.1).
  - In skein, 02.6's `Done::Window` added such arms in
    `crates/skein-io/src/file_layer/store.rs:610-658` and `layer.rs:393-578`.
- **Tests.**
  - Several behaviours under one name: `benchmarks/tests/events.rs:1458`
    holds 33, and is over 1 s; `offline.rs:188` holds 10.
  - The harness's offline tests grew from 99 to 155, so later.md's "95
    tests, about 8 s" is out of date.
  - Real-clock limits of 1 s in `tests/end-to-end/support/mod.rs:553`,
    `:641` and `support/login.rs:96`.
  - `tests/local-process/src/oauth.rs:138` still accepts the removed
    driver's page.
  - World constructors of 150–300 lines: `tests/local/src/world.rs:458`
    `with_mode`, 301; `tests/local-process/src/referee.rs:88` `observe`,
    231.
- **Words.**
  - "native" and "legacy" survive the removed face: `settings/resolve/relationships.rs:179`
    `fn native`, and `tests/end-to-end/support/live.rs:76` `fn legacy`.
  - `benchmarks/src/probe.rs:34` keeps "Immutable", landed before rule 12.
- **Raw terminal input without a user.** skein's `crates/skein-shell/src/terminal.rs:28-60`
  and `ring/terminal.rs:33-61` (02.6) have no smith caller, since 10.10
  kept the terminal's own line mode.
- **Formats changed under one version.**
  - Thirteen steps changed channel 2, charter 2, transcript 3 and events 1
    in place, as README rule 5 allowed.
  - smith-bench keys its readers to commit hashes (`faces.toml`).
  - The generated codecs move with every skein repin: 10.s2 regenerated
    about 751 unrelated lines (13.105).
