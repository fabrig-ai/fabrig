+++
id = "web-screen"
part = "web-screen"
kind = "feature"
needs = []
files = [
  "skein/crates/skein-ui-web/src/platform.rs",
  "skein/crates/skein-ui-web/src/platform/web.rs",
  "skein/crates/skein-ui-web/src/store.rs",
  "skein/crates/skein-ui-web/src/screen.rs",
  "skein/crates/skein-ui-web/src/screen/web.rs",
  "skein/crates/skein-ui-web/src/lib.rs",
  "skein/crates/skein-ui-web/Cargo.toml",
  "skein/crates/skein-ui-web/src/tests.rs",
  "skein/crates/skein-ui-web/src/tests/store.rs",
  "skein/crates/skein-ui-web/src/tests/screen.rs",
  "skein/crates/skein-ui-web/src/tests/fixtures.rs",
  "skein/docs/design/ui-web.status.md",
]
effort = "high"
minutes = 70
+++

# skein-ui-web: the kept store and full screen

## Goal

Two browser facilities a chat's terminal needs, generic in skein before temper uses them: the kept store, the browser's local storage beside the tab's small store, for a look a person chooses once; and full screen, entered in the turn of a person's act, with Escape held by the keyboard lock where the browser has it, and a report when the page leaves it. The shapes are `notes/contracts.md`, 1 and 2, binding. temper's shell uses both in a later task (`terminal-browser`).

Acceptance:

1. `Store::kept(prefix, bound)` reads, writes and removes through `Platform::read_kept`, `write_kept` and `remove_kept`, in the small store's encoding and under its bound; `Store::new` reaches the session methods as today. The trait's defaults read none and refuse every write, so the worker's and the playground's platforms build unchanged.
2. `Browser` implements the three over `window.local_storage()`, a missing or refused storage reading none and refusing writes.
3. `screen::changed` gives `Entered`, `Left` or `None`, as the contract says.
4. `screen::web::Screen` asks `requestFullscreen` on the document's element in `enter`, and reports `Refused` at once where the browser has no full screen or the request throws, and through the inbox where its promise rejects; it listens for `fullscreenchange` once, locks Escape on entering and unlocks on leaving, and reports by `changed`. `leave` exits full screen only where the page is in it.
5. `skein-ui-web` passes clippy for `wasm32-unknown-unknown` with warnings denied, as the gate checks it.
6. The `web-screen` **Not yet** entry in `skein/docs/design/ui-web.status.md` is deleted.

## Inputs

- `temper/docs/plan/look-round-16/notes/common.md`: the standing rules. This task is skein's: of the request's rules, the trailers, the runs and the checks hold.
- `temper/docs/plan/look-round-16/notes/contracts.md`, 1 and 2: binding.
- `skein/docs/design/ui-web.md`, 10, *The small store* and *The kept store*; 11, *A person's gesture*, *The clipboard* and *Full screen*.

## Owns and may change

Owns the header's files.

Departs from the implementer's text: `docs/design/` is otherwise closed, but the last commit deletes the `web-screen` entry in `skein/docs/design/ui-web.status.md`. If it can't close, it stays, and goes first under `## Left`.

Reader edits allowed:

- `skein/crates/skein-ui-web/Cargo.toml`: the `web-sys` features the web parts need (`Document`, `Element`, `Storage` and the like), one line each.
- A `Platform` implementor outside the header's files (`platform/worker/web.rs`, `tests/worker/fixtures.rs`, `skein/crates/skein-playground/src/memory.rs`): nothing, since the new methods have defaults; if one needs a line, that line alone.
- `skein/crates/skein-ui-web/src/tests.rs`: the `mod screen;` line.

Options, each decided:

- **`requestFullscreen`'s promise:** where `web-sys` returns no promise, call it by reflection, as `clipboard/web.rs` calls `writeText`, and settle its promise the same way; a rejection reports `Refused`.
- **The lock's promise:** fire and forget; a rejection changes nothing, as the design says.
- **The listener:** one `Closure` kept for the page's life (`into_js_value` or held in the struct), as the page's other listeners are.
- **Where the shell holds a `Screen`:** temper's shell, in `terminal-browser`; nothing in skein's example shell uses it.

Leave alone: `skein-ui`, the page's key handling (`page.rs`, `page/web.rs`), the transport (the kept store is not carried by it, ui-web.md, 10).

## Steps

### 1. The kept store

`platform.rs` gains the contract's three methods with their defaults; `store.rs` a place, session or kept, chosen by `new` or `kept`, read by `read_all`, `write` and `remove`; `platform/web.rs` the browser's local storage beside `storage()`'s session one.

### 2. Full screen

`screen.rs` and `screen/web.rs` as the contract has them, modelled on `clipboard.rs` and `clipboard/web.rs`; `lib.rs` gains `pub mod screen;` where `clipboard` sits.

### 3. The tests

- `src/tests/store.rs`, rows beside today's tests, no new test file: the `FakePlatform` gains `kept` records and calls; a kept write is base64 under the prefix and reaches `write_kept`, not `write`; a kept store reads only kept records, and a session store only session ones; a platform's default refuses a kept write. Fold these into the existing tests as rows where the test already loops; where none loops, one new test `the_kept_store_is_the_small_stores_form_in_local_storage` holds them as a table.
- `src/tests/screen.rs`, one new table test, `a_change_of_full_screen_reports_once`: the four `(was, now)` rows and their reports.

Break each rule once by hand (the place a kept write reaches, `changed`'s `Left`), run the test, see it fail, restore, and name each in the report.

### 4. The checks

One command at a time:

- `heavy cargo nextest run -p skein-ui-web --lib -E 'test(=NAME)'` for each test you added or changed.
- `cargo fmt`, then `python3 -I -B skein/scripts/old_forms.py`.
- `heavy cargo clippy -p skein-ui-web --all-targets -- -D warnings`.
- `heavy cargo clippy --target wasm32-unknown-unknown -p skein-ui-web -- -D warnings`.
- `heavy cargo nextest run -p skein-ui-web -p skein-playground`, and `heavy cargo check -p temper-client-shell-web -p jig-client-shell-web --tests` for the trait's other readers.
- `python3 -I -B tools/citations/citations.py --base origin/main`.

### 5. Commits

- **Trailers:** the kept store's commits `Look-request: chat-terminal` (the mode is remembered per browser); full screen's `Look-request: chat-terminal-esc`. One line above the other trailers.
- The last commit deletes the status entry. Push with `git push origin HEAD:refs/heads/<your branch>`.

## Report

The implementer's report, and:

- each rule broken and the failure seen;
- the `web-sys` features added;
- whether the status entry stands.
