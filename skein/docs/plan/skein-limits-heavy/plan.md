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

- `github_conformance_missing_delivery_fails_only_its_fact`: held by `github_conformance_faults_fail_only_their_facts_and_the_run_continues`. Section 8 allows it: the whole run is the fixture and the cost, and each fault's check costs little beside it.
- `github_conformance_null_merged_commit_fails_the_fact_and_continues`: held by the same test, for the same reason.

## Overlaps

These couldn't be avoided:

- **`sweeps` meets `limits-fixtures/view-words`** in `skein/crates/skein-ui/src/tests/phrases.rs`, where it deletes one test and nothing else: that's where the test is.
- **`machine-costs` meets `skein-world-costs`** in `skein/tests/forge/machine/src/hand.rs`, whose `peer.wire.trace` they change. It keeps its hunk to the quiet count and the copy of the peers, and on a merge keeps their form of the trace.
- **`memory-worlds` may meet `skein-world-costs`** in `skein/testing/skein-llm-kit/**` and `skein-tls-kit/**`, which that plan profiles for formatted traces. That plan leaves a test's own fixture to this one. On a merge, their change stands.
- **`scenario-worlds` may meet `limits-worlds/engine-causes`,** which may change any of `skein/crates/**`, if a profile sends it to a product crate. It's mostly in test files.

This plan leaves alone `skein_heap::Counting` and `skein_world::stream::Wire` (`skein-world-costs`), `skein-json`'s product code (`json-strings`), and what a process costs before its test starts. Other plans add tests to these crates all the time, so a task leaves what it finds and expects catch-ups. A helper's change stays in one task.
