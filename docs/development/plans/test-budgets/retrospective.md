# Retrospective: test-budgets

2026-10-14.

## 2. Outcome

The plan's "Done when", against the three `origin/main`s at skein
`9539e00`, smith `527884dc` and jig `0031b05e3`:

| # | Item | Result |
|---|---|---|
| 2 | Each focused suite has its 15 s timeout | Holds: line 9 of each `.config/nextest.toml`, under `[profile.default]`. skein (`bcf5e06`) and jig (`1525519e7`) moved 30 s back to 15 s. smith's line never changed: smith ran without it here under ruling 121's gate until smith-04.1 (`29fa97c9`) returned it to `scripts/check.sh`. |
| 3 | Each budget landing passed its own gate here | Holds. skein-02.5: `scripts/check.sh passed: 2104 tests / 6.039 s`. smith-04.1: 1995 / 10.089 s. jig-06.1: 3514 / 8.113 s. |
| 4 | Each suite met the aim | Holds: `Median: 5.590 s`, `9.021 s` and `4.728 s` in the three bodies. jig-06.1's own gate ran at 8.113 s against its median of 4.728 s: one gate under load varies by about 70%, which the 3 s margin is for. |
| 6 | The read of what has arrived is built and used | Holds by the record. skein's megabyte story passed once on `bcf5e06` (`logs/skein-01/done-when-main-nextest.log`), smith's once on `6e863b4f` (`logs/smith-01/done-main-proof.log`). Both tests are on `main`; this session did not run them again. |

## 5. What did not

2. **Plan defects: a step's Files missed what its change reaches.**
   - **What:** 11 rulings, 0.28 a step, against 0.30 and 0.38 in the two
     earlier runs. Nine of them were users or copies of what the step
     changed. Three of the nine (115, 117 and 118) came back because a
     ruling had searched too narrowly (item 4):
     - the drift test over all four codec families (101);
     - a worst-case literal in `replay_total.rs` (106), and a read-shape
       row the Shape itself changed (107);
     - a heap meter that counted a fixture cached by the step (113);
     - jig's four copies of smith's goldens and the literals derived from
       their sizes (114 to 118);
     - the one-process world's cuts, placed by a count of effects that
       the plan's own aim lowered (119).
   - **The other two were Shapes not checked against the code:**
     - a lazy trace over types without `Clone` (103);
     - a whole-transcript digest past its test's ceiling (105).
   - **Cause:**
     - The audit measured what each change would save.
     - It did not list who reads, copies or counts the thing each change
       moves.
     - One search for the thing's name, its copies and its users would
       have found each of the nine.
   - **First visible:** 23:15Z (ruling 101). **Acted on:** each in its
     ruling. The supervisor wrote the general lesson into the user's
     auto-memory at 01:52Z, not into a document.
3. **The generator's new shape was proven only in skein.**
   - **What:**
     - skein-02.2 made golden tables of up to 1,024 cases.
     - smith's regenerated `golden_0` then failed clippy's
       `too_many_lines` (233 and 445 lines against 100). That became
       skein-02.s1, ruling 110, at 00:41Z.
     - skein-02.s1's `const` items failed jig's
       `clippy::large_const_arrays`. That became skein-02.s2, ruling 111,
       at 01:12Z.
     - Ruling 110's fallback, "if clippy objects … emit a `static`",
       could not fire in skein, whose tables are small.
     - Both steps landed on the longest chain. smith 01.1 sat idle from
       00:39Z to 00:50Z, and jig 05.1 from 01:10Z to 01:21Z.
   - **Cause:**
     - A change to generated code was gated where it was made.
     - Its effect shows in the families downstream, which nobody
       regenerated before the step landed.
     - `plan downstream` type-checks, but it neither regenerates nor runs
       clippy.
   - **First visible:** 23:54Z. skein-02.2's own report had already
     worked around the line limit in skein's sample family.
     **Acted on:** 00:41Z and 01:12Z.
9. **The plan's fix of the window story was a wrong diagnosis.**
   - **What:**
     - smith-02.1 sized the terminal before the binary started.
       `findings/cost-smith.md` had proposed that fix under "Likely, not
       verified".
     - The story then failed two gates in a row under load (00:22Z and
       00:23Z), with `[85, 20]` against `[80, 20]`.
     - Ruling 108 allowed a retry. Ruling 109 found the echo race, and
       smith-02.s1 fixed the test at 00:43Z.
   - **Cost:** two failed gates, two escalations, one added step, about
     20 minutes of smith 02 and 03.
   - **Cause:** a finding that said it was not verified became a step's
     Shape without being verified.
   - **First visible:** 00:22Z. **Acted on:** 00:43Z.
