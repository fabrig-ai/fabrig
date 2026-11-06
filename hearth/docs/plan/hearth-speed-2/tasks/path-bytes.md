+++
id = "path-bytes"
part = "tree"
kind = "feature"
needs = ["profile"]
files = ["hearth/crates/hearth-lib/src/path.rs"]
effort = "medium"
minutes = 20
+++

# A path's checks use the library's search

## Goal

`path_valid` and `component` in `hearth-lib` loop over a path's bytes, on every tree item, written and read, and on every comparison of `tree_order`. A loop over bytes in an unoptimized build costs tens of times its release cost. `skein_lib::bytes::find` and `find_from` are built optimized, and `skein/docs/foundation/testing-strategy.md`, 8 says to use the library's search for this. Every answer stays the same.

Acceptance:

- `path_valid` finds a NUL and each `/` through `skein_lib::bytes`; `component` finds its `/` through `bytes::find`. Their signatures and answers don't change, and `tree_order` stays the same total order.
- `hearth/crates/hearth-lib/src/tests/path.rs` passes unchanged. If it has no row for a path with a NUL, a path ending in `/` or an empty component, add those rows.

## Inputs

- `hearth/docs/plan/hearth-speed-2/notes/common.md` and `hearth/docs/plan/hearth-speed-2/notes/repairs.md`
- `hearth/docs/plan/hearth-speed-2/figures.md`: `### Tree suite` and `## What it found`
- `skein/docs/foundation/testing-strategy.md`, 8: "A test is a process", its last sentence
- `skein/docs/foundation/programming-model.md`, 10: this is a step crate

## Owns and may change

- **Owns:** `hearth/crates/hearth-lib/src/path.rs`, and rows in `hearth/crates/hearth-lib/src/tests/path.rs`.
- **Readers, unedited:** `hearth/crates/hearth-protocol-tree/src/tree.rs`, `hearth/crates/hearth-protocol-tree/src/tree/read.rs`, `hearth/crates/hearth-domain-tree/src/parent.rs`, `hearth/tests/tree/tests/suite/trees.rs`.
- **Departs from your text:** if this adds only rows that move with no rule changed, there's no bite to show.
- **Not options:** anything else in `hearth-lib`. The encoder's `sha.update` calls and `Encoder::key`'s copy feed what's asserted and priced, and they're `last-round`'s if the profile shows them.

## Steps

```rust
pub fn path_valid(path: &[u8]) -> bool {
    // ... its checks before the loop stay ...
    if bytes::find(path, b"\0").is_some() {
        return false;
    }
    let mut start = 0_usize;
    while let Some(at) = bytes::find_from(path, b"/", start) {
        if !component_valid(path.get(start..at)) {
            return false;
        }
        start = at.saturating_add(1);
    }
    component_valid(path.get(start..))
}
```

1. The start check. Then `Ir` of `hash::wide_and_deep_layers_hash_to_the_oracles` and `trees::a_change_for_another_path_is_refused`, and `hearth-tree-world`'s `suite` figure, twice.
2. The two functions. The same figures after.
3. **The readers' tests:** `heavy cargo nextest run -p hearth-lib -p hearth-protocol-tree -p hearth-domain-tree -p hearth-tree-world -p hearth-domain-render`.
4. The end of `notes/common.md`. Commit with the saving; push.

## Report

The figures on both sides, the machine, and `## Left`.
