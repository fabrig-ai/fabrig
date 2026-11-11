# Limits of the deployment worlds: the repair's account

2026-11-07. The closing measurements use `bcddfe1ff1`, the plan's combined
trunk. Test builds and dependencies are unoptimized. CPU means user plus
system seconds, including waited-for descendants. The record of commands,
loads, individual runs, instruction counts and unsuccessful profiles is
the figure `close`. The plan's figures are preserved in `861a69c920` under
`temper/docs/plan/limits-worlds/figures/`; this account names each by its
file's name there.

| Suite | World | Before tests | Before CPU s | After tests | After median serial CPU s † |
|---|---|---:|---:|---:|---:|
| fuzzy | `temper-real-loop-world` | 22 | 85 | 18 | 60.139001 |
| fuzzy | `temper-conformance-world` | 15 | 44 | 15 | 30.586688 |
| fuzzy | `temper-engine-domain-world` | 26 | 26 | 26 | 28.201091 |
| fuzzy | `temper-one-process-world` | 4 | 10 | 4 | 11.275533 |
| focused | `temper-one-process-world` | 49 | 29 | 51 | 27.899207 |
| focused | `temper-real-loop-world` | 241 | 30 | 219 | 31.769481 |
| focused | `temper-system-world` | 42 | 8 | 48 | 10.806115 |
| focused | `temper-playground-world` | 6 | 5 | 6 | 5.841612 |
| focused | `temper-conformance-world` | 29 | 3.5 | 36 | 5.576544 |

† Before is the request's median of three gate runs near `50053b79a2`, with
load unknown. Every after row is measured differently: three warm serial
nextest commands, that world's own profile and `-j 1`, including the small
cargo/nextest cost. The builder has 16 CPUs, AMD EPYC-Genoa Processor; the
figures record its load at both ends of every run. An independently idle
machine was not established. These columns describe the request and final trunk; their differences do not
isolate an individual repair or remove machine/load effects. Changed test
counts include intervening work, new cause regressions and four removed
real-loop stories; no story moved into focused to make room.

## The suites against their limits

The four measured fuzzy-world medians sum to **130.202313 CPU s**
against temper's **120 s** budget: **10.202313 s over**.
The five measured focused-world medians sum to **81.892959 CPU s**
against **30 s**: **51.892959 s over**.
Both measured partial sums stand over their suite limits; these are not
complete-suite totals or exact per-test gate overages.

The real-loop focused median is **31.769481 CPU s**, **16.769481 s over
its standing 15 s ceiling**. One-process focused alone is 27.899207 s,
already over the 8 s guide for all worlds below the domain together.
Real loop plus one-process fuzzy alone totals 71.414534 s against
that group’s 80 s guide; other below-domain worlds are not included.

The budget comparisons remain red under the strategy's repair rule.

No readable whole-temper gate or end-check result was supplied in this
slot's plan records. The sums above cover only the request's worlds, not
the rest of temper, and add medians taken separately. They include driver
overhead and are not a per-test gate total. They establish no whole-suite
pass. The request's approximately 174 fuzzy and 336 focused CPU seconds
are historical whole-suite figures, not measurements of today's trunk.
The queue's whole-suite measurement remains necessary.

The next step is the strategy's repair rule
(`skein/docs/foundation/testing-strategy.md`, 8.1): decide where the
scripted survivors belong by what they prove; then profile and repair the
remaining necessary work. A further cause pass could profile the engine-domain
and one-process fuzzy sweeps, at 28.201091 and 11.275533 median CPU s,
retaining every cut and referee observation; that follow-up work needs the
user's authorization. The closest lower-story candidates are 5.5 and
6.5 below. Shared schema ownership and prompt-admission measurements need
explicit representation or protocol work, with their existing validation
and refusal ordering preserved. The user must decide the surviving
stories' placement and revisit the real-loop ceiling with these numbers.
Any change to a suite budget, ceiling or hot-leaf build override also
requires the user's decision; none is made here.

