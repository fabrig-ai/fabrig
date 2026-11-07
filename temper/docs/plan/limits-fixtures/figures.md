# limits-fixtures: before and after

Machine: `debian-32gb-hel1-11`, confirmed through the build wrapper for both commits.
Date: **2026-11-07 UTC**; profiling and annotation ran 01:17–01:38 UTC.
Base: `7ffd748466210b3b131113777ca14de89f592874`.
Tip: `f81186919ac77f2d6ef5ebef094fe026fc9c50e6` (this branch’s starting commit).

## Measurements

The method is `temper/docs/plan/limits-fixtures/notes/common.md`, “How to profile”.
Each exact name ran alone under nextest’s Callgrind tracer, with
`NEXTEST_DOUBLE_SPAWN=0` and `TEMPER_BUNDLE=none`. Library tests used `--lib`,
and integration tests their named binary. T21 used the fuzzy profile;
all other samples used default. Every `temper` sample used
`--trace-children=yes` and a separate Valgrind diagnostic log. Both inclusive
and self annotations were read with `callgrind_annotate`.

`Ir` includes loading, setup, assertions and teardown. The percentage is
`100 * (tip / base - 1)`, so a negative value is a reduction. Floor/median
labels retain the contributing tasks’ selections. The table includes every
request row and all 37 tests named in the task figures, including their
additional probes and verification checks. Net changes include intervening
plans; the causes table below records the tasks’ isolated findings.
These samples do not establish a new suite-wide CPU total or median.

**All 68 available traces passed:** 37 at the tip and 31 at the base,
66 default-profile traces and two fuzzy traces. Each selected exactly one test.
Six added checks have no base figure. Profiles, annotations, logs and JSON
name lists remain under ignored `target/lf/`.

