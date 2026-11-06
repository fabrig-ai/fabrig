+++
id = "copy-part"
part = "copy-part"
kind = "feature"
needs = []
files = ["skein/crates/skein-ui-web/src/clipboard.rs", "skein/crates/skein-ui-web/src/clipboard/web.rs", "skein/crates/skein-ui-web/src/tests/clipboard.rs", "skein/examples/ui/shell-web/src/copy_fixture.rs", "skein/tests/ui-web/tests/browser_stories.rs"]
effort = "high"
minutes = 50
+++

## Goal

Skein's browser parts write the clipboard: one write, asked in a person's act, gives exactly one report. It's a generic mechanism and names no application.

Acceptance:

- `skein_ui_web::clipboard`, as `temper/docs/plan/box-small/notes/contracts.md`, *The clipboard*, has it.
- `rejected` has a host test: `NotAllowedError` refused; any other name, and none, failed. Break it by hand and see it fail.
- On wasm32: `navigator.clipboard` missing, reported refused at once; a text that isn't UTF-8, refused at once; the promise resolved, done; rejected, by `rejected`. Each write reports once.
- A test build's `/test/copy` page and a story in `browser_stories`, as `skein/docs/design/ui-web.md`, 14.3, *A clipboard write*, says.
- Breaking the gesture by hand (deferring `write` past the listener, past the browser's activation window) makes the story fail, and the report says what it saw. Restore it.
- Remove `skein/docs/design/ui-web.status.md`'s **Not yet** line naming *The clipboard*. This task may make that edit, against the implementer's text.

## Inputs

- `skein/docs/design/ui-web.md`, 5.1 (the table's row), section 11, 14.3.
- `skein/docs/design/ui.md`, section 13.
- `temper/docs/plan/box-small/notes/contracts.md`, *The clipboard*; `temper/docs/plan/box-small/notes/common.md`.

## Owns and may change

- Owns the files in the header, creating all but `browser_stories.rs`.
- May add: `pub mod clipboard;` to `skein/crates/skein-ui-web/src/lib.rs`; `mod clipboard;` to its `tests.rs`; `mod copy_fixture;` and its address's branch to `skein/examples/ui/shell-web/src/lib.rs`, each under `#[cfg(debug_assertions)]`, as `folds_fixture` has them.
- `web-sys` is 0.3.103: `Navigator::clipboard` is typed as always there, and it isn't. Read `clipboard` with `js_sys::Reflect` first, and call `writeText` with `js_sys`' `Function`, or add the `"Clipboard"` feature to `skein/crates/skein-ui-web/Cargo.toml` if you call it through `web-sys`. Add no `cfg` flag. `fetch/web.rs`'s `listen` is the pattern for a promise.
- `sign-in-new-tab/web-tab-story` is on `shell-web/src/lib.rs` and `browser_stories.rs`, and its `tab_fixture.rs` is the nearest pattern, if it's on your base. If not, `folds_fixture.rs` is. Your hunks are additions.
- If a queued press does break the gesture, the fix is to make the press synchronous for this act, in skein. If that stays in `skein-ui-web/src/inbox.rs` or `page/web.rs` and under 30 lines, make it and say so; if larger, it's a `Question:`.
- If `skein/testing/` lacks something the story needs beyond what's named here, the story holds what it can and the report says what it doesn't. Don't grow the kit.

## Steps

`copy_fixture::start(mount, primary)`, on the pattern of `folds_fixture`: its own `Inbox` and `Dom`, a small `View`, a `Clipboard::new(Rc::clone(&inbox), Report::Copied)`. One button "Copy" and one status line, the line reading `Ready`, then `OUTCOME ACTIVATION, REPORTS`:

- `OUTCOME`: `Done`, `Refused` or `Failed`.
- `ACTIVATION`: `active` or `idle`, `navigator.userActivation.isActive` at the moment `write` was called (`js_sys::Reflect`).
- `REPORTS`: how many reports have come.

Its addresses: `/test/copy`; with `?at-start`, it writes from its first turn, with no press; with `?no-clipboard`, it first sets `navigator.clipboard` to undefined with `Object.defineProperty` (`js_sys`), then waits for the press.

The story, in `browser_stories`:

1. `/test/copy`; press "Copy"; the status reads `active, 1`, and `Done` if Chromium grants a trusted press the write, which you'll see. If it refuses a trusted press on the builder (headless may), the row holds `Refused active, 1`, and the report says so plainly: never loosen the row to "either".
2. A second person at `/test/copy?at-start`: `Refused idle, 1`.
3. A third at `/test/copy?no-clipboard`; press "Copy": `Refused active, 1`.

Run: build as `skein/scripts/browser.sh` does, one `heavy` command at a time, then `heavy cargo nextest run --profile browser-skein -p skein-ui-web-tests --test browser_stories -E 'test(=NAME)'`, `SKEIN_UI_EXAMPLE_BUNDLE` set. If the builder has no Chromium, say so plainly, and don't count it as run.

Finish: `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-ui-web --all-targets -- -D warnings`; `heavy cargo clippy -p skein-ui-web --target wasm32-unknown-unknown -- -D warnings`; `heavy cargo clippy -p skein-ui-example-shell-web --target wasm32-unknown-unknown -- -D warnings`; `heavy cargo clippy -p skein-ui-web-tests --all-targets -- -D warnings`; `heavy cargo nextest run -p skein-ui-web`. Citations. Commit and push.

## Report

As the implementer's text asks. Say plainly what ran on Chromium, and whether a trusted press got `Done`.
