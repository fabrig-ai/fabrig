# skein-limits-heavy: skein's costliest focused tests, by 8.1's questions

The request: `inputs/request.md`. The repair of `skein/docs/foundation/testing-strategy.md`, 8.1, in its order, for the costliest work above each process's own cost, aiming for at least 6 CPU s of a focused suite that reads 38.2 against 30. No budget, timeout or build setting changes. `walk = false`.

## Parts and tasks

| Part | Task | Needs | Rows | CPU s |
|---|---|---|---|---:|
| `github` | `tasks/github-runs.md` | | `github-conformance` `tests` | 4.59 |
| `lib-memory` | `tasks/lib-counts.md` | | `skein-lib-tests::suite` `memory` | 1.56 |
| `sweeps` | `tasks/sweeps.md` | | `skein-ui` `tests::date`, `tests::phrases`; `skein-json` `tests::text`; `skein-http` `tests::gzip`: the four named tests | 0.41 |
| `forge-machine` | `tasks/machine-costs.md` | | `skein-forge-machine-world::suite` `hand`, `machine` | 2.47 |
| `memory-worlds` | `tasks/memory-worlds.md` | | `skein-kv-tests::suite`, `skein-tls-world::suite`, `skein-llm-world::suite`, `skein-io-world::suite` | 3.43 |
| `scenario-worlds` | `tasks/scenario-worlds.md` | | `skein-conformance-real::suite`, `skein-world-tests::suite`, `skein-http-world::suite`, `skein-conformance-sim::suite`, `skein-fail-stop-tests::suite`, `skein-protocol-world::suite` | 2.94 |
| `close` | `tasks/limits-close.md` | all of the above | This file's table and `skein/docs/development/skein-limits-heavy.md` | |

Seven tasks, every row of the request's table and its four named sweeps: six that change tests, against the request's about six, and a close that writes the account and changes no code.


**Critical path:** any of the six, then `limits-close`. None of the six needs another. By the estimates the longest is `scenario-worlds` at about 140 minutes, then 40 for the close: about 180 in all. Those are estimates: the profiles decide how far each goes.

**What it should come to.** The aim is at least 6 CPU s. This is a reading of the code and the request's table, not a measurement, and the profiles replace it: about 2 from `github-runs`'s seven whole runs fewer, and more from whatever a run's profile finds; about 1.3 from `lib-counts`; about 0.4 from `sweeps`; and about 2 between the other three, whose causes are read and not yet profiled. That's close to the aim and not past it, so it rests on the profiles. The close gives the figures.

## What this plan decided

