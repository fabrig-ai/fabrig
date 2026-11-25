# Browser CPU baseline: 2026-11-25

## Result

**121.236472 CPU s** is the sum of the twelve isolated test medians,
**6.263528 s below** the 127.5 CPU s in `tools/tiers/budgets.toml`
(`[browser-temper]`). This is not a whole-tier run or evidence that the tier
passes its budget when run together. The after-landing figures that motivated
the measurement were 147.3 and 144.3 CPU s for twelve passing stories at
`a3a3878002`, at 02:26 and 02:33 UTC. They are not a before/after comparison
with these isolated runs.

**Machine:** builder W4, 8 logical CPUs, AMD EPYC-Genoa Processor.
**Base:** `d8e8c48ccad48bffdf35db3eb27278e7f09b6979`.
**Browser:** Chromium 147.0.7727.101 (Debian 13).
**Tools:** Linux 6.12.117, Rust 1.101.0-nightly (19833af72), nextest 0.9.148.
Use these figures as a comparison only on the same host; otherwise measure
this revision on the comparison host.

## Isolated attempts

The unoptimized dev web bundle and native binaries came from the bundle and
binary lists in `tools/tiers/bundles.toml` and `tools/tiers/packages.toml`.
The playground used `temper/scripts/bundle.sh --playground` and its own
profiles. Bundle builds were outside the timed attempts. With those bundles
and Chromium supplied to the tests, each command was `cargo nextest run
--profile default --ignore-default-filter -p temper-client-browser-world
--test browser_all --test-threads 1 --retries 0 -E 'test(=MODULE::TEST)'`,
with a private nextest config enabling a run wrapper. Each selected one test.
No story, timeout or membership changed.

The wrapper was `tools/tiers/cpu-wrapper.c`, built as in
`tools/tiers/design.md`, 7. It accounts user and system CPU of the test and
its waited-and-reaped descendants, excluding Cargo, nextest and the wrapper
itself. Wall is elapsed time through settlement. Round 3 also used tracing
for attribution. All 36 table attempts passed and settled, with no leaked
children reported. Two setup attempts are outside the table: a sign-in run
passed before the accounting wrapper was enabled; a journeys run passed but
its load envelope was interrupted. Neither replaces a timed entry.

Each cell is **user / system / total CPU / wall**, in seconds, rounded to four
places. The medians are independent; the median of total CPU is not the sum
of the user and system medians. Numbers in the first column identify the
same stories in the tables below.

| # | Test (`module::test`) | Run 1 | Run 2 | Run 3 (traced) | Median |
| ---: | --- | ---: | ---: | ---: | ---: |
| 1 | `browser_sign_in::signing_in_reloading_reconnecting_and_signing_out_keep_the_cookie_contract` | 16.0031 / 2.8708 / 18.8738 / 15.8740 | 16.5746 / 2.9590 / 19.5336 / 16.7974 | 16.3237 / 2.8487 / 19.1724 / 16.1487 | 16.3237 / 2.8708 / 19.1724 / 16.1487 |
| 2 | `browser_pages::history_keeps_the_chat_focus_and_filter_across_reload_and_back` | 8.8099 / 3.8843 / 12.6942 / 7.3782 | 9.5187 / 3.7425 / 13.2613 / 7.5059 | 8.9197 / 3.4337 / 12.3533 / 7.1165 | 8.9197 / 3.7425 / 12.6942 / 7.3782 |
| 3 | `browser_pages::enter_starts_one_chat_and_the_composer_keeps_the_focus` | 6.0266 / 2.6209 / 8.6475 / 6.5788 | 5.5218 / 2.6048 / 8.1266 / 6.3415 | 5.6077 / 2.5521 / 8.1598 / 5.1256 | 5.6077 / 2.6048 / 8.1598 / 6.3415 |
| 4 | `browser_attacks::cross_origin_requests_and_framing_cannot_act_as_the_person` | 4.2440 / 2.2472 / 6.4912 / 3.3103 | 4.1708 / 2.1425 / 6.3133 / 3.4719 | 4.5682 / 2.2004 / 6.7686 / 3.4308 | 4.2440 / 2.2004 / 6.4912 / 3.4308 |
| 5 | `browser_authorize::a_cross_site_authorization_returns_a_code_and_the_links_state` | 13.6623 / 2.0985 / 15.7608 / 12.9571 | 13.5973 / 1.9609 / 15.5582 / 13.9566 | 13.7383 / 2.0024 / 15.7407 / 13.0400 | 13.6623 / 2.0024 / 15.7407 / 13.0400 |
| 6 | `browser_accounts::the_sign_in_opens_a_tab_and_lands_on_accounts` | 7.1748 / 3.3026 / 10.4775 / 6.5364 | 7.5331 / 3.1699 / 10.7030 / 8.1162 | 7.0596 / 3.1785 / 10.2381 / 6.1135 | 7.1748 / 3.1785 / 10.4775 / 6.5364 |
| 7 | `browser_github::the_manifest_posts_once_and_history_waits_for_continue` | 1.4698 / 0.8608 / 2.3306 / 1.6464 | 1.4564 / 0.9931 / 2.4495 / 1.8153 | 1.5212 / 1.0457 / 2.5669 / 1.8883 | 1.4698 / 0.9931 / 2.4495 / 1.8153 |
| 8 | `browser_playground::story_a_walks_on_the_built_playground` | 10.8648 / 1.7864 / 12.6512 / 11.2536 | 10.8034 / 1.7378 / 12.5412 / 11.1137 | 11.2778 / 1.8608 / 13.1386 / 11.5372 | 10.8648 / 1.7864 / 12.6512 / 11.2536 |
| 9 | `browser_public::public_board_and_task_lines_are_readable_at_desktop_and_phone_widths` | 3.3282 / 1.9793 / 5.3075 / 3.9264 | 3.4145 / 2.1657 / 5.5802 / 4.1027 | 3.2593 / 2.0057 / 5.2649 / 3.9434 | 3.3282 / 2.0057 / 5.3075 / 3.9434 |
| 10 | `browser_pages::a_chat_opens_at_its_end_and_reads_back_keeping_its_place` | 10.0390 / 5.3286 / 15.3676 / 9.9663 | 10.7236 / 5.1341 / 15.8577 / 10.3709 | 10.8139 / 5.6369 / 16.4508 / 10.9892 | 10.7236 / 5.3286 / 15.8577 / 10.3709 |
| 11 | `browser_connect_flows::scrolling_the_sheet_reads_more_then_search_picks_and_adopts` | 4.3305 / 2.0663 / 6.3968 / 3.8222 | 4.1593 / 1.8492 / 6.0085 / 3.5954 | 4.5613 / 2.0996 / 6.6609 / 3.9566 | 4.3305 / 2.0663 / 6.3968 / 3.8222 |
| 12 | `browser_journeys::a_browser_journey_looks_and_a_tui_journey_is_approved_in_chromium` | 4.0737 / 1.6039 / 5.6776 / 4.3211 | 4.6099 / 1.6249 / 6.2348 / 4.5969 | 4.2537 / 1.5844 / 5.8381 / 4.6098 | 4.2537 / 1.6039 / 5.8381 / 4.5969 |

