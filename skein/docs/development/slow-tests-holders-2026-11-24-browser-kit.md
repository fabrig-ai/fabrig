# skein's browser kit: each slow test moved, beside its holders

2026-11-24. For `temper/docs/design/draft/slow-tests-down.md`, 5.1 and
3.2, from `temper/docs/development/slow-tests-survey-2026-11-23-skein-hearth.md`,
groups 1 and 3, checked again at the goal's commit (`28491cf110`).
The design is `skein/docs/design/ui/web/testing.md`, 1 to 3. A slow
test leaves Chromium only where the tests named beside it hold its
every assertion; each change that removes or merges one writes its row
here, in its family's section, naming the lower tests by binary and
name.

## 1. What stays in Chromium

- `skein-ui-web-tests::browser_harness`: `the_harness_fails_a_page_whose_module_throws`,
  `the_harness_reads_the_listening_line`, `an_unmet_see_reports_its_deadline_and_returns_promptly`,
  `the_harness_searches_the_next_accessible_role`, `a_page_of_another_origin_is_opened_by_its_step`,
  `a_framed_page_s_refusal_is_awaited_as_trouble`,
  `a_command_receives_the_held_line_and_eof_without_a_page_visit`,
  `an_exited_commands_printed_link_is_opened_and_kept_whole`,
  `a_command_waits_for_the_browsers_loopback_signal_before_exiting`,
  `a_person_types_over_words_then_clears_the_field`.
- `skein-ui-web-tests::browser_stories`: every story but the refused
  write; the live tree as one walk of its cases, and its two focus
  stories.
- `skein-ui-web-tests::browser_example`: every story but the note
  crossing pages, which the typing story witnesses.
- `skein-browser-world::browser_real`: every test but the sentence
  found by its words and the selector without an element, whose native
  facts ride the tests named in section 4.

## 2. The harness's bookkeeping, to `skein-world-tests::simulated`

### 2.1 Keeps and the harness's own request

| Slow test (`browser_harness`) | Held by |
|---|---|

### 2.2 Commands and their deadlines

| Slow test (`browser_harness`) | Held by |
|---|---|

### 2.3 Lent peers and substituted addresses

| Slow test (`browser_harness`) | Held by |
|---|---|

## 3. The example's patches, refused add and fan-out, to the host

### 3.1 The live tree

In each row the host test checks the operations and each revision's
HTML, including retained `data-node` ids. The walk checks the live DOM
and Chromium's accessibility tree in the same revisions.

| Slow test (`browser_stories::live_tree`) | Host (`skein-ui-web-tests::machine::tree`) | Live (`browser_stories::live_tree`) |
|---|---|---|
| `the_first_child_gets_a_group_in_the_drawn_tree` | `the_first_child_creates_a_tree_group` | `the_live_tree_applies_each_patch_in_place`: `first`, group creation and backend node identity. |
| `a_child_is_inserted_between_two_live_keyed_children` | `a_child_is_inserted_before_its_keyed_sibling` | `the_live_tree_applies_each_patch_in_place`: `middle`, sibling order and backend node identity. |
| `a_keyed_child_moves_within_its_live_group` | `moving_keyed_children_keeps_their_node_ids` | `the_live_tree_applies_each_patch_in_place`: `move`, sibling order and backend node identity. |
| `a_childs_name_changes_on_the_same_live_node` | `renaming_a_keyed_child_keeps_its_node_id` | `the_live_tree_applies_each_patch_in_place`: `rename`, same backend node found by its new name, old name gone. |
| `removing_the_last_child_leaves_no_empty_live_group` | `removing_the_last_child_removes_its_group` | `the_live_tree_applies_each_patch_in_place`: `last`, no empty group, child and grandchild gone, stable branch kept. |
| `a_refreshed_form_removes_its_field_and_companion_label` | `removing_the_refreshed_form_removes_its_label` | `the_live_tree_applies_each_patch_in_place`: `form`, renamed form with its label and input, then no form, label or input and no accessible Name field. |
| `a_paragraph_splits_two_live_runs_which_grow_and_merge_in_order` | `a_paragraph_splits_then_rejoins_keyed_runs` | `the_live_tree_applies_each_patch_in_place`: `mixed`, paragraph and run placement, child order and backend node identity. |
| `shown_folded_children_patch_in_the_live_summary` | `the_folded_group_patches_and_moves_through_the_summary` | `the_live_tree_applies_each_patch_in_place`: `shown-folded`, summary order, group placement, meter attributes, caption and Extra, and the disclosure's accessible name. |

### 3.2 The refused add

| Slow test (`browser_stories`) | Held by |
|---|---|
| `a_refused_write_makes_no_note` | `skein-ui-example-view` lib: `tests::only_the_small_store_test_address_sets_a_bound`; `skein-ui-web-tests::machine`: `refused_add::a_refused_add_makes_no_note` and `refused_add::an_unbounded_add_is_sent_and_listed_once`; `skein-ui-web` lib: `tests::transport::a_refused_write_fails_every_exchange_of_its_step`, `tests::transport::a_refused_write_sends_no_exchange` and `tests::transport::writes_go_before_exchanges`. |

### 3.3 A note crossing pages

| Slow test (`browser_example`) | Held by |
|---|---|
| `a_note_added_in_one_page_shows_in_another` | `skein-ui-example-server::bin/skein-ui-example` `fan_out::one_add_and_its_retry_reach_both_readers_once` (the server's two streams and the link in each view); `skein-ui-example-view` `tests::stream::a_stream_note_crosses_browser_pieces` and `tests::stream::live_from_the_lists_answer_until_the_stream_ends` (piece boundaries and Live); `skein-ui-web-tests::browser_stories` `typing_in_the_title_survives_a_note_arriving` (the Chromium witness). |

## 4. The kit's sentence and selector facts

| Slow test (`browser_real`) | Held by |
|---|---|
| `chromium_finds_a_sentence_by_a_few_of_its_words` | `skein-browser-world::suite` / `words::a_find_by_words_keeps_only_the_matching_sentence` holds the kit's filtering and whole returned name; `skein-browser-world` (lib) / `tests::a_find_by_words_asks_without_a_name` holds the command shape. `skein-browser-world::browser_real` / `chromium_opens_finds_and_presses_a_button` holds Chromium's `StaticText` reply on the button page. |
| `chromium_reports_a_selector_without_an_element` | `skein-browser-world` (lib) / `tests::attribute::a_selector_without_an_element_returns_no_element` holds `nodeId: 0` becoming `Read::NoElement`. `skein-browser-world::browser_real` / `chromium_reads_the_first_hidden_inputs_value_attribute` holds the native `.absent` read on the same page as the hidden inputs. |
