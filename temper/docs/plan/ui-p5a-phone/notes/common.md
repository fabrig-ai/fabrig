# ui-p5a-phone: rules every task follows

Milestone P5a of `temper/docs/design/draft/whole-story.execution.md`: the
phone, Ask temper on it, and Tell temper. The design is
`temper/docs/design/client/README.md`, 3.5, *One layout for every chat*,
*Ask temper's page* (its composer), *Needs you* (*Its head*, *On a phone*)
and *Tell temper*, and 5.3, *On a phone*; `temper/docs/design/client/browser.md`,
2.5 (the phone's paragraph and *What a phone presses*). The look is the whole
story's mockups, `temper/docs/design/client/web/whole-story/`
(`phone-home.html`, `phone-need.html`, `phone-tell.html`,
`whole-story.css`, `.ptop`, `.pchip`, `.pbody`, `.ptabs`), which win over the
greenfield mock where they differ; their words, parts, order and look bind,
their figures don't.

## Runs

- **Focused and fuzzy alone (the user, 11-06).** Never `tools/tiers/run`,
  a `<tier>-<tree>` profile, `scripts/browser.sh`, a browser, process or
  live story, a walk, a render check, screenshots or a throwaway driver.
  A plan is complete when it lands; walks follow landing.
- **Journeys are generation, not a walk.** A view change reprints the
  committed journeys: `heavy cargo run -p temper-client-journeys` (no
  argument) rewrites `temper/docs/design/client/journeys/**`, which the
  gate's focused `the_committed_journeys_are_their_print` reads. Only a
  recording the reprint calls stale is recorded again, `again NAME`; on "a
  key mark caught a moving page" retry it up to three times, each on a fresh
  fake box; if all fail, widen the mark's wait in the recording driver and
  say so.
- **Tests stay cheap:** a new case of one behaviour is a row of an existing
  table test, not a new test (`skein/docs/foundation/testing-strategy.md`,
  8); temper's focused and fuzzy suites are bounded at 30 and 120 CPU
  seconds in the gate. The gallery's tests are already past their budget
  (`temper/docs/design/client/browser.status.md`): this plan adds no
  gallery state, only rows that read the prints it already makes; report
  the gallery's `--lib` CPU seconds before and after your change.
- **Before calling a focused or fuzzy failure outside your change,** run it
  once on the pristine base. If the base passes and your branch fails, it
  is yours.

## Code

- Step crates (`temper-client-domain`, `temper-client-view`) keep the
  strict subset (`AGENTS.md`, 2): every `match` lists every variant, no
  indexing, no unchecked arithmetic, no `as`, no `unwrap`.
- **`temper.css`:** new rules go under the plan markers already in the
  file, `/* ==== ui: rail ==== */`, `/* ==== ui: home ==== */`, `/* ==== ui:
  needs ==== */`, `/* ==== ui: work ==== */` and `/* ==== ui: phone ==== */`,
  never at the end of the file; a replaced rule is removed where it stands.
  Class names come from `temper/crates/temper-client-view/src/look.rs`'s
  `NAMES` (a style is `temper-<name>`). No rule nests `:has()` in `:has()`
  (`browser.md`, 2.5, *A selector a browser accepts*).
- **A 44 px target** (`browser.md`, 2.5, *What a phone presses*): at
  `@media (max-width: 760px)`, a control stands at least 44 px tall to a
  press. Where the mockup draws it smaller (the Away chip's 36 px pill, the
  night switch's 30 px circle, a way's 36 px tab), its drawing stays and
  its target grows around it, by padding with a negative margin or a
  transparent `::after` positioned over it, so that the page's layout and
  look don't move. A link inside a line of words is no target of its own.
- **Style tokens and words** (`look.rs`'s styles, `words.rs`'s `Word` and
  its numbers) are appended at the ends of their lists, as every plan
  does; other plans append too, so a catch-up renumbers what it must and
  keeps the names.
- **Goldens, prints, journeys, the gallery's `names.txt`:** regenerate them
  by their own tools, never by hand-merging; keep `names.txt` sorted as it
  is.
- **A test's oracle or helper that your change's correct behaviour trips
  is yours to repair,** keeping what it checks for everything else, named
  in the report.
- **A design status entry your task builds** (`Owns and may change` names
  it) is deleted when its acceptance holds, leaving the paragraphs around
  it as they are; you write no other design text.

## Not to touch

- `temper/crates/temper-engine-domain*`, `temper/crates/temper-service`,
  `forge_route.rs` and the wire: this plan changes nothing on the engine's
  side or between the client and the engine.
- **Tell temper is not built** (`temper/docs/design/client/README.md`, 3.5,
  *Tell temper*, *Shown*): no `Tell` address, no region, no link, no word
  and no placeholder for it. Why a need reached the person and "Stop
  asking" are N1's (`ui-reaches`); leave its parts of a card as they stand.
- The rail's wide entries, its order and its foot (5.3) are P1's as they
  stand; the switcher's rows are P1's and P4a's.
- `docs/design/`, drafts and `docs/help/` are not the implementer's (the
  implementer's text), but for the status entry your task names and a help
  page your change pushes past its bound.
