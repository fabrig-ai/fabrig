# delivery-cost: one delivery beside a held conversation

2026-11-06 UTC, commit `32c015e965`, 8 CPUs, AMD EPYC-Genoa Processor.
Chromium 147.0.7727.101, native dev binary and dev Wasm bundle. No production
code changed.

One delivery rebuilds the semantic tree and rereads the held Markdown. It
does **not** replace the whole DOM. Log geometry is read after a nonempty
patch; following the end reads aggregate geometry, whereas keeping a
reader's position can scan entries across the window.

## The path in code

Paths below are relative to the repository root.

| Layer | File and function | Work on a delivery |
|---|---|---|
| Thread input | `temper/crates/temper-client-domain/src/pages/chat.rs`: `run_changed`, `activity_page`; `jig/crates/jig-client/src/thread/model.rs`: `Thread::run`, `prune_live` | A run-text update replaces that provisional turn, cloning its supplied text, not the committed window. `prune_live` visits and moves the retained provisional turns. A task delivery calls `Thread::document` / `moved` to request a newer activity tail when needed. It does not reread every committed turn from the server. |
| Committed window | `jig/crates/jig-client/src/thread.rs`: `Thread::accept`, `slide`, `window_weight`; `thread/window.rs`: `TurnWindow::slide`, `move_turnwindow` | Incoming activity is checked against held rows; accepting it visits the held window, recomputes its weight, moves retained rows into new list storage and drops whole rows at the opposite edge as needed. These are whole-window passes, with additional duplicate checks, not a deep clone of all held text. |
| Thread projection | `jig/crates/jig-client/src/thread/model.rs`: `Thread::lines` | Allocates and fills a chronological list of borrowed lines across the whole held window, plus pending words and provisional turns at the newest end. It borrows committed text. This is whole-window work each time the view asks. |
| Tree and Markdown | `temper/crates/temper-client-view/src/root.rs`: `build`; `src/pages/chat.rs`: `conversation_with_times`, `committed`, `provisional` | The shown-revision / clock gate skips unchanged builds. Once stale, the whole page is built; the conversation iterates `thread.lines()`. Every committed text block calls `skein_ui::markdown::read`; provisional text calls `read_growing`. There is no retained parsed Markdown or retained conversation subtree. |
| Tree comparison | `skein/crates/skein-ui/src/view.rs`: `View::finish`; `src/diff.rs`: `diff`, `match_nodes`, `old_match`, `attributes` | Matches and compares the complete old and new trees. Keys retain identities; equal attributes and runs produce no patch. Whole-tree comparison, changed-node output. Matching scans siblings' subtrees, so a node count alone is not a claim of linear cost. |
| Page model and DOM | `skein/crates/skein-ui-web/src/page.rs`: `Page::apply`, `collection_sizes`, `preorder`; `src/page/web.rs`: `Mounted::apply`, `refresh`, `insert` | Replays changed-node patches in place. Independently, `Page::apply` scans the model for collection sizes and rebuilds its preorder link list; the mounted half scans its nodes for dialogs. Thus DOM writes are incremental, but page-part bookkeeping still has whole-tree passes. A text patch replaces that node's runs, not every text node. |
| Following and measuring | `skein/crates/skein-ui-web/src/page.rs`: `Page::apply`, `finish_logs`, `scrolled`, `report_near`; `src/page/web.rs`: `Mounted::apply`, `measure` | For nonempty patches, following logs get `ScrollEnd`, which reads `scrollHeight` and writes `scrollTop`; bound logs also get `Measure` after replay, reading `scrollTop`, `scrollHeight`, `clientHeight`. Scroll events read the same three scalars. These reads can flush browser layout, but do not explicitly measure every entry. An unbound, fully loaded log has no post-patch Near measurement. |
| Keeping a reader's place | `skein/crates/skein-ui-web/src/page/web.rs`: `hold_view`, `view_entries`, `put_view` | Before patching an away log, recursively scans children for visible entries, skipping statuses and descending through boxless wrappers. It keeps at most four deepest visible entries, but can read rectangles for all earlier/offscreen siblings on the way: the scan is not bounded to four nodes. Afterwards it reads the first surviving anchor's rectangle and adjusts scroll. That rectangle flushes layout and browser anchoring before the correction. This path runs for any nonempty patch, not only a patch inside that log. |