12. **A design gap was ruled without the designer.**
    - **What:**
      - jig-repin filed "mid-run cut still recovers Refused" as a design
        gap (02:32Z).
      - The supervisor ruled it a fixture defect (120), which let the
        stories pass. It put what recovery records after a clean exit in
        `later.md`, "unchecked against jig's design … rather than waking
        the designer, since nothing in the run waits on it".
    - **Cause:** section 10.4 gives design gaps to the designer, but
      offers no way to let the step go on while the question waits.
    - **First visible:** 02:34Z. **Acted on:** the `later.md` row.
## 6. Decisions taken during the run

Rulings 101 to 121 and four steers. The supervisor marked each in
`done.md` "not in the design yet", with the document it would belong to.

| Rulings | What | Belongs in | Proposal |
|---|---|---|---|
| 101, 110, 111 | The generator's families regenerated whole; table cases as `static` items outside the test function | skein's `codec.md`, 6 ("generated tests"), which names table tests but not where their cases live | 3; for the designer |
| 104 | The coordinator checks a landed report on the landed branch only | development.md, 7.3 against 8.6 | 8 |
| 116, steer 2 | Timing and live runs from a committed, clean tree | development.md, 8.3; smith's `workflow.md`, 5 | 5 |
| 106, 107, 113, 114, 115, 117, 118, 119 | A step's Files widened to the users, copies and derived literals of what it changes | The session files, amended in the run; development.md, 5.4 and 5.7 | 2 |
| 103 | smith's run-world trace stays eager | jig's `workflow.md`, 2 ("Traces on failure"), with its `later.md` row | Section 9 |
| 105 | A large transcript named by its length and a bounded fingerprint | Test kit only | Section 9 |
| 108, 109, steer 3 | The window story's echo race, fixed in the test | Test only; smith's `shell.md` already says the terminal echoes | 13 |
| 112 | A cue spanning two output pieces replaced | Test only | — |
| 120, 121 | The one-process fixture keeps its worker alive past a turn; one more trace | Test only. The `Refused` recovery is a `later.md` row for the designer | 12; section 9 |
| Steers 1, 4 | The Proof trailer's shape; ruling 108's retry ended | Expired | — |

**Decided by the user during the run:** nothing; nothing was asked.

**Decided by the user after the run:** decision 9, no veto (jig
`386e2373e`; `supervision.md`, 2026-10-14T07:18:48Z).
- smith 01 stays proven by its counting stories and its `smoke-tools`
  runs.
- The user asks that smith's rule say so for the next such case. The
  supervisor drafted the text, and it is proposal 14.

**Still with the user** (`decisions.md`, "For the user"): decision 11,
smith's suite here against the reliability run there. The planner and
the supervisor (01:59Z) recommend option (a): the reliability run holds
smith's focused suite to about 7 s on its machine.

## 9. Debts left in the code

For jig's and smith's `tech-debt.md`. skein has none yet, so its rows are
listed for the designer.

**jig:**
- **Four copies of smith's goldens:** the two transcript `.bin` files,
  `FULL_TURN` (`testing/jig-test-client-records/src/activity_reads.rs:4`)
  and `SMALLEST` (`testing/jig-test-worker/src/tests/fixtures.rs:62`),
  with literals derived from their sizes.
  - Every repin of smith refreshes them by hand (rulings 114 to 118).
  - Paid back by a drift test that compares each copy with smith's
    golden at the pinned revision, or by reading the golden from the
    pinned crate.
- **The one-process world's power-loss cut is placed by a count of
  effects** (window 126 to 130, cut 128).
  - Any change in how many steps a run takes moves it (rulings 119 to
    121).
  - Paid back by placing the cut on an event: the first committed `Turn`
    with a live worker.
- **What recovery records after a power loss between a worker's clean
  exit and its `Finished` commit** (`Refused`): not checked against the
  design (`later.md`, ruling 120).
- **From `later.md`:** the core world's observer rebuilds its snapshot at
  every commit, and the fleet world formats every event into its trace.
- **jig-02.1** (`2497c8299`) carries no trailers git can read. It will
  stay so, since a landed commit is never rewritten.

**smith:**
- **The run world's trace is formatted eagerly** (ruling 103, `later.md`):
  `run::Event` and `run::Request` have no `Clone`.
- **The agent world names a transcript over 1 KiB by its length and a
  hash of its first and last 512 bytes** (ruling 105). Two runs that
  differ only in a large payload's middle look the same in the trace.
- **smith's focused suite is at a 9.0 s median here, against the 12 s
  aim,** while the reliability plan adds tests gated on a machine about
  twice as fast. Decision 11 is open.

**skein:**
- **From `later.md`:**
  - `RequestTable`'s O(N) scans;
  - the event-stream and JSON readers' byte-at-a-time loops;
  - the simulator's lack of short receives on pipes;
  - the terminal's keys read one at a time;
  - the fuzzy suites, at 40 s of their 60 s here.
- **The generator's `codec.md`, 6 does not yet say** that a golden table's
  cases are `static` items outside the test function (rulings 110, 111).

**Workspace:** temper's pins are 264, 788 and 1,481 commits behind
skein's, smith's and jig's `main`. That is outside this plan (decision
12), and a plan of its own.
