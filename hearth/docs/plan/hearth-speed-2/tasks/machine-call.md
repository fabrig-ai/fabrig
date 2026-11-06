+++
id = "machine-call"
part = "builder"
kind = "feature"
needs = ["profile"]
files = ["hearth/testing/hearth-fake-machine/src/linux.rs", "hearth/testing/hearth-fake-machine/src/linux/ledger.rs", "hearth/testing/hearth-fake-machine/src/linux/files.rs", "hearth/testing/hearth-fake-machine/src/tests/**"]
effort = "high"
minutes = 45
+++

# The fake machine's call costs what it touches

## Goal

`Machine::call` copies every node's change counter into a map before each call and walks every node twice after it, and `room` sums every node's size on each `add` and `resize`. So a call's cost grows with the machine. By reading, a build's run makes about 1,000 calls over about 110 nodes, in the largest family of the suite, and that may be 30-50% of a run (a judgement: `figures.md` decides). Track what changed instead of finding it.

Acceptance:

- `call` builds no map of the nodes, and walks no node it didn't touch. `Process.changed` holds exactly what it holds today, after every call.
- `room` reads a running total and no longer sums the nodes. `"scenario byte bound"` fires at exactly the bytes it fires at today.
- `Machine`'s hand-written `Hash` doesn't change, and leaves the new fields out: they follow from `nodes`. Every state digest stays the same.
- A new test in `hearth/testing/hearth-fake-machine/src/tests/` holds the running total to what the nodes hold: a sequence of creates, writes, truncations and removes, and the byte bound still fires at its exact limit after a removal has freed room.
- `hearth-builder-world`'s `suite` falls by what `figures.md` gives this cost. Say what it fell by, measured, against that.

## Inputs

- `hearth/docs/plan/hearth-speed-2/notes/common.md` and `hearth/docs/plan/hearth-speed-2/notes/repairs.md`
- `hearth/docs/plan/hearth-speed-2/figures.md`: `### Builder suite` and `## What it found`
- `hearth/docs/design/domain/testing.md`, 3.1 and 9.1: "A fake's call costs what it touches", and 9.3 for what reads the machine's counts

## Owns and may change

- **Owns:** the files above. The public API of `hearth-fake-machine` doesn't change. `hearth/testing/hearth-fake-git`, `hearth/testing/hearth-fake-mmdebstrap`, `hearth/tests/builder/src/io.rs` and `hearth/tests/conformance/src/` call `call`, and aren't edited.
- **A node created and reclaimed within one call** is in neither the before nor the after of today's code, so it's not in `changed`. A survey found no path for it, since a created file keeps its handle. Check by reading. If there is a path, drop the node from `changing` in `reclaim` and test it. Node identities aren't reused (`next_node`), so that's exact.
- **`facts.size` and `facts.changes`** are written only at `files.rs:25,245` and `files.rs:48`, and `nodes` changes only at `files.rs:20,279`, at this cut. If you find another writer, it keeps the fields too.
- **Calls made outside `call`:** `add` and `reclaim` also run while a machine is laid and when a process ends, which `changed` doesn't see today. `call` clears `changing` as it begins, so they still don't count.
- **Not options:** `touched`'s second path resolution in `ledger.rs`, unless `figures.md` shows it over 3% of the family. `Behind::list`'s one call per entry stays, since `hearth/tests/builder/tests/suite/budgets.rs` pins what it counts.

## Steps

The sketch binds the fields and what they mean; the rest is yours:

```rust
pub struct Machine {
    // ...
    /// The bytes all nodes hold, which `room` reads. Outside `Hash`: it follows from `nodes`.
    used: u64,
    /// The nodes created, changed or removed since `call` began. Outside `Hash`: `changed` holds the result.
    changing: Vec<Node>,
}
```

- `add`, `change` and `reclaim` push to `changing`. `add`, `resize` and `reclaim` keep `used`, with checked arithmetic as these files have it.
- `call` clears `changing` as it begins, and after `operate` moves it into `process.changed`, replacing `ledger.rs:10` and `:54-63`.

1. The start check. Then `Ir` of `domain::a_changed_config_part_builds_config_alone_staged_from_the_published_software_layer`, and `hearth-builder-world`'s `suite` figure, twice.
2. The two fields, then the test. Break it by hand once: skip `used`'s update in `reclaim`, see the new test fail, restore.
3. The same two figures after.
4. **The readers' tests:** `heavy cargo nextest run -p hearth-fake-machine -p hearth-fake-git -p hearth-fake-mmdebstrap -p hearth-builder-world -p hearth-conformance-world`, then `--profile fuzzy -p hearth-builder-world`, since its sweeps read these digests. Check the memory tests pass with no ceiling moved.
5. The end of `notes/common.md`. Commit with the saving; push.

## Report

What `figures.md` gave this cost and what the repair saved, the figures on both sides, the machine, and `## Left`.