| ID | Request row / shared check | Test | Selection | Ir base | Ir tip | Change |
|---|---|---|---|---:|---:|---:|
| T05 | engine-domain-world all; root_routes | `root_routes::read_only_role_stages_have_one_not_ready_terminal_and_empty_projects_need_no_financial_ledger` | Floor | 9,728,820 | 9,717,216 | -0.12% |
| T06 | engine-domain-world forge_routes | `forge_routes::hints::a_named_issue_still_gets_its_hint_reads_and_wakes_its_change` | domain-fixture floor | 14,111,779 | 14,100,442 | -0.08% |
| T07 | engine-domain-world forge_routes | `forge_routes::questions::a_restart_keeps_an_open_question_where_it_stood` | domain-fixture median; restart anchor | 222,463,765 | 178,817,569 | -19.62% |
| T08 | engine-domain-world root_routes | `root_routes::an_amendment_reaches_a_live_run` | Median | 59,988,767 | 54,424,878 | -9.27% |
| T09 | engine-domain-world forge_routes | `forge_routes::hints::both_fresh_base_facts_remain_dependencies_when_the_pull_and_branch_disagree` | forge-routes-steps floor | 14,685,916 | 14,674,668 | -0.08% |
| T10 | engine-domain-world forge_routes | `forge_routes::b2_tree::directory::an_advisor_lists_the_root_and_finds_src_main_rs` | forge-routes-steps median | 347,409,886 | 247,845,017 | -28.66% |
| T11 | engine-domain-world forge_routes | `forge_routes::delegates::a_resolution_delegate_carries_its_base_as_hexadecimal_text` | Resolution probe | 140,101,717 | 136,371,666 | -2.66% |
| T12 | engine-domain-world all | `walking::walking_story_replays_exactly_with_store_and_page_latency` | Named walking replay | 3,230,517,020 | 3,209,625,315 | -0.65% |
| T13 | service setup; all | `setup::tests::setup_reasons_fit_the_wire_at_a_character_boundary` | Floor | 11,602,433 | 11,554,030 | -0.42% |
| T14 | service setup; all | `setup::tests::an_unresolved_forge_keeps_its_setup_entry_but_restored_and_asked_connects_never_dial` | service-fixture median; setup anchor | 330,101,725 | 133,257,856 | -59.63% |
| T15 | service tests::stock | `tests::stock::the_chats_words_read_a_changes_code_before_briefing_it` | Floor | 11,672,324 | 11,623,721 | -0.42% |
| T16 | service tests::stock | `tests::stock::a_goal_is_told_to_call_decided_before_it_asks` | Median | 11,762,524 | 11,714,170 | -0.41% |
| T17 | service all (remainder) | `tests::forge::forge_component_limits_that_disagree_are_problems_naming_both` | Floor | 12,109,884 | 12,061,187 | -0.40% |
| T18 | service all (remainder) | `tests::store_route::a_loaded_turn_larger_than_the_tail_budget_keeps_its_texts_end` | Median | 47,296,258 | 32,812,150 | -30.62% |
| T19 | service tests::stock | `tests::stock::stock_chats_can_delegate_message_and_read_the_forge` | Stock construction probe | 606,484,375 | 127,235,758 | -79.02% |
| T20 | service setup; all | `setup::tests::the_host_starts_after_loading_with_temper_urls_and_the_configured_limits` | Shared setup probe | 392,231,140 | 88,501,158 | -77.44% |
| T21 | service all | `tests::sign_in::fuzzy_route_endings_replay_from_the_seed` | Named fuzzy replay | 2,740,008,765 | 1,804,869,730 | -34.13% |
| T22 | service setup | `setup::tests::a_kept_github_record_serves_its_routes` | setup-steps median | 392,487,417 | 88,766,714 | -77.38% |
| T23 | service setup | `setup::tests::written_is_answered_only_after_kept_and_the_routing_rows_follow_each_keep` | Request/keep-order probe | 487,682,813 | 124,685,598 | -74.43% |
| T24 | service setup | `setup::tests::lifecycle_tests::calls_are_not_connected_before_connect_then_wait_until_connected` | Named connection-only probe | 504,238,975 | 141,205,588 | -72.00% |
| T37 | service setup | `setup::tests::lifecycle_tests::setup_malformed_queries_are_bad_requests` | Named malformed-query check | 6,129,810,754 | 1,033,378,398 | -83.14% |
| T01 | client-world all | `forges_edges::forge_queries_canonicalize_good_bad_and_repeated_values` | Floor | 8,818,524 | 8,784,731 | -0.38% |
| T02 | client-world all | `dismissed::a_dismissal_outlives_a_restart` | Median | 188,225,543 | 74,494,676 | -60.42% |
| T03 | client-world all | `recovery::seed_five_recovers_online_after_the_four_faults` | Recovery probe | 275,900,596 | 254,731,468 | -7.67% |
| T04 | client-world all | `fixtures::tests::seed_five_replays_with_cold_then_warm_phrasebooks` | Added replay check | unknown | 474,792,299 | unknown |
| T30 | client-view outside tests::limits | `tests::frame::known_sign_in_ways_ignore_unknown_words_and_duplicates` | Floor | 3,400,211 | 3,382,326 | -0.53% |
| T31 | client-view outside tests::limits | `tests::accounts::a_sign_in_in_an_omitted_row_stays_visible_without_duplicating_drawn_rows` | Median | 93,201,462 | 74,111,943 | -20.48% |
| T32 | client-view outside tests::limits | `tests::a_long_forge_path_shows_its_last_whole_segment` | Words fixture probe | 45,485,333 | 37,876,696 | -16.73% |
| T25 | temper startup | `startup::a_rung_reads_created_chats_waited_changes_and_both_child_sets_in_ctls_forms` | Floor | 12,515,737 | 12,483,970 | -0.25% |
| T26 | temper startup | `startup::forge_run_refuses_a_listener_at_the_wrong_address_before_taking_the_lock` | Median | 79,063,382 | 72,473,775 | -8.33% |
| T27 | temper forge | `forge::the_run_names_prepare_for_a_runner_removed` | Floor | 83,125,667 | 76,439,968 | -8.04% |
| T28 | temper forge | `forge::a_forge_directory_open_to_others_is_refused_naming_it` | Median | 81,587,662 | 74,416,696 | -8.79% |
| T29 | temper startup/forge | `tests::box_kinds::a_checked_machine_keeps_resolved_programs_for_both_group_kinds` | Added checked-machine check | unknown | 41,438,267 | unknown |
| T33 | service / engine-domain-world shared store | `tests::over_refuses_insufficient_transcript_readers` | Added constructor-refusal check | unknown | 3,623,799 | unknown |
| T34 | engine-domain-world shared codec | `codec::people::pieces::tests::fixed_piece_framing_matches_the_generated_bounds` | Added fixed-framing check | unknown | 2,734,706 | unknown |
| T35 | client-view shared phrasebook | `tests::phrases::duplicate_ids_are_refused_before_conflicting_holes_or_forms` | Added duplicate-order check | unknown | 889,416 | unknown |
| T36 | client-view shared phrasebook | `tests::phrases::shared_declaration_order_and_independent_order_keep_every_id_bit` | Added ID/order check | unknown | 870,186 | unknown |

