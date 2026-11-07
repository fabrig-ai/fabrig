# domain-fixture: figures

Base `16c207bff23c4da176d17e59fd4f690c6e23bafa`, machine
`debian-16gb-hel1-8` (through `heavy hostname`), 2026-11-06 UTC.
Before profiles precede every edit; after profiles use the final source changes.
All profiles use the unoptimized `temper-engine-domain-world --test suite`,
nextest's default profile and Callgrind tracer, without double spawn.
`callgrind_annotate` supplies the instruction counts below. Profiles and logs
remain under ignored `target/lf/`.

The pristine focused run selected 893 tests and passed all 893. Sorting that
run's durations, with names breaking ties, selected the following tests;
the medians are positions 213 of 425 and 63 of 125. The binary's cheapest
test is also the cheapest `root_routes` test.

| Row | Test | Floor or median | Ir before | Ir after | What was built before the first assertion |
|---|---|---|---:|---:|---|
| binary | `root_routes::read_only_role_stages_have_one_not_ready_terminal_and_empty_projects_need_no_financial_ledger` | floor | 9,690,620 | 9,918,063 | Role-story limits with an oversized holdings bound; checks root validity before constructing the task child. No store or first start. |
| forge_routes | `forge_routes::hints::a_named_issue_still_gets_its_hint_reads_and_wakes_its_change` | floor | 14,073,616 | 14,301,100 | `hint_world`: forge-only world, fake repository, two changes and their name/hint subscriptions; creates an issue and delivers its name. No engine root, store or first start. |
| forge_routes | `forge_routes::questions::a_restart_keeps_an_open_question_where_it_stood` | median | 222,424,810 | 209,962,374 | Two policy configurations, fake repository/grants, memory-backed real store, real two-commit first start, account rows and cold root; then adoption, worker hello, requester chat and delegations to the change and producer, followed by a question. Fixture helpers also assert their setup. |
| root_routes | `root_routes::read_only_role_stages_have_one_not_ready_terminal_and_empty_projects_need_no_financial_ledger` | floor | 9,690,620 | 9,918,063 | Same test as the binary floor. |
| root_routes | `root_routes::an_amendment_reaches_a_live_run` | median | 59,950,749 | 55,090,744 | `batch_settings(2, 1, 500, false, false, false)`, two configurations, real memory store, real first start plus accounts, cold root, worker hello, sign-in and parent chat; delegates a child and amends its live run. |

The median forge story saves 12,462,436 Ir (5.60%); the median root story
saves 4,860,005 Ir (8.11%). The floors do not use the changed fixture path.
Their increases are almost entirely in loader relocations: relocation Ir
changes from 8,565,110 to 8,794,338. No build settings changed.

## Where the instructions go

The following top-level partitions are disjoint. The residual includes the
story, its teardown/restart oracle, and test harness work outside the loader.
The forge story constructs three worlds across its explicit restart and
oracle; its construction entry is the sum of those three calls, rather than
a claim that all three precede the first assertion.

| Test | Partition | Ir before | Ir after |
|---|---|---:|---:|
| forge median | loader, `_dl_start` | 8,673,573 | 8,902,800 |
| forge median | `World::configured_settings`, three calls | 61,408,235 | 48,721,969 |
| forge median | story, teardown and harness residual | 152,343,002 | 152,337,605 |
| root median | loader, `_dl_start` | 8,673,573 | 8,902,800 |
| root median | `batch_fixture_custom`, including its sign-in/chat | 23,859,886 | 19,641,736 |
| root median | story, teardown and harness residual | 27,417,290 | 26,546,208 |

The nested boundaries below explain construction and product costs. These
are inclusive annotations and overlap: first start includes a root and
store transfer; store construction includes opening. Root totals include
story/oracle constructions too. They must not be summed with each other.

| Test | Boundary | Ir before | Ir after |
|---|---|---:|---:|
| forge median | paired configuration builder, `array::from_fn` under `configured_settings` | 5,601,479 | 5,603,661 |
| forge median | fake domain construction | 4,860 | 4,860 |
| forge median | fake repository construction | 81,752 | 81,551 |
| forge median | `first_start_over`, three real starts | 40,467,159 | 30,393,993 |
| forge median | `Store::with_limits`, six calls before, three after | 20,658,598 | 7,722,899 |
| forge median | `Store::open` | 333,286 | 169,251 |
| forge median | engine `Domain::new`, nine calls | 8,406,072 | 8,406,540 |
| root median | `batch_settings` | 219,377 | 219,377 |
| root median | `first_start_over`, one real start | 8,434,627 | 5,077,207 |
| root median | `Store::with_limits`, two calls before, one after | 6,888,037 | 2,576,147 |
| root median | `Store::open`, including oracle reopen | 431,421 | 376,833 |
| root median | engine `Domain::new` | 1,533,451 | 1,533,498 |

