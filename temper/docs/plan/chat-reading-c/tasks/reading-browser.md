+++
id = "reading-browser"
part = "browser"
kind = "feature"
needs = ["chat-page", "page-reading", "log-busy", "beside-store"]
files = [
  "temper/tests/client-browser/tests/browser_all/browser_pages.rs",
  "temper/crates/temper-client-view/src/pages/chat.rs",
  "temper/crates/temper-client-view/src/tests/task.rs",
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

2026-11-06. **Should the owning task repair the inherited `Act::Near` matches and give a catch-up, or may this task repair them outside its files?** Neither is left to do: catch up. Merge `origin/chat-reading-c/integration` into this branch first, and keep the `KeepBox` commit (`9eb575dc58`).

- **The two matches:** `journey/fields.rs` and `journey/originals.rs` already hold their `Act::Near` arms on the trunk (`80c3200e26`).
- **A third, found checking the trunk:** `browser_all` still failed on `PersonStep::Near` at `temper/tests/client-browser/tests/browser_all/browser_playground.rs:68`. This answer adds its one pass-through arm on the trunk, and `browser_all` compiles there.
- **If another inherited match stops `browser_all` compiling** after the catch-up, add the missing arm yourself, whatever file it is in, in a commit of its own that says so, and name it in your report. That is a compiler repair, not a change of scope: do not park on it. Anything more than an arm is still a `Question:`.

2026-11-06. **May chat-page move the unchanging context heading outside the bound log, or should page-reading gain a generic boundary identifying conversation entries?** Neither. The context stays in the log and draws only at the chat's start, and the day's label stops staying at the window's head. `chat.rs` and `tests/task.rs` are this task's now, for these two hunks alone. No page part change, no new interface, no look change, so nothing here waits on fabrig/ui.

- **Why:** the context is at the thread's head (`temper/docs/design/client/README.md`, 5.6), so it belongs where the chat starts, not at whichever line the window has reached. While a page remains above, an entry drawn above the lines is one the lines arrive below, and the first entry in view is what holds the view (`skein/docs/design/ui-web.md`, section 7). Moving it out of the log would pin a line above every long chat, which is a look nobody asked for.
- **The context:** in `conversation_head`, call `context` only when `chat.thread.lines.edges.reaches(End::After)`. A long chat opens with its head 1.5 screens up, so the context was out of view there already. It arrives with the chat's start, an insert above the words, which the page part holds.
- **The day's label has the same fault:** the context is only the first of two. `turn_time` keys the label by its date alone, so a same-day page read in above leaves it first in the log with the lines arriving below it, and with the context gone it is what the page part notes. Key it by the date and the first line under it: `Key { a: date as today, b: at.as_nanos(), ..fixed_key(46) }`. A page read in above then replaces the label at the head, and the page part goes on to the next noted entry that is still there. A label further down keeps its key, since its first line doesn't change.
- **Failing first, in the lowest tier:** in `temper-client-view`'s `tests/task.rs`, beside `latest_binds_where_the_window_let_its_end_go`: with an earlier page remaining the log draws no context, and with the chat's start held it draws it; and the day's label changes key when an earlier line of its day arrives, and keeps it when an earlier day's does. Then the position story, red today at 1,365 px, is the proof, as the acceptance says.
- **If it still moves:** another entry that stays at the head while lines arrive below it gets the same treatment in `chat.rs`, named in your report. That is no new question. `322c826760`'s status skip stays.
- **The rest stands:** the acceptance is unchanged, with everything your report lists as pending. The status clause in `skein/docs/design/ui-web.status.md`, 7, *The view stays*, is yours to remove once your skein commit lands with it. A journey that moves, a long playground chat losing its context until its start is read, is named for fabrig/ui.
