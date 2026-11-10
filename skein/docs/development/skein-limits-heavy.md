# Skein focused test costs: the repair's account

## The result

2026-11-07. The merged measurement commit is `79b4301d0f`, whose tree
matches merged integration `c4d240223d` at the final comparison. It combines
integration `4a4bb4df4e` with the scenario work through `7869730f38`.
The six task figures supply their own before/after pairs; they do not
share one pristine baseline. Measurements use unoptimized test builds,
16 CPUs, AMD EPYC-Genoa Processor, and Callgrind 3.24.0 through nextest,
one exact test at a time, with double spawn disabled. Listing processes
are excluded. Fail-stop sums the parent and its fixture children; other
successful profiles count one test process. No slow tier or walk ran.

The selected test is the largest remaining measured candidate in its
row's figures, rather than a ranking of unprofiled tests. The request-table
memory test and owned-payload fail-stop test become those candidates after
the original largest tests shrink; both originals were also remeasured,
below. The two calendar sweeps have no focused execution left, so their
focused after and trunk entries are zero, not empty-process measurements.
Seventeen selected or supplementary profiles passed; the process-world
profile failed and its complete count remains unknown. No figures file
is missing, but the process row and individually unprofiled tests are thin.

Each CPU estimate is the request's CPU s × task-after Ir / before Ir for
the selected test. It assumes that ratio describes the whole row. This
is particularly weak where only a few tests changed, a different test
became largest, or tests moved. Trunk Ir is shown separately and is not
substituted for task-after Ir in that formula. The gate's own CPU reading
comes after landing and is not this page's.

| Request row / largest remaining measured test | Request CPU s | Before Ir | Task after Ir | Merged trunk Ir | Estimated CPU s after |
|---|---:|---:|---:|---:|---:|
| `github-conformance` `tests`: `tests::github_conformance_queue_refusals_fail_one_fact` | 4.59 | 6,366,114,766 | 1,964,375,225 | 1,819,399,392 | 1.416324 |
| `skein-forge-machine-world::suite` `hand`: `hand::github::github_installation_loss::stale_full_after_deleted_is_discarded` | 1.92 | 33,219,510 | 30,917,000 | 28,168,403 | 1.786921 |
| `skein-forge-machine-world::suite` `machine`: `machine::owner_close_wins_or_follows_each_terminal` | 0.55 | 1,947,965,463 | 919,345,951 | 688,512,497 | 0.259574 |
| `skein-lib-tests::suite` `memory`: `memory::a_request_table_grows_within_its_held_and_peak_bounds` | 1.56 | 627,061,810 | 627,061,701 | 621,298,830 | 1.560000 |
| `skein-kv-tests::suite` `target`, `disk`, `recovery`: `target::recovery_stays_near_the_target_under_a_large_chunk` | 1.22 | 1,964,883,988 | 1,487,365,593 | 1,465,947,147 | 0.923508 |
| `skein-llm-world::suite` `memory`: `memory::thirty_two_unknown_arrays_exercise_the_compact_record_ceiling` | 0.76 | 2,836,038,627 | 2,836,038,591 | 2,466,680,139 | 0.760000 |
| `skein-conformance-real::suite` `conformance`: `conformance::ring::sockets::backpressure_stalls_a_sender_and_reading_resumes_it` | 0.76 | 81,011,411 | 61,798,657 | 61,821,184 | 0.579758 |
| `skein-world-tests::suite` `end_to_end`: `end_to_end::an_observer_can_be_abandoned_before_its_first_submission` | 0.69 | unknown | unknown | unknown | unknown |
| `skein-tls-world::suite` `memory`, `exchange`: `memory::failures_and_closes_in_every_state` | 0.82 | 2,415,439,865 | 2,415,465,585 | 2,391,793,507 | 0.820009 |
| `skein-conformance-sim::suite` `conformance`: `conformance::backpressure_stalls_a_sender_and_reading_resumes_it` | 0.39 | 1,341,576,696 | 465,793,151 | 465,792,670 | 0.135407 |
| `skein-io-world::suite` `scenarios`, `files`: `scenarios::a_server_that_does_not_demand_stops_its_peer` | 0.63 | 1,475,183,378 | 181,517,712 | 170,084,870 | 0.077520 |
| `skein-fail-stop-tests::suite` `fail_stop`: `fail_stop::owned_and_non_string_payloads_follow_the_default_report` | 0.36 | 680,491,706 | 680,804,926 | 680,495,306 | 0.360166 |
| `skein-protocol-world::suite` `worlds`: `worlds::one_end_closing_while_the_other_sends_leaves_both_settled` | 0.3 | 584,651,574 | 929,211,659 | 653,842,509 | 0.476803 |
| `skein-http-world::suite` `transcripts`, `requests`: `transcripts::every_transcript_decodes_to_its_expectation_whatever_the_cuts` | 0.44 | 1,158,931,907 | 601,450,632 | 549,666,918 | 0.228347 |
| `skein-ui` `tests::date` sweep: `tests::date::every_wall_day_round_trips_at_midnight` | 0.105 | 896,193,271 | 0 | 0 | 0.000000 |
| `skein-ui` `tests::phrases` sweep: `tests::phrases::every_representable_wall_day_matches_gregorian_rollover` | 0.125 | 1,090,220,691 | 0 | 0 | 0.000000 |
| `skein-json` `tests::text` sweep: `tests::text::three_and_four_byte_sequences_are_judged_at_every_boundary` | 0.125 | 939,051,364 | 43,028,377 | 43,027,731 | 0.005728 |
| `skein-http` `tests::gzip` sweep: `tests::gzip::a_huffman_code_is_complete_canonical_and_as_short_as_any` | 0.058 | 768,988,106 | 152,422,834 | 152,422,285 | 0.011496 |
| **Known subtotal (world row excluded)** | **14.713** | — | — | — | **9.401559 (estimate)** |
| **All rows, allowing the world row its unchanged 0.69 s** | **15.403** | — | — | — | **10.091559 (conditional estimate)** |

