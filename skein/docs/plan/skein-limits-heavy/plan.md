# skein-limits-heavy: skein's costliest focused tests, by 8.1's questions

The request: `inputs/request.md`. The repair of `skein/docs/foundation/testing-strategy.md`, 8.1, in its order, for the costliest work above each process's own cost, aiming for at least 6 CPU s of a focused suite that reads 38.2 against 30. No budget, timeout or build setting changes. `walk = false`.

This file is brought up to date with each batch the cut pushes.

## Parts and tasks

| Part | Task | Needs | Rows | CPU s |
|---|---|---|---|---:|
| `github` | `tasks/github-runs.md` | | `github-conformance` `tests` | 4.59 |
| `lib-memory` | `tasks/lib-counts.md` | | `skein-lib-tests::suite` `memory` | 1.56 |
| `sweeps` | `tasks/sweeps.md` | | `skein-ui` `tests::date`, `tests::phrases`; `skein-json` `tests::text`; `skein-http` `tests::gzip` | 0.41 |

**Critical path, as it stands:** any task alone. None needs another. `github-runs` is the longest, at about 100 minutes.

## What this plan decided

- **Each task writes its own figures file** (`figures/<task>.md`, in `notes/common.md`'s form), so that no two branches meet in a table. A last task builds the per-row table from them.
- **The stop rule is in CPU s, the measure in instructions.** A cause's share of a test's inclusive instructions, times that test's CPU s in the request's table, gives its gain, against the request's 0.05.
- **`github-conformance` carries several faults in one whole run.** The request offers three shapes, and each has its place:
  - **One run with several overrides** shows "fails only the fact and continues" once: `github_conformance_faults_fail_only_their_facts_and_the_run_continues`.
  - **A shared helper's group alone** holds the rows that read one fact's outcome: framing runs `cases::logs`, epoch a new `cases::issues_since_epoch`.
  - **A run stopped early** isn't used. The faulted facts sit late in a run (`logs` is fifth of six groups), so stopping saves little of what comes before them, and it would need a sentinel through `complete`.
- **The request counts fourteen tests running `cases::run`; the code has fewer.** Seven tests run a whole `cases::run`, ten times between them, counting `run_fake`'s. An eighth runs `cases::queue` eleven times. At the request's 325 ms a run and its figures for the named tests, those account for about 4.35 of the row's 4.59 s, so the plan holds whatever the count.

## Belonging

`skein/docs/foundation/testing-strategy.md`, 8 and 8.1, are the test. What a test checks decides its place, never its cost. This is a cut's reading of the design and the tests, not a run.

- **`github-conformance`'s tests stay.** Each shows a named behaviour on the fake, or a refusal of the live path, from fixed inputs: section 8's "each world's scenarios". None is a sweep or a comparison.
- **`skein-lib-tests`' `memory` stays.** Section 8 names "memory under the scenarios' ceilings" for the focused suite. The three loops are one use repeated, not many generated cases, so their cost is a count, under the request's item 3, and not a place.
- **`every_wall_day_round_trips_at_midnight` moves to fuzzy.** It sweeps all 213,503 days a `Wall` can hold for one property, and names none of them. That's section 8's "sweeps of many generated cases". `known_midnights_follow_gregorian_centuries` holds the behaviour on named days in focused, and gains the one thing only the sweep held there: a day's last nanosecond is still that day. It moves with every assertion.
- **`every_representable_wall_day_matches_gregorian_rollover` moves to fuzzy.** It compares the phrasebook's date, day by day, with a counter of months and leap years written in the test: section 8's "a function against a naive one", over every day. `dates_and_moments_use_utc_calendar_words_in_every_wall_format` holds named days in focused (leap days, a century's March 1, a year's end). It moves with every assertion.
- **`three_and_four_byte_sequences_are_judged_at_every_boundary` stays, at a smaller count.** Its name is a behaviour: every range limit of the well-formed table. It compares with the standard library, but its point is the limits, and those are `edges`. `Utf8::next` judges by `(low..=high).contains(&byte)`, so a byte between two limits can't be judged otherwise than its limits are. The full second byte added 25 times the cases and no limit. The rest can't move as it is: `Utf8` is private to `skein-json`, whose product code is `json-strings'`, and a public item for a test isn't wanted. `fuzzy::writer::text_is_written_exactly_when_it_is_utf8_and_reads_back_the_same` already holds random bytes between the limits, through the public path.
- **`a_huffman_code_is_complete_canonical_and_as_short_as_any` stays, at four seeds.** It's a seeded comparison with a plain tree builder, so section 8 would have it in fuzzy, but `huffman` is `pub(crate)` and no public path reaches its widths. It stays as a scenario, and section 8 gives a scenario "the few seeds that show its behaviour": two for each kind of counts, each seed still running every check.

## Tests that go

Each is named with its holder, which exists and passes first.

- `github_conformance_missing_delivery_fails_only_its_fact`: held by `github_conformance_faults_fail_only_their_facts_and_the_run_continues`. Section 8 allows it: the whole run is the fixture and the cost, and each fault's check costs little beside it.
- `github_conformance_null_merged_commit_fails_the_fact_and_continues`: held by the same test, for the same reason.

## Overlaps

`sweeps` meets `limits-fixtures/view-words` in `skein/crates/skein-ui/src/tests/phrases.rs`, where it deletes one test and nothing else, and could not avoid it: that's where the test is. The other tasks are on no file another draft lists. `skein-world-costs` holds `skein_heap::Counting`, which `memory` meters with: `lib-counts` leaves it alone. `json-strings` holds `skein-json`'s product code, which a github run parses with: `github-runs` leaves it alone.
