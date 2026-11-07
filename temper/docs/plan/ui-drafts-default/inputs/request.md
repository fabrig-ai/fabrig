# ui-drafts-default: drafts are part of what a fresh project offers

The user's word, 2026-11-07, relayed from their laptop: "the configuring chat should offer Draft unconditionally. And drafts should be on by default for new projects." They looked for the pane's buttons on a fresh box and found none.

## What this plan makes true

1. **The configuring chat offers "+ Draft" whatever the project's `drafts` says,** with its other conditions as they are: no document yet, not ended. At `b75c4f3852` two gates need it: `temper/crates/temper-client-view/src/pages/chat.rs`, near line 67, and `temper/crates/temper-client-domain/src/pages/chat/sheet.rs`, near line 105, which drops `AddDraft`. Before a first message there's nothing to write up, so it follows the first message, as any chat's does. Only the configuring chat changes here: another chat keeps following `drafts`.
2. **A new project has `drafts` on by default:** `forge/ways.rs`, near 18; `temper-engine-domain-forge/src/settings.rs`, near 108; `stock/kinds.rs`, near 82; `temper/docs/design/protocol/configuration.md`, the kind 7 row, near 1894; `temper/docs/design/client/README.md`, near 2479 and 5314. The setting stays, so a project can turn it off.
3. **The pair holds together.** "Start goal…" follows `drafts_offered` and a home repository, not `goals`, so a fresh project now offers it with Goals hidden, and `frame.rs`, near 747, hides Goals from the bar. Read whether a goal started there can be found: its link on the chat, Home's goals section (`pages/home.rs`, near 541), and its address. **If a person can reach it from its chat and from Home, that's enough, and say so in `plan.md`.** If not, the small fix: a project holding a goal shows Goals in its bar, whatever the setting says, as the setting hides an offer and not a person's own work. Report which it was: the user's asked to hear.

## Limits

Two tasks at most. Focused and fuzzy alone: no browser, walk or render check, by the user's rule that a plan is complete when it lands. Regenerate prints and goldens that the default changes; a journey's with `temper-client-journeys again`. Stories that read the old default change with it, compiled and not run in a slow tier. New cases are rows of an existing table test. Beside it, `ui-proposal-2` and `ui-small-1` are on the same instructions and chat files: whichever lands second catches up.

## Sources

- `temper/crates/temper-client-domain/src/pages/chat/sheet.rs`
- `temper/crates/temper-client-domain/src/pages.rs`

## For every task of this plan

- **A plan is complete when it lands (the user, 11-06).** Walks, render checks, screenshots and throwaway browser drivers follow landing, outside the plan, and file findings back to fab: don't ask for them, or run them.
- **Only focused and fuzzy run in a plan (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure: "we run optimistically and repair if after-landing flags". A new story for those tiers is written and compiled (`cargo check --tests`), not run. This overrides anything below or in a spec. A look's render check by a throwaway driver isn't a tier.
- **Tests stay cheap (temper/tiers, 11-06):** a new case of one behaviour is a row of an existing table test, not a new test (skein's `docs/foundation/testing-strategy.md`, 8), and a story goes beside its siblings' fixture. temper's focused and fuzzy suites go to 30 and 120 CPU seconds in the gate, and a plan that adds cost gets rejected there.
- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
- **A test's oracle or helper that your change's correct behaviour trips is yours to repair in the same task,** keeping what it checks for everything else, and named in the report; don't park on it.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
