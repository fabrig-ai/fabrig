# budget-close: the four stories on the merged trunk

2026-11-06 UTC; commit `865cd995698a`;
16 CPUs, AMD EPYC-Genoa Processor. The branch and integration ref both named
this commit before measuring. All levers' decisions are merged: ctl history,
the smaller conversation window, and the decisions to leave find backoff and
delivery caching out.

CPU is user plus system for the whole test process tree, from
`tools/tiers/cpu-summary --json`. Compilation, listing, prerequisites and the
supervisor are excluded (`tools/tiers/design.md`, 7.3). Wall is the individual
test wall, not the command's prerequisite time. Loads are the one-, five- and
fifteen-minute averages from the machine running the test, immediately before
and after each tier command, including prerequisites; they are not CPU usage.

Each command selects exactly one test and one attempt:

```sh
tools/tiers/run --cpu target/browser-budget/final-LABEL-N temper browser -- -E 'test(=browser_pages::STORY)'
```

| Label | Exact story name within `browser_pages` |
|---|---|
| long | `a_long_chat_opens_at_its_end_and_reads_back_from_the_composer` |
| latest | `latest_returns_a_chat_to_its_end_after_new_words_while_reading_back` |
| earlier | `earlier_words_stay_put_without_browser_scroll_anchoring` |
| short | `a_short_chat_opens_whole_without_a_reading_status` |

## Three comparison runs each

The order is long, latest, earlier, short, repeated three times. The first
long run also has the initial sampler; it remains in the medians. There is
no source counter or production-code instrumentation.

| Story | Arm | Run | CPU s (user + system) | Wall s | Load before | Load after | Result |
|---|---|---:|---:|---:|---|---|---|
| long | merged | 1 | 17.642224 | 14.266906 | 30.16 / 11.18 / 7.62 | 19.49 / 14.31 / 9.33 | passed; `final-long-1.log` |
| latest | merged | 1 | 9.208361 | 4.880247 | 19.49 / 14.31 / 9.33 | 13.78 / 13.38 / 9.16 | passed; `final-latest-1.log` |
| earlier | merged | 1 | 8.671831 | 4.309206 | 13.78 / 13.38 / 9.16 | 10.04 / 12.53 / 8.99 | passed; `final-earlier-1.log` |
| short | merged | 1 | 6.451679 | 2.615083 | 10.04 / 12.53 / 8.99 | 8.63 / 12.06 / 8.91 | passed; `final-short-1.log` |
| long | merged | 2 | 21.731308 | 14.605676 | 8.63 / 12.06 / 8.91 | 6.13 / 11.08 / 8.70 | passed; `final-long-2.log` |
| latest | merged | 2 | 9.178371 | 5.026417 | 6.13 / 11.08 / 8.70 | 6.37 / 10.72 / 8.64 | passed; `final-latest-2.log` |
| earlier | merged | 2 | 8.399148 | 3.999198 | 6.37 / 10.72 / 8.64 | 6.69 / 10.48 / 8.62 | passed; `final-earlier-2.log` |
| short | merged | 2 | 6.414108 | 2.757858 | 6.69 / 10.48 / 8.62 | 5.08 / 9.87 / 8.45 | passed; `final-short-2.log` |
| long | merged | 3 | 20.858273 | 14.087272 | 5.08 / 9.87 / 8.45 | 4.43 / 9.18 / 8.28 | passed; `final-long-3.log` |
| latest | merged | 3 | 9.048547 | 4.623730 | 4.43 / 9.18 / 8.28 | 3.60 / 8.60 / 8.11 | passed; `final-latest-3.log` |
| earlier | merged | 3 | 3.576509 | 3.697623 | 3.60 / 8.60 / 8.11 | 4.50 / 8.46 / 8.07 | passed; `final-earlier-3.log` |
| short | merged | 3 | 4.997871 | 2.395595 | 4.50 / 8.46 / 8.07 | 8.14 / 9.03 / 8.27 | passed; `final-short-3.log` |

