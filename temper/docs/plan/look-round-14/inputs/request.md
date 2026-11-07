# look-round-14: what the render check found in `connect-flows` and page 07

A look repair, after landing, as the procedure now has it (`temper/docs/design/draft/look-requests.md`, 4): each repair carries its request's trailer, `Look-request: connect-flows` or `Look-request: model-picker`. The check is `~/src/fabrig/coord/codex-kit/reports/look-render-1.md` on fab's host, at `e572bf86aa`, at 1440, 1000 and 420. The request pages are at `86358b6de5` (connect-flows, 16.html and 18 to 22 below) and `3d0ed33559` (model-picker, 07.html) on `origin/look-requests`, by their trailers.

**The user called 16.html perfect, and asked that every Settings select match. These are misses, not new looks. Make the real pages look like the request's.** Most are a broad rule winning over a landed one, so fix the cascade, and don't add `!important`.

## 1. The select chevron, the user's own ask

The eight How work runs selects draw it. **Forge (connect-flows' sheet) and Provider (Models' add form) don't:** `.temper-settings .temper-settings-text` overrides the select's background on Forge, leaving `background-image: none`, position `0% 0%`, padding-right 12; and the Models add form's ID-scoped select rule wins on Provider, leaving appearance none with no arrow at all. Every Settings select draws `files/chevron.svg`, 14 px in from the border, with padding-right 36, whatever the select's own rules. One rule that can't be overridden by a later scope. A row in the existing stylesheet test if it can show this.

## 2. The sheet, 16 to 22

- **The source chooser** (`.temper-connect-sources`) is one row: flex, 2 px gap, 0 top margin, about 271 × 36. Today it's a two-by-two grid, 14 px top margin: the kit's radiogroup rule wins.
- **Copy public, 21 and 22:** a visible label, "Their addresses, one per line", 14 px; today it's only the textarea's `aria-label`. The textarea has `rows=4`, 107 px high; today it's 60.5.
- **The repository list, 16 to 19:** margins 0. The semantic `ul` keeps its 15 px.
- **Page 18, nothing found:** the row has 14 px vertical padding, 49.69 high, over the list rule's 9. The words: "Nothing here matches “QUERY”.", with the query. Hide the count on a completed search with no matches, where "0 of N repositories" shows today.
- **More's summary:** "More · Owned, it runs CI": the middle dot went.
- **Radio and check captions, 13, 14 and open More in 16 to 19:** As its admin, Owned, Fork, Context, It runs CI: 15 px, with their 24 or 23.25 px line height. Today they're 14 px, as the `strong` selector doesn't reach the semantic label.
- **The Forge caption:** weight 600, line height 21.69, bottom margin +4. Today it's 500, 22.4, −4, against the control.

## 3. The connect pages

- **Add key and Connect your Forgejo, 02, 05 and 11:** 14.5 px, line height 23.2, ink `rgb(24,24,27)`. The broad row-span rule (`.temper-connect-row span`) shrinks them to 13.5 px and mutes them: scope it to what it means.
- **"+ Connect a forge", 10 and 15, live Forges too:** today it says "+ Add a forge".
- **The model way's breadcrumb, 03, 06, 07:** it ends with ChatGPT, Claude or OpenAI API key after Connect a model, as forge ways' do on 12 to 14. Today it stops at Connect a model.

## 4. Settings, Models, page 07

- **The add form:** Provider, the name and Add on one row at every width, 36.39 high. The visible "The model's name" goes, kept as the field's accessible name. Today it narrows the input by about 95 px, and takes two rows at 420.
- **The Provider and Add sections' headings:** a 6 px bottom margin. Today 0.

## Leave alone

The report's second table, with its stated reasons, stands: the Models tab in the nav, paste fallback, step 3's words, and no counterfactual default. The Settings page's H1, shared with its frame, and the project menu's open row are frame differences, not this request's: leave them, and note them in the account. `model-catalogue-3` is adding "Read again" and the rest beside this: whichever lands second catches up.

## Limits

One task, or two at most. Focused and fuzzy alone, as below: temper/ui checks after it lands, with the same check. Prints regenerate by `temper-client-journeys again` if a journey's pages change. No new browser test.

## Sources

- `temper/docs/design/draft/look-requests.md`
- `temper/docs/design/client/forge.md`

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
