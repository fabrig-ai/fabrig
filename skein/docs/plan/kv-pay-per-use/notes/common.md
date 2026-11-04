# kv-pay-per-use: standing rules

Every task of this plan names this note, and `notes/contracts.md` beside it, which holds the
shapes that cross tasks: build to it exactly.

- **The plan:** skein-kv pays for what it holds (`skein/docs/design/kv.md`, sections 1, 3, 5,
  7 and 9): an open's reads sized by the file, a snapshot's trigger by the live map, recovery
  a chunk at a time, and restarts that count what they replay. The code is
  `skein/crates/skein-kv` and its world `skein/tests/kv` (`skein-kv-tests`); nothing else
  changes. jig's profiles and temper's limits are another plan's (fabrig-scale): never edit
  them.
- **The tree:** skein. Read `AGENTS.md`, then `skein/AGENTS.md`. No file in jig, smith or
  temper is yours: a reader's test there that turns red is reported under `## Left`, as
  your spec says, never fixed here.
- **The design is binding:** the sections each spec cites, as they stand on your base. Never
  edit `docs/design/` or `skein/docs/plan/`; what the code taught goes in the report. The
  status file is the plan's last task's.
- **The code's rules:** `skein/docs/foundation/programming-model.md`, sections 1, 6 and 10.
  `skein-kv` is a `no_std` step crate: no `as`, no indexing, no unchecked arithmetic, no
  `unwrap`, every `match` naming every variant, under `skein/clippy.toml` and the workspace's
  lints. `store.rs` keeps its five module-wide `#![expect]`s and adds none
  (`skein/docs/development/debt/D017-kv-store-states.md`): new data goes in the `Phase`
  variant that holds it, never in a new `bool` or `Option` field beside `phase`.
- **One file, several tasks:** `skein-kv/src/store.rs` is shared by every code task of this
  plan, each on the functions and arms its spec names. Change nothing else in it: no `use`
  line, no rename, no reformatting of code your task does not change. A change another
  task's region needs is a question.
- **The world:** `skein/tests/kv` is ordinary Rust under `skein/tests/clippy.toml`: one
  thread, one loop, a seed replays and each failure names its seed or cut. Its
  `World` (`skein/tests/kv/src/world.rs`) drives the store through `skein-io`'s file layer,
  the simulated kernel and the fake machine; `world.files()` reads every file of the root.
  Each new test file is one `mod` line in `tests/suite/main.rs`, at the place your spec
  names, so that tasks running side by side touch separate lines.
- **Budgets:** each new focused test stays under 0.5 CPU seconds; the skein focused suite has
  a 30 s budget (`skein/docs/foundation/testing-strategy.md`, 8 and 8.1). Over it, shrink the
  rounds or sizes within what the test must show, and report the measurement.
- **No public item** beyond your spec's and the contracts'; private helpers are yours. A
  compiler-forced arm or field in skein is always allowed, and listed in the report.
- **Old-form counts only fall:** no "actual", "genuine", "originally" in comments; run
  `python3 -I -B skein/scripts/old_forms.py`.
- **Every build and test through `heavy`,** one at a time, the narrowest first; check each
  run's count (nextest exits 4 when it selects nothing). Before pushing: `cargo fmt`,
  `heavy cargo clippy -p skein-kv -p skein-kv-tests --all-targets -- -D warnings`, the
  focused tests your spec lists, and `python3 -I -B tools/citations/citations.py --base
  origin/main`.
- **Other plans beside this one** touch none of this plan's files.
