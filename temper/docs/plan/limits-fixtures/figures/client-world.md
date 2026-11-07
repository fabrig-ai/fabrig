# client-world: figures

Base `ed1e8e03f4749b719abe98924a12b6605307ca98`, machine
`debian-32gb-hel1-11` (through the build wrapper), 2026-11-07 UTC.
After figures use the final source changes committed with this file.

| Row | Test | Floor or median | Ir before | Ir after | What was built before the first assertion |
|---|---|---|---:|---:|---|
| `temper-client-world::suite` | `forges_edges::forge_queries_canonicalize_good_bad_and_repeated_values` | Cheapest | 8,782,754 | 8,792,320 | Process and harness, then a formatted forge address and its parsed choice. No Joint, store, phrasebook, World or View. |
| Same binary | `dismissed::a_dismissal_outlives_a_restart` | Median | 157,649,730 | 74,917,043 | Two connector configurations, memory store, real first start and account/forge rows, cold engine, fake forge, worker hello, transport/component and durable Truth; two bound person roots and their startup watches, then a dismissal and its answer. Before: two full terminal Views, both discarded, including English construction, book clones and capacity checks. After: the same unlinked roots alone. The restart and third root come after the first dismissal assertions. |
| Same binary, additional joined-world probe | `recovery::seed_five_recovers_online_after_the_four_faults` | Pinned seed, not the selected median | 255,239,007 | 255,248,497 | Joint and real bootstrap/cold engine as above; World with the overridden complete decision language and referee; one unlinked person root, complete terminal English View and bindings; random person, four scheduled faults and the completed recovery run. |

The unchanged focused selection passed all 385 suite tests; sorting PASS
durations, with test names breaking ties, selected the first and the 193rd
of 385. Durations only selected names and are not performance evidence.
The request's older suite count is 384. The additional recovery profile
covers World construction, which the selected median does not call.
Every before profile precedes every edit. Profiles run one exact name with
nextest's default-profile Callgrind tracer and double spawn disabled;
inclusive and exclusive annotations were read with `callgrind_annotate`.
Ir includes loading and teardown. The setup column names the first story
assertion's inputs; fixture helpers also assert during bootstrap and delivery.

The median saves **82,732,687 Ir (52.48%)**. Its three calls to
`dismissed::probe` fall from 85,072,279 to 2,364,746 Ir. The fix takes the
existing `temper_client_domain::unlinked` constructor directly wherever
that setup discarded `Temper::new`'s View. It covers the dismissal probe,
two chat setups, the configuring-chat and adoption setups, and two front
unit tests. These are setup-expression changes alone, with the same limits
and seeds. Drawing stories keep their full View and all existing assertions.
Fixture names and signatures, limits, rows, phrasebook values and bytes stay
unchanged. No production code changes.

## Where the instructions go

These top-level partitions are disjoint. The median's residual includes
the restart, third probe, story and teardown. The recovery run includes
the story's actual rendering and faults, rather than fixture startup alone.

| Test | Partition | Ir before | Ir after |
|---|---|---:|---:|
| Median | Loader, `_dl_start` | 8,150,501 | 8,157,947 |
| Median | `Story::new`, including its two probes | 94,337,664 | 30,664,789 |
| Median | Story, restart, teardown and harness residual | 55,161,565 | 36,094,307 |
| Recovery | Loader, `_dl_start` | 8,150,465 | 8,157,911 |
| Recovery | `fixtures::world` | 13,761,533 | 13,761,543 |
| Recovery | `World::client`, including App and View construction | 44,163,037 | 44,163,031 |
| Recovery | `World::run` | 178,750,573 | 178,750,427 |
| Recovery | Teardown, fault scheduling and harness residual | 10,413,399 | 10,415,585 |

The nested annotations below overlap: first start contains a cold root;
Word construction contains loading; App construction contains the book,
limits and View; rendering contains diffing. They must not be added.
F is the floor, M the median, R recovery.

## Causes