Load is the 1/5/15-minute host average before and after each nextest command,
not CPU utilization. Times are command starts in UTC on 2026-11-25.

| Story | Run 1 | Run 2 | Run 3 |
| ---: | --- | --- | --- |
| 1 | 03:04:11 4.48/4.06/3.05 → 4.45/4.08/3.07 | 03:06:28 4.71/4.45/3.34 → 5.90/4.72/3.45 | 03:08:19 6.68/5.58/3.89 → 6.91/5.69/3.97 |
| 2 | 03:04:28 4.45/4.08/3.07 → 4.39/4.07/3.08 | 03:06:46 5.90/4.72/3.45 → 6.16/4.81/3.49 | 03:08:36 6.91/5.69/3.97 → 6.99/5.75/4.00 |
| 3 | 03:04:37 4.39/4.07/3.08 → 5.47/4.32/3.17 | 03:06:55 6.16/4.81/3.49 → 6.31/4.86/3.52 | 03:08:45 6.99/5.75/4.00 → 6.99/5.77/4.02 |
| 4 | 03:04:45 5.47/4.32/3.17 → 5.47/4.32/3.17 | 03:07:03 6.31/4.86/3.52 → 8.04/5.25/3.65 | 03:09:04 6.22/5.65/4.00 → 5.96/5.61/3.99 |
| 5 | 03:04:49 5.47/4.32/3.17 → 5.01/4.27/3.17 | 03:07:07 8.04/5.25/3.65 → 8.09/5.41/3.73 | 03:09:08 5.96/5.61/3.99 → 5.07/5.43/3.96 |
| 6 | 03:05:03 5.01/4.27/3.17 → 5.42/4.38/3.22 | 03:07:22 8.09/5.41/3.73 → 9.30/5.76/3.86 | 03:09:23 5.07/5.43/3.96 → 5.15/5.43/3.98 |
| 7 | 03:05:11 5.42/4.38/3.22 → 5.42/4.38/3.22 | 03:07:32 9.30/5.76/3.86 → 9.04/5.76/3.87 | 03:09:39 4.74/5.34/3.96 → 4.84/5.35/3.97 |
| 8 | 03:05:14 5.42/4.38/3.22 → 4.59/4.25/3.20 | 03:07:35 9.04/5.76/3.87 → 8.33/5.72/3.88 | 03:09:42 4.84/5.35/3.97 → 4.84/5.34/3.98 |
| 9 | 03:05:26 4.59/4.25/3.20 → 4.86/4.31/3.22 | 03:07:47 8.33/5.72/3.88 → 7.91/5.67/3.88 | 03:09:55 4.84/5.34/3.98 → 5.02/5.36/4.00 |
| 10 | 03:05:31 4.86/4.31/3.22 → 4.57/4.26/3.22 | 03:07:52 7.91/5.67/3.88 → 7.75/5.72/3.91 | 03:10:00 5.02/5.36/4.00 → 5.24/5.40/4.03 |
| 11 | 03:05:42 4.57/4.26/3.22 → 5.97/4.56/3.32 | 03:08:04 7.75/5.72/3.91 → 7.45/5.69/3.91 | 03:10:12 5.24/5.40/4.03 → 5.30/5.41/4.04 |
| 12 | 03:06:21 4.95/4.49/3.34 → 4.71/4.45/3.34 | 03:08:09 7.45/5.69/3.91 → 7.09/5.64/3.90 | 03:10:17 5.30/5.41/4.04 → 5.20/5.38/4.04 |

## Where the CPU went

