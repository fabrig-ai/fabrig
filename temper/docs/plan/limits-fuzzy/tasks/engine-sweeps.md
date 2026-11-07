+++
id = "engine-sweeps"
part = "engine"
kind = "feature"
needs = ["state-hash"]
files = [
  "temper/tests/engine/domain/**",
  "temper/crates/temper-engine-domain/src/**",
  "temper/crates/temper-engine-domain-calendar/src/**",
  "temper/crates/temper-engine-domain-forge/src/**",
  "temper/crates/temper-engine-domain-forge-change/src/**",
  "temper/crates/temper-engine-domain-forge-issues/src/**",
  "temper/crates/temper-engine-domain-forge-tiers/src/**",
  "temper/crates/temper-engine-domain-trials/src/**",
  "temper/docs/design/domain/testing.status.md",
  ".config/nextest.toml",
  "temper/docs/plan/limits-fuzzy/figures/engine-sweeps.md",
]
effort = "high"
minutes = 140
+++

# The engine world's sweeps replay by digest

## Goal

`temper-engine-domain-world`'s walking shards cost 18 CPU s and its escalation sweep 3.5, the largest fuzzy costs nobody has profiled. Each runs two worlds in lockstep and compares them at every iteration. Compare a digest of each world's state at each iteration, as `skein/docs/design/lib.md`, 5.1, says, and the text only where they differ and once at each seed's end. Then fix the world's next causes, and stop when the next is under about 2% of a sweep. Every seed, cut, delay and ending stays, and two worlds still step in lockstep, compared at every iteration.

`temper/docs/plan/limits-fuzzy/figures/state-hash.md`, `## Gate`, decides the first part. **If it says `Gate: fails`,** skip the digest, the derives and the module below, leave the status entry, and take the causes your own profile names.

Acceptance:

- `temper/docs/plan/limits-fuzzy/figures/engine-sweeps.md` holds, in `notes/measuring.md`'s form, instructions and native CPU before and after for each of `fuzzy::walking::fuzzy_walking_shard_0` to `_7`, `fuzzy::walking::fuzzy_terminal_commit_cut_preserves_complete_replay_and_funding`, `fuzzy::escalation::fifteen_small_escalation_histories_cover_every_story_and_durable_cut` and `fuzzy::roles::twelve_small_role_histories_cover_each_holder_owner_order_and_atomic_cut`, with each cause, what it saved, what's left and what you stopped at. It says which hasher you took, with both arms' figures on one shard.
- `run_replayed` in `walking.rs` and `escalation.rs` and `reroute_replayed` in `roles.rs` compare through `replay`, below. All three still step both worlds and compare them at every iteration, and still end with their settled and final checks.
- **A difference is still caught and still readable:** a new focused test in `tests/suite/walking.rs`, `walking::a_replay_differing_in_its_root_store_or_counters_fails_its_compare`, table-shaped and cheap, holds that a replay differing in each of these fails its compare: a counter (`pages`), a store row (`store.rows`), and the root's own state (one world an `iterate` ahead). Its failing message holds both worlds' text.
- The seeds' lists in `tests/suite/fuzzy/walking.rs`, `escalation.rs` and `roles.rs` don't change. All tests pass by the same names as your base: `heavy cargo nextest run -p temper-engine-domain-world`, and the same with `--profile fuzzy`, but for the one new focused test.
- `heavy cargo check --tests -p temper-service -p temper-conformance-world -p temper-one-process-world -p temper-real-loop-world` passes.
- In `temper/docs/design/domain/testing.status.md`, the sentence under **Not yet** on section 6, "a state digest for replay", and its reference to `run_replayed`, go. The file's date is the day you edit it.
- The comments in `tests/suite/fuzzy/walking.rs` say what's true now: no shard's comment still gives "the `{:?}` of both worlds" as its cost, or the digest as yet to come.

## Inputs

- `temper/docs/plan/limits-fuzzy/notes/common.md`, `temper/docs/plan/limits-fuzzy/notes/measuring.md`, `temper/docs/plan/limits-fuzzy/notes/contracts.md`, and `temper/docs/plan/limits-fuzzy/figures/state-hash.md`.
- `skein/docs/design/lib.md`, 5.1; `skein/docs/foundation/testing-strategy.md`, 6 and 8; `temper/docs/design/domain/testing.md`, 6.1.
- `temper/tests/engine/domain/src/walking.rs`, `escalation.rs`, `roles.rs`, `store.rs`; `tests/suite/fuzzy/`.

## Owns and may change

Owns `temper/tests/engine/domain/**`, temper's derive lines in `notes/contracts.md`, *`engine-sweeps`*, the status sentence, and the figures file. In `.config/nextest.toml` you may change only the fuzzy profile's override for the walking shards: see below. A cause in those crates beyond a derive, or anywhere else below this world, goes under `## Left`, with its share, for `under-causes`.

**A branch stays in its tree, with one departure:** a missing `Hash` the compiler asks for in `jig/crates/**` or `skein/crates/**` takes that one word here, and your report names it.

**The readers of what you change:**

