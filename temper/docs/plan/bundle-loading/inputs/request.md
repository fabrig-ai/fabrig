# bundle-loading: the client bundle goes compressed, and a placeholder shows while it loads

The user, 2026-11-06, on `temper/docs/design/draft/bundle-loading.md`: "ok for CSS+JS+compression", and of the look `bundle-loading`: "all approved". Read the draft whole and build it, in its order: section 4's compression first, since it's worth having by itself and shortens the wait; then the page and script, with the look. If something doesn't hold when you get there, say so and take the nearest thing.

## 1. Compression

**Report a release bundle's sizes, raw and gzipped:** so far only a debug build's are known (at 2 Mb/s, on a debug bundle, 46.5 s to something readable). Say how you measured, in `plan.md` and in `bundle-loading.status.md`.

The clients' policy (CSP) and every browser the tier runs still hold: a compressed answer hasn't been tried under temper's exact policy, as the look's script ran in Chromium with none. Failing tests first, in the lowest tier: a client asking for it gets the compressed answer with the right headers, and a client not asking gets the plain one, by the same bytes when decoded; a build's bundle isn't compressed twice; a start's time and memory are said. A generic mechanism goes in skein first, and skein names no temper.

## 2. The page, the script and the look

Land the look request `bundle-loading` (`0c1b78f3e1`, on `origin/look-requests`; the branch moves, so if that hash is gone, find it by its `Look-request: bundle-loading` trailer), by `temper/docs/design/draft/look-requests.md`, sections 3 to 5, with its account. Its pages are `journeys/bundle-loading/01.html` to `04.html`, and it uses the placeholder's own stylesheet, not the overlay. Its message holds every state. Every commit of this part carries `Look-request: bundle-loading` directly above `Co-Authored-By:`. Other requests on the branch are other plans': `model-picker` waits for `model-catalogue`.

## Beside it

`browser-trim` left the browser tier at 58 to 79 CPU seconds, against 85, with `coord/ui/report/browser-figures.md` on hel8 as its baseline: report the tier's figures before and after, since every browser test now loads this page. The playground has its own bundle and is `story_a`'s: section 7's open question on it, decide and say. `sign-in-new-tab` and `model-check` are temper/forge's until landed, and don't touch this.

## How the cut shapes it

Design first: skein's `ui-web.md`, 4.2 to 4.4 and 15; `client/browser.md`, 2.6 and 4.3; `protocol/configuration.md`, 7; the draft's status to none lagging. Three to five tasks. Proof: a throttled load at 2 Mb/s on a release bundle under temper's exact policy, each state beside the request's pages at 1440 and 420, in `~/src/fabrig/bundle-loading/` on the builder, outside any worktree, the report saying which builder. A person's first paint before and after is the number that counts. `walk = false`. A format or protocol change needs no `Question:`: decide, and note it in `plan.md`.

## Size

One cut is at most 30 minutes. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

## Sources

- `temper/docs/design/draft/bundle-loading.md`
- `temper/docs/design/draft/bundle-loading.status.md`
- `temper/docs/design/draft/look-requests.md`

## For every task of this plan

- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Any task may use a throwaway render script, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