The instruction-derived known subtotal is **9.401559 CPU s (estimate)**
against 14.713 CPU s before, or **5.311441 CPU s saved (estimate)**.
The full total is unknown because the 0.69 s world row has no complete
instruction ratio. Allowing that row its unchanged request cost gives
**10.091559 CPU s after and 5.311441 CPU s saved (conditional estimates)**:
**0.688559 CPU s short** of the request's at-least-6 aim. This estimate
neither establishes that aim nor establishes a suite CPU total. In
particular it scales the unchanged request-table test over the memory
row, although three other tests there lose 98.94% of their instructions.
Their own request-weighted saving is about 1.36 CPU s; that different
calculation is evidence about those tests, not a replacement for the
requested row estimate. No budget decision follows from either estimate
(`skein/docs/foundation/testing-strategy.md`, 8.1).

Supplementary original-largest measurements preserve the reductions
that a change of representative would otherwise obscure:

| Test | Before Ir | Task after Ir | Merged trunk Ir |
|---|---:|---:|---:|
| `memory::an_intake_with_a_large_capacity_used_lightly_holds_what_it_holds` | 6,720,266,382 | 111,498,172 | 84,522,813 |
| `fail_stop::a_panic_on_a_second_thread_aborts_the_process_after_its_report` | 1,043,350,176 | 348,430,893 | 348,256,005 |

The real-ring backpressure count is **61,798,657 → 61,821,184 Ir**,
+22,527 (0.0365%). The new profile still has exactly 40,370,907 Ir in
`scenarios::pattern`; `Run::start` is 7,984,918 Ir. No intervening merge
changes the real conformance, shared conformance or shell sources since
`e5f7735baf`. The unchanged-source comparison and asynchronous kernel work
do not identify a source merge as the cause of this small increase;
it remains run-dependent accounting/harness variation, not a repaired
regression. Other closing counts are lower than their paired task-after
counts. Those gains include other work already merged: accounting and
wire changes `3d8dcd2669`, `02de455df1`, `f89237da2f`, TLS fill copying
`0621c0e95a`, and JSON string handling `bc0d8f4c4f` and `ae9260a7e2`.
They are not claimed as this plan's fixes.

The process-world attempt stopped at **3,586,775 parent Ir**, an incomplete
profile, after unsupported syscall 424 (`pidfd_send_signal`) and the
unsubmitted observer's unchanged 2 s deadline. It observed 2.028392410 s
elapsed, 0.034030436 s thread CPU and zero reported runnable wait. A child
profile was incomplete too. None is a complete-test count or a native
test result. The task's native before/after runs passed; the closing
attempt adds no new native observation and makes no timeout change.

