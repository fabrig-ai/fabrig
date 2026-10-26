# Retrospective: skein-ui

2026-10-15.

## 2. Outcome

The supervisor ran the plan's "Done when" at 18:07 to 18:21. It used
worktrees of skein at `f12f29d` and jig at `06ac4c469` (`done.md`).
This session ran items 2, 3, 4, 5 and 9 again on `origin/main`: skein
`c20bc49` and jig `17694847c`, which add only Markdown since.

| # | Item | Result |
|---|---|---|
| 2 | skein's crates exist | Holds: the `ls-tree` prints 12. |
| 3 | jig's copies are gone | Holds. The three paths print nothing, and 199 files of jig name `skein_ui::`. |
| 4 | jig pins skein's last landing | Holds: jig's `Cargo.lock` names skein `f12f29d`, skein-13.s1's commit and the plan's last on skein. jig keeps smith at `29fa97c9` (decisions.md, 4). |
| 5 | The gate's wasm32 builds and browser suite | Holds: `scripts/check.sh` lines 11, 12, 23, 25 and 31. |
| 6 | The browser tier passes on `main` | Holds: `scripts/check.sh` exited 0, with 35 browser tests passed in 23.0 s. |
| 8 | The test-time aims | Holds. Each is the median of three gate-lane runs; see the table below. |

| Repository | Suite | Base | Aim | Median |
|---|---|---|---|---|
| skein | focused | 6.75 s | about 9.8 s | 6.357 s |
| skein | fuzzy | 45.88 s | 48.88 s | 42.643 s |
| skein | browser | 3.27 s | 63.27 s (budget 120 s) | 22.1 to 23.0 s |
| jig | focused | 10.22 s | 11.52 s | 10.563 s |
| jig | fuzzy | 15.75 s | 17.05 s | 15.649 s |

The supervisor's first browser run left out `SKEIN_UI_EXAMPLE_BUNDLE`,
and one test stopped on the missing bundle. Its second, with the
bundle the gate builds, passed all 35.

## 5. What did not

2. **Browser stories used the gate lane as their test loop.**
   - **What:**
     - From 11.4 on, 16 browser increments ran `scripts/check.sh` themselves:
       - 33 runs, 17 failed: skein-13 13, skein-14 9, skein-12 8,
         skein-11 2, skein-14.s2 1;
       - 2,340 s alone on the machine, each pausing the build lane.
     - The authors also ran the browser profile 65 times in the
       build lane, 28 of them failing.
     - The five dearest threads were browser sessions: w16 22.1M, w11
       19.4M, w21 19.1M for one increment, w24 17.3M, and w20 14.2M for
       one landing.
     - 13.1 ran its proofs 19 times, 12 failed; 12.3 ran them 11 times,
       7 failed.
   - **Cause:**
     - **The README made the gate a Proof:** "A browser story's Proof
       is `scripts/check.sh` once in the increment's
       committed worktree". An author whose story failed had no
       cheaper place named.
     - **The browser suite needs the bundle that only the gate builds**
       (`SKEIN_UI_EXAMPLE_BUNDLE`). The plan gave no command that builds
       it once and runs one story in the build lane, though that works:
       w16 did it at 14:29, and the supervisor at 18:21.
     - Protocol's proposal 1, "reproduced below its layer", was followed
       where a host test could hold the fault. The README named no lower
       layer for a story.
   - **First visible:** 13:35, 11.4's first failing gate. The 14:09
     check noted the queue. **Acted on:** not.
