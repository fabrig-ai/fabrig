+++
id = "file-readers-skein"
part = "file-layer"
kind = "migration"
needs = []
files = ["skein/crates/skein-shell/src/startup/store.rs", "skein/crates/skein-oauth-accounts/src/component.rs", "skein/crates/skein-io/src/tests/file.rs", "skein/testing/skein-world/src/files.rs", "skein/tests/io/src/{files,locks,private,pages,cuts,renames}.rs", "skein/tests/io/tests/suite/remove.rs", "skein/tests/kv/src/world.rs", "skein/tests/kv/tests/suite/real.rs"]
effort = "medium"
minutes = 60
+++

# skein's file readers call `file_layer::step`

## Goal

First half of skein's D049, in skein: every reader moves to `FileIo::with_limits` and `file_layer::step`. The cut counted 36 thin calls (3 in production, in `skein-shell`; 1 a doc comment) and 14 constructors. Behaviour doesn't change.

Acceptance:

- `rg -n 'file_layer::(down|down_until|up|expire)\b|FileIo::(new|with_whole_limit)\(' --type rust skein` finds only `skein-io/src/file_layer.rs`'s own definitions and module docs, and `skein-io/src/file_layer/`.
- The doc comment in `skein-oauth-accounts/src/component.rs` reads true.
- `skein-world`'s `files::Driver` keeps its signatures, so `skein/tests/oauth-accounts`'s six consumers need no edit.
- `skein/tests/io/tests/suite/files.rs`'s peak ceilings hold.
- Net at most +45 lines.

## Inputs

- `temper/docs/plan/debt-28/notes/file-readers.md`, the recipe.
- `skein/docs/design/io.md`, 5.
- `skein/docs/development/debt/D049-file-layer-thin-forms.md`.
- `temper/docs/plan/debt-28/notes/common.md`.

## Owns and may change

Owns the files in the header. Not `skein-io/src/file_layer.rs`, nor `store.rs` or `private.rs` beside it: other tasks hold them. A reader in skein a merge brings moves too.

`skein/tests/io`'s seven worlds repeat a near-identical `iterate`, and each moves on its own: don't merge them here.

## Steps

The recipe, file by file. `skein/tests/io` and `skein/tests/kv` follow one pattern, and their calls are sed-able once each world holds its limits.

No new test: the note's move changes no behaviour, and these worlds are what hold it.

`heavy cargo nextest run -p skein-io -p skein-shell -p skein-world -p skein-oauth-accounts`, and the `skein/tests/io` and `skein/tests/kv` packages (`cargo metadata` names them). `cargo fmt`; `(cd skein && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy` naming each, `--all-targets -- -D warnings`; `python3 -I -B tools/citations/citations.py --all`. Commit, push.

## Report

As your text says, with the search's reading and the net lines.