This agrees with `temper/docs/design/draft/chat-reading.status.md`, 3,
*Drawn once*. The reading contract is
`temper/docs/design/client/README.md`, 5.15, and the page-part contract is
`skein/docs/design/ui-web.md`, 7.

## What was measured

The throwaway Node driver ran through `heavy`, against its own `temper box`
and fake playground. History was created through `temper ctl`: sign in
using stdin, `playground start`, `chat start`, then `task message`, each
reply awaited with `task show --until replied`. No credential file was read
by the driver. The reader used a fresh printed sign-in link.

The bundle was built with:

```sh
heavy bash temper/scripts/bundle.sh ../target/browser-budget/delivery-bundle
heavy cargo build -p temper
```

The bundle script's default build names no features; this base has no
`small-window` feature. Its output digest was `c647a94b93bc35e5`.
`temper-client-shell-web/src/limits.rs::client` names `document_bytes =
1_048_576`; `temper-client-domain/src/pages/chat.rs::thread_limits` uses
that whole bound. Both arms used this same bundle, without reducing it.

Each large reply contained a fenced, horizontally scrolling code line of
12,500 repetitions of `data `, an end marker and 30 short paragraphs. Two
replies made the smaller conversation; sixteen made the larger one. The
driver scrolled to load history, then returned to the end before sampling.
The larger log held all sixteen replies: 1,034,047 UTF-8 text-content bytes,
1,403 semantic DOM nodes in the log and 1,512 in the page. The smaller log
held 129,258 bytes, 179 log nodes and 288 page nodes. These byte counts are
observable text, not the thread's structural byte charge.

A sample was **one real Task document delivery**, generated by changing
the chat's priority through `task amend --rev ... --priority ...`, three
times at each size. This exercises a delivery beside unchanged long text,
not the extra cost of receiving or parsing a large new answer. The driver
split the browser's real SSE response at record boundaries, queued the
records and released exactly one selected record (stream tag 1, jig
delivery kind 2). It preserved bytes and split transport chunks at 4,096
bytes. Other SSE records stayed queued; HTTP fetch responses arriving
during the sample waited until after it. Follow-up activity reads were
drained between samples. No delivery or document was fabricated.

CDP `Performance.getMetrics` was read immediately before release and
after 120 ms to allow the turn and rendering work to run. The table gives
deltas in milliseconds for `ScriptDuration`, `RecalcStyleDuration`,
`LayoutDuration`, and also `TaskDuration`, the measured main-thread task
time. The very small ScriptDuration counter is **not** the main-thread
total: use TaskDuration here, and do not add the three category counters
and call that the total. The interval also admits page clock/resize work;
it is an observed page cost, not an isolated Wasm-function benchmark.

A `MutationObserver` counted distinct `data-node` IDs touched by DOM
mutations, including inserted/removed descendants and a mutation's nearest
semantic ancestor. “Nodes patched” below means this observable DOM count,
not the number of Rust `Patch` variants. It includes attribute writes even
when the value is unchanged. Separate wrappers counted rectangle and
height getter calls. The observer and wrappers were present in both arms.

Chromium ran with `--headless=new`, at 1280 × 900 CSS pixels through CDP
`Emulation.setDeviceMetricsOverride`; the driver handled
`Page.javascriptDialogOpening`. Load is the machine's 1/5/15-minute load
average before and after each sample, not CPU utilization. These are
delivery measurements, not tier runs or tier CPU figures.

## Six deliveries

