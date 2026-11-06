+++
id = "file-readers-jig"
part = "file-layer"
kind = "migration"
needs = []
files = ["jig/crates/jig-client-shell-native/src/files.rs", "jig/crates/jig-protocol-store/src/host.rs", "jig/crates/jig-protocol-store/src/tests/phase.rs", "jig/crates/jig-protocol-store/src/tests/fixtures/{disk,store}.rs", "jig/tests/store/src/world.rs", "jig/docs/design/protocol/store.md"]
effort = "medium"
minutes = 30
+++

# jig's file readers call `file_layer::step`

## Goal

First half of skein's D049, in jig: every reader moves to `FileIo::with_limits` and `file_layer::step`. The cut counted 12 thin calls (6 in production) and 8 constructors. Behaviour doesn't change.

Acceptance:

- `rg -n 'file_layer::(down|down_until|up|expire)\b|FileIo::(new|with_whole_limit)\(' --type rust jig` finds nothing.
- `jig_protocol_store::seed`'s signature (`[&mut FileIo; 3]`) doesn't change.
- `jig/docs/design/protocol/store.md` names the bound without the constructor (below).
- Net at most +25 lines.

## Inputs

- `temper/docs/plan/debt-28/notes/file-readers.md`, the recipe.
- `skein/docs/design/io.md`, 5.
- `temper/docs/plan/debt-28/notes/common.md`.

## Owns and may change

Owns the files in the header. Of `jig/docs/design/protocol/store.md`, one line: "made with such a bound (skein-io's `FileIo::with_whole_limit`)" becomes "made with such a bound (skein-io's `file_layer::Limits`, its `whole`)". The file is read by tests, so `jig-protocol-store`'s tests run.

Where the thought is, in `jig-protocol-store/src/host.rs`: `Drivers` builds three drivers, and each stores its own limits, and a `FileIo::new(1, 1, 1, ...)` is a `mem::replace` placeholder for `with_effective_user`, which may keep a small `Limits` of its own.

## Steps

The recipe. `Driver::{down, complete}` and `Files::{complete, request, progress}` keep their signatures: their callers need no edit. No new test: the note's move changes no behaviour.

`heavy cargo nextest run -p jig-protocol-store -p jig-client-shell-native`, and the package for `jig/tests/store` (`cargo metadata` names it). `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy` naming each, `--all-targets -- -D warnings`; `python3 -I -B tools/citations/citations.py --all`. Commit, push.

## Report

As your text says, with the search's reading and the net lines.
