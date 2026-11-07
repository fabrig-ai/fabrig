# service-fixture: figures

Base `16c207bff23c4da176d17e59fd4f690c6e23bafa`, machine
`debian-32gb-hel1-14` through the build wrapper. Profiles measured
2026-11-06 UTC; verification completed 2026-11-07 UTC.

The pristine focused run passed 1,111 tests. Sorting its elapsed results by
time, then name, selected the floor and middle test from `setup::` (238),
`tests::stock::` (74), and the remainder (799). These names stay fixed for
the comparison. Each measurement uses one nextest process under Callgrind,
with double spawning disabled and the default profile. `Ir` is the whole
process, including the loader, harness, assertions and teardown, rather
than just the work preceding the first assertion. Inclusive and self
annotations were both read.

The selected stock median checks embedded words and builds no engine.
Two supplemental rows therefore measure stock construction and the shared
setup helper themselves.

| Row | Test | Floor or median | Ir before | Ir after | What was built before the first assertion |
|---|---|---|---:|---:|---|
| setup | `setup::tests::setup_reasons_fit_the_wire_at_a_character_boundary` | Floor | 11,535,102 | 11,792,634 | A 257-byte writer and boundary strings, including a two-byte character; no shared service fixture. |
| setup | `setup::tests::an_unresolved_forge_keeps_its_setup_entry_but_restored_and_asked_connects_never_dial` | Median | 330,036,179 | 136,116,045 | Engine settings, fixture files, two forge endpoints, a configured service and a completed records store. One real completed start precedes the assertion; another records-store replacement later in the test needs a second start. The whole test parsed the engine 11 times before; it now parses once and returns five independent copies. |
| tests::stock | `tests::stock::the_chats_words_read_a_changes_code_before_briefing_it` | Floor | 11,604,959 | 11,862,359 | An embedded instruction-string borrow and substring search; no generated document or parsed engine. |
| tests::stock | `tests::stock::a_goal_is_told_to_call_decided_before_it_asks` | Median | 11,695,171 | 11,952,571 | The embedded chat words and their goal paragraph, found by splitting paragraphs; no shared fixture. |
| remainder | `tests::forge::forge_component_limits_that_disagree_are_problems_naming_both` | Floor | 12,042,512 | 12,300,043 | Three refusal/expected-text pairs; the first refusal's diagnostic paths and formatted startup lines. No shared engine fixture. |
| remainder | `tests::store_route::a_loaded_turn_larger_than_the_tail_budget_keeps_its_texts_end` | Median | 47,228,970 | 33,051,260 | A 4,096-byte transcript, store routes, a decoded/rendered turn, and a core holding its transcript tail. Routes and core use two engine values; those now share one parse. |
| stock construction | `tests::stock::stock_chats_can_delegate_message_and_read_the_forge` | Supplemental | 606,417,447 | 127,478,937 | Init's generated document, its single-model variant, a parsed/passed engine, and the stock authority. The variant now shares one parsed full document across disjoint edits, plus one small provider document. Replacement bytes and untouched formatting are retained. |
| shared setup | `setup::tests::the_host_starts_after_loading_with_temper_urls_and_the_configured_limits` | Supplemental | 392,163,071 | 90,049,990 | A GitHub routing service with completed root/domain, durable records and a setup manifest/page. Three completed service starts become one, used for both domain state and durable maps; 12 engine parses become one parse and three copies. |

The setup median reaches **41.24%** of its original `Ir`, a **58.76%**
reduction, meeting the half criterion. The supplemental setup and stock
tests fall by 77.04% and 78.98%; the remainder's median falls by 30.02%.

## Causes

`M` is the selected setup median, `S` the supplemental setup test, and
`B` the supplemental stock test. Counts below are inclusive and overlap;
they must not be added. The threshold is about 0.463 CPU s, 1% of the
request's 46.3 CPU s for this binary. CPU estimates below use that
request's module totals, not elapsed times from this run.