For each story, a task-scoped Linux trace instance followed fork, exec and
exit from the nextest wrapper. Summing each descendant thread's
`sched_stat_runtime` deltas gave CPU by process. Chromium was classified by
process type after its zygote forks, not just by executable name. The trace
also placed probes on entry and return of the compiled fixture, harness and
playground functions. CPU deltas spanning a phase boundary were apportioned
across it by elapsed time. The reader ran outside the test tree and did not
collect URLs. Tracing adds some work to the measured processes. These are
one-run allocations, not allocations of the medians.

The round-3 trace is the source for stories 7–12. Stories 1–6 use one extra
passing trace each: the earlier classifier missed Chromium's role in rewritten
process titles. The extra attempts do not enter the medians. The sample and
wrapper clocks are separate; the last column is trace CPU divided by the
wrapper's CPU for the same attempt, not a correction factor. No trace
reported event loss.

| # | Fixture other | Install | Bootstrap | Seed in harness | Rest of harness | After: replay | After: gzip | Other after/between | Trace CPU s | Wrapper CPU s | Trace / wrapper |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 0.049 | 0.828 | 0 | 0 | 18.325 | 0 | 0 | 0.013 | 19.216 | 19.268 | 99.7% |
| 2 | 0.103 | 0.000 | 0.958 | 1.348 | 9.981 | 0 | 0 | 0.019 | 12.409 | 12.450 | 99.7% |
| 3 | 0.051 | 0.816 | 0 | 0 | 6.975 | 0 | 0 | 0.014 | 7.855 | 7.892 | 99.5% |
| 4 | 0.043 | 0.830 | 0 | 0 | 5.809 | 0 | 0 | 0.009 | 6.691 | 6.731 | 99.4% |
| 5 | 0.186 | 0.826 | 0 | 0 | 14.696 | 0 | 0 | 0.008 | 15.717 | 15.734 | 99.9% |
| 6 | 0.064 | 0.750 | 0 | 0 | 8.870 | 0 | 0 | 0.008 | 9.692 | 9.673 | 100.2% |
| 7 | 0.054 | 0.876 | 0 | 0 | 1.597 | 0 | 0 | 0.009 | 2.536 | 2.567 | 98.8% |
| 8 | 0.026 | 0 | 0 | 0 | 8.893 | 2.710 | 1.331 | 0.068 | 13.028 | 13.139 | 99.2% |
| 9 | 0.171 | 0.000 | 1.004 | 2.806 | 1.233 | 0 | 0 | 0.017 | 5.230 | 5.265 | 99.3% |
| 10 | 0.243 | 0.000 | 0.994 | 4.999 | 10.028 | 0 | 0 | 0.025 | 16.289 | 16.451 | 99.0% |
| 11 | 0.051 | 0.852 | 0 | 0 | 5.676 | 0 | 0 | 0.009 | 6.588 | 6.661 | 98.9% |
| 12 | 0.022 | 0 | 0 | 0 | 7.730 | 0 | 0 | 0.019 | 7.771 | 5.838 | 133.1% |

The phase columns are CPU seconds. `Install` is the no-task settings sitting
in `configuration::install`; `Bootstrap` is the two sittings in
`scripted::bootstrap`. The standalone fake starts in the fixture, before
bootstrap, and continues in the harness; its lifetime CPU is in the process
split below. `Seed in harness` includes browser and box startup while ctl
prepares the playground/history or public board. It ends at launch of the
link-opening command for history and reading, or at the explicit board
navigation for public reading. The remaining harness column excludes this
seed phase. The reading story has further ctl calls after the reader opens.
The journeys column combines its web and TUI drive sessions (3.103 and
4.628 traced CPU s); the playground replay and gzip are outside its harness.

The same samples, by process family (CPU seconds):

| # | Test binary | Chromium browser | Renderer | GPU | Utility/network | Other Chromium | Box | Serve | Static serve | ctl | Fake LLM | fake-claude | Other children |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 0.738 | 1.249 | 3.521 | 0.670 | 0.064 | 0.105 | 0.139 | 11.409 | 0 | 0.352 | 0 | 0 | 0.968 |
| 2 | 0.841 | 1.501 | 4.751 | 0.606 | 0.068 | 0.101 | 0.138 | 2.289 | 0 | 0.270 | 0.086 | 0 | 1.759 |
| 3 | 0.624 | 1.174 | 3.143 | 0.378 | 0.065 | 0.107 | 0.094 | 1.117 | 0 | 0.342 | 0 | 0 | 0.812 |
| 4 | 0.448 | 1.023 | 2.793 | 0.366 | 0.065 | 0.104 | 0.093 | 0.940 | 0 | 0.337 | 0 | 0 | 0.521 |
| 5 | 0.388 | 0.931 | 1.855 | 0.244 | 0.061 | 0.122 | 0.149 | 11.099 | 0 | 0.303 | 0 | 0 | 0.565 |
| 6 | 0.735 | 1.538 | 3.965 | 0.591 | 0.075 | 0.105 | 0.086 | 1.336 | 0 | 0.312 | 0 | 0 | 0.950 |
| 7 | 0.252 | 0.500 | 0.256 | 0.079 | 0.056 | 0.090 | 0.099 | 0.702 | 0 | 0.367 | 0 | 0 | 0.135 |
| 8 | 3.440 | 0.818 | 6.823 | 0.231 | 0.059 | 0.096 | 0 | 0 | 0.047 | 0 | 0 | 0 | 1.513 |
| 9 | 0.396 | 0.524 | 0.676 | 0.129 | 0.056 | 0.084 | 0.137 | 1.381 | 0 | 0.571 | 0.074 | 0 | 1.202 |
| 10 | 2.008 | 1.281 | 4.540 | 0.585 | 0.070 | 0.099 | 0.152 | 3.012 | 0 | 1.619 | 0.247 | 0 | 2.675 |
| 11 | 0.575 | 1.032 | 2.268 | 0.331 | 0.067 | 0.104 | 0.093 | 1.280 | 0 | 0.378 | 0 | 0 | 0.460 |
| 12 | 1.682 | 1.098 | 2.072 | 0.332 | 0.109 | 0.173 | 0.250 | 0.732 | 0 | 0.045 | 0.050 | 0 | 1.227 |

