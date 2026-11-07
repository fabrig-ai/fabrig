# close: temper fuzzy on the combined trunk

2026-11-07 UTC, code commit `182ef80c19a1a8eded699039c3f29d603b620c32`,
16 CPUs, AMD EPYC-Genoa Processor, Valgrind 3.24.0.
The workspace test profile and existing package overrides are unchanged.
No code, timeout, patience, seed, test membership or budget is changed.

## Method and commands

Build before the whole-suite samples:

```sh
heavy cargo nextest run --workspace --profile fuzzy -j 1 --no-run \
  -E 'package(/^temper(-|$)/) | package(=git-transcripts)'
```

The successful build took 3m 03s Cargo wall (12:05:10–12:08:15 UTC including
invocation overhead). An initial CLI attempt combined `--no-run` with
`--no-fail-fast`; nextest rejected those incompatible arguments before
building or running tests. The corrected build above succeeded.
The no-run command compiles tests; it supplies no test-pass evidence.

For native measurements, the `notes/measuring.md` RUSAGE_CHILDREN wrapper
runs inside heavy, immediately around Cargo/nextest. It records deltas of
child user/system usage after waiting for the child, monotonic wall,
start/end UTC and load at both ends. Waited-for descendants are included.
The wrapper's own Python CPU is excluded; Cargo/nextest CPU is included.
All commands have `TEMPER_BUNDLE=none` through the build configuration,
receive `</dev/null`, and run serially.

```sh
heavy bash -lc 'python3 <wrapper> cargo nextest run --workspace \
  --profile fuzzy -j 1 --no-fail-fast \
  -E "package(/^temper(-|$)/) | package(=git-transcripts)"'
heavy bash -lc 'python3 <wrapper> cargo nextest run -p <package> \
  --profile fuzzy -j 1 --no-fail-fast'
```

Run the workspace command three times without intervening builds. For each
of the five packages, warm its own selection first and then take three
samples. Package selection changes Cargo's feature union: all five initial
package invocations rebuilt dependencies, despite the workspace build.
They are recorded below as build-inclusive warm-ups, never as warm CPU
measurements. Logged package runs 2, 3 and 4 are the three warm samples;
every one has no `Compiling` line. All three workspace samples also have no
rebuild. No profile/build setting was changed to obtain that result.

The fuzzy profile's default filter still applies. The workspace expression
selects temper and `git-transcripts` only: **81 passing tests per sample**,
across 33 binaries, 3,401 skipped. Five package selections have respectively
18, 26, 15, 4 and 5 passing fuzzy tests, the same counts as the request.
No browser/process/live tier or product driver ran. Existing ignores stay.

The gate sums each test's user+system CPU (including descendants) at eight
at once. These samples use `-j 1` and include the driver; the request's
before table is the gate's. Package feature unions and independent medians
are another reason not to subtract package sums from workspace samples
as if they formed one run. These measurements establish no gate pass;
the queue's trunk end check is the first such record.

## Whole suite: three warm runs

| Run | UTC interval | Passed | User s | System s | CPU s | Wrapper wall s | Start load 1/5/15m | End load 1/5/15m |
|---|---|---|---|---|---|---|---|---|
| suite-1 | 12:08:35–12:10:44 | 81 | 111.400997 | 7.887302 | 119.288299 | 128.148764 | 33.85/37.66/27.79 | 12.67/28.43/25.70 |
| suite-2 | 12:10:44–12:12:57 | 81 | 118.554335 | 9.110021 | 127.664356 | 133.699390 | 12.67/28.43/25.70 | 16.88/23.85/24.31 |
| suite-3 | 12:12:58–12:15:15 | 81 | 115.794529 | 9.784503 | 125.579032 | 137.031430 | 16.88/23.85/24.31 | 38.06/27.21/25.28 |

All 243 test executions passed. The median by CPU is run 3:
125.579032 CPU s, 137.031430 wrapper wall s. CPU range
119.288299–127.664356, spread 8.376057 s (7.02% of the smaller sample).
Against 120, margins are +0.711701, −7.664356 and −5.579032 CPU s.
Two measured runs and the median exceed the limit: this suite does not
hold it. The request's roughly 15 s reduction from 125–128 is not achieved.
The gate's own result remains for the queue to record.

## Five worlds: three warm runs each