Binary names are `temper-engine-domain-world::suite`, `temper-service` (library),
`temper-client-world::suite`, `temper-client-view` (library) and `temper::suite`.
T04 and T29 use their packages’ libraries; T33–T34 use `jig-protocol-store --lib`,
and T35–T36 use `skein-ui --lib`. The service “all” row is represented by its
setup/stock samples and the two remainder samples; the engine “all” row by
its binary floor, route samples and walking replay. No aggregate saving is
inferred by adding overlapping request rows.

### Unknown base figures

These definitions were added after the base; their exact names have no base
source definition. Their tip measurements passed.

- T04: added by `d515e77e9c` (`client-world`); base `Ir` and percentage are `unknown`.
- T29: added by `efde31e01a` (`suite-startup`); base `Ir` and percentage are `unknown`.
- T33: added by `0f9f39128a` (`domain-fixture`); base `Ir` and percentage are `unknown`.
- T34: added by `341cc40763` (`forge-routes-steps`); base `Ir` and percentage are `unknown`.
- T35: added by `289d1453bb` (`view-words`); base `Ir` and percentage are `unknown`.
- T36: added by `289d1453bb` (`view-words`); base `Ir` and percentage are `unknown`.

### Command descendants

T25 has one process on each side. T26–T28 each have two: the suite process
and the final `temper` executable. Their per-process totals, base → tip, are:

- T26: suite 33,178,737 → 33,145,132; `temper` 45,884,645 → 39,328,643; sum 79,063,382 → 72,473,775.
- T27: suite 36,263,098 → 36,230,993; `temper` 46,862,569 → 40,208,975; sum 83,125,667 → 76,439,968.
- T28: suite 34,309,311 → 34,246,515; `temper` 47,278,351 → 40,170,181; sum 81,587,662 → 74,416,696.

Callgrind’s `%p` output retains the final executable image after exec;
the shell before exec in T26 is unobserved. Kernel work is also unobserved.
The sums follow the same per-process method as `figures/suite-startup.md`.

### Tip readings above a task’s own after

The following comparisons use the contributing figures’ after column,
rather than the plan base. Each larger reading is retained as measured:

- T07 against `forge-routes-steps`: 177,192,064 → 178,817,569, +1,625,505 Ir (+0.917%).
- T11 against `forge-routes-steps`: 134,885,818 → 136,371,666, +1,485,848 Ir (+1.102%).
- T13 against `service-fixture`: 11,546,466 → 11,554,030, +7,564 Ir (+0.066%).
- T15 against `service-fixture`: 11,616,313 → 11,623,721, +7,408 Ir (+0.064%).
- T16 against `service-fixture`: 11,706,557 → 11,714,170, +7,613 Ir (+0.065%).
- T17 against `service-fixture`: 12,054,038 → 12,061,187, +7,149 Ir (+0.059%).
- T18 against `service-fixture`: 32,804,786 → 32,812,150, +7,364 Ir (+0.022%).
- T19 against `service-fixture`: 127,232,788 → 127,235,758, +2,970 Ir (+0.002%).

