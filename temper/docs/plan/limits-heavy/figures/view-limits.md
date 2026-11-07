# view-limits: figures

| Test | Before (Ir) | After (Ir) | Cause | What changed |
|---|---:|---:|---|---|
| `temper-client-view::lib::tests::limits::a_window_full_of_small_lines_builds` | 23,174,371,947 | 493,362,367 | Page ordering: 22,711,430,720 Ir, 98.00%; pairwise comparisons and whole decoded-line clones | Unique positions use a bounded ordered map and move their lines; ordering now costs 30,407,393 Ir |
| `temper-client-view::lib::tests::limits::kept_words_at_the_threads_bytes_build` | 12,148,031,421 | 12,148,078,608 | Drawing and freeing the full newline tree; `draw_words` is 11,286,705,953 Ir before and 11,286,705,921 after | stays: both shells draw the same pending-byte bounds; this test does not order a page |
| `temper-client-view::lib::tests::limits::a_window_full_of_long_words_builds` | 9,458,177,324 | 9,452,682,538 | Whole-word drawing and tree destruction; `draw_words` is 9,408,818,670 Ir before and 9,408,804,913 after | Page ordering falls from 5,607,175 to 75,933 Ir; both byte patterns and shells stay |
| `temper-client-view::lib::tests::limits::a_window_full_of_markdown_builds` | 6,288,570,002 | 5,592,171,334 | Markdown reading/drawing: 4,679,171,085 Ir; page ordering: 703,019,960 Ir | Move ordered lines; ordering is 6,806,931 Ir after; all three Markdown patterns stay |
| `temper-client-view::lib::tests::limits::a_window_full_of_calls_builds` | 6,084,229,642 | 963,739,299 | Page ordering: 5,130,307,131 Ir, 84.32%; repeatedly cloning the calls and their inputs | Move ordered lines; ordering is 9,763,891 Ir after; the same calls and window remain |
| `temper-client-view::lib::tests::limits::a_request_and_a_composer_at_long_text_build` | 1,479,366,130 | 1,479,413,108 | Drawing the request and composer; `draw_words` is 1,390,929,422 Ir before and after | stays: both long-text bounds and shells; this test does not order a page |
| `jig-client::lib::thread::order::tests::ordering_keeps_whole_lines_and_their_room` | unknown | unknown | Not profiled: new focused regression | Checks descending order, unchanged source allocations, every line and retained capacity |
| `jig-client::lib::thread::order::tests::repeated_positions_keep_every_line_in_pairwise_order` | unknown | unknown | Not profiled: new focused regression | Checks that repeated positions keep every line and the same pairwise order |

`lib` denotes the Cargo `--lib` target. The six limit tests total
58,632,746,466 Ir before and 30,129,447,254 after, a 48.6% reduction.
Their source, assertions, fixture bytes and bounds are unchanged.

## Measurement

Before is the task base `0548af5889`; after is this task's implementation.
Every limit test ran alone through nextest and callgrind on the same x86-64
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

## Found and not fixed

- Drawing at the byte bounds is linear in the required output. Pending words
  draw 65,536 and 1,048,576 newline bytes, in 1,024 and 8 chunks respectively,
  through both shells: 2,230,288 body paragraphs, including the final empty
  paragraph of each chunk. Their conversation costs 8,181,360,880 Ir after;
  freeing their trees costs 1,945,104,756 Ir. Long words retain seven
  131,072-byte lines for each of the two patterns, through both shells:
  conversation costs 6,832,484,982 Ir and tree destruction 1,591,132,006 Ir.
  The request and composer retain 131,072 bytes each through both shells;
  `draw_words` stays at 1,390,929,422 Ir. Reducing these counts by drawing
  fewer paragraphs, bytes or shells would weaken the bound tests.
- Markdown's full three-pattern window still costs 4,678,930,342 Ir in
  `markdown::read` after. The calls' conversation costs 851,753,514 Ir,
  and the small lines' conversation 285,657,448 Ir. These are the retained
  drawing work, including the markup and call structures the tests price.
- Pending admission costs 598,135,002 Ir in `Thread::sending`, including
  430,964,823 Ir rebuilding retained rows in `prune_read`. This is repeated
  work, but even removing the entire sending path would save only
  `598135002 / 12148031421 * 2.5 = 0.123` CPU s using the request's scale.
  A further repair would keep admission's retained rows/byte accounting
  without rebuilding them per chunk; no admission behavior is bypassed here.
- Input construction, repeated weights and phrasebook work are below the
  stopping threshold. Across the six baseline profiles, `repeated` costs
  214,732,295 Ir, `DecodedActivityLine::weight` 9,790,223 Ir,
  `Words::english` 495,559,774 Ir and `Words::capacity` 269,864,610 Ir.
  Conservative gain ceilings are respectively 0.066, 0.002, 0.198 and
  0.096 CPU s: use the request's 2.7/2.5/1.9 CPU s for its named tests,
  then assign all remaining 2.0 CPU s to the largest relevant inclusive
  share among the other three. These are separate ceilings, not additive
  savings. Making byte runs in bulk, reusing word lines or phrasebooks,
  or avoiding repeated capacity/weight queries remains possible; none is
  the next cause worth about 0.2 CPU s. Shared fixture construction stays
  with its existing owner.
- Repeated-position pages retain the pairwise ordering path, to preserve
  its order among equal positions. It has zero instructions in these six
  profiles. A broader replacement would need to preserve that tie order;
  the focused regression records it. Unique positions use O(n log n)
  bounded-map work without copying payloads.

## Tests that went

None.
