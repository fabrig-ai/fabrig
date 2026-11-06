+++
id = "tree-once"
part = "tree"
kind = "feature"
needs = ["tree-fs"]
files = ["hearth/tests/tree/src/**", "hearth/tests/tree/tests/suite/walk.rs", "hearth/tests/tree/tests/suite/trees.rs", "hearth/tests/tree/tests/suite/hash.rs"]
effort = "high"
minutes = 75
+++

# What the tree world's checker computes twice

## Goal

With the fake filesystem's scans gone, repair what the tree suite's checker repeats, if the aim needs it: the same parent built, walked and listed per edit, the same parent file decoded per fault, and the oracle's bytes joined one byte at a time. The start check may end this task at once.

Acceptance:

- One commit per repair, each with what it saved, largest share first, until the options end.
- Every seed, edit, fault and limit is still checked, on the same values. The edit's draws are the same, so a seed replays.
- `hearth/tests/tree/tests/suite/memory.rs` and `hearth/tests/tree/tests/suite/fuzzy/tree.rs` don't change, and every ceiling stands. `checked` and `Helper::against` keep their signatures for them, as wrappers.
- `hash.rs:57` and `trees.rs:340`'s pinned values stand. The oracle shares no code with the crate's codec (`hearth/tests/tree/src/oracle.rs:1-4`): it computes once what it computed twice, by itself.
- Every test of `hearth-tree-world` passes in `default` and `fuzzy`.

## Inputs

- `hearth/docs/plan/hearth-speed-2/notes/common.md` and `hearth/docs/plan/hearth-speed-2/notes/repairs.md`
- `hearth/docs/plan/hearth-speed-2/figures.md`: `### Tree suite` and `## What it found`
- `hearth/docs/design/domain/testing.md`, 9.1: "A world computes once what its seed doesn't change"

## Owns and may change

- **Owns:** `hearth/tests/tree/src/**`, and in `hearth/tests/tree/tests/suite/walk.rs`, `trees.rs` and `hash.rs`, helpers and loop shape alone: every `assert` stays, on the same values.
- **Only `hearth/tests/tree/tests/suite/*` reads this crate.**
- **Options, pre-approved, in the order the profile's shares put them:**
  1. **The bytes joined one at a time:** `flat_map(oracle::item_bytes).collect()` at `hearth/tests/tree/tests/suite/hash.rs:46`, `hearth/tests/tree/src/against.rs:83` and `hearth/tests/tree/tests/suite/trees.rs:30` step an iterator per byte, unoptimized. A `pub fn body(items: &[Item]) -> Vec<u8>` in `oracle.rs` that does `extend_from_slice` per item. `tree::header` stays at the callers, because the oracle holds nothing of the crate's codec.
  2. **The parent per edit:** `each_edit_alone_walks_to_the_oracles_digests_and_file` (`walk.rs:74`) builds, walks and lists each of 10 parents 13 times. See the sketch.
  3. **The parent file per fault:** `an_io_failure_at_the_parent_stream_ends_the_walk_and_never_panics` (`walk.rs:287`) decodes the parent file and hashes it again per `Helper::against`, about 1,500 times: `record_of` once per seed, then `against_record`. The partial walk it checks still runs each time.
  4. **Inside `checked`:** `entries` of the same `Fs` computed twice (`hearth/tests/tree/src/against.rs:87-88` against `expected.rs:116,121`), `digests`' double read of the file (`helper.rs:385,393`), `oracle::order`'s two allocations per compare (`oracle.rs:195-202`).
  5. **A tree encoded whole per change:** `a_change_for_another_path_is_refused` (`trees.rs:121`) re-encodes every item for one change. See the sketch's `parts` and `joined`. `items_changed_in_order` (`trees.rs:201`) reads a prebuilt `Vec` of `file_of` results; each builds its own `Rng`, so replay holds.
- **`Rng` has no `Clone`** (`skein/crates/skein-lib/src/rng.rs:5`), and skein doesn't change. A test that needs the state after `layer` replays the draws from the start, counting them.
- **Stays local:** each of these lives inside its test. A process-wide cache adds nothing here, and one reachable from `Helper::answer` would first be built inside a meter.
- **Not options:** `hearth/tests/tree/tests/suite/memory.rs`'s reserve walks (`:188-191`): smaller, and the file's growth would count as the walk's. A separate binary for memory tests. Anything in `hearth/crates/`, or `hearth_lib::path`, which `path-bytes` owns.

## Steps

The sketch binds the names, within this crate:

```rust
// against.rs: a parent's side, computed once and checked against each child.
pub struct Parental<'a> { /* parent: &'a Fs, limits, first: Walked, record: Parent, entries, tree: Digest */ }
impl<'a> Parental<'a> {
    pub fn of(parent: &'a Fs, limits: &Limits) -> Self;
    pub fn checked(&self, child: &Fs, what: &str) -> Walked;
}
pub fn checked(parent: &Fs, child: &Fs, limits: &Limits, what: &str) -> Walked;  // Parental::of(..).checked(..)

// helper.rs. `Parent` is `Copy`.
pub fn record_of(file: &[u8], limits: &ProtocolLimits) -> Parent;
pub fn against_record(self, file: &'a [u8], record: Parent) -> Helper<'a>;  // against = record_of + this

// layers.rs: the state after a known number of draws.
pub fn draws_to(start: u64, at: &Rng) -> u64;
pub fn after(start: u64, draws: u64) -> Rng;

// check.rs: write_tree = joined(&parts(..)).
pub fn parts(items: &[TreeItem], limits: &Limits) -> Vec<Box<[u8]>>;
pub fn joined(parts: &[Box<[u8]>]) -> Box<[u8]>;
```

1. The start check. Stop if met.
2. `Ir` of `walk::each_edit_alone_walks_to_the_oracles_digests_and_file`, and `hearth-tree-world`'s `suite` figure, twice.
3. Each repair, then the same figures, then a commit with what it saved. Take out what saves nothing.
4. `heavy cargo nextest run -p hearth-tree-world`, then `--profile fuzzy -p hearth-tree-world`.
5. The end of `notes/common.md`. Push.

## Report

The ratio at your start, each repair and its saving, the figures on both sides, the machine, and `## Left`.
