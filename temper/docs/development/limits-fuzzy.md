# Temper's fuzzy suite against its limit

2026-11-07. Closing measurements are on combined plan trunk
`182ef80c19a1`. **The suite does not hold 120 CPU s:** the median of three
warm serial runs is **125.579032 CPU s**, exceeding it by **5.579032 s**.
All three executions passed their 81 tests; two exceed the CPU limit.
Passing assertions do not make those over-budget measurements green.

CPU is user plus system with waited-for descendants. These closing figures
also include the warm Cargo/nextest driver. The workspace's existing test
profile and package overrides are unchanged; in particular the existing
`skein-lib` and `skein-codegen` optimizations are not a change of this plan.
No code, test, seed, timeout, build setting or budget changes in this account.

## Against 120 CPU seconds

| Full temper fuzzy run | Passed tests | CPU s | Margin below 120 CPU s |
|---|---:|---:|---:|
| 1 | 81 | 119.288299 | 0.711701 |
| 2 | 81 | 127.664356 | −7.664356 |
| 3, median by CPU | 81 | 125.579032 | −5.579032 |

The measured range is **119.288299–127.664356 CPU s**, a swing of
**8.376057 s**, or 7.02% of the smaller run. The median's corresponding
wall time is 137.031430 s; wall time is not the budget. Loads and every
user/system component are in C below. These were loaded-machine runs,
not three demonstrations on an idle machine. The request warns that CPU
figures can swing by a third; these three runs give their observed range,
not a bound on future variation.

The request's aim was about **15 CPU s out** of its **125–128 CPU s**
gate baseline, roughly **110–113 CPU s**. The closing median is still
12.579032–15.579032 s above that target, and lacks headroom even for
this run-to-run spread. The large instruction reductions below therefore
do not establish the requested whole-suite reduction.

The request's before figures are gate per-test CPU sums with eight tests
at once. These after figures run one test at a time and include the
driver, on a later combined trunk and under the recorded load. Their
differences cannot be called a controlled CPU speedup or regression.
A gate run's own per-test accounting decides whether that run holds; the
queue's trunk end check supplies the first such record. No such gate
pass is established by this account.

The user enables a tree's limit with a gate entry in
`tools/tiers/budgets.toml`. This plan adds none. These serial figures do
not support declaring temper's fuzzy limit achieved.

A next task could profile the present real-loop leaders at ordinary
patience, investigate its nonsettling observation, and assess retained
tool schemas with explicit ownership/invalidation. The old tool-schema
envelope alone has only a 2.43 CPU s upper ceiling, before retaining
necessary admission, so it is not a demonstrated cure for this 5.579032 s
median excess plus variation. A larger producer/cache design needs the
user's authorization for that scope. If profiling finds no sufficient
avoidable work, the measured remaining cost and desired headroom go to
the user for a budget decision. Changing a budget is the user's; moving
tests merely to escape it is not a repair
(`skein/docs/foundation/testing-strategy.md`, 8.1).

## Figures and unchanged draws

The plan's figures are preserved in `5d847a4fdb` under
`temper/docs/plan/limits-fuzzy/figures/`; these keys name each by its
file's name there:

- **P:** `public-wire`.
- **O:** `one-process-sweeps`.
- **S:** `state-hash`.
- **E:** `engine-sweeps`.
- **U:** `under-causes`.
- **C:** `close`, the combined-trunk runs.

Ir means executed instructions, with the test binary run unoptimized under
Callgrind. Before counts are copied from the sweep tasks. E's baseline
supersedes S's exploratory baseline for its paired comparison; S's derives
alone changed no executed path. P's after arm was reprofiled by U after the
shared list and fixed-width measurement changes. Those changes also reach
the engine and one-process worlds, so C refreshes their final Ir; it does
not remeasure or replace their original before arms. No sweep is reprofiled
merely because native CPU varied.

