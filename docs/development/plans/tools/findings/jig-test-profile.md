# jig's focused suite: the profile behind session 04

2026-10-18, debian-16gb-hel1-1, 8 vCPU AMD EPYC Genoa, Linux 6.12.107+deb13-cloud-amd64.
The interrupted run's detached `tmp/jig/tools-profile`, at `f50c3f63`, supplied the serial run and CPU samples.
Reused them: `git -C jig diff --stat f50c3f63 b2d920e4` shows only `docs/design/STATUS.md` (one row).
Advanced that scratch checkout to `origin/main`, `b2d920e4`, for the three closing baseline runs below.
Its pins are skein `0f093bb7`, smith `29fa97c9`; the planned repin is skein `b4c6241`, smith `a277e922`.
No source or build setting was changed. The scratch worktree was removed after recording these findings.

## 1. Commands and measured times

Every cargo command ran; builds in its build lane, measurements in its gate lane.
`scripts/check.sh`'s focused figure is nextest's `Summary` for `cargo nextest run --workspace`, excluding compilation.
The default timeout remains 30 s under ruling 275. The existing `measure` profile allows the serial diagnostic to exceed it.

```text
$ cargo nextest run --workspace --no-run
Finished `test` profile [unoptimized + debuginfo] ... in 1m 37s
$ cargo nextest run --workspace --profile measure -j 1
Summary [  59.855s] 4270 tests run: 4270 passed, 0 skipped
$ cargo nextest run --workspace
Summary [   5.146s] 4270 tests run: 4270 passed, 0 skipped   # interrupted run, f50c3f63
$ cargo nextest run --workspace   # three runs at b2d920e4
Summary [   7.426s] 4270 tests run: 4270 passed (1 slow), 0 skipped
Summary [   5.447s] 4270 tests run: 4270 passed, 0 skipped
Summary [   5.546s] 4270 tests run: 4270 passed, 0 skipped
```

The last three have median **5.546 s**. Logs in workspace `tmp/logs/`: `20261018T230837-3796.log` (serial), `20261018T231137-3798.log`,
`20261018T232723-3795.log`, `20261018T232758-3795.log`, `20261018T232840-3795.log` (default).
Sum the serial log's rounded per-test times, retaining spaces before early test counters:

```sh
$ python3 - /home/free/src/rust/tmp/logs/20261018T230837-3796.log <<'PY'
import re, sys
from collections import defaultdict
binaries = defaultdict(list)
for line in open(sys.argv[1]):
    m = re.search(r'PASS \[\s*([\d.]+)s\]\s+\(\s*\d+/\d+\) (\S+) (.*)', line)
    if m: binaries[m[2]].append(float(m[1]))
print(sum(map(len, binaries.values())), round(sum(map(sum, binaries.values())), 3))
for name, times in sorted(binaries.items(), key=lambda row: -sum(row[1]))[:12]:
    print(name, len(times), round(sum(times), 3))
PY
```

Output: **4,270 tests, 59.384 s summed, 225 binaries**. Largest binaries:
| Binary | Tests | Summed s |
|---|---:|---:|
| jig-client-view | 568 | 9.384 |
| jig-client-world (lib) | 263 | 5.212 |
| jig-client-protocol-world::protocol | 14 | 2.510 |
| jig-channel-world::channel | 26 | 2.504 |
| jig-boundary | 74 | 1.832 |
| jig-protocol-store | 296 | 1.707 |
| jig-domain-core | 253 | 1.579 |
| jig-client-protocol-world::client | 34 | 1.501 |
| jig-test-service | 101 | 1.479 |
| jig-one-process-world::one_process | 5 | 1.468 |
| jig-core-world::stories | 50 | 1.125 |
| jig-client-world::stories | 27 | 1.091 |

Whole packages: view 568 / 9.384 s; client world 359 / 8.384 s; core world 359 / 8.350 s: **1,286 / 26.118 s**, 44% of the sum.
Slowest individual tests, seconds:
`jig-client-wire-world::drift::client_wire_schema_matches_generated_code_and_goldens` 0.873;
`jig-one-process-world::one_process::a_kill_before_a_commit_loses_nothing_a_person_saw` 0.706;
`jig-client-protocol-world::protocol::a_person_s_part_request_is_answered_from_the_connector_s_outcome` 0.432;
`jig-client-protocol-world::native::a_native_client_signs_in_by_the_service_s_link_and_starts_a_chat` 0.412.
Thus the earlier finding's 0.27 s maximum is not this run's maximum; shared costs still dominate its sum.

## 2. CPU sampling, reused from the interrupted run

```text
$ cat /proc/sys/kernel/perf_event_paranoid; command -v perf gprofng
3
/usr/bin/gprofng
$ sh -c 'cargo nextest list --workspace --message-format json > target/nextest-list.json'
Finished `test` profile [unoptimized + debuginfo]
```

`perf` is absent, with kernel profiling restricted. GNU **gprofng 2.44** was the installed sampler.
The saved the command log command ran this once per matching, nonignored testcase from that JSON, with its suite's `cwd`:
`NEXTEST=1 gprofng collect app -p 0.1 -a off -F off -o target/<group>-samples/<n>.er <binary-path> --exact <name> --nocapture`.
It ran inside one `python3 -c...` job. Output: `PROFILED 1294`, exit 0, 52 s.
Groups: all focused tests of view, client world and core world, plus five one-process stories and three client-wire drifts.
Each testcase remained a separate process. `-F off` excludes descendants; no libtest batch or cross-test cache was measured.
Reports: `gprofng display text -metrics e.totalcpu:i.totalcpu -functions target/<group>-samples/*.er` and `-callers-callees`;
repeated symbol rows from different binaries are summed below. Reused scratch `target/` reports: `view-{functions,calls}.txt`,
`client-{all,functions,calls}.txt`, `core-{all,functions,calls}.txt`, `tail-functions.txt`.

