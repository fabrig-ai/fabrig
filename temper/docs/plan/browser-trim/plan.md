# browser-trim: temper's browser tier back under its budget

The tier is at 94.2 CPU s against its 85 s budget, with 13 tests against 4.3's cap of 12. No budget, timeout or limit moves.

## Parts and tasks

| Part | Task | Needs | What |
|---|---|---|---|
| `goals-link` | `draft-link` | | `start_goal_opens_chats_for_a_draft` goes |
| `slow-walks` | `why-slow` | | Where the two slow walks' time goes, measured; causes fixed, red first for a product cause |

A third task, `trim-rows`, follows in the next push: it cuts each test to what a browser alone shows, then measures the tier three times.

## Critical path

`why-slow`, then `trim-rows`. `draft-link` runs beside them.

## Decisions

- **The profile runs before the cut.** The request lists reading the rows before finding the causes. The rows are read and decided in this cut, but cutting comes after the profile, because the cut removes the walks being profiled and would hide a cause in the page.
- **No world test for Goals' link.** The request adds one only if nothing holds the link's target. The view's `the_link_shows_with_a_tracked_template` holds it, at a lower tier than the client world.

## Overlaps not avoided

- `temper/tests/client-browser/tests/browser_all/browser_pages.rs`: `chat-reading-c/reading-browser` adds stories there. History's cut has to happen in that file.
- `temper/tests/client-browser/src/lib.rs`, `browser_accounts.rs`: `sign-in-new-tab/browser-story`.
- `draft-link` and `why-slow` both touch `temper-client-browser-world`, in different files, so neither waits for the other.