Run numbers 2/3/4 are the logged invocation numbers; warm-up 1 is below.
Each row has exit status 0. All 204 warm-package test executions passed.

| Run | UTC interval | Passed | User s | System s | CPU s | Wrapper wall s | Start load 1/5/15m | End load 1/5/15m |
|---|---|---|---|---|---|---|---|---|
| temper-real-loop-world-2 | 12:17:01–12:18:09 | 18 | 57.187021 | 3.865645 | 61.052666 | 68.410878 | 23.68/25.61/24.93 | 11.82/21.72/23.64 |
| temper-real-loop-world-3 | 12:18:09–12:19:21 | 18 | 61.690396 | 4.524192 | 66.214588 | 71.914400 | 11.82/21.72/23.64 | 7.64/18.51/22.38 |
| temper-real-loop-world-4 | 12:23:08–12:24:19 | 18 | 58.780295 | 4.633623 | 63.413918 | 71.324817 | 15.35/14.01/19.51 | 34.16/19.49/20.95 |
| temper-engine-domain-world-2 | 12:19:45–12:19:55 | 26 | 9.534881 | 0.768471 | 10.303352 | 10.254761 | 6.02/17.26/21.87 | 5.39/16.76/21.66 |
| temper-engine-domain-world-3 | 12:19:55–12:20:06 | 26 | 9.835998 | 0.801651 | 10.637649 | 10.567751 | 5.39/16.76/21.66 | 4.72/16.24/21.43 |
| temper-engine-domain-world-4 | 12:24:19–12:24:31 | 26 | 9.917193 | 0.742545 | 10.659738 | 11.244101 | 34.16/19.49/20.95 | 30.04/19.09/20.81 |
| temper-conformance-world-2 | 12:20:49–12:21:20 | 15 | 27.509940 | 2.893737 | 30.403677 | 30.435605 | 4.48/14.66/20.66 | 3.91/13.52/20.08 |
| temper-conformance-world-3 | 12:21:20–12:21:50 | 15 | 27.295681 | 2.873929 | 30.169610 | 30.089578 | 3.91/13.52/20.08 | 5.49/12.92/19.67 |
| temper-conformance-world-4 | 12:24:31–12:24:59 | 15 | 24.904402 | 2.972489 | 27.876891 | 28.495086 | 30.04/19.09/20.81 | 26.16/19.19/20.79 |
| temper-one-process-world-2 | 12:22:08–12:22:20 | 4 | 11.475432 | 0.324998 | 11.800430 | 11.813203 | 5.15/12.48/19.42 | 8.32/12.82/19.42 |
| temper-one-process-world-3 | 12:22:20–12:22:32 | 4 | 11.545989 | 0.352483 | 11.898472 | 12.253410 | 8.32/12.82/19.42 | 10.19/13.08/19.43 |
| temper-one-process-world-4 | 12:25:00–12:25:12 | 4 | 11.955841 | 0.447625 | 12.403466 | 12.730998 | 26.16/19.19/20.79 | 26.12/19.42/20.84 |
| temper-codecs-world-2 | 12:22:42–12:22:47 | 5 | 3.910931 | 0.248337 | 4.159268 | 4.267410 | 10.61/13.06/19.36 | 10.64/13.02/19.31 |
| temper-codecs-world-3 | 12:22:47–12:22:52 | 5 | 3.860471 | 0.324492 | 4.184963 | 4.792153 | 10.64/13.02/19.31 | 14.43/13.77/19.52 |
| temper-codecs-world-4 | 12:25:12–12:25:17 | 5 | 3.955482 | 0.303347 | 4.258829 | 4.729212 | 26.12/19.42/20.84 | 25.63/19.43/20.84 |

| Package | Tests each run | Median run | Median CPU s | Range CPU s |
|---|---|---|---|---|
| temper-real-loop-world | 18 | 4 | 63.413918 | 61.052666–66.214588 |
| temper-engine-domain-world | 26 | 3 | 10.637649 | 10.303352–10.659738 |
| temper-conformance-world | 15 | 3 | 30.169610 | 27.876891–30.403677 |
| temper-one-process-world | 4 | 3 | 11.898472 | 11.800430–12.403466 |
| temper-codecs-world | 5 | 3 | 4.184963 | 4.159268–4.258829 |

The five medians sum to 120.304612 CPU s for 68 tests. This is not a full
suite total. The full selection has 81 tests and its own single driver;
independent samples and Cargo feature unions prevent a residual-cost claim.