| Inclusive sampled CPU seconds | View | Client world | Core world |
|---|---:|---:|---:|
| Total | 0.890 | 0.720 | 0.692 |
| `sizes::card_nodes` | 0.076 | 0.110 | — |
| `jig_client::limits::max_out` | 0.083 | 0.064 | — |
| `Phrasebook::load` | 0.180 | 0.066 | — |
| `phrases::jig_capacity` | 0.112 | 0.017 | — |
| `Observer::durable` | — | 0.007 | 0.062 |
| `store::component_limits` | — | 0.007 | 0.046 |

These are samples, **not measured wall seconds**. gprofng recorded a 500 µs period and warning `500 -> 0`;
short processes and kernel work are poorly represented. Below, “attributed” means each package's serial sum times its
sample share, a rough scale for ranking only; it includes process floors and has no confidence interval.
Cohort counts include unaffected tests. Neither these estimates nor expected gains are measured before/after savings.
## 3. Causes, fixes and exclusions

**A. Repeated page bounds: about 2.1 s attributed in the 927-test / 17.768 s view-and-client cohort.**
`crates/jig-client-view/src/sizes/pages.rs::{arranged_nodes,arranged_task,arranged_project}` recalculate `card_nodes`
through six task sections, their header and ten project sections. `owner_tree.rs::max_nodes` and
`sections/project/owner.rs::{max_nodes,historical_nodes}` recalculate it inside those sections too: 29 calls for one
ordinary `arranged_nodes` evaluation. Each repeats `sizes/owner_escalation.rs::nodes`' eight cases, including via
`owner_historical.rs::max_nodes`. Pass the same checked card/chip bounds through private helpers; compute the project's
common owner bound once. Keep every checked operation, public result and overflow refusal. Expected gain **1–1.5 s summed**.

**B. Wire validation on every drain: about 1.6 s attributed to `max_out` in that same cohort.**
`crates/jig-client/src/client.rs::Client::drain` calls `limits.rs::max_out`, which builds
`jig-client-protocol-link/src/limits.rs::Limits::new`, including `wire`, generated `check_limits` and `worst_case`.
Caller attribution puts 0.077 s of view samples and 0.013 s of client-world samples under `drain`: about **1.0 s** scaled.
`Client::build` already puts the checked bound in `domain_out.capacity()` and both pending-record lists.
Read that capacity for draining and replacement lists; retain construction-time validation. Expected gain **0.7–0.9 s**.
The broader proof cohort adds jig-client and client-protocol-world: **1,042 tests / 22.816 s**. Root callers' own
`max_out` computations remain; the estimate does not promise to remove their cost.

**C. Phrase work: about 2.7 s loading, 1.4 s querying capacities, in the 927-test cohort.**
`tests/client/src/test.rs::{phrasebooks,fixture_with}` and view `tests/cards.rs::shared_words` already share within
one process. The first load sorts and validates checked language data; another `OnceLock` saves zero across tests.
`phrases.rs::jig_capacity` scans the vocabulary from many `owner_*::required/worst_case` calls. Reusing its result within
one render is a possible later fix, but `words.rs::Words.book` is public and replaceable: a persistent cache could go stale.
No safely bounded saving established here; do not schedule a cache or a leaf-crate override from this profile alone.

**D. Core-world floors.** `tests/core/src/effects/constructors.rs::configured_with` includes bootstrap, restart and
initial exchanges, not merely allocating a fixture. `testing/jig-test-store-files/src/store.rs::Store::new` uses
`skein_kv::Store::memory`; its files are the test `Files`, not disk. `tests/core/src/observations.rs::Observer::durable`
rescans durable rows (about **0.8 s** attributed across 718 core/client tests, 16.734 s total). Incremental observations
could save part of that, but must keep all intermediate referee checks; no ≥0.5 s saving established. `component_limits`
is about 0.6 s attributed, principally startup validation, with no demonstrated duplicate to remove. Leave both out.
The drift tail reads schemas and goldens in `tests/client-wire/tests/drift.rs::check` once per family; the sampler does
not attribute its 0.873 s outlier to repeated parsing. The five one-process stories spend their samples in simulated
world iteration and client sizing. No sleep or disk-backed store cause was established in the three main cohorts.

## 4. What this permits

Plan A, B, then the exact inverse of jig `0509b985`'s timeout change, at the kept pins. Builds stay unoptimized.
The original finding's optimization levers are superseded by skein `b4c6241`, testing-strategy.md 8; workflow.md 2 and
tech-debt.md 4 stay unchanged. The old 12 s closing aim is superseded here by the user's under-15 s condition.
No run was on carbon-forge: its earlier ~14.4 s and excursions past 15 s leave a real question after this machine's pass.
The expected combined 1.7–2.4 s serial saving is modest; it is not proof of four-core headroom after the repin.
Graph: skein's `home-free-src-rust-skein` supplied `Queue.capacity`; source confirmed it at the pin and in main.
The graph did not find `Phrasebook`; source was read directly. jig is unindexed; paths and callers were read with `rg`.
Session 04's file checks were run at the base: unoptimized profiles and unchanged cost documents pass; the 15 s timeout,
removed comment and new helper/drain checks fail as expected before implementation. The matching proof cohorts above
were extracted from the full serial run; no fix or new private helper was implemented in this profiling task.
