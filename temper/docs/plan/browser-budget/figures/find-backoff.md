# find-backoff: repeated-miss backoff does not pay on the repaired stories

2026-11-06, comparison base `d42854c5ea` (the task branch after merging
`window-story` commit `d8c8d7ff90`); 16 CPUs, AMD EPYC-Genoa Processor.
All 24 comparison runs use this base. The final fetch and merge of
`origin/browser-budget/window-story` were already up to date; the story,
bundle and face inputs did not change.

A is the base's semantic-find polling. B applies the request's fixed
50, 100, 200, 200 ms repeated-miss backoff, with a press's re-find kept at
50 ms, a successful find clearing `missing`, and a person's act clearing
`missing` and shortening a later `look_again` to at most one poll away.
A temporary `FIND_BACKOFF` environment switch selected the arm in one
binary. A marker in `BrowserFace::ask`, immediately before each emitted
`Request::Find`, counted accessibility queries and their role index in
both arms. Both the switch and candidate code were removed after measuring.

Each story ran A B A B A B. Every run selected exactly one test, one
attempt, and passed, with no starvation, leaks or incomplete accounting.
CPU is user plus system for the whole test process tree, excluding
prerequisite builds. Loads are the one-, five- and fifteen-minute averages,
read on the machine running the test. The command was
`heavy env FIND_BACKOFF=ARM bash -lc '… python3 -I -B tools/tiers/run --cpu DIR temper browser -- -E FILTER --success-output immediate …'`,
with load readings around the tier command. Each `FILTER` was
`test(=browser_pages::STORY)`:

- **long:** `a_long_chat_opens_at_its_end_and_reads_back_from_the_composer`.
- **latest:** `latest_returns_a_chat_to_its_end_after_new_words_while_reading_back`.
- **earlier:** `earlier_words_stay_put_without_browser_scroll_anchoring`.
- **short:** `a_short_chat_opens_whole_without_a_reading_status`.

## Old-base failure, excluded from every median

The first run used base `1ba0412483`, incorporating `window-story` commit
`7485300d7b`, before the script-admission repair. Its failed A/1 row is
retained; it did not reach a semantic find and counts in no comparison.

| Story | Arm | Run | CPU s (user + system) | Wall s | Load before | Load after | Result |
|---|---|---:|---:|---:|---|---|---|
| long | A | 1 | 1.022407 | 0.761347 | 36.49 / 19.73 / 16.39 | 55.91 / 34.64 / 22.67 | failed at person 0, step 0: scripts past their joint byte bound; first run's `A-long-1.log` |

Accessibility queries: **0**. This startup failure is not a measure of
the long story. The repaired dependency's passing after arm is recorded
in [window-story.md](window-story.md).

## Comparison on the repaired base