`git log 998ad2daa7..f81186919a -- jig/crates/jig-protocol-store temper/tests/engine/domain temper/crates/temper-engine-domain`
shows the main catch-up `ca4a1776be`, including `ui-proposal-2` merge
`1e0a79fbee`, proposal revision `74918c6e56`, goal-delivery merge `7629156b6a`,
run-branch delivery `5c03a98bcf` and next-turn amendment `9d6132ffbe`.
For T07/T11 the loader is lower than FR’s after: `_dl_start`
8,908,297 → 8,695,737 / 8,695,743; their net increases lie outside it.

The corresponding service fixture history includes the later `setup-steps`
merge `f81186919a`, the main catch-up, proposal/goal changes, and stock-charter
room change `971a621eee`. T13’s relocation count is 10,401,972 versus SF’s
10,399,796; that accounts for only part of its increase. T13 is below ST’s
later after total, 11,579,609. The precise contribution of each intervening
change to these eight increases is **unknown**; task-local figures also use
different starting trees and sometimes different builders. No repair is
attempted by this measurement task.

## Causes and what remains

Costs here are the contributing tasks’ own before/after profiles, whose
bases are recorded in those files. They explain the fixes and retained
boundaries; they are not new partitions of the table above. Inclusive costs
overlap and must not be added. “About”/million figures retain the source’s
precision. F rows identify fixed inefficiencies; R rows identify every
remaining cause category, including the task-local stop-rule candidates.
Later fixes are folded in: discarded restart worlds reported by DF were
removed by FR, and unused Views were removed by CW while drawing retains them.

Sources:

- SF: [service-fixture](figures/service-fixture.md).
- ST: [setup-steps](figures/setup-steps.md).
- VW: [view-words](figures/view-words.md).
- DF: [domain-fixture](figures/domain-fixture.md).
- FR: [forge-routes-steps](figures/forge-routes-steps.md).
- CW: [client-world](figures/client-world.md).
- SS: [suite-startup](figures/suite-startup.md).

