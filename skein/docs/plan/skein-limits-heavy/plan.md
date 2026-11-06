# skein-limits-heavy: skein's costliest focused tests, by 8.1's questions

The request: `inputs/request.md`. The repair of `skein/docs/foundation/testing-strategy.md`, 8.1, in its order, for the costliest work above each process's own cost, aiming for at least 6 CPU s of a focused suite that reads 38.2 against 30. No budget, timeout or build setting changes. `walk = false`.

This file is brought up to date with each batch the cut pushes.

## Parts and tasks

| Part | Task | Needs | Rows | CPU s |
|---|---|---|---|---:|
| `github` | `tasks/github-runs.md` | | `github-conformance` `tests` | 4.59 |
| `lib-memory` | `tasks/lib-counts.md` | | `skein-lib-tests::suite` `memory` | 1.56 |

**Critical path, as it stands:** either task alone. Neither needs the other. `github-runs` is the longest, at about 100 minutes.

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

## Tests that go

Each is named with its holder, which exists and passes first.

- `github_conformance_missing_delivery_fails_only_its_fact`: held by `github_conformance_faults_fail_only_their_facts_and_the_run_continues`. Section 8 allows it: the whole run is the fixture and the cost, and each fault's check costs little beside it.
- `github_conformance_null_merged_commit_fails_the_fact_and_continues`: held by the same test, for the same reason.

## Overlaps

Neither task is on files another draft is changing. `skein-world-costs` holds `skein_heap::Counting`, which `memory` meters with: `lib-counts` leaves it alone. `json-strings` holds `skein-json`'s product code, which a github run parses with: `github-runs` leaves it alone.
