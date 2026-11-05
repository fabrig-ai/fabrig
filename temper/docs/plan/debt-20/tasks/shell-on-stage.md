+++
id = "shell-on-stage"
part = "host-lanes"
kind = "feature"
needs = []
files = ["smith/crates/smith-local-shell/src/front/headless/input.rs", "smith/crates/smith-local-shell/src/credentials/program_source/runner.rs"]
effort = "medium"
minutes = 15
+++

# smith: the local shell checks io with skein's `valid` and drives it through `stage`

## Goal

Part of `smith/docs/development/debt/D003-host-io-route-table.md`, whose reader outside the local service this is. `front/headless/input.rs` calls `io::valid`, not `smith_local_service::native_io_valid`, so `host-io-lanes` can remove it. `runner.rs`'s hand-driven `io::resume` loop becomes `io::stage::up`.

Acceptance:

- `rg -n 'native_io_valid|io::resume' smith/crates/smith-local-shell/src` finds nothing.
- `heavy cargo nextest run -p smith-local-shell` passes, with no assertion changed.
- The branch's net production lines fall (estimate: -4). `smith/scripts/old-forms.txt`'s `io_resume` falls by one.

## Inputs

- `temper/docs/plan/debt-20/notes/common.md`
- `smith/docs/development/debt/D003-host-io-route-table.md` (stale on one point: its skein side is done, `skein_io::valid` and `file_layer::owns` both exist)
- `skein/docs/design/io.md`, 2
- `skein/crates/skein-io/src/stage.rs`

## Owns and may change

- **Owns:** the files in `files`.
- **The verdict:** `io::valid` refuses from 2^24 sockets, where `native_io_valid` passed and `Io::new` then panicked. A clean refusal in its place is what `smith/docs/design/protocol/limits.md`, 4 asks for.
- **`runner.rs` today runs completions, then ready, then fired;** `stage::up` takes ready first. **If a test's events change because of that swap,** keep the hand drive at this site, say so in your report, and leave the rest: the count's floor rises by one. Don't change an assertion.
- **`io::down` loops stay:** the stage's `down` isn't in the item, and the pacing tests pin those loops.
- **`Queue::uncapped().room()`** is `u32::MAX - len`, so the stage's room checks don't bind here.

## Steps

1. The two files.
2. `cargo fmt`; `(cd smith && python3 -I -B scripts/old_forms.py --write)` per the common note; `heavy cargo clippy -p smith-local-shell --all-targets -- -D warnings`; the tests above. The citations check. Commit and push.

## Report

The common note's items; whether `runner.rs` moved.
