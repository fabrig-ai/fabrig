# under-causes: shared work below the worlds

2026-11-07 UTC, base `baaa0068d3`, 16 CPUs, AMD EPYC-Genoa Processor,
Valgrind 3.24.0. The workspace's default dev/test profile is unchanged in
every arm, including its inherited `skein-lib` and `skein-codegen`
`opt-level = 3` package overrides. No build setting or override was added.
Test consumers remain unoptimized. The sweep bases are the
three predecessor tasks' final arms. R, P and G name the exact stories in
`temper/docs/plan/limits-fuzzy/notes/real-loop.md`; G is focused, and contributes
no CPU seconds to the fuzzy-suite estimates below. Real-loop extrapolations
use the request's 57 CPU s for its 18 fuzzy stories. They screen candidates;
footprints overlap and are not additive savings.

## Causes, ranked before changes

Only an eligible removable cause gets a fix rank. Estimated seconds below
are prospective ceilings or explicit estimates, not measured savings.

| Fix rank | Candidate / location | Share and arithmetic | Estimated CPU s out of temper fuzzy | Decision / outcome |
|---|---|---|---:|---|
| 1 | Append bookkeeping, `skein-lib/src/list.rs` | Public-wire: `push` 25.53%, `grow` 12.29%, `room` about 5.4%, overlapping. Estimate 40 Ir removed per append: 20,894,402 × 40 / 18,975,274,038 = 4.40%. R run 1 has 4,615,923 appends: 4,615,923 × 40 / 21,614,835,263 = 0.854% | 2.767242 × .0440 + 57 × .00854 = **0.61**, extrapolated; no extra credit for the other worlds | Fixed: **2.2978%** of public-wire; **0.063586 CPU s** instruction-scaled there. No demonstrated real-loop saving |
| 2 | Measure fixed-width named records, `skein-codegen/src/emit_codec.rs` | Public-wire `PublicQueued::measure`: 1,135,984,640 / 18,975,274,038 = 5.99%, plus its loop; almost all can become count × fixed width | 2.767242 × .0599 = **0.166**, before paying for the constant-time replacement | Fixed: **9.9101%** of the list arm; **0.248299 CPU s** instruction-scaled there |
| — | Repeated tool-schema admission, `smith-protocol-llm/src/prompt.rs`, `object` | R 926 calls / 923,283,992 Ir = 4.27%; P 552 / 590,479,900 = 2.72%; G 313 / 317,920,801 = 2.73%. Includes parser and admission, not only parsing | Ceiling 57 × .0427 = **2.43**; actual removable work smaller | Retaining parsed schemas changes the producer's representation or needs a cache keyed by charter/namespace/grants. Not a small change under the current design |
| — | Independent JSON validation, `skein-json::parse` and callers | R 9.10 / 8.86%; P 7.20%; G 6.70%. R fake-provider request admission alone is 1,133,498,654 Ir, 5.24%; P 710,193,829, 3.28%; G 382,319,076, 3.28% | Full-parser ceiling 57 × .0910 = **5.19**; **0** identified removable independent validation | Required validation of separate peer requests remains |
| — | Whole-prompt measurement, guard/framing/send | R all `client::measure` 3.49 / 3.29%; P 2.07%; G 2.01%; `observed_prompt` itself 980 / 637 / 343 Ir in R/P/G | Even deleting every measurement in R gives 57 × .0349 = **1.99**; handing off a witness saves only part | Below real-loop bar. Guard whole/prefix, framing allowance and connection admission have different purposes. Observation is disabled in these stories |
| — | Fixture configuration and admission, `own_defaults`, `landing::admit` | R entire envelopes 0.71% + 0.45%, upper bound on any duplicate parsing | 57 × .0116 = **0.66** ceiling | Below real-loop bar; all twelve admission checks stay |
| — | Replay write/parse and other small LLM JSON callers, `translate.rs`, `dialect.rs` | R `translate::replay` 1,555,032 Ir, 0.0072%; P 1,974,180 Ir through replay JSON admission, 0.0091%; G 553,888, 0.0048% | 57 × .000091 = **0.0052** ceiling | Below real-loop bar. No trial of the previously discarded writing-pass measurement |
| — | Byte search, `skein-lib/src/bytes.rs` | R `find_from` 0.22%; P 1.27%; G 2.86% | Fuzzy ceiling from R/P: 57 × .0127 = **0.72**; even using focused G as a screen gives 57 × .0286 = **1.63** | Below real-loop bar before retaining the necessary search |
| — | Plain-string scan, `skein-json/src/string.rs`, `after_plain` | R 1.94%; P 1.23%; G 1.15% self | 57 × .0194 = **1.11** ceiling | Below real-loop bar. A word scanner also changes json.md, 7's selected shape |
| — | Policy deadline scan, `temper-service/src/system.rs` | P 9.33% raw; 289,741 calls. Slowdown 222.771610 / 9.135526 = 24.385; scaled calls about 11,882 and share .383% | 57 × (.0933 / 24.385) = **0.218** ceiling | Timer-scaled, below real-loop bar; readiness and deadline checks stay |
| — | Allocation headers / core copy checks | R `read_unaligned` and `write_unaligned` have 3,015,640 / 3,015,669 calls, all from counted deallocation/allocation; their check edges total 1,055,479,060 Ir, 4.88% | Header-check footprint 57 × .0488 = **2.78**; **0** permitted layout/check removal | Header layout and checks stay. No independent dispensable copy was found |
| — | Iterator leaves | R slice `Iter::next` self 2.07%, `Enumerate::next` self 1.33%; P 2.21% and 1.11% | Combined R footprint ceiling 57 × .0340 = **1.94**, before retaining traversal | Spread across B-tree search, referee observations, fake LLM cache/cues, JSON builder and HTTP framing; not a single byte-search loop |
| — | Public-wire remaining scalar codec work | Decode/measure/encode envelope 96.29%; writer 8.55%, scalar reader calls 4.21%, overlapping | 2.767242 × .0855 = **0.237** writer ceiling; × .0421 = **0.117** reader ceiling; **0** identified dispensable reads/writes | Records and field-specific shortage/bound errors remain; checks are not removable work |
| — | One-process fake person, native startup/iteration, service iteration and root formatting | Kill/power: person 10.32/4.65%; startup 5.49/8.44%; client iteration 10.24/10.46%; service iteration 22.09/22.91%; root formatting 3.22/3.29% | Respective footprint ceilings: 4.658312 × share + 4.572799 × share = **0.69 / 0.64 / 0.96 / 2.08 / 0.30**; **0** demonstrated removable whole operations | Each real iteration/session and each of twelve complete secret-leak root checks stays |
| — | Engine store / restore / domain / full-state hash | Walking-0, escalation, roles: store 12.03/10.33/15.55%; restore 7.52/6.70/11.93%; domain step 14.02/11.92/16.06%; hash 30.54/28.23/8.71%, overlapping | Using 6.067245 s for that measured set gives conservative max-share ceilings **0.94 / 0.72 / 0.97 / 1.85**; **0** whole traversal/restore/step identified removable | Independent world roots, real codec cold readback, and complete frozen-state hash traversal stay |
| — | Predecessors' stopping points | Public scratch refill at most 1.45%; native-link limits 1.60/1.70%; engine old-trace equality 0.42–1.17% | Ceilings: 2.767242 × .0145 = **0.040**; 4.658312 × .016 + 4.572799 × .017 = **0.152**; 6.067245 × .0117 = **0.071** | Each already below 2% of its sweep; actual gain smaller than the whole footprint |
| — | Smaller public-wire setup | `drawn` at most 0.50%; unwind envelope 0.012% | 2.767242 × .005 = **0.01384**; × .00012 = **0.00033** | Below 2%; unchanged |
| — | Smaller one-process setup | Old commit observations at most 1.4%; root text outside service formatter about 1%; full snapshots at most 0.02% | (4.658312 + 4.572799) × share = **0.129 / 0.092 / 0.00185** | Below 2% even before retaining new entries and required checks; unchanged |

