# view-limits: figures

| Test | Before (Ir) | After (Ir) | Cause | What changed |
|---|---:|---:|---|---|
| `temper-client-view::lib::tests::limits::a_window_full_of_small_lines_builds` | 23,174,371,947 | 493,358,090 | Page ordering: 22,711,430,720 Ir, 98.00%; pairwise comparisons and whole decoded-line clones | Unique positions use a bounded ordered map and move their lines; ordering now costs 30,407,393 Ir |
| `temper-client-view::lib::tests::limits::kept_words_at_the_threads_bytes_build` | 12,148,031,421 | 11,991,007,201 | Drawing and freeing the full newline tree; byte-at-a-time input construction costs another 158,888,502 Ir | Fill byte runs in bulk: input construction is 1,821,468 Ir; `draw_words` stays at 11,286,705,917 Ir |
| `temper-client-view::lib::tests::limits::a_window_full_of_long_words_builds` | 9,458,177,324 | 9,415,975,227 | Whole-word drawing and tree destruction; byte-at-a-time input construction costs 37,231,370 Ir | Move ordered lines and fill byte runs in bulk: input construction is 531,284 Ir; both byte patterns and shells stay |
| `temper-client-view::lib::tests::limits::a_window_full_of_markdown_builds` | 6,288,570,002 | 5,592,630,919 | Markdown reading/drawing: 4,679,171,085 Ir; page ordering: 703,019,960 Ir | Move ordered lines; ordering is 6,806,931 Ir after; all three Markdown patterns stay |
| `temper-client-view::lib::tests::limits::a_window_full_of_calls_builds` | 6,084,229,642 | 963,736,342 | Page ordering: 5,130,307,131 Ir, 84.32%; repeatedly cloning the calls and their inputs | Move ordered lines; ordering is 9,763,891 Ir after; the same calls and window remain |
| `temper-client-view::lib::tests::limits::a_request_and_a_composer_at_long_text_build` | 1,479,366,130 | 1,461,058,898 | Drawing the request and composer; byte-at-a-time input construction costs 18,612,583 Ir | Fill byte runs in bulk: input construction is 262,540 Ir; `draw_words` stays at 1,390,929,414 Ir |
| `jig-client::lib::thread::order::tests::ordering_keeps_whole_lines_and_their_room` | unknown | unknown | Not profiled: new focused regression | Checks descending order, unchanged source allocations, every line and retained capacity |
| `jig-client::lib::thread::order::tests::repeated_positions_keep_every_line_in_pairwise_order` | unknown | unknown | Not profiled: new focused regression | Checks that repeated positions keep every line and the same pairwise order |

`lib` denotes the Cargo `--lib` target. The six limit tests total
58,632,746,466 Ir before and 29,917,766,677 after, a 49.0% reduction.
All six tests, assertions, fixture bytes and bounds remain. The test helper
allocates a byte run once and fills its slice instead of calling `Writer::put`
for each byte; its signature and chat/task readers remain unchanged.

## Measurement

Before is the task base `0548af5889`, retained from the ordering fix's figures
in `6108204664`; after includes the subsequent bulk byte-run construction.
Each measurement pair ran alone through nextest and callgrind on one x86-64
builder, with the same unoptimized test build and this command on both sides:

```sh
NEXTEST_DOUBLE_SPAWN=0 \
CARGO_TARGET_X86_64_UNKNOWN_LINUX_GNU_RUNNER="valgrind --tool=callgrind --callgrind-out-file=$PWD/target/limits-heavy/NAME.%p.cg" \
heavy cargo nextest run -p temper-client-view --lib -E 'test(=tests::limits::NAME)'
```

For `kept_words_at_the_threads_bytes_build` and
`a_window_full_of_long_words_builds`, both sides additionally used
`--profile fuzzy --ignore-default-filter` for the longer existing profiling
patience. Each invocation selected and passed one test. Counts are the test
process's whole `Ir` totals, excluding nextest's test-listing processes. The
tests spawn no child processes. Small changes in unchanged tests' whole
counts include harness/allocation variation; their drawing counts above
show the unchanged work. No build, budget or timeout setting changed.

Before the byte-run change, all six were profiled again at `6108204664`,
in the table's order: 493,317,352; 12,148,033,377; 9,452,634,432;
5,592,747,295; 963,697,716; 1,479,367,997 Ir. A matching six-test pair
used the default profile on both sides, on the same builder and unoptimized
build: 30,129,798,169 Ir before and 29,917,766,592 after. The bulk fill
therefore removes about 212 million instructions beyond the ordering fix.
The two longer final measurements in the main table additionally repeat
the original fuzzy-profile commands above. The changed helper retains
each byte: the three affected tests' drawing counts remain unchanged
apart from a few harness/allocation instructions.

## Found and not fixed

- Drawing at the byte bounds is linear in the required output. Pending words
  draw 65,536 and 1,048,576 newline bytes, in 1,024 and 8 chunks respectively,
  through both shells: 2,230,288 body paragraphs, including the final empty
  paragraph of each chunk. Their conversation costs 8,181,360,880 Ir after;
  freeing their trees costs 1,945,104,756 Ir. Long words retain seven
  131,072-byte lines for each of the two patterns, through both shells:
  conversation costs 6,832,484,982 Ir and tree destruction 1,591,132,006 Ir.
  The request and composer retain 131,072 bytes each through both shells;
  `draw_words` stays at 1,390,929,414 Ir. Reducing these counts by drawing
  fewer paragraphs, bytes or shells would weaken the bound tests.
- Markdown's full three-pattern window still costs 4,679,394,088 Ir in
  `markdown::read` after. The calls' conversation costs 851,754,746 Ir,
  and the small lines' conversation 285,657,448 Ir. These are the retained
  drawing work, including the markup and call structures the tests price.
- Pending admission costs 598,135,002 Ir in `Thread::sending`, including
  430,964,823 Ir rebuilding retained rows in `prune_read`. This is repeated
  work, but even removing the entire sending path would save only
  `598135002 / 12148031421 * 2.5 = 0.123` CPU s using the request's scale.
  A further repair would keep admission's retained rows/byte accounting
  without rebuilding them per chunk; no admission behavior is bypassed here.
- Remaining input construction, repeated weights and phrasebook work are
  below the stopping threshold. Bulk `repeated` costs 2,615,292 Ir across
  its three tests. Across the six original baseline profiles,
  `DecodedActivityLine::weight` costs 9,790,223 Ir,
  `Words::english` 495,559,774 Ir and `Words::capacity` 269,864,610 Ir.
  Conservative gain ceilings for weight, English construction and capacity
  are respectively 0.002, 0.198 and 0.096 CPU s: use the request's
  2.7/2.5/1.9 CPU s for its named tests,
  then assign all remaining 2.0 CPU s to the largest relevant inclusive
  share among the other three. These are separate ceilings, not additive
  savings. Reusing word lines or phrasebooks, or avoiding repeated
  capacity/weight queries remain possible; none is the next cause worth
  about 0.2 CPU s. Shared fixture construction stays
  with its existing owner.
- Repeated-position pages retain the pairwise ordering path, to preserve
  its order among equal positions. It has zero instructions in these six
  profiles. A broader replacement would need to preserve that tie order;
  the focused regression records it. Unique positions use O(n log n)
  bounded-map work without copying payloads.

## Tests that went

None.
