# Session 14: the fuzzy walking story's cost

temper's fuzzy suite runs as long as one test of `fuzzy_walking.rs`,
about 7.8 s alone, against its 8 s aim (decisions.md, 12). The profile
(`findings/fuzzy-walking.md`) names the cause: 84% of the test is the
`Debug` text of both worlds, which `run_replayed` writes and compares at
each iteration. The cure, a state digest, needs `Hash` on skein's and
jig's state types: the next plan's, outside this one (decisions.md, 3
and 12; `later.md`). This
session shards the test into eight tests of eight seeds, with the same
64 seeds and the same compare at every iteration: off the suite's
critical path, not off the machine. Needs at start: nothing. Step
`temper-14.1`.

## Read first

- This plan's README (its Rules), `decisions.md` (3, 12),
  `findings/fuzzy-walking.md` ("Cause", "Temper-only alternative"),
  `findings/base-times.md` (the fuzzy rows), `later.md` (the digest row).
- temper's `docs/development/workflow.md`: 2. skein's
  `testing-strategy.md`: 6 ("Replay"), 8, 10.
- Code: `tests/engine/domain/tests/fuzzy_walking.rs`; the world's
  `walking::{Settings, World, run_replayed}`, `World::{referee,
  restarts}`, `WalkingReferee::done`; `.config/nextest.toml`;
  `scripts/proof-live.sh`.

## What the design asks

- **testing-strategy.md 6, "Replay":** the same trace and the same
  digest of the state. This step keeps the walk's `Debug` compare; the
  digest is later.
- **testing-strategy.md 8:** a group of rows takes a test of its own to
  share the work across threads; build settings never stand in for a fix.

## Choices this plan makes

- **Shards, not the cure** (decision 12).
- **Eight shards of eight seeds,** each 1.0 to 1.5 s in the findings;
  the suite is then bound by its total CPU (the other tests sum about
  42 s), so more shards gain nothing.
- **The shards take the override's priority 20** and start first, as the
  suite's longest tests; the original test, now about 6 ms, loses it.
- **The covered assert stays in the original test:** it reads the drawn
  `Settings` alone and runs no walk.
- **One literal range per shard,** so `git grep` reads the 64 seeds.

## Rules that bind it

- **No optimized test builds and no `[profile.*]` change** (README) bars
  Cargo's build profiles and the suites' settings; an override's filter
  and a comment in `.config/nextest.toml` change neither.
- **Never "and" in a new name** (README): the original, cut to its coverage claim,
  is renamed for it.
  **Rows carry data** (testing-strategy.md 10): a shard is `shard(range)`.

## Increments

### 14.1 Eight shards of the walking story
Scope: the walking story's 64 seeds run as eight tests of eight seeds,
which nextest starts first (decision 12; testing-strategy.md 8).
Needs: nothing beyond the session's start.
Files: `tests/engine/domain/tests/fuzzy_walking.rs`;
`.config/nextest.toml` (no other step of this plan edits it).
Shape:
- `fn drawn(seed: u64) -> Settings`: the original loop's draw, as it is
  (`Rng::new(seed + 7100)`, `terminal_restart: false`).
- `fn shard(seeds: Range<u64>)`: `run_replayed(drawn(seed))` for each
  seed, then the `referee.done()` and `restarts` asserts, as today.
- `.config/nextest.toml`: the `[[profile.fuzzy.overrides]]` filter
  becomes `test(/^fuzzy_walking_shard_/)`, `priority = 20`, under a
  comment `# The walking story's eight shards first, the suite's longest
  tests.` `[profile.live]`'s comment, stale since the `live` binary
  landed (`crates/temper/tests/live.rs`), becomes two lines: `# The live
  tier (docs/design/domain/testing.md, 2.7): crates/temper's live binary` and
  `# alone, one test at a time, which scripts/proof-live.sh runs.` Its
  filter and settings stay.