## Base real-loop runs

`PATIENCE` is locally 3,000 s in every profiled and paired native arm. No
call-specific timeout or answer bound needed a change. R stopped at
`skein-fake-peers/src/forge/observe.rs:15`, `frozen record count fits`, in
both profiles; those figures are partial, never successful complete-story
figures. Both native R runs pass. P and G profiles each report 1 passed.

| Story | Draws | Arm | Run | Ir | Native CPU s (user + system, descendants included) | Profile wall s | Result |
|---|---:|---|---:|---:|---:|---:|---|
| R | one repair story | Base | 1 | 21,614,835,263 | 7.602190 | 216.919233 | partial, forge observation cap; native passed |
| R | one repair story | Base | 2 | 21,985,351,066 | 7.605484 | 232.435991 | partial, same cap; native passed |
| P | one case 6.4 story | Base | 1 | 21,677,906,409 | 6.315805 | 222.771610 | passed |
| P | one case 6.4 story | Base | 2 | 21,366,608,783 | 6.300049 | 224.640846 | passed |
| G | one connected landing story | Base | 1 | 11,651,837,055 | 3.297763 | 127.056394 | passed |
| G | one connected landing story | Base | 2 | 11,653,418,240 | 3.317145 | 121.288643 | passed |

Base spreads relative to the smaller Ir: R 1.71%, P 1.46%, G 0.014%.
Real `sh` children execute outside Callgrind; paired native CPU includes them.
R run 1 has four calls from ring preparation into the real process spawn;
P and G run 1 have zero. Hosted agent/git/CLI program spawns are in-process
and must not be counted as native children.