| Sweep, exact test name | Draws before = after | Before Ir | After Ir | Figures |
|---|---|---|---|---|
| `fuzzy::walking::fuzzy_walking_shard_0` | 8 seeds, 2 worlds each | 21,334,206,518 | 1,586,213,658 | E before; C after |
| `fuzzy::walking::fuzzy_walking_shard_1` | 8 seeds, 2 worlds each | 22,768,256,748 | 1,669,610,875 | E before; C after |
| `fuzzy::walking::fuzzy_walking_shard_2` | 8 seeds, 2 worlds each | 22,603,153,455 | 1,641,449,784 | E before; C after |
| `fuzzy::walking::fuzzy_walking_shard_3` | 8 seeds, 2 worlds each | 20,420,165,157 | 1,555,988,312 | E before; C after |
| `fuzzy::walking::fuzzy_walking_shard_4` | 8 seeds, 2 worlds each | 23,579,272,030 | 1,702,478,232 | E before; C after |
| `fuzzy::walking::fuzzy_walking_shard_5` | 8 seeds, 2 worlds each | 21,075,888,205 | 1,581,022,596 | E before; C after |
| `fuzzy::walking::fuzzy_walking_shard_6` | 8 seeds, 2 worlds each | 19,517,047,126 | 1,498,920,321 | E before; C after |
| `fuzzy::walking::fuzzy_walking_shard_7` | 8 seeds, 2 worlds each | 24,822,020,279 | 1,774,405,927 | E before; C after |
| `fuzzy::walking::fuzzy_terminal_commit_cut_preserves_complete_replay_and_funding` | 4 seeds, 2 worlds each | 15,408,212,864 | 1,161,519,574 | E before; C after |
| `fuzzy::escalation::fifteen_small_escalation_histories_cover_every_story_and_durable_cut` | 15 histories, 2 worlds each | 20,272,440,601 | 4,445,836,527 | E before; C after |
| `fuzzy::roles::twelve_small_role_histories_cover_each_holder_owner_order_and_atomic_cut` | 12 histories, 2 worlds each | 22,165,342,372 | 2,538,271,845 | E before; C after |
| `fuzzy::one_process::drawn_kills_keep_the_conformance_rules_and_answer_adoptions_once` | 4 crash seeds; 4 cold readbacks | 14,203,061,119 | 11,265,017,048 | O before; C after |
| `fuzzy::one_process::drawn_power_losses_keep_the_conformance_rules` | 4 crash seeds; 4 cold readbacks | 13,925,573,290 | 10,994,220,374 | O before; C after |
| `fuzzy::public_wire::public_records_decoders_never_panic_and_reencode_the_same` | 2,000 seeds; 24,000 drawn input checks | 31,243,278,730 | 16,702,000,918 | P before; U after |

The eight walking rows retain 64 seeds in all, with two independent lockstep
worlds per seed. Terminal, escalation and role histories likewise retain both
worlds, every cut, trace check, invariant and ending. Each crash sweep retains
all four original cuts and complete cold readbacks. Public-wire's six fixed
checks now run once rather than per seed; these are not random draws.
Its 24,000 drawn inputs and RNG order remain identical.

The following real-loop observations come from U, already on the final
shared-code arm. All draw the same one story per execution. They were run
twice per arm with 3,000 s local patience, restored to 20 s before ordinary
validation. No additional closing real-loop profiling or raised-patience
run was performed.

| Exact test | Draws before = after | U base Ir, runs 1 / 2 | U final Ir, runs 1 / 2 |
|---|---|---|---|
| `fuzzy::repair::a_change_failing_ci_is_repaired_and_lands` | 1 unchanged story | 21,614,835,263 / 21,985,351,066, both partial | 21,950,690,208 / 22,614,881,899, both partial |
| `fuzzy::case_6_4::a_red_tip_pauses_kits_queue_for_one_repair_that_lands_before_a4_while_the_directors_interim_timer_waits_without_adopting_a4` | 1 unchanged story | 21,677,906,409 / 21,366,608,783 | 30,689,747,291 partial / 22,327,423,932 complete |
| `github_land::a_chats_change_lands_on_connected_github` (focused) | 1 unchanged story | 11,651,837,055 / 11,653,418,240 | 12,010,689,479 / 11,990,572,648 |

