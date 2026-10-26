# Retrospective: protocol

2026-10-14.

## 2. Outcome

The plan's "Done when", on temper's `origin/main` at `0ffa7b13`, whose
code is `b1994cdf`'s, the last code step. This session ran items 2 to 7,
9, 10 and 12 again. Items 1, 8 and 11 are the record's: session 17's last
thread ran them in `wt/temper-done` at `b1994cdf`
(`logs/temper-17/done-report.md`).

| # | Item | Result |
|---|---|---|
| 2 | The gate passes within budgets | Holds: `b1994cdf`, `scripts/check.sh passed: 1265 tests / 4.050 s; 46 tests / 15.276 s; 53 s in all`. The highest of the 116 code landings were 5.665 s and 19.914 s (14.2, `ae79f386`). |
| 3 | The crates and worlds exist | Holds: the loop prints nothing. |
| 4 | The web crates are gone | Holds, since 11.4 (`27197d49`). |
| 5 | No Python | Holds. |
| 6 | The pins held | Holds: jig `d91a89c4`, smith `dfc0fe57`, skein `b5d3a2c3`. |
| 7 | The engine worlds on jig's real store | Holds. |
| 8 | The process tier passes | Holds by the record: six tests in 43.0 s (`logs/20261014T214342-0.log`). |
| 9 | The live tier within its budget | Holds: `Completions: 7`, `Wall: 41 s`. The plan's first live run used 5 completions and 849 s, and failed on the model's words. Its numbers are only in the commit body's prose, which the item does not read (section 5, item 9). |
| 10 | Forgejo's facts recorded | Holds: the four `fails` lines are ruling 103's. |
| 11 | The new worlds' share | Holds by the record: 2.0 s focused and 11.2 s fuzzy, against 4.0 and 25.0. |
| 12 | The code rules | Holds as ruling 164 reads it. The variables and the threads hold as written. The lock's new names include 16 that the item's list lacks: transitive dependencies of `serde_json`, `serde`, `rustls` and smith's `smith-local-shell`. No temper manifest names any of them. Run at the base, as decision 24 did, the command diffs the base with itself and prints nothing, so it could not find this (section 5, item 4). |
| 13 | Only these aims | Holds. |

## 5. What did not

1. **The process tier was the first layer to join the service's halves,
   with its real limits.**
   - **What:**
     - 8 production faults surfaced between 13:10Z and 20:23Z:
       - limits: the hello's message bytes (136), documents the native
         client refuses (140), and the shipped profile's
         `conflicts: 0`, which refused every merge (153);
       - wiring: a worker start with no paths (144), a change with no
         resource (146);
       - behaviour: Forgejo's `changed` status (150), Adopt enabled
         before a party was confirmed (157), 12.2a's `projects`.
     - About as many test faults came with them (147's world, 151, 155,
       159, 160, 161).
     - In the last third, 16 steps landed in 6.7 hours.
   - **Cause:**
     - The worlds and fixtures set their limits by hand.
       `temper-service/src/limits.rs` has 61 comments that give a test
       fixture as the source of a shipped number.
     - No story ran the engine, the worker and the binary together on
       the shipped profile before 15.3.
     - Each increment proved at its planned layer: 38 of 83 at the unit
       layer, 25 at the world layer.
   - **First visible:** 13:10Z (ruling 136). The supervisor named the
     pattern at 18:36Z.
   - **Acted on:** 17.s1 (ruling 153) for `conflicts` only.
     - skein `e30fe58` (10:31Z, on `main` only) since asks that parts
       that meet connect from the start, and that each proof run above its
       tests.
     - Nothing yet says where tests take their limits from.
2. **authors found those faults by running the proof again.**
   - **What:**
     - 99 process and live runs for 12 steps, 3,711 s with the machine to
       themselves.
     - 15.3: 38 runs, of which 37 failed, across 12 threads and about
       124M tokens, 15:24Z to 20:49Z, with 11 blocks.
     - 12.2: 20 runs. 16.5: 17 runs.
     - Session 17 did the same in its world, in the build lane: 66 runs
       for 17.3, at about 4 s each.
     - Nine rulings set a budget of runs for one step each:
       - 145 and its amendment;
       - 147 to 150, 156, 158, 159 and 161.
       - They averaged 215 words, up to 455 (147).
       - Two had to be read again: 147 withdrew 145's amendment, and 158
         said that 145 counts failures by symptom.
   - **Cause:**
     - Section 8.8 says to fix a failing proof before landing, not where
       to look for its cause.
     - The process script was the only thing that ran the stories.
   - **First visible:** 13:04Z, 10.4's seven runs in 16 minutes. The
     supervisor noted it at 13:35Z.
   - **Acted on:**
     - Ruling 145 at 16:36Z, for 15.3 only; 15.3 still ran 13 more times.
       Ruling 148 for 17.3, ruling 158 at 20:41Z.
     - Section 10.4's "the same block twice becomes a standing rule"
       never made one.