9. **Work left for temper reached only jig's `later.md`.**
   - **What:**
     - Three `later.md` rows hold what temper's next repin onto this jig
       must change:
       - the renamed tree types and its users of the moved crates
         (decision 3, at planning);
       - its four `Limits` literals (ruling 133);
       - its joint front's `Reply::SignOut` (ruling 136).
     - temper's `STATUS.md` has none of them. In client-composition,
       temper `2dec7fb5` wrote that run's part there.
     - Rulings 127, 133 and 136 also named temper five times in jig's
       plan: `decisions.md` lines 817, 1394 and 1408, and the two rows.
       The plan's README forbade it ("no name of the repository
       downstream of jig in any file"), but its `rg` searched only
       skein's code.
   - **Cause:**
     - development.md 5.7 sends a repin's gap to the downstream
       `STATUS.md` and to `later.md`.
     - A plan that runs no repin, and a ruling that defers work
       downstream, have no such rule. The supervisor writes only
       `decisions.md` and `later.md`.
   - **First visible:** at planning, for decision 3's row; at 15:22 for
     the names. **Acted on:** not.
12. **A supervisor's ruling put code on `main` that the design forbade.**
    - **What:**
      - Ruling 126 (15:07) made step skein-14.s1, whose Shape takes a
        find's root from `Runtime.evaluate` of `document`.
      - browser.md 4 says "The kit never sends `Runtime.evaluate` or
        `Runtime.callFunctionOn` to run script". The plan's own skein 07
        had made that a check.
      - 14.s1 landed at 15:23. The designer's 16:13 answer named "a
        related point". The supervisor asked the question at 16:15. The
        designer amended browser.md 4 and 6 at 16:33 (`b4c7740`).
    - **Cause:** the ruling's check quoted browser.md 6 and not 4: the
      supervisor's own miss, as its question says. A step the supervisor
      writes is checked against the design by nobody else before it
      lands.
    - **First visible:** 16:13. **Acted on:** 16:15. It was settled at
      16:37, with step 14.s2 at 16:50.
## 6. Decisions taken during the run

Rulings 101 to 136 and six steers. The supervisor marked each decision
in force in `done.md` with its design commit, or "not in the design yet".

| Rulings | What | Reached the design | Proposal |
|---|---|---|---|
| 101–104, 106–111, 114–119, 121, 123, 133 | Session files corrected against `main` by the ready checks, and re-checks | Not a design matter: the session files, amended in the run | 4 |
| 105 | A gate whose only failure is the flaky terminal test retries once | Not a design matter. `later.md` has the test's row | Section 9 |
| 112, 113 | The primary modifier is a platform fact passed to the page and the mapping, outside Look | Yes: `ui-web.md`, skein `1b49e88` | 1 |
| 118, 120, 122 | What skein 14's stories take from the application: `arrives`, `failed`, and `loaded` before absent | Yes: skein `7c5a482`, `1ff0531` | 1 |
| 124, 125 | 14.2's crash: two bounded diagnoses | Not a design matter | — |
| 126, 131 | The browser kit finds across a page's own navigation; a find's root and scope resolve in the current document | Yes, after the fact: browser.md 4 and 6, skein `b4c7740` (section 5, item 12) | — |
| 127 | A go accepts a closing warning as `Warned`; other dialogs are dismissed as `Trouble::Dialog` | Yes: skein `506b940` | 1 |
| 128 | `browser.rs` past its allowance, named in the README; its split is a `later.md` row | Not a design matter | Section 9 |
| 129, 130 | A native select is found with `pierce`; a date or time field is typed by `keys` | Yes: skein `cef6a1a` | — |
| 132 | `Taller` measures, types its words, and polls for growth | Not in the design: a harness step. It corrected ruling 121 | 4 |
| 134, 136 | The referee takes no frame after its watch ends; the shell drops an earlier sign-out's answer after a newer sign-in | 136 yes: `client.md` 12, jig `09864655c`. 134 is a harness defect | 5 |
| 135 | The chart test's control bytes are written as escapes | Not a design matter | — |
| Steers 11:57, 13:18, 15:30 | One behaviour per test; named node ids; ids from position | Already in development.md 5.6 and 14.3, and the plan's "Don't" list | — |
| Steers 14:17, 14:22, 17:31 | Start orders and an amended session file | The plan only | — |