Partial profiles are failed prefixes, not comparable complete-story totals.
G is a focused collateral observation and contributes nothing to fuzzy CPU.
O also records a focused collateral test,
`one_process::an_adopted_repository_gets_its_change_pushed_through_git`:
2,913,791,518 → 2,764,434,204 Ir in that task's paired arms, one unchanged
seed. Its native samples showed no gain. It is not a closing measurement
or a fuzzy-suite saving.

## The request's worlds

Before is the request's median gate per-test CPU sum at eight at once.
Every after row is the median by CPU of three warm `-p` runs at `-j 1`,
with Cargo/nextest and descendants included. **Every row is measured
differently across its before and after columns.** Cargo's workspace and
single-package selections can also build different feature unions: each
package's initial rebuild was recorded as warm-up and excluded, then three
warm samples were taken. No build setting was changed.

| World | Tests before = after | Before CPU s, gate | After CPU s, serial + driver | After range CPU s | Looked at by limits-worlds? |
|---|---:|---:|---:|---|---|
| `temper-real-loop-world` | 18 | 57 | 63.413918 | 61.052666–66.214588 | Yes |
| `temper-engine-domain-world` | 26 | 27.5 | 10.637649 | 10.303352–10.659738 | No |
| `temper-conformance-world` | 15 | 21 | 30.169610 | 27.876891–30.403677 | Yes |
| `temper-one-process-world` | 4 | 11 | 11.898472 | 11.800430–12.403466 | Partly, focused stories |
| `temper-codecs-world` | 5 | 5 | 4.184963 | 4.159268–4.258829 | No |
| Sum of these world medians | 68 | 121.5 | 120.304612 | — | — |

These five worlds account for 68 tests, not the full suite's 81. Their
independent medians include five drivers and use package selections; they
are not a reconstruction of the workspace run. The full-suite total is
measured directly above. In particular, neither its difference from this
table nor the increased real-loop/conformance figures establishes an
unprofiled code cause. C gives every warm run and the five excluded
build-inclusive warm-ups.

## Causes fixed

The source figures distinguish successive arms from overlapping inclusive
footprints. The following are whole-sweep instruction differences unless an
edge is explicitly named. Savings from different sweeps or native samples
are not a controlled suite timing experiment.

