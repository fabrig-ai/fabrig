# ui-copy-once: the welcome's Copy reads once

The user, from the laptop (2026-11-06), on main at `81f0250dd3`: the welcome's Claude way shows its button as "CopyCopy". Seen in a browser on a fake box: the rendered button is `Copy<span>Copy</span>`. "Copied" and the not-copied word double the same way.

**Cause:** `temper-client-view/src/pages/welcome_copy.rs`, `block()`: the button has both `b.name(&w.say(word, &[]))` and `w.text(b, word, &[])`. skein's web mapping (`skein-ui-web/src/mapping.rs`, the arm for `Role::Button | Role::Tab | Role::Option`) already writes a button's name as its text, so the text child draws it a second time.

**Fix:** remove the text child, keeping the name, the styles and the binding. Don't change skein's mapping.

**Other buttons:** on main's pages I found one other `Role::Button` with a name and a text child, Home's Skip (`pages/home.rs`), and its text is terminal-only, so it doesn't double in a browser: leave it. My search read `b.open` by eye, and 14 sites open a role held in a variable. If one of those opens a `Button`, `Tab` or `Option` in the browser shell with a text child, it's the same bug: fix it here too and name it in the report.

**Test:** a test in `temper-client-view`'s tests, in the browser shell, that for each of the three labels (`Copy`, `Copied`, not copied) the button's text reads once: its name is the word and it has no text child. Show it red without the fix. If the view's gallery tests walk every state simply, also assert across them that no `Button`, `Tab` or `Option` has both a name and a text child in the browser shell; if that means building anything new, skip it and say so.

**Proof:** one throwaway render of the welcome's Claude way, showing the button reading "Copy" once.

One task. No look decision is needed. No `Look-request` trailer.

## Sources

- `temper/crates/temper-client-view/src/pages/welcome_copy.rs`
- `skein/crates/skein-ui-web/src/mapping.rs`

## For every task of this plan

- **Only focused and fuzzy run in a plan (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, one of their stories, or a tier's CPU figure: "we run optimistically and repair if after-landing flags". A new story for those tiers is written and compiled (`cargo check --tests`), not run. This overrides anything below or in a spec. A look's render check by a throwaway driver isn't a tier.
- **Decide, don't ask.** The user is asleep and takes fabrig/ui's judgement. A stored format, a protocol or a public interface may change freely: no compatibility, no `Question:`; note it in `plan.md`. A small edit a task needs just outside its file list (a re-export, a `mod` line, a design line, a golden) is made and named in the report, not parked. A `Question:` is only for what truly blocks the task.
- **`temper.css`:** put new rules under this plan's marker comment, `/* ==== ui: AREA ==== */` (chats, needs, goals, changes, settings, welcome, phone), never at the end of the file, so that parallel plans merge cleanly. Remove replaced rules where they stand.
- **Goldens, prints and the gallery's `names.txt`/`tests.rs`:** they conflict across plans; on a catch-up, regenerate rather than hand-merge, and keep `names.txt` sorted as it is.
- **Render proofs:** Chromium 147's new headless won't go below 500 px by `--window-size`; a 420 px proof uses CDP's `Emulation.setDeviceMetricsOverride`. Any task may use a throwaway render script, untracked and gone before the push. Handle `Page.javascriptDialogOpening`, or a pending request's `onbeforeunload` hangs a CDP driver.
- **Reach for the mock, not the old page:** `temper/docs/design/client/greenfield/`; where old tests walk a part the mock removes, change them with it.
- **Generated files** (`docs/help/reference/*`, prints, goldens, counted baselines) that your change makes stale are yours to regenerate in the same task; don't park on them.
- **Recording flakes:** a re-record that stops on "a key mark caught a moving page" is retried up to three times, each on a fresh fake box; if all fail, widen the mark's wait in the recording driver and say so. A wire change makes journey recordings stale: re-record them in that task.
- **Help pages:** `docs/help/` pages a change grows past their bound are that task's to shorten, losing no facts, and never raise the bound.
- **A test's oracle or helper that your change's correct behaviour trips is yours to repair in the same task,** keeping what it checks for everything else, and named in the report; don't park on it.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