## Build-inclusive warm-ups, excluded from the medians

All five first package invocations contain `Compiling` output and passed.
They supplied build warmth, not warm native timing. Each subsequent
package sample has no rebuild. These 68 passing warm-up executions are
reported separately; none replaces one of the fifteen warm samples.

| Run | UTC interval | Passed | User s | System s | CPU s | Wrapper wall s | Start load 1/5/15m | End load 1/5/15m |
|---|---|---|---|---|---|---|---|---|
| temper-real-loop-world-1 | 12:15:15–12:17:00 | 18 | 174.600415 | 18.113415 | 192.713830 | 105.186916 | 38.06/27.21/25.28 | 23.68/25.61/24.93 |
| temper-engine-domain-world-1 | 12:19:21–12:19:45 | 26 | 31.578818 | 3.301586 | 34.880404 | 23.435263 | 7.64/18.51/22.38 | 6.02/17.26/21.87 |
| temper-conformance-world-1 | 12:20:06–12:20:49 | 15 | 75.304053 | 9.041449 | 84.345502 | 42.995293 | 4.72/16.24/21.43 | 4.48/14.66/20.66 |
| temper-one-process-world-1 | 12:21:50–12:22:08 | 4 | 21.860405 | 2.437235 | 24.297640 | 17.862123 | 5.49/12.92/19.67 | 5.15/12.48/19.42 |
| temper-codecs-world-1 | 12:22:33–12:22:42 | 5 | 16.792385 | 2.405890 | 19.198275 | 9.661491 | 10.19/13.08/19.43 | 10.61/13.06/19.36 |

## Refreshed sweeps after shared changes

E and O measured these paths before U changed shared list append bookkeeping
and generated fixed-width measurement. These thirteen exact tests are
therefore reprofiled on the combined trunk, retaining E/O's original before
counts for the account. Public-wire is not reprofiled: U already supplies
its final shared-code arm. No other sweep is profiled merely to obtain a
different native sample.

For each package, obtain the suite binary and cwd with:

```sh
heavy cargo nextest list -p <package> --test suite --message-format json
heavy bash -lc 'cd <cwd> && CARGO_MANIFEST_DIR=<cwd> valgrind \
  --tool=callgrind \
  --callgrind-out-file=<slot>/target/limits-fuzzy/close/<label>.%p.out \
  <binary> --exact <test> --nocapture'
callgrind_annotate --inclusive=yes --auto=no --threshold=100 <profile>
callgrind_annotate --inclusive=no --auto=no --threshold=100 <profile>
```

Annotations also run through heavy in this measurement. Each profile has
one process/file and finishes with `1 passed; 0 failed`. Its Ir is that
file's `PROGRAM TOTALS` (the raw `summary`); no child totals or timer
normalization apply. Then run the same binary/cwd/exact name natively,
through heavy, with CARGO_MANIFEST_DIR set and RUSAGE_CHILDREN around the
waited-for binary alone. These direct native values exclude Cargo/nextest,
unlike the suite/world tables. All thirteen native executions also finish
with `1 passed; 0 failed`. There are 26 passing test executions in this
refresh and no partial profile. Job-end loads are after annotation, not an
assertion of idle CPU at the native sample.

