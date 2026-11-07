# skein-browser-starts: fewer Chromium starts in skein's browser tier

The request: `inputs/request.md`. `skein/docs/foundation/testing-strategy.md`, 8.1, in its order, for the tier's 102 tests: what doesn't belong goes down, and what belongs shares a start. No budget, timeout or deadline changes. `walk = false`, and the tier isn't run: a merged test is compiled, and after-landing runs it.

This file is brought up to date with each batch the cut pushes.

## The result

**102 tests and 105 Chromium starts become 23 tests and 23 starts.** That passes the aim of about 70, because the request's own rule goes further: tests of one binary that use one server and setup become one test. Nothing was dropped to get there, and every assertion has a row below.

| Binary | Tests before | Tests after | Starts before | Starts after |
|---|---:|---:|---:|---:|
| `skein-browser-world::browser_real` | 33 | 9 | 35 | 9 |
| `skein-ui-web-tests::browser_harness` | 35 | 2 | 36 | 2 |
| `skein-ui-web-tests::browser_stories` | 17 | 8 | 17 | 8 |
| `skein-ui-web-tests::browser_example` | 12 | 3 | 12 | 3 |
| `skein-browser-world::browser_facts` | 5 | 1 | 5 | 1 |
| Total | 102 | 23 | 105 | 23 |

**What that should buy, by reading alone:** at the request's 0.6 CPU s a start, 82 starts are about 49 CPU s of a tier that reads from 99 to 167. The figure comes after landing. Nothing here measured it.

**What it costs:** a merged test reruns whole on a flake, and a failure may hide the stories after it. The request takes that for the budget. Every merged browser test is also unproven until after-landing runs it, so expect repairs there: the likeliest is `Rig::cases`, new logic that only compiles in this plan. `a_person_closed_opens_again_with_its_tokens` holds the kit's half of it in focused.