## What moved to fuzzy and which counts fell

Four tests move because they compare generated cases with a reference,
under `skein/docs/foundation/testing-strategy.md`, 8, rather than because
of their cost. Every assertion moves with them. The calendar sweeps cover
**213,504** representable days, including the partial last day; the plan's
survey text says 213,503, but the implemented sweep includes that last day.
Fuzzy gains the complete text round trip (902,606,331 Ir) and Gregorian
phrase comparison (1,900,707,510 Ir): **2,803,313,841 Ir** together. Focused
keeps named leap/century/year-end dates and the largest instant; the named
midnight test gains each day's last-nanosecond check. Both moved holders
and both focused holders passed at `4327ac3b36`.

HTTP's generated-cut comparisons move intact: **188 response worlds and
1,808 request prefixes**, **1,754,463,334 Ir** after, measured together at
0.254003 native CPU s. Fuzzy gains every reference-reader comparison;
focused retains named incomplete-event behaviours. Both moved holders
passed at `e5f7735bafee9614d7040416ccca19be15a0d4fa`. The four moves add
**4,557,777,175 Ir** to fuzzy, rather than erasing that work.

Counts fall only where the unchanged assertions still distinguish the
behaviour. The figures record a restored mutation and its failure for
every reduction:

- GitHub keeps **57 facts and 284 requests in order** in a whole run.
  Focused tests become 29 → 28. Ten whole runs become three; eleven
  queue-refusal rows use six fixtures, each with at most one refusal per
  fact. Explicit queue runs total seven; queue-group executions including
  whole runs fall 21 → 10. Framing's two runs use six log facts each,
  rather than all 57; epoch's three runs use one fact each. Organization
  discovery runs the queue group alone. Exact reasons, later facts,
  transcript/provenance checks and cleanup remain. Two separate fault
  tests are held by the combined four-fault continuation test, passed at
  `726e06247ae00d3076bd20a1a6d2f126d76d1813`.
- Ring and lightly used queue loops become **1,000,000 → 4,096** turns,
  every turn metered. Two pushes/pops pass 65,536 bytes through a
  1,024-byte ceiling: at least 63 wraps. The queue capacity stays
  15,000,000. Intake capacity becomes **1,048,576 → 16,384** and turns
  **1,000,000 → 16,384**: sixteen ceilings, with twice-capacity lifetime
  throughput. Mutation checks still fail at 1,536 bytes against 1,024.
  Median and other container counts and capacities stay.
- UTF-8 comparisons become **1,802,240 → 70,400** by taking ten boundary
  values for the second byte too. All continuation limits and outside
  neighbours still compare with the standard library; random text checking
  remains in the JSON fuzzy writer. Huffman seeds become **20 → 4**,
  two per rarity regime, keeping completeness, canonical decoding, width,
  frequency and reference weight checks; deterministic Fibonacci counts
  still force both width cuts. No public interface is added for the sweep.
- KV target/recovery rows become **3,300 → 2,500**, each 1,040 encoded
  bytes. Two copies are 5,200,000 bytes, still above unchanged recovery
  4,718,592 and snapshot 2,359,296 bounds. Packing still needs three chunks
  (1,008, 1,008, 484 rows); snapshot peak is 2,121,364 and recovery transient
  1,600,220 bytes. The shared-target mutation fails recovery's read bound;
  packing-only mutation fails packing, but did not fail recovery by itself.
- The largest IO store keeps **65,536 bytes and a 132,257-byte peak** under
  the 170,000 ceiling. Reads become **2 → 1,024 bytes**, reducing 32,768
  data reads to 64, plus EOF. Small stories and fuzzy file cases retain
  two-byte reads. Named IO scenarios use **seven → four** seeds: calm
  `[1,2,3,4]` → `[1,2]`, chaos `[5,6,7]` → `[5,6]`; all expectations stay,
  and fuzzy sweeps the same scenarios with 150 of each kind.
- TLS close-state seeds become **60/10 → 40/6**, twice the smallest
  prefixes observing every asserted wait. The memory test keeps both
  protocol versions, both injected endings, all 30 offsets and its ceiling.
  LLM keeps **32 arrays × 1,900 values**, a 4,100,000 ceiling and observed
  peak above 2,000,000; no dense-array count falls.