- **Each task writes its own figures file** (`figures/<task>.md`, in `notes/common.md`'s form), so that no two branches meet in a table. A last task builds the per-row table from them.
- **The stop rule is in CPU s, the measure in instructions.** A cause's share of a test's inclusive instructions, times that test's CPU s in the request's table, gives its gain, against the request's 0.05.
- **`github-conformance` carries several faults in one whole run.** The request offers three shapes, and each has its place:
  - **One run with several overrides** shows "fails only the fact and continues" once: `github_conformance_faults_fail_only_their_facts_and_the_run_continues`.
  - **A shared helper's group alone** holds the rows that read one fact's outcome: framing runs `cases::logs`, epoch a new `cases::issues_since_epoch`.
  - **A run stopped early** isn't used. The faulted facts sit late in a run (`logs` is fifth of six groups), so stopping saves little of what comes before them, and it would need a sentinel through `complete`.
- **A loop that waits for quiet ends at sixteen quiet rounds, for a reason read from `Wire`.** With bytes pending and no demand, `Wire::answer` delivers only on `chance(700)`, so nothing moving doesn't prove nothing is left, three times in ten. Sixteen in a row leaves about four in a billion. `Wire` is `skein-world-costs'`, so its private `pending` stays private.
- **A count falls only with a reason a test's assertions show, and each smaller one bites** (`notes/common.md`). Where a count is decided here, its spec has the arithmetic. Where a seed count backs an aggregate assertion (`reset > 3`, the waits seen), it falls to the smallest range that holds it, doubled for margin.
- **`ROWS` in `kv`'s `target.rs` falls to 2,500, not as low as the survey read.** The survey's 1,009 rows would keep the chunk checks, but the store would then fit inside the bounds it's held to: holding every row beside one whole encoding would pass both. 2,500 rows are 4.96 `TARGET` held that way, past `SNAPSHOT_BOUND`'s 2.25 and `recover`'s 4.5.
- **Tasks are by what they repair, one writer per crate.** The request's about six tasks can't give each of fourteen rows its own, so the small rows share two tasks. Each row's stop rule is its own, and a wide task reports what's left rather than asking.
- **The request counts fourteen tests running `cases::run`; the code has fewer.** Seven tests run a whole `cases::run`, ten times between them, counting `run_fake`'s. An eighth runs `cases::queue` eleven times. At the request's 325 ms a run and its figures for the named tests, those account for about 4.35 of the row's 4.59 s, so the plan holds whatever the count.

## Belonging

`skein/docs/foundation/testing-strategy.md`, 8 and 8.1, are the test. What a test checks decides its place, never its cost. This is a cut's reading of the design and the tests, not a run.

- **`github-conformance`'s tests stay.** Each shows a named behaviour on the fake, or a refusal of the live path, from fixed inputs: section 8's "each world's scenarios". None is a sweep or a comparison.
- **`skein-lib-tests`' `memory` stays.** Section 8 names "memory under the scenarios' ceilings" for the focused suite. The three loops are one use repeated, not many generated cases, so their cost is a count, under the request's item 3, and not a place.
- **`every_wall_day_round_trips_at_midnight` moves to fuzzy.** It sweeps all 213,503 days a `Wall` can hold for one property, and names none of them. That's section 8's "sweeps of many generated cases". `known_midnights_follow_gregorian_centuries` holds the behaviour on named days in focused, and gains the one thing only the sweep held there: a day's last nanosecond is still that day. It moves with every assertion.
- **`every_representable_wall_day_matches_gregorian_rollover` moves to fuzzy.** It compares the phrasebook's date, day by day, with a counter of months and leap years written in the test: section 8's "a function against a naive one", over every day. `dates_and_moments_use_utc_calendar_words_in_every_wall_format` holds named days in focused (leap days, a century's March 1, a year's end). It moves with every assertion.
- **`three_and_four_byte_sequences_are_judged_at_every_boundary` stays, at a smaller count.** Its name is a behaviour: every range limit of the well-formed table. It compares with the standard library, but its point is the limits, and those are `edges`. `Utf8::next` judges by `(low..=high).contains(&byte)`, so a byte between two limits can't be judged otherwise than its limits are. The full second byte added 25 times the cases and no limit. The rest can't move as it is: `Utf8` is private to `skein-json`, whose product code is `json-strings'`, and a public item for a test isn't wanted. `fuzzy::writer::text_is_written_exactly_when_it_is_utf8_and_reads_back_the_same` already holds random bytes between the limits, through the public path.
- **`a_huffman_code_is_complete_canonical_and_as_short_as_any` stays, at four seeds.** It's a seeded comparison with a plain tree builder, so section 8 would have it in fuzzy, but `huffman` is `pub(crate)` and no public path reaches its widths. It stays as a scenario, and section 8 gives a scenario "the few seeds that show its behaviour": two for each kind of counts, each seed still running every check.
- **`skein-forge-machine-world`'s `hand` and `machine` stay.** Their tables name behaviours; none is a generated sweep or a comparison. `owner_close_wins_or_follows_each_terminal` is a named lifecycle behaviour, and all 76 of its worlds stay: each row is a phase, a delivery or a transport it names.
- **`skein-kv-tests`' `target`, `disk` and `recovery` stay.** They name packing, bounded recovery, retained-log and corruption behaviours, and section 8 names "memory under the scenarios' ceilings". The disk model gives log-bound expectations; it isn't a naive function under comparison.
- **`skein-llm-world`'s `memory` stays,** the same way. `thirty_two_unknown_arrays_exercise_the_compact_record_ceiling` is one named scenario, and its 32 arrays of 1,900 values stay: its assertions fix them, 32 outright and the rest through `document().len() > 1900` and `most > 2_000_000`.
- **`skein-tls-world`'s `memory` and `exchange` stay.** They're each world's scenarios, and the ceilings under them. The seeds and offsets serve named close-state behaviours.
- **`skein-io-world`'s `scenarios` and `files` stay.** Four calm seeds and three chaos seeds are a few seeds of a named behaviour, not a sweep, though fewer will do: `fuzzy::scenarios` sweeps the same scenarios.
- **`skein-conformance-real`'s tests stay.** They're named kernel behaviours on the ring and on readiness, two backends, so not one behaviour repeated.
- **`skein-conformance-sim`'s tests stay,** at fewer calm seeds: section 8's "the few seeds that show its behaviour", with `CHAOS`'s 200 in the fuzzy suite.
- **`skein-world-tests`' `end_to_end` stays:** named startup, containment, signalling, teardown and capture behaviours against fixture processes.
- **`skein-protocol-world`'s `worlds` stays,** at fewer seeds. `the_wire_resetting_at_any_moment_leaves_both_settled` reads like a sweep, but it's also the named scenario for a reset: it stays at four if the fuzzy sweep draws resets, which its task checks, and otherwise keeps its 20.
- **`skein-http-world`'s two prefix tests move to fuzzy:** `a_transcript_cut_short_anywhere_decodes_as_the_reference_reads_its_prefix`, 188 random cuts, and `a_request_transcript_cut_short_anywhere_comes_to_what_the_reference_reads_of_its_prefix`, 1,808 cuts. Each compares what's read with a reference reader over generated cuts: section 8's "a function against a naive one". Named cut-short behaviours stay in focused, as `the_end_of_the_stream_drops_an_event_cut_short` and `an_event_cut_before_dispatch_drops_its_document_even_when_json_was_whole`. They move with every check.
- **`skein-http-world`'s other tests stay.** They hold kept goldens, each fixture to its expectation, and not generated cases.
- **`skein-fail-stop-tests` stays:** a named behaviour that needs a process's death. Its forms are all still asserted, in three runs in place of five, because the hook takes one path whatever `RUST_BACKTRACE` says.

Moving these four tests is by what they check. By the request's figures they add at most about 0.7 CPU s to a fuzzy suite with room: 0.23 for the two day sweeps, and at most the 0.44 of the http row for the other two. Nothing else leaves focused.

## Tests that go

Each is named with its holder, which exists and passes first.

- `github_conformance_missing_delivery_fails_only_its_fact`: held by `github_conformance_faults_fail_only_their_facts_and_the_run_continues`. Holder passed at `726e06247ae00d3076bd20a1a6d2f126d76d1813`. Section 8 allows it: the whole run is the fixture and the cost, and each fault's check costs little beside it.
- `github_conformance_null_merged_commit_fails_the_fact_and_continues`: held by the same test, for the same reason; holder passed at `726e06247ae00d3076bd20a1a6d2f126d76d1813`.

- `skein-ui::tests::date::every_wall_day_round_trips_at_midnight`: held whole in fuzzy by `skein-ui-tests::suite::fuzzy::date::every_wall_day_round_trips_at_midnight`, passed at `4327ac3b36`. The focused `tests::date::known_midnights_follow_gregorian_centuries`, including the last-nanosecond check, passed there too.
- `skein-ui::tests::phrases::every_representable_wall_day_matches_gregorian_rollover`: held whole in fuzzy by `skein-ui-tests::suite::fuzzy::phrases::every_representable_wall_day_matches_gregorian_rollover`, passed at `4327ac3b36`. The focused `tests::phrases::dates_and_moments_use_utc_calendar_words_in_every_wall_format` passed there too.
- `skein-http-world::suite::transcripts::a_transcript_cut_short_anywhere_decodes_as_the_reference_reads_its_prefix`: held intact by `fuzzy::client::a_transcript_cut_short_anywhere_decodes_as_the_reference_reads_its_prefix`, passed at `e5f7735bafee9614d7040416ccca19be15a0d4fa`. Named cut-short behaviours remain in focused.
- `skein-http-world::suite::requests::a_request_transcript_cut_short_anywhere_comes_to_what_the_reference_reads_of_its_prefix`: held intact by `fuzzy::server::a_request_transcript_cut_short_anywhere_comes_to_what_the_reference_reads_of_its_prefix`, passed at `e5f7735bafee9614d7040416ccca19be15a0d4fa`, with every reference-reader assertion.

## Overlaps

These couldn't be avoided:

- **`sweeps` meets `limits-fixtures/view-words`** in `skein/crates/skein-ui/src/tests/phrases.rs`, where it deletes one test and nothing else: that's where the test is.
- **`machine-costs` meets `skein-world-costs`** in `skein/tests/forge/machine/src/hand.rs`, whose `peer.wire.trace` they change. It keeps its hunk to the quiet count and the copy of the peers, and on a merge keeps their form of the trace.
- **`memory-worlds` may meet `skein-world-costs`** in `skein/testing/skein-llm-kit/**` and `skein-tls-kit/**`, which that plan profiles for formatted traces. That plan leaves a test's own fixture to this one. On a merge, their change stands.
- **`scenario-worlds` may meet `limits-worlds/engine-causes`,** which may change any of `skein/crates/**`, if a profile sends it to a product crate. It's mostly in test files.

This plan leaves alone `skein_heap::Counting` and `skein_world::stream::Wire` (`skein-world-costs`), `skein-json`'s product code (`json-strings`), and what a process costs before its test starts. Other plans add tests to these crates all the time, so a task leaves what it finds and expects catch-ups. A helper's change stays in one task.

## The table

Every test touched or profiled in all six figures files appears below, including unchanged tests, moved holders, new regressions and unknown counts. Counts are instructions (Ir), not CPU seconds; inclusive causes overlap. Before and after belong to each task's own pair. Zero in the calendar rows means no execution in that suite. The closing measurements and estimates are in `skein/docs/development/skein-limits-heavy.md`.

### github-runs

Source: `figures/github-runs.md`.

| Row | Test | Before (Ir) | After (Ir) | Cause | What changed |
|---|---|---:|---:|---|---|
| `github-conformance` `tests` | `tests::github_conformance_holds_every_fact_on_the_fake` | 2,557,261,726 | 1,670,813,970 | Secret scans, tiny padding chunks, and copies for absent opaque values | Fix those causes; keep the whole run and every assertion |
| `github-conformance` `tests` | `tests::github_conformance_names_a_token_expiring_within_30_days` | 90,195,918 | 69,910,384 | Setup pays the shared scan and padding costs | Same setup-only test; unchanged expiry and cleanup checks |
| `github-conformance` `tests` | `tests::github_conformance_queue_refusals_fail_one_fact` | 6,366,114,766 | 1,964,375,225 | Eleven fixtures and queue runs, plus the whole-run causes | Keep eleven refusal rows in six queue runs, with exact reasons and every cleanup check |
| `github-conformance` `tests` | `tests::github_conformance_masks_secrets_and_ids` | 2,769,528 | 2,589,454 | Body copied for each known opaque value even when absent | Search first; keep every masking category and embedded GraphQL node-id check |
| `github-conformance` `tests` | `tests::github_conformance_faults_fail_only_their_facts_and_the_run_continues` | unknown | 1,712,490,140 | Separate whole runs checked individual faults | New holder: one whole run, four faults, every later fact and cleanup held; no before profile |
| `github-conformance` `tests` | `tests::github_conformance_response_framing_records_the_exchange_and_continues` | unknown | 284,010,002 | Two whole runs checked a log exchange | Two log-group runs, six facts each; all outcome, transcript, masking, provenance and cleanup assertions stay; no before profile |
| `github-conformance` `tests` | `tests::github_conformance_epoch_refusals_and_non_json_fail_only_the_fact` | unknown | 406,786,398 | Three whole runs checked one epoch fact | Three epoch-only runs; all three status/body rows and cleanup assertions stay; no before profile |
| `github-conformance` `tests` | `tests::github_conformance_discovers_the_organization_before_minting` | unknown | 473,310,731 | Whole run checked organization discovery and queue rulesets | Queue alone still covers every merge-queue ruleset; discovery and before-minting checks stay; no before profile |
| `github-conformance` `tests` | `tests::github_conformance_missing_delivery_fails_only_its_fact` | unknown | unknown | Whole fixture for one absent input | Removed; holder below; no separate profile |
| `github-conformance` `tests` | `tests::github_conformance_null_merged_commit_fails_the_fact_and_continues` | unknown | unknown | Whole fixture for one malformed merged-pull answer | Removed; holder below; no separate profile |

### lib-counts

Source: `figures/lib-counts.md`.

| Row | Test | Before (Ir) | After (Ir) | Cause | What changed |
|---|---|---:|---:|---|---|
| `skein-lib-tests::suite` `memory` | `memory::a_deadline_table_holds_no_more_than_its_price` | 144,186,938 | 144,186,826 | `Deadlines::arm` 52,813,366; `Metered::end` 38,932,868 inclusive Ir | stays: per-operation price checks |
| `skein-lib-tests::suite` `memory` | `memory::a_journal_scenario_stays_under_its_peak_ceiling` | 1,417,947 | 1,417,712 | Process startup and counting-allocator initialization dominate | stays: no repeated large allocation |
| `skein-lib-tests::suite` `memory` | `memory::a_list_clone_holds_only_its_items_and_grows_as_it_fills` | 1,307,552 | 1,307,432 | Process startup and counting-allocator initialization dominate | stays: no repeated large allocation |
| `skein-lib-tests::suite` `memory` | `memory::a_list_filled_to_its_capacity_stays_within_its_price` | 9,460,427 | 9,460,251 | `Metered::end` 5,046,959 inclusive Ir; container operations | stays: per-operation price checks |
| `skein-lib-tests::suite` `memory` | `memory::a_list_grows_to_its_capacity_exactly` | 2,365,976 | 2,365,842 | `Metered::end` 618,549 inclusive Ir; container operations | stays: per-operation price checks |
| `skein-lib-tests::suite` `memory` | `memory::a_lru_holds_no_more_than_its_price` | 239,813,089 | 239,812,942 | `Lru::insert` 104,152,667; `Lru::remove` 66,608,792 inclusive Ir | stays: tree operations over the named capacity scenarios |
| `skein-lib-tests::suite` `memory` | `memory::a_map_holds_no_more_than_its_price` | 119,403,784 | 119,403,768 | `Metered::end` 42,035,703 inclusive Ir; container operations | stays: per-operation price checks |
| `skein-lib-tests::suite` `memory` | `memory::a_queue_filled_to_its_capacity_stays_within_its_price` | 20,737,342 | 20,737,152 | `Metered::end` 12,754,407 inclusive Ir; container operations | stays: per-operation price checks |
| `skein-lib-tests::suite` `memory` | `memory::a_queue_grows_to_its_capacity_exactly` | 3,364,663 | 3,364,531 | `Metered::end` 1,206,386 inclusive Ir; container operations | stays: per-operation price checks |
| `skein-lib-tests::suite` `memory` | `memory::a_queue_with_a_large_capacity_used_lightly_holds_what_it_holds` | 3,287,265,890 | 14,725,579 | A million metered turns: `Metered::end` 1,920,002,132 inclusive Ir | Turns 1,000,000 → 4,096; every step still metered |
| `skein-lib-tests::suite` `memory` | `memory::a_request_table_grows_within_its_held_and_peak_bounds` | 627,061,810 | 627,061,701 | `RequestTake::next_out` scans admission order: 324,144,260 inclusive Ir | stays: next cause about 0.047 CPU s; counts preserve named lifecycle checks |
| `skein-lib-tests::suite` `memory` | `memory::a_ring_used_lightly_holds_what_it_holds` | 3,287,267,961 | 14,727,308 | A million metered turns: `Metered::end` 1,920,002,132 inclusive Ir | Turns 1,000,000 → 4,096; every step still metered |
| `skein-lib-tests::suite` `memory` | `memory::a_set_holds_no_more_than_its_price` | 113,493,073 | 113,492,973 | `Metered::end` 38,619,824 inclusive Ir; container operations | stays: per-operation price checks |
| `skein-lib-tests::suite` `memory` | `memory::a_slab_grows_to_its_capacity_exactly` | 13,871,565 | 13,871,562 | Earlier-handle checks: `Slab::get` 2,410,493 inclusive Ir | stays: checks handles survive every growth |
| `skein-lib-tests::suite` `memory` | `memory::a_slab_grows_within_its_held_and_peak_bounds` | 29,532,249 | 29,532,115 | `Metered::end` 17,362,716 inclusive Ir; container operations | stays: per-operation price checks |
| `skein-lib-tests::suite` `memory` | `memory::a_stack_filled_to_its_capacity_stays_within_its_price` | 18,072,561 | 18,072,405 | `Metered::end` 11,004,269 inclusive Ir; container operations | stays: per-operation price checks |
| `skein-lib-tests::suite` `memory` | `memory::a_stack_grows_to_its_capacity_exactly` | 3,105,292 | 3,105,115 | `Metered::end` 1,196,168 inclusive Ir; container operations | stays: per-operation price checks |
| `skein-lib-tests::suite` `memory` | `memory::an_intake_filled_to_its_capacity_stays_within_its_price` | 16,894,834 | 16,894,780 | `Metered::end` 8,001,094 inclusive Ir; container operations | stays: per-operation price checks |
| `skein-lib-tests::suite` `memory` | `memory::an_intake_grows_to_its_capacity_exactly` | 2,826,433 | 2,826,326 | `Metered::end` 950,432 inclusive Ir; container operations | stays: per-operation price checks |
| `skein-lib-tests::suite` `memory` | `memory::an_intake_with_a_large_capacity_used_lightly_holds_what_it_holds` | 6,720,266,382 | 111,498,172 | A million metered deliveries: `Metered::end` 3,779,002,213; `Intake::meet` 1,893,000,172 inclusive Ir | Capacity 1,048,576 → 16,384; turns 1,000,000 → 16,384 |
| `skein-lib-tests::suite` `memory` | `memory::an_uncapped_list_allocates_only_as_items_arrive` | 1,274,582 | 1,274,467 | Process startup and counting-allocator initialization dominate | stays: no repeated large allocation |
| `skein-lib-tests::suite` `memory` | `memory::an_uncapped_map_allocates_only_as_entries_arrive` | 1,303,700 | 1,303,618 | Process startup and counting-allocator initialization dominate | stays: no repeated large allocation |
| `skein-lib-tests::suite` `memory` | `memory::an_unused_intake_holds_nothing` | 1,260,179 | 1,260,120 | Process startup and counting-allocator initialization dominate | stays: no repeated large allocation |
| `skein-lib-tests::suite` `memory` | `memory::an_unused_list_holds_nothing` | 1,260,074 | 1,259,982 | Process startup and counting-allocator initialization dominate | stays: no repeated large allocation |
| `skein-lib-tests::suite` `memory` | `memory::an_unused_queue_holds_nothing` | 1,260,218 | 1,260,159 | Process startup and counting-allocator initialization dominate | stays: no repeated large allocation |
| `skein-lib-tests::suite` `memory` | `memory::an_unused_slab_holds_nothing` | 1,261,410 | 1,261,365 | Process startup and counting-allocator initialization dominate | stays: no repeated large allocation |
| `skein-lib-tests::suite` `memory` | `memory::an_unused_stack_holds_nothing` | 1,261,380 | 1,260,026 | Process startup and counting-allocator initialization dominate | stays: no repeated large allocation |
| `skein-lib-tests::suite` `memory` | `memory::an_unused_uncapped_queue_holds_nothing` | 1,263,802 | 1,263,743 | Process startup and counting-allocator initialization dominate | stays: no repeated large allocation |

### sweeps

Source: `figures/sweeps.md`.

| Row | Test | Before (Ir) | After (Ir) | Cause | What changed |
|---|---|---:|---:|---|---|
| `skein-ui` `tests::date` | `tests::date::every_wall_day_round_trips_at_midnight` | 896,193,271 | 0 | Parsing every day's text costs 442,777,692 inclusive instructions, formatting 320,866,908 | Moves whole to fuzzy: a sweep over every representable day, rather than a named scenario |
| `skein-ui` `tests::phrases` | `tests::phrases::every_representable_wall_day_matches_gregorian_rollover` | 1,090,220,691 | 0 | Repeated `Phrasebook::say` costs 767,847,787 inclusive instructions | Moves whole to fuzzy: comparison against a naive Gregorian counter |
| `skein-ui-tests::suite` `fuzzy::date` | `fuzzy::date::every_wall_day_round_trips_at_midnight` | 0 | 902,606,331 | The complete calendar and text round trips | Fuzzy gains every assertion over all 213,504 representable days, including the partial final day |
| `skein-ui-tests::suite` `fuzzy::phrases` | `fuzzy::phrases::every_representable_wall_day_matches_gregorian_rollover` | 0 | 1,900,707,510 | Saying and building expected words for every day, now under the world's counting allocator | Fuzzy gains the complete Gregorian comparison over the same 213,504 days |
| `skein-ui` `tests::date` | `tests::date::known_midnights_follow_gregorian_centuries` | unknown | unknown | Not profiled: five named dates | Also checks that each day's last nanosecond still maps to that date |
| `skein-ui` `tests::phrases` | `tests::phrases::dates_and_moments_use_utc_calendar_words_in_every_wall_format` | unknown | unknown | Not profiled: unchanged focused holder | Stays: named leap days, century rollover, year end, and the largest wall instant |
| `skein-json` `tests::text` | `tests::text::three_and_four_byte_sequences_are_judged_at_every_boundary` | 939,051,364 | 43,028,377 | Repeated `checked` costs 579,302,752 inclusive instructions before; the second byte enumerates interiors as well as limits | Second byte uses the same ten edges as subsequent bytes; every equality assertion stays |
| `skein-http` `tests::gzip` | `tests::gzip::a_huffman_code_is_complete_canonical_and_as_short_as_any` | 768,988,106 | 152,422,834 | Repeated sorting in the plain `least` tree builder costs 542,600,173 inclusive instructions before | Four seeds, two for each `rarest`, retaining all code and weight checks |
| `skein-http` `tests::gzip` | `tests::gzip::a_code_deeper_than_its_widest_is_cut_to_it` | unknown | unknown | Not profiled: unchanged deterministic cut holder | Stays: Fibonacci counts force both width cuts |

### machine-costs

Source: `figures/machine-costs.md`.

| Row | Test | Before (Ir) | After (Ir) | Cause | What changed |
|---|---|---:|---:|---|---|
| `suite` `machine` | `machine::owner_close_wins_or_follows_each_terminal` | 1,947,965,463 | 919,345,951 | Quiet ticks repeatedly allocate peer keys, inspect requests and check deadlines; settling is 97.44% of the baseline | Stop after sixteen quiet ticks, retaining the 1,000-tick bound and both existing exit conditions; all 76 worlds stay |
| `suite` `machine` | `machine::a_returned_answer_is_never_offered_again` | 41,177,459 | 35,374,946 | Three owner actions settle even when their work has ended; settling costs 5,934,129 Ir | Quiet termination reduces settling to 97,254 Ir; every delivery and close assertion stays |
| `suite` `machine` | `machine::the_start_reaches_ready_from_the_recorded_answers` | 9,841,209 | 9,851,503 | Stream delivery and recorded-response processing through `run_for`, rather than `settle` | Stays: `run_for` keeps its clock-driving behaviour; tick progress bookkeeping adds 0.10% |
| `suite` `hand` | `hand::github::github_installation_loss::stale_full_after_deleted_is_discarded` | 33,219,510 | 30,917,000 | Two RSA signings cost 22,621,385 Ir; draining adds 7,353,732 Ir | Remove key-list copies and halve the quiet tail; signing stays |
| `suite` `hand` | `hand::forgejo::contents::a_failed_step_after_more_steps_than_inner_is_found` | 16,144,095 | 13,167,685 | Draining costs 12,840,858 Ir, including 1,053,650 Ir copying peer keys; stream tracing and allocation accounting also recur | Visit peers in place, gather events locally, and stop after sixteen quiet rounds |
| `suite` `hand` | `hand::github::github_calls::repository_account_selects_its_accepted_installation` | 5,473,654 | 4,471,441 | Draining costs 2,359,104 Ir, including 187 peer-key copies costing 333,282 Ir | No peer-key list on rounds or stream requests; empty worlds return immediately; draining falls to 1,353,517 Ir |
| `suite` `machine` | `machine::settlement_counts_fragments_before_a_header_line_is_complete` | unknown | unknown | New regression, not profiled: a header needs more fragments than the quiet threshold before a complete-line demand can answer | Counts trace growth as progress; without it the test fails at `world.script_done()`, and passes after restoration |
| `temper-engine-protocol-forge-world::suite` `protocol` | `protocol::github_reaches_ready_through_its_accepted_installations` | 57,923,886 | 51,058,287 | Its installation action settles 1,000 ticks despite finishing its same-time work | Shared quiet termination reduces that action to 177 ticks; the reader also costs less |

### memory-worlds

Source: `figures/memory-worlds.md`.

| Row | Test | Before (Ir) | After (Ir) | Cause | What changed |
|---|---|---:|---:|---|---|
| `skein-kv-tests::suite target, disk, recovery` | `target::recovery_stays_near_the_target_under_a_large_chunk` | 1,964,883,988 | 1,487,365,593 | CRC-32C over committed, snapshotted and recovered bytes; 64% inclusive | 3,300 rows → 2,500; bounds unchanged |
| `skein-kv-tests::suite target, disk, recovery` | `target::many_small_rows_pack_to_the_target` | 1,420,279,847 | 1,075,059,510 | CRC-32C and snapshot row copying; CRC is 59% after the count change | 3,300 rows → 2,500; three chunks still required |
| `skein-kv-tests::suite target, disk, recovery` | `disk::a_tiny_store_keeps_its_log_under_the_floor` | 359,990,914 | 359,989,379 | Commit simulation; Observed::check includes full-file copies | stays: snapshot headers and retained-log assertions remain exact |
| `skein-tls-world::suite memory, exchange` | `memory::failures_and_closes_in_every_state` | 2,415,439,865 | 2,415,465,585 | Repeated real handshakes, key exchange and certificate verification | stays: both versions, both injected endings and all 30 offsets |
| `skein-tls-world::suite memory, exchange` | `exchange::closes_in_every_state` | 1,142,402,294 | 716,734,618 | Repeated handshakes at seed-selected close points | 60/10 seeds → 40/6; every asserted wait still observed |
| `skein-tls-world::suite memory, exchange` | `exchange::close_notify_from_the_server_ends_the_stream` | 45,660,336 | 45,661,924 | Fragmented exchange and TLS handshake | stays: one exchange |
| `skein-llm-world::suite memory` | `memory::thirty_two_unknown_arrays_exercise_the_compact_record_ceiling` | 2,836,038,627 | 2,836,038,591 | JSON collection, tokenization, retained replay decoding and encoding | stays: 32 arrays × 1,900 values; 4,100,000 ceiling; peak > 2,000,000 |
| `skein-llm-world::suite memory` | `memory::preparation_and_streaming_stay_under_the_peak_ceiling` | 16,721,823 | 16,721,925 | Client preparation, JSON collection and fragmented SSE delivery | stays: both HTTP framings and all ceiling checks |
| `skein-io-world::suite scenarios, files` | `files::the_largest_admitted_store_stays_within_its_memory_ceiling` | 1,292,419,870 | 17,117,922 | 32,768 two-byte reads, simulator turns and allocations | 1,024-byte reads; whole-file size, contents and ceiling unchanged |
| `skein-io-world::suite scenarios, files` | `scenarios::a_server_that_does_not_demand_stops_its_peer` | 1,475,183,378 | 181,517,712 | Byte-by-byte matching-prefix rescans in referee::common | Slice prefix comparison on matching deliveries; 7 seeds → 4 |
| `skein-io-world::suite scenarios, files` | `files::a_conflict_leaves_the_other_writers_file` | 3,152,037 | 3,154,853 | Small replacement conflict world | stays: two-byte reads and all conflict assertions |
| `skein-io-world::suite scenarios, files` | `scenarios::a_listener_accepts_binds_and_rejects` | unknown | unknown | Not separately profiled; shares the scenario seed helpers and referee | 7 seeds → 4; assertions unchanged |
| `skein-io-world::suite scenarios, files` | `scenarios::connects_are_made_refused_for_want_of_a_slot_and_refused_by_the_peer` | unknown | unknown | Not separately profiled; shares the scenario seed helpers and referee | 7 seeds → 4; assertions unchanged |
| `skein-io-world::suite scenarios, files` | `scenarios::connects_waiting_on_a_full_backlog_are_cancelled_by_their_abort` | unknown | unknown | Not separately profiled; shares the scenario seed helpers and referee | 7 seeds → 4; assertions unchanged |
| `skein-io-world::suite scenarios, files` | `scenarios::a_socket_and_an_accept_out_of_descriptors_are_refused_and_starved` | unknown | unknown | Not separately profiled; shares the scenario seed helpers and referee | 7 seeds → 4; assertions unchanged |
| `skein-io-world::suite scenarios, files` | `scenarios::the_accept_batch_holds_a_second_listener_back_an_iteration` | unknown | unknown | Not separately profiled; shares the scenario seed helpers and referee | 7 seeds → 4; assertions unchanged |
| `skein-io-world::suite scenarios, files` | `scenarios::a_socket_accepted_with_no_slot_left_is_discarded` | unknown | unknown | Not separately profiled; shares the scenario seed helpers and referee | 7 seeds → 4; assertions unchanged |
| `skein-io-world::suite scenarios, files` | `scenarios::a_burst_of_connects_fills_the_refusals_and_io_takes_no_more_until_told` | unknown | unknown | Not separately profiled; shares the scenario seed helpers and referee | 7 seeds → 4; assertions unchanged |
| `skein-io-world::suite scenarios, files` | `scenarios::bytes_go_both_ways_under_demands_of_every_kind` | unknown | unknown | Not separately profiled; shares the scenario seed helpers and referee | 7 seeds → 4; assertions unchanged |
| `skein-io-world::suite scenarios, files` | `scenarios::a_response_sent_in_the_middle_of_an_upload_reaches_the_peer` | unknown | unknown | Not separately profiled; shares the scenario seed helpers and referee | 7 seeds → 4; assertions unchanged |
| `skein-io-world::suite scenarios, files` | `scenarios::an_abort_cancels_what_waits` | unknown | unknown | Not separately profiled; shares the scenario seed helpers and referee | 7 seeds → 4; assertions unchanged |
| `skein-io-world::suite scenarios, files` | `scenarios::a_close_the_peer_never_ends_is_aborted_by_its_deadline` | unknown | unknown | Not separately profiled; shares the scenario seed helpers and referee | 7 seeds → 4; assertions unchanged |
| `skein-io-world::suite scenarios, files` | `scenarios::closes_and_aborts_in_every_state_settle` | unknown | unknown | Not separately profiled; shares the scenario seed helpers and referee | 7 seeds → 4; assertions unchanged |
| `skein-io-world::lib` | `referee::tests::common_keeps_matching_prefixes_and_the_first_mismatch_offset` | unknown | unknown | New focused check, not separately profiled | Matching prefixes, empty input, overlong input and exact mismatch offsets |

### scenario-worlds

Source: `figures/scenario-worlds.md`.

| Row | Test | Before (Ir) | After (Ir) | Cause | What changed |
|---|---|---:|---:|---|---|
| `skein-conformance-real::suite` | `conformance::ring::sockets::backpressure_stalls_a_sender_and_reading_resumes_it` | 81,011,411 | 61,798,657 | Costliest: deadline snapshots, then pattern generation | Reap first and take one nonblocking backend turn before accounting |
| `skein-conformance-real::suite` | `conformance::ring::sockets::records_wrong_for_the_socket_state` | 7,059,319 | 1,578,979 | Lower median: deadline snapshots on immediate refusals | Completed operations need no new deadline |
| `skein-conformance-real::suite` | `conformance::ring::files::a_file_is_made_written_at_offsets_read_back_and_stated` | 12,698,275 | 2,384,777 | 10,579,511 Ir in wait_begins before; 397,705 after | Reap first; enter deferred work without blocking; keep a deadline if anything remains in flight |
| `skein-conformance-sim::suite` | `conformance::backpressure_stalls_a_sender_and_reading_resumes_it` | 1,341,576,696 | 465,793,151 | Costliest: repeated pattern construction and simulated transfers | 16 calm worlds become 4; 4 smoke worlds stay |
| `skein-conformance-sim::suite` | `conformance::a_close_with_bytes_unread_meets_a_send` | 11,122,646 | 5,328,112 | Median: repeated socket scenarios | 16 calm worlds become 4; smoke worlds stay |
| `skein-conformance-sim::suite` | `conformance::a_calm_world_pairs_each_race_as_the_ring_does` | 43,542,271 | 11,529,157 | Six explicit races and an established-connect cancel per seed | 112 scenario runs become 28; every pairing check stays |
| `skein-conformance-sim::suite` | `conformance::a_file_is_made_written_at_offsets_read_back_and_stated` | 30,712,809 | 13,413,326 | Repeated file scenarios and boundary checks | 16 calm worlds become 4; smoke worlds stay |
| `skein-conformance-sim::suite` | `conformance::a_connection_lives_and_ends_over_ipv4` | unknown | 101,039,244 | Supplementary after-only profile of transfer copies | CALM changes through the shared helper; source-copy gain is below the stop rule |
| `skein-http-world::suite` | `transcripts::every_transcript_decodes_to_its_expectation_whatever_the_cuts` | 1,158,931,907 | 601,450,632 | Costliest: four runs of each golden, bytes read one at a time | Two seeds per fixture; every golden comparison stays |
| `skein-http-world::suite` | `transcripts::every_event_stream_reads_to_its_expectation_whatever_the_cuts_and_scans` | 434,273,572 | 434,273,604 | Median: event-stream decoding and metered delivery | Stays: all four named chunk sizes |
| `skein-http-world::suite` | `requests::every_request_transcript_comes_to_its_expectation_whatever_the_pace` | 304,201,086 | 143,144,308 | Five runs of each request golden | Two seeds per fixture; every head, body and outcome check stays |
| `skein-http-world::suite` | `fuzzy::client::a_transcript_cut_short_anywhere_decodes_as_the_reference_reads_its_prefix` | 583,167,779 | 583,168,024 | Generated cuts compared with the reference | Moves from transcripts to fuzzy::client, with all 188 runs and checks |
| `skein-http-world::suite` | `fuzzy::server::a_request_transcript_cut_short_anywhere_comes_to_what_the_reference_reads_of_its_prefix` | 1,171,294,679 | 1,171,295,310 | Generated prefixes compared with the reference | Moves from requests to fuzzy::server, with all 1,808 cuts and checks |
| `skein-protocol-world::suite` | `worlds::one_end_closing_while_the_other_sends_leaves_both_settled` | 584,651,574 | 929,211,659 | Costliest: generated JSON and stacked decoding | 30 becomes 46: the first prefix with four resets has 23 seeds, doubled for margin |
| `skein-protocol-world::suite` | `worlds::the_wire_resetting_at_any_moment_leaves_both_settled` | 160,908,075 | 30,676,450 | Lower median: repeated reset worlds | 20 becomes 4; the fuzzy sweep injects resets and asserts that one fell |
| `skein-protocol-world::suite` | `worlds::an_llm_s_stream_goes_from_the_server_s_top_to_the_client_s_top` | 225,104,587 | 34,719,646 | Generated streaming worlds | 12 becomes 2; ids > 0 stays |
| `skein-protocol-world::suite` | `worlds::a_connection_carries_two_or_three_calls_through_both_stacks` | 215,449,640 | 125,094,335 | Repeated multi-call worlds | 8 becomes 4; explicitly assert that both call counts appeared |
| `skein-protocol-world::suite` | `worlds::a_seed_replays_to_the_same_run` | 118,309,352 | 86,948,501 | Two complete runs for each replay seed | 4 seeds become 2; both runs still compare in full |
| `skein-protocol-world::suite` | `worlds::a_slow_consumer_at_the_server_s_top_stops_the_upload_at_the_client_s` | 273,156,432 | 273,159,000 | JSON string decoding: 86,420,843 Ir after | Stays: three seeds, each asserting upload_held and complete requests |
| `skein-fail-stop-tests::suite` | `fail_stop::a_panic_on_a_second_thread_aborts_the_process_after_its_report` | 1,043,350,176 | 348,430,893 | Backtrace symbolization in fixture processes; parent plus children summed | Costliest: three borrowed-payload runs become one |
| `skein-fail-stop-tests::suite` | `fail_stop::owned_and_non_string_payloads_follow_the_default_report` | 680,491,706 | 680,804,926 | Backtrace symbolization in fixture processes; parent plus children summed | Lower median: two named-payload runs stay; they carry the other backtrace values |
| `skein-world-tests::suite` | `end_to_end::an_observer_can_be_abandoned_before_its_first_submission` | unknown | unknown | Costliest: Callgrind cannot complete the process operations; native runs pass | Stays: eight fixture starts for two backends, two modes and queued/unsubmitted observation |
| `skein-world-tests::suite` | `end_to_end::abandonment_removes_the_cgroup_after_its_blocked_member_exits` | unknown | unknown | Median: Callgrind cannot complete the process operations; native runs pass | Stays: one fixture tree and its blocked member |
| `skein-world-tests::suite` | `end_to_end::repeated_stderr_overflow_unwinds_without_aborting` | unknown | unknown | Specified cause: Callgrind cannot complete the process operations; native runs pass | 20 fixture starts become 3; exact diagnostic checked on every unwind |
| `skein-world-tests::suite` | `end_to_end::a_binary_that_exits_at_once_settles_with_counts_above_zero` | unknown | unknown | Specified representative: Callgrind cannot complete the process operations; native runs pass | Stays: three containment forms, each with a fixture start |
| `skein-conformance-sim::suite` | `conformance::child_processes_and_pipes` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_connection_lives_and_ends_over_ipv6` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_peer_closes_gracefully` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_send_after_the_peer_closed` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_connect_where_nothing_listens` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::where_an_address_is_in_use` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::ipv6_sockets_are_ipv6_only` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::records_wrong_for_the_socket_state` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_full_accept_queue` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_close_with_bytes_unread_meets_a_recv` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_reset_after_the_end_of_stream` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_client_closed_before_its_connection_is_accepted` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_cancel_of_a_waiting_accept` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_cancel_of_a_waiting_recv` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_cancel_of_a_recv_racing_bytes` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_cancel_of_an_accept_racing_a_connect` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_cancel_of_a_waiting_connect` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_cancel_of_a_connect_established_while_its_client_was_away` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::closing_a_listener_resets_the_connections_waiting_on_it` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::an_accept_past_the_descriptor_limit` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_calm_world_reads_and_writes_whole_as_the_ring_does` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::renames_over_across_and_beneath` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::removes_of_files_directories_and_links` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::new_directories_and_one_removed_while_open` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::listings_whole_one_at_a_time_and_cut_short` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_root_beneath_a_root` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::paths_that_leave_their_root_and_paths_that_stay` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::what_the_owner_may_not_do` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::an_open_past_the_descriptor_limit` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_cancel_of_a_read_of_a_file` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::the_group_is_signalled_after_its_leader_exits` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::usage_counts_the_child_only_after_reaping` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::appends_preserve_the_prefix_and_follow_the_other_descriptors_bytes` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_cancel_of_an_append_stops_it_or_comes_too_late` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::stat_keeps_the_owner_and_counts_hard_links_through_removal` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::path_only_metadata_needs_no_permission_on_its_entry` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::window_on_a_descriptor_that_is_not_a_terminal_answers_other` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_write_open_finds_what_exists_and_truncate_sets_its_length` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_directory_lock_is_busy_through_a_second_open_until_the_first_closes` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |
| `skein-conformance-sim::suite` | `conformance::a_no_replace_rename_refuses_a_taken_target_in_linux_order` | unknown | unknown | Shared CALM count; not individually profiled | 16 calm seeds become 4; all other counts and assertions stay |

The plan's implementation and closing account are complete; the gate's own CPU reading follows landing.