**skein's focused suite gains 13 tests.** They hold 31 tests' assertions, and cost the suite about 0.4 CPU s by estimate (13 processes at 8 ms and 14 harness runs at about 20 ms), against a suite that reads 38.2 of 30 (`skein-limits-heavy`'s request). `harness-world` reports their times. They belong there by what they check, and they cost milliseconds. Their children are small programs, a server, a fake browser and the commands, not Chromium's process tree. That is this plan's reading of the request's "not a process tree".

## Parts and tasks

| Part | Task | Needs | Rows | Minutes |
|---|---|---|---|---:|
| `in-turn` | `tasks/in-turn.md` | | the form 27-33 and 56-67 take | 25 |
| `harness-world` | `tasks/harness-world.md` | | 68-102 | 75 |
| `kit-facts` | `tasks/rig-cases.md`, in the next batch | `in-turn` | 1-38 | 90 |
| `example-tier` | `tasks/example-merges.md`, in the next batch | `in-turn` | 39-67 | 75 |

**Critical path:** `in-turn`, then `rig-cases`: about 115 minutes. `harness-world` needs nothing. All four are effort medium, as the request asks.

**Overlaps:** the listed files of the drafts beside this one don't touch these. Plans that add stories to `skein/tests/ui-web` can meet `harness-world` and `example-merges` there, in `browser_stories.rs` and `lib.rs`: expect catch-ups, as the request does. `example-merges` keeps a story the table doesn't name working, and names it.

## What this plan decided

- **The seam is `Setup.chromium`, a path.** The harness runs whatever is there, so a fake browser needs no change to the harness and none to what jig's and temper's tiers call. It is `harness_fixture` started with `--headless`. An in-process fake would need a new `Setup` and a second loop path, to save one small process.
- **A failed run needs its own run,** since the first failure ends one. That's why six command failures stay six lower tests, and why `the_harness_fails_a_page_whose_module_throws` keeps a start.
- **Lower tests merge too,** since focused is over its budget: where runs were the same but for their assertions, they are one run. Failed runs and the tests with their own setup stay whole.
- **The kit holds four people in a run** (`Limits::persons`), and the example's server keeps its notes for a run. Those two decide each merge's form: a person each where stories can run side by side, in turn where there are more than four or they must follow on a known store.
- **`Harness::run_stories` plays stories in turn** and names the story in a failure. One page is also cheapest, since a person's context costs a renderer.
- **`Rig::cases` gives each case a person and a page,** closed before the next, so that each fact still has what a fresh browser gave it: its own storage and its own tab. The rig's one minute stays one minute for the whole run. Its loop's guard against a run that doesn't settle counts per case, as each test had one; it's a count of turns, not time.
- **The rig's groups are kept small enough to fit its minute by a wide margin,** and to hide little: acts, reports, reloads and dialogs, downloads, tabs.
- **`reading_without_browser_anchoring_follows_requests_and_page_keys` stays alone.** It is the longest story in the tier, and merged it would press on the forty seconds with no measure to say by how much.
- **`chromium_applies_each_requested_viewport_to_layout` plays last in its group:** `browser.status.md` says it has only ever compiled, so a red there hides nothing.
- **The placeholder joins `browser_example`'s opening story,** across binaries, since that story already opens the example at its first page, on the same server.

## Belonging

What each test checks decides, never its cost (`skein/docs/foundation/testing-strategy.md`, 8.1). This is a reading of the code and the design, not a run.

- **`browser_harness`:** a child's arguments, environment, standard input, exit code and teardown, a link's deadline, and a peer's tokens and shutdown are the harness's and the kernel's. `See Ready` in those tests waited for a page and asserted nothing about it. A printed link reaching the server and a blank page reaching nothing are held by a fake that fetches what it's sent to. **A real browser stays in:** a module's exception through the real loader, a computed role, typing over words, a frame's refusal, an attribute read from a real document.
- **`browser_real` and `browser_facts`:** the kit's facts about Chromium (`skein/docs/design/browser.md`, 9), and the page part's folding and reading (`skein/docs/design/ui-web.md`, 14.3), are what only a browser shows. All stay in the tier.
- **`browser_stories` and `browser_example`:** the shell in Chromium. All stay in the tier.

## What the two tab tests wait for

**Reading points at one cause, and doesn't prove it.** Neither test waits out a deadline to prove something didn't happen. Each `Tabs` step ends on its first match, polled every 50 ms, bound by 2 s; each `Tabs` read in the rig is read once; `Keep` reads once.

The one thing they share, and that the tier's other tab test (`chromium_finds_and_presses_the_page_beside_its_loading_tab`) doesn't, is **a press on a page a tab has taken the front of.** A press holds no look (`skein/testing/skein-browser/src/browser.rs`: `Scroll` to `MouseUp`, no `Page.bringToFront`), and listing tabs and reading attributes don't bring a page back (`skein/docs/design/browser.md`, 6). Both tests find their buttons before the tab opens, then press those nodes:

| Test | Presses on a hidden page | Wall past a test's usual | Per press |
|---|---:|---:|---:|
| `chromium_lists_a_pressed_tab_...` | 1 (`Cut opener`) | about 4.7 s of 5.6 | about 4.7 s |
| `tab::a_tab_opens_only_from_a_press` | 5 (`Point`, `Open`, a closer, `Open`, a closer) | about 25 s of 26.8 | about 5 s |

The wall is the CPU's shape too: the waits are idle. The kit sets no 5 s anywhere (its `answer` is 10 s, its `poll` 50 ms), so **the wait is in Chromium's answer to something in the press, and which one can't be read here.** Nothing may be run to find out.

**The change doesn't need the cause to be right.** A person comes back to a page before pressing it, which is what a look is (`browser.md`, 6), and the `beside` test already holds that a fresh find and press land. Pressing a kept node on a hidden page is no part of what either test asserts, and `support/tab.rs`'s `TabStory`, a `Play` of its own for that purpose, goes away. If after-landing shows the wall unmoved, the fact to report is that this reading was wrong, and the next step needs a run.

**A product question it leaves:** whether a real person's press can land on a hidden page. It can't, so no product code changes.

## The table

Every test of the tier, by binary, in its source's order. *Lower* means a focused test: it exists and passes before the browser test goes, on the same branch, and its task's *bite* breaks the rule it holds. *Compiled* means a merged browser test, written and compiled, not run, whose assertions stay as its steps: none can fail first in this plan.

### `skein-browser-world::browser_real`

| # | Test | What it checks | Fate | Where its assertions stay |
|---:|---|---|---|---|
| 1 | `chromium_opens_finds_and_presses_a_button` | The kit's whole conversation: a version, a first page, a press, a screenshot, a clean exit | **Stays,** a fresh browser's start and exit | As they are |
| 2 | `a_render_between_mouse_down_and_up_keeps_the_click_only_on_the_held_element` | Chromium's click across a render, replaced and not; two starts | Merges into `chromium_acts_on_a_page` | Two cases. Compiled |
| 3 | `chromium_selects_all_replaces_and_submits_a_form` | Select-all, typing over, Enter submits | Merges into `chromium_acts_on_a_page` | A case. Compiled |
| 4 | `chromium_rejects_a_covered_button` | The hit test refuses a covered button | Merges into `chromium_acts_on_a_page` | A case. Compiled |
| 5 | `chromium_scrolls_to_a_far_button` | A press scrolls first | Merges into `chromium_acts_on_a_page` | A case. Compiled |
| 6 | `chromium_obeys_content_security_policy` | A blocked script, and its log | Merges into `chromium_reports_what_a_page_holds` | A case. Compiled |
| 7 | `chromium_reports_a_script_exception` | An exception reaches the kit | Merges into `chromium_reports_what_a_page_holds` | A case. Compiled |
| 8 | `chromium_finds_a_sentence_by_a_few_of_its_words` | A find by words | Merges into `chromium_reports_what_a_page_holds` | A case. Compiled |
| 9 | `chromium_presses_an_svg_shape_through_its_figure_at_a_point` | A press at a point of a figure | Merges into `chromium_acts_on_a_page` | A case. Compiled |
| 10 | `chromium_survives_a_page_that_reloads_itself` | A reload after a JavaScript or a Wasm exception; two starts | Merges into `chromium_keeps_a_page_through_reloads_and_dialogs` | Two cases, named by their source. Compiled |
| 11 | `chromium_accepts_the_closing_warning_on_a_reload` | A warning accepted; a close meeting none | Merges into `chromium_keeps_a_page_through_reloads_and_dialogs` | A case. Compiled |
| 12 | `chromium_dismisses_a_confirm_as_trouble` | A confirm dismissed, reported | Merges into `chromium_keeps_a_page_through_reloads_and_dialogs` | A case. Compiled |
| 13 | `chromium_refuses_a_scope_found_before_a_reload` | A scope from before a reload is `Gone` | Merges into `chromium_keeps_a_page_through_reloads_and_dialogs` | A case. Compiled |
| 14 | `chromium_reads_the_first_hidden_inputs_value_attribute` | An attribute of the first match | Merges into `chromium_reports_what_a_page_holds` | A case. Compiled |
| 15 | `chromium_reads_a_named_markup_attribute` | A named attribute | Merges into `chromium_reports_what_a_page_holds` | A case. Compiled |
| 16 | `chromium_reports_a_selector_without_an_element` | `NoElement` | Merges into `chromium_reports_what_a_page_holds` | A case, and row 82's `NoAttribute` a new one beside it. Compiled |
| 17 | `a_download_is_saved_in_the_persons_directory` | A file saved with the page's bytes | Merges into `chromium_saves_a_download_only_for_a_person_with_a_directory` | A case and its check. Compiled |
| 18 | `a_person_without_a_directory_downloads_nothing` | None saved; `Trouble::Download` | Merges into `chromium_saves_a_download_only_for_a_person_with_a_directory` | A case and its check. Compiled |
| 19 | `a_letter_with_alt_and_shift_reaches_the_page` | A letter's code and modifiers | Merges into `chromium_acts_on_a_page` | A case. Compiled |
| 20 | `chromium_lists_a_pressed_tab_and_its_cut_opener_but_blocks_an_unpressed_opening` | The pop-up rule, a tab's listing, its opener, an exit holding it | Merges into `chromium_holds_tabs_beside_a_page`, last, and its press looks first | A case; the exit's assertion on the whole test. Compiled |
| 21 | `chromium_finds_and_presses_the_page_beside_its_loading_tab` | A page usable beside its tab | Merges into `chromium_holds_tabs_beside_a_page`, first | A case. Compiled |
| 22 | `chromium_applies_each_requested_viewport_to_layout` | A viewport reaches layout | Merges into `chromium_acts_on_a_page`, last | A case. Compiled |
| 23 | `folds_away::removing_a_focused_form_returns_to_its_disclosures_summary` | Focus after a removal, in Chromium | Merges into `folds_away::page_parts_fold_away_and_a_log_keeps_following` | Person 0. Compiled |
| 24 | `folds_away::a_panel_folds_away_without_taking_the_outside_press_or_a_forms_escape` | A panel's folding | Merges into the same | Person 1. Compiled |
| 25 | `folds_away::a_press_outside_closes_a_menu_and_goes_on` | A menu closed by a press past it | Merges into the same | Person 2. Compiled |
| 26 | `log_end::a_log_scrolled_away_returns_to_its_end_and_keeps_following` | A log's return to its end | Merges into the same, across modules: same setup | Person 3. Compiled |
| 27 | `log_reading::a_short_log_reads_each_page_once_until_there_is_no_more` | Reading a short log | Merges into `log_reading::a_log_reads_on_and_holds_its_words` | A story. Compiled |
| 28 | `log_reading::the_view_stays_with_the_browsers_own_anchoring` | Words held with the browser's anchoring | Merges into the same | A story. Compiled |
| 29 | `log_reading::reading_without_browser_anchoring_follows_requests_and_page_keys` | Words held without it; an end request; page keys | **Stays,** the tier's longest story (*What this plan decided*) | As it is |
| 30 | `log_reading::scrolling_into_the_head_can_render_busy_during_the_scroll_callback` | A busy render in a scroll's turn | Merges into `log_reading::a_log_reads_on_and_holds_its_words` | A story. Compiled |
| 31 | `log_reading::a_status_at_the_head_does_not_anchor_instead_of_the_words` | A status isn't the anchor | Merges into the same | A story. Compiled |
| 32 | `log_reading::a_contents_wrapper_keeps_its_visible_words_in_place` | A wrapper with no box isn't either | Merges into the same | A story. Compiled |
| 33 | `log_reading::a_read_in_a_scroll_turn_reads_on_without_a_tick` | Reads on with no timer; its 1000 ms stays, and in turn no other story runs beside it | Merges into the same | A story. Compiled |

### `skein-browser-world::browser_facts`

| # | Test | What it checks | Fate | Where its assertions stay |
|---:|---|---|---|---|
| 34 | `chromium_computes_each_roles_accessible_role` | Each role's computed role, the face's table | Merges into `chromium_holds_the_faces_facts` | A case and its check. Compiled |
| 35 | `chromium_chooses_an_option_by_the_kits_keys` | An option chosen by keys | Merges into the same | A case and its check. Compiled |
| 36 | `chromium_reports_both_labels_when_type_ahead_chooses_a_shared_prefix` | Type-ahead's shared prefix, both labels | Merges into the same | A case and its check. Compiled |
| 37 | `chromium_presses_a_select_inside_its_shadow_tree` | A press inside a select's shadow tree | Merges into the same | A case. Compiled |
| 38 | `chromium_takes_a_date_and_a_time_by_the_kits_keys` | A date's and a time's keys, in their order | Merges into the same | A case and its check. Compiled |

### `skein-ui-web-tests::browser_stories`

| # | Test | What it checks | Fate | Where its assertions stay |
|---:|---|---|---|---|
| 39 | `a_clipboard_write_keeps_the_press_and_reports_once` | A write's gesture, three people | **Stays,** already one test of three stories | As it is |
| 40 | `a_panic_reloads_then_says_it_failed` | skein's story: starts its own run, which other trees call | **Stays** | As it is |
| 41 | `a_refused_write_makes_no_note` | skein's story, likewise | **Stays** | As it is |
| 42 | `typing_in_the_title_survives_a_note_arriving` | skein's story, likewise | **Stays** | As it is |
| 43 | `back_and_forward_move_between_the_list_and_a_note` | skein's story, likewise | **Stays** | As it is |
| 44 | `a_placeholder_gives_way_to_the_client` | The placeholder goes, the client comes | Merges into `browser_example`'s `a_person_goes_through_the_example_in_turn`, the first story | Its `Gone` a step there. Compiled |
| 45 | `a_module_not_found_says_so_and_offers_to_try_again` | The loader's failure, on a bundle of its own | **Stays,** its own setup | As it is |
| 46 | `live_tree::the_first_child_gets_a_group_in_the_drawn_tree` | A first child's group, in the live DOM | Merges into `live_tree::the_live_tree_keeps_its_structure_and_nodes` | A case in turn. Compiled |
| 47 | `live_tree::a_child_is_inserted_between_two_live_keyed_children` | An insertion | Merges into the same | A case in turn. Compiled |
| 48 | `live_tree::a_keyed_child_moves_within_its_live_group` | A move | Merges into the same | A case in turn. Compiled |
| 49 | `live_tree::a_childs_name_changes_on_the_same_live_node` | A rename keeps its node | Merges into the same | A case in turn. Compiled |
| 50 | `live_tree::removing_the_last_child_leaves_no_empty_live_group` | No empty group | Merges into the same | A case in turn. Compiled |
| 51 | `live_tree::a_refreshed_form_removes_its_field_and_companion_label` | A form's refresh | Merges into the same | A case in turn. Compiled |
| 52 | `live_tree::a_disclosure_keeps_focus_when_its_summary_name_changes` | Focus through a rename | Merges into the same | A case in turn. Compiled |
| 53 | `live_tree::a_paragraph_splits_two_live_runs_which_grow_and_merge_in_order` | Runs that split, grow and merge | Merges into the same | A case in turn. Compiled |
| 54 | `live_tree::clicking_the_visible_field_label_focuses_its_named_input` | A label focuses its input, on the example's own page | Merges into the same | A second person. Compiled |
| 55 | `tab::a_tab_opens_only_from_a_press` | The pop-up rule through the shell's press path | **Stays,** its own context's tabs, and its presses look first (*What the two tab tests wait for*) | Each step, on a `Script`. Compiled |

### `skein-ui-web-tests::browser_example`

| # | Test | What it checks | Fate | Where its assertions stay |
|---:|---|---|---|---|
| 56 | `a_person_opens_the_example_and_sees_its_heading` | The first page, under the strict policy | Merges into `a_person_goes_through_the_example_in_turn`, first: an empty store | A story. Compiled |
| 57 | `a_reload_keeps_a_pending_add_made_once` | A pending add kept, made once | Merges into `a_reload_meets_the_closing_warning_and_keeps_a_pending_add_made_once` | Its steps, as they are. Compiled |
| 58 | `a_note_added_in_one_page_shows_in_another` | Two people on one server | **Stays,** two people, one story | As it is |
| 59 | `a_person_opens_a_note_and_comes_back_by_its_link` | A note's address and title, and back | Merges into `a_person_goes_through_the_example_in_turn` | A story; its address `/notes/3`, after two notes. Compiled |
| 60 | `a_reload_with_an_add_pending_meets_the_closing_warning` | The warning, while an add is pending | Merges into `a_reload_meets_...`: row 57's steps to its first reload are this test's | `Warned(true)` after that reload. Compiled |
| 61 | `a_reload_once_the_add_is_made_meets_no_closing_warning` | No warning once it's made | Merges into `a_reload_meets_...` | `Warned(false)` after the second reload, the add seen before it. Compiled |
| 62 | `a_person_fills_each_entry_kind_and_chooses_a_label` | Each entry kind and a label | Merges into `a_person_goes_through_the_example_in_turn` | A story. Compiled |
| 63 | `a_person_chooses_a_label_by_its_option` | A label by its option | Merges into the same | A story. Compiled |
| 64 | `escape_in_the_dialog_keeps_the_draft` | Escape, focus back, the draft kept | Merges into the same | A story; its note retitled, so that the find can't pass on row 59's. Compiled |
| 65 | `a_press_on_a_bar_shows_its_labels_notes` | A bar's mark pressed; the list filtered | Merges into the same, second: a store with its two notes alone | A story. Compiled |
| 66 | `the_charts_fallback_is_read_by_its_name` | The chart's name and fallback, on an empty store | Merges into the same | Steps of the first story. Compiled |
| 67 | `the_body_grows_with_its_words_and_reports_them` | The component grows and reports | Merges into the same | A story. Compiled |

### `skein-ui-web-tests::browser_harness`

*Goes to* names a test of `skein-ui-web-tests::suite`, its module `harness`.

| # | Test | What it checks | Fate | Where its assertions stay |
|---:|---|---|---|---|
| 68 | `the_harness_fails_a_page_whose_module_throws` | A module's exception, through the real loader in a real browser, fails the run | **Stays:** a failed run needs its own | As it is |
| 69 | `the_harness_reads_the_listening_line` | A bundled module draws under the strict policy | Merges into `people_read_a_bundled_page_and_pages_of_another_origin` | Person 0. Compiled. The line's own reading is `a_printed_line_names_the_url_to_open_and_its_address`, a step test that exists |
| 70 | `the_harness_searches_the_next_accessible_role` | Chromium's `spinbutton` found by the role's next name | Merges into the same | Person 0. Compiled |
| 71 | `a_page_of_another_origin_is_opened_by_its_step` | An origin's page, read in a browser | Merges into the same | Person 1. Compiled |
| 72 | `a_framed_page_s_refusal_is_awaited_as_trouble` | Chromium's `frame-ancestors` refusal | Merges into the same | Person 1. Compiled |
| 73 | `a_request_with_another_host_has_its_status_awaited` | The harness's own exchange | Goes to `people_run_commands_beside_their_pages` | Person 1. Lower. `a_request_step_with_another_status_fails_naming_it` already holds its failure |
| 74 | `a_lent_peer_listens_before_the_server_requests_it` | A peer's readiness before the server | Goes to `lent_peers_listen_first_and_return_empty_and_usable` | The exact request. Lower |
| 75 | `a_lent_peer_returns_empty_with_its_handle_usable` | A peer back, emptied, its handle usable | Goes to the same | Lower |
| 76 | `a_lent_peers_assertion_names_it_in_the_failure` | A peer's panic named | Goes to `a_lent_peers_assertion_names_it_in_the_failure` | Moved. Lower |
| 77 | `colliding_peer_tokens_reach_their_own_hosts` | Two peers' same tokens kept apart | Goes to `lent_peers_listen_first_and_return_empty_and_usable` | Two peers. Lower |
| 78 | `a_peer_shutdown_assertion_preserves_a_failed_outcome` | A shutdown's panic named | Goes to `a_peer_shutdown_assertion_preserves_a_failed_outcome` | Moved. Lower |
| 79 | `repeated_peer_arguments_each_reach_the_server` | A placeholder repeated | Goes to `lent_peers_listen_first_and_return_empty_and_usable` | `{first}` twice. Lower |
| 80 | `keeps_read_values_named_and_empty_attributes_in_one_run` | Real attributes, the empty one too | Merges into `people_read_a_bundled_page_and_pages_of_another_origin` | Person 2, every row. Compiled |
| 81 | `a_keep_without_an_element_fails_its_step` | The failure's words and step | Goes to `a_failed_run_retains_completed_keeps`, and a row of `a_failed_keep_reports_its_own_step` | Lower. Chromium's `NoElement` is row 16 |
| 82 | `a_keep_without_an_attribute_fails_its_step` | The failure's words | Goes to a row of `a_failed_keep_reports_its_own_step` | Lower. Chromium's `NoAttribute` is a new case beside row 16 |
| 83 | `keeps_append_duplicate_names_in_completion_order_with_their_person` | The order and the person kept | Goes to `keeps_append_in_completion_order_and_a_new_run_clears_them` | Lower; bites by keeps not clearing |
| 84 | `a_new_run_clears_the_preceding_keeps` | A new run's `kept()`; two starts | Goes to the same | Lower; bites by keeps not clearing |
| 85 | `a_failed_run_retains_completed_keeps` | What was kept outlives a failure | Goes to `a_failed_run_retains_completed_keeps` | Moved, one step fewer. Lower |
| 86 | `a_command_receives_the_held_line_and_eof_without_a_page_visit` | A held line on stdin; a blank page reaches nothing | Goes to `a_held_line_reaches_a_command_and_closed_stdin_only_eof` | The fixture's own assertions. Lower |
| 87 | `closed_command_stdin_receives_only_eof` | Closed stdin | Goes to the same | Lower; bites by no finish |
| 88 | `an_exited_commands_printed_link_is_opened_and_kept_whole` | A link opened, kept whole, after its exit | Goes to `people_run_commands_beside_their_pages` | Person 0. Lower |
| 89 | `a_command_without_the_link_fails_naming_the_command` | The failure's words, stderr kept | Goes to `a_command_without_the_link_fails_naming_the_command` | Moved. Lower |
| 90 | `exited_accepts_the_commands_expected_code` | An expected code | Goes to `people_run_commands_beside_their_pages` | Person 1. Lower |
| 91 | `exited_rejects_a_different_code_naming_the_command` | The failure's words | Goes to `exited_rejects_a_different_code_naming_the_command` | Moved. Lower |
| 92 | `a_live_command_at_story_end_fails_and_settles` | A failure, and a teardown that settles | Goes to `a_live_command_at_story_end_fails_and_settles` | Moved. Lower |
| 93 | `a_person_runs_two_commands_in_turn` | Commands in turn | Goes to `people_run_commands_beside_their_pages` | Person 1. Lower |
| 94 | `a_second_run_while_the_command_runs_fails` | The failure's words | Goes to `a_second_run_while_the_command_runs_fails` | Moved. Lower |
| 95 | `command_arguments_receive_peer_and_server_addresses` | Placeholders in arguments, reached by the child | Goes to `people_run_commands_beside_their_pages` | Person 2. Lower; bites by no substitution |
| 96 | `command_environment_values_receive_peer_and_server_addresses` | Placeholders in the environment; no other's inherited | Goes to the same | Person 2. Lower |
| 97 | `a_link_keep_and_a_selector_keep_append_to_one_sequence` | One sequence, in order | Goes to the same | Person 0. Lower |
| 98 | `two_peoples_commands_have_independent_status_and_keeps` | Each person's own | Goes to the same | By `person`. Lower |
| 99 | `a_command_waits_for_the_browsers_loopback_signal_before_exiting` | A link fetched while its command lives | Goes to the same | Person 3, the fake's fetch. Lower; bites by no fetch |
| 100 | `a_missing_link_deadline_fails_and_command_teardown_settles` | Its 50 ms, and the teardown | Goes to `a_missing_link_deadline_fails_and_command_teardown_settles` | Moved. Lower |
| 101 | `an_exited_deadline_fails_and_command_teardown_settles` | Its 50 ms, and the teardown | Goes to `an_exited_deadline_fails_and_command_teardown_settles` | Moved. Lower |
| 102 | `a_person_types_over_words_then_clears_the_field` | Typing over, then clearing, in a real field | Merges into `people_read_a_bundled_page_and_pages_of_another_origin` | Person 0. Compiled |