- Simulated conformance calm seeds become **16 → 4**. Smoke remains four,
  chaos 200; explicit races and their pairing checks remain. The cancellation
  scenario falls **112 → 28** runs and retained seeds show both completion
  orders. Fuzzy still draws faults over the same shared scenarios. Real
  conformance changes no scenario count.
- HTTP response goldens use **four → two** seeds (**188 → 94** runs),
  request goldens **five → two** (**180 → 72**). Every head, outcome and
  body check remains. Event streams keep four seeds for chunk sizes
  1, 7, 64 and 4096; moved prefix tests keep their original counts.
- Protocol streaming uses **12 → 2** seeds, retaining positive ID tally
  and the complete event count. Multi-call uses **eight → four**, asserting
  both two and three calls occurred. Reset uses **20 → 4**, with resets
  also drawn and asserted in fuzzy. Replay uses **four → two** seeds,
  comparing both complete runs each time. Closure instead grows
  **30 → 46**: the first four resets require 23 seeds, doubled for margin;
  `reset > 3` stays. Holding tests retain 3, 3 and 6 seeds.
- Fail-stop uses **five → three fixture deaths**, retaining the full
  report assertions: borrowed/unnamed with `full`, owned/named with unset,
  opaque/named with `1`. Repeated stderr-overflow unwinds become
  **20 → 3**: first use, reuse, and no accumulation, each checking the exact
  diagnostic. Other process containment and observation starts stay.
- Forge machine retains **76 close worlds** and every scenario count.
  Harness settling uses sixteen quiet rounds, counting fragmented trace
  growth as progress, with the old 1,000-tick bound and exit conditions.
  Close-test settling calls fall **176,286 → 10,609**; this changes
  scheduling, not the named behaviours or timers.

## Each row

All pairs here are the tasks' instruction counts; merged representatives
are in the result table. The plan's table retains all **144** touched or
profiled test rows, including unknowns and unchanged samples.

- **GitHub:** a whole run falls **2,557,261,726 → 1,670,813,970 Ir**
  (34.7%). Validate a text stream once and use standard text search,
  register identical secrets once preserving the first kind, pad scratch
  writers in 8 KiB instead of 128-byte chunks, and search before copying
  absent opaque values. Inclusive `write::check` falls 600,478,062 →
  241,639,290, fake `answers::finish` 421,858,593 → 33,016,425, masking
  245,264,993 → 106,315,035. Queue refusal falls **6,366,114,766 →
  1,964,375,225** (69.1%): shared causes first leave 4,292,311,878,
  then six fixtures remove another 2,327,936,653. Setup-only falls
  90,195,918 → 69,910,384; masking-only 2,769,528 → 2,589,454.
  Framing, epoch and discovery after counts are 284,010,002, 406,786,398
  and 473,310,731; their separate before counts are unknown.
- **Forge hand:** the largest sample falls **33,219,510 → 30,917,000**;
  the Forgejo median **16,144,095 → 13,167,685** and GitHub routing
  **5,473,654 → 4,471,441**. Visiting peers in place removes key-list
  copies, and a shorter quiet tail removes repeated inspection. RSA stays.
- **Forge machine:** close falls **1,947,965,463 → 919,345,951**;
  returned-answer median **41,177,459 → 35,374,946**, with settling
  5,934,129 → 97,254. Recorded start stays on clock-driving `run_for`,
  **9,841,209 → 9,851,503** (+0.10% bookkeeping). The direct protocol
  reader falls **57,923,886 → 51,058,287**, its installation settlement
  stopping at 177 ticks. A new fragmentation regression holds progress.
- **Lib memory:** the three loops total **13,294,800,233 → 140,951,059**
  (98.94%). Queue **3,287,265,890 → 14,725,579**, ring **3,287,267,961 →
  14,727,308**, intake **6,720,266,382 → 111,498,172**. The cause is
  repeated metered work beyond what wrap/reclamation needs. The now-largest
  request table stays **627,061,810 → 627,061,701**; the median pair stays
  **3,364,663 → 3,364,531** and **9,460,427 → 9,460,251**. Per-operation
  accounting assertions remain in all 28 memory tests.
