# browser-reading: the reading story costs less, checking the same

`browser-temper` is over its budget: 85 CPU s, and today's median reads 91. One story of its 12 takes almost a quarter of it: `browser_pages::a_chat_opens_at_its_end_and_reads_back_keeping_its_place` (`temper/tests/client-browser/tests/browser_all/browser_pages.rs`). Its median is 18 to 24 CPU s a run over the last two days.

A measurement on the after-landing machine split one run of about 20 CPU s:

| Part | CPU s |
|---|---:|
| Chromium's renderer | 7.4 to 7.8 |
| temper's server and ctl | 3.2 to 4.0 |
| the test thread | 2.7 |
| Chromium's browser, GPU and utility processes | about 3.1 |
| unattributed | 2.5 to 2.9 |

There were 839 CDP commands, 130 of them accessibility-tree queries, returning 5.65 MB. The fixture is the cost: `temper/tests/client-browser/src/reading.rs` writes 8 long replies of 9,600-word code lines and 40 paragraphs each, plus 3 position replies, so that an evicting window has room at both of its read margins.

testing-strategy 8.1 says the story belongs where it is. This plan answers the second question: can it be made faster?

## What's wanted

1. **The smallest history that still checks the same.** The story checks reading back from a chat's end, eviction, and keeping its place (`temper/docs/design/client/browser.md`, 4.3 and 4.6, item 14). The tier's bundle already holds an eighth of a person's reading window "with the same mechanism" (`READING_WINDOW_BYTES`).
   - Make the tier's window, and with it the replies' sizes and counts, as small as the mechanism allows while every step and assertion keeps its meaning: the story still evicts, still reads back past each margin, and still keeps its place.
   - Each constant gets a comment saying why it's that size.
   - Any other story that uses `reading.rs` keeps passing and keeps checking what it checks.
   - 4.3's words on the tier's window change with it, if they name the eighth.
2. **CDP replies parsed once.** `skein/testing/skein-browser/src/wire/decode.rs`: `Value::get` (about line 70) reparses subtrees that `Document::parse` has already validated (about line 84), so each field lookup walks them again. Index the bounded value and member spans once and look them up from that index. Its unit tests cover the lookups.
3. **Nothing weaker.** No assertion is removed or loosened. No wait is lengthened and no poll interval changed.

## Limits

- The tier, its budget and the story's place don't change.
- The tier is over its budget, so this repair may run the stories it changes, by name and alone, to see them pass. It reads and compares no CPU figure.
- `walk = false`. Effort medium. Two tasks: the reading fixture, and the decoder.

## Sources

- `temper/tests/client-browser/src/reading.rs`
- `temper/tests/client-browser/tests/browser_all/browser_pages.rs`
- `skein/testing/skein-browser/src/wire/decode.rs`
- `skein/docs/foundation/testing-strategy.md`

Read by section: `temper/docs/design/client/browser.md`, 4.3 and 4.6.

## For every task of this plan

- **Only focused and fuzzy run in a plan, and a plan is complete when it lands.** The exception is the stories named above, each by name and alone. Never `tools/tiers/run`, nor a process or live tier. No walk, render check or throwaway driver.
- **Decide, don't ask.** A small edit just outside a task's file list is made, and named in the report. A `Question:` is only for what truly blocks the task.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
