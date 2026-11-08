# ui-p2-home: rules every task follows

Milestone P2 of `temper/docs/design/draft/whole-story.execution.md`: Home
and Needs you across projects, and the brief. The design is
`temper/docs/design/client/README.md`, 3.5, *Home* (its *The cards* and
*The Ask temper line*), *Home on All projects*, *Needs you*, *What the
client keeps* and *A page's parts*; `temper/docs/design/client/browser.md`,
2.3 and 7.2. The look is the whole story's mockups,
`temper/docs/design/client/web/whole-story/` (`home.html`, `needs.html`,
`phone-home.html`, `phone-need.html`, `whole-story.css`), which win over
the greenfield mock where they differ; their words, parts, order and look
bind, their figures don't.

## Runs

- **Focused and fuzzy alone (the user, 11-06).** Never `tools/tiers/run`,
  a `<tier>-<tree>` profile, `scripts/browser.sh`, a browser, process or
  live story, a walk, a render check, screenshots or a throwaway driver.
  A plan is complete when it lands; walks follow landing.
- **`temper-client-journeys again` is generation, not a walk.** A task
  whose change makes a recorded journey stale (the gate's focused replay,
  `temper/tools/journeys`, says which) records it again with
  `cargo run -p temper-client-journeys -- again NAME` through heavy. On
  "a key mark caught a moving page" retry it up to three times, each on a
  fresh fake box; if all fail, widen the mark's wait in the recording
  driver and say so.
- **Tests stay cheap:** a new case of one behaviour is a row of an
  existing table test, not a new test (`skein/docs/foundation/testing-strategy.md`, 8);
  a story goes beside its siblings' fixture. temper's focused and fuzzy
  suites are bounded at 30 and 120 CPU seconds in the gate. The gallery's
  tests are already past their budget (`temper/docs/design/client/browser.status.md`):
  add states as rows, and report the gallery's `--lib` CPU seconds before
  and after your change.
- **Before calling a focused or fuzzy failure outside your change,** run it
  once on the pristine base. If the base passes and your branch fails, it
  is yours.

## Code

- Step crates (`temper-client-domain`, `temper-client-view`) keep the
  strict subset (`AGENTS.md`, 2): every `match` lists every variant, no
  indexing, no unchecked arithmetic, no `as`, no `unwrap`.
- **`temper.css`:** new rules go under the plan markers already in the
  file, `/* ==== ui: home ==== */`, `/* ==== ui: needs ==== */` and
  `/* ==== ui: phone ==== */`, never at the end of the file; a replaced rule
  is removed where it stands. Class names come from
  `temper/crates/temper-client-view/src/look.rs`'s `NAMES` (a style is
  `temper-<name>`).
- **Style tokens and words** (`look.rs`'s styles, `words.rs`'s `Word`)
  are appended at the ends of their lists, as every plan does; other
  plans append too, so a catch-up renumbers what it must and keeps the
  names.
- **Goldens, prints, the gallery's `names.txt`:** regenerate them by their
  own tools, never by hand-merging; keep `names.txt` sorted as it is.
- **A test's oracle or helper that your change's correct behaviour trips
  is yours to repair,** keeping what it checks for everything else, named
  in the report.
- **Generated files your change makes stale** (`docs/help/reference/*`,
  prints, goldens, counted baselines) are yours to regenerate in the same
  task.
- **Reach for the mock, not the old page:** where an old test walks a part
  the whole story's mockups remove ("Later" on the browser's Needs you,
  Home's "Work carries on…" line, its first-card-and-lines form), change
  the test with it.

## Not to touch

- `temper/crates/temper-engine-domain*`, `temper/crates/temper-service`
  and `forge_route.rs`: this plan changes nothing on the engine's side; a
  read the page lacks stays a **Not yet** in the design's status.
- The rail, the switcher and the tab bar (`temper/docs/design/client/README.md`,
  5.3) are milestone P1's: draw within them as they are. The only rail
  edit allowed is the compiler's arm for `Address::AllHome`, with "Home"
  current on it.
- Why a need reached the person, "Stop asking", Only you, Tell temper and
  the terminal's "Talk it through" line belong to other milestones (N1, N3,
  P5, T3): leave no placeholder for them.
- `docs/design/`, drafts and `docs/help/` are not the implementer's (the
  implementer's text), but for a help page your change pushes past its
  bound.
