# setup-steps: figures

Base `76453cdc2bffd377992317f45c083aab001743ff`, machine
`debian-32gb-hel1-11` through the build wrapper, 2026-11-07 UTC.
Before profiles were taken before editing; after profiles use the final
code in this commit. Each row ran alone under Callgrind with nextest's
double spawn disabled and the default profile. Inclusive and self
annotations were read; the fixture split also uses the calling tree.
Counts include the loader, harness, assertions and teardown.

The pristine `setup::` run passed 238 tests. Its elapsed ordering selected
F and M; elapsed times are not performance figures here. F also remains
the floor from `figures/service-fixture.md`. P retains that document's
median by name, beside this run's selected median M. R drives conversion
requests and replies through `Face` and checks every durable write.

| Row | Test | Floor or median | Ir before | Ir after | What was built before the test's first assertion |
|---|---|---|---:|---:|---|
| F: setup | `setup::tests::setup_reasons_fit_the_wire_at_a_character_boundary` | Floor | 11,549,549 | 11,579,609 | A 257-byte writer and boundary strings; no service fixture. |
| M: setup | `setup::tests::a_kept_github_record_serves_its_routes` | Median | 90,073,677 | 90,103,346 | A completed service, retained records, two keeps of the runtime GitHub forge and its local manifest. The test checks the kept forge's three page routes. |
| R: setup request | `setup::tests::written_is_answered_only_after_kept_and_the_routing_rows_follow_each_keep` | Request through Face | 185,270,953 | 126,019,296 | The same service fixture, a manifest, conversion replies, App-key decoding and signed GitHub requests. Each AppId/AppSlug/AppKey/User keep remains checked before Origin and the terminal. |
| P: setup comparison | `setup::tests::an_unresolved_forge_keeps_its_setup_entry_but_restored_and_asked_connects_never_dial` | service-fixture's median | 135,875,793 | 135,907,241 | Configured settings and endpoints, an unresolved GitHub entry beside a resolved Forgejo entry, and a configured service. A deliberately independent records-store replacement follows the first assertions. |

## What follows the fixture

The fixture column is inclusive cost at the test's fixture call, not a
sum of overlapping descendants. For P, the calling tree separates the
top-level `configured_engine`, `configured_service` and later
`records_store` calls; the store made inside `configured_service` is not
counted twice. Everything else, including process startup, is in the
last two columns.

| Row | Fixture Ir before | Fixture Ir after | Ir beyond fixture before | Ir beyond fixture after |
|---|---:|---:|---:|---:|
| F: none | 0 | 0 | 11,549,549 | 11,579,609 |
| M: `service` | 75,938,237 | 75,938,135 | 14,135,440 | 14,165,211 |
| R: `service` | 75,936,259 | 75,936,063 | 109,334,694 | 50,083,233 |
| P: the three top-level fixture calls | 122,211,705 | 122,211,511 | 13,664,088 | 13,695,730 |

R saves **59,251,657 Ir (31.98%)** overall, and **54.19%** beyond its
fixture. M still spends 84.28% in `service`; its local manifest and kept
forge checks do not use Face. F's loader relocation alone is 90.09% of
its final count. Relocation rises from 10,400,272 to 10,432,470 Ir with
the larger test binary, explaining the approximately 30,000-instruction
increase in the rows with no changed execution path.

The historical 77 ms median belongs to the request's earlier suite,
not this run's elapsed ordering. Keeping its comparison test P, the
original 330,036,179 Ir in `service-fixture.md` becomes 135,907,241 Ir:
41.18%, or **31.71 ms at the old instruction rate**. This task adds no
saving on P; the earlier fixture work accounts for that reduction. M
is likewise effectively unchanged here. These are instruction ratios,
not a new measured CPU median for all 239 setup tests.

## Causes

The stop threshold is about **0.463 CPU s**, 1% of the request's 46.3 CPU s
for this binary. Inclusive entries overlap and must not be added.
Retained fixture costs above the threshold have the construction change
they would need listed explicitly.

