# ui-history-red: the history story passes every time

The request: `inputs/request.md`, red-78. `temper-client-browser-world`'s `browser_pages::history_keeps_the_chat_focus_and_filter_across_reload_and_back` sometimes fails at step 107, the `needs_page()` keep after `/nowhere` (`temper/tests/client-browser/tests/browser_all/browser_pages.rs:181-183` on the trunk): `main.temper-needs` isn't there.

## Parts and tasks

| Part | Task | Needs | What it makes run | Minutes |
|---|---|---|---|---|
| `repair` | `tasks/repair.md` | | The story run by name, alone, until it fails, the page at step 107 recorded; the race fixed at its lowest place; at least 10 runs passing after | 150 |

**Critical path:** `repair` alone, about 150 implementer-minutes, most of them the story's runs.

## What was decided

- **One direct task, no survey,** as the request says: the cause is known only once the story runs, so one task runs it, by name and alone, as a red's repair may.
- **The lead's reading points at the story,** not the product, and the task says so as leads, not as a finding. Since `b5d1bc6daf` (look-round-22, "Keep an empty busy main until me is read"), the client draws an empty `<main aria-busy>` while `me` is read (`temper/docs/design/client/README.md`, 5.1). The story's `see(Find::exactly_named(Role::Main, b""))` meets it at once after the `/nowhere` page load. The `Step::Keep` after it reads once, without waiting, before `me` has drawn the Needs page. Before `b5d1bc6daf`, no main was drawn then, so the `see` waited. This is when the red began.
- **The default fix is a wait a person's client gives:** the rail's "New chat" link, which `crate::frame::rail_project` draws only once `me` gives a project, in the same build as `main.temper-needs`. It goes before the unchanged `see(Main "")` and `needs_page()` lines, at `/nowhere` and at `/` (same window, not yet seen failing). The story's checks stay.
- **A product fix is pre-approved** if the run shows a second page at `/nowhere` once `me` is read ("No project yet", another page, or the Needs page and then another). It goes under a failing focused test, as a row of `temper-client-view`'s frame table where the draw is the view's.
- **A change to what a busy main means to `See` across skein's browser harness is not this task's.** It would change every tree's stories; the task names it under `## Left` if the run argues for it.
- **The task writes this file's *Found*,** as the request asks: which case it was, and how many runs passed after the fix.
- **No design written:** the story's design (`temper/docs/design/client/browser.md`, 4.6, item 2) and the reading draw (`README.md`, 5.1) already say what the fix keeps. A product fix may add one status line under 5.1.
- **No notes:** one task. The request's `For every task of this plan` stands in for `notes/common.md`.

No stored format, protocol or public interface changes.

## Found

The story was early: run 2 failed at step 107 after `/nowhere`, with the top bar's
mark and an empty main while `me` was read, rather than another page's body.
`temper/tests/client-browser/tests/browser_all/browser_pages.rs` now waits for the
rail's "New chat" link before the unchanged main and Needs checks at `/` and `/nowhere`.
All 10 consecutive runs after the fix passed, by name and alone, with no added load,
the same conditions that reproduced the failure; no product change was needed.

## Overlaps with drafts beside this one

None for the default fix: no draft beside this one names `browser_pages.rs`. A product fix could touch `temper/docs/design/client/README.status.md`, which `look-round-24/path-scale`, `tui-box-front/*` and `client-binary/client-table` also edit. Those are other sections' lines, so they merge cleanly. No draft beside this one names `temper-client-view`'s `root.rs`, `outside.rs` or `frame.rs`, or `temper-client-domain`'s `pages.rs` or `route.rs`.