| ID | Cause | Where | Task-sampled Ir | Fixed by / remaining work | Sources |
|---|---|---|---|---|---|
| F1 | Repeated engine parses within one service test | `service fixture engine/store builders` | Setup anchor engine: 176,591,331 → 19,889,167; one cold parse 16,090,054 and five clones 3,754,582 | **service-fixture**. Retain one parsed engine per process; share settings within builders. Cold parse and clone costs remain. | SF |
| F2 | Completed stores and services built only to discard them | `completed_maps; setup service/routing builders` | Setup anchor maps: 200,630,276 → 75,344,282; supplemental ready: 179,109,777 (three) → 52,888,762 (one) | **service-fixture**. Consume completed maps and reuse the ready service. Required independent starts remain. | SF |
| F3 | Full stock document parsed for each edit | `single_model; without_projects; startup fixture JSON edits` | Stock construction single_model: 562,056,021 → 82,890,359 | **service-fixture**. Apply disjoint raw-span edits from one parse; preserve replacement bytes and formatting. | SF |
| F4 | Same codec view mapped repeatedly | `jig Store::over/valid and bound checks` | Service anchor Store::over: 49,607,570 → 20,999,660; domain per remaining constructor about 1.711m → 0.840m | **domain-fixture; retained by service-fixture replay**. One checked_codec mapping serves checks and construction; no refusal changes. | DF, SF |
| F5 | Replacement store opened just before bootstrap driver is dropped | `first_start_over; consuming driver/store transfer` | Domain root: 3,462,531 for the removed transfer; forge: 10,387,701 across three before | **domain-fixture**. Consume bootstrap storage into the live world, leaving its broken oracle an empty shell. Reusable take_store is unchanged. | DF, CW |
| F6 | Identical init profiles/model parsed repeatedly in one forge story | `box_limits/box_joins; b2_tree::model` | Advisor limits: 134,169,194 → 67,274,150; model: 32,302,792 → 6,610,150 | **forge-routes-steps**. Retain real parsed Copy projections per process; their first parses remain. | FR |
| F7 | A started world discarded just to obtain restart configuration | `restart_with_charter` | Question anchor, two calls: 39,577,677 → 9,568,400; three full worlds → one | **forge-routes-steps**. Use the paired configuration builder directly; retain the live world and real restore. This resolves domain-fixture’s discarded-start finding. | FR, DF |
| F8 | Family validation repeated for constant piece framing | `jig codec people pieces shared_sum/permission_sum` | Question shared_sum: 3,752,738 → 54,322; permission_sum: 86,340 → 984 | **forge-routes-steps**. Hold 17-byte grant and 12-byte permission framing against generated bounds. | FR |
| F9 | Same phrase ID sequence sorted twice | `Phrasebook::load` | Words load: 24,221,512 → 16,863,413; two sorts before 18,448,343 | **view-words**. Reuse declaration positions after checking language ID order; independent languages still sort. English overall: 32,916,144 → 25,558,177. | VW |
| F10 | Nested phrase ID comparisons on every lookup | `Phrasebook::capacity/say` | Same intermediate scan count: 31,748,760 → 27,082,408; final capacity scans 20,311,910 | **view-words**. Private u64 keys preserve every public owner/phrase bit. | VW |
| F11 | Whole vocabulary maximum scanned repeatedly per limits calculation | `Limits::of` | View median: 23,828,048 → 13,557,736, including cheaper lookup | **view-words**. Compute the maximum once per invocation. Further presence/capacity scans remain. | VW |
| F12 | Views built and immediately discarded by root-only setups | `dismissal/chat/adoption probes and front tests` | Client median App: 77,544,952 → absent; probes: 85,072,279 → 2,364,746 | **client-world**. Use the existing unlinked root constructor. Drawing stories retain complete Views. | CW |
| F13 | Fixed turns continue after setup request settles | `Face::sent; reply_status; raw reply and lifecycle helpers` | Request Face::turn: 83,514,988 → 23,767,492; lifecycle advance: 47,500,853 → 1,810,203 | **setup-steps**. Stop after setup io, readiness, due delays/deadlines and closure settle. Preserve real future timers and scenario turns. | ST |
| F14 | Already checked machine decoded again | `forge entry; box/serve preparation; setup cross-checks` | Startup median separate machine read: 6,291,849 → zero; forge floor 6,387,123 → zero; forge median 6,722,214 → zero | **suite-startup**. Reuse Named.machine, including within setup validation, preserving validation and refusals. | SS |
| R1 | Loader relocations and process/harness floor | `dynamic loader; test harness` | Task floors: service relocation 10,399,796; domain 8,794,338; client 8,049,579; view 2,693,383; temper suite 11,664,909 plus child 10,588,953 | **Remaining**. Requires binary/loader/linker work; build settings are closed. Fixture changes cannot remove process startup. | SF, DF, CW, VW, SS |
| R2 | First typed engine/init configuration parse and validation | `parse_engine; box_profiles; startup::engine_with` | Service 16,090,054; advisor box_limits about 67m; init model 6,621,723 including copies; temper engine about 9,581,142 | **Remaining**. A generated typed stock/projection checked against the shipped input, or parser/validator work, including skein Slice::answer allocation/copy. Per-process caching cannot remove the first call. | SF, ST, FR, SS |
| R3 | Two startup document parses with different consumers | `startup::named_with; setup::check_with` | Startup median config::document 17,487,400 → 11,669,980; forge samples about 12.1m/12.3m after | **Remaining**. Share the bounded parsed document through startup/setup interfaces while preserving depth/token/byte limits and diagnostic ordering. | SS |
| R4 | Remaining stock parse and init generation | `Json::new; temper_init::files` | Stock construction 82,386,337 and 3,355,072 after | **Remaining; below task stop rule**. Further parser/generator work. Task whole-stock upper estimates are 0.42 and 0.017 CPU s; a generated-byte cache was not justified. | SF |
| R5 | Real first start, durable rows and cold restoration | `ready; first_start_over; restored_forges` | Service anchor ready 73,462,262 over two starts; domain root first start 5,077,207; client recovery first start 3,132,782 | **Remaining**. An independent configuration-dependent writer for complete policy/settings/start rows, counters and durable state, checked against the real two-commit script. A start mark alone is insufficient. | SF, ST, DF, FR, CW |
| R6 | Repeated admission and owned policy/configuration snapshots | `Service::prepared; Domain::new; Core::new; engine clones` | Service prepared 22,503,386 over three; setup median 14,989,265; domain root constructor 1,533,498; service five engine clones 3,754,582 | **Remaining**. Share checked construction results or change snapshot/config ownership while retaining distinct admission checks and journal/restart lifetimes. | SF, ST, DF, FR |
| R7 | Hosted store replacement and secret exchanges | `memory_store_for; seed_forge_rows` | Service anchor 14,128,888 and 11,576,556; setup median 7,064,836 and 5,677,577 after | **Remaining**. An in-memory hosted-store constructor over supplied maps, retaining driver state, wires, durable startup rules and secret exchanges. | SF, ST |
| R8 | Sizing translations and generated bound validation | `store::component_limits; service application/limits translation` | Domain root component sizing 1,454,971; service application 4,128,514 and root limit translation 1,461,550 after | **Remaining**. Shared checked sizing/codec interfaces across application, commit/frame and store boundaries, or generator changes with schema/bound tests. Discarded stores no longer pay this cost. | DF, SF, CW |
| R9 | Reusable replacement store during a story restart | `Driver::take_store; Store::with_limits` | Client recovery take_store 2,542,239; store construction 5,044,655 across construction/restart; dismissal stores 5,045,040 after | **Remaining**. A distinct consuming restart cut path preserving the retained driver destructor/oracle contract; the public reusable take_store must leave usable replacement storage. | CW |
| R10 | Real row encoding/decoding and temporary owned values | `jig codecs; temper store routing` | Question anchor encode 9,516,131; decode 10,654,467 after | **Remaining**. Borrowed translation/writer interfaces that preserve encoded bytes, malformed/bound refusals and real codec execution. Typed-mirror bypass would weaken the world. | FR |
| R11 | Separate owning rows for history, mirror and inflight write | `world Store::tick/event` | Question Write::clone 3,209,023; resolution story 3,292,190 after | **Remaining**. A borrowed commit-encoding seam or durable-fork representation; mirror and replayable history still retain independent values. | FR |
| R12 | One-row restore/map requests and cursor/key copies | `pages::reader::walk_map; load_map` | Question walk_map 12,359,983, including load_map 5,000,247 after | **Remaining**. A batching/paging algorithm preserving encoded bounds, lookahead cursor, first-row refusal and corruption/diagnostic order, with boundary tests. | FR |
| R13 | Actual story decisions, engine stepping and readback | `root/core step; pump_component; Driver::advance; page_with_texts` | Resolution root step 59,051,897 and next_delegate about 55.09m; client dismissal advance 19,606,484 and pages 6,841,271 after | **Remaining**. Profile admission/snapshot and page projection/stepping mechanisms with lower-tier stories; retain all inputs, durable commits, deliveries and assertions. | FR, CW |
| R14 | Independent teardown and restart oracle | `driver close/oracle and final-store comparison` | Client dismissal 7,261,031; recovery 6,566,600 after | **Remaining**. Cheaper snapshot/paging and retained-configuration representation while preserving the independent restart and final-store oracle. | CW |
| R15 | Cold English construction, remaining ordering and allocations | `english_language; Phrasebook::load; vocabulary writers` | Words construction 8,691,991; remaining shape order 9,725,414 and language permutation 1,172,579; client recovery English 25,508,487 after | **Remaining**. Already ordered vocabulary writers with a checked ordered-input path and a cheaper construction representation, retaining every word, form and hole. | VW, CW |
| R16 | Owned nested Words clones and teardown | `fixtures::words; Words::clone; dropping book` | View words clone 6,740,696; median two clones 13,401,334; drop 1,945,186 / 3,874,423; client recovery clone 6,540,983 after | **Remaining**. Flatten loaded-book allocations or change caller/ownership/lifetime interfaces. View owns Words; removing its cache would rebuild the book for repeat clients. | VW, CW |
| R17 | Remaining capacity/presence/maximum scans | `Words::capacity; limits::valid; View::new` | View median scans 20,311,910; valid 13,543,512; View::new 6,773,820; client recovery capacity 10,160,091 after (overlapping) | **Remaining**. A trusted checked vocabulary summary/representation that still detects public book/connector mutation, missing vocabulary and overflow. Domain limits alone cannot key a safe cache. | VW, CW |
| R18 | Whole-book validation and capacity arithmetic | `check_forms; form_capacity; Limits::of residual` | Words 1,188,027 and 702,498; median local limits arithmetic 14,224 after | **Remaining**. Fuse checked passes with preserved whole-load error precedence and bounded occurrence counters. Local limits arithmetic alone is below the stop rule. | VW |
| R19 | Semantic drawing and tree diff during real stories | `view build; skein UI diff/match/parent/move traversal` | Client recovery view build 70,550,502; diff 36,960,027, including match_nodes 20,422,823 and parents 9,939,548 | **Remaining**. A different indexed traversal preserving keyed/role sibling matching, stable identities and exact patch behavior, checked in skein UI tests. | CW |
| R20 | Applying and independently checking drawn trees | `world build; model apply/equality` | Client recovery world build 106,345,381 including view; model check 8,512,729; apply entry points 5,049,215 and 5,036,207 after | **Remaining**. An incremental model/checking representation preserving the independent patch oracle and callback builds. | CW |
| R21 | Domain setup and semantic rendering that assertions consume | `account_root; shown_at` | View median roots 4,017,336; shown_at 70,387,990 including setup after | **Remaining**. A separate domain/rendering profile and mechanism change; retain differently bounded Views, domain facts and asserted text. | VW |
| R22 | Paired configurations, piece/config copies and idle bookkeeping | `configured_settings; piece splitting; restart copy; reclaim` | Domain three paired builds 5,603,661; question piece split 1,613,137, config copy 1,579,880, reclaim 345,881 after | **Remaining; below local stop rule**. Factor paired per-configuration construction or introduce borrowed codec/config views without losing consumed inputs. Reported individual upper estimates stay below 0.529 CPU s. | DF, FR |
| R23 | Referee/default words, fake/component/Truth setup and engine limits | `World::new; decision_words; Settings::default; Composition; core_rows; fake forge; walking config` | Client recovery 1,684,853 / 775,331 / 301,609; component 495,875; core rows 366,864; Truth 11,211; fake forge 95,150; engine limits 4,163; walking config 99,397 | **Remaining; below local stop rule**. An explicit words constructor or lazy/copy changes would add interfaces/state for sub-threshold savings. Preserve fakes and complete decision vocabulary. Domain fake construction is only 86,411 over three pairs. | CW, DF |
| R24 | Required GitHub signing | `skein_tls::rs256::sign_rs256` | Setup request 22,611,818 before and after | **Remaining**. Cryptographic implementation work, or a separately justified fixture seam retaining wire authentication; a canned token would skip the signing path. | ST |
| R25 | Remaining Face, connection and local settling/manifest mechanics | `Face::settle; connection_turn; terminal; load_routing; manifest rendering` | Request settle minus signing 1,717,026; connection-only test 16,219,412 including signing; setup median advance 763,060 and manifest 337,246 after | **Remaining; task stop rule**. Connection settling must retain forge.above for assertions; keep required framing/routing and scenario time advancement. Connection-only estimated reach is 0.060 CPU s; local advance/manifest upper estimates 0.211/0.093. | ST |
| R26 | Process fixtures, measured turn bounds and scripts | `support configuration::documents; turn_bounds::turn_body; owned::scripted` | Startup documents 12,603,611 before; turn_body about 8.638m after; owned setup about 21m; script-only work about 19,400; kernel cost unknown | **Remaining**. A cheaper equivalent bound encoder or byte-identical runtime-path templates; shared process fixture work belongs to model-catalogue-2/process-fixtures. Measure kernel file/exec cost separately while preserving process proof. | SS |
| R27 | Cheap rule checks and harness work | `sign-in ways; stock words; CLI JSON answer assertions` | View floor excluding relocations 714,248 after; startup floor body 330,332 before | **Remaining; below local stop rule**. Retain the behavior checked. No shared expensive fixture is reached; harness work is outside fixture repairs. | VW, SS |

