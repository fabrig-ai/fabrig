use skein_lib::Rng;
use std::collections::BTreeSet;
use std::ops::Range;
use temper_engine_domain_world::walking::{Settings, run_replayed};

/// The seeds the walk draws; the shards below cover `0..SEEDS`, eight each.
const SEEDS: u64 = 64;

fn drawn(seed: u64) -> Settings {
    let mut random = Rng::new(seed + 7100);
    Settings {
        seed,
        restart: random.below(2) == 0,
        terminal_restart: false,
        commit_delay: u32::try_from(random.below(3)).expect("tiny latency"),
        page_delay: u32::try_from(random.below(3)).expect("tiny latency"),
        facts: random.below(2) == 0,
    }
}

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

/// Seeds 8 to 15 of the walk. Sharded for its cost: the `{:?}` of both worlds
/// at every iteration, 493 MB of text and 84% of the walk's time, until the
/// state digest (`shard`).
#[test]
fn fuzzy_walking_shard_1() {
    shard(8..16);
}

/// Seeds 16 to 23 of the walk. Sharded for its cost: the `{:?}` of both worlds
/// at every iteration, 493 MB of text and 84% of the walk's time, until the
/// state digest (`shard`).
#[test]
fn fuzzy_walking_shard_2() {
    shard(16..24);
}

/// Seeds 24 to 31 of the walk. Sharded for its cost: the `{:?}` of both worlds
/// at every iteration, 493 MB of text and 84% of the walk's time, until the
/// state digest (`shard`).
#[test]
fn fuzzy_walking_shard_3() {
    shard(24..32);
}

/// Seeds 32 to 39 of the walk. Sharded for its cost: the `{:?}` of both worlds
/// at every iteration, 493 MB of text and 84% of the walk's time, until the
/// state digest (`shard`).
#[test]
fn fuzzy_walking_shard_4() {
    shard(32..40);
}

/// Seeds 40 to 47 of the walk. Sharded for its cost: the `{:?}` of both worlds
/// at every iteration, 493 MB of text and 84% of the walk's time, until the
/// state digest (`shard`).
#[test]
fn fuzzy_walking_shard_5() {
    shard(40..48);
}

/// Seeds 48 to 55 of the walk. Sharded for its cost: the `{:?}` of both worlds
/// at every iteration, 493 MB of text and 84% of the walk's time, until the
/// state digest (`shard`).
#[test]
fn fuzzy_walking_shard_6() {
    shard(48..56);
}

/// Seeds 56 to 63 of the walk. Sharded for its cost: the `{:?}` of both worlds
/// at every iteration, 493 MB of text and 84% of the walk's time, until the
/// state digest (`shard`).
#[test]
fn fuzzy_walking_shard_7() {
    shard(56..64);
}

#[test]
fn fuzzy_terminal_commit_cut_preserves_complete_replay_and_funding() {
    for seed in 0..4 {
        let mut random = Rng::new(seed + 8100);
        let settings = Settings {
            commit_delay: u32::try_from(random.below(3)).expect("tiny latency"),
            page_delay: u32::try_from(random.below(3)).expect("tiny latency"),
            facts: random.below(2) == 0,
            ..Settings::terminal(seed)
        };
        let world = run_replayed(settings);
        assert!(world.referee.done_after_cut() && world.referee.terminal_replay_done(), "seed {seed}");
        assert_eq!(world.restarts, 1, "seed {seed}");
    }
}
