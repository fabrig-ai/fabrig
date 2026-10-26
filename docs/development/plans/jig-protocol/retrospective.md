# Retrospective: jig-protocol

2026-10-13.

## 2. Outcome

The plan's "Done when", against jig's `origin/main` at `d91a89c`:

| # | Item | Result |
|---|---|---|
| 2 | jig's gate passes on `main` | Holds. `2048f39b7`'s Gate-Result reads "scripts/check.sh passed: 3813 tests / 15.041 s; 109 tests / 15.679 s; 42 s in all". The focused suite is past its 15 s budget. It passes because ruling 249 doubled the global timeout to 30 s. |
| 3 | The crates carry their new names | Holds: the grep prints nothing. |
| 4 | jig's worlds run on the real store | Holds: the grep prints nothing. `testing/jig-fake-store` (354 lines) stays for `ops`, as the plan said: `testing/jig-conformance` keeps its `Durable` impl for it (`harness.rs:27`), and `tests/ops` uses it. later.md's `ops` row retires it. |
| 5 | The process proof passes once | Holds: `jig-26.5` `dddb2ead` passed in one run (1 test, 241 ms). `logs/jig-26/proof-one-process.log` records the real binary, the smith agent, the native client, the kill and restart, and the chat with its answer. |
| 6 | The live proof ran once within its budget | Holds only after a change: the proof ran five times. Runs 1 and 2 failed on the fixture: a 69-byte prompt against the native client's 32-byte text limit. Runs 3 and 4 failed because the model answered "No brief provided". Run 5 passed in 1,889 ms with plainer wording. Rulings 279 and 280, and the user's 283, replaced "once". Each run stayed within 5 completions and 5 minutes; the run used 20 of its 40 completions (`sessions/jig-26.md:148`). |
| 7 | No browser or wasm32 entered jig | Holds: the grep prints nothing. |

## 5. What did not

10. **The focused suite outgrew its budget on this machine.**
    - **The timeouts.** jig's focused suite took 14.3 s by 17:00 (H2). The
      first timeouts came at 17:58 (smith) and 20:18 (jig). There were 55
      in all, 17 of them in the 22:00 hour.
    - **The rules loosened first:**
      - ruling 140 (20:22): three attempts;
      - ruling 153: the drift tests' ceilings;
      - ruling 165: 200 ms for generated tests.
    - **The first test-cost pass.** At 22:12 the user wrote: "please
      escalate to the designer, most likely there are inefficiencies in
      the tests themselves". Ruling 169 followed at 22:35, and the suite
      had recovered by 00:20.
    - **The second pass** (ruling 219, 05:35).
    - **What the passes bought.** About 5 s. The median focused time was
      9.75 s over the first 20 landings and 14.9 s over the last 20.
    - **The doubling.** At 08:42 the user wrote: "If the budget starts
      biting, we will double the budget temporarily". It bit at 09:08, and
      the budget was doubled at 09:11 (ruling 249). The last gate took
      15.041 s.
    - **Still doubled.** The timeout is still 30 s on jig's and skein's
      `main`. skein's commit (`886a7d4`) does not name the plan's end, and
      rewrote its header to say "within 30 seconds".
    - **The machine:**
      - carbon-forge's i5-8265U laptop CPU runs the suites 2.5 to 3.5
        times slower than smith's machine. This was found at 08:3x on
        10-13, at H17.5.
      - the code-graph indexer was the busiest process at 183 of 488 gate
        starts, using up to 778% CPU. It was reniced at 20:35 (ruling
        144), was back at 590% at 08:50, and the user had it wrapped at
        09:2x.
      - `plan check` measured nothing on this machine before the run.
    - **First visible:** 17:00 on 10-12. **Acted on:** 20:22 (retries),
      22:12 (the user), 09:11 on 10-13 (the doubling): 16 hours from
      first visible to the doubling.