| Label / exact sweep | Draws, same as before | Arm / run | Ir | Native user + system s | Native CPU s | Native wall s | Job-end load 1/5/15m | Result |
|---|---|---|---|---|---|---|---|---|
| walking-0 / `fuzzy::walking::fuzzy_walking_shard_0` | 8 seeds, 2 worlds each | combined / 1 | 1,586,213,658 | 0.502369 + 0.004019 | 0.506388 | 0.507038 | 21.63/19.13/20.69 | 1 passed profile; 1 passed native |
| walking-1 / `fuzzy::walking::fuzzy_walking_shard_1` | 8 seeds, 2 worlds each | combined / 1 | 1,669,610,875 | 0.533794 + 0.019918 | 0.553712 | 0.554782 | 15.39/17.89/20.23 | 1 passed profile; 1 passed native |
| walking-2 / `fuzzy::walking::fuzzy_walking_shard_2` | 8 seeds, 2 worlds each | combined / 1 | 1,641,449,784 | 0.522982 + 0.015969 | 0.538951 | 0.539461 | 11.22/16.71/19.77 | 1 passed profile; 1 passed native |
| walking-3 / `fuzzy::walking::fuzzy_walking_shard_3` | 8 seeds, 2 worlds each | combined / 1 | 1,555,988,312 | 0.545949 + 0.016058 | 0.562007 | 0.562466 | 9.01/15.65/19.32 | 1 passed profile; 1 passed native |
| walking-4 / `fuzzy::walking::fuzzy_walking_shard_4` | 8 seeds, 2 worlds each | combined / 1 | 1,702,478,232 | 0.539980 + 0.023823 | 0.563803 | 0.564160 | 6.70/14.57/18.87 | 1 passed profile; 1 passed native |
| walking-5 / `fuzzy::walking::fuzzy_walking_shard_5` | 8 seeds, 2 worlds each | combined / 1 | 1,581,022,596 | 0.540299 + 0.008004 | 0.548303 | 0.549640 | 5.37/13.50/18.37 | 1 passed profile; 1 passed native |
| walking-6 / `fuzzy::walking::fuzzy_walking_shard_6` | 8 seeds, 2 worlds each | combined / 1 | 1,498,920,321 | 0.488650 + 0.012115 | 0.500765 | 0.509993 | 7.36/13.16/18.10 | 1 passed profile; 1 passed native |
| walking-7 / `fuzzy::walking::fuzzy_walking_shard_7` | 8 seeds, 2 worlds each | combined / 1 | 1,774,405,927 | 0.629660 + 0.016043 | 0.645703 | 0.650296 | 11.19/13.53/18.09 | 1 passed profile; 1 passed native |
| terminal / `fuzzy::walking::fuzzy_terminal_commit_cut_preserves_complete_replay_and_funding` | 4 seeds, 2 worlds each | combined / 1 | 1,161,519,574 | 0.385997 + 0.036187 | 0.422184 | 0.529671 | 21.10/15.71/18.67 | 1 passed profile; 1 passed native |
| escalation / `fuzzy::escalation::fifteen_small_escalation_histories_cover_every_story_and_durable_cut` | 15 histories, 2 worlds each | combined / 1 | 4,445,836,527 | 1.342174 + 0.024110 | 1.366284 | 1.441451 | 18.87/16.29/18.71 | 1 passed profile; 1 passed native |
| roles / `fuzzy::roles::twelve_small_role_histories_cover_each_holder_owner_order_and_atomic_cut` | 12 histories, 2 worlds each | combined / 1 | 2,538,271,845 | 0.654554 + 0.020079 | 0.674633 | 0.675263 | 11.57/14.75/18.11 | 1 passed profile; 1 passed native |
| kill / `fuzzy::one_process::drawn_kills_keep_the_conformance_rules_and_answer_adoptions_once` | 4 crash seeds; 4 cold readbacks | combined / 1 | 11,265,017,048 | 4.210288 + 0.016008 | 4.226296 | 4.226874 | 5.02/10.76/16.15 | 1 passed profile; 1 passed native |
| power / `fuzzy::one_process::drawn_power_losses_keep_the_conformance_rules` | 4 crash seeds; 4 cold readbacks | combined / 1 | 10,994,220,374 | 4.479941 + 0.027975 | 4.507916 | 4.566250 | 21.16/14.14/16.58 | 1 passed profile; 1 passed native |

## Largest retained work by inclusive Ir

Test/runtime, replay/world-loop and generic dispatch envelopes are omitted.
The table retains a large work function per sweep: same_digest for walking,
terminal and escalation, role-world construction for roles, fake-person
iteration for kill, and service iteration for power. These are the largest
work families in the task profiles, not a partition of the execution.
Calls are summed incoming raw Callgrind edges for the displayed function.
Nested inclusive shares overlap and must not be added.