| Cause | Where | Ir per test, on which test | Fixed in | Or what fixing it would take |
|---|---|---:|---|---|
| Fixed turns after a request has settled | `Face::sent`, `reply_status` and matching raw-reply helpers | `Face::turn`: 83,514,988 → 23,767,492, R; whole-test `advance`: 72,244,776 → 14,647,425; lifecycle `advance`: 47,500,853 → 1,810,203 | `tests/setup.rs`, `setup_lifecycle.rs`, `setup_reachable.rs`, `setup_scopes.rs`; scheduling query in `skein/testing/skein-world/src/stream.rs` | Drain setup io and above, lifecycle readiness, wire delays and deadlines due now. Future timers and reads awaiting peer input settle. The fake server also stops once Closed has arrived and its output queues are empty. |
| A completed service's state and store | `service`, `fixtures::ready` | `service`: 75,938,135; `ready`: 52,895,163, M after | Retained; `tests/fixtures.rs` unchanged | Avoiding the real start needs an equivalent complete first-start record set, settings, counters, durable mark and domain state. A mark alone is insufficient. P intentionally needs a second independent store. |
| Cold engine decoding and validation | `fixtures::parse_engine` | 16,090,161, M after | Retained | A generated typed stock configuration or a parser/validator improvement preserving the full checked value. The existing per-process cache cannot remove its cold call. |
| Repeated admission across necessary constructors | `Service::prepared` | 14,989,265, M after; 22,504,529, P after | Retained | Share checked construction results across root/component constructors while preserving each admission/refusal check. This is a construction-interface change, rather than another duplicate fixture call. |
| Hosted-store construction and secret exchanges | `memory_store_for`, `seed_forge_rows` | 7,064,836 and 5,677,577, M after | Retained | An in-memory hosted-store constructor over supplied maps, preserving driver state, wires, durable startup rules and secret exchange behaviour. |
| Required GitHub signing | `skein_tls::rs256::sign_rs256` | 22,611,818 → 22,611,818, R | Retained | Cryptographic implementation work, or a separately justified fixture seam retaining wire authentication. A canned token would omit the exercised signing path. No build settings change. |
| Loader relocation | `_dl_relocate_object` | 10,400,272 → 10,432,470, F | Retained | Process startup and binary relocation; linker/build changes are outside this plan. |
| Remaining Face settling mechanics after signing | `Face::settle` less signing | 1,717,026, R after | Retained; stop rule | Even charging that entire share across all 24.9 baseline setup CPU s estimates 0.339 CPU s. It includes necessary framing, delivery and routing. |
| Connection-only fixed turns | `setup_lifecycle.rs::connection_turn` and its callers in `setup_github.rs` | 16,219,412 inclusive, on `setup::tests::lifecycle_tests::calls_are_not_connected_before_connect_then_wait_until_connected`, after the shared Face change | Retained; stop rule | Five tests reach these helpers. Charging the entire measured call, including required signing, at five of 238 tests' module-average exposure estimates 0.060 CPU s. A connection settler would need to retain `forge.above` for those tests' assertions. |
| Short local advancement and manifest encoding | `terminal`, `load_routing`, manifest rendering | All `advance`: 763,060; manifest encoding: 337,246, M after | Retained; stop rule | Removing all of either sampled share across the full 24.9 setup CPU s estimates 0.211 and 0.093 CPU s respectively. Encoding is what the page test checks, and was not duplicated. |

## Settlement and verification

The shared Face bound is 4,096 turns, the picker's existing full-page
ceiling. Its full page did not drain inside 1,024 turns; the bounded
helper now completes that response before returning. Exhaustion panics
with queue/readiness/deadline facts. The clock, product deadlines,
budgets, wire bytes, seeds and every existing assertion are unchanged.

The four disk loops in `setup_admin.rs` and the 4,096-turn root restore
loop already break on their completion conditions and are unchanged.
The connection-only 1,024-turn loops, and the matching 256-turn GitHub
connection loops, remain under the stop rule above. The short local
passes and scenario loops that advance simulated time also remain.
No count-dependent scenario loop was converted. The body of
`setup_malformed_queries_are_bad_requests` is unchanged.

The pristine service run passed 1,115 focused tests. The final run
passes **1,116 focused tests**, including **239 setup tests**, plus its
one fuzzy test. The new wire check also passes; the two changed packages
and their 113 direct dependent packages pass **13,920 focused tests**
combined. All direct dependent packages pass `cargo check --tests`.
Clippy passes for both changed packages; formatting, both trees'
old-form ratchets and citations pass without baseline changes. Slow
tiers were compiled where applicable and were not run.

Both new checks were deliberately broken and restored. Ignoring pending
wire fragments made the wire check fail at `delivered`; omitting setup
advancement made the Face check exhaust its bound. The restored tests
pass, including delayed room, partial reads, EOF, due pages, future
pages, refused replies and full socket closure.
