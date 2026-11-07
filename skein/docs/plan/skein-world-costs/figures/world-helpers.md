# world-helpers: the remaining shared world costs

2026-11-07 UTC, base `7fc50fc4ac`, 16 CPUs, AMD EPYC-Genoa Processor,
valgrind 3.24.0. Unoptimized test binaries, one exact named test per
Callgrind process, with its nextest cwd and `CARGO_MANIFEST_DIR`. Every
reading below ran and passed exactly one test. Totals include startup
and teardown. Inclusive rows overlap; they are not additive.

## The three tests on the task base

| Key | Package | Profile | Test | Base Ir |
|---|---|---|---|---:|
| T3 | `skein-forge-machine-world` | default | `hand::github::github_installation_loss::terminal_deferred_viewer_status_stops_after_connected` | 31,721,051 |
| T4 | `skein-llm-world` | default | `client::reuse_preserves_binding_and_rejects_other_authority` | 10,622,141 |
| T5 | `skein-tls-world` | default | `handshake::a_chain_longer_than_the_records_held_is_too_long` | 70,685,306 |

These names are the medians pinned by `counting.md`; no new selection run.
The base already contains Counting and Wire changes. The pristine-base
before arm remains the one in `counting.md`, not this task's base.

| Test | Cause, largest inclusive first | Inclusive Ir | Share | Scope and decision |
|---|---|---:|---:|---|
| T3 | `skein_tls::rs256::sign_rs256` | 22,580,773 | 71.19% | Product and its cryptography dependency; outside scope. |
| T3 | `Harness::drain` | 6,164,510 | 19.43% | Test-local fixture under `skein/tests/forge/machine/src/hand.rs`; outside scope; overlaps product calls. |
| T3 | loader `_dl_start` | 1,731,277 | 5.46% | Process startup; outside scope. |
| T3 | `Counting::alloc` | 1,301,811 | 4.10% | Prior task; retained accounting and library costs. |
| T3 | `Counting::dealloc` | 1,265,754 | 3.99% | Prior task; retained accounting and library costs. |
| T3 | `Wire::answer` | 408,028 | 1.29% | Prior task; inspect children for avoidable work, not inclusive wrapper cost alone. |
| T3 | `Trace::clear` | 229,890 | 0.72% | Shared helper, below 1% even if entirely removed; must still free entries. |
| T3 | `Wire::take` | 88,050 | 0.28% | Prior task; below threshold. |
| T4 | `World::run` | 6,341,707 | 59.70% | Shared wrapper; most is product execution, not removable helper work. |
| T4 | `client::resume` | 5,733,259 | 53.97% | Product; outside scope. |
| T4 | `collector::up` | 2,571,482 | 24.21% | Product JSON; outside scope. |
| T4 | `Client::prepare` | 2,371,343 | 22.32% | Product; outside scope. |
| T4 | JSON `string::decode` | 834,576 | 7.86% | Product string path, another plan's. |
| T4 | `Counting::alloc` | 814,642 | 7.67% | Prior task; retained accounting and library costs. |
| T4 | `Counting::dealloc` | 769,878 | 7.25% | Prior task; retained accounting and library costs. |
| T4 | loader `_dl_start` | 691,578 | 6.51% | Process startup; outside scope. |
| T4 | `text_response` | 308,759 | 2.91% | Shared fixture; contains `events` and `response` below. |
| T4 | `events`, formatted documents flattened byte by byte | 230,192 | 2.17% | In scope; slice assembly can remove most of this. Formatting alone is 37,322 Ir (0.35%); collection is the larger cause. |
| T4 | `World::next_turn` | 181,051 | 1.70% | Shared helper; necessary transport draws, intake, state selection; inspect after the larger fix. |
| T4 | `World::take` | 81,471 | 0.77% | Shared helper; below threshold. |
| T4 | `response` | 76,699 | 0.72% | Shared fixture framing; below threshold. |
| T5 | `Pair::settle` | 62,297,304 | 88.13% | Shared wrapper over both TLS implementations; not removable helper overhead. |
| T5 | `Server::receive` | 21,968,215 | 31.08% | Shared wrapper; 21,577,873 Ir (30.53%) is rustls `process_new_packets`, outside scope. |
| T5 | `Wire::answer` | 10,977,233 | 15.53% | In scope. |
| T5 | `Wire::answer` drain collection, 30 fills | 10,926,497 | 15.46% | In scope, largest cause; copy from queue slices instead. Subset of preceding row. |
| T5 | `pki::Server::config` | 1,391,687 | 1.97% | Shared wrapper around required rustls configuration and certificate setup; inspect after drain fix. |
| T5 | `Counting::alloc` | 726,353 | 1.03% | Prior task; retained accounting and library costs. |
| T5 | `Counting::dealloc` | 658,987 | 0.93% | Prior task; retained accounting and library costs. |
| T5 | `Wire::pull` | 354,155 | 0.50% | Shared helper; below threshold. |

