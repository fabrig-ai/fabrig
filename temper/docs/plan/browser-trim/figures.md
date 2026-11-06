# Browser trim measurements

## Before

Measured on 2026-11-06, starting at base
`f4e7e4446636be86e94505310b8a3f7ebc578cba`, before changing either walk.
Machine: `debian-32gb-hel1-14`, 16 AMD EPYC-Genoa virtual CPUs,
Linux 6.12.115, Chromium 147.0.7727.101. The slot was exclusive;
the machine was shared. Load averages (1/5/15 minutes) were
7.43/8.92/11.88 at 09:55 UTC, 19.58/18.53/15.23 at 10:00,
20.07/19.97/16.19 at 10:02, and 17.06/20.00/16.96 at 10:05.
These are loaded-machine observations, not idle-machine budget estimates.

### Method and accounting

Each walk ran alone, through `heavy`, with the browser-tier prerequisites
built first. Builds are excluded from the test figures. Temporary `Play`
wrappers printed zero-based step boundaries; the public walk also printed
its page and text-read boundaries. The tables partition the whole observed
run, including startup and shutdown. The small residual includes fixture
initialization before the script clock and teardown after its last step;
it is an upper bound on teardown alone.

A temporary Chromium pipe proxy preserved the harness's CDP connection,
including its handling of `Page.javascriptDialogOpening`. It recorded
command counts, network requests and metrics every 500 ms. It also queried
`SystemInfo.getProcessInfo`. A separate sampler followed only the command's
process tree through `/proc`, retaining each process's CPU counters and
classifying Chromium's browser, renderer, GPU and utility processes. The
CPU attribution below is sampled user plus system time, rounded to 10 ms;
short-lived processes and CPU after the last sample may be missed.
The observer's Python processes and builds are excluded from that table.

The tier collector's figures are reported separately. Its
`waited-and-reaped-descendants` scope is narrower than these live process
observations (`tools/tiers/design.md`, 7.2): do not sum the attribution table
and call that the collector's CPU. In particular, the sampled renderer CPU
is substantial. The discrepancy is visible here, rather than silently
rescaling processes to the collector's total. These measurements do not
establish which descendant-reaping path accounts for the discrepancy.

### History walk

`browser_pages::history_keeps_the_chat_focus_and_filter_across_reload_and_back`:
the unchanged first run passed in **25.073 s wall**. Timing-only runs passed
in **23.072 s** and **25.014 s**; the final detailed baseline below passed
in **23.182 s**, with **13.324 CPU s** in the tier collector.

| Steps or phase | Wall s | What happened |
| --- | ---: | --- |
| 0–17 | 1.780 | Fresh Chromium, sign-in, demo startup, compose the first chat |
| 18–24 | 1.098 | First Back/Forward and the change card appearing |
| 25–64 | 1.852 | Task, plan and agent pages; return to the chat |
| 65 | 8.215 | Wait for the change card to say Landed |
| 66–74 | 0.299 | Read landing, confirm Close, navigate to Closed chats |
| 75 | 8.539 | Wait for T1 to appear in the closed list |
| 76–78 | 0.046 | Closed row's status, text and address |
| 79–109 | 1.282 | Remaining history, focus, filter, reload and unknown-address assertions |
| Fixture initialization and teardown residual | 0.070 | Includes browser exit and server drain; no 30 s close wait |
| **Whole test** | **23.182** | **100% assigned**, rounding at the last decimal |

The two long waits account for **16.754 s**, 72% of this run. They are
inside `change_and_close_steps`, owned by the following cut. Closing is
not teardown: it happens while the browser and server remain live. The
stock chat template offers 5 s of closing grace, and the demo worker's
agent-stop grace is 5 s. The step measurement locates the 8.539 s wait;
it does not separately attribute it to those grace stages. No grace,
pace or story step was shortened here.

### Public walk

`browser_public::a_visitor_reads_a_public_project_and_loses_it_when_made_private`:
two detailed baseline runs passed in **20.740 s wall / 9.786 CPU s** and
**28.883 s wall / 13.516 CPU s** in the tier collector. The second run
looked off, so it was compared with the first: the large change was in
waiting for landing, **9.215 s instead of 1.990 s**, after CI had passed.
Public page reading was nearly unchanged. The table uses the slower run.

