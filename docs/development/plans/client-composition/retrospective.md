# Retrospective: client-composition

2026-10-14.

## 2. Outcome

The supervisor ran the plan's "Done when" at 16:53Z, in a worktree at
`origin/main` 9cba0a532 (`done.md`). This session checked items 2, 3, 4,
6 and 8 again on `origin/main` b254cd2e6.

| # | Item | Result |
|---|---|---|
| 2 | Owners are jig or an application's number | Holds: the grep prints nothing. It has held since jig-01.1 landed at 12:45Z. |
| 3 | The referee checks the outline | Holds: `tests/client/src/harness/world.rs:631`. |
| 4 | A page is its sections | Holds: `pages.rs:22` (`pub enum Section`), `:39` (`pub struct Arrangement`), `sections.rs:10` (`pub fn section(`). |
| 5 | The new stories and the text shell pass on `main` | Holds: "55 tests run: 55 passed, 381 skipped", in 0.293 s. |
| 7 | The aim, about 12 s | Holds: 8.846, 9.182 and 9.183 s, a median of 9.182 s. |

## 5. What did not

5. **Defects in increments no ready check read.**
   - **What:** a ready check reads a session's first increment, or its
     first two. That covered 12 of 36. All three plan-defect blocks
     came from the other 24:
     - **10.3 (14:48):** the plan said skein-world gives no pipes.
       jig 10's check had found the same premise, for 10.1, and missed
       `World::run_with_inputs`; its correction stopped at 10.2.
       Ruling 111.
     - **07.2 (15:30):** 204 lines against a Size of 120. Ruling 113.
     - **09.3 (15:59):** `for_world` overflowed on the inbox's limits,
       and the fixture took 263 lines against 120. Ruling 115.
   - **One correction was not run.** jig 08's check (ruling 116) told
     08.2 to configure the arrangement in `ops`'s joint world. The check
     did not apply it, and ran the three packages' default profile, not
     the fuzzy suite. 08.2's gate then failed in the fuzzy joint world
     at 16:16, ten minutes after the ruling. Rulings 117 and 118, about
     20 minutes on the chain, found a real design gap. Run by the check,
     the failure would have shown before 08.2 started, and the question
     could have gone to the designer before the session's work began.
   - **First visible:** 14:48. **Acted on:** each in its ruling.
9. **Test-time shares were smaller than one gate's noise.**
   - **What:** shares were 0.2 to 0.8 s a session. One gate's focused
     time swings by about 1.3 s:
     - 04.1 at 9.861 s, then 04.2 at 8.534 s;
     - 10.1 at 9.881 s, then 10.2 at 8.898 s.

     By the README's rule, jig 04 passed its share at 04.1, by 1.14 s
     beyond the 0.2 s. Neither the report nor any check noticed, and it
     was noise.
   - **Effect:** none on the suite: the whole run added 0.69 s. But the
     rule measured nothing it could act on.
   - **First visible:** 13:22. **Acted on:** 10.1's case, by the 14:40
     check ("likely the gate's variance").
## 6. Decisions taken during the run

Rulings 101 to 120 and two steers. The supervisor marked each in
`done.md` with its design commit, or "not in the design yet".

| Rulings | What | Reached the design | Proposal |
|---|---|---|---|
| 105, 106 | The shell's name is at most a node's text; `Terminal::new` refuses a longer one | Yes: `view.md` 15.1, 346aaa4c7 | — |
| 108's question, 109 | A card a shown conversation holds stays there and is never owed; `Conversation::shows` moves into the domain | Yes: `view.md` 4.1, 38c0a1c7 | — |
| 117, 118 | Only the engine's restart excuses telling the core of a taken delivery; the referee forgets at the cold hook | Yes: `protocol/client.md` 7, `domain/engine.md` 11, `client/domain/testing.md` 6, 74bce1ac2 | — |
| 101–104, 107, 108, 112, 116, 119 | Session files corrected against `main` by the ready checks | Not a design matter: the session files, amended in the run | 1, 3 |
| 111 | 10.3 reaches the simulator through `World::run_with_inputs` | Not a design matter: skein-world's own interface | 3 |
| 113, 115 | 07.2 at up to 215 lines; 09.3 fixes `for_world` at up to 200 | Not a design matter | Section 5, item 10 |
| 114, steer of 15:40 | jig 08 starts at 08.2 | The plan only | 2 |
| 120 | `plan downstream`'s scratch worktree removed for the release | The kit | Section 9 |
| Steer of 14:40 | One behaviour per test name | Already in author.md 8.2 and development.md 5.6; a test before the steer broke it | — |

**Decided by the user during the run:** nothing; nothing was asked.

**Still with the user:** `decisions.md`'s "For the user", four
long-term choices the planner made under the user's delegation:
- the shortcut's modifier and key;
- region names as an owner and a `u16`;
- the owner number in the older names `Part.connector` and `family`;
- the terminal's interface.

None has been vetoed.

## 9. Debts left in the code

For jig's `tech-debt.md`. skein's rows are for the designer.

**jig:**
- **`crates/jig-domain-client/src/pages/shown.rs` (07.2):** its 19-bit
  `section_context` is a field set before each open and reset after it,
  a hidden parameter, and the bit loop is written three times (the
  16:17 check).
- **`tests/client/src/stories/text_pages.rs` (09.3):** its `special`
  helper (46 lines) dispatches five pages on strings, with an
  `unreachable!` arm, where an enum would name them (the 16:17 check).
- **06.3's two tests** carry 73 lines of setup for bodies of about 9, as
  ruling 109 prescribed (the 15:39 check).
- **07.1's 25-line inline `fits` match** would read better as a function
  (the 15:39 check).
- **Files past 800 lines that grew in the run:**
  - `domain.rs` 1,278 (from 1,241);
  - `pages/routing.rs` 1,153 (from 1,111);
  - `sections/project/owner.rs` 1,202;
  - `tests/client/src/harness/world.rs` 948 (from 866);
  - `tests/client/src/stories.rs` 943 (from 880).

  `jig-view/src/tests/work_pages.rs` crossed 800 in the run (842).
- **10.3's terminal story** takes 0.21 to 0.62 s by the machine's load.
- **jig-06.1** (`cdfce095a`) carries no trailers git reads, for good.
- **From `later.md`:**
  - the field names `Part.connector` and `family` carry an owner number
    (decision 5);
  - the composer is in the confirmation's dialog, where `pages.md` 12.2
    lists it inside the activity's section;
  - `Conversations { project }` is ignored when the page opens;
  - the project's overview sets the inbox's filter but asks no read;
  - drawings are acted on in their place only.

**Workspace:**
- temper's pins are 115 commits behind jig's `main`.
- temper fails against jig's `main` with 16 to 18 errors. Nine of them
  are this plan's: the owners' new names and `Act::Drawing`'s arm.
- The user repins after this run (decision 2).