- **KV:** CRC-32C and row copying dominate; smaller justified fixtures
  yield recovery **1,964,883,988 → 1,487,365,593** and packing
  **1,420,279,847 → 1,075,059,510**. Tiny-store disk stays
  **359,990,914 → 359,989,379**. A four-byte checksum prototype worsened
  recovery to 1,766,352,475 and packing to 1,261,092,713 at the same
  2,500 rows and was discarded; no product optimization ships here.
- **LLM memory:** dense arrays stay **2,836,038,627 → 2,836,038,591**,
  dominated by JSON collection, tokenization and replay conversion.
  Preparation median stays **16,721,823 → 16,721,925**. Smaller arrays
  would cease to exercise the stated compact-record ceiling.
- **Real conformance:** reap completions and take a nonblocking backend
  turn before accounting, so completed operations need no new deadline.
  Backpressure falls **81,011,411 → 61,798,657**, wrong-state median
  **7,059,319 → 1,578,979**, file case **12,698,275 → 2,384,777**.
  File `wait_begins` falls 10,579,511 → 397,705; unfinished asynchronous
  operations retain deadline accounting.
- **Process worlds:** complete instruction pairs are **unknown** because
  profiling cannot complete the process paths. Native supporting totals
  are **2.099656 → 1.356667 CPU s**, not a causal instruction comparison;
  the request's row was 0.69 s on different loaded runs. Exact-diagnostic
  unwinds fall 20 → 3; eight observer starts, one blocked tree, and three
  exit-containment starts stay. The closing observer profile also fails.
- **TLS:** real crypto remains; memory stays **2,415,439,865 →
  2,415,465,585**, while close-state exchange falls **1,142,402,294 →
  716,734,618** at the smaller complete wait census. Single-close median
  stays **45,660,336 → 45,661,924**. Tiny increases are TLS/startup
  variability, not claimed savings.
- **Simulated conformance:** justified seed reduction gives backpressure
  **1,341,576,696 → 465,793,151**, unread-close median **11,122,646 →
  5,328,112**, explicit races **43,542,271 → 11,529,157**, file case
  **30,712,809 → 13,413,326**. Pattern generation remains the next cause.
- **IO:** coarser reads give largest-store **1,292,419,870 → 17,117,922**
  with identical peak. Backpressure falls **1,475,183,378 → 181,517,712**:
  four seeds first leave 981,587,877; slice prefix comparison then removes
  matching-byte rescans (`common` was 800,759,652). Conflict median stays
  **3,152,037 → 3,154,853**. A focused regression checks matching, empty,
  overlong and first-mismatch offsets.
- **Fail-stop:** three borrowed-payload deaths become one,
  **1,043,350,176 → 348,430,893** summed parent/children Ir. The two
  owned/opaque deaths stay **680,491,706 → 680,804,926** and now form
  the largest test. Native row totals **0.449436 → 0.309627 CPU s** are
  supporting observations; symbolized reports remain the cost.
- **Protocol:** closure grows **584,651,574 → 929,211,659** (+58.9%)
  for its reset coverage margin. Other counts fall: wire-reset
  **160,908,075 → 30,676,450**, streaming **225,104,587 → 34,719,646**,
  multi-call **215,449,640 → 125,094,335**, replay **118,309,352 →
  86,948,501**. Slow-consumer stays **273,156,432 → 273,159,000**.
  Native row totals are **0.410156 → 0.417854 CPU s**: no row saving
  is claimed from those observations.
- **HTTP worlds:** response goldens fall **1,158,931,907 → 601,450,632**,
  requests **304,201,086 → 143,144,308**. Event streams stay
  **434,273,572 → 434,273,604**. The two cut comparisons retain
  **583,167,779 → 583,168,024** and **1,171,294,679 → 1,171,295,310**
  in fuzzy; the move changes focused membership, not their work.
- **Calendar sweeps:** focused loses **896,193,271** and **1,090,220,691**
  Ir, and fuzzy gains **902,606,331** and **1,900,707,510**. The larger
  phrase count includes the world's counting allocator. Calendar rendering
  and every comparison remain; this is a move, not an algorithmic saving.
- **UTF-8:** boundary-only second bytes give **939,051,364 → 43,028,377**;
  repeated `checked` was 579,302,752 inclusive Ir. Both E0 lower-limit and
  F4 upper-limit mutations still fail at their precise boundary examples.