The browser, process and live tiers were not run. Their CPU budgets are
not read by this plan's runs; after-landing supplies those results.

## Every profiled story, first to last

Ir is executed instructions. Labels below also identify the closing
figures' exact-name table. Paired real-loop figures are separate runs,
never their sum. A partial profile describes a failed prefix, so its
total cannot establish a successful-story saving.

| Story | First Ir | Last Ir on combined trunk | First figure |
|---|---:|---:|---|
| `fuzzy::stories::a_ci_repair_is_crashed_at_every_live_commit` (C) | 41,386,230,122 | 14,810,315,805 | `figures-before` |
| `fuzzy::shared::a_changed_judge_is_crashed_at_every_live_commit` (J) | 37,550,774,224 | 12,794,367,523 | `cuts-fork` |
| `stories::a_forked_cut_ends_as_a_fresh_cut_does` (Q) | 2,105,510,429 parent only; whole total unknown | 1,060,592,335 | `cuts-fork` |
| `stories::failed_ci_is_repaired_and_the_new_head_is_reviewed` (B) | 488,398,612 | 291,672,676 | `cuts-fork` |
| `public::every_inaccessible_identifier_is_one_unknown_byte_for_byte` (U) | 18,514,602,590 | 10,996,373,926 | `figures-before` |
| `public::every_owner_version_and_kind_answers_as_declared` (O) | 6,377,149,314 | 3,792,362,463 | `public-causes` |
| `m15::whole_run_and_renewal_scan_every_fake_issued_token_byte` (M) | 17,791,335,749 | 11,756,350,787 | `public-causes` |
| `fuzzy::repair::a_change_failing_ci_is_repaired_and_lands` (R) | 25,889,333,321 / 28,824,912,164, both partial | 23,471,805,382 (partial) / 22,906,059,880 (partial) | `figures-before` |
| `fuzzy::case_6_4::a_red_tip_pauses_kits_queue_for_one_repair_that_lands_before_a4_while_the_directors_interim_timer_waits_without_adopting_a4` (P) | 32,077,040,893 / 39,395,136,937, both partial | 21,430,390,341 / 21,249,528,979 | `figures-before` |
| `github_land::a_chats_change_lands_on_connected_github` (G) | 14,649,575,831 / 16,436,998,947 | 11,561,761,382 / 11,595,415,230 | `figures-before` |

The five requested baseline stories come from
`figures-before`.
The additional judge sweep and comparison come from
`cuts-fork`; owner and M15
from `public-causes`.
The uncut repair (B, RepairCi at seed 71) was also profiled as cuts-fork's
prefix estimate. Q's first figure is the unreset comparison profile's
parent only; its whole-program total is unknown and cannot be compared
with the closing sum of parent and children.
All last figures are from `close`. The forked sweep figures sum parent
and every child's PROGRAM TOTALS, resetting at child entry; their small
inherited libtest overhead is retained. The first sweep figures reran a
fresh prefix per cut. Both methods keep every live cut, but count different
work. The focused comparison now checks complete fresh outcomes at the
first, middle and last cuts while every forked child still runs.

R has two partial profiles (2.47% count spread). P and G each have two
passing profiles (0.85% and 0.29% spread).
R’s completed-story Ir remains unknown. P’s first completed profile in real-loop-causes is 25,606,221,778 Ir;
the closing completed pair is shown above. That is a combined-trunk
observation, not an isolated saving attributed to a single repair.
A failed prefix has no completed-story total or saving. The closing figures
record the failure, native passing reference, timer correction and external
spawn count for every sample. Native CPU includes children; parent Ir does not.
The final three real-loop fuzzy runs pass all eighteen survivors with shipped
patience. Slow-tier results and a complete-tree suite total remain unknown.

## What went, and what holds it

The plan's Belonging section, preserved in `861a69c920` beside the figures,
records the assertion reading. `stories-go` records nineteen distinct holders, each selected alone and passing before
removal, and the resulting 22-to-18 fuzzy count. The two lower holders
were added first. All names below are tests; `fuzzy::case_*` abbreviations
refer to the full names in the closing figures.

