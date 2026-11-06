# The tier's tests, read against its rule

The rule is `temper/docs/design/client/browser.md`, section 4: only what a browser alone shows, and each test names it. These are the plan's binding calls, test by test. CPU and wall are the request's, at `ebc08ddafc43`, one machine under load.

| Test | CPU s | Wall s | Only a browser shows | Call |
|---|---:|---:|---|---|
| `story_a_walks_on_the_built_playground` | 18.0 | 13.6 | The built playground in its page, its download, and its log replayed natively | **Keep whole** |
| `history_keeps_the_chat_focus_and_filter_across_reload_and_back` | 15.9 | 23.1 | Back and Forward, a replaced history entry, the composer's focus, a filter's address across a reload, an address that's no page; the page's own bounds on the box's `Me` (4.6, 11) | **Cut to that** (below) |
| `a_visitor_reads_a_public_project_and_loses_it_when_made_private` | 14.1 | 27.3 | A signed-out page with no stream, its public reads, a 404 under the page's policy | **Cut the walk to landing** (below) |
| `latest_returns_a_chat_to_its_end_after_new_words_while_reading_back` | 7.6 | 4.0 | Scrolling with the browser's keys, and the view staying put | **Keep whole**: `chat-reading-c/reading-browser` builds beside it |
| `signing_in_reloading_reconnecting_and_signing_out_keep_the_cookie_contract` | 6.6 | 4.2 | The cookie, reloads and reconnects, signing out, focus in the menu | **Keep whole** |
| `cross_origin_requests_and_framing_cannot_act_as_the_person` | 6.0 | 2.2 | Origins and framing | **Keep whole** |
| `enter_starts_one_chat_keeping_a_shift_enter_break` | 5.4 | 2.4 | Keys in a text area, a `details` dismissed by a press or Escape, the panic page | **Keep whole** |
| `the_accounts_page_returns_after_a_key_send` | 4.4 | 2.0 | Back restoring an emptied field, Forward | **Cut the tail** (below) |
| `the_sign_in_leaves_the_page_and_comes_back_to_ready` | 4.3 | 1.7 | The tab's round trip | **Keep whole**: `sign-in-new-tab/browser-story` rewrites it |
| `a_cross_site_authorization_returns_a_code_and_the_links_state` | 4.0 | 1.6 | The strict cookie across sites | **Keep whole** |
| `closing_project_rename_returns_focus_to_the_summary` | 3.5 | 1.8 | Where the focus lands | **Keep whole** |
| `start_goal_opens_chats_for_a_draft` | 2.5 | 1.1 | Nothing of its own | **Gone** (`draft-link`) |
| `the_manifest_posts_once_and_history_waits_for_continue` | 1.9 | 0.9 | A form posted under `form-action`, and Back | **Keep whole** |

The request's table gives `browser_accounts`'s two tests as 4.4 and 4.3 without saying which is which: the order above is assumed.

## The history story

Stays on `Tier::demo`: `/` reaches the needs page only once the first run is over.

**Keeps:**

- `start_playground_chat`, whose focus check stays.
- Back to `/chats` and Forward: starting replaces `NewChat`.
- From the Chats page, the chat by its link, Back and Forward.
- `reload_chats`, moved here (4.6, 11).
- `/` showing the needs page; its New chat link; the composer focused at `/chats/new`.
- The Closed filter's address and `pressed(3)` across a reload; Back to `/chats` and `pressed(1)`.
- `/nowhere` showing the needs page at `/`.

**Cuts:**

- `change_and_close_steps`, `plan_and_agent_steps` and `playground_landed`. The client world holds them: `suite::chat::a_new_chat_lands_its_delegated_change_and_closes_in_your_chats`, `suite::b2::roadmap_b2_through_the_task_pages`, `suite::task::a_delegates_page_shows_its_brief_run_and_report`.
- The card's reads, including that the web's card has no `Steps` list: the view's `tests/delegation.rs`.
- The Closed filter's `see(Link T1)`: the chat is no longer closed.

## The public walk

**Keeps:**

- ctl's sign-in; the owner's printed link; `Try the playground`, up to Ready and Live.
- `visibility(public)`, the chat's cue, and the `Task 2` card, so that the goal exists.
- Signing out and `Step::Reopen`: the visitor is the signed-out page.
- Every visitor page: `/public`, the board, the goal's page, landings, private and unknown, empty, malformed.
- `check_page`'s forbidden words, on every page.

**Cuts:**

- From `owner_steps`: pressing `Task 2`, `see(heading(TITLE))`, both `change_until`s, `Passed`, `Approved`, `Landed`, and `change_until` itself.
- From the visitor's pages: `see(Find::text(b"ended"))`, `see(Find::text(b"landed"))`, `see(Find::text(b"Passed"))`, `Gate 1 · review · blocking`.
- From `check_page_fields`: the review task's title, and the `count() >= 2` on `TITLE`, which falls to `contains`: a delegate may not exist yet when the visitor reads.

The view holds a landed goal's pages (`tests/public.rs`: `the_task_and_forge_pages_use_public_links_and_mechanics`, `the_landing_pages_show_the_turn_queues_commits_and_proofs`). jig holds the index keeping an ended goal, and a chat never being named (`jig/tests/core/tests/suite/public.rs`).

## The accounts story

Cuts everything after the last `see(accounts())`: `Settings`, `Accounts`, `Your accounts · 1`, the disclosure, `Replace`, `Remove account`. The view's `tests/accounts.rs` holds what they read, and `temper/tests/client/tests/suite/accounts_page.rs` the key joining its account. The first `Step::Back` and re-entry into the emptied field stay, as do Forward and the address.
