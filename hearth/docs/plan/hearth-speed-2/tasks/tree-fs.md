+++
id = "tree-fs"
part = "tree"
kind = "feature"
needs = ["profile"]
files = ["hearth/tests/tree/src/fs.rs", "hearth/tests/tree/src/helper.rs", "hearth/tests/tree/src/layers.rs", "hearth/tests/tree/src/edits.rs", "hearth/tests/tree/src/expected.rs", "hearth/tests/tree/tests/suite/fs.rs"]
effort = "high"
minutes = 45
+++

# The tree world's filesystem looks up what it scans

## Goal

The tree suite's costliest tests spend most of their time, by reading, in the fake filesystem's scans: `Fs::links` walks every name of every node per `stat`, `Fs::lookup` does a linear find per path component, and `Fs::list` clones every name of a directory per page. So a walk is O(N²) in the fake and not in the code under test, and a 2,000-wide layer pays millions of steps for it. Give `Fs` the indexes. This is test code, so no assertion moves.

Acceptance:

- `links`, `lookup` and `names_of` answer from an index; a page of a directory clones only that page's names.
- Every answer is what it was: `Node::names` stays the listed order, `shuffle`, `paths` and `visit` are unchanged, and `layers::name` draws the same. A seed replays: `hash.rs:57` and `trees.rs:340`'s pinned values don't change.
- **A new test holds the index to a recount:** after `edits::edit` on generated layers, `links` and `lookup` agree with a brute-force count over `fs.paths()`. The helper and the oracle both read `Fs::links`, so a wrong index wouldn't show as a difference between them, and only this test sees it.
- Every memory test passes with no ceiling moved. `hearth/tests/tree/tests/suite/memory.rs` doesn't change.
- `hearth-tree-world`'s `suite` falls by what `figures.md` gives these functions (by reading, 1.2-1.6 CPU s). Say what it fell by.

## Inputs

- `hearth/docs/plan/hearth-speed-2/notes/common.md` and `hearth/docs/plan/hearth-speed-2/notes/repairs.md`
- `hearth/docs/plan/hearth-speed-2/figures.md`: `### Tree suite` and `## What it found`
- `hearth/docs/design/domain/testing.md`, 9.1: "A fake's call costs what it touches"

## Owns and may change

- **Owns:** `hearth/tests/tree/src/fs.rs`, and a new `hearth/tests/tree/tests/suite/fs.rs` with its `mod` line in `hearth/tests/tree/tests/suite/main.rs`.
- **Readers, for the new signatures alone:** `hearth/tests/tree/src/helper.rs:346-354` (`page`), `hearth/tests/tree/src/layers.rs:29,99,100,122,123`, `hearth/tests/tree/src/edits.rs:70,71,92`. `hearth/tests/tree/src/expected.rs:41` reads `links` and needn't change. Only `hearth/tests/tree/tests/suite/*` reads this crate.
- **`Fs::list`'s other readers:** if `names` replaces it everywhere, it goes; if something still needs owned names, it stays.
- **A caller that writes `.inode` or `.names` through `node_mut`:** by `rg` at this cut, none does (`node_mut`'s callers change content, mode, uid, gid, grown and xattrs). If you find one, route it through an `Fs` method that keeps the index.
- **`hash.rs:144`'s `*root_mut() = ...`** is on an `Fs` with no names. Keep it working.
- **Memory:** the index is built in `add`, `name`, `remove` and `detach`, before any meter's base, so it can't move a peak. `Helper::answer` runs inside a meter, so a lookup there allocates nothing new.
- **Not options:** `HashMap` (`hearth/tests/clippy.toml`); a separate binary for the memory tests, because a suite has one counting allocator (`hearth/docs/design/domain/testing.status.md`, 8.4's **Departs**).

## Steps

The sketch binds the fields' meaning and the public signatures:

```rust
pub struct Fs {
    nodes: Vec<Node>,
    /// Each directory's names, by name, beside `Node::names`, which keeps the listed order.
    by_name: Vec<BTreeMap<Vec<u8>, usize>>,
    /// How many names lead to each node.
    named: Vec<u32>,
    /// How many names lead to each inode number.
    links: BTreeMap<u64, u32>,
    clock: u64,
}
pub fn lookup(&self, path: &[u8]) -> Option<usize>;              // unchanged
pub fn links(&self, inode: u64) -> u32;                          // the map's count, at least 1, as today
pub fn names_of(&self, at: usize) -> usize;                      // from `named`
pub fn names(&self, path: &[u8]) -> Option<&[(Vec<u8>, usize)]>; // borrows; a page clones only its names
pub fn name(rng: &mut Rng, taken: impl Fn(&[u8]) -> bool) -> Vec<u8>; // in layers.rs: the same draws, as the predicate has the same truth
```

1. The start check. Then `Ir` of `hash::wide_and_deep_layers_hash_to_the_oracles` and `memory::walk::walking_wide_and_deep_layers_holds_no_more_than_its_ceiling`, and `hearth-tree-world`'s `suite` figure, twice.
2. The index and its readers. The recount test. Break it by hand once: leave `links` unchanged in `remove`, see it fail, restore.
3. The same figures after.
4. `heavy cargo nextest run -p hearth-tree-world`, then `--profile fuzzy -p hearth-tree-world`.
5. The end of `notes/common.md`. Commit with the saving; push.

## Report

What each lookup saved if you can split it, the figures on both sides, the machine, and `## Left`.
