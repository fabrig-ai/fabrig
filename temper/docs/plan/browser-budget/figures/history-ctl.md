# history-ctl: histories prepared before the reader opens

2026-11-06. Before: commit `32c015e965`, 18:17–18:36 UTC, recorded as 16 CPUs,
AMD EPYC-Genoa Processor. Latest's three passing before runs are the task answer's
reference, [story-baseline.md](story-baseline.md), 18:12–18:19 UTC on 8 CPUs of the
same model. The failed local baseline remains below; its two missing repetitions
were not run, as the answer directs.

After: 19:31:00–19:43:08 UTC, on 8 CPUs, AMD EPYC-Genoa Processor, with the
history implementation `64dafc5857`, listener repair `c592067aa0`, and pure ctl
constructor `6e89d5c2f2`, with latest's opening wait fixed in `f0bdd20ed8`, on
integration base `c6b91fbab0`. Long, earlier and short are unchanged across that
last fix; their first two after runs precede it. The first after run
of the long story supplies the final classified CPU split.
The differing recorded CPU counts and the load are part of this comparison.

Each measured invocation selected exactly one story by its qualified name through
`heavy python3 -I -B tools/tiers/run --cpu DIR temper browser -- -E FILTER`.
CPU is user plus system for the whole test tree; prerequisites and compilation are
excluded. Load columns are the one-, five- and fifteen-minute averages. No run was
starved. Before runs preceded the first source edit, so the arms could not be
alternated; after repetitions rotate among the stories. No budget, deadline,
reply size, window setting or `READING_MESSAGES` changed.

- **long:** `a_long_chat_opens_at_its_end_and_reads_back_from_the_composer`.
- **latest:** `latest_returns_a_chat_to_its_end_after_new_words_while_reading_back`.
- **earlier:** `earlier_words_stay_put_without_browser_scroll_anchoring`.
- **short:** `a_short_chat_opens_whole_without_a_reading_status`.

| Story | Arm | Run | CPU s (user + system) | Wall s | Load before | Load after | Result |
|---|---|---:|---:|---:|---|---|---|
| long | before | 1 | 37.722038 | 22.377607 | 4.08 / 13.92 / 19.00 | 3.52 / 12.40 / 18.24 | passed |
| long | before | 2 | 37.068353 | 21.842051 | 3.52 / 12.40 / 18.24 | 4.54 / 11.42 / 17.64 | passed |
| long | before | 3 | 37.742639 | 22.020055 | 4.54 / 11.42 / 17.64 | 5.57 / 10.81 / 17.16 | passed |
| latest | before (failed local attempt) | 1 | 16.668209 | 16.447417 | 5.57 / 10.81 / 17.16 | 16.43 / 12.60 / 17.48 | failed: reader missed incoming paragraph after both tab signals; `before-latest_returns_a_chat_to_its_end_after_new_words_while_reading_back-1.log` |
| earlier | before | 1 | 7.420224 | 5.560072 | 20.60 / 13.98 / 17.78 | 19.71 / 14.33 / 17.78 | passed |
| earlier | before | 2 | 8.318314 | 4.766546 | 19.71 / 14.33 / 17.78 | 19.42 / 14.85 / 17.85 | passed |
| earlier | before | 3 | 8.389864 | 4.625810 | 19.42 / 14.85 / 17.85 | 17.52 / 14.78 / 17.74 | passed |
| short | before | 1 | 5.155677 | 2.352053 | 17.52 / 14.78 / 17.74 | 15.41 / 14.58 / 17.60 | passed |
| short | before | 2 | 3.779673 | 2.623958 | 15.41 / 14.58 / 17.60 | 14.11 / 14.29 / 17.42 | passed |
| short | before | 3 | 4.216724 | 3.232838 | 14.11 / 14.29 / 17.42 | 20.68 / 15.91 / 17.87 | passed |
| latest | before (reference) | 1 | 12.769298 | 5.336897 | 2.49 / 5.27 / 10.33 | 2.33 / 5.01 / 10.11 | passed |
| latest | before (reference) | 2 | 12.480270 | 5.329647 | 2.21 / 4.37 / 9.42 | 2.07 / 4.16 / 9.21 | passed |
| latest | before (reference) | 3 | 12.020166 | 5.198036 | 3.61 / 4.14 / 8.74 | 4.55 / 4.35 / 8.69 | passed |
| long | after | 1 | 23.707820 | 18.070390 | 3.90 / 4.43 / 6.64 | 8.26 / 5.66 / 6.95 | passed |
| long | after | 2 | 20.698928 | 27.531035 | 17.56 / 10.05 / 8.38 | 19.96 / 12.21 / 9.24 | passed |
| long | after | 3 | 21.952863 | 17.308406 | 18.93 / 17.20 / 12.52 | 15.13 / 16.54 / 12.52 | passed |
| latest | after | 1 | 6.461383 | 5.429844 | 19.16 / 15.84 / 11.35 | 21.23 / 17.09 / 12.00 | passed |
| latest | after | 2 | 5.573748 | 5.521919 | 17.17 / 16.74 / 12.22 | 18.93 / 17.20 / 12.52 | passed |
| latest | after | 3 | 5.118337 | 4.177332 | 14.19 / 16.08 / 12.58 | 14.16 / 15.92 / 12.62 | passed |
| earlier | after | 1 | 5.866731 | 7.253129 | 8.26 / 5.66 / 6.95 | 11.18 / 6.62 / 7.21 | passed |
| earlier | after | 2 | 5.775053 | 7.507890 | 19.88 / 12.32 / 9.30 | 20.70 / 13.47 / 9.81 | passed |
| earlier | after | 3 | 3.575603 | 3.864213 | 15.13 / 16.54 / 12.52 | 14.54 / 16.30 / 12.55 | passed |
| short | after | 1 | 3.912371 | 5.777852 | 11.18 / 6.62 / 7.21 | 15.95 / 8.40 / 7.79 | passed |
| short | after | 2 | 3.532472 | 4.083622 | 20.70 / 13.47 / 9.81 | 21.11 / 14.49 / 10.31 | passed |
| short | after | 3 | 3.341053 | 2.379821 | 14.54 / 16.30 / 12.55 | 14.19 / 16.08 / 12.58 | passed |

