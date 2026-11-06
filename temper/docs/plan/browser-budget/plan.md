# browser-budget: `browser-temper` back under 85 CPU seconds

The request: `inputs/request.md`. The repair of a flagged tier, in the order of `skein/docs/foundation/testing-strategy.md`, 8.1. No budget, timeout or tier membership changes.

## Parts and tasks

| Part | Task | Needs | What it makes run |
|---|---|---|---|
| `figures` | `tasks/story-baseline.md` | | The four stories' *before*, with load, and the long story's CPU split |
| `history` | `tasks/history-ctl.md` | | Lever 1: the history made through `temper ctl`, the page opened afterwards |
| `window` | `tasks/window-seam.md` | | Lever 2's surface: `Limits::window_bytes` and the shell's `small-window`, off |
| `delivery` | `tasks/delivery-cost.md` | | Lever 4: what one delivery costs at the whole bound; a fix only if small |
| `window-on` | `tasks/window-story.md` | `history-ctl`, `window-seam` | Lever 2 on: the tier's bundle at 131,072 bytes, the stories past it at both ends |
| `finds` | `tasks/find-backoff.md` | `window-story` | Lever 3: the patch measured on what's left, landed only if it pays |
| `close` | `tasks/budget-close.md` | all of the above | The account for the user: `temper/docs/development/browser-budget.md` |

Critical path: `history-ctl`, `window-story`, `find-backoff`, `budget-close`. `window-seam` runs beside `history-ctl` and is the shorter. `story-baseline` and `delivery-cost` run beside them and hold only `budget-close`.

## What this plan decided

- **The four stories run by name.** The lead's rules have a spec name no slow-tier run. The request overrides that for these four stories alone, and `AGENTS.md`, 5, lets a repair run what it repairs, by name, alone. So the specs name those runs, and no others. Never the whole tier.
- **The window is its own field, off a bundle feature.** The tier can't set `document_bytes` today: it's a constant in the web shell. Lowering it in the tier's bundle would bound every document in all seventeen stories, and thirteen of them can't be run here. `Limits::window_bytes` narrows a conversation's window alone. A feature keeps the seam out of a person's bundle, as `loopback-navigation` does; nothing reads a setting at runtime.
- **Three windows, not an eighth of everything.** Scaling replies to an eighth at sixteen messages would hold one window, barely passing the bound, and the request asks for two windows' worth past it at both ends. So the default is eight messages of about 50 KB. A new `gone` holds the earliest lines leaving on the way back down, which no step checks today.
- **`latest_returns` loses its second tab and its delegate.** Its new words come through ctl, since what it proves is the view holding, not who sent them. Its Task 2 is read by nothing in the story. `SENT` is its one typed message.
- **"It pays" and "not slower to notice" have numbers.** The patch lands if the long story's median CPU is at most 90% of the trunk's, lower in two of three alternated pairs, and no story's median wall rises past 105%. The patch's back-off can notice a change up to 150 ms later during a long wait, so wall is what holds the request's "must not" at a story's scale. The rule added to the patch, that any act of the person's clears the back-off, holds it step by step.
- **Keeping read Markdown isn't a small fix.** `temper/docs/design/draft/chat-reading.status.md`, 3, declined it as a new mechanism. `delivery-cost` reports what it would save, for the user to weigh against a person's cost.

## Belonging

`temper/docs/design/client/browser.md`, section 4, is the test: a story is in this tier only when no world can show it. No story moves, and budget played no part in that call. This is a cut's reading of the design and of the lower worlds' tests by name, not a run.

- **`a_long_chat_opens_at_its_end_and_reads_back_from_the_composer` stays.** The client world holds the window's logic on a tree (`suite::reading::reading_back_then_forward_recovers_whole_lines_and_follows_new_turns`), and the domain the end coming back (`the_end_comes_back_after_enough_older`). Neither has a layout. Only a browser shows the end in view with the head a screen and a half above it, PgUp scrolling from a focused text area, and real measurements driving the reads.
- **`latest_returns_a_chat_to_its_end_after_new_words_while_reading_back` stays.** `suite::reading::latest_replaces_the_past_with_the_ends_window`, `sending_from_the_past_returns_to_the_end_and_shows_the_persons_words` and `live_text_and_its_commit_wait_while_the_person_reads_back` hold what the tree does, and skein's `a_log_scrolled_away_returns_to_its_end_and_keeps_following` holds the page part on its example. Only here is the view seen holding and returning, in temper's page.
- **`earlier_words_stay_put_without_browser_scroll_anchoring` stays, and is the nearest overlap.** skein's `reading_without_browser_anchoring_follows_requests_and_page_keys` holds the page part without anchoring, on skein's example. This story holds it on temper's keyed lines and sheet, which `temper/docs/design/draft/chat-reading.status.md`, 5, *In every browser*, names as the nearest a run comes to Safari. Moving it would go to another tree's browser tier, not a lower world, and leave temper's lines and sheet unproved.
- **`a_short_chat_opens_whole_without_a_reading_status` stays, and is the closest call.** That no status or pill shows is the tree's, and a lower world could show it. That a short chat's start and end both stand inside the log, with nothing more read, takes a layout.

## What the numbers may not reach

- **The targets don't add up to 70 on the request's own figures.** The four at their targets, with the short story as it is, make about 27 CPU s (12, 6, 5 and 4.3). The other thirteen took 45 to 70, so the tier would stand at about 72 to 97: under 85 only if they stay under about 57, and never under 70 at their best. Lever 3 is the only one that reaches them, and they can't be run or measured in this plan. After-landing will say. If it's still over, a next repair could take the first lever to the other stories that start a chat through the page.
- **The stories could share setups.** `temper/docs/design/client/browser.md`, 4.3, has stories that share a setup become one test before a proof is dropped, and the tier holds seventeen tests against that section's cap of twelve. This plan keeps the four apart because the request measures them story by story and `-STOP` holds the whole server. It stays a lever for later, and the cap is the user's.
- **A ctl call beside an open page has missed its deadline once** (`temper/docs/design/client/browser.status.md`, 4.3), with no cause found. `latest_returns`' new words are now such a call, the only one of this plan's made with a page open.
- **The other thirteen stories run on the small window without being run here.** `window-story` reads them for a chat past 131,072 bytes, and expects none.

## Overlaps this plan could not avoid

`temper/docs/design/client/browser.md`, `browser.status.md`, `README.md` and `README.status.md` are also `model-check/trial-close`'s, `ui-public-2/public-page`'s, `long-words`' and `ui-proposal-card/card-close`'s: our hunks are small and in sections of their own. `window-seam` adds one literal field to `temper-client-view/src/tests/fixtures.rs` (`ui-qa-small/no-false-empties`) and a few lines to `temper-client-domain/src/pages/chat.rs` (`ui-proposal-card`, `long-words/start-reads-back`), away from what they change. A small fix from `delivery-cost` could meet `ui-pgup/page-follow` in `skein-ui-web`, `ui-pgup/live-kept` in `jig-client`, or `ui-proposal-card` and `long-words` in `temper-client-view`. `browser_pages.rs` and `temper/tests/client-browser/src/history.rs` are this plan's alone.