**`fuzzy::landing::a_chat_lands_its_change_on_the_forge`:**

- The changed-digest and replied watch is held by
  `tests::jig::wait::both_requirements_must_hold_on_the_same_delivery`
  (`temper-client-tools`, focused).
- The ended chat and “The change landed.” result are held by
  `github_land::a_chats_change_lands_on_connected_github` (real loop, focused).
- The Forgejo merge, green head and edit on main are held by
  `fuzzy::landing::a_chat_keeps_its_live_delegate_until_the_change_lands`.

**`fuzzy::case_5_6`:**

- Tracking and priority: `root_routes::an_agent_changes_a_live_delegates_controls_and_replays_the_answer`
  (engine domain, focused), and `fuzzy::case_4_1_4_3` end to end.
- The timer through a fresh run, correction and cancel:
  `root_routes::timer_cancellation::an_armed_timer_survives_a_fresh_director_run_and_its_helpers_cancellation`
  and `root_routes::a_cancel_closes_three_levels_with_runs_live_deepest_first`
  (engine domain, focused). Real paths remain in `fuzzy::case_6_4`,
  `fuzzy::case_4_1_4_3` and `fuzzy::case_5_1`.
- Its held helper has no change to land. Cancellation of a produced change
  leaving its file off main is held by `fuzzy::case_5_4`.
- Notes: `root_routes::a_note_written_corrected_by_a_person_and_recalled_is_durable`
  (engine domain, focused), and `fuzzy::case_4_6`'s search and recall.
  Notes do not depend on a preceding cancel.

**`fuzzy::case_6_1`:**

- Hierarchy, deadlines, questions, answers and complete journals:
  `fuzzy::case_6_5`.
- Batch-local references:
  `root_routes::added_dependencies::one_batch_of_four_resolves_its_local_references_and_gates_only_the_fourth`
  (new engine-domain focused holder).
- Three running, one gated, unrelated work proceeding:
  `root_routes::added_dependencies::both_prerequisite_orders_and_completion_orders_gate_only_the_fourth_delegate`.
- The assembled paths: `fuzzy::case_6_6`'s batch dependency,
  `fuzzy::landing::a_chat_keeps_its_live_delegate_until_the_change_lands`'s
  start after Met, and `fuzzy::case_4_1_4_3`'s worker placement.

**`fuzzy::case_6_2`:**

- Questions and answers through two owners, and delegation after the answer:
  `fuzzy::case_6_5`.
- Live dependency revisions and keeping:
  `root_routes::added_dependencies::a_live_delegate_is_kept_until_both_added_dependencies_are_met`.
- Last Met wakes the delegate:
  `stories::dependencies::a_kept_delegate_is_woken_by_its_last_dependencys_met`
  (`jig-core-world`, focused).
- Journal prefix and committed turns at the next attempt:
  `root_routes::added_dependencies::a_kept_delegate_resumes_with_its_committed_turns_and_its_journal_whole`
  (new engine-domain focused holder).
- Green landing followed by work: the kept landing story above. Resumed
  real-loop history: `fuzzy::case_4_6`. Complete journals: `fuzzy::case_6_6`.

The process tier was not relied on: its only remaining reference-case
story, 4.6, is ignored pending jig's draining-worker placement repair.

## What stays, for the user's placement decision

These are all eighteen survivors of the belonging table. The rationale is
the plan's reading of what only each holds, not an additional proof run.
CPU is one native exact-name run, descendants included, on closing code
with shipped patience, except the two already-profiled fuzzy stories whose
same-build native figures use profiling patience as explicitly recorded.
Those single-story figures are not the world's three-run median. Every
CPU figure is compared with the 2 s real-loop story guide; an over-guide
story is not an exception. Ir is unknown when not profiled.