The contributing tasks apply the common approximately 1%-of-binary stop
rule using instruction shares and the request’s CPU totals. Those CPU
extrapolations remain estimates. Larger retained boundaries name the
representation/interface/algorithm work required; cheap rule assertions,
real story work and independent oracles remain exercised.

## What has not moved

### Test names (step 3)

At each commit the exact requested command completed successfully:

```text
TEMPER_BUNDLE=none heavy cargo nextest list -p temper-service -p temper-client-view -p temper-engine-domain-world -p temper-client-world -p temper --message-format json
```

The outputs are `target/lf/base-list.json` and `target/lf/tip-list.json`;
`target/lf/names-comparison.json` records the set comparison. Keys are
(package, binary kind/name, test name). Every listed test is included,
including ignored and profile-filtered names; listing compiled their targets
and ran none of them. Package totals, base → tip:

- `temper-service`: 1,112 → 1,118.
- `temper-client-view`: 1,454 → 1,474.
- `temper-engine-domain-world`: 931 → 942.
- `temper-client-world`: 420 → 422.
- `temper`: 495 → 496.

**Result: 4,412 → 4,452 names; 51 added, 11 removed/renamed, all accounted
for by other work. No base name was removed by limits-fixtures.**
All 31 sampled base names are present at the tip.

