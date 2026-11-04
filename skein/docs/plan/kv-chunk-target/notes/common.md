# kv-chunk-target: standing rules

Every task of this plan names this note, and `notes/contracts.md` beside it, which holds the
shapes that cross tasks: build to it exactly.

- **The plan:** a snapshot chunk packs to a small target and `Limits::chunk` is its ceiling
  (`skein/docs/design/kv.md`, 3, 5, 7 and 9). The code is `skein/crates/skein-kv` and its
  world `skein/tests/kv` (`skein-kv-tests`); nothing else changes. jig's and temper's
  profiles and limits, the budget knobs and layout migration are out of scope: no file in
  jig, smith or temper is yours, and a reader's test there that turns red is a question
  (these tasks are not to change a recovered store's outcome), never fixed here.
- **The design is binding:** the sections each spec cites, as they stand on your base. Never
  edit `docs/design/` or `skein/docs/plan/`, but where a spec says so; what the code taught
  goes in the report.
- **The stored format does not change.** A chunk's bytes, the frame's bytes and the format's
  version stay; `skein/tests/kv/tests/suite/golden.rs` and `golden/` stay unchanged and green
  (its `chunk` of 52 is below the target, so its packing is the same).
- **The kv-pay-per-use tests keep passing unchanged:** `open.rs`, `recovery.rs`, `store.rs`,
  `disk.rs`, `memory.rs`, `real.rs` and `fuzzy/map.rs`. Their `chunk`s are at or below the
  target, so each packs and reads as it did. `recovery.rs`'s one allowed edit is named in
  `target-read`'s spec.
- **The code's rules:** `skein/docs/foundation/programming-model.md`, sections 1, 6 and 10.
  `skein-kv` is a `no_std` step crate: no `as`, no indexing, no unchecked arithmetic, no
  `unwrap`, every `match` naming every variant, under `skein/clippy.toml` and the workspace's
  lints. New data goes in the `Phase` or `Compaction` variant that holds it, never in a new
  field beside them.
- **The world:** `skein/tests/kv` is ordinary Rust under `skein/tests/clippy.toml`: one
  thread, one loop. `World` (`skein/tests/kv/src/world.rs`) drives the store through
  `skein-io`'s file layer, the simulated kernel and the fake machine; `world.files()` reads
  every file of the root. A world that measures heap is `World::untraced`.
- **What a heap bound measures:** `skein_heap::Span` around the call, as `recovery.rs` does.
  Recovery's is the whole world's `peak − net`. A snapshot's is `World::fire_measured`'s
  `peak`, the store's `fire` alone: around `drive`, the fake disk's own growth and its
  pending writes would count (`skein-fake-machine/src/fs.rs`, `write`).
- **Budgets:** each new focused test stays under 0.5 CPU seconds; the skein focused suite has
  a 30 s budget (`skein/docs/foundation/testing-strategy.md`, 8 and 8.1). `skein-kv` builds
  unoptimized in the dev profile, so its CRC costs about 40 ns a byte: keep a test's bytes to
  what it must show, as the specs size them. Over budget, shrink a test's rows within what it
  must show and report the measurement.
- **No public item** beyond the contracts'; private helpers are yours.
- **Old-form counts only fall:** no "actual", "genuine", "originally" in comments; run
  `python3 -I -B skein/scripts/old_forms.py`.
- **Every build and test through `heavy`,** one at a time, the narrowest first; check each
  run's count (nextest exits 4 when it selects nothing). Before pushing: `cargo fmt`,
  `heavy cargo clippy -p skein-kv -p skein-kv-tests --all-targets -- -D warnings`, the
  focused tests your spec lists, and `python3 -I -B tools/citations/citations.py --base
  origin/main`.
- **A change of meaning runs its readers' tests:** the store's outcomes stay, but what it
  writes and reads under a `chunk` past the target does not, so each code task also runs the
  packages that make and recover stores from files: `heavy cargo nextest run -p
  jig-protocol-store -p jig-store-world -p temper-one-process-world`. This replaces the
  implementer's text's "direct dependents" for this plan: no public signature changes, one
  constant is added and no reader names it.
- **Other plans beside this one** touch none of this plan's files.