| Story under `fuzzy::` | What only it holds | Native CPU s | Ir if profiled | Lower-story estimate in plan |
|---|---|---:|---:|---|
| `case_4_1_4_3::the_opener_and_the_two_hour_check_run_on_three_workers` | Three-worker placement including late s2, stall and timer cancel; subtree authority and Markdown-only change | 4.254333 | unknown | 265–350 |
| `case_4_4::the_pace_question_reaches_the_person_and_its_answer_is_kept` | Model amendment on the next wire query; delegate uses the person's first-ranked account | 2.247000 | unknown | 95–145 |
| `case_4_5_limited::a_limited_account_holds_its_work_until_its_reset` | Provider wire limit/reset, waiting and resuming without a person's act | 0.772398 | unknown | 80–130 |
| `case_4_5_refused::a_refused_key_holds_its_work_until_it_is_replaced` | Provider wire refusal, replace-key and resumed work on the new credential | 0.862726 | unknown | 80–120 |
| `case_4_6::a_restarted_workstation_keeps_the_directors_question_and_answers_it_once` | Worker stopped with an unanswered committed question; retry elsewhere, one answer and two worker lives | 1.596838 | unknown | 80–120 |
| `case_5_1::a_held_director_keeps_the_owners_question_and_its_fresh_run_reads_the_new_rules` | Changed guide on main reaches fresh system text through checkout preparation; rendered fresh brief tail | 1.983960 | unknown | 120–180 |
| `case_5_2::a_stopped_owner_holds_its_delegates_and_its_release_resumes_them_and_its_news_waits_for_its_release_without_a_park_or_a_retired_charter` | Running reviewed change stopped, released and landed; B's news once after release | 3.818982 | unknown | 110–180 |
| `case_5_4::a_new_direction_cancels_x3_under_its_closed_pull_and_bs_landing_news_waits_for_its_release_with_an_overlap_wake_and_a_closing_comment` | Produced-change cancellation against the pushed forge head; held owner's overlap news after release | 5.310823 | unknown | 80–140 |
| `case_5_5::an_owners_answered_plan_starts_a_sub_owner_under_a_deadline_that_cannot_delegate_without_the_deadline_firing_or_a_fresh_run_only_at_the_milestones_end` | Hosted depth refusal creates no task; fresh-run plan recovery by note recall; closest overlap | 1.817697 | unknown | 90–150 |
| `case_6_4::a_red_tip_pauses_kits_queue_for_one_repair_that_lands_before_a4_while_the_directors_interim_timer_waits_without_adopting_a4` | Deployment-requested red-tip repair, first in forge merge order and queue pause | 5.430119 | 21,430,390,341 / 21,249,528,979 | 60–105 |
| `case_6_5::a_budget_question_climbs_two_owners_to_the_persons_inbox_and_a_raised_investigation_hands_its_report_to_an_advisor_without_the_director_asking_the_person` | Whole inbox before/after one answer; effort amendment reaches next run; advisor's model/report/one turn; no clearly unique assembled path | 2.838387 | unknown | 95–145 |
| `case_6_6::a_kit_change_lands_before_the_apps_early_repin_without_relay_without_the_a3_a4_epilogue` | Atomic batch across two repositories; early producer and pin bytes against the real merge | 4.328254 | unknown | 100–150 |
| `case_6_7::a_sub_owner_past_its_deadline_is_held_at_its_next_run_and_escalated_through_its_owner_to_the_director_who_splits_it_by_amendment_and_release_without_a_grace_for_its_delegates` | Real-clock deadline wakes parked work, hold escalates, amendment/release splits it | 2.448953 | unknown | 85–130 |
| `case_6_8::m1s_end_message_starts_m2_beside_a_tester_without_forge_effects_whose_test_lands_only_through_a1s_change` | Refused tester forge call has no effect; M2 starts beside A1; tester report lands through A1's change | 3.572302 | unknown | 175–270 |
| `green_light::a_green_light_in_the_inbox_lets_the_change_land` | B0 recorded-model twin; goal(advisor: true) and delivered stock review brief | 5.977125 | unknown | unique transcript history |
| `landing::a_chat_keeps_its_live_delegate_until_the_change_lands` | Landing dependency added to report delegate; its first run follows real Forgejo landing | 2.154811 | unknown | 60–100 |
| `program_landing::a_chat_on_claude_code_lands_its_change_on_the_forge` | Claude Code backend across a whole change, activations and Read/Edit | 2.165572 | unknown | unique backend path |
| `repair::a_change_failing_ci_is_repaired_and_lands` | B1's six recorded-model transcripts, repair wording and bounded result; independent of scripts | 7.350897 | 23,471,805,382 (partial) / 22,906,059,880 (partial) | unique transcript history |

