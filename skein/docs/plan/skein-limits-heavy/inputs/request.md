# skein-limits-heavy: skein's costliest focused tests, by 8.1's questions

skein's focused suite has a limit of 30 CPU s and reads 38.2, in 7,687 tests (`skein/docs/foundation/testing-strategy.md`, 8 and 8.1), and it has grown about a second a day since 11-04. About 15 s of that is each process's cost before a test does anything, and other plans take it. This plan takes the costliest work above it, and aims for at least 6 CPU s.

Medians of three gate runs, on loaded machines:

| Binary | Module or test | Tests | CPU s | What's known |
|---|---|---:|---:|---|
| `github-conformance` | `tests` | 29 | 4.59 | `forgejo-conformance`'s 47 read 0.16. Fourteen tests each run `cases::run` whole, about 325 ms a time, up to three times: `..._epoch_refusals_and_non_json_fail_only_the_fact` 969 ms, `..._queue_refusals_fail_one_fact` 854, `..._response_framing_records_the_exchange_and_continues` 699 |
| `skein-forge-machine-world::suite` | `hand` | 387 | 1.92 | Median 4.9 ms, against the binary's floor of 2.7 |
| | `machine` | 26 | 0.55 | `owner_close_wins_or_follows_each_terminal` 279 ms |
| `skein-lib-tests::suite` | `memory` | 28 | 1.56 | Three metered loops of a million turns: `an_intake_with_a_large_capacity_used_lightly_holds_what_it_holds` 698 ms, `a_queue_...` 339, `a_ring_used_lightly_holds_what_it_holds` 336 |
| `skein-kv-tests::suite` | `target`, `disk`, `recovery` | 16 | 1.22 | `recovery_stays_near_the_target_under_a_large_chunk` 235 ms, `many_small_rows_pack_to_the_target` 186 |
| `skein-llm-world::suite` | `memory` | 19 | 0.76 | `thirty_two_unknown_arrays_exercise_the_compact_record_ceiling` 358 ms |
| `skein-conformance-real::suite` | `conformance` | 82 | 0.76 | Median 8 ms, against a floor of 2.7 |
| `skein-world-tests::suite` | `end_to_end` | 43 | 0.69 | |
| `skein-tls-world::suite` | `memory`, `exchange` | 16 | 0.82 | `failures_and_closes_in_every_state` 271 ms, `closes_in_every_state` 140 |
| `skein-conformance-sim::suite` | `conformance` | 47 | 0.39 | |
| `skein-io-world::suite` | `scenarios`, `files` | 29 | 0.63 | `the_largest_admitted_store_stays_within_its_memory_ceiling` 221 ms, `a_server_that_does_not_demand_stops_its_peer` 159 |
| `skein-fail-stop-tests::suite` | `fail_stop` | 2 | 0.36 | Five fixture runs that panic, at about 70 ms each |
| `skein-protocol-world::suite` | `worlds` | 8 | 0.30 | Its floor is 17 ms |
| `skein-http-world::suite` | `transcripts`, `requests` | 7 | 0.44 | |

Sweeps worth question 1, below: `skein-ui`'s `tests::date` and `tests::phrases` (`every_representable_wall_day_matches_gregorian_rollover` 125 ms, `every_wall_day_round_trips_at_midnight` 105), `skein-json`'s `tests::text` (`three_and_four_byte_sequences_are_judged_at_every_boundary` 125), `skein-http`'s `tests::gzip` (`a_huffman_code_is_complete_canonical_and_as_short_as_any` 58).

## What's wanted

**1. Does each belong in focused?** Section 8 puts "a function against a naive one" and sweeps of many generated cases in fuzzy, and fuzzy has room. Decide from what each checks, and name the reason in `plan.md`. A test that shows a named behaviour stays, however much it costs. Moving a test to escape a budget is raising it.

**2. `github-conformance` first: run as much as a test checks.** One test holds every fact on the fake in a whole run, and that stays. The others set one or a few overrides and read one fact. Options: one run carrying several overrides, a run stopped once the fact and what follows it are settled, or a shared helper. "Fails only the fact and continues" must still be shown once. Profile a whole run as well: 325 ms is itself worth explaining.

**3. A test's count is what its behaviour needs.** A loop of a million turns, or a capacity of a megabyte, is kept only if a smaller one wouldn't show the same thing. Show why the smaller one still does, by what the test asserts: for a ring, that it still wraps at least once; for a ceiling, that the structure is still far larger than the limit it's held to. **Every assertion stays as strict as it was.**

**4. What the rest repeat.** For each row, profile the costliest test and a median one, and fix the cause: setup repeated, data copied or encoded again, a scan where a lookup would do. A cost in the code under test is fixed there if it's small and no behaviour changes, with its own focused test. A person pays that one too, so prefer it.

**5. Report before and after, per row, in instruction counts,** with each cause and whether it's fixed. Causes found and not fixed get a line, with what fixing them would take.

Largest first, and stop once the next cause's gain is under about 0.05 CPU s. Keep tests as they are unless a test's own setup is the cause: rows of table tests may be another plan's, once this one shows what's left. At most about six tasks. `walk = false`. Effort medium.

**Overlaps:** leave `skein_heap::Counting` and `skein_world::stream::Wire` (`skein-world-costs`), `skein-json`'s product code (another plan), and what a process costs before its test starts. Other plans add tests to these crates all the time: expect catch-ups, and keep a helper's change in one task.

## Sources

Read by section, too large to be a source: `skein/docs/foundation/testing-strategy.md`, 8 and 8.1.

- `skein/tests/forge/github/src/tests.rs`
- `skein/tests/forge/github/README.md`
- `skein/tests/lib/Cargo.toml`
- `skein/tests/fail-stop/tests/suite/fail_stop.rs`

## For every task of this plan

- **Only focused and fuzzy run in a plan, and a plan is complete when it lands (the user, 11-06).** Never `tools/tiers/run`, a browser, process or live tier, or one of their stories. No walk, render check or throwaway driver.
- **8.1's order, always** (`skein/docs/foundation/testing-strategy.md`, 8.1): does the test belong in this suite; if it does, can it be made faster, at the cause; if neither, the measurements go in the report and the test stays. What a test checks decides its place, never its cost.
- **Find the cause with a profile before changing anything** (section 8, "A slow test is understood, never hidden"). `valgrind --tool=callgrind` on one test by name gives instruction counts that a loaded machine doesn't move; quote them before and after. CPU seconds on these machines swing by a third or more between runs.
- **No budget, timeout, deadline or build setting changes.** Test builds stay unoptimized. If a profile shows a cost no test can avoid, say so in the report, and leave it.
- **Nothing is lost.** A test that goes names what holds its behaviour now, by test name, in `plan.md`, and that holder exists and passes first. A test past its guide isn't merged with another or named an exception to make room.
- **Decide, don't ask.** A small edit just outside a task's file list is made and named in the report. A `Question:` is only for what truly blocks the task.
- **Before calling a focused or fuzzy failure "outside the change",** run it once on the pristine base. If main passes and your branch fails, it's yours.