Test shape: in `fuzzy_walking.rs`, beside its imports, in full:
```rust
use std::ops::Range;
/// The seeds the walk draws; the shards below cover `0..SEEDS`, eight each.
const SEEDS: u64 = 64;

/// Walks each seed twice in lockstep and compares the complete state of both
/// worlds at every iteration (`run_replayed`). The walk is slow by its
/// compare: the `{:?}` of both worlds at every iteration, 493 MB of text over
/// the 64 seeds and 84% of the walk's time, so the seeds run as eight shards
/// that share the fuzzy suite's threads (testing-strategy.md, section 8). The
/// cure is the state digest of skein's testing-strategy.md, section 6, once
/// the state types derive `Hash`; the shards hold only until it lands.
fn shard(seeds: Range<u64>) {
    for seed in seeds {
        let settings = drawn(seed);
        let world = run_replayed(settings);
        assert!(world.referee.done(), "seed {seed}");
        assert_eq!(world.restarts, u32::from(settings.restart), "seed {seed}");
    }
}

/// The 64 seeds draw both commit cuts across a delayed and an immediate store
/// and page terminal.
#[test]
fn fuzzy_walking_seeds_cover_every_commit_cut() {
    let covered: BTreeSet<_> =
        (0..SEEDS).map(drawn).map(|s| (s.restart, s.commit_delay != 0, s.page_delay != 0)).collect();
    assert_eq!(covered.len(), 8, "both commit cuts across delayed/immediate store and page terminals");
}

/// Seeds 0 to 7 of the walk. Sharded for its cost: the `{:?}` of both worlds
/// at every iteration, 493 MB of text and 84% of the walk's time, until the
/// state digest (`shard`).
#[test]
fn fuzzy_walking_shard_0() {
    shard(0..8);
}
```
The others, alike, each with the same doc and its own seeds:
`fuzzy_walking_shard_1` (`8..16`) to `fuzzy_walking_shard_7` (`56..64`).
The coverage test, the old test cut to its claim, takes a name for it,
with no `_and_`. The worlds' limits stay
`World::new`'s. Ceilings: each shard about 5 s; the original under 20 ms.
Proof: world, the fuzzy profile, no higher: the step changes how the
domain world's fuzzy tests are cut and scheduled, and nothing above runs
them. On the increment's commit (the gate lane takes a committed tree):
- each shard alone, three runs: `cargo nextest run -p
  temper-engine-domain-world --profile fuzzy -j 1 -E
  'test(/^fuzzy_walking_shard_/)'`. Each shard's median is under about
  5 s, 1.0 to 1.5 s expected; the commit body gives the eight medians.
- the fuzzy suite, three gate-lane runs, then the package's summed time:
  ```
  sh -c 'export NEXTEST_DOUBLE_SPAWN=0; L=$HOME/src/rust/tmp/temper/real-loop/logs/temper-14; mkdir -p $L; for i in 1 2 3; do cargo nextest run --workspace --profile fuzzy >$L/fuzzy-$i.log 2>&1; grep Summary $L/fuzzy-$i.log; done'
  for i in 1 2 3; do sed -nE 's/^ *PASS \[ *([0-9.]+)s\] \([^)]*\) ([^: ]+)[: ].*/\1 \2/p' ~/src/rust/tmp/temper/real-loop/logs/temper-14/fuzzy-$i.log | awk '$2 == "temper-engine-domain-world" {s += $1} END {printf "%.2f\n", s}'; done
  ```
  The commit body gives the median `Summary` against the base's 8.65 s
  (`findings/base-times.md`), about 7.5 s expected, and the package's
  median sum against the base's 11.72 s. The sum does not fall, as the
  CPU is unchanged, and the commit body says so.
Don't: raise a budget or a timeout; run fewer seeds or iterations, or
skip the compare at any iteration; use an optimized profile or an
`opt-level`; change skein, jig or smith, or derive `PartialEq` or `Hash`
on temper's domain (the digest, later); name a new test with `_and_`;
change `walking.rs`, the terminal test, or any profile's settings.
Size: no production line; about 85 net test lines, 3 of configuration.
Removes: the single 7.8 s test's serial cost from the fuzzy suite's
critical path; `[profile.live]`'s stale comment.

## Questions to raise when reached

- **A shard's median over 5 s alone:** raise it with the eight medians;
  no shard is re-cut without an answer.
- **The suite's median `Summary` not below 8 s:** raise it with the three
  Summaries and each log's three slowest tests: the suite is then bound
  by its other tests, a question for the review, not more shards.
- **The package's median sum more than 2.6 s above 11.72 s** (one gate's
  swing of the sums, README Pulse): raise it with the three logs.

## Done when

Each in `~/src/rust/temper`, after `git fetch`; each prints what it says:
- `git grep -n -F 'fn fuzzy_walking_shard_7()' origin/main -- tests/engine/domain/tests/fuzzy_walking.rs`
  prints one line.
- `git grep -h -o -E 'shard\([0-9]+\.\.[0-9]+\)' origin/main -- tests/engine/domain/tests/fuzzy_walking.rs | tr '\n' ' '`
  prints `shard(0..8) shard(8..16) shard(16..24) shard(24..32)
  shard(32..40) shard(40..48) shard(48..56) shard(56..64)`.
- `git grep -n -F -e 'fn fuzzy_walking_seeds_cover_every_commit_cut()' -e 'state digest' origin/main -- tests/engine/domain/tests/fuzzy_walking.rs`
  prints ten lines: the coverage test, `shard`'s doc and the eight shards'.
- `git grep -c -F '493 MB' origin/main -- tests/engine/domain/tests/fuzzy_walking.rs`
  prints the file with 9.
- `git grep -n -F 'filter = "test(/^fuzzy_walking_shard_/)"' origin/main -- .config/nextest.toml`
  prints one line.
- `git grep -n -F -e 'arrives in 15.4' -e 'does not hold yet' -e 'fuzzy_walking_commit_cuts' origin/main -- .config/nextest.toml`
  prints nothing.
- `git grep -n -F 'global-timeout = "60s"' origin/main -- .config/nextest.toml`
  prints one line: the fuzzy budget is unchanged.
- `git grep -n 'opt-level' origin/main -- '*.toml'` prints one line,
  `Cargo.toml`'s for `skein-codegen`, as at the base.
