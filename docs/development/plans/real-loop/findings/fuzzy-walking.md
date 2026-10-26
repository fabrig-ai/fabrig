# fuzzy_walking: where the 7.4 s of `..._commit_cuts_latency_replay_and_observation_pressure` go

BASE = `~/src/rust/tmp/planning/real-loop-base` (temper `e139ccaf4`), clean at the end. Other sessions loaded the
machine (load 5-13) in the later runs: compare variants only within one interleaved run.

```
$ cd BASE && cargo nextest run -p temper-engine-domain-world --profile fuzzy -E 'test(fuzzy_walking_commit_cuts_latency_replay_and_observation_pressure)'  # x3: 7.358 s, 8.366 s, 7.842 s
$ (temporary Instant/eprintln in run_replayed per phase) ... --no-capture -E 'test(...)'; summed over the 64 seeds:
6411 iterations (100 per seed, max 145; 600 is only the cap)   run_replayed 7285 ms
  iterate() of both worlds 1022 ms (14%)   trace == trace 14 ms   {:?} of both worlds 6153 ms (84%)   assert_eq of strings 8 ms
  493 MB of Debug text, 38 KB per world per iteration; 0.48 ms to format one world (about 80 MB/s, unoptimized)
$ (same, {:?} timed per field on the settled state)
domain 28.4 KB 0.40 ms | store 10.2 KB 0.13 ms | environment 4.4 KB 0.04 ms | assignment 1.7 KB | rest < 1 KB
  inside Domain, jig's Core is 74% of the text, the constant Limits 11%, the forge client 9% (from {:#?} of seed 3)
$ (state format and assert_eq removed, trace comparison kept) ... same command, 2 times
PASS [ 0.911s]   PASS [ 0.922s]
```

## Cause
`run_replayed` (tests/engine/domain/src/walking.rs:817) writes `{first:?}` and `{replay:?}` at every one of the
~6400 iterations and compares the strings. The lead holds with two corrections: the trace is already compared
directly (the string compare costs 8 ms), and the loop runs 100 iterations a seed, not 600. The cost is the
`Debug` walk of about 38 KB of state per world per iteration: 84% of the test. The rest (0.9 s) is the two worlds.
A hasher or early-exit writer still pays the walk; only a structural compare does not.

## What the test claims
`run_replayed`'s doc: "replay every frozen root, child, store and pending script state in lockstep, including
diagnostic queues with identical observation settings" (docs/design/domain/root.md, 12; skein
testing-strategy.md:394, "a seed replays to the same trace, and to the same digest of the state, which state
types derive `Hash` for"). The test draws 64 seeds of restart cut, commit and page latency, facts; each must
match at every iteration, end done, restart as drawn, and cover all 8 (restart, delayed commit, delayed page).
`Store`'s `Debug` is hand-written (store.rs:64: applied, rows, pending, held).

## Fix that keeps the claim in full: a structural equality (needs upstream)
`first.domain == replay.domain` (plus store, referee, protocol, events, out, assignment) at every iteration;
format both with `{:?}` only when they differ, so the failure message is unchanged. Probe: `#[derive(PartialEq)]`
on `engine::Domain` (crates/temper-engine-domain/src/domain.rs:32) is refused for 13 field types:
```
$ sed -i '31s/Debug/Debug, PartialEq/' crates/temper-engine-domain/src/domain.rs && cargo check -p temper-engine-domain
E0369 on skein_lib::Map x7, Slab x2, Queue x3, temper's Loads (loads.rs:123)
```
- skein (skein-lib `Map`, `Slab`, `Queue`, `List`, `Set`, `Deadlines`: all `derive(Debug)` only; logical equality
  for `Queue` and `Slab`), jig (`jig_domain_core::Core`, lib.rs:233, `derive(Debug)` only, and the nine child
  domains it holds), then temper (`Domain`, `Loads`, `forge::Domain`, the boundary `Event`/`Work`/`Fact`).
  The design's wording (`Hash`) needs the same set. A skein and jig change: not for temper's plan.
- Expected after: the test alone 0.95-1.2 s (0.91 s measured without the compare); suite median 9.0 s to 6.5 s.

## Temper-only alternative: shard the test across nextest slots
Same 64 seeds, same complete-state compare at every iteration, no smaller budget, no optimized build.
- tests/engine/domain/tests/fuzzy_walking.rs: factor `drawn(seed) -> Settings` and `shard(Range<u64>)` (run_replayed,
  `referee.done()`, `restarts` asserts) out of the test; 8 tests `fuzzy_walking_shard_0..7` of 8 seeds each; the
  original name keeps the `covered.len() == 8` assert, which depends on the drawn `Settings` alone (0.006 s).
- `.config/nextest.toml:24`: the `priority = 20` filter becomes `test(/^fuzzy_walking_shard_/)`.
- Expected after: each shard 1.0-1.5 s; CPU unchanged (7.4 s), so the suite is bound by total CPU (the other 49 sum 42 s).
```
$ sh ab.sh   # 3 rounds, each: base, no compare (bound for the upstream fix), 8 shards; suite wall (Summary)
base 9.318 8.535 8.965 | no compare 6.471 6.676 5.948 | shards 7.508 7.765 6.959   (50, 50, 58 tests)
```
Medians 9.0, 6.5, 7.5 s: sharding takes the test off the critical path, not off the machine.

## The next slowest tests of the fuzzy profile
```
$ cargo nextest run --workspace --profile fuzzy        # Summary [ 9.957s] 50 tests
9.956 temper-engine-domain-world::fuzzy_walking ..._commit_cuts_latency_replay_and_observation_pressure
4.655 temper-conformance-world::fuzzy_shared a_changed_judge_is_crashed_at_every_commit      (2.267 s alone)
4.276 temper-conformance-world::fuzzy_stories a_ci_repair_is_crashed_at_every_commit          (2.004 s alone)
2.597 temper-engine-protocol-forge-world::fuzzy_machine a_response_cut_at_each_byte_ends_its_call_once
$ cargo nextest run --workspace --profile fuzzy -j 1     # Summary [50.159s]; walking 8.413, fuzzy_shared 3.043, fuzzy_stories 2.688
$ git -C BASE checkout -- . && git -C BASE status --short        # empty
```