13. **Downstream breakage was found late, and one repin has been undone.**
    - **`plan downstream` never ran.** Section 14.1 asks each check to run
      it once the plan has landed upstream code; it was run 0 times.
    - **The breakage surfaced only at the repins:**
      - **smith, 23:09 on 10-12:** 510 lint errors from the tests that
        skein-02.s4 generated.
      - **smith, 23:48:** its settle test took 2.05 s against its 2 s.
        This plan's base64 decoder costs about 200 ms in every smith
        process, and smith's test person reads 24,940 bytes one at a time.
        Ruling 186 held smith-repin for the user from 01:09 to 08:16 (7.1
        hours). The user's question at 07:14 was "Please elaborate on
        ruling 186, I didn't get it". smith-repin landed at 08:56.
      - **temper, 19:35 on 10-13:** 288 errors in 23 files. Ruling 282
        recorded the gap.
    - **After the run, smith-repin was undone.** smith's `83122ea0`, from
      the reliability plan on the other machine at 19:35, moved smith's
      skein pin from `8a34235` (smith-repin's) back to the older
      `cb3cefb`, calling it "current upstream main". smith's `main` no
      longer has smith-repin's skein, and skein-02.s8's fix is out of its
      lock.
14. **"Once" was the wrong plan for a live model.**
    - **Runs 1 and 2** failed on the fixture, which an offline run would
      have caught: a 69-byte prompt and charter against the 32-byte text
      limit. Run 2's answer check also matched the echo of the request.
    - **Runs 3 and 4** failed because the model answered "No brief
      provided", although the brief was there.
    - **The wait.** The step waited for the user from 18:21 to 20:13, 113
      minutes and the run's longest block, with no push notification. The
      whole run sent 3 push notifications.
## 6. Decisions taken during the run

183 rulings, 101 to 283. `done.md` gives each one's design commit, as
section 7.6 asks. Where each belongs:

| Rulings | What | In the design | Proposal |
|---|---|---|---|
| 37/162, 170, 176, 183/185, 187, 190, 202, 207/209/210, 224, 228/234, 244, 250, 256, 257, 264, 269/273, 275, 276, 277, 278 | The designer's answers: grants, chats, links, walls, undecodable turns, first start and first period, UTF-8 texts, a service's credential, an activity page's turns, durability by path, own watch subjects, refused starts, sign-ins while stopping, bounded pieces, the world's power loss | Yes: 39 design commits. `done.md`'s table names them, for example jig `87195d801`, `e484056a1` and `2fa034fbd`, and skein `63e81de`, `b638df3` and `2f89cf7` | — |
| 180, 201, 200, 203–206, 215, 217/218 | StoredTurn's 8 MiB bound; the clamp of smith's internal ceilings; typed links and staged activation; the caller's connector room as a sum; an unadmitted Busy; the map's sizes without transcripts; the keeper's versions | **Not in the design yet** (`done.md`) | For the designer |
| 138, 149, 151, 152, 154–156, 175, 182/184 | Codegen: boxed variants, minima, recursive facts, sums as pieces | skein `482691a`, `58cc905`; jig `3c5f607c6` | — |
| 108, 112, 114, 116, 122–125, 129, 145, 195, 197, 198, 209, 225, 227, 230, 247, 262 | Edits outside Files, expected values, removed mechanisms, private fields, files that moved, fixtures, visibility, version sweeps, harnesses | The plan's README only | 5 |
| 103, 110, 130, 132, 151, 188, 194, 196, 214, 252, 271 | Size: what it counts, and its margin | The README and code-organization.md | 6 |
| 121, 140, 153, 165, 167–169, 173, 174, 179, 186, 219, 242, 248, 249, 253, 268 | Test cost and budgets | 219 in skein's testing-strategy.md, 8 (`cb3cefb`). 249 is the user's, and still in force on `main` | 7, section 9 |
| 101, 120, 144 | Infrastructure: a primary checkout fast-forwarded; the indexer at idle priority | Not in development.md | 7, 17 |
| 105 | jig-10.1's trailers accepted as they are | — | 17 |
| 279, 280, 283 | The live proof's runs | This plan only | 14 |
| 254, 281, 282 | jig 25.6's pins; temper's gap | temper's STATUS.md, section 3; later.md | 12, 13 |
| The other plan-defect rulings, 69 in all | Shapes, Files, order, checks | The session files, amended in the run | 2, 3 |
| 118, 142, 147, 158, 163, 191, 193, 205, 231, 232, 243, 246, 260, 266, 270, 272 | Corrections of the run's own rulings and added steps | — | 1, 2 |

**Decided by the user during the run** (decisions.md, "For the user"):
- not yet running each crate's tests in one process;
- keeping optimized tool builds, since the build cache absorbs them;
- accepting the real store's test cost;
- measuring budgets on the slowest machine;
- doubling a budget that bites until the plan ends (249);
- moving smith's byte-at-a-time reader into a skein plan of its own,
  option (a), not started;
- more live runs (283).

**Still with the user:**
- the test-cost pass after the plan, which ruling 249's doubling waits on;
- a turn too large for a page;
- the gate lane's three kit changes;
- signing out on a shared machine;
- retention;
- the browser and wasm32;
- live text from workers.

## 9. Debts left in the code

For jig's `docs/development/tech-debt.md`. None of these is listed there
yet. Paths and lines are at `d91a89c`.

- **Ruling 249's doubled timeout.**
  - `.config/nextest.toml:9` reads 30 s, while its header (lines 1–2) and
    `workflow.md:41` still say 15 seconds.
  - The last gate took 15.041 s. After the doubling, 10 of 37 passing
    gates took over 15 s.
  - Restoring 15 s needs the test-cost pass that is still with the user.
- **Numbered splits with a copied loop.**
  - `jig-protocol-client-link/src/translate/` has `documents_part0` to `8`,
    `pages_part0` to `8` and `facts_part0` to `2`.
  - The same piece-decoding loop is written 51 times in 21 files, for
    example `documents_part0.rs:98`.
- **Orphaned goldens.** `jig-protocol-store/golden/{core,people,tasks}/v1`
  hold 2,078 files that no test reads; `tests/store-codecs/tests/drift.rs:45-57`
  and `fuzzy_decode.rs:42` read only v2.
- **Long translation functions.**
  - `jig-protocol-client-link/src/translate/asks.rs:574` `ask` (182
    lines).
  - `jig-protocol-engine-clients/src/translate/asks.rs:9` `ask` (160
    lines).
  - `jig-protocol-store/src/codec/people/asks.rs:348` `encode_ask` (156
    lines) and `:506` (120 lines).
  - Production functions over 60 lines went from 292 to 389. Of the 94 new
    ones, 29 are in engine-clients, 23 in the store and 12 in client-link.
- **Lint exceptions in production code** went from 214 to 360.
  `#[expect(clippy::manual_map)]` went from 33 to 159 (for example
  `jig-domain-accounts/src/domain.rs:343`), and `too_many_lines` from 69
  to 76.
- **`struct_excessive_bools` in test worlds**, against the plan's "Don't"
  list: `tests/channel/src/referee.rs:10`, `relay.rs:42`, `world.rs:39`
  and `processes.rs:19` (jig 27.1 and 27.3).
- **Arms that the types could remove.**
  - `jig-protocol-accounts/src/translate.rs:40,47,48,54,56,82,98`: arms
    for variants the engine never meets.
  - `jig-shell-native/src/files.rs:280` and `shell.rs:181`: jig's first
    `panic!`s in production code.
  - `testing/jig-test-worker/src/domain.rs:285,431`: `unreachable!`.
- **Files past 800 lines that grew:**
  - `tests/ops/src/conformance/mod.rs` (+104 lines, now 1,079);
  - `jig-domain-core/src/tests.rs` (2,987);
  - `jig-view/src/owner_deployment_words.rs` (1,109);
  - `jig-domain-core/src/routing/tasks.rs` (1,449).
- **Tests over 40 lines, written or grown in the run:**
  - `jig-protocol-charter/src/tests.rs:11` (57);
  - `jig-domain-host/src/tests.rs:140` (51);
  - `tests/client/src/joint/stories/fault_stories.rs:143` (45);
  - `tests/boundary/src/tests/graph.rs:294` (43);
  - `jig-domain-core/src/tests.rs:1691` (43);
  - `tests/core/tests/fuzzy_watches.rs:11` (41).
- **Steps that added many tests after ruling 219**, candidates for table
  tests:

  | Step | Tests added |
  |---|---|
  | 27.1 | 28 |
  | 31.5 | 20 |
  | 32.4 | 19 |
  | 33.2 | 18 |
  | 31.3 | 17 |
  | 31.6 | 17 |
  | 18.2 | 15 |
- **Generated volume.**
  - 187K generated lines in 9 files;
    `jig-protocol-client-wire/src/generated/v7.rs` alone is 99,451 lines.
  - The client wire went from v1 to v7 in the run.
  - Its build cost was not measured.
- **The fake store** stays for `ops` (later.md).

Outside jig:
- **skein:**
  - `.config/nextest.toml` also reads 30 s (`886a7d4`), and does not say
    when it ends;
  - skein's fuzzy suite reached 56.8 s of its 60 s (skein-02.s6).
- **smith:**
  - its STATUS row is still "active";
  - its skein pin went backwards (`83122ea0`);
  - its test person still reads one byte at a time, which waits for the
    skein plan of option (a).
- **temper:** its reconciliation, with 288 errors, is later.md's last row.