`Box` and `Serve` group by command, not executable name. Other children
include Git, shell helpers, gzip, agent processes and the box's launch
trampolines. No `fake-claude` process was observed.

**Journeys accounting discrepancy.** The round-3 trace accounted for 7.771 s
against the wrapper's 5.838 s. A separate passing cross-check gave 7.781 s by
scheduler trace, 7.15 s by polling and summing each observed process's own
`/proc/PID/stat` user and system ticks, and 5.918 s by the wrapper. The proc
poll omits the tails of processes that exit between polls, so it is not a
complete second total. The difference is reproducible; the measurements do not establish its cause. The phase and family tables retain the trace measurement, and
the medians retain the same wrapper accounting for all twelve stories. The
wrapper total is not a reliable complete-tree total for this story.

Extra attribution and cross-check attempts, with UTC start, wrapper CPU,
wall, and 1/5/15-minute load before and after:

| Story | Start | CPU s | Wall s | Before → after |
| ---: | --- | ---: | ---: | --- |
| 1 | 03:10:28 | 19.2675 | 16.2420 | 4.94/5.33/4.03 → 4.74/5.24/4.03 |
| 2 | 03:10:46 | 12.4501 | 6.9041 | 4.74/5.24/4.03 → 4.68/5.22/4.03 |
| 3 | 03:10:54 | 7.8923 | 4.8037 | 4.68/5.22/4.03 → 4.46/5.17/4.01 |
| 4 | 03:11:00 | 6.7307 | 3.5162 | 4.46/5.17/4.01 → 4.43/5.15/4.01 |
| 5 | 03:11:05 | 15.7339 | 12.8472 | 4.43/5.15/4.01 → 3.89/4.99/3.98 |
| 6 | 03:11:19 | 9.6734 | 5.7082 | 3.89/4.99/3.98 → 3.99/4.98/3.99 |
| 1 | 03:11:33 | 19.1898 | 16.0380 | 3.91/4.95/3.98 → 3.62/4.83/3.96 |
| 5 | 03:11:50 | 15.5646 | 12.7356 | 3.62/4.83/3.96 → 3.48/4.74/3.94 |
| 12 | 03:12:04 | 5.9183 | 4.3698 | 3.48/4.74/3.94 → 3.44/4.71/3.94 |

## Observed starts and loads

Starts come from execs in the same attribution samples, excluding init and
launch trampolines. Count one Chromium launch per browser PID, not once for
its shell wrapper and again for the browser executable. Page loads are the
`Page.loadEventFired` notifications observed by the driver, including initial
loads it saw; same-document navigation and back/forward cache restoration
are not counted as loads.

| # | Box starts | Serve starts | Static host starts | ctl processes | Settings box starts¹ | Settings ctl² | Chromium launches | Page loads |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 3 | 3 | 0 | 13 | 1 | 11 | 1 | 14 |
| 2 | 3 | 3 | 0 | 10 | 2 | 7 | 1 | 15 |
| 3 | 2 | 2 | 0 | 13 | 1 | 11 | 1 | 11 |
| 4 | 2 | 2 | 0 | 13 | 1 | 11 | 1 | 10 |
| 5 | 3 | 3 | 0 | 12 | 1 | 11 | 1 | 8 |
| 6 | 2 | 2 | 0 | 13 | 1 | 11 | 1 | 14 |
| 7 | 2 | 2 | 0 | 14 | 1 | 11 | 1 | 4 |
| 8 | 0 | 0 | 1 | 0 | 0 | 0 | 1 | 1 |
| 9 | 3 | 3 | 0 | 14 | 2 | 8 | 1 | 2 |
| 10 | 3 | 3 | 0 | 46 | 2 | 7 | 1 | 9 |
| 11 | 2 | 2 | 0 | 14 | 1 | 11 | 1 | 9 |
| 12 | 2 | 2 | 0 | 2 | 0 | 0 | 2 | 6 |
| **Total** | 27 | 27 | 1 | 164 | 13 | 99 | 13 | 103 |

¹ Included in box starts. ² Included in ctl processes. Settings here means
`install` and the two bootstrap sittings, including the latter's model
account setup. The start and ctl counts match the earlier read-only survey:
27 boxes with their serve, 13 Chromium launches and 164 ctl processes.
Settings alone used 13 of those boxes and 99 ctl processes, and the two
settings phases accounted for 8.734 traced CPU s.

## Largest costs

1. **Sign-in and reconnect: 19.172425 median CPU s.** In a diagnostic trace,
   `serve` used 10.010 CPU s in the 10.040 wall s from the harness's stop action
   to its next start action. The test binary used only 0.107 CPU s in that
   interval. This is a serving-process cost during the stop wait, not a
   ten-second idle sleep or ten seconds of test-binary polling.
