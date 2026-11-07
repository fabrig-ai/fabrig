# browser-cap: temper's browser tier back to its twelve tests

`temper/docs/design/client/browser.md`, 4.3, caps the tier at 12 tests. It holds 18: the request's 17, and `browser_model_picker`, which reached main while this plan was cut. Six merge, none leaves the browser, and twelve remain. Nothing here runs in a browser: after-landing is the proof, and a red there comes back as a repair.

## The table

Every test of the tier on main at the cut (`0e81125709`).

| # | Test | What it checks | Where it goes | Nothing lost because |
|---|---|---|---|---|
| 1 | `browser_sign_in::signing_in_reloading_reconnecting_and_signing_out_keep_the_cookie_contract` | 4.6, 1: the cookie across a reload, a spent link, signing out and a restart | Stays | Unchanged |
| 2 | `browser_pages::history_keeps_the_chat_focus_and_filter_across_reload_and_back` | 4.6, 2 and 11: Back and Forward, the composer's focus, a filter's address, no page, the box's project across a reload | Stays, and takes row 17 | Unchanged but for its scripted turns, which it doesn't read |
| 3 | `browser_pages::enter_starts_one_chat_and_the_composer_keeps_the_focus` | 4.6, 4 and 5: Enter and Shift+Enter, the focus, a panic | Stays, and takes row 18 | Unchanged but for a second model offered, and the picker's steps, which hold the ones it had for that menu |
| 4 | `browser_attacks::cross_origin_requests_and_framing_cannot_act_as_the_person` | 4.6, 6 | Stays | Unchanged |
| 5 | `browser_authorize::a_cross_site_authorization_returns_a_code_and_the_links_state` | 4.6, 7 | Stays | Unchanged |
| 6 | `browser_accounts::the_accounts_page_returns_after_a_key_send` | 4.6, 8 | Stays | Unchanged |
| 7 | `browser_accounts::the_sign_in_opens_a_tab_and_the_page_moves_on_to_ready` | 4.6, 9: the tab, the page that stayed, its reload | Stays, and takes row 16 | Its steps as they stand; two tabs and two exchanges in all, one for each sign-in |
| 8 | `browser_github::the_manifest_posts_once_and_history_waits_for_continue` | 4.6, 10 | Stays | Unchanged |
| 9 | `browser_playground::story_a_walks_on_the_built_playground` | 4.6, 12 | Stays | Unchanged |
| 10 | `browser_public::public_board_and_task_lines_are_readable_at_desktop_and_phone_widths` | 4.6, 13 | Stays | Unchanged |
| 11 | `browser_pages::closing_project_rename_returns_focus_to_the_summary` | 4.6, 15 | Stays | Unchanged |
| 12 | `browser_pages::a_long_chat_opens_at_its_end_and_reads_back_from_the_composer` | The end in view, PageUp from the composer, reads held in place, the window dropped and recovered at both ends, Latest | Merges into `a_chat_opens_at_its_end_and_reads_back_keeping_its_place` (4.6, 14), story `long`, on T3 | Every step and measure stays, in order. Same lines, each reply's code line 2,000 bytes shorter |
| 13 | `browser_pages::earlier_words_stay_put_without_browser_scroll_anchoring` | The words held across a read above, by the page's own anchoring | Merges, story `unanchored`, on T2 | Every step and measure stays, in order. New: two measures that the rule held and went |
| 14 | `browser_pages::latest_returns_a_chat_to_its_end_after_new_words_while_reading_back` | The view holding across new words; Latest and a send returning the end | Merges, story `latest`, on T1 grown to six messages | Every step and measure stays, in order |
| 15 | `browser_pages::a_short_chat_opens_whole_without_a_reading_status` | No status or pill; both ends in the log | Merges, story `short`, on T1 at one message | Every step and measure stays, in order |
| 16 | `browser_accounts::the_welcome_sign_in_opens_a_tab_and_moves_on_to_ready` | `/welcome?at=model` to `/welcome?at=ready`, one authorization and one code exchange | Merges into row 7, story `welcome`, first, on a second provider | Every step stays. New, lower: `suite::second_sign_in`, which runs here and passes first |
| 17 | `browser_call_line::a_run_of_calls_opens_to_its_rows` | The folded line hiding its rows, open on a press, its node kept across a reply | Merges into row 2, story `call line` | Every step and the node's equality stay. `suite::call_line::opened_it_holds_one_row_a_call` already holds the rows' words; 4.6's list says so |
| 18 | `browser_model_picker::the_model_picker_at_the_composer` | One press applies a model and an effort, at 420 px and 1440 px, and a press away or Escape closes it | Merges into row 3, story `model picker` | Every step stays, and they hold the ones row 3 had for the same menu |

Twelve at the end: rows 1 to 11, and the reading test.

## What was decided

**A step can change anchoring on a live page, so reading is one test.** The harness can't evaluate script, but `Step::Viewport` needs nothing new: the private bundle's stylesheet puts `overflow-anchor: none` inside `@media (height: 801px)`, one pixel taller than the kit's 1280 by 800. The position story goes to that height on its live page and back at its end. Nothing could otherwise tell a rule that never matched from one that held, and a browser story that passed on default anchoring would have lost its proof without a sound, so the same block sets the log 7 px lower: the story measures that it moved, and the next story that it's gone. The existing rule had no such check, so this also tests its selector for the first time.