**Decided by the user during the run:** nothing; nothing was asked.

**Still with the user:** `decisions.md`'s "For the user" lists the
plan's eight long-term choices, all accepted on 2026-10-15:
- the glue pinned at 0.2.126;
- one JavaScript snippet;
- `jig-shell-native` kept one crate;
- the example's page as the bundler's;
- no repin downstream of jig;
- the digest's encoding, SHA-256 in the bundler, and an FNV-1a ETag.

No ruling added an item.

## 9. Debts left in the code

For jig's `tech-debt.md`, and for skein's (its rows are for the
designer).

**jig:**
- **33 hand-written `skein_ui::Limits` literals in 30 files,** each with
  `scene_marks: 0` and `scene_bytes: 0`. Examples:
  `crates/jig-view/src/limits.rs:94`, `owner_deployment.rs:685`,
  `owner_result.rs:122`, `tests/client/src/model.rs:188`. One
  constructor would replace them.
- **`crates/jig-view/src/tests.rs:67`'s `text_bytes`** re-implements
  skein-ui's private node accounting.
- **Files past 800 lines grew:** `tests/client/src/joint/session.rs`
  1,538 → 1,555, and `harness/world.rs` 949 → 955 (05.s2).
- **From `later.md`:**
  - a sign-out's answer kept to its sign-in in `jig-shell-native` and
    the scripted front;
  - the client world's transport forgetting a newer sign-in;
  - jig's browser layer, its frame parts and cards as primitives, which
    wait for the boundary analysis.

**skein:**
- **`testing/skein-browser/src/browser.rs` at 2,041 lines,** grown 267
  in the run. Its split is a `later.md` row.
  - Close to the limit: `skein-ui-web/src/page.rs` 777, `page/web.rs`
    767, `tests/browser/tests/support/real.rs` 759.
- **Lint exceptions:**
  - new: `skein-ui-web/src/platform/web.rs:54` and `:90` (casts), and
    `browser.rs:720` (`disallowed_methods`);
  - moved from jig into tests, against the plan's "Don't" list:
    `skein-ui/src/tests/diff.rs:1-3`, `tests/markdown.rs:1-3`,
    `skein-ui-text/src/tests.rs:8,17`, and `allow-unwrap-in-tests` in
    both fake-person `clippy.toml`s.
- **"actual" in three tests:**
  - `skein-ui-text/src/tests/bound.rs:97-103`;
  - `skein-fake-person-browser/src/tests.rs:308-309`;
  - `skein-fake-person/src/tests/random.rs:236-237`.
- **Fixtures defined again:**
  - `page()` in 7 files, and `binding()` in 4;
  - `tests/ui-text/src/lib.rs:19,22` copies
    `skein-ui-text/src/tests/fixtures.rs:21,24`.
- **The root id is a bare `NodeId(0)`** in 13 places, among them
  `skein-ui/src/tree.rs:308` and `skein-ui-web/src/page.rs:224`.
- **Code cites a plan's rulings:** `skein-ui-text/src/render.rs:7`
  ("ruling 106") and `harness/script/growth.rs:1` ("Ruling 132"). It
  should cite the design.
- **From `later.md`:**
  - the flaky `terminal_is_controlling_and_the_persons_interrupt_
    reaches_only_the_child`;
  - the face's typing into a field holding words;
  - a test answering a native dialog;
  - a press on a node of an earlier document refused as `Protocol`, not
    `Gone`;
  - a find within a removed scope.

**Workspace:**
- temper's pins are 188 commits behind jig's `main`. Its `STATUS.md`
  lacks this plan's three rows for its next repin, now only in this
  plan's `later.md` (proposal 8).
- jig pins smith 127 commits behind smith's `main`, by decision 4.
- **The run is not released.** The three `wt/*-downstream` worktrees
  hold only `plan downstream`'s regenerated `Cargo.lock`.