Every row remains in the fuzzy suite as the request required. These are
named scenarios, scripted or replaying recorded transcripts, with no sweep
of seeds; being past a guide is not a reason to put them there (`skein/docs/foundation/testing-strategy.md`,
8 and 8.1). The recorded-transcript twins also remain for their independent
model histories. Their placement is the user's next decision, not a budget
repair performed by this account. No process story currently replaces the
reference case; 4.6's existing process story is ignored. The estimates are
lines of a prospective lower story, not permission to move an assembled
path into a tier that cannot show it.

## Causes fixed

The savings below are each task's controlled arms, not differences between
the request and closing world CPU totals. Sequential savings use their
preceding arm and must not be added as percentages. Inclusive footprints
overlap. “No isolated saving” is a measurement limit, not a zero cost.

| Cause and source figure | What changed and what it saved |
|---|---|
| Fresh prefix at every crash, `cuts-fork` | Fork at each live commit; CI 41,387,037,208 → 25,170,467,106 Ir (39.18%); judge 37,550,774,224 → 21,871,236,484 (41.76%). All 78 / 77 cuts remain. |
| Whole-store projection, `conformance-causes` | Keyed row contributions: CI saves 980,994,301 Ir (3.90%); judge 1,651,083,158 (7.55%). Complete referee snapshots remain. |
| Repeated full policy equality, same | Shared immutable atomic snapshots with detached edits: another 3,138,736,310 / 2,204,383,450 Ir (12.98% / 10.90%). |
| Borrowed-map rebuilds and joins, same | Direct durable-key lookup and keyed cross-row facts: another 1,200,578,818 / 1,473,143,333 Ir (5.70% / 8.18%). |
| Effect-evidence scans, same | Typed ranges and exact lookups: another 412,112,143 / 952,988,136 Ir (2.08% / 5.76%); full-walk comparison remains. |
| Fixture rebuilt for public watch, same | Reuse fixed watch parameters, all 954 reads retained. CI factory calls 1,128 → 252; its inclusive Ir 1,190,351,569 → 268,678,903. Whole arm saves 1,115,429,189 Ir (5.74%), including compatibility work. |
| Empty per-row payloads and temporary spend map, same | Compact row facts and ordered expense totals. Combined final arm saves 628,227,774 / 466,432,531 Ir (3.43% / 2.99%); compaction's net saving is not isolated. Adapter work excluding projection/resolution falls 442,664,283 / 298,873,669 Ir. Heap ceilings and observations remain. |
| Repeated fresh comparison outcomes, same | Complete first/middle/last references with every forked cut retained; native 1.082 → 0.491 CPU s, within the 1 s guide. |
| Allocation-accounting TLS lookup, `engine-causes` | Group destructor-free counters, one lookup per allocation/free. Unforked CI saves 3,564,161,827 Ir (8.61%), same allocation/free counts; 3,563,853,699 is bookkeeping. No whole-GitHub saving beyond its spread is established. |
| Durable-row landing scan, `public-causes` | Key-range lookup saves 3,804,986,652 Ir (20.55% of original identifiers); readiness short-circuit saves another 1,009,651,739 (6.86%); applied-commit cache another 401,584,371 (2.93%). Whole identifiers 28.17% cheaper; owners 28.42%. |
| Repeated issued-token searches, same | One cached overlapping matcher, original needle priority and old-artifact checks. Search path saves 4,778,588,655 Ir; M15 whole story saves 5,129,459,177 (28.83%), including shared drain change. |
| Repeated forge-history collection, `real-loop-causes` | Change-triggered snapshots and suffix cursor. Repair 833,682,304 → 6,401,309 inclusive Ir (70098.6 → 500.8 Ir/call); case 4,625,233,683 → 6,693,892 (186238.5 → 481.2). Partial endpoints prevent a whole-story gain claim. |
| Terminal byte windows, same | Shared linear byte search, about 74% fewer instructions per query: 105,000–108,000 → 27,475–27,563. Final footprint 1.85–2.08%; readiness counts vary. |
| Case cue byte windows, same | Earliest cue and tie priority retained, about 79% fewer instructions per observed query: 81.05 million → 16.94–16.95 million; final footprint 0.82–0.86%. |