| Item | Read by | What happens there |
|---|---|---|
| `walking::run_replayed` | `tests/suite/walking.rs`, `tests/suite/fuzzy/walking.rs` | Nothing: same signature |
| `escalation::run_replayed` | `tests/suite/escalation.rs`, `tests/suite/fuzzy/escalation.rs` | Nothing: same signature |
| `roles::reroute_replayed` | `tests/suite/roles.rs`, `tests/suite/fuzzy/roles.rs`, `tests/suite/memory.rs` | `memory.rs`, `candidate_rosters_waiting_snapshots_keyed_history_and_recovery_fit_counted_memory`, runs it under `ROLE_RECOVERY_HEAP_CEILING`. The ceiling stays, and fewer strings should only make it easier. Read `meter.check`, and say what it does now |
| `Store`'s `Debug` | every world in this crate | Nothing: `Debug` stays |

**The compare,** a new module `temper/tests/engine/domain/src/replay.rs`, with its `pub mod` line in `lib.rs`:

```rust
//! Two worlds in lockstep compare by digest at each iteration, and by text where they differ
//! and once at a seed's end (skein/docs/foundation/testing-strategy.md, 6; skein/docs/design/lib.md, 5.1).

/// Panics, with both states' text, unless `first` and `replay` digest alike.
pub fn same_digest<T: Hash + Debug>(first: &T, replay: &T, at: &dyn Display);

/// Panics unless both print alike, which holds what `Hash` leaves out by design.
pub fn same_text<T: Debug>(first: &T, replay: &T, at: &dyn Display);
```

`World` in each of the three derives `Hash`, with its other state types, and `Store` takes its hand-written `Hash` (`notes/contracts.md`). The signatures are binding, and the bodies are yours. Each `run_*` calls `same_digest` after each pair of iterations and `same_text` once when settled, before `close`.

Calls already made, for you to build on:

- **Digests differ but the text is equal:** the states differ where `Debug` doesn't print (`Secret`, `Buffer`). `same_digest` panics all the same, and says that.
- **The trace:** `walking` and `roles` take `trace` out and compare it directly, so that a growing trace isn't reformatted at each iteration. Keep that, and do the same for `escalation`, which doesn't yet. `same_digest` then hashes a world with its trace empty, and the trace's own compare stays exact.
- **The hasher.** Try two on one shard:
  - `skein_lib::state_digest`, the design's own, is SHA-256 in an unoptimized build, and it may well cost more than the formatting it replaces.
  - A small `core::hash::Hasher` in `replay.rs`, with no key, taking whole words with a multiply and a rotate. These digests are compared once, in one process, and never kept, so nothing needs them stable across runs or machines. Its comment says so.

  Default: take the cheaper, and give both figures. A weak mix can't hide a difference for long here, since `same_text` still compares what's left at the seed's end.
- **`notes/contracts.md`,** *What `Hash` and `Debug` don't agree on*, is a reading of the code. Check it, and say what you found. If a field in its left column can change during a run and later answers depend on it, that's a `Hash` impl missing what 5.1 asks of it: the fix is in skein, and it's small, so make it.

Where to look next, once the compare is done. Reading shows these; the profile decides whether any is the cause:

- **`World::new`, per world, twice per seed:** `config(seed)`, `configured_forge()`, the limits and the models. A fixture that doesn't vary with the seed is built once per test.
- **`Store::request` and `Store::drive`** run each commit through the real store component and its codec, over `Files`. In `temper-conformance-world`'s profiles they were about 22% of a sweep. What the world's sweeps rely on stays real, but encoding the same rows twice needn't.
- **The referee, per step,** and each iteration's `settled()`: a scan where a position would do.
- **The trace's own `format!`s,** per boundary event in `iterate`. The trace is what's compared, so its contents stay, though not necessarily its cost: formatting the same value twice can go.

Options, each decided:

- **The shards stay eight, under the same names.** Their comment says they hold only until the digest lands, and merging them would save about 56 ms of processes and cost the gate's wall time. The comment changes. The sweep's seeds don't.
- **The fuzzy profile's override** names the walking shards "the suite's longest tests". If your native figures say they no longer are, remove that override block and its comment. If they still are, leave it.
- **`fuzzy::removal` and `fuzzy::reset`,** the same crash cases in both profiles, by `#[path]`: leave them. They aren't this plan's.
- **`fuzzy::endings` and `fuzzy::loads`:** they don't compare whole worlds. Give their figures only if a fix reaches them.
- **After the digest, a shard's still over its 10 s guide:** say so with its figures. It's not merged, moved or excepted.

## Steps

1. Read the gate. Profile all eleven tests in the acceptance on your base, as `notes/measuring.md` says, and take their native CPU.
2. If the gate holds, add temper's derives, following the compiler, and `Store`'s `Hash`. Write `replay.rs` and its test, and move the three `run_*` onto it, then try both hashers. Break the compare once by hand, and see the new test fail for that reason.
3. Fix the next cause, re-profile, repeat until the stop rule.
4. Edit the comments and the status sentence, and decide the override.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p PKG [-p PKG ...] --all-targets -- -D warnings` for the world and every crate you changed; the runs in the acceptance; `python3 -I -B tools/citations/citations.py --base origin/main`; commit and push.

## Report

Instructions and CPU seconds before and after, per test; the hasher's two arms; each cause and what it saved; how you know each seed, cut, delay and ending stands; what you found of `Hash` against `Debug`; what you left and whose it is; each derive you added beyond the list and each listed one the compiler never asked for; and your implementation and check minutes apart.