Final run: 2026-11-06 18:38:34–18:39:30 UTC. Times are milliseconds.

| Held size | Delivery | Held text bytes | Script | Style | Layout | Main-thread tasks | Nodes patched | Load before | Load after |
|---|---:|---:|---:|---:|---:|---:|---:|---|---|
| About 128 KiB | 1 | 129,258 | 0.035 | 1.800 | 0.134 | 18.854 | 1 | 13.19 / 11.68 / 10.28 | 13.19 / 11.68 / 10.28 |
| About 128 KiB | 2 | 129,502 | 0.039 | 1.237 | 2.733 | 15.428 | 1 | 14.14 / 11.91 / 10.36 | 14.14 / 11.91 / 10.36 |
| About 128 KiB | 3 | 129,746 | 0.036 | 1.253 | 0.146 | 19.262 | 1 | 14.14 / 11.91 / 10.36 | 14.14 / 11.91 / 10.36 |
| About 1 MiB | 1 | 1,034,047 | 0.478 | 10.796 | 0.577 | 38.397 | 8 | 17.78 / 12.93 / 10.74 | 17.78 / 12.93 / 10.74 |
| About 1 MiB | 2 | 1,034,076 | 0.026 | 5.089 | 0.134 | 26.995 | 1 | 17.78 / 12.93 / 10.74 | 17.78 / 12.93 / 10.74 |
| About 1 MiB | 3 | 1,034,105 | 0.027 | 4.964 | 0.134 | 35.023 | 1 | 17.78 / 12.93 / 10.74 | 17.78 / 12.93 / 10.74 |

**Medians:** smaller: script 0.036, style 1.253, layout 0.146, tasks
18.854 ms, one node. Larger: script 0.027, style 5.089, layout 0.134,
tasks 35.023 ms, one node. The larger first sample also had four style
recalculations and two layouts, versus two style recalculations and one
layout in its other samples; its eight mutated IDs are retained in the
table, not discarded. Smaller samples each had one of each.

There were no rectangle reads in these six end-following samples. Height
getter counts were 1/1/1 and 3/1/1 respectively; that is not a measurement
of the away-reader anchor scan. The reading design's away path is described
above from code, not assigned a timing from these end-following samples.

For a person, a delivery beside a chat holding about 1 MiB occupied the
main thread for 27.0–38.4 ms, median 35.0 ms: about **2.2 frames at 16 ms**
even though the median DOM change touched one node. These dev-build,
loaded-machine figures show a visible-frame risk, not a release-build
latency guarantee or the cost of a whole streamed answer.

## Earlier attempts, retained

All logs below were private under `target/`; the driver, deployments and
logs were removed after transcription. Names identify the original logs.
These were driver/setup findings, not failed tier stories.

