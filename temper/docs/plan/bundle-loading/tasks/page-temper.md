+++
id = "page-temper"
part = "page-temper"
kind = "feature"
needs = ["page-skein", "gzip-serve", "before"]
files = [
  "temper/crates/temper-client-shell-web/loading.html",
  "temper/crates/temper-client-shell-web/loading.css",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/crates/temper-client-shell-web/tests/suite/placeholder.rs",
  "temper/crates/temper-client-shell-web/tests/suite/main.rs",
  "temper/crates/temper-client-bundle/build/steps.rs",
  "temper/crates/temper-client-bundle/tests/steps.rs",
  "temper/scripts/bundle.sh",
  "tools/tiers/run",
  "tools/tiers/test_run.py",
  "temper/docs/plan/bundle-loading/plan.md",
  "temper/docs/plan/bundle-loading/figures.md",
  "temper/docs/plan/bundle-loading/driver/**",
  "temper/docs/design/draft/bundle-loading.status.md",
  "temper/docs/design/client/browser.status.md",
  "temper/docs/design/client/journeys.md",
  "temper/docs/design/client/mockups/STATUS.md",
]
effort = "high"
minutes = 120
+++

## Goal

Land the look request `bundle-loading` by `temper/docs/design/draft/look-requests.md`, sections 3 to 5, every step, with its account; then prove it. This is the plan's last task: a person's first paint before and after, on a release bundle at 2 Mb/s under temper's exact policy, is the number that counts.

Every commit of this task carries the trailer `Look-request: bundle-loading`, directly above `Co-Authored-By:` where the commit has one, the last commit's account included.

Departures from the implementer's text: you write the design files in `files`, as step 8 below says; and the driver stays committed in the plan's directory.

Acceptance:

- `temper serve` and `temper box`, from a bundle built by each way temper builds one, serve the page of `notes/contracts.md`, P2: title `temper`, the placeholder's markup and stylesheet, the loader.
- Every state the request's message names is rendered in a real page, beside the request's pages, at 1440 and 420 (and 1000, by section 4), each difference with its reason. The screenshots are in `~/src/fabrig/bundle-loading/after/`, outside any worktree.
- `figures.md`'s `## After` holds the same figures as `## Before`, on the same driver, and the before-and-after of a person's first paint leads it.
- `temper/docs/design/draft/bundle-loading.status.md` and `plan.md` say the release sizes and how they were measured.
- The browser tier's figures, before (from `figures.md` and `browser-trim`'s baseline) and after, are reported, against 85 CPU s; it passes, every test loading the compressed module under temper's policy.
- The account is in the last commit, with no row open.

## Inputs