2. **Chat reading: 15.857703 median CPU s.** The attribution sample has
   4.999 CPU s for startup and history prep before opening the reader and
   10.028 CPU s for the remaining harness. There were 26 ctl processes in
   that first phase and 13 more in the remaining harness.
3. **Cross-site authorization: 15.740704 median CPU s.** Its stop-to-start
   wait likewise took 10.023 wall s, of which `serve` used 10.008 CPU s and
   the test binary used 0.087 CPU s. The trace locates the cost in the
   serving process, not the function responsible for it.

Across the twelve attribution samples, the three largest families were
Chromium renderers (36.663 CPU s), `serve` (35.299 CPU s, excluding the
static host), and the Chromium browser process (12.671 CPU s). This is a
sum of trace allocations, including the journeys discrepancy, not a sum
of medians. The playground spent another 4.041 traced CPU s *after* the
harness (2.710 replay and 1.331 gzip), despite having no box start or ctl call.

## After

Both passes were within the **127.5 CPU s** `browser-temper` suite budget.
Pass 1 used **95.524422 CPU s** (31.975578 s below budget); pass 2 used
**94.081936 CPU s** (33.418064 s below budget). Each total sums the twelve
stories' user and system CPU before rounding the individual results.

**Machine:** builder W4, 8 logical CPUs, AMD EPYC-Genoa Processor.
**Browser:** Chromium 147.0.7727.101 (Debian 13). Both passes measured
`0f7e243ee80240258a376578f513d2e991e7a7e1`. The before column reuses the
medians above, measured on this same host at
`d8e8c48ccad48bffdf35db3eb27278e7f09b6979`.

Each pass ran the twelve `temper-client-browser-world::browser_all` stories
in the order below, one exact name per nextest command, on the `default`
profile. The tier's two bundles and native prerequisites were built once
outside timing; the test build was unoptimized. The same C wrapper counted
user and system time of the test and its waited-and-reaped descendants,
excluding Cargo, nextest and the wrapper. Wall time includes settlement.
All 24 attempts passed and settled without reported leaks or retries.

Load is the host's 1/5/15-minute load average, sampled immediately before
and after each command. Times are UTC on 2026-11-25; ranges span all those
samples in the pass.

| Pass | Start–end (UTC) | Start → end load | Load ranges (1 / 5 / 15 min) |
| --- | --- | --- | --- |
| 1 | 03:38:35–03:40:09 | 2.36/1.03/1.72 → 2.87/1.59/1.86 | 1.67–3.68 / 0.98–1.59 / 1.69–1.86 |
| 2 | 03:40:09–03:41:42 | 2.87/1.59/1.86 → 1.61/1.51/1.80 | 1.61–2.87 / 1.45–1.59 / 1.79–1.86 |

Each cell is **user / system / total CPU / wall** in seconds, rounded to
four places. The before column uses independent medians as above.

| # | Test (`module::test`) | Before (W4 median) | Pass 1 | Pass 2 |
| ---: | --- | ---: | ---: | ---: |
| 1 | `browser_sign_in::signing_in_reloading_reconnecting_and_signing_out_keep_the_cookie_contract` | 16.3237 / 2.8708 / 19.1724 / 16.1487 | 5.8332 / 2.5478 / 8.3809 / 15.7984 | 5.7173 / 2.5588 / 8.2762 / 15.6209 |
| 2 | `browser_pages::history_keeps_the_chat_focus_and_filter_across_reload_and_back` | 8.9197 / 3.7425 / 12.6942 / 7.3782 | 8.1267 / 3.4709 / 11.5976 / 6.0747 | 7.9590 / 3.2820 / 11.2410 / 5.9067 |
| 3 | `browser_pages::enter_starts_one_chat_and_the_composer_keeps_the_focus` | 5.6077 / 2.6048 / 8.1598 / 6.3415 | 5.2670 / 2.4237 / 7.6907 / 4.5470 | 5.0158 / 2.1433 / 7.1591 / 4.2381 |
| 4 | `browser_attacks::cross_origin_requests_and_framing_cannot_act_as_the_person` | 4.2440 / 2.2004 / 6.4912 / 3.4308 | 3.9240 / 1.9943 / 5.9183 / 2.8640 | 4.0072 / 2.1125 / 6.1197 / 2.9998 |
| 5 | `browser_authorize::a_cross_site_authorization_returns_a_code_and_the_links_state` | 13.6623 / 2.0024 / 15.7407 / 13.0400 | 3.5458 / 1.8078 / 5.3536 / 12.5963 | 3.4767 / 1.7553 / 5.2320 / 12.4682 |
| 6 | `browser_accounts::the_sign_in_opens_a_tab_and_lands_on_accounts` | 7.1748 / 3.1785 / 10.4775 / 6.5364 | 6.6081 / 3.1796 / 9.7877 / 5.4031 | 6.8027 / 3.2259 / 10.0286 / 5.6177 |
| 7 | `browser_github::the_manifest_posts_once_and_history_waits_for_continue` | 1.4698 / 0.9931 / 2.4495 / 1.8153 | 1.2146 / 0.8185 / 2.0331 / 1.4051 | 1.2881 / 0.8174 / 2.1054 / 1.4129 |
| 8 | `browser_playground::story_a_walks_on_the_built_playground` | 10.8648 / 1.7864 / 12.6512 / 11.2536 | 10.9088 / 1.7099 / 12.6187 / 10.9744 | 10.7214 / 1.8027 / 12.5241 / 10.9318 |
| 9 | `browser_public::public_board_and_task_lines_are_readable_at_desktop_and_phone_widths` | 3.3282 / 2.0057 / 5.3075 / 3.9434 | 3.0157 / 1.8743 / 4.8900 / 3.3981 | 3.0377 / 1.8660 / 4.9037 / 3.4657 |
| 10 | `browser_pages::a_chat_opens_at_its_end_and_reads_back_keeping_its_place` | 10.7236 / 5.3286 / 15.8577 / 10.3709 | 9.9742 / 4.9154 / 14.8895 / 9.4028 | 10.0505 / 4.8698 / 14.9203 / 9.5792 |
| 11 | `browser_connect_flows::scrolling_the_sheet_reads_more_then_search_picks_and_adopts` | 4.3305 / 2.0663 / 6.3968 / 3.8222 | 4.4545 / 1.8929 / 6.3474 / 3.4295 | 4.3702 / 1.7467 / 6.1169 / 3.4127 |
| 12 | `browser_journeys::a_browser_journey_looks_and_a_tui_journey_is_approved_in_chromium` | 4.2537 / 1.6039 / 5.8381 / 4.5969 | 4.3311 / 1.6857 / 6.0168 / 4.4842 | 3.8331 / 1.6219 / 5.4550 / 4.2353 |