## Fix 1: list append bookkeeping

`List::push` compares its native buffer length to the native form of its
logical cap once. It grows a full buffer in place, rather than asking `room`
to convert/subtract the length and calling a helper even when no growth is
needed. The same `grown`, `reserve_exact`, cap refusal and Vec append remain;
no layout, logical capacity, growth floor, allocation size or held item changes.

| Sweep | Worlds or inputs drawn | Arm | Run | Ir | Native CPU s | Result |
|---|---:|---|---:|---:|---:|---|
| Public records | 2,000 seeds; 24,000 input checks | Predecessor final / base | 1 | 18,975,274,038 | 2.767242 | passed |
| Public records | 2,000 seeds; 24,000 input checks | List | 1 | 18,539,255,110 | 2.505522 | passed |

Savings: **436,018,928 Ir, 2.2978%**. The native sample falls 0.261720 CPU s,
but such a sample is not a variance-controlled speedup. Applying the Ir
fraction to the predecessor CPU gives **0.063586 CPU s** as a cautious suite
estimate for this sweep alone. It clears the sweep's approximately 2% bar.
The next measured removable codec cause is now `PublicQueued::measure`,
1,135,984,640 Ir, **6.13%** of the list arm. Its list holds fixed 12-byte
records (two u16 and one u64); a count multiplication can replace the walk.

The focused test
`tests::containers::a_grown_list_refuses_at_its_logical_cap_after_refilling`
checks zero, below-floor, and growth-clipping caps after clearing/refilling.
Changing `len == cap` to `len > cap` made it fail at cap zero: `Ok(())`
instead of `Err(0)`. Restoring equality passes. Existing container tests
continue to cover owned-item transfer, order, removal, room and clearing.

## Base profile leaves and callers

Inclusive rows overlap. Main function rows below omit harness envelopes and
inline fragments; self rows are the largest five leaf rows in each run 1.
The full call graphs were read with both inclusive and self annotation.

### R: inclusive work

| Share | Ir | Calls | Function |
|---:|---:|---:|---|
| 9.55% | 2,063,162,806 | 1,051 | `skein_llm::wire::json::Json::from_bytes` |
| 9.10% | 1,966,167,740 | 1,085 | `skein_json::parse::parse` |
| 6.13% | 1,325,094,788 | 4,615,923 | `skein_lib::list::List<T>::push` |
| 4.43% | 957,925,829 | 40,550 | `temper_service::system::Service::next_policy_deadline` |
| 4.27% | 923,283,992 | 926 | `smith_protocol_llm::prompt::object` |
| 3.49% | 755,155,071 | 79 | `skein_llm::client::measure` |
| 1.94% | 419,859,263 | 879,248 | `skein_json::string::after_plain` |
| 0.22% | 46,934,186 | 146,755 | `skein_lib::bytes::find_from` |

