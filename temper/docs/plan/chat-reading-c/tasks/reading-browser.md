+++
id = "reading-browser"
part = "browser"
kind = "feature"
needs = ["chat-page", "page-reading", "log-busy", "beside-store"]
files = [
  "temper/tests/client-browser/tests/browser_all/browser_pages.rs",
  "temper/crates/temper-client-shell-web/temper.css",
  "temper/tests/client-browser/src/lib.rs",
  "skein/testing/skein-fake-person-browser/src/harness/script.rs",
  "skein/testing/skein-fake-person-browser/src/face.rs",
]
effort = "high"
minutes = 50
+++

# A long chat is read by scrolling in a real browser

## Goal

The whole thing, in real Chromium on a real temper with fake backends, by what the browser tier runs. The tier runs Chromium alone: the position staying put is the one thing browsers differ on, and your report says plainly what's run and what's only read.

**Your base must hold `look-round-6`:** this plan stands on it, and `fabrig/ui` has priority. Check first: `rg -n 'State::ToLogEnd' temper/crates/temper-client-view/src/pages/chat.rs`. If that finds nothing, change nothing and report `Question: look-round-6 has not reached my base; wait for the catch-up?`, recommending yes.

Acceptance, in `temper/tests/client-browser/tests/browser_all/browser_pages.rs`, beside `latest_returns_a_chat_to_its_end_after_new_words_while_reading_back`, as few page loads as prove them:

- **Opening:** a long chat opens with its end in view, and the log's head at least one and a half screens above it, with no "Older" on the page. A short one shows whole, with its start and no status.
- **Reading back:** PgUp from the composer, with the focus held there, scrolls the conversation, and earlier lines load as the head nears. "Loading earlier messages…" shows only while a read is out.
- **The position stays:** the words in view don't move on screen when lines load above, within 1 px: `getBoundingClientRect` on a chosen line before and after a page reads in. A run with `overflow-anchor: none` injected on the log, which is what Safari gives, and one without.
- **The end comes back:** past the window's bound, the pill shows, the foot of the window reads later lines as it nears, and they stay put in the same way; a press of "Latest" from far back goes to the end.
- **Sending from far back** goes to the end and shows the words.
- **New words while reading back** don't move the view: this story exists from `look-round-6`. It stays, and it holds.

If `temper.css` needs a hunk for what a state now does, it's the least, under `/* ==== ui: chats ==== */`, and it keeps `#app > .temper-session .temper-thread > [role=log]`'s `position: relative`.

## Inputs

- `temper/docs/design/client/README.md`, 5.15.
- `skein/docs/design/ui-web.md`, section 7, *Reading a log*.
- `temper/docs/plan/chat-reading-c/notes/common.md`.
- `skein/docs/foundation/testing-strategy.md`, 8.1: the browser tier's budget is the user's.

## Owns and may change

Owns the files in the header. `skein/testing/skein-fake-person-browser/src/harness/script/growth.rs` shows the shape of the one step this task adds there; a module beside it is this task's too. No page, shell or browser-kit code gains a hook for this.

Options, each decided:

- **Fake backends:** a long chat comes from the fake's agent speaking in long pieces, or from a fixture's activity if it has one. Take what `latest_returns_a_chat_to_its_end_after_new_words_while_reading_back` takes, longer.
- **A fault below** is reproduced and fixed in the lowest tier that shows it (`skein/tests/browser` for the page part) in a commit that says so, and named in your report. If it needs another task's interface changed, it's a `Question:`.
- **Over budget** is red: does a story belong here; can it be made faster; otherwise your report's measurements go to the user.

Not this task: the journeys' prints, design text.

## Steps

0. In the harness, failing unit test first, beside `growth.rs`'s: `Step::KeepBox { find: Find, name: Box<[u8]> }`, `name` keeping `<y> <height>` in whole CSS pixels from the find's box (`BrowserFace::find_boxed`). In temper's world, `Tier::demo_without_anchoring(name)`.
1. The stories, one at a time, with `heavy cargo nextest run --profile browser-temper -p temper-client-browser-world --test browser_all -E 'test(=NAME)'`.
2. Break the rule once for the position story: with the page part's `PutView` off, it fails with `overflow-anchor: none`; restore.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p temper-client-browser-world --all-targets -- -D warnings`; one whole `browser_pages` run. Commit and push.

## Report

As the implementer's text asks, with: wall and CPU against the tier; and, plainly, for each of Chromium, Firefox and Safari, what's run and what's only read.

## Answers

2026-11-06. **May this task add test-only fixture hooks outside its two owned files, or should a prerequisite extend the browser harness?** Neither: no hook in temper's shell, no new request in the kit, and no prerequisite. The kit already gives a node's box, and temper's world already serves a private copy of the bundle. The acceptance stands whole; the header above now lists the files this needs.

- **Measuring, no script run:** `skein_browser::boundary::Query.boxes` returns `Seen.rect`, the box in viewport coordinates, `y` signed and whole pixels, so a line above the view is negative (`skein/docs/design/browser.md`, section 5). `Step::Taller` reads it through `BrowserFace::find_boxed` and `height`. Add `Step::KeepBox { find, name }` to the harness the same way: it fails the story on an absent find or a find with no box, and keeps `<y> <height>` for `Harness::kept`, as `Keep` does. `Request`, `Event` and the rule that `document` is the one expression stay as they are.
- **The position:** a numbered paragraph as the chosen line. `KeepBox` it, read in (`See` a line that wasn't on the page, so the read has landed), `KeepBox` it again, and compare with skein's `held`, within 1 px. A rounded `y` is what the 1 px allows.
- **The head at opening:** `KeepBox` the log (`Role::Log`) and a numbered line that the story's sizes put past one and a half screens: the line's `y` is at or above the log's `y` less one and a half times the log's height, and the end's line is in the log's box.
- **`overflow-anchor: none`, injected by the bundle:** `Tier::demo_without_anchoring(name)` is `Tier::demo` on a private bundle, made as `Tier::accounts` makes its own (`copy_bundle`, `application.clients.bundle`), with `#app > .temper-session .temper-thread > [role=log] { overflow-anchor: none !important; }` appended to the digest directory's `temper.css`. Nothing checks a bundle file against its digest. Step 2's break, `PutView` off and this story red, is what shows the rule took.
- **Budget:** the second run is one more page load of the position story alone, not of every story. The tier's 85 s a suite and 60 s a test stand.