Chat reading (10), the playground walk (8), history (2), and accounts (6)
were the four heaviest stories in both passes, in that order. The largest
CPU drops were in sign-in and reconnect (1) and cross-site authorization
(5), each down by more than 10 CPU s with wall time down by less than a
second. This is consistent with the stop wait no longer using a whole
core; the passes measure all the repairs together and ran at lower host
load than the baseline, so they do not isolate any one repair's effect.
Neither pass was within 2.5 CPU s of the budget, so no new process-family
traces were taken. The baseline journeys accounting discrepancy above
was not rechecked.

## Landed repairs: 06:11:05 UTC

**Host:** one Debian 13 builder, AMD EPYC-Genoa Processor, 8 logical CPUs.
**Browser:** Chromium 147.0.7727.101 (Debian 13). **Date:** 2026-11-25 (UTC).

- **Before:** `39d5b23dfc511e9be232761c19e87484a7874be1`, the history repair on main.
- **After:** `92d81dd26b2d1aa0438babf303ddbfbd6669b877`, the landed goal tip.
  The eight landed changes in that tip were 17503 (`d9cedf1c20`),
  17488 (`084cee0a04`), 17502 (`ff96386451`), 17487 (`a35e015ff6`),
  17501 (`278ee7f799`), 17489 (`a2c746b0bd`), 17493 (`d19ddb4b1f`),
  and 17494 (`92d81dd26b`). Chromium flag survey 17495 ended without a
  change: its best candidate, `--disable-breakpad`, saved 0.781 CPU s
  against 1.948 s of spread, and no flag was kept.

Both snapshots were built in separate checkout worktrees before timing:
the dev web bundle, playground in its own profiles, native prerequisites
from the tier's lists, and the dev test binary. The suite ran by exact
name, one nextest invocation per story and one story at a time, with the
`default` profile, the default filter ignored, one test thread and no
retries. `TEMPER_TEST_BUNDLE` and `TEMPER_TEST_PLAYGROUND` pointed to the
respective snapshot's bundles, and each test found the built binaries in
its own target directory. `TEMPER_INTERNAL_BUNDLE=none` kept the native
build from embedding a web bundle.

The same `tools/tiers/cpu-wrapper.c` counted user and system time for the
test and its waited-and-reaped descendants, excluding builds, Cargo,
nextest and the supervisor. Wall includes settlement. A complete record
proves settlement within that scope, not CPU coverage of automatically
reaped descendants (`tools/tiers/design.md`, 7.2). All 72 attempts passed;
nextest's inventory matched one settled record per invocation, with no
retries, starvation, incomplete records or reported leaks.

### Pass totals

Passes ran **Before, After, After, Before, After, After**. Times are UTC;
loads are 1/5/15-minute host averages sampled immediately before and after
each pass. Ranges include samples at both ends of every invocation in the
pass. Timing cells are **user / system / total CPU / wall**, in seconds.
Each pass total sums the unrounded wrapper timings; wall is a sum of story
walls, not the elapsed time between the load samples. Budget comparisons
use the 127.5 CPU s figure in `tools/tiers/budgets.toml` (`[browser-temper]`).