### R: self work

| Share | Ir | Function |
|---:|---:|---|
| 4.17% | 901,545,815 | `__memcpy_avx_unaligned_erms` |
| 2.07% | 447,733,784 | `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next` |
| 1.94% | 419,859,263 | `skein_json::string::after_plain` |
| 1.47% | 317,360,683 | `core::result::Result<T,E>::expect` |
| 1.40% | 303,313,154 | `core::option::Option<T>::expect` |
### P: inclusive work

| Share | Ir | Calls | Function |
|---:|---:|---:|---|
| 9.33% | 2,022,175,050 | 289,741 | `temper_service::system::Service::next_policy_deadline` |
| 7.20% | 1,561,295,999 | 796 | `skein_json::parse::parse` |
| 6.04% | 1,308,434,811 | 705 | `skein_llm::wire::json::Json::from_bytes` |
| 4.11% | 891,354,038 | 3,175,929 | `skein_lib::list::List<T>::push` |
| 2.72% | 590,479,900 | 552 | `smith_protocol_llm::prompt::object` |
| 2.07% | 448,570,657 | 92 | `skein_llm::client::measure` |
| 1.27% | 275,302,825 | 106,058 | `skein_lib::bytes::find_from` |
| 1.23% | 266,911,852 | 685,804 | `skein_json::string::after_plain` |

### P: self work

