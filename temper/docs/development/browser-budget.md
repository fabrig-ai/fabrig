# Browser budget: the repair's account

## The result

2026-11-06. Final measurements use merged commit `865cd995698a` on 16 CPUs,
AMD EPYC-Genoa Processor. The [baseline](../plan/browser-budget/figures/story-baseline.md)
uses `32c015e965da` on 8 CPUs of the same model. The request's figures are
from the 17-story run at `668f7cd3e503`; its machine load is unknown.
[Final figures](../plan/browser-budget/figures/budget-close.md) retain every
run and the long story's process split. CPU seconds include descendants,
user plus system. Each median uses three runs of one story by name through
`tools/tiers/run --cpu`, rather than a whole-tier run.

| Story | Request CPU s | Baseline median CPU s | Final median CPU s | Target CPU s | Result |
|---|---:|---:|---:|---:|---|
| long | 35.2 | 34.876193 | 20.858273 | about 12 | Not met; 8.858273 s above 12 |
| latest | 12.7 | 12.480270 | 9.178371 | about 6 | Not met; 3.178371 s above 6 |
| earlier | 7.9 | 7.993236 | 8.399148 | about 5 | Not met; 3.399148 s above 5 |
| short | 4.3 | 4.193832 | 6.414108 | none | Reported; no target was set |

Labels mean `a_long_chat_opens_at_its_end_and_reads_back_from_the_composer`,
`latest_returns_a_chat_to_its_end_after_new_words_while_reading_back`,
`earlier_words_stay_put_without_browser_scroll_anchoring`, and
`a_short_chat_opens_whole_without_a_reading_status`, all in
`temper/tests/client-browser/tests/browser_all/browser_pages.rs`.

The loads beside each CPU median below belong to the run producing that
median. They are the one-, five- and fifteen-minute averages before and after
the command, including prerequisites. They describe load, not CPU utilization.
All twelve final comparison runs passed. Baseline and final arms were taken
at different times and CPU counts, so these differences do not isolate a
single lever's effect.

| Story | Baseline load before → after | Final load before → after | Baseline median wall s | Final median wall s |
|---|---|---|---:|---:|
| long | 2.95 / 4.11 / 8.95 → 3.61 / 4.14 / 8.74 | 5.08 / 9.87 / 8.45 → 4.43 / 9.18 / 8.28 | 20.957697 | 14.266906 |
| latest | 2.21 / 4.37 / 9.42 → 2.07 / 4.16 / 9.21 | 6.13 / 11.08 / 8.70 → 6.37 / 10.72 / 8.64 | 5.329647 | 4.880247 |
| earlier | 2.33 / 5.01 / 10.11 → 3.54 / 5.06 / 9.99 | 6.37 / 10.72 / 8.64 → 6.69 / 10.48 / 8.62 | 4.481255 | 3.999198 |
| short | 3.54 / 5.06 / 9.99 → 2.88 / 4.81 / 9.80 | 6.69 / 10.48 / 8.62 → 5.08 / 9.87 / 8.45 | 2.162703 | 2.615083 |

Wall medians are independent of CPU medians. Long and Latest use less CPU
than the baseline; earlier and short use more. Short rose by 2.220276 CPU s,
about 53%. Earlier's three final CPU samples span 3.576509–8.671831 s; all
remain in the figures, including the low sample. These are targets missed,
not changes to limits.

## The whole tier isn't measured here

After-landing proves the whole tier on the next main. The four final CPU
medians add to **44.849900 s**, against the baseline's 59.543531 s. This leaves
**40.150100 s** for the other thirteen against the 85 CPU-second budget, or
**25.150100 s** against the request's quiet-run target of 70.

The request's other thirteen ranged from 45 to 70 s. Adding that range to
the four final medians gives **89.849900–114.849900 s**: 4.849900–29.849900 s
over 85, or 19.849900–44.849900 s over the target of 70. With the request's
specific 69.3 s remainder, the estimate is 114.149900 s. None is a measured
whole-tier result, and none establishes that the tier is within budget.
The other thirteen were not run here. No budget, timeout or tier membership
changed; CPU budgets and wall starvation remain separate
(`skein/docs/foundation/testing-strategy.md`, 8.1).

## Each lever

**History through ctl landed.** Histories are prepared through the tier's own
service before the page opens. Latest keeps its typed message from far back;
incoming words use ctl rather than another tab and delegate. The listener
repair admits a waiting connection by ending an idle one, preserving active
calls, partial requests, connection bounds and deadlines. Latest's opening
wait observes the history it needs, rather than requiring the chat's first
sentence already to be loaded.
[History figures](../plan/browser-budget/figures/history-ctl.md) give these
medians, with each run's load and the failed development attempts retained:

| Story | Before CPU s | After CPU s | Reduction | Before wall s | After wall s |
|---|---:|---:|---:|---:|---:|
| long | 37.722038 | 21.952863 | 41.8% | 22.020055 | 18.070390 |
| latest | 12.480270 | 5.573748 | 55.3% | 5.329647 | 5.429844 |
| earlier | 8.318314 | 5.775053 | 30.6% | 4.766546 | 7.253129 |
| short | 4.216724 | 3.532472 | 16.2% | 2.623958 | 4.083622 |

The local Latest baseline failed to observe incoming words; its three-run
reference is story-baseline. The before machine was recorded as 16 CPUs,
except that 8-CPU reference; after used 8 CPUs of the same model. Arms were
not alternated. A ctl deadline failure with a page open led to the listener
repair, with 12 held connections at capacity, three in calls and one rejected
incoming connection. The final three Latest runs passed unchanged deadlines.

**The smaller conversation window landed.** The browser tier alone uses
131,072 bytes, an eighth of the person's 1,048,576-byte bound. Eight distinct
replies total 420,272 bytes, beyond three windows. The long story now drops
and recovers both ends through repeated traversal, while holding position.
The unanchored fixture stays within one window at 109,642 bytes. The reply
allowance remains 65,536 bytes; the fake's joint script allowance is 524,288.
[Window figures](../plan/browser-budget/figures/window-story.md) show:

| Story | Before CPU s | After CPU s | Before wall s | After wall s |
|---|---:|---:|---:|---:|
| long | 16.582325 | 19.389239 | 19.442755 | 12.064276 |
| latest | 9.205289 | 9.380930 | 5.020786 | 5.109279 |
| earlier | 7.066986 | 8.213927 | 4.782042 | 4.086366 |
| short | 4.220289 | 5.761806 | 2.480827 | 2.364767 |

These 16-CPU runs were not alternated, and the after machine was much less
loaded. Latest's baseline resumed after the ctl repair; the original failed
ctl deadline remains recorded. The changed fixture also proves more traversal.
Long's wall fell, but all four CPU medians rose: the table does not establish
an isolated CPU saving from the smaller window. The other thirteen were read
for histories reaching the smaller bound, and were not run.

**Find backoff did not land.**
[Its figures](../plan/browser-budget/figures/find-backoff.md) compare A, the
merged polling, with B, repeated misses waiting 50, 100, 200, 200 ms, with
press polling preserved and acts and successful finds resetting the wait.
Each story ran A B A B A B on 16 CPUs of the same model; all 24 comparisons
passed. A startup refusal on an older base is retained outside the medians.

| Story | A CPU s | B CPU s | B/A CPU | A wall s | B wall s | B/A wall |
|---|---:|---:|---:|---:|---:|---:|
| long | 19.570480 | 20.613688 | 105.33% | 12.820851 | 13.664994 | 106.58% |
| latest | 9.054425 | 9.289662 | 102.60% | 4.999002 | 4.940550 | 98.83% |
| earlier | 8.785078 | 7.938130 | 90.36% | 4.185837 | 3.893208 | 93.01% |
| short | 5.859718 | 4.394034 | 74.99% | 2.425084 | 2.614682 | 107.82% |

Long needed at most 90% of A's median CPU and improvement in two of three
pairs; it reached 105.33% and improved only one pair. Long and short also
exceeded the permitted 105% median wall comparison. Long's accessibility-query
median was 67 for A and 68 for B; all 401 queries used role index zero.
The repaired story did not show the earlier 519-query pattern. The candidate
and instrumentation were removed; only its measured decision landed.

## A delivery's cost

[Delivery figures](../plan/browser-budget/figures/delivery-cost.md) used the
person's whole bound, on 8 CPUs of the same model, Chromium 147.0.7727.101,
native dev binary and dev Wasm. A real Task priority amendment delivered
beside unchanged held text, three times per size. This measures neither a
whole streamed answer nor browser-tier CPU.

When a delivery makes the view stale, the semantic page tree is rebuilt,
held Markdown is read again, and old and new trees are compared across the
whole window. Unchanged builds are gated. DOM patches update changed nodes
in place, so the whole DOM is not replaced. Page bookkeeping also has
whole-tree passes. End following reads aggregate scroll geometry after
nonempty patches; it does not explicitly measure every entry. Holding an
away reader can scan rectangles across the window, even for a patch outside
that log. That away scan was read in code, not timed in these samples.

| Held text | Main-thread task range ms | Median tasks ms | Median style ms | Median layout ms | Median nodes touched |
|---|---:|---:|---:|---:|---:|
| about 128 KiB, 129,258–129,746 bytes | 15.428–19.262 | 18.854 | 1.253 | 0.146 | 1 |
| about 1 MiB, 1,034,047–1,034,105 bytes | 26.995–38.397 | 35.023 | 5.089 | 0.134 | 1 |