| Pass | Snapshot | UTC window | Load before → after | Within-pass load ranges | Total timing | CPU s below 127.5 |
| ---: | --- | --- | --- | --- | ---: | ---: |
| 1 | Before | 06:11:05–06:12:39 | 3.51/2.59/2.75 → 2.14/2.30/2.62 | 1.75–3.51 / 2.23–2.59 / 2.60–2.75 | 65.753020 / 27.178597 / 92.931617 / 80.202506 | 34.568383 |
| 2 | After | 06:12:39–06:14:12 | 2.14/2.30/2.62 → 2.33/2.32/2.60 | 1.92–2.71 / 2.24–2.41 / 2.57–2.65 | 63.080540 / 26.076338 / 89.156878 / 79.059148 | 38.343122 |
| 3 | After | 06:14:12–06:15:45 | 2.33/2.32/2.60 → 1.29/1.98/2.45 | 1.29–2.33 / 1.98–2.32 / 2.45–2.60 | 64.472745 / 26.130654 / 90.603399 / 79.044579 | 36.896601 |
| 4 | Before | 06:15:45–06:17:19 | 1.29/1.98/2.45 → 1.53/1.85/2.35 | 1.16–1.67 / 1.82–1.98 / 2.35–2.45 | 66.339182 / 27.222925 / 93.562107 / 80.507933 | 33.937893 |
| 5 | After | 06:17:20–06:18:52 | 1.53/1.85/2.35 → 1.81/1.82/2.29 | 1.19–1.86 / 1.73–1.85 / 2.27–2.35 | 63.409868 / 26.109969 / 89.519837 / 78.252887 | 37.980163 |
| 6 | After | 06:18:52–06:20:24 | 1.81/1.82/2.29 → 1.98/1.85/2.25 | 1.63–2.05 / 1.78–1.87 / 2.23–2.29 | 61.964760 / 24.810737 / 86.775497 / 78.317872 | 40.724503 |

All four after passes were below 127.5 CPU s on this builder. Mean pass CPU
was **93.246862 s before** and **89.013903 s after**.
Mean pass system time fell from 27.200761 to 25.781924 s on this host.
The after/before mean CPU ratio was **0.954605**
(4.54% lower). Applying that ratio to the supplied
127.7 CPU s historical tier mean gives **121.90 CPU s**, **not below 118 CPU s**.
This is an **estimate, not a tier run**. It assumes the same proportional
change on the tier machine; neither machine parity nor the tier's execution
pattern is tested by these serial passes.

### Individual attempts

The story numbers are the execution order and match the names in "After"
above. Cells are **user / system / total CPU / wall** in seconds.

| Story | 1 Before | 2 After | 3 After | 4 Before | 5 After | 6 After |
| ---: | ---: | ---: | ---: | ---: | ---: | ---: |
| 1 | 5.5826 / 2.4021 / 7.9847 / 15.8251 | 5.9463 / 2.5119 / 8.4582 / 16.2158 | 5.7598 / 2.4600 / 8.2199 / 15.2621 | 6.4032 / 2.5765 / 8.9797 / 16.0794 | 5.7367 / 2.4745 / 8.2112 / 15.2878 | 5.4885 / 2.3375 / 7.8260 / 15.6777 |
| 2 | 8.2214 / 3.3365 / 11.5579 / 6.2655 | 8.0148 / 3.4013 / 11.4161 / 6.1589 | 7.9461 / 3.3327 / 11.2788 / 6.3256 | 8.1446 / 3.2477 / 11.3923 / 6.2477 | 7.7507 / 3.2182 / 10.9689 / 6.1920 | 7.9150 / 3.1085 / 11.0235 / 5.9504 |
| 3 | 4.9567 / 2.3025 / 7.2591 / 4.4715 | 4.6420 / 2.1933 / 6.8353 / 4.2760 | 4.7871 / 2.2316 / 7.0186 / 4.3391 | 5.0230 / 2.3503 / 7.3733 / 4.5259 | 4.8788 / 2.2368 / 7.1156 / 4.4566 | 5.0755 / 2.3802 / 7.4557 / 4.5780 |
| 4 | 3.9982 / 1.8989 / 5.8970 / 2.9224 | 3.6052 / 1.8932 / 5.4984 / 2.7443 | 3.9819 / 1.8689 / 5.8509 / 2.9159 | 3.8038 / 2.1400 / 5.9439 / 2.9264 | 3.7532 / 2.0262 / 5.7794 / 2.8086 | 3.9975 / 1.8998 / 5.8973 / 2.8656 |
| 5 | 3.4724 / 1.7747 / 5.2471 / 12.5008 | 3.3389 / 1.6695 / 5.0084 / 12.4757 | 3.3249 / 1.5898 / 4.9147 / 12.4090 | 3.4415 / 1.7257 / 5.1672 / 12.5018 | 3.2505 / 1.6671 / 4.9177 / 12.3681 | 3.4195 / 1.5543 / 4.9738 / 12.6345 |
| 6 | 6.6359 / 2.9953 / 9.6313 / 5.5537 | 6.3656 / 2.9042 / 9.2698 / 5.2226 | 6.8902 / 3.0269 / 9.9171 / 5.7622 | 6.6443 / 2.8675 / 9.5118 / 5.6656 | 6.4237 / 2.7365 / 9.1601 / 5.2429 | 6.3427 / 2.7738 / 9.1165 / 5.5512 |
| 7 | 1.2915 / 0.8556 / 2.1471 / 1.4047 | 1.2609 / 0.8068 / 2.0676 / 1.3935 | 1.2689 / 0.8329 / 2.1018 / 1.3583 | 1.3008 / 0.7372 / 2.0380 / 1.4357 | 1.4121 / 0.7838 / 2.1958 / 1.4786 | 1.1724 / 0.7739 / 1.9463 / 1.3382 |
| 8 | 10.9992 / 1.5876 / 12.5868 / 10.9243 | 10.3266 / 1.6533 / 11.9799 / 10.9580 | 10.1012 / 1.5978 / 11.6989 / 10.7943 | 10.8404 / 1.7010 / 12.5414 / 10.9809 | 9.6860 / 1.6152 / 11.3012 / 10.6641 | 9.6466 / 1.3343 / 10.9809 / 10.5364 |
| 9 | 3.0286 / 1.9104 / 4.9390 / 3.4256 | 2.9418 / 1.6840 / 4.6258 / 3.3256 | 2.9267 / 1.8244 / 4.7511 / 3.3447 | 3.0918 / 1.7858 / 4.8776 / 3.3255 | 2.9650 / 1.7549 / 4.7199 / 3.2067 | 2.7506 / 1.7058 / 4.4564 / 3.2226 |
| 10 | 9.7455 / 4.7990 / 14.5444 / 9.3790 | 9.1944 / 4.1242 / 13.3186 / 8.8326 | 9.5827 / 4.1694 / 13.7521 / 8.9405 | 9.6936 / 4.8429 / 14.5365 / 9.2144 | 9.3564 / 4.3145 / 13.6709 / 8.8451 | 9.1340 / 3.9945 / 13.1285 / 8.7440 |
| 11 | 4.0416 / 1.7718 / 5.8134 / 3.3722 | 3.8985 / 1.6807 / 5.5792 / 3.3081 | 4.1142 / 1.5583 / 5.6725 / 3.3380 | 4.2684 / 1.7247 / 5.9931 / 3.3670 | 4.0828 / 1.7801 / 5.8629 / 3.2948 | 3.5763 / 1.5263 / 5.1027 / 3.1992 |
| 12 | 3.7795 / 1.5443 / 5.3237 / 4.1577 | 3.5457 / 1.5539 / 5.0996 / 4.1481 | 3.7891 / 1.6379 / 5.4270 / 4.2550 | 3.6837 / 1.5236 / 5.2073 / 4.2376 | 4.1139 / 1.5022 / 5.6162 / 4.4076 | 3.4461 / 1.4218 / 4.8679 / 4.0202 |