- **Huffman:** four representative seeds give **768,988,106 →
  152,422,834**. Repeated sorting in the plain reference builder was
  542,600,173 inclusive Ir. Incomplete-code mutation still fails at
  32,736 filled slots against 32,768.

## Found and not fixed

Every finding from the six figures is retained here, ordered by the
largest quantified inclusive cause where comparable. These costs overlap
and cannot be added. Original-loop and fuzzy counts are labelled; they
do not claim that all that work remains in focused. Unknown and incomplete
process figures follow the quantified causes.

- **Exhaustive calendars in fuzzy:** 1,900,707,510 phrase-sweep and
  902,606,331 round-trip Ir after moving. Exhaustive calendar comparison
  and rendering are their behaviour; reducing that work would need an
  equivalent calendar/rendering optimization with every comparison retained.
- **Lib metering/accounting, original loops:** intake `Counting::alloc`
  1,694,291,811 and `dealloc` 1,488,221,584; queue `Meter::start`
  740,000,740, `end` 783,000,783 and `check` 862,001,074 Ir. Loop counts
  fall here, but every retained turn stays metered. Optimizing these shared
  mechanisms requires allocator/meter changes and their accounting tests;
  that work belongs to the separate heap/wire-cost repair.
- **KV checksum:** recovery still 952,029,012 and packing 634,821,686 Ir
  in CRC-32C. The discarded table prototype made both worse. Another
  algorithm must preserve Castagnoli bytes, block and tail verification
  without unoptimized conversion/lookup overhead.
- **LLM JSON/replay and metering:** `Json::from_bytes` 789,685,779,
  collector input 780,667,328, tokenizer 743,105,245 and parse
  697,153,999 Ir, overlapping. Retained replay conversion needs product
  changes outside this task; JSON and heap accounting are separately owned.
  Fixture `events` 21,317,574 and `response` 15,204,682 together estimate
  only 0.0046 CPU s; median helpers total 362,940 Ir. Changing them or the
  retained 37-byte HTTP chunks cannot reach the stop rule. Formatted world
  traces remain with the separate shared-helper repair.
- **TLS memory crypto and settings:** key-exchange completion 526,803,252,
  key generation 309,033,251 and verification 253,164,773 Ir exercise real
  TLS. Server/client configuration 44,746,105/8,759,776 together estimate
  only 0.006 CPU s, so meter ownership and settings construction stay.
  TLS 1.3 still finds a distinct failure wait at offset 23; removing only
  three of 30 offsets with margin estimates 0.027 CPU s before setup.
  A cheaper implementation must preserve the real handshake and that census.
- **Fuzzy phrases accounting:** 440,549,378 alloc and 384,535,557 dealloc
  Ir account for much of the increased moved-sweep figure. That allocator
  belongs to the separate heap/wire repair; changing it needs its own
  exact-accounting tests. The sweep's guide stays unchanged.
- **Forge machine crypto:** 397,066,112 Ir in rustls message processing
  remains in the close test. Shared settings cannot remove its handshakes.
  Repeated server/trust setup was 17.33 million baseline Ir, only 0.0025
  estimated CPU s; further work must optimize real TLS rather than omit it.
- **HTTP byte/chunk work:** response `World::below_acts` retains
  373,016,067 and event-stream `sse::pump` 145,287,917 Ir. One-byte body
  input and all four named chunk forms are the checks. Metering adds
  83,855,016/80,838,246 inclusive Ir and belongs to the separate repair.
  Further gains need a cause-specific kit/decoder change retaining those forms.
- **Protocol generators/JSON:** closure `scenario::partway` 366,935,334,
  writer `put_string` 253,984,137 and `escaped_len` 178,199,210 Ir overlap;
  slow-consumer decoding is 86,420,843. JSON product code belongs to the
  separate string repair. Optimizing shared generation/encoding must keep
  the worlds used by fuzzy and all holding assertions at 3, 3 and 6 seeds.
- **Fail-stop symbolization:** a borrowed fixture's backtrace display was
  346,453,769 Ir, 99.88% of that child. Three payload/backtrace forms still
  print symbols and abort. Omitting symbols or reusing the dying process
  would change the check; a cheaper symbolizer must retain the full reports.