## Readings after fixes

### TLS fill slices

| Test | Arm | Total Ir | Cause | Inclusive Ir | Share |
|---|---|---:|---|---:|---:|
| T5 | base | 70,685,306 | `Wire::answer` | 10,977,233 | 15.53% |
| T5 | slice copies | 59,869,435 | `Wire::answer` | 165,190 | 0.28% |
| T5 | slice copies | 59,869,435 | two `extend_from_slice` calls per fill | 115,317 | 0.19% |
| T5 | slice copies | 59,869,435 | `pki::Server::config` | 1,391,703 | 2.32% |
| T5 | slice copies | 59,869,435 | rustls `with_single_cert`, within config | 1,339,747 | 2.24% |
| T5 | slice copies | 59,869,435 | product `client::up` | 28,934,026 | 48.33% |
| T5 | slice copies | 59,869,435 | peer rustls `process_new_packets` | 21,578,001 | 36.04% |

The total falls by 10,815,871 Ir (15.30%). The same 30 fills are delivered,
in byte order, and the queue drains the same prefix. No scheduling, RNG,
assertion or diagnostic changes. A wrapped-queue regression test passes;
omitting its second slice copy fails with `[6, 7]` instead of `[6, 7, 8, 9]`,
and passes after restoration. The helper's focused suite passes (one test),
as does the exact median TLS test.

The next TLS helper wrapper above 1% is server configuration. Of its
1,391,703 Ir, 1,339,747 Ir (2.24% of the test) is rustls certificate/key
validation. The remaining wrapper and other setup calls together are
51,956 Ir (0.09%). Retain independent configurations: caching them would
change their ownership/lifecycle and the setup exercised by the tests.
No removable helper cause reaches 1%; product and peer TLS now dominate.


### LLM literal event slices

| Test | Arm | Total Ir | Cause | Inclusive Ir | Share |
|---|---|---:|---|---:|---:|
| T4 | base | 10,622,141 | `events` | 230,192 | 2.17% |
| T4 | slice assembly | 10,419,593 | `events` | 25,923 | 0.25% |
| T4 | slice assembly | 10,419,593 | `text_response` | 104,518 | 1.00% |
| T4 | slice assembly | 10,419,593 | `World::next_turn` | 181,259 | 1.74% |

The total falls by 202,548 Ir (1.91%). The same documents, including empty,
UTF-8 and embedded-newline documents, retain the exact `data: ` prefix and
two-newline suffix. The regression test fails when the suffix loses one
newline, and passes after restoration; all four helper tests and the exact
median LLM test pass. The public signature and all callers stay unchanged.

`text_response` is the composition of the now 0.25% event builder and
0.74% response builder. `World::next_turn` spends 88,332 Ir (0.85%) in
`Intake::meet`, 37,972 Ir (0.36%) in `Client::has_work` and 24,281 Ir (0.23%)
in `Client::waiting`; each is product code. Its entire remainder is less
than 0.31%, including necessary intake append and RNG calls. There is no
removable 1% helper cause left here.

For T3, `Wire::answer`'s largest children are trace recording (99,474 Ir,
0.31%), slice extension (96,416 Ir, 0.30%) and product `Intake::meet`
(62,552 Ir, 0.20%). Even deleting the first two would save less than 1% and
would discard the required trace. `Trace::clear` is already below 1%.
RSA signing dominates this median, not a shared helper; no further forge
helper change follows. Counting's remaining costs belong to the earlier
task and are carried into the account by cause.