| Owner steps, visitor page, or phase | Wall s | What happened |
| --- | ---: | --- |
| Owner 0–19, including ctl's first-profile setup | 4.501 | Two fresh browser contexts, sign-in, demo, publish, chat and task |
| Owner 20 | 9.668 | `change show 2 --until ci:passed` runs to exit |
| Owner 21–23 | 0.309 | Read Passed/Approved; start the landing wait |
| Owner 24 | 9.215 | `change show 2 --until landed` runs to exit |
| Owner 25–29 | 0.236 | Read landing, sign out, reopen anonymously |
| Projects 0–3 and complete text read | 0.429 | Public index |
| Board 0–5 and complete text read | 0.501 | Project board |
| Task 0–4 and complete text read | 0.585 | Goal/task and change |
| Landings 0–4 and complete text read | 0.414 | Empty public landings |
| Private 0–6 and complete text read | 2.383 | Reopen task, make private, observe freshness-driven removal |
| Empty 0–3 and complete text read | 0.134 | Public index after revocation |
| Malformed 0–2 and complete text read | 0.349 | Malformed identifier returns to the index |
| Fixture initialization and teardown residual | 0.159 | Includes both contexts closing and server drain |
| **Whole test** | **28.883** | **100% assigned**, rounding at the last decimal |

The owner phase was **21.798 s** after the public driver's first call;
the earlier ctl/context setup is included in the owner rows above.
The visitor portion was **4.795 s**. Revocation consumed **2.383 s** of
that, consistent with the fixture's two-second freshness plus navigation
and assertions. The two ctl waits consumed **18.883 s**, 65% of the run.
They are the following task's steps, left intact.

### CPU by process

These are the two detailed runs used in the wall tables, observed through
teardown. Rows combine processes of the same kind, including replacement
renderers. The internal-program row includes the service's server/worker
process and its agent subprocesses; the short `box` process is separate.

| Process kind | History CPU s | Public CPU s |
| --- | ---: | ---: |
| Test binary | 2.62 | 2.86 |
| `temper box` | 0.04 | 0.08 |
| `temper internal …` | 2.98 | 3.07 |
| `temper fake …` | 0.42 | 0.44 |
| `temper forge …` | 0.02 | 0.03 |
| `temper ctl …` | Not run | 0.19 |
| Chromium browser | 3.13 | 3.06 |
| Chromium renderers | 7.15 | 6.48 |
| Chromium GPU | 0.73 | 0.51 |
| Chromium utility/network/storage | 1.43 | 1.28 |
| Chromium zygotes | 0.07 | 0.13 |

The public run also observed 0.01 s in init and 0.03 s in git; processes
missed between samples have no claimed zero cost. The history walk issued
**519** `Accessibility.queryAXTree` commands; the public walk issued
**185**. Waiting therefore costs browser and harness CPU as well as wall.

### Settled pages on the box

Separate temporary probes kept the completed demo chat and task open for
8 s each, and the public project board for 6 s. The measured intervals
below are at least 5 s within those holds, after navigation/setup. No
person acted during them. Metrics were compared within the same CDP
session, never across the public walk's two contexts.

| Page | Interval s | Δ ScriptDuration s | Δ TaskDuration s | Δ LayoutCount | Δ RecalcStyleCount | New requests | New streams |
| --- | ---: | ---: | ---: | ---: | ---: | --- | ---: |
| Chat `/chats/1` | 5.0068 | 0 | 0.006612 | 0 | 0 | 0 | 0 |
| Task `/tasks/2` | 5.0112 | 0 | 0.006041 | 0 | 0 | 0 | 0 |
| Anonymous public board | 5.0066 | 0.008826 | 0.047305 | 5 | 5 | 4 public GETs | 0 |

Chat and task **stand still** in script/layout/style counters. Their tiny
TaskDuration deltas include 0.005081 s and 0.004614 s of
`DevToolsCommandDuration`, respectively. Existing authenticated watch
connections stayed open; neither page opened another watch or reconnected.

The public board **does not stand still**: two freshness cycles each
opened a jig board GET and a forge landings GET, at 2.008 s spacing.
There was no authenticated watch, SSE connection or WebSocket. This is the
explicit public freshness behavior, including dropping expired documents
before displaying a new answer (`temper/docs/design/client/browser.md`,
8.2). It is bounded expiry/read/redraw work, not an unbounded render or
reconnect loop. The fixture deliberately sets max-age to 2 s so that the
open-page revocation assertion is observable.

### Candidates and verdicts

