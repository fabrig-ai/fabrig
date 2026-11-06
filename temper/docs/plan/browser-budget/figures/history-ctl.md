# history-ctl: histories prepared before the reader opens

2026-11-06 18:17–18:36 UTC; before commit `32c015e965`, implementation commit `64dafc5857`; 16 CPUs, AMD EPYC-Genoa Processor.

This is a partial measurement: the incoming ctl call with the page open exceeded its unchanged ten-second deadline. The task stops for a scope decision. Missing runs and the CPU split are unobserved.

All measured runs selected exactly one test, by `test(=browser_pages::NAME)`, through `heavy python3 -I -B tools/tiers/run --cpu DIR temper browser -- -E FILTER`. Load columns are the one-, five- and fifteen-minute load averages.

- **long:** `a_long_chat_opens_at_its_end_and_reads_back_from_the_composer`.
- **latest:** `latest_returns_a_chat_to_its_end_after_new_words_while_reading_back`.
- **earlier:** `earlier_words_stay_put_without_browser_scroll_anchoring`.
- **short:** `a_short_chat_opens_whole_without_a_reading_status`.

Before runs precede every source edit, as the task requires; the arms were therefore not alternated. Failed runs are retained below. Their sanitized logs are saved beside the task report under the basenames shown. The measurement scratch directory is deleted before push.

| Story | Arm | Run | CPU s (user + system) | Wall s | Load before | Load after | Result |
|---|---|---:|---:|---:|---|---|---|
| long | before | 1 | 37.722038 | 22.377607 | 4.08 / 13.92 / 19.00 | 3.52 / 12.40 / 18.24 | passed |
| long | before | 2 | 37.068353 | 21.842051 | 3.52 / 12.40 / 18.24 | 4.54 / 11.42 / 17.64 | passed |
| long | before | 3 | 37.742639 | 22.020055 | 4.54 / 11.42 / 17.64 | 5.57 / 10.81 / 17.16 | passed |
| latest | before | 1 | 16.668209 | 16.447417 | 5.57 / 10.81 / 17.16 | 16.43 / 12.60 / 17.48 | failed: reader missed incoming paragraph after both tab signals; `before-latest_returns_a_chat_to_its_end_after_new_words_while_reading_back-1.log` |
| latest | before | 2 | — | — | — | — | not run: no unchanged rerun after failure |
| latest | before | 3 | — | — | — | — | not run: no unchanged rerun after failure |
| earlier | before | 1 | 7.420224 | 5.560072 | 20.60 / 13.98 / 17.78 | 19.71 / 14.33 / 17.78 | passed |
| earlier | before | 2 | 8.318314 | 4.766546 | 19.71 / 14.33 / 17.78 | 19.42 / 14.85 / 17.85 | passed |
| earlier | before | 3 | 8.389864 | 4.625810 | 19.42 / 14.85 / 17.85 | 17.52 / 14.78 / 17.74 | passed |
| short | before | 1 | 5.155677 | 2.352053 | 17.52 / 14.78 / 17.74 | 15.41 / 14.58 / 17.60 | passed |
| short | before | 2 | 3.779673 | 2.623958 | 15.41 / 14.58 / 17.60 | 14.11 / 14.29 / 17.42 | passed |
| short | before | 3 | 4.216724 | 3.232838 | 14.11 / 14.29 / 17.42 | 20.68 / 15.91 / 17.87 | passed |
| long | after | 1 | 27.864329 | 19.239707 | 9.29 / 17.49 / 18.54 | 5.89 / 15.58 / 17.85 | passed |
| long | after | 2 | — | — | — | — | not run: stopped at ctl blocker |
| long | after | 3 | — | — | — | — | not run: stopped at ctl blocker |
| latest | after | 1 | 8.596913 | 14.367402 | 9.39 / 13.44 / 16.49 | 7.50 / 12.43 / 16.02 | failed: incoming ctl exceeded WITHIN at exit; empty stderr; `after-latest_returns_a_chat_to_its_end_after_new_words_while_reading_back-1.log` |
| latest | after | 2 | — | — | — | — | not run: no unchanged rerun after failure |
| latest | after | 3 | — | — | — | — | not run: no unchanged rerun after failure |
| earlier | after | 1 | 8.956908 | 4.827397 | 5.89 / 15.58 / 17.85 | 4.39 / 14.28 / 17.35 | passed |
| earlier | after | 2 | — | — | — | — | not run: stopped at ctl blocker |
| earlier | after | 3 | — | — | — | — | not run: stopped at ctl blocker |
| short | after | 1 | 5.342507 | 2.202364 | 4.92 / 13.91 / 17.17 | 8.95 / 14.04 / 17.12 | passed |
| short | after | 2 | — | — | — | — | not run: stopped at ctl blocker |
| short | after | 3 | — | — | — | — | not run: stopped at ctl blocker |

**Before medians (three passing runs):**

- long: 37.722038 CPU s, 22.020055 wall s.
- earlier: 8.318314 CPU s, 4.766546 wall s.
- short: 4.216724 CPU s, 2.623958 wall s.

Latest has one failed baseline, so no passing before median. No after median is reported: long, earlier and short each have one passing run; latest failed. The long story’s single after result is below its before median but above the shared 12-CPU-second target. Earlier is above its shared 5-second target; latest’s 6-second target cannot be assessed from a failing run.

## Development failures

| Story | Trial | CPU s | Wall s | Load before | Load after | Finding / log |
|---|---|---:|---:|---|---|---|
| long | development-reader-sign-in | 11.133751 | 9.203243 | 31.25 / 20.92 / 19.50 | 28.73 / 21.45 / 19.73 | Navigation before sign-in settled: unauthorized stream. Fixed by waiting for the welcome, choosing the playground, then observing Owner. `development-reader-sign-in.log` |
| long | development-reader-welcome | 16.702608 | 19.367146 | 25.90 / 21.57 / 19.83 | 13.33 / 18.88 / 19.00 | Owner is absent on the first-run welcome. Fixed by joining the already-running playground before navigating. `development-reader-welcome.log` |
| latest | development-latest-opening | 9.157580 | 14.281231 | 6.33 / 12.66 / 16.52 | 20.12 / 15.66 / 17.34 | PageUps preceded completion of the opening history; Latest absent. Fixed by observing the oldest scripted message before scrolling. `development-latest-opening.log` |

Two setup invocations ran no story: the first rejected a nonempty CPU directory, and the second used an unqualified exact test name and selected zero tests (exit 4). They are not measurements.

## Incoming ctl finding

The reader reached `before-incoming` after two PageUps, with its composer focused and Latest present. The subsequent `task message 1` command missed its expected exit: elapsed 10.000160215 s for a 10-second allowance, thread runnable wait 0.009044937 s, thread CPU 0.133167674 s, tree runnable wait 0 s. Its stderr was empty. Load was 9.39 / 13.44 / 16.49 before the run and 7.50 / 12.43 / 16.02 after. This was not reported as starvation.

`after-incoming`, the held-position comparison, Latest after incoming and sending from far back were not reached in this run. The existing assertions and steps remain. The listener-connection hypothesis in the task notes has not been established as the cause. WITHIN remains unchanged.

## CPU split

Unobserved: stopped at the acceptance-blocking ctl deadline before the three-run after arm and the long-story process-family sampling. No renderer/server/ctl attribution is claimed.
