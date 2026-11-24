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

| Slow test (`browser_stories`) | Held by |
|---|---|

### 3.2 The refused add

| Slow test (`browser_stories`) | Held by |
|---|---|

### 3.3 A note crossing pages

| Slow test (`browser_example`) | Held by |
|---|---|

## 4. The kit's sentence and selector facts

| Slow test (`browser_real`) | Held by |
|---|---|
| `chromium_finds_a_sentence_by_a_few_of_its_words` | `skein-browser-world::suite` / `words::a_find_by_words_keeps_only_the_matching_sentence` holds the kit's filtering and whole returned name; `skein-browser-world` (lib) / `tests::a_find_by_words_asks_without_a_name` holds the command shape. `skein-browser-world::browser_real` / `chromium_opens_finds_and_presses_a_button` holds Chromium's `StaticText` reply on the button page. |
| `chromium_reports_a_selector_without_an_element` | `skein-browser-world` (lib) / `tests::attribute::a_selector_without_an_element_returns_no_element` holds `nodeId: 0` becoming `Read::NoElement`. `skein-browser-world::browser_real` / `chromium_reads_the_first_hidden_inputs_value_attribute` holds the native `.absent` read on the same page as the hidden inputs. |