The binary/root floor has no configuration, fake, store, first start or
engine `Domain::new`; its later task-child constructor costs 4,972 Ir.
Loader startup is 89.50% before and 89.76% after; relocations alone are
88.39% and 88.67%. The forge floor's loader startup is 61.63% before and
62.25% after; `hint_world` costs 3,405,697 Ir before, including the fake
domain (1,620 Ir), repository (19,338 Ir) and story setup. Its remaining
work is forge stepping and teardown, without an engine first start.

## Causes

The stop-rule reference is 1% of the request's 52.9 CPU s, about 0.529 s.
CPU estimates below use the request's module totals and the measured Ir
shares; they are estimates, not new timing measurements.

| Cause | Where | Ir per test, on which test | Fixed in | Or what fixing it would take |
|---|---|---:|---|---|
| A store opened only to replace a transferred store before dropping its bootstrap driver | `direct::Driver::take_store` reached by `fixtures::first_start_over` | forge median: 10,387,701 in three transfers before; root median: 3,462,531 in one | `direct.rs`, `fixtures.rs`, `store.rs`: a consuming transfer leaves only an empty shell for the broken oracle's destructor | Public `take_store` retains its reusable replacement behavior. The new consuming path alone avoids the replacement component, size mapping and open. Intermediate forge total after this fix alone: 212,361,500 Ir. |
| Same codec families mapped for decoded bounds, frame bounds, then the live store | `jig-protocol-store/src/lib.rs`, `Store::over` | forge median: 10,264,897 over six constructors before, 2,520,202 over three after; per remaining constructor approximately 1.711m to 0.840m | `checked_codec` retains the one mapping used by both checks and construction | All limits, refusals, row encodings and public signatures stay the same. Constructor refusal is held by `tests::over_refuses_insufficient_transcript_readers`; existing validity tables hold the other edges. |
| Loader relocations | binary loader | floor: 8,565,110 before, 8,794,338 after | left | Binary/linker work; build settings are closed by this task. A fixture cannot remove it. |
| Real first-start script and its two commits | `fixtures::first_start_over` | forge median: 30,393,993 after; root median: 5,077,207 after | replacement-store overhead removed | Remaining work constructs the real root, settles the real script, encodes/commits its rows, and drops the bootstrap root. Direct seeding would need a configuration-dependent independent writer for policy, settings and start rows, checked against the real script. That is not a cheap constant for these varied policies; keep the real start. No pre-seeded replacement was introduced. |
| Repeated starts and cold readback during a story | `forge_routes` restart and teardown/oracle | forge median: three constructor calls, 48,721,969 after; story/harness residual 152,337,605 | left to story stepping | `forge-routes-steps` owns restart/stepping; eliminating these calls requires retaining constructor inputs through that path. Store page/restore routing and record cloning also appear in that residual. No stepping code changed here. |
| Store sizing and repeated generated bounds checks | `store::component_limits` through service sizing and generated family bounds | forge median: 4,365,015 after in three calls; root median: 1,454,971 in one | eliminated for discarded stores | Sharing a checked view across commit/frame sizing and the application/store boundary needs additional sizing APIs; generated `encoded_bound` routines also repeatedly validate family ceilings. Further reduction needs those interfaces or the codec generator and its schema tests, beyond this small constructor fix. |
| Root admission/policy snapshots and configuration copies | engine `Domain::new`, jig `Core::new`, restart oracle config | forge median: engine constructor 8,406,540 after; root median: 1,533,498 | left | Removing policy clones requires changing ownership of admission snapshots and restart configurations; the root must still validate each distinct configuration. No limits-sized empty table allocation dominates: the maps grow with their contents. |
| Building both policy configurations independently | `configured_settings` paired builder | forge median: 5,603,661 after for three paired builds | left below stop rule | Even removing half the entire builder, before paying for a replacement copy, estimates only `31.2 * 2.802m / 222.425m = 0.393` CPU s, 0.74% of the binary. The two configurations also differ when settings kinds are enabled. |
| Fake construction | fake domain and repository | forge median: 86,411 after for three pairs | left below stop rule | Less than 0.013 estimated CPU s even across all 31.2 module CPU s; giving the fake less changes no material floor. |

The unresolved larger boundaries above are recorded with the work they
require; the next local fixture saving, the paired builder, is below the
stop rule. A process-wide cache was not introduced. No test, seed, limit,
fixture row or assertion was removed or weakened.
