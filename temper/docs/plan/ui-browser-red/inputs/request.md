# ui-browser-red: the browser cookie and Sign out test's intermittent failures

`browser_sign_in::signing_in_reloading_reconnecting_and_signing_out_keep_the_cookie_contract` (`temper/tests/client-browser/tests/browser_all/`) fails intermittently, in two ways, and nobody has found why. It's red-39 in after-landing. The user also reported trouble signing out on dev.fabrig.ai today (which build runs there isn't known), so a real product fault matters.

## What's known

- **A 503:** `unexpected Log: Failed to load resource: the server responded with a status of 503 (Service Unavailable)` at person 0, step 0, on "Hello · temper". After-landing at 47457baa78, 16:14Z, and its rerun; log on hetzner8, `~/src/fabrig/fab/after-landing/47457baa780f-20261105T155451083307Z/browser-temper.log`. Also in `fab/runs/ui-welcome/red-36-2/report.md`, in a full suite run, at the Chats page.
- **A protocol refusal:** the same test, alone, person 0, step 31, pressing Sign out on Accounts: `browser refused act: Protocol`, with the Sign out button visible (same report; `ui-welcome/red-36-1` too).
- **It also passes:** `fab/runs/ui-welcome/red-36-3/report.md` (or the latest red-36 run) passed it after a navigation fix, so it's intermittent. That repair found no cause.
- A plain search of `temper/crates/temper/src`, `temper-client-bundle/src` and skein's web crates for 503 found nothing, so which server answers it, and for which resource, is the first thing to find. The test's Log check doesn't name the URL: make it do so.

## The task

1. Make the failure say what failed: the 503's URL, and what `Protocol` means at that act (`temper-client-browser-world`, the CDP driver). Run the test repeatedly, alone and under the suite's load, until each failure is caught.
2. Find the cause at its lowest tier, and fix it. If it's the product (a resource served too early, a font or a bundle file, a sign-out request refused, a click racing a re-render), fix the product, with a test at the lowest tier that reproduces it. If it's the harness, fix the harness and say why the product is sound.
3. Say plainly in the report whether a person can hit this, and whether signing out works on a real browser.

Keep every cookie, reload, reconnect and sign-out assertion. Don't change ports, budgets or timeouts to hide it; never mark the test ignored or retried.

## Size

One or two tasks. One cut is at most 30 minutes. Budgets and timeouts are the user's (skein's `docs/foundation/testing-strategy.md`, 8.1).

## Sources

- `temper/tests/client-browser/src/lib.rs`

## For every task of this plan

- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Any task may use a throwaway render script, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
