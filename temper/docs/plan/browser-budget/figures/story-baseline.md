# story-baseline: four browser stories before repair

2026-11-06, 18:12:20–18:19:15 UTC; commit `32c015e965da`;
8 CPUs, AMD EPYC-Genoa Processor.

The unmodified checkout supplies the twelve baseline runs. Each command selects
one story in `browser-temper`, using
`tools/tiers/run --cpu target/browser-budget/before-<story>-<n> temper browser -- -E 'test(=browser_pages::<story>)'`.
The stories ran in the order long, latest, earlier, short, repeated three times.
CPU is user plus system for the whole test process tree; wall is the individual
test wall from `tools/tiers/cpu-summary --json`. Compilation, prerequisites,
listing and the supervisor are excluded (`tools/tiers/design.md`, 7.3).

Loads are `/proc/loadavg`’s 1-, 5- and 15-minute averages, recorded immediately
before and after the tier command, including its prerequisites. Every valid run
selected exactly one test and one attempt. All thirteen story runs passed,
with no starvation, unobserved attempts, leaks or incomplete CPU accounting.

| Story label | Exact name within `browser_pages` |
|---|---|
| long | `a_long_chat_opens_at_its_end_and_reads_back_from_the_composer` |
| latest | `latest_returns_a_chat_to_its_end_after_new_words_while_reading_back` |
| earlier | `earlier_words_stay_put_without_browser_scroll_anchoring` |
| short | `a_short_chat_opens_whole_without_a_reading_status` |

| Story | Arm | Run | CPU s (user + system) | Wall s | Load before | Load after | Result |
|---|---|---:|---:|---:|---|---|---|
| long | before | 1 | 35.850555 | 21.666180 | 3.23 / 5.82 / 10.76 | 2.49 / 5.27 / 10.33 | passed |
| long | before | 2 | 34.250733 | 20.586934 | 2.88 / 4.81 / 9.80 | 2.21 / 4.37 / 9.42 | passed |
| long | before | 3 | 34.876193 | 20.957697 | 2.95 / 4.11 / 8.95 | 3.61 / 4.14 / 8.74 | passed |
| latest | before | 1 | 12.769298 | 5.336897 | 2.49 / 5.27 / 10.33 | 2.33 / 5.01 / 10.11 | passed |
| latest | before | 2 | 12.480270 | 5.329647 | 2.21 / 4.37 / 9.42 | 2.07 / 4.16 / 9.21 | passed |
| latest | before | 3 | 12.020166 | 5.198036 | 3.61 / 4.14 / 8.74 | 4.55 / 4.35 / 8.69 | passed |
| earlier | before | 1 | 7.993236 | 4.481255 | 2.33 / 5.01 / 10.11 | 3.54 / 5.06 / 9.99 | passed |
| earlier | before | 2 | 8.214616 | 4.360570 | 2.07 / 4.16 / 9.21 | 2.58 / 4.12 / 9.06 | passed |
| earlier | before | 3 | 7.834560 | 4.720400 | 4.55 / 4.35 / 8.69 | 4.97 / 4.47 / 8.61 | passed |
| short | before | 1 | 4.193832 | 2.162703 | 3.54 / 5.06 / 9.99 | 2.88 / 4.81 / 9.80 | passed |
| short | before | 2 | 3.574879 | 2.134704 | 2.58 / 4.12 / 9.06 | 2.95 / 4.11 / 8.95 | passed |
| short | before | 3 | 5.044884 | 2.531651 | 4.97 / 4.47 / 8.61 | 5.12 / 4.57 / 8.53 | passed |

**Medians:** independent medians of the three CPU and three wall measurements
per story. The request’s figures came from the whole-tier run on `668f7cd3e503`;
these figures are stories run alone, without a claim about whole-tier cost.

| Story | Before median CPU s | Before median wall s | Request CPU s | Request wall s |
|---|---:|---:|---:|---:|
| long | 34.876193 | 20.957697 | 35.2 | 21.0 |
| latest | 12.480270 | 5.329647 | 12.7 | 5.6 |
| earlier | 7.993236 | 4.481255 | 7.9 | 4.3 |
| short | 4.193832 | 2.162703 | 4.3 | 2.1 |

## Instrumented long story

One additional run used a temporary counter in `BrowserFace::ask` and a
temporary process sampler. It is separate from the baseline medians; both
instrumentation overhead and the different load may affect its timing.

| Story | Arm | Run | CPU s (user + system) | Wall s | Load before | Load after | Result |
|---|---|---:|---:|---:|---|---|---|
| long | before, sampler + counter | 1 | 30.424502 | 22.410618 | 4.35 / 4.43 / 8.42 | 7.70 / 5.20 / 8.47 | passed; sampler captured no processes |

The collector reports 24.588061 user s and 5.836441 system s.

### Process families: 0% sampler coverage

The sampler was configured to read each process’s own `utime + stime` from
`/proc/<pid>/stat` every 20 ms, with 100 clock ticks per second, keeping the
largest observed counters per PID and process start time. It traversed each
process’s main-thread children, looking for the selected test executable and
its exact test argument before sampling its descendants. That search did not
locate the test process: it collected zero samples and observed no processes.
The family values below are unobserved, rather than measured zeros.

| Process family | Sampled CPU s |
|---|---:|
| Chromium renderer | unobserved |
| Chromium other | unobserved |
| temper serving | unobserved |
| temper ctl and other children | unobserved |
| test binary | unobserved |

**Accounted for: 0.000000 / 30.424502 CPU s (0%).**
The remaining 30.424502 CPU s (100%) is unassigned.
This is below the task’s 80% threshold. Following its decided option, the
sampler was neither improved nor rerun. CPU attribution to process families
remains open; the whole-tree collector’s accounting is complete.

### Accessibility queries

`BrowserFace::ask` dispatched **167** accessibility `Request::Find` queries.
All **167** completed with a matching terminal event; none remained unmatched.
A monotonic timestamp was taken immediately before queuing each request,
and elapsed time was recorded when the matching `Found` or `Refused` event
reached the face. This includes queueing, browser work and delivery through
the harness; it excludes the sleeps between queries and is wall time, not CPU.

| Query wall measurement | Seconds |
|---|---:|
| Sum | 15.725366527 |
| Mean | 0.094163871 |
| Median | 0.070389824 |
| Minimum | 0.004304818 |
| Maximum | 0.722563758 |

## Selection and retained evidence

An initial command used the short exact story name from the measurement
template. Nextest selected zero tests and exited 4. Its zero CPU and wall
prove nothing and are excluded from the story measurements. Using the exact
`browser_pages::`-qualified name selected one test in every subsequent run.

The run report retains the JSON measurements, collector records, JUnit and
command logs under `measurements/`: `before-<label>-<n>/command-output.txt`,
`instrumented-long-1/command-output.txt`, and
`selection-empty/command-output.txt`. There were no failed or starved story
runs to omit. No other browser stories or whole tier ran.

The counter source was restored byte for byte. The sampler, counter backup
and `target/browser-budget/` were removed before committing.