**Medians:** CPU and wall medians are calculated independently from the three
comparison runs per story.

| Story | Median CPU s | Median wall s | Target CPU s | Result |
|---|---:|---:|---:|---|
| long | 20.858273 | 14.266906 | about 12 | target not met |
| latest | 9.178371 | 4.880247 | about 6 | target not met |
| earlier | 8.399148 | 3.999198 | about 5 | target not met |
| short | 6.414108 | 2.615083 | none | reported; no target |

All twelve comparison runs passed; no test was starved or unobserved, and no
process tree was leaked or incompletely accounted for. Earlier's third CPU
sample, 3.576509 s, is retained alongside 8.671831 and 8.399148 s; short's
third sample, 4.997871 s, likewise remains alongside 6.451679 and 6.414108 s.
The spread is not grounds to discard a run. The four CPU medians sum to
44.849900 s; no whole-tier run was made.

## Long story attribution

The initial sampler found the first long story but overwrote process-family
labels as processes exited. It sampled 16.22 of that run's 17.642224 CPU s
(91.9%), but reliably retained only the test binary's 2.32 s and ctl's 0.19 s.
The other 13.71 s could not be reliably divided between Chromium, serving and
helpers. That is a failed attribution, not measured zero CPU for those families.

Two additional long-story runs are attribution diagnostics, separate from the
three-run comparison medians. The first retained labels but grouped
`temper internal serve` with other children. Its preliminary categories were
renderer 4.07, other Chromium 2.31, outer serving 0.35, ctl 0.38, test 1.79 and
unresolved serving/helpers 4.31 CPU s: 13.21 of 14.479078 s (91.2%). The final
sampler recognizes internal serving separately. Neither diagnostic replaces a
comparison run or measures another lever.

| Story | Arm | Run | CPU s (user + system) | Wall s | Load before | Load after | Result |
|---|---|---:|---:|---:|---|---|---|
| long | split diagnostic | 1 | 14.479078 | 11.036599 | 9.53 / 9.30 / 8.37 | 10.41 / 9.50 / 8.47 | passed; `split-long-1.log` |
| long | final classified split | 1 | 13.415869 | 11.970142 | 16.20 / 11.04 / 9.02 | 30.43 / 15.19 / 10.50 | passed; `classified-long-1.log` |

The throwaway sampler reads each process's own user and system ticks from
`/proc/PID/stat`, nominally every 10 ms, at 100 ticks per second. It follows
all spawning threads' child lists from the selected test binary, remembers
PID and start time, and keeps the maximum observed ticks. Executable names
and command arguments identify families; a disappearing command line keeps
its previously observed family. Renderer is Chromium's `--type=renderer`;
other Chromium includes the browser, zygotes and utility processes. Serving
includes `temper box` and `temper internal serve`; ctl is separate from
helpers and agents. No process environment or credential file is read.

| Process family | Sampled CPU s |
|---|---:|
| Chromium renderer | 4.26 |
| Chromium other | 2.24 |
| temper serving | 1.68 |
| temper ctl | 0.15 |
| other children | 1.96 |
| test binary | 1.75 |
| Sampled total | 12.04 |
| Whole-tree CPU | 13.415869 |
| Unassigned | 1.375869 |

The final split accounts for **12.04 / 13.415869 CPU s
(89.7%)**, leaving 1.375869 s unassigned. Short-lived children
and final ticks can fall between samples. Other children include agents,
fakes and setup helpers; their sampled `pkill` share is
0.24 s.
This is a process-family split, not a function profile. The attribution run's
13.415869 CPU s is below the comparison median of 20.858273 s;
the split is not scaled to that median, and does not establish which renderer
functions or accessibility queries dominate it.

All fourteen executed stories passed with exactly one selected test and one
attempt each, no starvation, unobserved tests, leaks or incomplete accounting.
No other story or whole tier ran. Logs named above, JSON summaries, collector
records and JUnit are retained beside the run report under `measurements/`.
The samplers and `target/browser-budget/` are removed before commit.