| Sweep label | Share | Ir | Calls | Function |
|---|---|---|---|---|
| walking-0 | 47.03% | 746,019,634 | 1,382 | `temper_engine_domain_world::replay::same_digest` |
| walking-1 | 47.84% | 798,766,958 | 1,464 | `temper_engine_domain_world::replay::same_digest` |
| walking-2 | 48.24% | 791,881,762 | 1,466 | `temper_engine_domain_world::replay::same_digest` |
| walking-3 | 45.87% | 713,661,509 | 1,315 | `temper_engine_domain_world::replay::same_digest` |
| walking-4 | 48.60% | 827,464,441 | 1,521 | `temper_engine_domain_world::replay::same_digest` |
| walking-5 | 46.56% | 736,103,792 | 1,366 | `temper_engine_domain_world::replay::same_digest` |
| walking-6 | 45.37% | 680,068,054 | 1,268 | `temper_engine_domain_world::replay::same_digest` |
| walking-7 | 49.21% | 873,228,662 | 1,592 | `temper_engine_domain_world::replay::same_digest` |
| terminal | 49.05% | 569,723,794 | 866 | `temper_engine_domain_world::replay::same_digest` |
| escalation | 54.77% | 2,435,054,755 | 3,379 | `temper_engine_domain_world::replay::same_digest` |
| roles | 38.18% | 969,087,557 | 24 | `temper_engine_domain_world::roles::World::new` |
| kill | 28.47% | 3,207,395,783 | 22,747 | `<temper_one_process_world::world::Person as skein_shell::drive::Host>::iterate` |
| power | 23.10% | 2,540,114,735 | 25,966 | `temper_service::system::iterate` |

## Largest functions by self Ir

| Sweep label | Share | Ir | Function |
|---|---|---|---|
| walking-0 | 7.15% | 113,335,690 | `__memcpy_avx_unaligned_erms` |
| walking-1 | 7.02% | 117,198,666 | `__memcpy_avx_unaligned_erms` |
| walking-2 | 7.04% | 115,500,920 | `__memcpy_avx_unaligned_erms` |
| walking-3 | 7.22% | 112,407,480 | `__memcpy_avx_unaligned_erms` |
| walking-4 | 6.95% | 118,361,294 | `__memcpy_avx_unaligned_erms` |
| walking-5 | 7.17% | 113,337,756 | `__memcpy_avx_unaligned_erms` |
| walking-6 | 7.32% | 109,761,476 | `__memcpy_avx_unaligned_erms` |
| walking-7 | 6.86% | 121,745,092 | `__memcpy_avx_unaligned_erms` |
| terminal | 6.32% | 73,464,984 | `<temper_engine_domain_world::replay::WordHasher as core::hash::Hasher>::write_u64` |
| escalation | 7.01% | 311,752,392 | `<temper_engine_domain_world::replay::WordHasher as core::hash::Hasher>::write_u64` |
| roles | 6.99% | 177,416,281 | `__memcpy_avx_unaligned_erms` |
| kill | 8.64% | 973,016,522 | `__memcpy_avx_unaligned_erms` |
| power | 9.06% | 996,556,931 | `__memcpy_avx_unaligned_erms` |

The refreshed engine subtotal is **21,155,717,651 Ir** and the two
crash sweeps total **22,259,237,422 Ir**. These are closing counts, not
isolated shared-fix experiments. Their small differences from E/O's
post-task figures receive no separate native speedup claim.

## Causes, largest first

The fixed and retained causes, their source arms, shares and savings are
reconciled in [the account](../../../development/limits-fuzzy.md).
This closing task fixes none. E's expensive repeated formatting is replaced
by complete Hash traversal plus final text; O retains independent sessions,
all crash phases and cold readbacks; U already pays the redundant list and
fixed-record measurement causes. Required full-state comparison, product
steps, validation, actual growth and complete secret checks are not inferred
removable from their inclusive footprints.

## Left

- Whole-suite median **125.579032 CPU s**, **5.579032 s above 120**;
  observed range **119.288299–127.664356**. No gate entry or budget change
  is made. The account names the next investigation and the user's decisions.
- U's partial real-loop profiles and intermittent raised-patience P failure
  remain. Closing ordinary-patience suite/package runs all passed, but they
  do not explain or repair that earlier reliability observation.
- Focused totals and scripted real-loop placement are outside this account's
  fuzzy-cost measurements; their status sentences remain.
- The gate's per-test, eight-at-once result is unobserved here; the queue
  records it in the trunk end check. No browser/process/live story ran.

## Stopped at

The scope is measurement and account, not another cause repair. The account
carries every predecessor Left/Stopped-at item, including the sub-2% local
ceilings, required work and representation/design boundaries. No seed,
invariant, ending, timeout, allocator contract or budget is weakened.
Raw profiles in target/limits-fuzzy/ and any temporary measuring code are
removed after the figures are written, before push.