| Attempt / log | Result and load information |
|---|---|
| `delivery-cost-measure.log` | `timeout: reader sign-in`; GET of the spent link only fetched the shell. Changed the driver to submit the spent link through ctl. Initial load 11.45 / 10.96 / 10.10. No delivery measured. |
| `delivery-cost-measure-2.log` | Driver read `document.body.innerText` before body existed. Added a null check. Initial load 9.46 / 10.57 / 10.01. No delivery measured. |
| `delivery-cost-measure-3.log` | `timeout: chat log`; synthetic popstate did not navigate the application. Changed to CDP navigation. Initial load 7.34 / 10.01 / 9.84. No delivery measured. |
| `delivery-cost-measure-4.log` | ctl rejected the probe's mixed-case request key with a usage error. Changed it to lowercase. Initial load 3.79 / 8.66 / 9.39. No delivery measured. |
| `delivery-cost-measure-5.log` | Fast fake finished before the page subscribed to live text; no text-tagged event appeared. Driver deliberately stopped at its next release. Initial load 2.68 / 8.01 / 9.16. No delivery measured. |
| `delivery-cost-measure-6.log` | `application.demo.llm.pace: unknown member`; reply pacing exists in the standalone fake document, not this embedded demo configuration. Used Task document deliveries instead. Initial load 1.01 / 5.39 / 8.03. No delivery measured. |
| `delivery-cost-measure-7.log` | Three smaller samples and one larger-history sample below. Larger page held only 645,483 bytes; it was not a whole-bound proof. A subsequent `ctl task show --until replied` did not complete; stopped that driver's own waiting ctl child (exit 143), then cleaned up. No deadline was raised. |
| `delivery-cost-measure-8.log` | `task amend needs --rev REV`; initial load 15.40 / 12.57 / 10.32. No delivery measured. |
| `delivery-cost-measure-9.log` | Omitting the optional key did not remove the revision requirement. Same refusal; initial load 9.57 / 11.45 / 10.06. No delivery measured. |
| `delivery-cost-measure-10.log` | Read the current revision before amending; three smaller samples below. Larger page held only 451,848 bytes / seven replies, so the driver's whole-bound assertion failed before sampling it. Added the final fixture's paragraphs, retaining approximately the same bytes, to permit normal history scrolling. |
| `delivery-cost-measure-11.log` | Six successful final samples above, all sixteen large replies held. |

The preliminary fixture had a long code line and one paragraph per reply.
Its measurements remain here, but are not interchangeable with the final
fixture or evidence at the whole bound. Load before equalled load after
for each preliminary sample, at the displayed precision.

| Attempt | Held bytes | Sample | Script ms | Style ms | Layout ms | Task ms | Nodes | Load before = after |
|---|---:|---:|---:|---:|---:|---:|---:|---|
| 7 | 129,198 | 1 | 0.060 | 0.771 | 0.136 | 17.279 | 1 | 9.61 / 6.72 / 8.22 |
| 7 | 129,250 | 2 | 0.042 | 8.998 | 0.177 | 31.570 | 1 | 10.36 / 6.93 / 8.28 |
| 7 | 129,302 | 3 | 0.052 | 2.282 | 0.154 | 34.274 | 1 | 11.22 / 7.16 / 8.35 |
| 7 | 645,510 | 1 | 0.271 | 5.934 | 0.776 | 40.160 | 4 | 10.46 / 7.14 / 8.33 |
| 10 | 129,198 | 1 | 0.044 | 0.929 | 0.158 | 21.418 | 1 | 9.68 / 11.02 / 9.97 |
| 10 | 129,442 | 2 | 0.036 | 0.851 | 0.131 | 24.033 | 1 | 9.68 / 11.02 / 9.97 |
| 10 | 129,686 | 3 | 0.034 | 0.930 | 0.139 | 13.994 | 1 | 11.31 / 11.34 / 10.08 |

Attempt 7 used incoming messages; attempts 10 and 11 used priority
amendments. The waiting ctl observation does not establish the listener
connection-limit hypothesis in `notes/common.md`: no ctl diagnostic
identified its cause, and the page still had queued deliveries.

## Decision

No fix landed. The repeated whole-window Markdown read and whole-tree
build/comparison are present, while the measured DOM output is already
small. Keeping parsed blocks or a conversation subtree across deliveries
would introduce the mechanism declined in `chat-reading.status.md`, 3;
it is expressly outside this task's small-fix allowance. It could avoid
rereading roughly 1.03 MB and constructing unchanged conversation nodes
per delivery in this fixture. The 27.0–38.4 ms task totals bound the entire
observed opportunity; they are not a measured Markdown-only saving.

Reducing page-wide bookkeeping or anchor scans would need invalidation
rules identifying which logs, collections and links a patch can affect,
and tests preserving Near, following and position keeping. These samples
do not isolate a needless repeated layout or unchanged whole-window DOM
patch to remove in a justified fix under 80 production lines. In
particular, the away-reader scan was read, not timed. No code, design,
budget, timeout or tier membership changed; no browser story was run.