**Three chats, not one or four** (`browser.md`, 4.3).
- T1 is the short chat, which six messages through ctl then make Latest's: PageUp counts from the end, so a different head leaves its geometry alone.
- T2 is the position story's. It can't read the long chat: `browser-budget`'s window story keeps it inside one window because "a window dropping lines would change what it proves".
- T3 is the long story's. Its tuned reads run to the first message and back from it, so it can't carry a prefix.

**The long chat's replies shrink.** All replies share the fake's 524,288-byte joint allowance, and 420,272 for the long chat with 109,642 for the position chat don't fit. Raising the allowance isn't this plan's call. Its code lines drop from 10,000 words to 9,600, so 8 replies total about 404,272 bytes, still past 3 windows (393,216). The code line is one line, so no height changes; the window still holds two replies and can't fit a third's code line, so eviction takes the same lines. The position history can't shrink instead: its opening depends on its reply sizes in ways nothing here measures. A focused test feeds every reply through the fake's own admission, and every request's cue stays disjoint, since long's and position's used the same words.

**The call line goes to test 2, not the most-likely 4 and 5.** Test 2 is already `temper box`, starts its chat with `start_playground_chat`, and uses the same words the call line's script is cued on, so it changes `Tier::demo` for `Tier::demo_asked` and nothing else. Test 4 and 5's `temper serve` has no model to answer, so hosting it would rebuild that test's setup and every step that walks it. Nothing in test 2 reads the model's answer. The rows stay as steps because `details` hiding them is the browser's: the model's tree holds them whether or not they're open, as `opened_it_holds_one_row_a_call` shows by seeing them with no press.

**The welcome goes first, on a second provider.**
- The issuer has one subject, and one subject signing in again on one provider *renews*. That draws "Signed in to the same account again" where test 9 looks for "Signed in to Test provider" (`jig-domain-accounts`, `signing.rs`; `suite::sign_in::renewed_sign_in_explains_the_same_account_on_connect_model`).
- Running 9 first instead would leave a usable model, and the welcome offers no sign-in then. So the welcome signs in first, on a second provider of the same issuer.
- A provider claims its models alone, so the second one needs its own model. Two limits of the fixture's issuer also rise: `codes` from 1 to 2, since a redeemed code keeps its place, and `connections` from 2 to 4, since each tab may hold one. Neither is a budget or a deadline.
- Each tab stays open, so the second `Tabs` step expects two.
- This path is new and can't run here, so `suite::second_sign_in` runs it in the client world, and a focused test puts the two-provider deployment through startup validation.

**A failed step names its story, with `Stories`** (`notes/contracts.md`). `Outcome::Failed` already gives the person and the step, so a small type in the tier's crate maps a step to the story it stands in, and no harness changes. Measures read after a run carry the story in their kept names and messages.

**The forty seconds stay.** The harness ends every run at 40 s, 4 of them its close window. Reading's four wall medians add to 25.761 s apart, and share a setup now, against 36 s. None of this has run together. If after-landing finds it doesn't fit, `browser.md`, section 5, has the default: two tests, the long chat apart, with test 15 becoming a step of 4 and 5, its same setup, to stay at twelve.

## Parts and tasks

| Part | Task | Needs | Files |
|---|---|---|---|
| `steps` | `stories-named` | none | `src/story.rs`, `src/history.rs` |
| `pages` | `reading-one-test` | `stories-named` | `browser_pages.rs`, `browser_call_line.rs`, `browser_model_picker.rs`, `main.rs`, `src/reading.rs`, items of `src/lib.rs` |
| `welcome` | `welcome-step` | `stories-named` | `browser_accounts.rs`, `src/issuer.rs`, an item of `src/lib.rs`, `suite/second_sign_in.rs`, `front/account_acts.rs` |

All of `browser_pages.rs` is in one task, as the request asks: reading, the call line and the picker. The critical path is `stories-named`, then `reading-one-test`. `welcome-step` runs beside it. Both edit `src/lib.rs`, in different items, and `stories-named` adds its export before either starts.

## Overlaps this plan couldn't avoid

- **`look-round-13/sheet-story`** owns `browser_accounts.rs`, `browser_pages.rs` and `src/lib.rs`. It also reads stories that walked removed parts of `ConnectModel` as the page now does, so it and `welcome-step` meet on test 9.
- **`look-round-13/model-cards` and `model-end`** own `suite/connect_model*.rs` and rework Ready. `suite::second_sign_in` is a file of its own to stay off them, but it reads the same words.
- **`model-catalogue-2/process-fixtures`** owns `temper/tests/client-browser/src/**` and may change how models are configured, which the second provider and the second model depend on.
- **`model-catalogue-2/models-proof` and `look-round-13/look-proof`** edit `browser.md`.
- **`watch-room/red-50`** changes `browser_public` and the world's measuring. This plan edits neither.

## The cap can break again

Two drafts in flight add browser tests this plan can't merge, as they aren't on main: `model-catalogue-2/models-story` adds `browser_models.rs` with two, and `look-round-13/sheet-story` adds `browser_connect_flows.rs` with two. When they land, the tier is at 16. 4.3's rule holds for them too. Nothing fails when that happens, because no test counts the tier.