### Story medians

Independent medians in the same four units, ordered by CPU saved. Positive
savings mean lower CPU at the landed tip. Small changes are not evidence
of a per-story effect when the observed values overlap.

| # | Story | Before (2) | After (4) | CPU s saved |
| ---: | --- | ---: | ---: | ---: |
| 8 | `browser_playground::story_a_walks_on_the_built_playground` | 10.9198 / 1.6443 / 12.5641 / 10.9526 | 9.8936 / 1.6065 / 11.5001 / 10.7292 | +1.0640 |
| 10 | `browser_pages::a_chat_opens_at_its_end_and_reads_back_keeping_its_place` | 9.7195 / 4.8209 / 14.5405 / 9.2967 | 9.2754 / 4.1468 / 13.4947 / 8.8389 | +1.0457 |
| 6 | `browser_accounts::the_sign_in_opens_a_tab_and_lands_on_accounts` | 6.6401 / 2.9314 / 9.5715 / 5.6096 | 6.3946 / 2.8390 / 9.2150 / 5.3970 | +0.3566 |
| 2 | `browser_pages::history_keeps_the_chat_focus_and_filter_across_reload_and_back` | 8.1830 / 3.2921 / 11.4751 / 6.2566 | 7.9305 / 3.2754 / 11.1511 / 6.1754 | +0.3239 |
| 11 | `browser_connect_flows::scrolling_the_sheet_reads_more_then_search_picks_and_adopts` | 4.1550 / 1.7483 / 5.9033 / 3.3696 | 3.9907 / 1.6195 / 5.6259 / 3.3014 | +0.2774 |
| 1 | `browser_sign_in::signing_in_reloading_reconnecting_and_signing_out_keep_the_cookie_contract` | 5.9929 / 2.4893 / 8.4822 / 15.9523 | 5.7483 / 2.4673 / 8.2155 / 15.4827 | +0.2667 |
| 5 | `browser_authorize::a_cross_site_authorization_returns_a_code_and_the_links_state` | 3.4569 / 1.7502 / 5.2072 / 12.5013 | 3.3319 / 1.6285 / 4.9457 / 12.4423 | +0.2614 |
| 3 | `browser_pages::enter_starts_one_chat_and_the_composer_keeps_the_focus` | 4.9898 / 2.3264 / 7.3162 / 4.4987 | 4.8329 / 2.2342 / 7.0671 / 4.3979 | +0.2491 |
| 9 | `browser_public::public_board_and_task_lines_are_readable_at_desktop_and_phone_widths` | 3.0602 / 1.8481 / 4.9083 / 3.3756 | 2.9342 / 1.7303 / 4.6728 / 3.2741 | +0.2355 |
| 4 | `browser_attacks::cross_origin_requests_and_framing_cannot_act_as_the_person` | 3.9010 / 2.0194 / 5.9204 / 2.9244 | 3.8676 / 1.8965 / 5.8152 / 2.8371 | +0.1053 |
| 7 | `browser_github::the_manifest_posts_once_and_history_waits_for_continue` | 1.2961 / 0.7964 / 2.0925 / 1.4202 | 1.2649 / 0.7953 / 2.0847 / 1.3759 | +0.0078 |
| 12 | `browser_journeys::a_browser_journey_looks_and_a_tui_journey_is_approved_in_chromium` | 3.7316 / 1.5340 / 5.2655 / 4.1976 | 3.6674 / 1.5281 / 5.2633 / 4.2016 | +0.0022 |

The playground walk and chat reading each saved just over 1 CPU s at the
median. Every after attempt of each was lower than both before attempts.
For chat reading, the larger part of the median drop was in system time;
for the playground, it was in user time. These combined-snapshot comparisons
do not attribute the savings to individual repairs.
