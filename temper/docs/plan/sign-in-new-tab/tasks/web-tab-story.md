+++
id = "web-tab-story"
part = "web-tab"
kind = "feature"
needs = ["web-tab", "kit-tabs"]
files = ["skein/examples/ui/shell-web/src/tab_fixture.rs", "skein/examples/ui/shell-web/src/lib.rs", "skein/tests/ui-web/tests/browser_stories.rs", "skein/tests/ui-web/tests/support/**"]
effort = "high"
minutes = 50
+++

## Goal

Chromium runs skein's tab through the page's own press path: `Dom`'s click listener, the inbox's turn, `open_tab`. So the gesture holds in the tier that runs, and a queued press would fail here.

Acceptance:

- A test build's `/test/tab` page and a story in `browser_stories`, as `skein/docs/design/ui-web.md`, 14.3, *A tab*, says.
- Breaking the gesture by hand (deferring `open_tab` past the listener with a `setTimeout` of more than the browser's activation window, or calling it at start) makes the story fail, and the report says what it saw. Restore it.
- Remove section 10's and 14.3's **Not yet** line from `skein/docs/design/ui-web.status.md`. This task may make that edit, against the implementer's text.

## Inputs

- `skein/docs/design/ui-web.md`, section 10, *A tab of the page's own*; section 11, *A person's gesture*; 14.2, *Its tabs*; 14.3, *A tab*.
- `temper/docs/design/draft/sign-in-new-tab.md`, 2.
- `temper/docs/plan/sign-in-new-tab/notes/common.md`.

## Owns and may change

- Owns the files in the header. May add `mod tab_fixture;` and its address's branch in `shell-web/src/lib.rs`, each under `#[cfg(debug_assertions)]`, as `folds_fixture` has them.
- If a queued press does break the gesture, the fix is to make the press synchronous for this act, in skein first. If that fix stays in `skein-ui-web/src/inbox.rs` or `page/web.rs` and is under 30 lines, make it, and say so in the report; if larger, it's a `Question:`.

## Steps

`skein/examples/ui/shell-web/src/folds_fixture.rs` is the pattern: a debug-only page, its own `Inbox` and `Dom`, built from a small `View`. `tab_fixture::start(mount, primary)` draws four buttons and a status line:

- "Open": `platform.open_tab(b"Opening the page…")`; the status reads "Opened" or "Refused".
- "Point": `point_tab(b"/test/tab/landed")`.
- "Point outside": `point_tab(b"javascript:alert(1)")`.
- "Close": `close_tab()`.

With `?at-start`, it calls `open_tab` from its first turn, with no press, and the status reads what came of it.

The story, in `browser_stories`, through `Step::Tabs`:

1. `/test/tab`; press "Open"; status "Opened"; one tab, title "Opening the page…", `opener` true.
2. Press "Point"; one tab, address holding `/test/tab/landed`, `opener` false.
3. Press "Open", then "Point outside"; the first tab alone remains.
4. Press "Open", then "Close"; the first tab alone remains.
5. A second person at `/test/tab?at-start`: status "Refused", no tab.

The landed tab's page draws the example at an unknown note or nothing of note: no trouble may be reported of it, since the kit doesn't attach there.

Run: build as `skein/scripts/browser.sh` does, one `heavy` command at a time, then `heavy cargo nextest run --profile browser-skein -p skein-ui-web-tests --test browser_stories -E 'test(=NAME)'`, `SKEIN_UI_EXAMPLE_BUNDLE` set. If the builder has no Chromium, say so plainly, and don't count it as run.

Finish: `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-ui-example-shell-web --target wasm32-unknown-unknown -- -D warnings`; `heavy cargo clippy -p skein-ui-web-tests --all-targets -- -D warnings`. Commit and push.

## Report

As the implementer's text asks. Say plainly what ran on Chromium and what's only read for Firefox and Safari.
