+++
id = "head-reads-on"
part = "page-part"
kind = "feature"
needs = ["page-reading", "log-busy"]
files = [
  "skein/crates/skein-ui-web/src/page.rs",
  "skein/crates/skein-ui-web/src/page/web.rs",
  "skein/crates/skein-ui-web/src/tests/page/**",
  "skein/examples/ui/shell-web/src/reading_fixture.rs",
  "skein/tests/browser/tests/suite/log_reading.rs",
  "skein/testing/skein-fake-person-browser/src/**",
]
effort = "high"
minutes = 50
+++

# A log at its head goes on reading

## Goal

A real browser, with a person's wheel held at a bound log's head, reads page after page with nothing in between but the reads themselves, and throws nothing. It's generic: skein's alone, naming no temper.

What a check on this trunk saw at `55ea17c1b4`, in real Chromium on a real temper with fake backends: 66 messages of 492 bytes, 1280x720, the wheel held at the head. The check's scripts and output are outside this repository, so the figures here are all you have.

- **Reads came one at a time, 10.0 s apart** (20 s twice): 518 s to reach the chat's start. Each read was answered in under 135 ms, and the status showed for under 69 ms, so nothing was waiting on the server.
- **`scrollTop` stayed 0** throughout, with the log's height growing from 2,506 to 16,988 px.
- **One uncaught page exception in every run,** about 40 ms after the first wheel: `closure invoked recursively or after being dropped`, through `__wbg_dispatchEvent` to `HTMLDivElement.real`.

**Causes, from reading, not run; reproduce each first and say if one's wrong:**

- `Op::Measure` in `skein/crates/skein-ui-web/src/page/web.rs` dispatches a `scroll` event synchronously. When the apply runs inside the scroll listener's own turn, that re-enters the listener's closure, which wasm-bindgen refuses. The measure is lost.
- Whether that's what makes the 10 s isn't established. `skein/docs/design/ui-web.md`, section 7, *Reading a log*, wants `Near` again when the height moves while the edge stays near, and again when a log's busy state goes, so the head shouldn't wait on anything that ticks. Find what actually starts each read.
- Scroll anchoring doesn't work at offset 0, so a page landing at `scrollTop` 0 is the page part's to hold. `reading-browser` is fixing what the page part holds on to (commit `d6bb22ad80`, on its branch), and that may be part of this: read it, and take it from its branch's landing if it reaches the trunk first.

## Acceptance

Failing first, in the lowest tier that shows each. In `skein-ui-web/src/tests/page` where it's pure; otherwise in real Chromium in `skein/tests/browser/tests/suite/log_reading.rs`, reusing the reading fixture, as few page loads as prove it:

- **It goes on:** a bound log whose owner adds a page at each `Near { Head }` and holds it busy while the read is out, with the wheel at the head and no other input, reads until its head is one and a half screens off or nothing's left. The story reads the fixture's count, with no wait that a tick could satisfy: 8 pages in less time than two of whatever a tick is.
- **A read landing inside a scroll's turn** still measures once the apply is done, and reports `Near` if the edge is still near.
- **No uncaught exception:** the `log_reading` stories fail on any uncaught page exception. If the harness can't see one, add that to `skein-fake-person-browser` (Chromium's `Runtime.exceptionThrown`); every story that uses the harness may then fail on one, so say what else that finds, and fix nothing outside these files without asking.
- **The view holds:** the existing *position stays* story still holds, with and without `overflow-anchor: none`, including a page landing at `scrollTop` 0.

## Owns and may change

The files in the header. The tier's budget and its timeouts are the user's (`skein/docs/foundation/testing-strategy.md`, 8.1): a story takes the fewest page loads that prove it.

**On a doubt:** if the cause is outside skein (temper's chat page, or jig's thread), stop and report `Question:` with what you found.

## Inputs

- `skein/docs/design/ui-web.md`, section 7.
- `temper/docs/plan/chat-reading-c/notes/common.md`.