Ten removed view names belong to the model-picker look request,
`50e7171ce137237af4d581af9aec6ca2febf3a74` (“Keep model and effort choices
beside the composer”), carried by `look-round-11/integration` merge
`3c7305789e`. Nine are in
`temper/crates/temper-client-view/src/tests/choice.rs`:

- `tests::choice::a_byline_without_an_amend_offer_has_no_panel`.
- `tests::choice::an_empty_provider_links_to_its_accounts_fix`.
- `tests::choice::an_empty_provider_offers_connect_without_any_model`.
- `tests::choice::an_unapplied_model_has_a_separate_draft_label`.
- `tests::choice::change_model_contains_the_controls_and_is_never_expanded`.
- `tests::choice::every_byline_keeps_the_accounts_provider_with_its_models`.
- `tests::choice::manage_usage_is_the_panels_last_control_and_never_in_the_byline`.
- `tests::choice::pending_choice_outcome_and_wait_are_siblings_after_the_chooser`.
- `tests::choice::the_byline_and_chooser_are_in_the_session_bar`.

The tenth is `tests::configure::configure_uses_template_threes_preview_with_a_chooser`,
in `temper/crates/temper-client-view/src/tests/configure.rs`, removed by the
same commit. Its diff replaces the old chooser assertions with the new picker.