| Cause and location | What the measured fix saved | Figure |
|---|---|---|
| Hash derivations in skein/jig state types | No executed-path saving by themselves. They enable the engine comparison; the original comparison occupied 96.09% of walking-0 and 90.58% of escalation. | S |
| Complete state rendered and compared every iteration, engine replay loops | Initial word-hash arm saves 19,538,647,202 Ir walking-0; 15,153,393,176 escalation; 18,289,942,552 roles, after paying for complete hashing and final text comparison. | E |
| Checked word/conversion overhead in the transient hasher, engine `src/replay.rs` | Another 170,101,101 / 599,567,498 / 242,707,555 Ir in walking-0 / escalation / roles (9.47 / 11.71 / 6.26% of their initial word arms). | E |
| Rescanning acknowledged role history, engine `src/roles.rs` | 1,036,069,181 Ir, 28.52% of roles' cheaper-word arm, by consuming each new trace entry once. | E |
| Constructing a placeholder store before teardown, engine `src/oracle.rs` | 40,861,268 / 76,680,983 / 61,272,552 Ir in walking-0 / escalation / roles. The real store is transferred to the oracle. | E |
| Repeated seed-independent bound and refusal inputs, codecs `fuzzy/public_wire.rs` | 12,268,004,692 Ir, 39.27% of public records. All 24,000 drawn checks remain; the six fixed checks run once. | P |
| Reconstructing the adoption codec per service start, one-process `src/fixtures.rs` | 806,924,595 / 808,593,984 Ir in kill / power, constructing it once rather than sixteen times. | O |
| Rebuilding saved keys and rescanning unchanged rows, one-process `src/referee.rs` | 701,700,255 / 673,368,002 Ir. History and durable positions retain the initial empty observation and every later change. | O |
| Rebuilding typed adoption defaults, one-process `src/fixtures.rs` | 650,184,529 / 652,248,196 Ir. Each independent start still receives every default field and capacity. | O |
| Recounting adopted rows per action, one-process `src/referee.rs` | 518,050,668 / 526,757,985 Ir by refreshing on durable commit. | O |
| Re-encoding identical settings per setup, one-process `src/stories.rs` | 235,818,631 / 236,220,205 Ir, including the final initial-cursor repair; identical bytes are still written to each fresh disk. | O |
| Repeated append-capacity bookkeeping, `skein-lib/src/list.rs` | 436,018,928 Ir, 2.2978% of P's final public-record arm. Actual growth, logical-cap refusal and element order stay. | U |
| Measuring every fixed-width named record, `skein-codegen/src/emit_codec.rs` | 1,837,254,192 Ir, 9.9101% of U's list arm, by multiplying the validated count by the fixed width. Variable records keep per-value measurement; wire bytes and errors stay. | U |

E's eleven sweeps together fall from 233,966,005,355 to
21,135,491,538 Ir (90.97%) in its own paired arms; its native samples fall
25.913258 to 6.067245 CPU s, an observed 19.846013 s. O's two sweeps save
5,809,867,050 Ir in total; native samples fall 10.818533 to 9.231111 CPU s,
an observed 1.587422 s. These are the tasks' figures, before the shared
changes, not substitutes for the closing suite measurement.

U's two fixes together save 2,273,273,120 Ir (11.9802%) from P's final arm.
Its native sample changes 2.767242 to 2.330604 CPU s. Instruction-scaling
that same base sample gives 0.331521 CPU s; the independently based per-arm
estimates must not be added. No real-loop saving exceeded its measured
variation, so none is credited there. No test, crash, invariant or ending
was removed or moved.

## Causes found and left

Every figures file's `Left` and `Stopped at`, and the readable prerequisite
reports, is reconciled below. S's comparison assignment is resolved by E;
P's list/measurement assignments are resolved in part by U. Shares identify
their measured arm, remain inclusive and overlapping unless marked self,
and are footprints rather than promised gains. R/P/G below are U's real-loop
story labels, not the figure P.