- **Conformance pattern generation:** real backpressure retains 40,370,907
  and simulated backpressure 324,802,733 Ir (65.33%/69.73% in task-after
  profiles). A shared precomputed fixture or another pattern algorithm
  needs both backends and fuzzy to hold identical bytes. CALM's justified
  minimum and unchanged SMOKE leave this cost; another seed cut hides it.
- **Request-table drain and LRU:** `RequestTake::next_out` scans admission
  order, 324,144,260 of 627,061,810 Ir. At measured 0.090 native CPU s,
  eliminating it estimates 0.0465 s, below the stop rule. A fix needs a
  traversal position across bounded visits preserving order and lifecycle.
  LRU insert 104,152,667 of 239,813,089 Ir at 0.062 s estimates 0.027 s;
  no repeated setup above the rule was found. Counts remain nine capacities
  and the named lifecycle/size scenarios.
- **GitHub transcript/leak checks:** `write::check` still 241,639,290 and
  `Mask::body` 106,315,035 Ir in a whole run. Tests read those files; newly
  registered secrets must be checked against retained bytes. Further changes
  must preserve exchange, flush/output scans, every mask category and
  first-appearance numbering, rather than remove those observations.
- **GitHub JSON:** whole-run collection/parsing is 203,092,901 Ir and
  belongs to the separate JSON product repair. Four redundant merge-repeat
  parses are about 803,000 Ir per whole run, about 0.001 CPU s across all
  ten original runs; repeated merged-queue parsing is 964,311 Ir over the
  final refusal test, about 0.00013 s. Avoiding those parses would need
  retained parsed responses; both estimates are below the stop rule.
- **Huffman reference sorting:** the entire retained test is 152,422,834
  Ir; removing all of it estimates only 0.012 CPU s. A different plain
  reference builder could avoid repeated sorts, but must retain independent
  weight checking; no further change reaches the stop rule.
- **GitHub RS256:** 90,067,998 whole-run and 22,519,347 setup-only Ir.
  The fake already caches a verified JWT and checks claims at each use.
  Signing, renewal, wrong credentials and expiry remain real; replacing
  the crypto implementation lies outside this repair and build settings stay.
- **IO residuals:** backpressure retains 80,167,477 Ir in `Proc::iterate`,
  including IO stages and metering; removing its entire remaining test
  estimates only 0.020 CPU s. Largest-store referee 4,208,798 and
  `read_whole` 176,411 Ir estimate 0.00072/0.00003 s. Further IO/referee
  optimization needs the same backpressure, bytes and peak assertions.
- **KV duplicate-key scans:** recovery/packing `has_later` costs
  51,242,700/49,994,515 Ir; their combined request-weighted upper bound
  is 0.017 CPU s. A last-operation index per batch must preserve validation
  and refusal order. Disk/recovery commit counts stay because no smaller
  count was established from their assertions.
- **KV observation:** `World::files` is 36,975,270 Ir, 10.27% of the
  tiny-store baseline. `Observed::check` reads snapshot-start bytes 6..14
  as well as log length. A metadata/short-header observer could avoid
  whole-file copies while keeping that assertion. The retained 2,000
  commits show three snapshots, last start 1,948, maximum log 64,842 bytes
  under 65,536 plus an 8,192-byte segment.
- **GitHub simulator:** `World::wait` is 22,710,261 whole-run Ir. The
  remaining passes drive requests, domain calls, delivery and CI clocks;
  no idle spin was found. Removing that work needs a broader simulator or
  protocol change retaining every clock observation.
- **Forge hand RSA:** two signings cost 22,621,385 Ir, 73.17% of the
  largest sample after. `limits::access` no longer signs: it parses
  (`skein/docs/design/net/tls.md`, 4), so the remaining signing is `Jwt::bearer`'s.
  Retaining a parsed key would depart from
  the parse-at-each-signing contract (`skein/docs/design/net/tls.md`, 4).
- **Forge listener/fixtures:** listener settlement is 15,287,984 baseline
  Ir, about 0.0022 CPU s; `locate_script` 1,106,458 and `deadline_rows`
  55,232. None reaches 0.05 s. A further harness change would have to
  preserve listener waits and per-row fixtures, so these stay.
- **Forge hand shared trace/accounting:** median `Wire::answer`
  2,745,941, `Wire::take` 1,291,648, formatting 3,115,557, allocator alloc
  2,577,962 and dealloc 2,190,174 Ir overlap real delivery and accounting.
  Optimizing these needs the separately owned shared trace/accounting work.
