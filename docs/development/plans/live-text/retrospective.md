# Retrospective: live-text

2026-10-19.

## 2. Outcome

`done.md`'s "Done when, run 2026-10-19T11:33:25Z" holds each command's output. After the last landing, the supervisor's Sonnet sub-agents ran the commands in detached worktrees of skein `9ffcb60` and smith `cd76fb64`.

| Item | Result |
|---|---|
| 1. Every step landed or dropped | **Met.** `plan status`: 8 of 8 landed. |
| 2. skein's fake LLM writes pieces, held until released | **Met.** The three greps print nothing. |
| 3. smith's channel at version 3, alone | **Met.** `channel-v3.schema` alone, then `1`. |
| 4. The host asks, and hears the pieces | **Met.** |
| 5. Pieces on the interval, drops counted, no second switch | **Met.** No `live_text` field remains. |
| 6. One skein pin, which has the release | **Met.** skein `5c8ae967`. |
| 7. The memory floor in smith's shipped-values test | **Met.** It prints 1. The headroom above the 490,000,000-byte floor fell from about 2.82 MB at the base to 2.26 MB, by 20% (section 9). |
| 8. The test-time aims | **Met on a second measurement.** As written, smith focused failed: the command's own logs in the worktree made `smith-bench`'s probe test refuse a dirty tree. It was re-measured at 11:36 in a clean worktree of smith `ff623361`, with the logs outside the tree (ruling 106). Medians of three package sums: skein focused 4.45 s against 5.7, fuzzy 6.93 s against 8.4; smith focused 11.62 s (11.38, 11.62, 12.67) against 12, fuzzy 17.88 s against 18.5. |
| 9. Each session's own "Done when" | **Met.** |
| 10. Nothing else checked | Nothing to run. |

smith's two aims passed by 3%, and one of the three focused runs was past its aim.

What the plan names but leaves for later is in `later.md`'s nine rows. They include jig's and temper's parts, their repins, and the allocation in `Peer::release`. One row is stale (section 5, item 11).

## 5. What did not

11. **Ruling 104 and `later.md` say what the code does not.**
    - **The ruling and the row.** Ruling 104 says "the two release stories call no check_memory", and `later.md`'s `Peer::release` row says the same.
    - **The code.** smith-02.2 landed release stories that call `check_memory` (`tests/agent-process/tests/live.rs:20`). They use a fixed 64-byte `ReleaseName` (`tests/agent-process/src/world.rs:356`), so the release allocates nothing.
    - **So the row is stale.** The debt that remains is skein's: `Peer::release` allocates a `Box` (section 9, skein item 2).
    - **First visible:** here.
    - **Acted on:** no.
## 6. Decisions taken during the run

Six rulings, 101 to 106, each one smith commit, and two steers.

| Ruling | Commit | What | Reached the design? |
|---|---|---|---|
| 101 | `7eaba5ab`, 09:33:49 | smith 02's ready check: `Time`'s missing operators, `profile/tests.rs` in Files, `call:` at the worlds, `release_after` and `maximum_queries` | Plan only; no design text needed |
| 102 | `e225df8a`, 10:26:55 | README rule 7: testing-strategy.md section 10 before a reference test; landed tests stay | Plan only; section 10 already says it |
| 103 | `e2b9f45a`, 10:36:44 | `service::Limits` carries `live_interval` from `profile::derive` into `Live::new`, apart from the repin's commit | The interval was already limits.md 2.3's; its carrier is code |
| 104 | `544ebe44`, 10:56:28 | The re-check's Shape corrections for 02.2, with a `later.md` row | Held bytes waiting for room were already channel.md 7's. The rest is code. The `later.md` row is stale (section 5, item 11) |
| 105 | `b1df8148`, 11:01:07 | The one-frame world checks loss at the held bound and one past it; `Service::lost_live_bytes()` | The count was already channel.md 7's; the accessor is code |
| 106 | `fabc3116`, 11:36:14 | Item 8's miss as written, and the clean re-measure | Plan only |

Both steers are in the table: steer 1 (10:26:53) carried point 1, the waits, and point 2, which ruling 102 recorded; steer 2 (10:56:42) carried ruling 104. Point 1 belongs in the implementation rules, which it did not reach (proposal 1).

**"For the designer"** (`decisions.md`): five items the plan asked the designer to write. **None reached the design.** smith has no design commit after the plan's `0e9c3308`, except STATUS rows (`c0c89ddc`, `7b4dfc07`). skein's `fake-llm.md` has none after `d4ef168` (07:39).
1. Reword `fake-llm.md` 2.2 and 6: the release comes from the world or the loop, and a reply scripted whole keeps its bytes. 2.2 still says "The peer as a process of its own ... refuses a script with one at its start" (lines 97 to 100) and "A reply scripted whole is one piece" (line 80). Both now disagree with the code.
2. `events.md`: a field for the live bytes not projected.
3. llm.md 6: "main conversation only", read as the channel's rule.
4. shell.md 6.5 and `live_interval`.
5. `fake-llm.md` 2.2: no fault while a reply is held.

