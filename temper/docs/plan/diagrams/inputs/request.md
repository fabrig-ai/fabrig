# diagrams: flows in Markdown are drawn

The user's word, 2026-11-07: build `skein/docs/design/draft/diagrams.md`. Its section 10 keeps "when" as the user's, and this is that word. They saw its cases on 11-04 in `diagrams/preview.html`.

**Acceptance:** the cases in `skein/docs/design/draft/diagrams/`, as its section 8 has them, byte for byte, so that a change of look means changing a case with the user. The tests read them. A change of the look isn't this plan's to make.

## temper's side

- The open item *A document's drawing* in `temper/docs/design/client/README.md`.
- The stock's "Mermaid is not drawn, so do not draw diagrams" in `chat.txt` and `configuring.txt` becomes what the draft's section 7 says agents write.

**Not part of it:** temper's reference stays without figures, the user's word of 11-06 (`own-way.status.md`, 7), so the item *The reference's picture as a flow* doesn't come back with this.

## Settled here, by temper/ui

- **skein-ui as an owner** of a drawing kind: yes. The flow is skein-ui's own, routed as any, and `ui.md`, 3.3 says so.
- **A scene that says its height:** yes, as section 4 asks: a scene may say its height, with an SVG in the figure's flow. An owner's stylesheet can't know a flow's height. Other scenes are unchanged.

**Still the user's, not settled:** whether every look-bearing draft keeps cases like these.

## Limits

Focused and fuzzy alone: the cases' scenes are checked as bytes there. No browser, walk or render check, by the user's rule that a plan is complete when it lands; temper/ui looks after landing. The tree's focused and fuzzy budgets apply: cases are rows of a table test, not a test each, and the drawing code's tests are within skein's. No new browser test. Keep a flow's cost at zero for a page with none.

## Sources

- `skein/docs/design/draft/diagrams.md`
- `skein/docs/design/draft/diagrams/README.md`

## For every task of this plan

- **A plan is complete when it lands (the user, 11-06).** Walks, render checks, screenshots and throwaway browser drivers follow landing, outside the plan, and file findings back to fab: don't ask for them, or run them.
- **Only focused and fuzzy run in a plan (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure: "we run optimistically and repair if after-landing flags". A new story for those tiers is written and compiled (`cargo check --tests`), not run. This overrides anything below or in a spec. A look's render check by a throwaway driver isn't a tier.
- **Tests stay cheap (temper/tiers, 11-06):** a new case of one behaviour is a row of an existing table test, not a new test (skein's `docs/foundation/testing-strategy.md`, 8), and a story goes beside its siblings' fixture. temper's focused and fuzzy suites go to 30 and 120 CPU seconds in the gate, and a plan that adds cost gets rejected there.
- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **`temper-client-journeys again` is generation, not a walk:** it regenerates what the gate's focused replay checks read, so a task whose change makes a recording stale runs it, though it drives a browser.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
- **A test's oracle or helper that your change's correct behaviour trips is yours to repair in the same task,** keeping what it checks for everything else, and named in the report; don't park on it.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