- **Conformance contract copies:** lifecycle slice-to-box work is
  2,609,792 of 101,039,244 Ir (2.58%). Returned buffers, receive tails,
  names, entries and spawn pipes are read by `handed_back`. Recycling the
  returned send box removes only a subset, below 0.05 CPU s; a change must
  retain those byte comparisons.
- **Real conformance pending waits:** file `Deadline::watch_process`
  remains 257,799 Ir. Nonblocking submission cannot promise completion;
  a per-token readiness probe needs a backend-interface change. Existing
  patience, scheduler accounting, idle-listener path and all three
  deadline checks stay.
- **Forge hand request inspection:** median `request_lines` is 88,629
  Ir, estimating 0.0105 CPU s even across the whole 1.92 s row; selected
  GitHub `request_bytes` is 2,283 Ir, estimating 0.0008 s. Avoiding repeated
  inspection would need equivalent cached assertions. Waiting-refusal
  prefix and both twenty-call scenarios stay; no reduction clears the rule.
- **Process worlds, incomplete counts:** base profiles stopped at
  2,328,864 Ir (overflow), 4,788,865 (exit), 4,064,667 (costliest observer)
  and 2,211,899 (median), never full test counts. Callgrind reports
  `Child(Other(95))` on process paths and unsupported syscall 424; the
  closing observer also fails, as recorded above. Complete comparisons
  need a profiler supporting those operations, not a changed deadline or
  containment path. Eight observer fixture starts, one median tree, three
  exit-containment and three overflow starts remain. Supervised stderr
  overflow keeps 262,144 zero bytes because no pipe-capacity constant
  justifies a smaller input.
- **Process startup, unisolated:** included in all figures, unchanged by
  this plan and assigned to other work. Protocol profiles put only
  0.9–1.1 million Ir outside each test body; they do not measure all of
  the request's 17 ms native floor. Median and unused-container profiles
  still include startup. Unused/list-clone capacities allocate nothing
  up front, so their capacities stay. Further work needs process-cost
  measurement independent of these tests' named behaviours.

## What stays heavy and why

The row estimate exposes the largest retained costs, including regressions;
none is a proposal about a budget. Focused retains memory under named
ceilings, named close/holding behaviours, real crypto, golden byte streams,
and process death reports (`skein/docs/foundation/testing-strategy.md`, 8).

The LLM ceiling still needs 32 dense arrays and **2.836 billion task-after
Ir** (2.467 billion merged); TLS memory still needs both protocol versions
and every injected ending at 30 offsets, **2.415 billion** (2.392 billion
merged). KV recovery retains checksums and a store too large for its
**4,718,592-byte** transient bound, **1.487 billion** (1.466 billion merged).
Making these fixtures small enough to fit under their ceilings without
reclamation would stop checking the rule they name.

GitHub's retained whole run still holds **57 facts, 284 ordered requests**,
**1.671 billion Ir**, and its six-fixture refusal test **1.964 billion**
(1.819 billion merged); six dequeue rows require six runs. Forge close
keeps all **76 worlds**, **919,345,951 Ir** (688,512,497 merged), including
real TLS. Protocol closure grows to **46 worlds, 929,211,659 Ir**
(653,842,509 merged) to retain `reset > 3` with doubled margin.

HTTP's response golden still reads bodies one byte at a time,
**601,450,632 Ir** (549,666,918 merged), and event streams retain all four
named chunk forms, **434,273,604 Ir**. Fail-stop retains three symbolized
fixture deaths: the largest two-form test costs **680,804,926 Ir**
(680,495,306 merged), and the one-form test **348,430,893 Ir**
(348,256,005 merged). A report without symbols or a fixture that survives
would check a different behaviour. Process-world complete Ir remains
unknown; its observer's eight fixture starts name distinct backend,
terminal/pipe and queued/unsubmitted combinations and remain.

The request-table test remains **627,061,701 Ir** (621,298,830 merged):
nine capacities, held/peak bounds and lifecycle/order checks. Its next
identified scan estimates **0.0465 CPU s**, below the repair's stop rule.
Fuzzy keeps the exhaustive calendar and reference-prefix work,
**4,557,777,175 Ir** in the task figures, with every assertion. No test,
budget, timeout or build setting changes in this closing account.