9. **The live run waited out its deadline.**
   - **What:**
     - The first real run took 849 s of the script's 900, long after its
       coding task had failed.
     - A process proof waited 792 s behind it, and hour 21's queue ratio
       was 0.60.
     - Its 5 completions and 849 s are in the body of `b1994cdf` as prose.
       Only the passing run's `Completions:` and `Wall:` lines are what
       "Done when" 9 reads, and git does not read them as trailers either.
   - **Cause:** the live story stops only at its deadline.
   - **First visible:** 21:27Z.
   - **Acted on:** ruling 163 at 21:31Z, for the retry only.
14. **Two of three deferrals left no `later.md` row.** The designer's
    answers deferred two things:
    - pages of 2 rows and 128-byte texts (`b5dd62bc`);
    - no page of the forge's work until jig takes pages (`318a21ba`).

    Both are under "For the user", and only the second is in `STATUS.md`.
    Neither is in `later.md`; only the adopted repository's listing got a
    row (`2b2e9214`).
## 6. Decisions taken during the run

Rulings 101 to 164, 9 steers and 7 design answers. The supervisor
classified each in `done.md`.

| Rulings | What | Belongs in | Proposal |
|---|---|---|---|
| 103, 111, 117, 121, 122 and 140, and the answers for 15.2 and 15.3 | Forgejo's facts; statuses with no author; the core's ranges; the forge connection's deadlines; documents that fit the native client; adopted repositories; the forge's work in the inbox | In the design: `ba448afd`, `b65b41ef`, `f5e4672d`, `0be9dd34`, `b5dd62bc`, `e76cf0b2` (and jig `7b65bd1e`), `318a21ba` | — |
| 104, 109, 114, 124, 125, 127, 129, 144, 146, 160 | The code's means for what the design says | The design already says it | — |
| 132 | One process's io is its parts' io, summed | configuration.md, 7, beside performance.md, 6's "summed over its root and every component" | For the designer |
| 133 | A TLS forge's trust roots, priced as the accounts' are | configuration.md, 7 (the worst case), with the `transport` row | For the designer |
| 134 | A native shell idle before it loaded is a refused directory | client.md, the native shell; jig's part is in `later.md` | For the designer |
| 136 | The hello advertises raw message bytes; the domain keeps rendered ones | configuration.md, where the hello's states come from | For the designer |
| 137 | The fake forge reads a written document back through a call | testing.md, 4.2 | For the designer |
| 150's fix | Forgejo's `changed` file status is a modification | forge.md, 3.2, which names no status words | For the designer; section 9 |
| 153 | The shipped profile admits a merge's conflicts (4 paths of 255 bytes) | configuration.md, 7, as a requirement on the profile | 3; for the designer |
| 157 | Adopt is disabled until a party is confirmed | client.md, 4.4, with the other disabled states | For the designer |
| 102, 106, 126, 139, 142, 143, steer 04:21 | Sizes and split parts | development.md, 5.3 | 2 |
| 101, 107, 108, 110, 120, 123 | Edits outside Files | `main`'s `the implementation rules`, 8.2 has each since (jig-protocol's proposal 5) | — |
| 145, 147 to 150, 156, 158, 159, 161 | Diagnostic runs, one budget per step | the implementation rules | 1 |
| 112, 118, 128 | Disk | development.md, 11.4 | 10 |
| 115 | `plan resume` on an unloaded thread | The kit | 8 |
| 152 | The one-process share is measured serially | The plan | — |
| 105, 164 | A session's "Done when" fix; item 12 read as its rule | The plan's shape, development.md, 5.2 | — |
| 113, 116, 119, 130, 131, 138, 141, 151, 154, 155, 162, 163 | Order, fixtures, the live retry, other plan corrections | The plan | — |
| Steers 05:21, 08:21, 11:36, 19:39 | Helpers in the fixtures module; table rows; named numbers | Rules that exist: development.md, 5.2 and 5.6; `main`'s the implementation rules | — |
| Steers 04:29, 05:06, 14:35, 17:34 | Session files amended; 12.2a and 15.3a split off | Expired | — |

**Decided by the user during the run:** nothing was asked. The user
resumed the run after the reboot (07:00Z), and asked for the
supervisor's handover (17:34Z).

**Still with the user** (`decisions.md`, "For the user"):
- the plan's seven defaults, which the planner took under the user's
  delegation;
- and three designer answers:
  - an adopted repository is not listed until jig's core lists it;
  - pages of 2 rows and texts of 128 bytes;
  - no page of the forge's work until jig takes pages.

