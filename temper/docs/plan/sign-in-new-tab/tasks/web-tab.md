+++
id = "web-tab"
part = "web-tab"
kind = "interface"
needs = []
files = ["skein/crates/skein-ui-web/src/platform.rs", "skein/crates/skein-ui-web/src/platform/web.rs", "skein/crates/skein-ui-web/src/tab/**", "skein/crates/skein-ui-web/src/tab.rs", "skein/crates/skein-ui-web/src/lib.rs", "skein/crates/skein-ui-web/Cargo.toml"]
effort = "high"
minutes = 40
+++

## Goal

skein's web platform gains a tab of the page's own: opened with a line of words, pointed at an address under the navigation's target rule with its opener cut first, closed, and answering whether the browser refused it. Temper's shell builds on this surface. The story that runs it in Chromium is the later task `web-tab-story`.

Acceptance:

- `skein_ui_web::platform::Platform` has `open_tab`, `point_tab` and `close_tab`, with the defaults below, so that `Worker`, `skein-playground`'s `Memory` and every fixture compile unchanged.
- `platform::Browser` carries them through a new `tab` part.
- A native test in `skein-ui-web` holds the defaults: a platform with no tabs answers `false` and does nothing else.
- `heavy cargo check -p skein-ui-web --target wasm32-unknown-unknown` passes.

## Inputs

- `skein/docs/design/ui-web.md`, section 10, *A tab of the page's own* and *The navigation away*; section 11, *A person's gesture*.
- `temper/docs/design/draft/sign-in-new-tab.md`, 2.
- `temper/docs/plan/sign-in-new-tab/notes/common.md`.

## Owns and may change

- Owns the files in the header.
- May add the `web-sys` features the part needs to `skein/crates/skein-ui-web/Cargo.toml` (`Window` and `Document` are already there).
- Don't change `skein-ui-web/src/transport.rs`: the tab is outside the transport's whole. Don't change `skein/crates/skein-ui-web/src/inbox.rs` or `page/web.rs`: the press already reaches the turn inside the click's listener. If you find otherwise, stop and say so in a `Question:`.
- If `web_sys` offers no setter for `opener`, use `js_sys::Reflect::set` and say so in the report.

## Steps

The sketch, binding in its names and signatures:

```rust
// skein-ui-web/src/platform.rs, in trait Platform
/// Opens the page's tab holding a line of words, in the turn of a person's act;
/// false when no tab opened (ui-web.md, 10).
fn open_tab(&mut self, _words: &[u8]) -> bool { false }
/// Points the page's tab at an address, its opener cut first, and lets it go (ui-web.md, 10).
fn point_tab(&mut self, _to: &[u8]) {}
/// Closes the page's tab, if one is held (ui-web.md, 10).
fn close_tab(&mut self) {}
```

```rust
// skein-ui-web/src/tab.rs, a part laid out as `title` and `timer` are
#[cfg(target_arch = "wasm32")]
pub mod web;

// skein-ui-web/src/tab/web.rs
pub struct Tab { held: Option<web_sys::Window> }
impl Tab {
    pub fn open(&mut self, words: &[u8]) -> bool;
    pub fn point(&mut self, to: &[u8], loopback: bool);
    pub fn close(&mut self);
}
```

Inside the part:

- `open`: closes a tab still held; `window.open_with_url_and_target("", "_blank")`, an empty address so that the tab's first document stays synchronously reachable on Firefox too (read, not run). `Err` or `None` answers `false`. Sets the document's title and the body's text to `words`, as UTF-8 or nothing. A missing document or body skips the words and never panics: the words aren't what the tab is for.
- `point`: takes the handle. With no handle, does nothing. `navigation::allowed(to, loopback)` false closes the tab. Otherwise sets `opener` to null, then `location().replace(to)`, so that the blank entry is not in its history. A failure of either closes the tab. Never panics: the provider's page and the browser own that tab.
- `close`: takes the handle and closes it.
- `Browser::point_tab` passes `cfg!(feature = "loopback-navigation")`, as `navigate` does.
- Module doc cites `ui-web.md`, section 10. Add a comment at `open` saying why no `noopener` is passed.

Test: in `skein-ui-web/src/tests/`, beside its other platform tests, a small struct implementing `Platform` with no tab methods (or `tests/fixtures.rs`'s `FakePlatform`, if it takes none) holds the defaults. `heavy cargo nextest run -p skein-ui-web --lib -E 'test(tab)'`, reading the count.

Finish: `cargo fmt`; `python3 -I -B skein/scripts/old_forms.py`; `heavy cargo clippy -p skein-ui-web --all-targets -- -D warnings`; `heavy cargo clippy -p skein-ui-web --target wasm32-unknown-unknown -- -D warnings`; `heavy cargo nextest run -p skein-ui-web`; `heavy cargo check -p skein-playground -p jig-client-shell-web -p temper-client-shell-web --tests`. Commit and push.

## Report

As the implementer's text asks. Name what you read and did not run of Firefox and Safari.