| Candidate | Measurement | Verdict and action |
| --- | --- | --- |
| Box pace | Embedded model `reply_ms=1200`, forge `ci_ms=4000`; first public-run reply wait 1.240 s; history's Landed step 8.215 s; public CI wait 9.668 s | `init --box` retains Story A's paces in `application.demo.llm` and its fake forge. Real paced work and forge observation dominate these waits. Leave pace and landing steps for the cut. |
| Polling while waiting | History 519 AX queries in 23.182 s; 16.754 s in its two long waits | The harness's 10 ms `POLL` wakes its loop; the face additionally spaces repeat finds by the browser kit's **50 ms** `poll`. Thus this is not 100 full AX queries/s. It still burns CPU during long waits. The cut removes the waits; do not change the shared poll policy here. |
| `ctl change show --until` | CI 9.668 s, landing 9.215 s; all ctl subprocesses together 0.19 sampled CPU s | The command opens a change-family watch and a task watch; it is not a timed HTTP polling loop (`temper-client-tools/src/change.rs`). The engine's forge client polls resources at 5 s, backing off to 30 s, with 1 s hinted reads. Waiting for actual paced CI/gate/landing state, not ctl polling CPU. |
| One-node-at-a-time public reading | 60 text nodes across 7 pages, plus 7 absence queries; 1.088 s wall and approximately **0.99 CPU s** excluding the CDP observer. A no-proxy repetition measured **0.81 / 0.78 / 0.73 CPU s** | Below the specified >1 CPU s threshold. Keep the reading mechanics and every assertion; no browser-kit API extension is justified by this measurement. |
| Public freshness | Private phase 2.383 s; quiet public board 4 GETs / 5.007 s | Expected two-second fixture expiry, which the browser assertion checks. Preserve it. |
| Teardown/history connections | History residual ≤0.070 s; public ≤0.159 s, including initialization | No 30 s server-close stall. Browser shutdown precedes stopping the server. No teardown fix. |
| Page render/reconnect loop | Quiet chat/task: script/layout/style deltas all zero, no requests or new streams for >5 s | No product rendering or reconnect cause found. Public expiry redraw is explained above. |
| Closed-list wait, found during timing | 8.539 s at history step 75, after closing the chat | Separate from landing and teardown; in `change_and_close_steps`, which the following task cuts. Retain the figure and the configured grace policy. |
| Story A gzip | **4.85 CPU s / 4.880 s wall** in `download_size`; whole story 10.863 collector CPU s / 13.956 s wall | Compression is a measurement tool, not the shipped code being tested. Use the machine's gzip. Native replay took 2.280 s wall / 2.23 sampled CPU s and remains the downloaded-log proof. |
| Every test's floor | GitHub: 1.912 collector CPU s / 1.102 s wall; sampled test 0.10, temper 0.22, browser 0.80, renderer 0.37, GPU 0.12, utilities 0.25, zygotes 0.11 CPU s | Mostly fresh browser/server startup, page navigation and CDP, not a needless rebuild. No bundle generation or Rust compilation happens inside a test. |
| Fresh-profile wasm compilation | GitHub trace: **zero wasm requests and zero wasm compile events**. Account-page trace: two wasm requests; streaming validation 0.140 s, lazy compilation 0.130 s, compilation-unit intervals 0.149 s | GitHub exercises static manifest/forge documents, so its floor cannot be called wasm compilation. A wasm page does pay compilation; the second load in its own profile had a native-module cache hit (0.001508 s). Trace durations are thread elapsed intervals, with nested top-tier compilation included in the 0.149 s, not additional CPU totals. |
| Compile/load cache | Account test 3.782 collector CPU s / 2.516 s wall; existing within-profile native-module cache hit above | Keep fresh server, page state, context/profile and dynamic origin per test. Sharing a browser profile or authenticated page to reuse compilation changes those semantics. No such cache added. |
| `Tier::accounts::copy_bundle` | 10,818,796 B copied; outer recursive call **0.019097 s wall** | Too small to justify another cache or sharing the mutable issuer bundle. Leave it. |
| Bundle preparation outside the tests | Two bundle-tool processes sampled at 11.55 and 26.52 CPU s in one warm prerequisite run | The tier entry point builds both bundles once before nextest. This is outside every test and the tier CPU total; it is not a rebuild per story. No prerequisite/cache policy changed. |
| CPU accounting scope | Detailed process attribution exceeds the collector's totals; e.g. playground renderer alone sampled at 6.95 CPU s | Preserve both measurements and the collector's stated scope. No budget decision or collector change inferred from this profile. |

The account-page trace's compilation intervals overlap other activity and
some are nested. They are evidence of the fresh-profile work and the
within-profile cache, not a sum to add to the account test's CPU. No
Chromium launch flag was changed: the measured GPU cost does not establish
a needless feature, and these stories must keep their browser semantics.

