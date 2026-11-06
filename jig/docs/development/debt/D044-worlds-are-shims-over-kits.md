# D044: Worlds are shims over kits that hold the worlds

- **Kind:** tests
- **Where:** `jig/tests/client/src/lib.rs`, `jig/tests/channel/src/lib.rs`, `jig/tests/engine-clients/src/lib.rs`, `jig/tests/tasks/src/lib.rs`, `jig/testing/jig-client-kit`, `jig/testing/jig-tasks-kit`
- **Seen:** 2026-11-06 at `683f4e8b92`, in the spot review of debt-24 (finding 2)
- **Checked:** 2026-11-06
- **Value:** medium (a fixture change stays in its story's world)
- **Size:** L
- **Status:** open
- **Paid when:** a kit holds only what a second package reads, under `test = false`; its tests are back in their world's suite; and a kit's modules aren't mutually cyclic, so a fixture change breaks only its readers.
- **Measure:** `rg -c '#\[test\]' jig/testing/jig-client-kit/src | awk -F: '{s+=$2} END {print s+0}'` (158)

## What

`jig/tests/client/src/lib.rs`, `jig/tests/channel/src/lib.rs` and
`jig/tests/engine-clients/src/lib.rs` each hold 2 lines;
`jig/tests/tasks/src/lib.rs` holds 3. Each re-exports its kit with
`pub use <kit>::*`. skein's `tests/llm/src/lib.rs` does the same in 2 lines.

`jig/testing/jig-client-kit/src` holds 19,181 Rust source lines and 158
`#[test]`s; its manifest has no `test = false`. `jig/testing/jig-tasks-kit/src`
holds 3,208 Rust source lines. Outside `jig/tests/tasks`, only two Rust files
read `jig_tasks_kit`: `temper/tests/engine/domain/tests/suite/panel_before_change.rs`
and `temper/tests/engine/domain/tests/suite/referee.rs`.

The spot review of debt-24 (2026-11-06, merge `683f4e8b92`, finding 2)
reports 46,295 lines moved into jig kits for net +79, and mutually cyclic
client-kit modules, so a fixture change still breaks dependents. Its real
gains stand: the directory predicts what's a kit, there are no aliases,
and no shipping crate depends on a kit (`jig/tests/boundary/src/tests/kits.rs`).

## Why it matters

D031 moved the worlds without narrowing what a fixture change breaks.
The world's library is a shim while the kit holds the world and its tests;
the client kit's mutually cyclic modules keep fixtures coupled across stories.

## What pays it back

Move kit tests back to their world's suite. Keep in a kit only what a
second package reads, with `test = false` as the rule. Separate the kit's
modules so they aren't mutually cyclic and a fixture change breaks only
its readers.