## 9. Debts left in the code

temper keeps no `docs/development/tech-debt.md` yet. These are for one,
in jig's shape, ordered by what they cost the next plan.

- **Facts go nowhere.**
  - The service's engine pops its root's facts and only counts them
    (`crates/temper-service/src/engine.rs:100-107`).
  - The worker drops its link's facts (`worker.rs:331`).
  - Nothing pops the worker root's facts, so its queue fills and the
    rest count as `lost` (`temper-worker-domain/src/domain.rs:227-231`).
  - The design says nowhere where facts go: a question for the designer
    first.
- **The shipped limits are copied from test fixtures.** 61 comments in
  `crates/temper-service/src/limits.rs` give a fixture as a number's
  source, and `reserve: 0` stays. Ruling 153 found one of them refusing
  every merge. Paid by proposal 3, and by deriving each number from its
  worst case (configuration.md, 7).
- **The socket count is a hand count** (`checks.rs:267-291`).
  - It has no term for the forge component's connections.
  - Its refusal reports sockets as "33 bytes" (`tests/checks.rs:185`).
- **Forgejo's file-status words** (`temper-forge-forgejo/src/client/files.rs:143`).
  - `changed` is read as a modification (ruling 150).
  - The transcripts hold only `added`, and Forgejo's per-commit lists
    also use `removed`, which `status()` may refuse.
  - Paid by a transcript of each word.
- **The store's generators live outside the repository.**
  `logs/temper-04/*.mjs` generated the store's schema and translations,
  rewritten for each increment. The next change to a stored record will
  redo them by hand.
- **`tests/codecs/tests/fuzzy_forgejo.rs:4` takes a median of 10.9 s**
  (4.2 to 16.0), against about 5. It runs 200 mutations a transcript
  case, and grew with each case.
- **44 of temper-service's 153 unit tests pass 20 ms.** Each builds a
  whole `Service` (`tests/fixtures.rs:393-475`).
- **The binary reads its configuration twice.** `lib.rs:264-300` reads
  the forges again with `.expect("checked …")`, and `lib.rs:206-229`
  reads the trust file again.
- **Copies of jig's reference service:** `process_io`
  (`checks.rs:292-304`) and `agent()` (`crates/temper/src/lib.rs:146-195`).
- **Rulings' code without a test:**
  - 133's pricing of TLS trust roots (`temper-service/src/forge.rs:296-320`),
    where every fixture carries no roots;
  - 136's separator underflow (`lib.rs:172-178`).
- **Two guesses.**
  - Ruling 134's "idle before it loaded is a refused directory"
    (`temper-shell-terminal/src/client.rs:350-357`), until jig says why it
    refused.
  - Ruling 137's read-back, a second call that another call could
    precede (`testing/temper-fake-forge-protocol/src/server.rs:153-170`).
- **Parallel types:** `startup/application.rs`'s `Transport` (line 32)
  repeats `settings.rs:31`. Its `Forge` (line 21) differs from
  `settings.rs:7` only in `api_token`. Their `trust` is a file name in
  one and the file's bytes in the other.
- **Six lint exceptions in tests,** which the plan's "Don't" list forbids
  and no ruling allowed:
  - `temper-service/src/tests/fixtures/startup.rs:1` and
    `src/tests/startup.rs:1`;
  - `temper-shell-terminal/tests/support/mod.rs:1`;
  - `tests/one-process/src/forge.rs:88`;
  - `tests/one-process/src/referee.rs:181` and `:483`.

  The run added 42 `#[expect(` in all, 23 of them `clippy::manual_map` in
  the store's records. It also added one `too_many_lines`
  (`temper-service/src/limits/names.rs:10`).
- **77 `unreachable!` arms in production code,** 45 of them in
  `temper-forge-forgejo`, from encoders and decoder tables that match the
  whole enum. Not audited.
- **The one-process world's person host** holds story-specific branches
  and two `thread_local!` cells (17.4; supervision.md, 21:38Z).
- **The live story waits out its 900 s after its task fails**
  (`crates/temper/tests/support/live.rs`). Proposal 13.
- **Files near the limit:**
  - `tests/system/src/composed.rs` 795;
  - `temper-protocol-store/src/tests/fixtures.rs` 790;
  - `temper-fake-forge-domain/src/api.rs` 785;
  - `temper-engine-domain/src/decision.rs` 785;
  - `temper-engine-domain-forge/src/boundary.rs` 775.

  Also the 15 large files the plan named, unsplit (`later.md`).
- **From `later.md`:** `ByteRing`, `PushDiagnostic` defined twice, and
  `Role::Adopted`.
- **Design text owed** for the eight rulings "not in the design yet"
  (section 6).
