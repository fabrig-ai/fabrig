+++
id = "file-thin-gone"
part = "file-layer"
kind = "feature"
needs = ["file-readers-skein", "file-readers-smith", "file-readers-jig", "file-readers-temper", "file-readers-hearth", "store-states", "opening-states"]
files = ["skein/crates/skein-io/src/file_layer.rs", "skein/docs/design/io.status.md", "skein/docs/development/debt/D049-file-layer-thin-forms.md"]
effort = "medium"
minutes = 15
+++

# The file layer's thin forms go

## Goal

Pay skein's D049. Every reader has moved, and `Store` and `Opening` have their states: `file_layer`'s own `down`, `down_until`, `up` and `expire` and `FileIo::{new, with_whole_limit}` have no caller left, so they go. `FileIo`'s `limits` field goes with them, since only they read it.

Acceptance:

- The measure reads 0: `rg --type rust 'file_layer::(down|down_until|up|expire)\b' skein smith jig temper hearth | wc -l`.
- `rg -n 'FileIo::(new|with_whole_limit)\(' --type rust skein smith jig temper hearth` finds nothing.
- `FileIo` has no `limits` field, and `file_layer.rs`'s module docs name only `step`'s entry points.
- `skein/docs/development/debt/D049-file-layer-thin-forms.md` is deleted, and the **Not yet** entries that name it leave `skein/docs/design/io.status.md`, with that section's other entries as they are.

About 40 fewer lines.

## Inputs

- `skein/docs/development/debt/D049-file-layer-thin-forms.md`, whole.
- `skein/docs/design/io.md`, 5, which already reads as the code will.
- `temper/docs/plan/debt-28/notes/common.md`.

## Owns and may change

Owns the files in the header. A reader a merge brought since its tree's task ran moves too, by `temper/docs/plan/debt-28/notes/file-readers.md`, in any tree, each edited only for that: the one case for this task to leave `file_layer.rs`. If `step`'s inner functions now read oddly with the thin forms gone (`down_inner`, `up_inner`), fold them only if that removes lines without moving behaviour.

`file_layer::step`'s path stays public as it is.

## Steps

Delete, and let the compiler find what's left. No new test: nothing has changed for a reader.

`heavy cargo check -p skein-io --tests`; then the workspace's dependents' tests compile: `heavy cargo check --tests` naming `skein-shell`, `skein-world`, `smith-agent-service`, `smith-local-service`, `smith-local-shell`, `jig-protocol-store`, `jig-client-shell-native`, `temper-box`, `temper-client-shell-cli`, `temper-service`, `hearth-service-builder`. `heavy cargo nextest run -p skein-io`. `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p skein-io --all-targets -- -D warnings`; `python3 -I -B tools/citations/citations.py --all`; `python3 -I -B tools/debt/debt.py`. Commit, push.

## Report

As your text says, with the measure's reading and the net lines.