**Medians (three passing runs in each arm):**

| Story | Before CPU s | After CPU s | Before wall s | After wall s | CPU reduction |
|---|---:|---:|---:|---:|---:|
| long | 37.722038 | 21.952863 | 22.020055 | 18.070390 | 41.8% |
| latest | 12.480270 | 5.573748 | 5.329647 | 5.429844 | 55.3% |
| earlier | 8.318314 | 5.775053 | 4.766546 | 7.253129 | 30.6% |
| short | 4.216724 | 3.532472 | 2.623958 | 4.083622 | 16.2% |

The shared CPU targets are 12 seconds for long, 6 for latest and 5 for earlier;
the table gives the measured medians. Latest's after wall times are 5.429844, 5.521919, 4.177332 seconds.
All three passed the unchanged per-call deadline and the held-position assertion.
The whole tier was not run and no whole-tier budget result is claimed.

## Intermediate measurements retained

These are the first implementation's measurements, before the listener repair;
they are not included in the final after medians.

| Story | Arm | Run | CPU s (user + system) | Wall s | Load before | Load after | Result |
|---|---|---:|---:|---:|---|---|---|
| long | ctl before listener repair | 1 | 27.864329 | 19.239707 | 9.29 / 17.49 / 18.54 | 5.89 / 15.58 / 17.85 | passed |
| latest | ctl before listener repair | 1 | 8.596913 | 14.367402 | 9.39 / 13.44 / 16.49 | 7.50 / 12.43 / 16.02 | failed: incoming ctl exceeded WITHIN at exit; empty stderr; `after-latest_returns_a_chat_to_its_end_after_new_words_while_reading_back-1.log` |
| earlier | ctl before listener repair | 1 | 8.956908 | 4.827397 | 5.89 / 15.58 / 17.85 | 4.39 / 14.28 / 17.35 | passed |
| short | ctl before listener repair | 1 | 5.342507 | 2.202364 | 4.92 / 13.91 / 17.17 | 8.95 / 14.04 / 17.12 | passed |

The first listener-repaired series preceded extracting pure command construction for
unit tests without a built binary. These passing runs remain recorded; the final
medians above use the repeated series on the pure constructor.