| Cause | Where | Ir per test, on which test | Fixed in | Or what fixing it would take |
|---|---|---:|---|---|
| Repeated engine decoding, stock construction and validation | `fixtures::engine`, `store_limits`, `memory_store`, `records_store` | `engine`: 176,591,331 → 19,889,483, M | `fixtures.rs`; mechanical `Clone` derives on jig's engine and its owned members | Limits/models share one settings value within each store builder. A process retains one parsed engine and clones it. The final cold parse costs 16,090,370; all five clones together cost 3,754,582. |
| A store built only to extract another store's maps | `completed_maps` | 200,630,276 → 75,344,114, M | `fixtures.rs` | Consume the completed service's component directly. No replacement memory store, limits/model parse or extra map construction is needed. |
| Rebuilding completed services for store replacement and domain state | `setup::tests::service`, `routing_service_at` | `ready`: 179,109,777 over three starts → 52,888,806 over one, S | `setup.rs`, `records_store_from`, private configured-service helpers | Pass the ready service's maps into the routing fixture and retain its domain/limits. The configured service's already seeded store needs no second replacement. Public fixture signatures stay unchanged. |
| Recomputing the same checked codec view | `jig_protocol_store::Store::over`, `valid` | `Store::over`: 49,607,570 → 20,999,064, M | `jig/crates/jig-protocol-store/src/lib.rs` | A checked view serves decoded-record bounds, frame bounds and construction. `over` computes it once rather than three times; `valid` once rather than twice. Isolating this edit after fixture deduplication/caching lowered M from 157,919,114 to 136,116,045. Existing admission/refusal checks remain. |
| Parsing the full generated stock document for each read/edit | `single_model`, `without_projects`, fixture JSON helpers | `single_model`: 562,056,021 → 82,890,186, B | `stock.rs`, `fixtures/startup.rs` | Borrow raw value spans from one parsed document; apply disjoint replacements in source order. The provider is edited separately because its model edit lies inside the replaced provider array. |
| The first engine parse in each test process | `parse_engine`, `jig_protocol_config::pass` | 16,090,370, including 8,448,295 in `pass`, M after | Retained | A process cache cannot remove its cold construction. Removing this would require a typed/generated equivalent of the complete parsed stock and configuration, or a broader parser/validator optimization; no such replacement is introduced. |
| Real first-start settings, counters, records and restart | `ready`, `completed_maps` | `ready`: 73,462,094 across two starts, M after | Extra starts removed; remaining starts retained | A durable mark alone omits the real start's settings rows and deployment counters. Replacing this requires constructing and checking the complete first-start record set, not just writing `Wires`. The median deliberately replaces its store later; those two stores remain independent. |
| Component admission and construction | `Service::prepared` | 22,503,402 across three preparations, M after | Retained after removing redundant preparations | Further savings need checked settings shared across root/component constructors while preserving each constructor's admission checks. This spans the construction interfaces rather than a repeated fixture call. |
| Transient hosted components and secret-store exchanges | `memory_store_for`, `seed_forge_rows` | 14,128,844 and 11,576,284 respectively, M after | Repeated fixture calls removed; remaining construction retained | Avoiding the remaining disk-component-to-memory replacement and secret exchanges needs an in-memory hosted-store constructor over supplied maps, retaining driver state, wires and start-mark rules. Product internals are not copied into fixtures. |
| Application family sizes and root limit translation | `temper_protocol_store::application`, `limits::engine` | 4,128,478 and 1,461,550 respectively, M after | Retained | Sharing checked family/admission results across constructors is the same broader construction change above. Their setup-only maximal savings estimate is about 0.31 and 0.11 CPU s respectively. |
| Remaining single-pass stock parsing and init generation | `Json::new`, `temper_init::files` | 82,386,337 and 3,355,013 respectively, B after | Retained; stop rule | Charging even all 3.1 baseline stock CPU s at B's instruction rate bounds removing every remaining `Json::new` at about 0.42 CPU s, below 0.463. Init generation is about 0.017 CPU s on the same estimate, so a generated-byte cache is not justified here. |
| Loader relocation floor | `_dl_relocate_object` | 10,391,262 → 10,645,718, each floor | Retained | About 90.27% of the final setup floor. The larger test binary adds relocation work, explaining almost all the roughly 257,000-instruction increases in the cheap rows. Changing linker/build settings is outside the plan. |

The remaining engine-clone work has a setup-only maximal saving estimate
of about 0.28 CPU s. The larger retained costs above identify the state or
construction boundary that a further change would have to preserve.

## Verification

The base's 1,111 focused service tests remain, and all pass alongside
three new fixture checks: **1,114 focused tests passed**. The existing
separately filtered `tests::sign_in::fuzzy_route_endings_replay_from_the_seed`
also passed in the fuzzy profile. No test, assertion, case, seed, fixture
limit, expected value or generated baseline was removed or weakened.

Each new check was made to fail deliberately, then restored: changing a
returned cached policy failed complete parsed-value equality; removing
edit ordering failed the disjoint-span check; discarding the supplied
store lost its durable row. All three restored checks passed.

Clippy passed for `temper-service`, `jig-protocol-config` and
`jig-protocol-store`; both trees' old-form ratchets passed without changing
their baselines. All 42 direct dependent packages and
`temper-conformance-world` passed `cargo check --tests`. The two changed
jig packages and those consumers passed **5,990 focused tests**. Browser,
process and live tiers were not run.