**The design questions and "For the user".**
- The five design questions were answered (a) in the decision before the plan landed.
- The ten "For the user" items were all accepted in the decision at 09:02:52 (`2efd6c92`), before the claim.

## 9. Debts left in the code

For smith's `docs/development/tech-debt.md`, which lists none of these, and for skein, which has no such file yet. They are ordered by what they cost the next plans.

**smith**

1. **smith-bench's probe test fails in any dirty checkout.**
   - **Where:** `benchmarks/tests/probe_command.rs:105` builds an arm from the real checkout, and `benchmarks/src/arm.rs:93-98` and `probe.rs:593` refuse a dirty tree.
   - **What it cost:**
     - "Done when" item 8;
     - six of the authors' runs, 536 s;
     - the re-check's 2,381 of 2,382;
     - the planner's run at the base.
   - **Since:** the guard dates from `98163c75`.
   - **Needs:** the test builds from a synthetic repository, as `arm.rs`'s unit test at lines 408 to 430 already does.
2. **The memory headroom fell 20%.**
   - **How much:** above the 490,000,000-byte floor, from about 2.82 MB to 2.26 MB.
   - **Cause:** `HeldByte { byte, arrived: Time }` keeps one record for each held byte, and there is a `Gap` queue beside it (`crates/smith-agent-service/src/live.rs:60-75`, `94`).
   - **Risk:** the next plan that adds to the worst case meets the floor.
3. **Repeated test code and bare literals.**
   - smith's tests have six identical `other @ (Request::Piece { .. } | Request::Hold { .. }) => panic!` arms.
   - The 64-byte hold bound is a bare literal at five sites.
   - The frame kinds `0x0109` and `0x0112` have no named constant.
4. **Landed tests that break testing-strategy.md section 10.** Ruling 102 let them stay.
   - `tests/channel/tests/live.rs`: repeated setup, two behaviours in a test, bare indices at lines 61, 72 and 82, and `sent > 0 && sent < 8`.
   - `crates/smith-agent-service/src/live.rs:335` asserts three behaviours, and `AT_ZERO.saturating_add(...)` appears eight times.
   - `tests/agent-process/tests/live.rs`, at lines 25, 54, 66 and 158.
   - `tests/channel/tests/referee.rs:86`.
   - `content_observations.rs:90`.
5. **Records with no producer.** `LiveKind::Reasoning` and `LiveKind::Input` have no producer. Only tests read `lost_live_bytes`.
6. **Taste.**
   - `ends: [Option<(u32, u64)>; 3]` is indexed by bare 0, 1 and 2 (`crates/smith-host-protocol/src/live.rs:30-61`).
   - `[0; 4096]` repeats `PIECE`'s value (`crates/smith-agent-service/src/live.rs:221`).
7. **`later.md` and `STATUS.md`.**
   - The `Peer::release` row is stale (section 5, item 11).
   - The repin row leaves out jig's four paths from v2 to v3, and temper's `encode_start` (audit A32).
   - temper's `STATUS.md` (lines 124 to 128) still says it "waits for the live-text plan".
8. **Timeouts under load.** Three test runs hit nextest's global 15 s timeout while the downstream checks loaded the machine.
9. **Files past 800 lines grew.** The agent process's test world `tests/agent-process/src/world.rs` went from 1,433 to 1,586 lines, and `crates/smith-agent-service/src/service.rs` from 2,920 to 2,968. Needs the consolidation pass (code-organization.md, 7).
10. **Not code.** Local `live-text/*` branches remain in smith.

**skein**

1. **The shared fake LLM fixtures grew.**
   - `tests/llm/tests/fake.rs` went from 1,210 to 1,365 lines; `40d85476` alone added 121, against an aim of 100.
   - The `Line::Pieces` literal is copied at three sites.
   - smith's `tests/hosts/tests/simulation.rs` repeats the constants.
2. **`Peer::release` allocates a `Box`.** smith's stories call `check_memory` only because they keep the release's name in a fixed buffer.
3. **Landed tests that break section 10.**
   - `testing/skein-fake-llm-domain/src/tests.rs:121`: one test asserts four behaviours.
   - The `use` lines at 58 to 60 are out of place.
   - `tests/llm/tests/fake.rs` lines 1293, 1308 and 1348.
5. **Not code.** Local `live-text/*` branches remain in skein.