| Story | Arm | Run | CPU s (user + system) | Wall s | Load before | Load after | Result |
|---|---|---:|---:|---:|---|---|---|
| long | before pure constructor | 1 | 23.967314 | 17.919549 | 4.51 / 8.72 / 10.25 | 5.16 / 8.42 / 10.08 | passed |
| long | before pure constructor | 2 | 24.023623 | 17.237609 | 3.94 / 7.65 / 9.75 | 3.42 / 7.06 / 9.46 | passed |
| long | before pure constructor | 3 | 24.404902 | 17.941262 | 5.40 / 7.08 / 9.28 | 4.55 / 6.66 / 9.04 | passed |
| latest | before pure constructor | 1 | 9.751455 | 12.784291 | 6.82 / 10.42 / 10.90 | 4.57 / 9.11 / 10.42 | passed |
| latest | before pure constructor | 2 | 8.235388 | 4.753111 | 7.39 / 7.56 / 9.49 | 5.40 / 7.08 / 9.28 | passed |
| latest | before pure constructor | 3 | 9.586729 | 12.737905 | 7.33 / 7.11 / 9.07 | 5.49 / 6.67 / 8.85 | passed |
| earlier | before pure constructor | 1 | 7.620160 | 4.332893 | 5.16 / 8.42 / 10.08 | 4.69 / 8.05 / 9.91 | passed |
| earlier | before pure constructor | 2 | 6.451837 | 4.722953 | 3.42 / 7.06 / 9.46 | 6.09 / 7.35 / 9.48 | passed |
| earlier | before pure constructor | 3 | 5.676652 | 4.249919 | 4.55 / 6.66 / 9.04 | 6.72 / 6.98 / 9.09 | passed |
| short | before pure constructor | 1 | 3.903177 | 1.957541 | 4.69 / 8.05 / 9.91 | 3.94 / 7.65 / 9.75 | passed |
| short | before pure constructor | 2 | 4.457093 | 2.609048 | 6.09 / 7.35 / 9.48 | 7.39 / 7.56 / 9.49 | passed |
| short | before pure constructor | 3 | 3.919904 | 2.083035 | 6.72 / 6.98 / 9.09 | 7.33 / 7.11 / 9.07 | passed |

## Development failures

| Story | Trial | CPU s | Wall s | Load before | Load after | Finding / log |
|---|---|---:|---:|---|---|---|
| long | development-reader-sign-in | 11.133751 | 9.203243 | 31.25 / 20.92 / 19.50 | 28.73 / 21.45 / 19.73 | Navigation before sign-in settled: unauthorized stream. Fixed by waiting for the welcome, choosing the playground, then observing Owner. `development-reader-sign-in.log` |
| long | development-reader-welcome | 16.702608 | 19.367146 | 25.90 / 21.57 / 19.83 | 13.33 / 18.88 / 19.00 | Owner is absent on the first-run welcome. Fixed by joining the already-running playground before navigating. `development-reader-welcome.log` |
| latest | development-latest-opening | 9.157580 | 14.281231 | 6.33 / 12.66 / 16.52 | 20.12 / 15.66 / 17.34 | PageUps preceded completion of the opening history; Latest absent. Fixed by observing the oldest scripted message before scrolling. `development-latest-opening.log` |

Two setup invocations ran no story: the first rejected a nonempty CPU directory, and the second used an unqualified exact test name and selected zero tests (exit 4). They are not measurements.

## Listener observation and repair

The incoming command reached `before-incoming` but failed at the command's exit
(step 17). A throwaway file counter in `Server::up`, removed before commit,
recorded **12 held connections, capacity 12, three in calls, none closing**, then
one rejected incoming connection. That run exited 1 with `temper ctl: unreachable`.
The nine remaining connections had no routed call; the counter did not distinguish
partial heads. The repair tracks the first request bytes separately, so a partial
head is never eligible for replacement.

The task notes hypothesized eight connections. The observed deployment instead
uses `temper-service`'s `jig.clients_http.calls.max(8)`, which resolves to 12 here.
That expression and the configured limit remain unchanged.

A full listener now aborts one idle connection and holds the accepted socket until
io reports the old connection closed, then binds the new one. Its bounded map never
grows; the listener itself waits for that bind or rejection before another accept.
Calls and partial requests remain held. If the listener stops meanwhile, it rejects
the waiting socket. A graceful-close trial still exceeded WITHIN while admission
waited for the peer; immediate idle ending is the passing repair. The 30-second
stall and every caller deadline remain unchanged.

