# ui-proposal-2: revising a proposal works from its own button, and what a real model's walk left

temper/qa walked Accept in the web chat on a real model, `q-accept-2`, at `876edafab4`: `gpt-6.1-sol` at medium. Accept works, and the card's place holds. It left these. Its report is on fab's host, `~/src/fabrig/coord/codex-kit/reports/q-accept-2.md`, evidence on hetzner8 at `~/src/fabrig/fab/walks/q-accept-2/evidence/`.

## 1. "Change something" dead-ends: the one that matters

Pressing a pending card's "Change something" and asking for a different figure gets: "I can't replace a proposal while it's pending. Please reject the 28,000 proposal under Needs you". That follows `configuring.txt`, line 25, and the charter has no way to withdraw. So a person who pressed the card's own button is sent to another page to reject with a typed reason, and asking for a change is the common way to revise.

**Ruling, what a person sees:** they press "Change something" and say what should change; the chat proposes again, the new card stands below, and the old one reads "Replaced by the proposal below", as `temper/docs/design/client/README.md`, near line 5335, already says. No trip to Needs you.

**The mechanism is the cut's to choose, smallest first:** a chat's new `propose_settings` replacing its own pending proposal on that task, or a tool to withdraw its own, since jig's `WithdrawProposal` exists (`jig/crates/jig-domain-tasks/src/boundary.rs`). Keep what's safe: an Accept pressed on the replaced card is refused and says so; only the proposer's own pending proposal goes, never another's; and the proposal's decider still decides. `configuring.txt` and the chat's instructions are temper/ui's, so change them with it, and take jig's busy refusal's "or withdraw it" out if it's still untrue. It changes jig's tasks: name any stored format or protocol change in `plan.md`, as temper/migration's plans share those.

## 2. A Goals-only proposal's card has no setting row

ctl's `t1/14` has `settings.items=[]`, with `goals=true` in `settings.application_items`, and the view reads `items` alone (`proposal.rs`, near 532 to 568). The design's 5.13 wants "Goals: shown". Show application items as rows, and trace whether they're lost anywhere on the way, with a focused test on the facts.

## 3. On a phone, Latest sits over "Change something"

At 420 px a real press at the button's centre hits Latest, which scrolls to the end (`04-phone-overlap.png`, `04-phone-overlap-hit.json`). Make Latest not cover a card's controls: from the rules alone, with a test of the geometry in the lowest world that can hold it, if one can. temper/ui looks after landing.

## 4. Smaller

- `temper/docs/help/proposal.md`, line 99, says rejecting "currently fails". It works: correct it.
- The activity row shows the raw tool name `propose_settings`, and the card ends with a monospace "template 1" (`02-new-card-below.png`). A person's words for both: "Proposed a change" or the row's existing family, and the template's name, in the card's type.
- Asked for a model with no connected account, the configuring chat said no connected account has it and stopped. Its instructions say to point to connecting one in the browser, on Settings' Accounts, in one line. One sample on one model.

## Limits

Two or three tasks. Focused and fuzzy alone: no browser, walk or render check, by the user's rule that a plan is complete when it lands. A browser story's words that change are updated and compiled, not run, and no new browser test: the tier is over its cap. Beside it, `look-round-13` and `model-catalogue-2` are changing Settings and the configuring chat's instructions: whichever lands second catches up.

## Sources

- `temper/crates/temper-service/src/stock/configuring.txt`
- `temper/crates/temper-client-view/src/cards/proposal.rs`
- `temper/docs/help/proposal.md`

## For every task of this plan

- **A plan is complete when it lands (the user, 11-06).** Walks, render checks, screenshots and throwaway browser drivers follow landing, outside the plan, and file findings back to fab: don't ask for them, or run them.
- **Only focused and fuzzy run in a plan (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure: "we run optimistically and repair if after-landing flags". A new story for those tiers is written and compiled (`cargo check --tests`), not run. This overrides anything below or in a spec. A look's render check by a throwaway driver isn't a tier.
- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
- **A test's oracle or helper that your change's correct behaviour trips is yours to repair in the same task,** keeping what it checks for everything else, and named in the report; don't park on it.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