- `temper/docs/plan/bundle-loading/notes/common.md`, `notes/contracts.md`, P and C4; `temper/docs/plan/bundle-loading/figures.md`, its `## Method` and `## Before`, from `before`, and its `driver/load.py`.
- `temper/docs/design/draft/look-requests.md`, sections 3, 4 and 5, and 6: read them whole; they're your steps.
- `temper/docs/design/draft/bundle-loading.md`, whole.
- The request: `git show` of its commit (`notes/contracts.md`, P's opening), its whole message and its pages.
- `temper/docs/design/client/browser.md`, 2.5 (the first paint), 2.6, 4.3. `temper/docs/design/client/journeys.md`, 7 and 9.

## Owns and may change

- Owns: `files` above.
- Status, as `notes/common.md` says: `temper/docs/design/client/browser.status.md`'s two lines of this plan. `temper/docs/design/draft/bundle-loading.status.md` gains one **Measured** paragraph, the request asking for it there, and keeps the rest.
- `temper.css`: remove the selector `body > noscript` from the rules it heads with others, where they stand (a page with a placeholder has its `noscript` in `#boot`, and with scripts off `temper.css` never loads). Add nothing. If the gallery or a test reads that selector, change that reader. If only the bundler's page without a placeholder would have used it, it goes all the same: temper has none.
- `loading.css` may change for what the render check finds; its rules aren't `temper.css`'s, which is the request's own departure from section 3, step 4.
- The harness: if a browser story trips on `#boot` being in the page while it loads, the wait goes in `skein/testing/skein-fake-person-browser/src/`, smallest thing, and you say so. Stay off `temper/tests/client-browser/src/lib.rs`, which two other plans are changing, but for a one-line assertion you can't put elsewhere.
- Not landed: the request's own pages. `temper/docs/design/client/journeys/bundle-loading/` stays on its branch (`temper/docs/design/client/journeys.md`, 7); `look-refresh.sh` drops the request once main has its trailer.
- Options: if a state can't be reached in a real page (section 4 allows it) draw it on a throwaway copy, and the account says so. If the release load finishes within 8 s at 2 Mb/s, as it may with the forms, reach the 8 s line at 0.5 Mb/s and say so. If `Network.emulateNetworkConditions` won't show the stall or the drop, use a throwaway TCP proxy in front of `temper serve`, stopping or closing mid-answer. If over budget, ask `notes/common.md`'s questions; and the first thing to look at is CPU per page load: Chromium now inflates the module, and the tier's dev module is 11 MB.
- Not yours: `temper-client-view`, `temper-client-domain`, the wire, journeys' recordings (a change of the look alone never makes one stale: `journeys.md`, 8), and the gallery's `names.txt`.

## Steps

Tests first (`notes/common.md`):

1. `temper-client-shell-web/tests/suite/placeholder.rs` (a `mod` line in `main.rs`): `loading.html` holds `{noscript}`, `id="boot"`, `#boot-line`, `#boot-note`, a `hidden` button, `aria-label="Loading temper"`, and every attribute and word of P3's table; no `style=`, `<style`, `<script`, `<link` or `href=`. `loading.css` holds no `url(`, `@import` or `@font-face`, since nothing may be fetched. Together they're at most 6 KiB, which is the first paint's cost; the page is sent before anything is compressed.
2. `temper-client-bundle/tests/steps.rs`: its arguments and its rerun paths (`rerun_paths`) name `loading.html` and `loading.css`, and the title.

Then the landing, by section 3's order:

3. **Read it**, and **settle it with the design:** `browser.md`, 2.5 already carries the request's words; anything left that disagrees is yours to settle in the design file and the account.
4. **The markup and the rules:** P4. `steps.rs`, `scripts/bundle.sh`, `tools/tiers/run`'s temper branch, `test_run.py`.
5. **The terminal:** a row saying it has no bundle and no placeholder, and its prints are untouched: check by `heavy cargo nextest run -p temper-client-shell-terminal`.
6. **Reprint:** no journey's pages change (their heads are the tool's); `heavy cargo nextest run -p temper-client-journeys -p temper-client-gallery` to show it.
7. **Prove it,** on a release bundle, in `~/src/fabrig/bundle-loading/after/`:
   - Build as `before` did (`figures.md`, `## Method`): the `web` module, the bundler, `temper`. With `--title`, `--placeholder`, `--placeholder-stylesheet`. Then one `TEMPER_BUNDLE=embed` `heavy cargo build --release -p temper`, to say the binary's size and that it serves the page with nothing beside it, by `curl`.
   - Sizes: each file, raw and `.gz`, with totals, against `## Before`'s `gzip -9`.
   - Answers: `curl -H 'Accept-Encoding: gzip' -D-`, and plain, on `temper serve` for the module; `gunzip`'s bytes equal the plain answer's; the policy's fields on both.
   - Load at 2 Mb/s by `driver/load.py`, three from fresh and one cached, at 1440: first paint and what it shows, bytes by then, something to read, the client's first node. No `Log.entryAdded` of source `security`, no exception. `encodedDataLength` of the module against its `.gz` file. The cached load shows the paper alone, then the client, with no placeholder.
   - A start: `temper serve`'s time to listening and `VmHWM`, as `## Before`.
   - **The render check,** section 4: each of `01.html` to `04.html` from the request's commit, under its `loading.css`, against the real page in that state, under `loading.css` from your tree: each element's computed style and box, then the screenshots, at 1440, 1000 and 420. And each state in words, beside the request's: the paper for 250 ms; downloading, with the estimate; the 8 s line; no byte for 15 s; `Starting…`; each failure: dropped, 404 (a bundle copy without the module and its form), the server not answering, no WebAssembly (`Page.addScriptToEvaluateOnNewDocument` deleting it), and an `init` that threw (a bad module); scripts off (`Emulation.setScriptExecutionDisabled`); handing over; reduced motion (`Emulation.setEmulatedMedia`); the tab's title. A figure's number and a total differ, by the release sizes and by what travels: that's a reason, in the account.
   - **The tier:** `python3 -I -B tools/tiers/run temper browser`, then `python3 -I -B tools/tiers/run skein browser`. CPU and wall, against budget, beside `## Before` and `browser-trim`'s 58 to 79 CPU s.