## Changes measured in this task

No product cause was found or product code changed. No story step, wait,
pace, freshness, timeout or budget was cut or raised.

The first public-extraction estimate was 1.06 CPU s including the pipe
observer. Excluding that observer gave 0.99 s, close enough to the task's
1 s threshold to require another measurement. A throwaway probe then read
each page three times, without the CDP proxy. Summing the corresponding
extraction windows across all seven pages gave **0.81, 0.78 and 0.73 CPU s**
(**0.773 s mean**), with **1.099, 1.114 and 1.114 s wall**. The sampler
followed only the test's descendants and excluded itself, nextest and build
processes. These are repeated reads within a page, not three whole-test
runs. The first read of every page alone totaled 0.81 CPU s. The scalar
reading mechanics remain unchanged: the task's threshold was not met.

- Gzip measures the same **29 files / 21,241,423 raw bytes**, each as an
  independent gzip member with no file name or timestamp. The machine's
  gzip 1.13 uses its default compression rather than skein-http's encoder:
  **6,882,055 B**, versus **7,593,581 B** before. Both are under the design's
  **8,000,000 B** bar (`temper/docs/design/playground.md`, 8 and 9).
  The printed figure remains compressed static-download size, not a test
  of skein-http's compression bytes or ratio; the static test host serves
  identity files. The compressor change must accompany comparisons of
  these byte figures. Compressing those same files as a separate tool
  measurement took **1.046 CPU s / 1.076 s wall**, versus 4.85 / 4.880 s
  in the test's unoptimized encoder. The changed story passed in 10.430 s
  wall; the page's memory figure stayed **6,606,028 B**.

All timing wrappers, CDP proxies, traces, process samples and probe tests
are temporary and are removed before pushing. The following task should
measure its cut against the Before walks, rather than attributing a
loaded-machine whole-run difference to the compression-tool improvement.

## Verification on this branch

One full browser-tier run, with the retained gzip change and the original
public reader, ran all **13 tests: all passed**, with no starvation, leaks,
incomplete records or unobserved tests. The collector measured
**96.151911 CPU s / 76.202315 s wall**: **over the 85 CPU s budget**.
This is a remaining red for the following story-cut task, not permission
to raise a budget or shorten the assigned-away waits.

| Test | Collector CPU s | Wall s |
| --- | ---: | ---: |
| `browser_accounts::the_accounts_page_returns_after_a_key_send` | 4.315 | 2.254 |
| `browser_accounts::the_sign_in_leaves_the_page_and_comes_back_to_ready` | 4.511 | 1.821 |
| `browser_attacks::cross_origin_requests_and_framing_cannot_act_as_the_person` | 6.333 | 2.372 |
| `browser_authorize::a_cross_site_authorization_returns_a_code_and_the_links_state` | 4.778 | 1.889 |
| `browser_github::the_manifest_posts_once_and_history_waits_for_continue` | 1.984 | 0.887 |
| `browser_pages::closing_project_rename_returns_focus_to_the_summary` | 4.165 | 2.003 |
| `browser_pages::enter_starts_one_chat_keeping_a_shift_enter_break` | 5.409 | 2.249 |
| `browser_pages::history_keeps_the_chat_focus_and_filter_across_reload_and_back` | 16.722 | 23.081 |
| `browser_pages::latest_returns_a_chat_to_its_end_after_new_words_while_reading_back` | 8.978 | 4.150 |
| `browser_playground::story_a_walks_on_the_built_playground` | 12.299 | 10.217 |
| `browser_public::a_visitor_reads_a_public_project_and_loses_it_when_made_private` | 14.557 | 19.188 |
| `browser_sign_in::signing_in_reloading_reconnecting_and_signing_out_keep_the_cookie_contract` | 8.488 | 4.624 |
| `browser_task_pages::start_goal_opens_chats_for_a_draft` | 3.614 | 1.445 |

The playground story's final figure was 12.299 CPU s / 10.217 s wall.
The controlled compression measurement above separates its saving from
whole-story variance. The remaining history and public walks cost
31.279 collector CPU s together and retain the waits documented in Before.

The package's 10 focused tests and 20 public domain/view reader tests also
passed. `cargo fmt --check`, the temper old-form ratchet, clippy for
`temper-client-browser-world` on all targets with warnings denied, and
citations against `origin/main` passed. No new test, public API, generated
file, cross-tree code change or browser launch flag is retained.