| Latest diagnostic | CPU s | Wall s | Load before | Load after | Finding |
|---|---:|---:|---|---|---|
| counter sent to stderr | 5.902125 | 15.920278 | 15.28 / 8.94 / 8.57 | 20.77 / 13.38 / 10.28 | exit deadline; counter absent from captured report |
| counter sent to stdout | 6.780898 | 14.378135 | 18.40 / 13.21 / 10.27 | unknown | exit deadline; report still omitted counts |
| observe-latest-4 | 5.246533 | 5.894332 | 10.66 / 12.91 / 10.72 | 17.80 / 14.66 / 11.56 | private file counter: listener full; exit 1 unreachable |
| after-latest-1 | 7.536193 | 21.918102 | 11.18 / 13.72 / 11.85 | 9.05 / 12.46 / 11.56 | graceful idle close: incoming exit deadline |

The first file-counter invocation failed to compile for the playground's wasm
build because its Unix imports were not guarded; it ran no story. Guarding the
throwaway instrumentation fixed that setup error. Diagnostic logs and the counts
are retained with this continuation's report; inherited failure logs retain the
basenames recorded above. No failed run was rerun unchanged.

## Opening history observation

A repeat after the pure constructor failed before scrolling: the initial chat
sentence was absent. The snapshot already held Reading paragraphs 1–6 and their
answers, with no reading status. An opening reads only until its head is far
enough above the viewport; this does not require the initial chat sentence.
Latest now waits for Reading paragraph 1, which gates the history its two PageUps
need. The PageUps, word-2 position measurements, incoming ctl call, Latest and
sending assertions are unchanged. The long story still reads all the way to its
start. No deadline was extended.

| Story | Arm | Run | CPU s (user + system) | Wall s | Load before | Load after | Result |
|---|---|---:|---:|---:|---|---|---|
| latest | before opening wait repair | verified-latest-1 | 4.159309 | 15.926574 | 15.23 / 8.37 / 7.78 | 17.56 / 10.05 / 8.38 | passed |
| latest | before opening wait repair | verified-latest-2 | 5.542358 | 15.195359 | 21.11 / 14.49 / 10.31 | 19.75 / 15.03 / 10.69 | failed |

`verified-latest-2.log` records the missing initial sentence, not a ctl timeout.

## Long-story CPU split

The final classified sample reads only `/proc/PID/stat` and process labels of the
story's descendants, at 50 ms intervals. CPU is each sampled process's maximum
user plus system ticks, grouped by executable and the retained Chromium `--type`.
Keeping a process's observed renderer type matters: Chromium can rewrite its
command line. Earlier preliminary samples did not retain that type and are kept as
whole-tree measurements below, without claiming their provisional family split.

| Story | Arm | Run | CPU s (user + system) | Wall s | Load before | Load after | Result |
|---|---|---:|---:|---:|---|---|---|
| long | after, additional sample | preliminary split | 24.653050 | 18.338401 | 3.90 / 5.95 / 8.47 | 3.43 / 5.39 / 8.08 | passed |
| long | after, additional sample | first classified split | 24.586733 | 18.461697 | 9.55 / 6.68 / 8.19 | 6.30 / 6.24 / 7.97 | passed |
| long | after, final classified sample | 1 (also above) | 23.707820 | 18.070390 | 3.90 / 4.43 / 6.64 | 8.26 / 5.66 / 6.95 | passed |

| Family | Sampled CPU s |
|---|---:|
| Chromium renderer | 7.15 |
| Chromium other | 2.31 |
| temper serving | 2.04 |
| temper ctl | 0.32 |
| other children | 6.27 |
| test binary | 4.15 |
| Sampled total | 22.24 |
| Whole-tree accounting | 23.707820 |
| Not accounted for by samples | 1.467820 |

The split accounts for **93.8%** of the whole tree. Short-lived
children can leave between samples, and sampling can miss a process's last ticks.
Other children group temper helpers and agents with sampled `pkill`
commands. This is a process-family split, not a function profile.

Chromium still takes 9.46 sampled CPU seconds, the test binary 4.15,
and other children 6.27; serving itself takes 2.04 and ctl 0.32.
The remaining work includes drawing and querying the whole-sized window and its
helper processes. This sample does not establish which renderer functions or
accessibility queries dominate. The next lever owns the smaller window.