The remaining name is
`configuring::a_pending_proposal_holds_the_next_until_an_owner_rejects_it`,
in `temper/tests/engine/domain/tests/suite/configuring.rs`.
`74918c6e56af1d24343f4edf788a11a7622e8828` (`ui-proposal-2`, carried by merge
`1e0a79fbee`) replaces it with
`configuring::a_chats_fresh_proposal_replaces_its_own_that_waits` and
`configuring::a_chats_refused_fresh_proposal_leaves_its_own_waiting`.
Both new names are in the tip list. The removal attribution comes from
`git log -p -G 'fn NAME' 7ffd748466..f81186919a` and its source diffs.

### Settings (step 4)

`git diff 7ffd748466 f81186919a -- .config/nextest.toml tools/tiers/budgets.toml`
is empty (zero bytes). `git log` over that range for those paths is empty too.
The diff of **all** `Cargo.toml` files, including the workspace manifest,
is also empty (zero bytes), so it contains no changed `[profile` line.
The captured diffs are `target/lf/settings.diff` and `target/lf/manifests.diff`.

**Result: unchanged nextest settings, tier budgets and Cargo profiles.**
There is no settings change to attribute to this or another plan.
This task changes only the two plan documents: no code, design section,
test, assertion, fixture, build setting, budget or timeout changes.