8. **The words:**
   - `figures.md`, `## After`, its first table a person's first paint, before and after.
   - `temper/docs/plan/bundle-loading/plan.md`, `## Measured`: the release sizes, raw and gzipped, the wait before and after, and how it was measured, in words that name no machine.
   - `temper/docs/design/draft/bundle-loading.status.md`, a **Measured** paragraph: the same sizes and how, and a start's time and memory.
   - `temper/docs/design/client/journeys.md`, 9: a line beside the other hand-made requests', that `bundle-loading`'s pages aren't journeys to reprint or land. `temper/docs/design/client/mockups/STATUS.md`: a row only if a group's row says what this changes (its page isn't a mock's); otherwise nothing, and the account says so. The client's `README.md` holds nothing of this, and the account says so.
9. Once: `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-shell-web -p temper-client-bundle --all-targets -- -D warnings`; `heavy cargo clippy -p temper-client-shell-web --target wasm32-unknown-unknown -- -D warnings`; `heavy cargo nextest run -p temper-client-shell-web -p temper-client-bundle`; `tools/tiers/test_run.py` as it runs.
10. **The account**, the last commit's message, section 5: one row per `loading.css` rule (there's no overlay), per page change, per state of the message, with the render check's result, and a row each for the terminal, the design and the mockups. Every row landed, or differing and why, or deferred and to whom. `Firefox's and Safari's progress` is deferred to `skein/docs/design/ui-web.md`, section 16.
11. `python3 -I -B tools/citations/citations.py --base origin/main`. Commit, push.

## Report

Which builder. The before-and-after table. Sizes. The tier's figures. The render check's result, with the paths of the screenshots beside the request's under `~/src/fabrig/bundle-loading/after/`. The account again. What's left, under `## Left`.

## Answers

2026-11-06 (temper/ui). Expand: diagnose it here. It's very likely this plan's: main passes that test in every after-landing run today (7.5 to 14 s, latest 39b490f13846), and it fails on your branch twice, so it isn't outside the change. First run it on the pristine base, once, to confirm. Then find what ctl waits on for 10 s. To check first: serve's client mount admits a bounded number of calls and now queues heads (ui-browser-red), so does the new loader or a gzip answer keep a connection or call open, starving ctl's? Also the page's new requests, gzip's work at start or per request, and the second tab's sign-in. Red first at the lowest tier; no limit raised, the story's assertions kept. If it's a real product wait, a person's ctl meets it too. Then the skein browser tier, as your acceptance says. The first-paint figures are very good, keep them.

2026-11-06 (temper/ui). Correction, from the user just now: plan execution only ever runs focused and fuzzy; nothing at after-landing's level, so no tools/tiers/run, no browser tier, temper's or skein's, not even that one test. We run optimistically and repair if after-landing flags. This replaces my last answer and your acceptance's tier runs. So: read why ctl could wait 10 s (the client mount's admission, a connection the loader holds), and if you can reproduce it in the focused suite, fix it, red first. If you can't within about 20 minutes, finish on focused and fuzzy and land, naming the suspected red and your notes in the report, for after-landing's repair to start from.