| Story | Arm | Run | CPU s (user + system) | Wall s | Load before | Load after | Result |
|---|---|---:|---:|---:|---|---|---|
| long | A | 1 | 19.200691 | 12.820851 | 1.35 / 3.64 / 6.89 | 8.80 / 4.92 / 6.85 | passed |
| long | B | 1 | 20.986008 | 13.664994 | 8.80 / 4.92 / 6.85 | 6.26 / 4.71 / 6.71 | passed |
| long | A | 2 | 19.570480 | 12.813974 | 6.26 / 4.71 / 6.71 | 5.82 / 4.75 / 6.66 | passed |
| long | B | 2 | 15.834527 | 12.451371 | 5.82 / 4.75 / 6.66 | 8.95 / 5.64 / 6.88 | passed |
| long | A | 3 | 20.412939 | 13.261065 | 8.95 / 5.64 / 6.88 | 6.53 / 5.35 / 6.75 | passed |
| long | B | 3 | 20.613688 | 13.680584 | 6.53 / 5.35 / 6.75 | 5.12 / 5.15 / 6.63 | passed |
| latest | A | 1 | 9.054425 | 4.787608 | 5.12 / 5.15 / 6.63 | 3.84 / 4.85 / 6.49 | passed |
| latest | B | 1 | 9.289662 | 4.940550 | 3.84 / 4.85 / 6.49 | 5.10 / 5.08 / 6.52 | passed |
| latest | A | 2 | 8.933560 | 4.999002 | 5.10 / 5.08 / 6.52 | 4.10 / 4.85 / 6.41 | passed |
| latest | B | 2 | 9.124544 | 4.885144 | 4.10 / 4.85 / 6.41 | 4.24 / 4.79 / 6.34 | passed |
| latest | A | 3 | 9.224583 | 5.060394 | 4.24 / 4.79 / 6.34 | 4.16 / 4.71 / 6.27 | passed |
| latest | B | 3 | 9.409560 | 5.181985 | 4.16 / 4.71 / 6.27 | 4.16 / 4.68 / 6.22 | passed |
| earlier | A | 1 | 8.785078 | 4.185837 | 4.16 / 4.68 / 6.22 | 4.51 / 4.71 / 6.19 | passed |
| earlier | B | 1 | 8.277866 | 4.415538 | 4.51 / 4.71 / 6.19 | 4.92 / 4.76 / 6.16 | passed |
| earlier | A | 2 | 8.786493 | 4.197398 | 4.92 / 4.76 / 6.16 | 4.05 / 4.56 / 6.06 | passed |
| earlier | B | 2 | 7.938130 | 3.816918 | 4.05 / 4.56 / 6.06 | 3.37 / 4.38 / 5.97 | passed |
| earlier | A | 3 | 7.757885 | 4.170259 | 3.37 / 4.38 / 5.97 | 4.29 / 4.54 / 5.98 | passed |
| earlier | B | 3 | 7.067404 | 3.893208 | 4.29 / 4.54 / 5.98 | 4.54 / 4.59 / 5.96 | passed |
| short | A | 1 | 5.701520 | 2.425084 | 4.54 / 4.59 / 5.96 | 5.42 / 4.82 / 6.00 | passed |
| short | B | 1 | 5.610472 | 2.614682 | 5.42 / 4.82 / 6.00 | 6.06 / 5.02 / 6.03 | passed |
| short | A | 2 | 6.300151 | 2.555752 | 6.06 / 5.02 / 6.03 | 4.62 / 4.75 / 5.92 | passed |
| short | B | 2 | 4.394034 | 2.338714 | 4.62 / 4.75 / 5.92 | 10.51 / 6.15 / 6.36 | passed |
| short | A | 3 | 5.859718 | 2.399687 | 10.51 / 6.15 / 6.36 | 8.37 / 5.96 / 6.29 | passed |
| short | B | 3 | 4.332537 | 3.466765 | 8.37 / 5.96 / 6.29 | 15.99 / 7.84 / 6.90 | passed |

## Medians and decision

| Story | A CPU s | B CPU s | B / A CPU | A wall s | B wall s | B / A wall | Wall at most 105% |
|---|---:|---:|---:|---:|---:|---:|---|
| long | 19.570480 | 20.613688 | 105.33% | 12.820851 | 13.664994 | 106.58% | no |
| latest | 9.054425 | 9.289662 | 102.60% | 4.999002 | 4.940550 | 98.83% | yes |
| earlier | 8.785078 | 7.938130 | 90.36% | 4.185837 | 3.893208 | 93.01% | yes |
| short | 5.859718 | 4.394034 | 74.99% | 2.425084 | 2.614682 | 107.82% | no |

**Decision: do not land the patch.** The long story's B median CPU is
105.33% of A, above the required 90%. B is lower only in pair 2 (1 of 3,
below the required 2). Its median wall is 106.58% of A, above 105%.
Short also fails its wall check at 107.82%; Latest and earlier pass.
Neither required condition holds;
thresholds and backoff intervals were not tuned. Only these figures land;
no browser code, tests or design sentence changes.

## Accessibility queries

| Story | A/1 | B/1 | A/2 | B/2 | A/3 | B/3 |
|---|---:|---:|---:|---:|---:|---:|
| long | 67 | 69 | 63 | 65 | 69 | 68 |
| latest | 38 | 31 | 41 | 32 | 41 | 32 |
| earlier | 25 | 25 | 27 | 25 | 25 | 26 |
| short | 15 | 14 | 15 | 13 | 14 | 13 |

The long story's A counts are 67, 63 and 69 (median 67); B's are 69, 65
and 68 (median 68). **All 401 long-story queries have role index zero;
zero are extra accessible-role probes.** Thus the per-role absent-search
cost does not dominate these long-story runs. Counts do not show a query
reduction on the repaired fixture; the earlier 519-query observation is
not this base. The marker counts face requests, not every CDP command
inside a request.

Logs `ARM-STORY-N.log`, machine-readable summaries `ARM-STORY-N.json`,
and raw CPU collector records under `cpu/ARM-STORY-N/` are retained beside
the resumed task's report. The old failure's log belongs to the first run.
The temporary driver, switch, query markers, candidate code and
`target/browser-budget/` are removed before push.
