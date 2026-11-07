+++
id = "store-sweeps"
part = "store-sweeps"
kind = "feature"
needs = ["world-setup"]
files = [
  "jig/tests/store/src/patches.rs",
  "jig/tests/store/tests/suite/main.rs",
  "jig/tests/store/tests/suite/generations.rs",
  "jig/tests/store/tests/suite/patching.rs",
  "jig/tests/store/tests/suite/pending_switch.rs",
  "jig/tests/store/tests/suite/upgrades/mod.rs",
  "jig/tests/store/tests/suite/upgrades/patching.rs",
  "jig/tests/store/tests/suite/upgrades/check.rs",
  "jig/tests/store/tests/suite/upgrades/frames.rs",
  "jig/tests/store/tests/suite/fuzzy/patching.rs",
  "jig/tests/store/tests/suite/fuzzy/generations.rs",
  "jig/tests/store/tests/suite/fuzzy/pending_switch.rs",
  "jig/tests/store/tests/suite/fuzzy/upgrades.rs",
  "jig/docs/design/domain/testing.status.md",
  "jig/docs/design/protocol/store.status.md",
]
effort = "medium"
minutes = 60
+++

# The store world's sweeps move to fuzzy, and focused keeps cuts chosen for why

## Goal

`jig/docs/design/domain/testing.md`, 5 and 9 draw the line: focused crashes after a few chosen cuts, fuzzy sweeps every one. Seven focused tests of `jig/tests/store` sweep. Each sweep moves under `fuzzy::` whole, keeping its name, and focused keeps a few cuts named for why they're chosen.

**`plan.md`'s table is the map:** `jig/docs/plan/jig-world-cost-2/plan.md`, *Sweeps*. It says, per test, why no existing fuzzy test holds it.

Acceptance:

1. Each row of the table holds: the focused test by its new name, with the cuts named; the fuzzy test by its old name, making every check it makes today, over every cut it sweeps today. Each old name is gone from focused.
2. Both suites pass for `jig-store-world`, and `--profile fuzzy` selects the seven moved tests.
3. Sweep and chosen cuts run **one body**, so a focused cut makes the same checks as the sweep's.
4. Each chosen cut is found by what it does, not by a bare number: a `Change` in the traced `changes()`, or a commit found in `written()`, and the test asserts it was reached.
5. A focused test whose sweep asserts it reached both endings still does: `cuts_across_check_and_switch` asserts `source_opened && switched`, and the switch's test sees the old generation and the new.
6. `heavy cargo nextest run -p jig-protocol-store --lib -E 'test(/words/)'` passes: this cut edited `jig/docs/design/protocol/store.md`, 14, which `.config/gate/read-by-tests.txt` lists, in text those tests don't pin.
7. Your two **Not yet** entries are gone (below).

## Inputs

- `jig/docs/design/domain/testing.md`, 5 and 9, as this cut wrote them.
- `jig/docs/design/protocol/store.md`, 10.4 and 14, as this cut wrote them.
- `jig/docs/plan/jig-world-cost-2/plan.md`, *Sweeps*.
- `jig/docs/plan/jig-world-cost-2/notes/common.md`.
- `jig/tests/store/src/world.rs`, `trace_writes`, `changes`, `written` and `cut_at`, and `jig/tests/store/src/patches.rs`, `cuts`.

## Owns and may change

- **Owns:** the spec's `files`.
- **Departure from the implementer's text:** `docs/design/` isn't yours, except that this task removes the two **Not yet** entries this cut wrote for it, and nothing else there: in `jig/docs/design/domain/testing.status.md`, 9's "the store world's whole sweeps of a generation's, a patch's, a check's and a patched read's cuts still run in focused", and in `jig/docs/design/protocol/store.status.md`, 14's "the store world's whole sweeps of a copy's mutation cuts and of the check and switch still run in focused". If the table lands differently from what testing.md, 9 or store.md, 14 say, say so in the report: the design isn't yours to change.
- **Readers, read at the cut:** the helpers these tests use are read only in their files. They become `pub(crate)` where a fuzzy file needs them, reached as `crate::generations::..`, `crate::patching::..`, `crate::pending_switch::..` and `crate::upgrades::check::..`. `upgrades/mod.rs`'s `check`, `frames` and `patching` become `pub(crate) mod`. `main.rs` gains `mod generations; mod pending_switch; mod upgrades;` inside `mod fuzzy`. `upgrades/check.rs:478`'s comment and `fuzzy/patching.rs:10`'s ("beside the focused exhaustive mutation sweep") follow the move.
- **Options:**
  - `cut_at` numbers kernel operations in a traced world and file requests in an untraced one. The two `generations` sweeps are untraced today and stay so, keeping their numbering: their body takes a prepared world or a `traced: bool`, and their chosen cuts use a traced one. Either is yours.
  - A chosen cut the code doesn't have as described (no `Rename` where the table says one, say): choose the nearest that shows the same thing, name it for what it does, and put it in the report so `plan.md` can be corrected.
  - The moved tests keep their names, since those still say what they check. Rows of one table test stay one test: `frames`' `kinds()` stays one test, and so does its closing `waiting` block.

## Steps

1. A small helper in `jig/tests/store/src/patches.rs`, beside `cuts`:

   ```rust
   /// The operation of the first and the last change of this kind in a traced world.
   pub fn first_change(world: &World, kind: Change) -> u32;
   pub fn last_change(world: &World, kind: Change) -> u32;
   ```
2. `generations.rs`: `pub(crate) fn a_first_start_cut_is_whole(world: World, cut: u32, seed: u64)` and `pub(crate) fn a_stray_cut_is_gone(world: World, cut: u32, seed: u64)`, holding today's loop bodies. The focused tests and their cuts are the table's. `fuzzy/generations.rs` holds the two sweeps, with today's loops.
3. `patching.rs`: `pub(crate) fn a_patch_cut_recovers(cut: u32)`, and the focused test. The sweep goes to `fuzzy/patching.rs`.
4. `pending_switch.rs`: `pub(crate) fn a_switch_cut_leaves_one_generation_whole(cut: u32) -> bool`, returning whether the old generation was active. The focused test asserts it saw both. `fuzzy/pending_switch.rs` holds the sweep.
5. `upgrades/patching.rs`: `pub(crate) fn a_drop_copy_or_switch_cut_recovers(operation: u32, change: Change, expected_last: u64)`. `upgrades/check.rs`: `pub(crate) fn a_check_cut_recovers(records: &Records, cut: u32) -> bool`, returning whether the source stayed active, and a `pub(crate) fn seal_and_switch(..)` giving `first`, `after_switch` and the rename's operation. `upgrades/frames.rs`: `pub(crate) fn a_read_cut_preserves_files(read: TranscriptRead, cuts: impl Fn(u32) -> Vec<u32>)`, holding today's per-kind body with its `waiting` block. The three sweeps go to `fuzzy/upgrades.rs`.
6. No break-it-once for what only moved. For the cuts chosen by `first_change` and `last_change`, point one test at a wrong `Change` once, see its assertion of reaching that cut fail, restore it, and report that.
7. Remove the two **Not yet** entries.
8. Narrow runs by name in each profile (`jig/docs/plan/jig-world-cost-2/notes/common.md`, *Running*), then acceptance 6, then the implementer's closing steps for `jig-store-world`. Commit and push.

## Report

The implementer's report, plus each focused test's cuts as they landed (the operation, what it does and why it's chosen), any row of the table that differs and why, and the check you broke and what it showed.
