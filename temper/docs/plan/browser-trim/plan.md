# browser-trim: temper's browser tier back under its budget

The tier is at 94.2 CPU s against its 85 s budget, with 13 tests against 4.3's cap of 12. No budget, timeout or limit moves.

## Parts and tasks

| Part | Task | Needs | What |
|---|---|---|---|
| `goals-link` | `draft-link` | | `start_goal_opens_chats_for_a_draft` goes |
| `slow-walks` | `why-slow` | | Where the two slow walks' time goes, measured; causes fixed, red first for a product cause; the playground and the rest for a missing cache or needless rebuild |
| `rows` | `trim-rows` | `draft-link`, `why-slow` | Each test cut to what a browser alone shows, as `notes/rows.md` calls them; three runs of the tier, before and after |

## Critical path

`why-slow`, then `trim-rows`. `draft-link` runs beside them.

## Decisions

- **The profile runs before the cut.** The request lists reading the rows before finding the causes. The rows are read and decided in this cut, but cutting comes after the profile, because the cut removes the walks being profiled and would hide a cause in the page.
- **No world test for Goals' link.** The request adds one only if nothing holds the link's target. The view's `the_link_shows_with_a_tracked_template` holds it, at a lower tier than the client world.

- **`Tier::demo` keeps its pace.** Nothing in the cut stories waits on the fakes any more, and an unpaced fake would do the whole landing in the background for CPU nobody reads. Only the public walk's fallback runs unpaced, if it must still wait for a landing.
- **The public walk reads a goal under way.** A landed goal's pages hold nothing a browser alone shows, and the view and jig hold them. This changes `browser.md`, 8.4. `trim-rows` has a stated fallback if the board won't hold still.
- **`browser.md`, 4.6 now matches the tier.** It listed the task pages' steps as a test of their own and left out `latest_returns…` and the rename's focus, so 11 changes and 14 and 15 are new. Surviving numbers stay, and item 6's citation of 4.1 is corrected. 4.2 loses the task pages' story.
- **No stored format, protocol or public interface changes,** unless `why-slow` finds a product cause that needs one.

## The budget's room

The request's figures suggest about 76 CPU s after the cuts: `start_goal` takes off 2.5 s, and the other three cuts about 16 s. That's an estimate from read code, not a measurement: `trim-rows` measures it. `chat-reading-c/reading-browser` and `sign-in-new-tab/browser-story` each add stories to this tier, which will spend that room, and the second may pass the cap of 12 tests in `browser.md`, 4.3. If `trim-rows`' base doesn't hold them, its figures don't count them.

## Overlaps not avoided

- `temper/tests/client-browser/tests/browser_all/browser_pages.rs`: `chat-reading-c/reading-browser` adds stories there. History's cut has to happen in that file.
- `temper/tests/client-browser/src/lib.rs`, `browser_accounts.rs`: `sign-in-new-tab/browser-story`. `trim-rows` cuts only the other test in that file.
- `temper/docs/design/client/browser.md`, 4.2: the same plan rewrites the round trip's paragraph, next to this plan's edit.
- `temper/crates/temper-client-domain/**`, `temper/crates/temper-client-view/**`: `ui-walk-3` and `chat-reading-c`, if `why-slow` finds a product cause there.
- `draft-link` and `why-slow` both touch `temper-client-browser-world`, in different files, so neither waits for the other.