The larger window held 1,403 log nodes and 1,512 page nodes; its first delivery
touched eight nodes, the other two one each. No rectangle reads occurred in
these end-following samples; height getter counts were 3/1/1 at the whole
bound, 1/1/1 at the smaller size. Whole-bound load was
17.78 / 12.93 / 10.74 before and after each sample. Script-duration medians
were 0.027 ms and 0.036 ms respectively; those counters are not main-thread
totals and must not be added to style and layout as if they were.

A person at the whole bound sees 27.0–38.4 ms of main-thread tasks per sampled
delivery, median 35.0 ms, about 2.2 frames at 16 ms. The median is 16.169 ms
above the smaller sample. These loaded, dev-build observations indicate a
frame risk, not a release latency guarantee. No fix landed: retaining parsed
Markdown or a conversation subtree would introduce the mechanism already
declined in `temper/docs/design/draft/chat-reading.status.md`, 3, *Drawn once*.
Avoiding the repeated roughly 1.03 MB read is part of the measured opportunity,
not a measured Markdown-only saving. Reducing bookkeeping or anchor scans
needs invalidation rules and supporting proofs; no justified small fix was
identified. The tier's smaller window leaves the person's whole bound and
these costs in place.

## Belonging

`temper/docs/design/client/browser.md`, section 4, is the test: a story is in this tier only when no world can show it. No story moves, and budget played no part in that call. This is a cut's reading of the design and of the lower worlds' tests by name, not a run.

- **`a_long_chat_opens_at_its_end_and_reads_back_from_the_composer` stays.** The client world holds the window's logic on a tree (`suite::reading::reading_back_then_forward_recovers_whole_lines_and_follows_new_turns`), and the domain the end coming back (`the_end_comes_back_after_enough_older`). Neither has a layout. Only a browser shows the end in view with the head a screen and a half above it, PgUp scrolling from a focused text area, and real measurements driving the reads.
- **`latest_returns_a_chat_to_its_end_after_new_words_while_reading_back` stays.** `suite::reading::latest_replaces_the_past_with_the_ends_window`, `sending_from_the_past_returns_to_the_end_and_shows_the_persons_words` and `live_text_and_its_commit_wait_while_the_person_reads_back` hold what the tree does, and skein's `a_log_scrolled_away_returns_to_its_end_and_keeps_following` holds the page part on its example. Only here is the view seen holding and returning, in temper's page.
- **`earlier_words_stay_put_without_browser_scroll_anchoring` stays, and is the nearest overlap.** skein's `reading_without_browser_anchoring_follows_requests_and_page_keys` holds the page part without anchoring, on skein's example. This story holds it on temper's keyed lines and sheet, which `temper/docs/design/draft/chat-reading.status.md`, 5, *In every browser*, names as the nearest a run comes to Safari. Moving it would go to another tree's browser tier, not a lower world, and leave temper's lines and sheet unproved.
- **`a_short_chat_opens_whole_without_a_reading_status` stays, and is the closest call.** That no status or pill shows is the tree's, and a lower world could show it. That a short chat's start and end both stand inside the log, with nothing more read, takes a layout.


## What's left and where it goes

The three CPU targets remain unmet by 8.858273 s for long, 3.178371 s for
Latest, and 3.399148 s for earlier. The whole tier remains unmeasured here,
with the estimated range above already exceeding both 85 and the target of
70. Short has no target, but its measured CPU regression remains in the result.

The final classified long sample used **13.415869 CPU s**, 11.970142 s wall,
on the same merged commit and 16-CPU model. Its load went from
16.20 / 11.04 / 9.02 to 30.43 / 15.19 / 10.50. It is an attribution sample,
separate from the three-run median of 20.858273 s; it is not substituted for
that median. Initial sampler diagnostics and both extra attribution runs are
retained in the final figures.

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

The split accounts for **89.7%** of that sample. Chromium takes
6.50 observed CPU s, the test binary 1.75, serving 1.68, ctl 0.15, and other
children 1.96, including agents, fakes and setup helpers. The latter includes
0.24 s of `pkill` calls used to hold reads during traversal. Sampling misses
short-lived work and final ticks; the remaining 1.375869 s is unassigned.
These numbers locate process families, not functions: they do not distinguish
renderer layout, Markdown, accessibility queries or measurement, and cannot
attribute the comparison median's higher CPU by scaling this sample.

The person's repeated whole-window tree and Markdown work remains, and the
away-reader anchor scan still lacks a timed split. Find backoff did not earn
its landing. After-landing must measure the other thirteen with the four on
the next main; neither their final cost nor a whole-tier pass is claimed here.