| Cause | Where | Ir per test, on which test | Fixed in | Or what fixing it would take |
|---|---|---:|---|---|
| Building and immediately discarding Views | App construction in root-only setups | M: App 77,544,952 before, absent after; probes 85,072,279 → 2,364,746 | The seven setup expressions above | Use the existing root constructor with the same limits and seed. No new fixture or product interface is needed. |
| Loader relocations | `_dl_relocate_object` | F: 8,042,133 → 8,049,579 | Retained | Binary/linker work; fixture changes cannot avoid it, and build settings are closed. |
| Complete English construction and loading | `Words::english`, `Phrasebook::load` | R after: 25,508,487; phrasebook loads 17,849,975 including the referee's separate book | Removed for root-only callers; retained for drawing | Further savings need changes to vocabulary construction/loading representation. The complete book is consumed by View; this task excludes its words. The view-words figures describe the remaining ordering/construction work. |
| Owned nested phrasebook clone and teardown | `Words::clone` | M: 19,468,240 before, absent after; R: 6,540,983 after | Removed for root-only callers | A drawing View owns Words. Removing its clone needs a different ownership/loaded-book representation, or caller lifetime changes. Dropping the cache would rebuild English for repeat clients. |
| Repeated capacity and presence scans | `Limits::of`, `limits::valid`, `Words::capacity` | M before: Limits 20,344,900, validity 20,323,586; all absent after. R after: Limits 6,781,620, Words capacity 10,160,091 | Removed for root-only callers | A trusted summary or checked constructor must still detect missing vocabulary and overflow after public book/connector mutation. Simply caching by domain limits would accept changed Words. This needs the words representation/API work recorded by view-words. |
| Real first start, followed by cold restore | `restored_forges`, `first_start_over`, `Driver::cold` | M: restored 3,350,774 → 3,350,770; R: restored 3,429,101, first start 3,132,782 after; cold constructors 1,517,596 before | Retained | An independent writer for configuration-dependent policy, project, settings and start rows, checked against the real two-commit start. These worlds vary their configurations. Keep the real start and its oracle; do not substitute a few constant rows. |
| Store sizing and the replacement store during a story's restart | `Store::with_limits`, `Driver::take_store` | R: stores 5,044,751 → 5,044,655 over construction and restart; take_store 2,542,239 after. M: stores 5,045,040 after | Bootstrap transfer already fixed by domain-fixture | Eliminating the restart replacement needs a distinct consuming cut path that preserves the retained driver's destructor/oracle contract. The reusable public take_store still leaves a usable replacement. Remaining store sizing needs shared checked sizing/codec interfaces, as domain-fixture records. |
| Actual engine stepping and readback | `Joint::pump_component`, `Driver::advance`, `Store::page_with_texts` | M after: pump 20,830,567, advance 19,606,484, pages 6,841,271; R: advance 27,384,932, pages 6,377,740 | Retained | These deliver the real engine's answers, watches and restart readback. Reducing them needs page/record projection and stepping changes with their own lower-tier stories; skipping deliveries would weaken this world. |
| Teardown and restart oracle | driver close/oracle | M: 7,261,031 after; R: 6,566,600 after | Retained | Preserve the independent restart and final-store comparison. A cheaper oracle needs changes to its snapshots, paging and retained configuration, beyond constructing the client fixture. |
| Semantic drawing and tree diffs during the story | client view build, skein UI diff | R: build 70,550,502, diff 36,960,027, match_nodes 20,422,823, parents 9,939,548, old_match 4,969,147, moves 6,549,846; unchanged | Retained | A different diff traversal/index must preserve sibling matching by key/role, stable identities and exact patch behavior, with skein's diff tests. These are repeated real drawings, not an unused View or fixed-iteration startup loop. |
| Applying and independently checking every drawn tree | world build, model apply/equality | R after: world build 106,345,381 including the view above; model check 8,512,729; model apply entry points 5,049,215 and 5,036,207 | Retained | A new incremental model/checking representation, preserving the independent patch oracle. Removing checks or callback builds loses what the world checks. |
| Referee vocabulary and default words discarded by Settings update | `World::new`, `decision_words`, `Settings::default` | R: 1,684,853; 775,331; 301,609, all unchanged | Below stop rule individually | Even deleting World construction entirely estimates only `24.3 * 1.685 / 255.239 = 0.160` CPU s. A constructor accepting words could remove the unused default language, about 0.029 estimated CPU s. A new interface is not justified here. |
| Component, Truth and fake forge startup | `Composition::new`, `core_rows`, `Truth::after`, `ForgeStand::with` | R after: 495,875; 366,864; 11,211; 95,150 | Below stop rule | At most about 0.047, 0.035, 0.0011 and 0.0091 estimated CPU s respectively across the whole request row. Lazy copies would add state without a material saving. |
| Engine limits and paired walking configurations | `engine_limits`, `walking::config` | R before: 4,163; 99,397 across initial construction and restart | Below stop rule | Less than 0.010 estimated CPU s for even the entire configuration boundary. |