`lower-stories` and `stories-go` changed coverage placement by supplying
holders and removing overlap; they claim no isolated instruction saving.
The conformance projection integration reconciliation supplied no new
timing claim. Closing profiles measure that combined code explicitly.
The trunk also includes intervening JSON repairs `bc0d8f4c4f` (whole string
runs and no second writing-pass measurement) and `ae9260a7e2` (plain-string
run decoding). They are not the discarded engine-causes trial. This plan
has no isolated before/after arm for either commit, so their individual
instruction savings are unknown here; the closing profiles include them.
The remaining JSON parser/measurement costs below retain validation and
escaping, rather than restating every earlier footprint as unpaid work.

## Causes found and left

Every figures file and readable task report's `Left` was reconciled here.
The old snapshot/policy/scan assignments and focused comparison overage
are resolved above; old blocked profiling notes are superseded by the
committed figures. The remaining shares retain their identifying arm.
No local cause was fixed by this closing task.

| Cause, location | Measured share or size | Why it stands / next work |
|---|---|---|
| Required allocation header layout, `skein/testing/skein-heap/src/lib.rs` | 3,929,295,579 inclusive Ir, 9.49% of engine-causes' unforked CI before; padding unchanged | Required checked System layout for the header, a cost no counted allocation can avoid. No meter or build override removed it; an override is the user's. Final conformance padding self 2.50% / 2.55% belongs to the same kit. |
| Independent JSON parsing, `skein/crates/skein-json/src/parse.rs`, `string.rs`, LLM/config producers | Baseline R 12.85–13.12%, P 6.03–9.81%, G 9.75–10.03%, U 10.15%; public-causes final U/O/M 14.13 / 13.92 / 6.99%; closing 8.74 / 8.59 / 3.85% | Each independent input must be validated. Avoiding schema reconstruction needs producer representation or bounded retained schema keyed by charter, namespace and grants; not a parser check to skip. |
| Earlier writing-pass recount, `skein/crates/skein-json/src/writer.rs` | Baseline R 10.52–11.09%, P 4.15–7.53%, G 6.81–7.03%, U 9.37%; closing `escaped_len` U/O/M 2.69 / 2.61 / 1.08% | Engine-causes discarded its validating one-pass trial: 141,324,686 Ir, 0.76% of U, below the stop rule. The intervening JSON commits above partly pay this cause by another implementation; their isolated savings are unknown here. Remaining measuring and escaping work stays, not the old duplicate-writing implementation. |
| Repeated prompt measurement, `smith/crates/smith-protocol-llm/src/component.rs`, `skein-llm` | R 6.20–7.35%, P 2.03–3.98%, G 3.70–4.26%, U 5.80% | Retain refusal before observation; transferring an admitted measurement through observation and reservation needs protocol work. `Component::start`'s larger public U/O/M footprints, 15.31 / 14.87 / 7.94%, include required work and overlap; closing start footprints are 9.24 / 8.94 / 4.27%. |
| Generated store bounds, `jig/crates/jig-protocol-store/src/codec.rs` | Baseline C 2.17%; R/P/G/U ≤0.29% | Production store already retains its view. C splits into independent codec-view construction 1.50% and largest/frame 0.33% each; stop rule. |
| Required engine steps, `temper/crates/temper-engine-domain/src/domain.rs` | Conformance-causes final 22.82 / 25.04%; closing 23.27 / 25.37%, unchanged step calls | Product execution, not demonstrated removable adapter work. Shared engine investigation remains separate. |
| Engine deadline queries, `temper/crates/temper-engine-domain/src/deadline.rs` | Public-causes final U/O/M 5.05 / 5.08 / 1.90%; closing 6.10 / 6.11 / 2.05% (same Ir) | Lower product footprint, not an isolated avoidable scan; changing deadline maintenance needs its own profile and proof. |
| Complete owned observations, `jig/testing/jig-conformance/src/core/projection.rs` and referee vocabulary | `Task::clone` conformance-causes final 3.62 / 5.83%, including scope copies; closing 3.38 / 5.37%. Closing durable `TaskRecord::clone` is another overlapping 5.01 / 4.98% | Full owned snapshots are the observation contract. Changed-row-only verdicts would skip checks; changing ownership requires a separate design. |
| Required restart and cold readback, conformance harness and adapters | 157 / 78 builds, 1,600 / 810 startup pages; unchanged | Every restart and independent public readback remains. Underlying setup is lower-layer work; no saving inferred from its enclosing frame. |
| Remaining conformance naming and words | Naming final 1.93 / 1.17%; Result/Words iterator 0.62 / 0.95% in indexed arm; direct diagnostic format 22.1 / 30.1 million Ir | Below local stop rule. Word checks, observations and diagnostic evidence remain. Inbound/event-debug retention was absent: no measured cause there. |
| Native process and copy-on-write floor, `skein/testing/skein-fork` | Split/wait plus hook ≈3,239 user-space Ir/cut; native suite system CPU 0.303 → 2.300 s | Kernel page-table/COW work is outside Ir. Forking saved repeated prefixes but retained this cost. |
| Typed defaults/configuration admissions, real-loop `fixtures.rs`, `landing.rs` | R/P/G defaults 0.88 / 2.54 / 1.48%; case admissions 2.17% baseline, 3.30% completed after | Twelve checks cover three successive configurations on four hosts. Required checks stay; parsing descendants need lower representation work, not a cross-story cache hiding admissions. |
| Remaining real-loop matching/setup | Terminal 1.85–2.08%, cues 0.82–0.86%; removable terminal preflight ≈1.2%; cluster writes 0.09% | Stop rule. Terminal opens, including spent-link recovery, remain. Timer paths must be slowdown-corrected before ranking. |
| Remaining public setup/context | Identifier engine fixture 236,030,770 Ir (1.77%); one owner parse ≈59 million (1.29%); M15 context 133,599,685 (1.06%) | Below local stop rule. Three identifier deployments retain tracked calibration and separate same-instant direct/proxy reads; sharing would exceed the eight-read burst or change that assertion. |
| Remaining M15 matcher | Construction 920,860,241 Ir (7.27% of final M15); overlapping search 22.25% | Required search footprint is not a saving. Retaining separate matcher groups regressed by 7.37%; individual prepared finders regressed to 29,193,119,109 whole-story Ir. Both trials reverted. |
| Callgrind-only forge observation cap and ending-attempt oracle | Partial repair/case runs in figures-before and real-loop-causes; closing outcomes above | Instrumentation alters scheduling and poll counts. These are failed prefixes, not accepted whole-story measurements; native confirmations pass. Observation capacity and journal promises remain intact. |
| Ignored reference process 4.6, `temper/bin/temper-box/tests/process.rs` | CPU and Ir unknown; not run | Jig must remove a draining worker from placement before its runs end. It supplies no current replacement coverage. After-landing owns slow-tier outcomes. |

Shares of R/P from unsuccessful profiles are diagnostic only. Allocator,
parsing, encoding and their parent functions overlap; adding those shares
would double-count work. Causes with no demonstrated fix remain causes,
including the unsuccessful writer experiment. The remaining suite and
placement decisions are open with their numbers above, without a raised
budget, merged test, reduced assertion or new exception.