| Cause and location | Measured share or size | Why it stands |
|---|---|---|
| Retained public-record decoding, `temper-client-protocol-wire/src/generated/public_v1.rs` | P final: decode/measure/re-encode envelope 96.29%; `PublicLandings::decode` 63.80%, queue decode 44.29%, queued-scalar decode 18.35%; encode 17.75%. | The drawn mutated records are the checks. Removing this work removes coverage. U pays only the redundant append bookkeeping and fixed-width measurement. |
| Actual list growth, `skein/crates/skein-lib/src/list.rs` | U final: 2,656,476 growths, 1,880,272,816 Ir / 11.26% in `Vec::reserve_exact`; full-footprint ceiling about 0.262 CPU s. | Final allocation is still necessary. Eager decoder allocation needs a different entrance/allocation contract from `skein/docs/design/lib.md`, 5's grow-as-filled list. |
| Scalar writing, `skein/crates/skein-lib/src/writer.rs` | P final footprint 8.55%; U has 27,898,884 puts, 423,377,449 Ir in memcpy and 1,199,652,012 self Ir. | Room is already inlined. Coalescing writes needs a mechanism preserving field order, whole-put refusal and partial progress on overflow. Required bytes and checks cannot be skipped. |
| Scalar reading, `skein/crates/skein-lib/src/reader.rs` | P final: 797,987,918 Ir / 4.21%, overlapping decode. | Each field must split/check input and convert big-endian bytes, preserving field-specific shortage/bound errors. No dispensable read was identified. |
| Remaining public measurement and conversions | U final: whole `PublicLandings::measure` 0.58%, fixed queue measurement 0.14%, list `usize_of` conversions 0.63% (at most 0.0146 CPU s). | Even their full footprints are below about 2%; caching conversions cannot eliminate required operations. |
| Public scratch setup and diagnostics, codecs world | P final: two scratch refills 1.45%, initial materialization 0.75%, arbitrary draw 0.50%, unwind envelope about 0.012%. | Refills are below the stop rule even if eliminated completely; draws and panic/seed/input diagnostics remain. |
| Fake-person stepping and fresh native sessions, one-process world | O final kill / power: person 10.32 / 4.65%; native startup 5.49 / 8.44%; native iteration 10.24 / 10.46%. | `skein/testing/skein-fake-person/src/lib.rs` retains all steps/deadlines; `temper-client-shell-terminal/src/client.rs` retains independent sessions and real client iterations. No whole operation was dispensable. |
| Service stepping and complete secret checks, `temper/crates/temper-service/src/system.rs` | O final kill / power: iteration 22.09 / 22.91%; root formatting 3.22 / 3.29%. | Every deployment/restart still runs the real engine/worker and all twelve complete root secret-leak checks. Footprint ceilings of about 2.08 and 0.30 CPU s are not removable savings. |
| Smaller one-process setup/observations | O final kill / power: native-link limits 1.60 / 1.70%; old commit observations at most 1.4%, root text outside service about 1%, full snapshots at most 0.02%. | Fixed-bound caching still copies its returned view; old-prefix cursors still check new entries. Each upper bound is below about 2%. |
| One-process setup tails and armed prefixes | O codec kill arm: full tails 268,338,682 Ir / 2.00%; armed prefixes 3,215,213,736 Ir / 24.00%. | Required tail root checks leave under 1% removable before early-drain costs. Story and fault phases have different process/root graphs and startup sources, so sharing a live prefix would change the world. |
| Store and cold restore, `jig/crates/jig-protocol-store/src/{lib,restore}.rs` | E final walking-0 / escalation / roles: store 12.03 / 10.33 / 15.55%; restore 7.52 / 6.70 / 11.93%, nested in store. | Real codec writes/reads, cold restore and independent roots are required; no complete restore or store operation can be removed. |
| Product steps and complete frozen-state traversal, `temper-engine-domain/src/domain.rs` | E final walking-0 / escalation / roles: step 14.02 / 11.92 / 16.06%; `Domain::hash` 30.54 / 28.23 / 8.71%. | Both worlds must step and compare the complete mutable state. S's old iterate/close footprints (walking 1.80 / 1.28%; escalation 4.67 / 3.17%) were not savings either. No test can avoid these full operations under the comparison contract. |
| Old trace-prefix equality and final complete text, engine world | E final old-trace equality 0.42–1.17%; final text 2.50–4.54% of the displayed sweeps. | New entries still need comparing. Final text is required once per seed. A streaming alternative increased Ir by 0.77 / 0.80 / 1.47% in walking-0 / escalation / roles and was discarded. |
| Retained tool-schema admission, `smith/crates/smith-protocol-llm/src/prompt.rs` | U base R/P/G: `object` 4.27 / 2.72 / 2.73%; R envelope ceiling 57 × .0427 = 2.43 CPU s. | The removable part is smaller than the envelope. Retention needs a producer representation or cache keyed by charter, namespace and grants, with ownership/invalidation in the design. |
| Independent JSON validation, `skein/crates/skein-json/src/parse.rs` and callers | U base R 9.10 / 8.86%, P 7.20%, G 6.70%; fake-provider admission alone R 5.24%, P/G 3.28%. | Separate peer inputs require validation. No test can avoid those checks, and no removable independent validation was found; the 5.19 CPU s full-parser ceiling is not a gain. |
| Whole-prompt measurement and small JSON callers, `skein-llm/src/{client,translate}.rs`, `smith-protocol-llm` | U base R/P/G measurement 3.49 / 2.07 / 2.01%; all R measurement would be 1.99 CPU s. Replay JSON at most 0.0091% / 0.0052 CPU s; fixture/default admission 0.71% + 0.45% / 0.66 CPU s. | Guard whole/prefix checks, framing and connection admission serve different purposes. Observation is disabled here; its function itself costs only 980 / 637 / 343 Ir. No >about-2-CPU-s removable real-loop cause is established; all twelve configuration admissions remain. The discarded writing-pass trial was not repeated. |
| Byte searches and plain-string scan, `skein-lib/src/bytes.rs`, `skein-json/src/string.rs` | U base R/P/G: search 0.22 / 1.27 / 2.86%; plain scan self 1.94 / 1.23 / 1.15%. Fuzzy ceilings 0.72 and 1.11 CPU s. | Necessary searches remain; even using focused G gives only a 1.63 s search ceiling. A word scanner also changes `skein/docs/design/json.md`, 7's selected shape. |
| Timer-inflated policy deadline scan, `temper-service/src/system.rs` | U base P: 9.33% raw, 289,741 calls; slowdown 24.385 gives about 11,882 native-equivalent calls and 0.383% / 0.218 CPU s ceiling. | Scaled before ranking. Readiness/deadline checks stay; the raw instrumented footprint is not a saving. |
| Counted allocation headers and core copy checks, `skein-heap`, Rust core | U base R: 3,015,640 unaligned reads and 3,015,669 writes; check edges 1,055,479,060 Ir / 4.88%. | Every counted allocation/deallocation needs its header layout and checks. No test can avoid that cost under the meter contract. No dispensable independent copy was found; no build override or check removal is proposed. |
| Iterator leaves across required callers | U base R self slice iteration 2.07%, enumeration 1.33%; P 2.21 / 1.11%. Whole-R footprint ceiling 1.94 CPU s. | Callers are B-tree lookup, referee, fake-provider cache/cues, JSON construction, HTTP framing, SSE and CRC; this is not one avoidable loop. |
| Instrumentation caps and intermittent real-loop nonsettling | U: eight observation-cap partials among twenty profiles; one failed raised-patience native P among twenty natives. No defensible saving share. | `skein/crates/skein-fake-peers/src/forge/observe.rs` keeps its cap. `temper/tests/real-loop/src/world.rs` has an unproven release-ordering suspicion; ordinary base/current isolates passed. Reliability needs a separate investigation, not weakened checks. |

U's R profiles are partial in every arm; P also has two partial profiles.
Final complete G profiles differ by 0.168% Ir, but cross-arm differences and
native samples establish no real-loop speedup. R spawns four native shell
children, outside Callgrind; native CPU includes them. P and G spawn none.
The partial totals cannot compare complete executions.

The raised-patience P failure ran 3,000.661528 wall s / 2,995.070261 CPU s
and was nonsettling, not a timing sample. At ordinary 20 s patience the
pristine base and changed branch each passed by exact name; a later identical
raised-patience native attempt passed too. The suspected acknowledgement of a
queued peer release before the fake observes its hold remains unproved.
This closing task neither repairs nor attributes it.

O's existing ignored stalled-startup story and manual memory measurement stay
ignored. The focused-suite overage and real-loop scripted-story placement
remain in `temper/docs/design/domain/testing.status.md`; this fuzzy-cost
account changes neither. No slow tier, walk or product driver ran.