The request's 24.3 CPU s gives a stop threshold of about **0.243 CPU s**.
The estimates above conservatively assign a cause the whole binary's CPU
total; actual reach is smaller. They are estimates from instruction shares,
not measured CPU savings. Every larger retained boundary has its required
repair above. The next local startup candidates are below the threshold;
the larger remaining work changes representations, store/oracle mechanics
or the rendering algorithm. `World::run` already stops on `Until::Done`;
4096 remains a bound, rather than a required number of iterations.

The floor is predominantly the loader: `_dl_start` is **92.80% before and
92.78% after**, and relocations alone **91.57% and 91.55%**. Non-loader
instructions are 632,261 and 634,381, including the address assertions.
Its 9,566 Ir increase is principally relocation growth, not new fixture work.

## Savings already present at this base

Both prerequisite implementations are already on the base. The client
profiles therefore measure their remaining cost, not a client binary
without them. The exact aggregate client-world saving from those earlier
changes is **unknown**: their figures use different test names and inputs.

The view-words figures measure English at 32,916,144 → 25,558,177 Ir,
a 7,357,967 Ir reduction per construction in that probe, plus cheaper
capacity lookups and maximum scans. R now constructs English once at
25,508,487 Ir; M constructed it once at 25,496,499 before this task and
does not construct it after. Their constructor sizes differ, so subtracting
these client counts from the older probe would not be a before/after figure.
The upstream words-only test saves 7,325,753 Ir and its two-View median
saves 18,844,254 Ir. These improvements already reach the clients that draw.

The domain-fixture figures measure a remaining store constructor at about
3.444m → 2.576m Ir and remove the bootstrap transfer's replacement store
(about 3.463m Ir in its root probe). R uses that consuming bootstrap path
once: its real first start is now 3,132,782 Ir. It still uses the reusable
take_store for the story's later restart. The upstream approximately 4.3m
Ir combined reduction for a bootstrap plus its remaining store is a scale
comparison, not a measured delta for R. No additional first-start shortcut
was introduced here.

## Verification

The final focused run of this package and its only direct dependent,
`temper`, passes **888 tests**: 397 in temper-client-world (385 suite,
eight library, four live-text domain stories) and 491 in temper. The client
fuzzy profile passes **two tests**, including its eight random-person seeds.
No browser, process or live profile was run. Existing ignored tests remain
ignored; no assertion, case, seed, product timeout or budget changed.

`fixtures::tests::seed_five_replays_with_cold_then_warm_phrasebooks` compares
the complete nonempty trace, referee counts, clock, iterations, callbacks,
faults and drops for two seed-five runs. Changing the second seed to six
fails the trace equality assertion; restoring five passes. This also checks
that warming the phrasebook cache does not change the run. The changed
root-only callers pass their focused stories. Formatting, temper's old-form
ratchet and package clippy pass; counters need no baseline update.
