# ui-p3b-reaches: rules every task follows

- **Read first:** fabrig's `AGENTS.md`, `temper/AGENTS.md`, your spec, and
  `temper/docs/plan/ui-p3b-reaches/notes/contracts.md`, which binds the
  shapes that cross tasks. Where a spec and the note differ, the note
  wins; say so in the report.
- **The request:** `temper/docs/plan/ui-p3b-reaches/inputs/request.md`,
  read whole (it is short). Its `For every task of this plan` list binds
  every task; the lines below repeat what bites here.
- **The design** is `temper/docs/design/client/README.md`, 5.13, *What
  reaches you*, *Its parts* and *Its rows*, and 3.5, *Away*'s *On How work
  runs*; `temper/docs/design/client/browser.md`, 2.5, *How work runs'
  look*, and 7.2's `how` to `night-how-reaches` rows. The look is the
  whole story's `temper/docs/design/client/web/whole-story/how.html`, its
  section "What reaches you", and `whole-story.css`. Read those sections,
  not the whole documents.
- **Only focused and fuzzy run.** Never `tools/tiers/run`, a
  `<tier>-<tree>` profile, a browser, process or live tier, or
  `scripts/browser.sh`. No walk, render check, screenshot or throwaway
  driver: the plan is complete when it lands. No journey records How work
  runs (`temper/docs/design/client/journeys/` shows only its rail link), so
  no task re-records one.
- **Tests stay cheap.** A new case of a behaviour is a row of an existing
  table test where one fits; a new test only where none does, beside its
  siblings' fixture. temper's focused and fuzzy suites have 30 and 120 CPU
  seconds in the gate. The gallery's tests are already over their budget
  (`temper/docs/design/client/browser.status.md`, 7.3's departure): this
  plan removes two states for the two it adds, and adds no test function
  to the gallery.
- **Fakes, never the real thing:** the client domain's fixtures
  (`temper/crates/temper-client-domain/src/tests/fixtures.rs` and
  `tests/pages/how.rs`'s `fixture`), the view's
  (`temper/crates/temper-client-view/src/tests/how.rs`' `fixture` and
  `tests/settings.rs`' `away_settings`), the client world
  (`temper/tests/client`, package `temper-client-world`) and the gallery's
  hand-made deliveries (`temper/tools/gallery/src/worlds.rs`).
- **Design files are the cut's,** with one departure each spec states: the
  task deletes the **Not yet** entries it closes from the status files it
  names, and touches no other design text.
- **Generated files your change makes stale** (the terminal's goldens under
  `temper/crates/temper-client-shell-terminal/src/tests/golden/`, the
  gallery's `names.txt`) are yours to regenerate with their own tool
  (`SKEIN_CODEGEN_WRITE=1` on the test that checks them), in the same task,
  and to name in the report. On a catch-up, regenerate rather than
  hand-merge, and keep `names.txt` sorted as it is.
- **A test's oracle or helper that the correct new behaviour trips** is
  yours to repair in the same task, keeping what it checks for everything
  else, named in the report.
- **Before calling a focused or fuzzy failure "outside the change",** run
  it once on the pristine base. If the base passes and your branch fails,
  it is yours.
- **Decide, don't ask.** A small edit just outside your files (a re-export,
  a `mod` line, a doc comment, a golden) is made and named in the report. A
  `Question:` is only for what truly blocks your acceptance.
- **Parallel plans:** `ui-n3a-only` (Only you, on Needs you) is cut beside
  this one and writes the same shared files: `words.rs`, `look.rs`,
  `temper.css`, the gallery's `samples.rs`, `tests.rs` and `names.txt`.
  Keep this plan's lines in the How work runs neighbourhoods each spec
  names, never at a file's or a list's end: words after `HowRoleInstructions`,
  look slots in the reserved run after `temper-how-role-more`, CSS under
  `/* ==== ui: how ==== */`, samples and expectations beside `how` to
  `night-how`.
- **Not to touch:** `temper/docs/design/**` but the status entries your
  spec names, and every other task's owned files below.

| Task | Owns |
|---|---|
| `reaches-model` | `temper-client-domain`'s `pages/how.rs` (contracts, section 1) and its tests' `pages/how.rs` |
| `reaches-fuzz` | `temper/tests/client/tests/suite/fuzzy/how.rs` |
| `reaches-page` | `temper-client-view`'s How work runs (`pages/how.rs`, `pages/settings.rs`), its words, look slots and CSS (contracts, section 2), the view's tests of the page, the terminal's `settings.txt` golden, the classes' checks' removal across the client |
| `reaches-gallery` | `temper/tools/gallery`'s `how` to `night-how-reaches` states and `names.txt` |
| `reaches-story` | `temper/tests/client/tests/suite/away.rs`' story `what_reaches_a_person_follows_the_classes_away_and_stop_asking` |