| Share | Ir | Function |
|---:|---:|---|
| 5.27% | 1,143,273,261 | `__memcpy_avx_unaligned_erms` |
| 2.21% | 479,173,974 | `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next` |
| 1.24% | 268,901,784 | `core::option::Option<T>::expect` |
| 1.23% | 266,911,852 | `skein_json::string::after_plain` |
| 1.16% | 250,963,100 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to` |
### G: inclusive work

| Share | Ir | Calls | Function |
|---:|---:|---:|---|
| 6.70% | 780,517,802 | 423 | `skein_json::parse::parse` |
| 6.03% | 702,488,880 | 362 | `skein_llm::wire::json::Json::from_bytes` |
| 5.82% | 678,478,597 | 28,770 | `temper_service::system::Service::next_policy_deadline` |
| 3.53% | 411,147,649 | 1,636,164 | `skein_lib::list::List<T>::push` |
| 2.86% | 332,786,411 | 54,201 | `skein_lib::bytes::find_from` |
| 2.73% | 317,920,801 | 313 | `smith_protocol_llm::prompt::object` |
| 2.01% | 234,494,091 | 36 | `skein_llm::client::measure` |
| 1.15% | 133,867,322 | 318,166 | `skein_json::string::after_plain` |

### G: self work

| Share | Ir | Function |
|---:|---:|---|
| 5.48% | 639,055,307 | `__memcpy_avx_unaligned_erms` |
| 2.04% | 237,740,762 | `<core::slice::iter::Iter<T> as core::iter::traits::iterator::Iterator>::next` |
| 1.33% | 155,027,964 | `<core::iter::adapters::enumerate::Enumerate<I> as core::iter::traits::iterator::Iterator>::next` |
| 1.22% | 141,698,135 | `core::option::Option<T>::expect` |
| 1.16% | 134,746,580 | `core::ptr::const_ptr::<impl *const T>::is_aligned_to` |

R iterator call edges trace chiefly to B-tree key search (6,328,738
enumerations, 361,470,174 Ir), fake-forge path unescaping (1,032,599,
59,248,777), client HTTP line pieces (932,488, 56,860,526) and fake-LLM
cues (904,616, 51,261,790). Direct slice iteration includes the world
referee (3,225,867, 67,380,467), fake-LLM prefix fields (2,840,706,
59,609,211), JSON builder (1,574,193, 32,219,543), SSE (955,265,
20,053,260) and CRC (955,062, 20,049,422). These are callers of the
leaf, not permission to drop the traversal. P and G show the same families
of callers. The large unaligned copy checks in every story trace back
to counted allocator headers; independent slice copies and B-tree moves
remain smaller. No header or core check is changed.

## List arm: real-loop observations

| Story | Draws | Arm | Run | Ir | Native CPU s | Profile wall s | Result |
|---|---:|---|---:|---:|---:|---:|---|
| R | one story | List | 1 | 21,421,624,303 | 8.457395 | 240.432923 | partial, forge observation cap; native passed |
| R | one story | List | 2 | 21,915,982,608 | 8.224616 | 220.367776 | partial, forge observation cap; native passed |
| P | one story | List | 1 | 22,265,899,436 | 6.896793 | 223.183744 | passed |
| P | one story | List | 2 | 30,938,082,144 | 6.472725 | 357.834256 | partial, forge observation cap; native passed |
| G | one story | List | 1 | 12,326,621,247 | 3.648652 | 151.345468 | passed |
| G | one story | List | 2 | 11,994,296,984 | 3.534194 | 134.888955 | passed |

R remains partial in both profiles. P run 2 also stopped at the same forge
observation cap; its 30.94b Ir is partial and is excluded from comparisons
of complete P stories. No observation ceiling was raised and no retry was
used. All six paired native executions passed. R has four native children
in each profile; P and G have none.

No real-loop suite gain is claimed. The R changes are below its repeated-run
spread. The one complete P total and both complete G totals increased; G
list-arm variation alone is 2.77%. Native samples increased too. These
figures do not establish a whole-story speedup or quantify a fuzzy-suite
saving. The deterministic public-wire reduction is the acceptance figure
for this shared cause. List push envelopes themselves fell from 891,354,038
to 831,919,206 Ir in P run 1, and 411,147,649 to 384,584,005 in G run 1,
but that is not a whole-story claim.

The timer-driven policy scan remains a screen, not a saving. Calls below
are scaled by profile wall / paired native wall; partial rows are omitted
because their profile and native executions cover different amounts of work.

| Story / arm / run | Raw calls | Slowdown | Scaled calls |
|---|---:|---:|---:|
| P / base / 1 | 289,741 | 24.385 | 11,881.8 |
| P / base / 2 | 286,038 | 25.420 | 11,252.3 |
| G / base / 1 | 28,770 | 38.019 | 756.7 |
| G / base / 2 | 28,786 | 36.723 | 783.9 |
| P / list / 1 | 291,661 | 25.085 | 11,627.1 |
| G / list / 1 | 29,336 | 37.065 | 791.5 |
| G / list / 2 | 28,603 | 37.084 | 771.3 |
| P / fixed / 2 | 290,617 | 24.084 | 12,066.7 |
| G / fixed / 1 | 28,680 | 35.006 | 819.3 |
| G / fixed / 2 | 28,441 | 44.185 | 643.7 |

## Fix 2: fixed-width named record measurement

The generator's existing primitive-width multiplication now uses fixed widths
computed once beside its declaration metadata. A fixed record uses its known
minimum width as its exact width, including the version prefix. Nested fixed
records and unit-only enums qualify. Optional fields, variable bytes/text/
secrets, lists and payload enums remain unclassified and keep value-dependent
measurement. No schema, stored format, constructor, decoder or limit changes.
This is `skein/docs/design/codec.md`, 4's encoded length with a cheaper
calculation.

All 28 changed generated Rust files were regenerated by existing drift tests
with `SKEIN_CODEGEN_WRITE=1`; no wire-byte golden changed. The tool also
reformatted the unchanged OAuth schema fingerprint constant; that unrelated
formatting was restored. The 24 regeneration checks and 38 generated golden
tables passed. The generator's 106 focused tests passed.

Clippy found that `AddressTask::measure` and `AddressProject::measure` in
`jig/crates/jig-client-view/src/generated/records_v1.rs` no longer read
`self`. Their existing module-scoped expectation in `src/lib.rs` now also
covers `clippy::unused_self`, preserving the uniform codec API. No generated
file was patched by hand for that repair.

| Sweep | Worlds or inputs drawn | Arm | Run | Ir | Native CPU s | Result |
|---|---:|---|---:|---:|---:|---|
| Public records | 2,000 seeds; 24,000 input checks | List / before fixed widths | 1 | 18,539,255,110 | 2.505522 | passed |
| Public records | 2,000 seeds; 24,000 input checks | Fixed widths | 1 | 16,702,000,918 | 2.330604 | passed |

Saved **1,837,254,192 Ir (9.9101%)**, and the native sample fell 0.174918 CPU s.
An Ir-scaled estimate is 2.505522 × .09910076 = **0.248299 CPU s** from this
sweep. This includes the eliminated loop overhead, which the initial estimate
based on `PublicQueued::measure` alone did not include.

The two fixes together save **2,273,273,120 Ir (11.9802%)** against the
predecessor's final arm. Its native sample changes from 2.767242 to 2.330604
CPU s, an observed 0.436638 s. An instruction-scaled suite estimate using that
same base sample is 2.767242 × .11980186 = **0.331521 CPU s**. These per-arm
CPU estimates use different native samples and must not be added as if they
were a common timing experiment.

`PublicQueue::measure` still runs **126,784** times, but falls from
1,799,191,744 Ir to **23,708,608**. Its **8,114,176** calls to
`PublicQueued::measure` become **zero**. The list append count remains
**20,894,402**; its exact 4,449,388,509 Ir envelope is identical in the list
and fixed-width arms. All generated wire golden bytes remain unchanged.

The focused test `tests::fixed_record_lists_measure_by_count` covers scalar,
empty and nested versioned records with a unit enum. Forgetting named widths
made its scalar row fail: a per-item `.measure()` loop replaced multiplication
by twelve. `tests::variable_record_lists_measure_each_value` covers bytes,
optional scalars and unequal enum payloads. Classifying an optional scalar as
fixed made its option row fail: multiplication by one replaced the required
per-value walk. Both rules were restored and both tests pass, at 7–13 ms in
the observed focused runs.

## Final real-loop arm

The same raised-patience configuration, exact story names and inputs are
used as in the base and list arms. Native P run 1 is a failed execution,
not a usable timing sample; its diagnosis is recorded below.

| Story | Draws | Arm | Run | Ir | Native CPU s (user + system, descendants included) | Profile wall s | Result |
|---|---:|---|---:|---:|---:|---:|---|
| R | one unchanged story | Fixed widths | 1 | 21,950,690,208 | 8.110137 | 217.942047 | partial, forge observation cap; native passed |
| R | one unchanged story | Fixed widths | 2 | 22,614,881,899 | 8.467347 | 235.750108 | partial, forge observation cap; native passed |
| P | one unchanged story | Fixed widths | 1 | 30,689,747,291 | 2995.070261 | 357.820138 | partial, forge observation cap; native deadline failure |
| P | one unchanged story | Fixed widths | 2 | 22,327,423,932 | 7.030350 | 241.716441 | passed; native passed |
| G | one unchanged story | Fixed widths | 1 | 12,010,689,479 | 3.364097 | 128.208042 | passed; native passed |
| G | one unchanged story | Fixed widths | 2 | 11,990,572,648 | 3.071253 | 137.705770 | passed; native passed |

R's final instruction spread is 3.03%, but both profiles are partial. P's
partial and complete profiles cannot form a spread for the same work.
G's final spread is 0.168%. No real-loop saving is demonstrated: these runs
are sensitive to polling and scheduling, and their totals do not establish
a gain beyond the arms' variability. Every R profile has four real process
spawns; every P/G profile has zero. No extra suite saving is credited.

## Left

The ranked table records every predecessor cause and real-loop candidate,
including its footprint and CPU arithmetic. None of the larger envelopes is
silently treated as removable work. In particular:

- `skein-lib/src/list.rs`: 2,656,476 actual growth operations retain
  1,880,272,816 Ir in `Vec::reserve_exact`, about **11.26%** of the final sweep.
  A full-footprint ceiling is 2.330604 × (1,880,272,816 / 16,702,000,918) =
  **0.262 CPU s**; an eager allocation still pays for the final allocation.
  Reserving a validated decoder count upfront needs an eager list entrance
  and a different allocation contract from `skein/docs/design/lib.md`, 5's
  grow-as-it-fills list. That design change is outside this task's small
  mechanism changes; no eager allocation or trusted unchecked append is added.
- `skein-lib/src/writer.rs`: the public-wire profile shows **27,898,884** puts,
  **423,377,449 Ir** in their memcpy calls and **1,199,652,012 self Ir**.
  `room` is already inlined. The writes, whole-put refusal and checks remain.
  Coalescing generated scalar writes would need an encoding mechanism that
  preserves field order and partial progress on writer overflow, not merely
  removal of a duplicate call.
- `skein-lib/src/reader.rs`: scalar reads split/check once and convert
  big-endian bytes. Their checks and field-specific codec errors remain.
- `smith-protocol-llm/src/prompt.rs`: a retained parsed schema needs a producer
  representation or a cache keyed by charter, namespace and grants, and the
  design must describe its ownership/invalidation. Independent fake-provider
  request admission remains validation of another input.
- `skein-json/src/string.rs`: the plain-run scanner is the shape chosen by
  `skein/docs/design/json.md`, 7, and its whole footprint is below the real-loop
  bar. Parser validation, allocator header layout, core copy checks, complete
  state hashes, cold restore, fresh client sessions and complete root secret
  checks remain required work. No build-profile override is introduced.
- The fake forge's observation cap prevents complete R profiles and two P
  profiles under instrumentation. These are explicitly partial. R natives
  passed; P's separate intermittent native failure is recorded below.
  The cap is not weakened to obtain a figure.

## Stopped at

After the accepted fixes, even deleting the entire remaining
`PublicLandings::measure` envelope would remove only **96,952,121 Ir, 0.58%**
of public-wire. Its fixed queue measurement is **0.14%**. The list's remaining
`usize_of` conversions cost **104,472,010 Ir, 0.63%**; even caching all of them
cannot clear approximately 2% (2.330604 × .006255 = **0.0146 CPU s**). Required
actual growth and scalar codec operations above are not such a gain. The
predecessors' local stopping points remain below 2% as recorded in the ranked
table. No additional real-loop-only cause demonstrates more than about 2 CPU s
of removable suite work under the existing contracts. The second JSON writing
measurement trial was not repeated.

## Raised-patience native P failure

The first fixed-arm native P execution failed after **3,000.661528 wall s**
and **2,995.070261 CPU s** (2,937.119520 user + 57.950741 system). It was
nonsettling and is not a performance sample. The judge's snapshot says the
caller remained at step 14 / call 10, polling project roots for
`Repair landing branch CI` for 20,817 attempts. Tasks 1 and 2 were held in
landing, task 3 held in reporting, four LLM queries unanswered, and the engine
remained ready at revision 12,096,739. No bound, observation cap or invariant
was weakened. The permitted 3,000 s patience was allowed to report its failure;
the unit was not stopped.

The same story was then run by exact name in its fuzzy profile with ordinary
20 s patience, once on pristine `baaa0068d3` and once on this branch. Both
passed (8.816 s and 10.049 s test wall respectively). These runs do not make
the failed raised-patience sample comparable with a successful run, and no
causal attribution or suite speedup is claimed from it. Its full log and the
two normal-patience isolation logs are retained beside the task's run report.

Reading identifies a possible harness ordering weakness, not a proven cause:
the real-loop adapter acknowledges a queued `Peer::release` before the fake
domain applies it, while `skein/docs/design/fake-llm.md`, 2.2, explicitly says
a release before anything holds that name changes nothing. A writing-phase
observation is not an observation that the fake has reached its hold. A fix
would need an observed hold or a causal release acknowledgement; the current
figures do not prove this was the failing interleaving. The domain's existing
early-release semantics remain unchanged.

The second fixed-arm native P sample, with the same 3,000 s setting and no
product change, passed: **7.030350 CPU s**, **10.036296 wall s**. The earlier
failure is therefore intermittent. Both raw native logs are retained in the
run report's `logs/` directory together with the pristine/current ordinary
isolation logs. It is an unresolved reliability observation, not evidence
of a measured real-loop speedup.

## Reproducing the measurements

Build each arm with the existing workspace profile. For R/P/G only, set
`temper/tests/real-loop/src/lib.rs`'s `PATIENCE` locally to 3,000 seconds,
then obtain `binary-path` and `cwd` with:

```sh
heavy cargo nextest list -p temper-real-loop-world --test suite --message-format json
heavy cargo nextest list -p temper-codecs-world --test suite --message-format json
```

Run from that suite's `cwd`, with its `CARGO_MANIFEST_DIR`, one test by exact
name, through `heavy bash -lc`. The R/P/G names are the exact names in
`temper/docs/plan/limits-fuzzy/notes/real-loop.md`. The public-wire name is
`fuzzy::public_wire::public_records_decoders_never_panic_and_reencode_the_same`.
Callgrind arguments are `--tool=callgrind --callgrind-out-file=<label>.%p.out`;
the binary arguments are `--exact <name> --nocapture`. Read the same file
with `callgrind_annotate --inclusive=yes` and `--inclusive=no`, both with
`--threshold=100 --auto=no`. There is one profiled process per row; the real
shell children execute outside Callgrind. Native runs invoke the same binary,
cwd and exact name without Valgrind. The untracked wrapper records deltas of
`resource.getrusage(resource.RUSAGE_CHILDREN)` user/system time after waiting
for the child, elapsed monotonic wall time, exit status and load average.

There are two native and two profiled attempts per R/P/G arm, and one per
public-wire arm. The sweep base is the predecessor's final figure, as the
task requests. Twenty native attempts produced nineteen passes and the
explicit P failure; twenty profiles produced twelve passes and eight
observation-cap partials. No zero-test selection is used as evidence.
Restore `PATIENCE` to 20 seconds before ordinary validation and every commit.
Temporary scripts, annotations, profiles and scratch measurements under
`target/limits-fuzzy/` are deleted before pushing.

## Final validation

`PATIENCE` is restored to 20 seconds. `cargo fmt` passes, and a later
`cargo fmt --check` after the scoped lint repair passes. From each of
`skein/`, `smith/`, `jig/` and `temper/`, `python3 -I -B scripts/old_forms.py`
passes; jig was checked again after the lint repair. No count or baseline
changed.

`heavy cargo clippy -p ... --all-targets -- -D warnings` passes for all
20 changed packages. Their focused tests and every direct dependent's
focused tests pass, selected by explicit package names in five commands,
not by `--workspace`:

| Group | Named packages | Passed focused tests | Nextest test wall s |
|---|---:|---:|---:|
| skein | 94 | 7,689 | 5.710 |
| smith | 41 | 3,648 | 6.343 |
| jig | 98 | 8,452 | 11.761 |
| temper | 54 | 10,306 | 37.595 |
| hearth | 31 | 2,144 | 2.486 |
| Total | 318 | 32,239 | — |

`heavy cargo nextest run -p ... --profile fuzzy --no-fail-fast` names the
same 20 changed packages plus the four required worlds: **56 passed** in
44.816 s test wall. This includes all 53 required-world fuzzy tests and
three skein-codec-tests fuzzy tests. P passes at ordinary patience in
this full run as well as its isolated run.

`heavy cargo check -p ... --tests` passes for all **316** direct dependent
packages from Cargo metadata, across every tree (92 skein, 41 smith,
98 jig, 54 temper, 31 hearth). `heavy cargo check -p skein-tls` also passes
standalone. No public signature changed.

The four worlds' final metadata matches the base for all **1,291** test
names, ignore flags and filter metadata. The actual focused and fuzzy pass
name sets also equal the corresponding base selections, not just their
counts:

| World | Focused passed | Fuzzy passed | Existing ignored |
|---|---:|---:|---:|
| temper-codecs-world | 41 | 5 | 0 |
| temper-engine-domain-world | 914 | 26 | 10 |
| temper-one-process-world | 51 | 4 | 2 |
| temper-real-loop-world | 219 | 18 | 1 |

The generated-code audit checks all 28 files: after removing measurement
methods and their empty helper impls, all other source is identical modulo
whitespace. The encoding/decoding, bounds, types and byte golden data are
unchanged. The only handwritten consumer adjustment is the scoped lint
expectation above. No slow tier, browser/process/live story, walk, render
check or product driver ran. The existing ignores remain unrun; the
instrumentation partials and native measurement failure remain reported
rather than being counted as passes.
