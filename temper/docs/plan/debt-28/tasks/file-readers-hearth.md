+++
id = "file-readers-hearth"
part = "file-layer"
kind = "migration"
needs = []
files = ["hearth/crates/hearth-service-builder/src/lib.rs"]
effort = "medium"
minutes = 10
+++

# hearth's file reader calls `file_layer::step`

## Goal

First half of skein's D049, in hearth: `hearth-service-builder` moves to `FileIo::with_limits` and `file_layer::step`: one constructor and four calls. Behaviour doesn't change. The item says this file is "being migrated apart"; that has landed, and nothing else holds it now.

Acceptance:

- `rg -n 'file_layer::(down|down_until|up|expire)\b|FileIo::(new|with_whole_limit)\(' --type rust hearth` finds nothing.
- `limits.files`, whose five fields are `file_layer::Limits`'s, is the one source of those values.
- Net at most +10 lines.

## Inputs

- `temper/docs/plan/debt-28/notes/file-readers.md`, the recipe.
- `skein/docs/design/io.md`, 5.
- `temper/docs/plan/debt-28/notes/common.md`.

## Owns and may change

Owns the file in the header. `hearth-service-builder/src/limits.rs`'s `Files` stays: a conversion to `file_layer::Limits`, in `lib.rs`, is enough.

## Steps

The recipe: `io_env` is set in `lib.rs`, so a `files_env` is set beside it, and `up` and `expire` read that. `io_up` and `io_down` already have an env to hand. No new test: the note's move changes no behaviour.

`heavy cargo nextest run -p hearth-service-builder`, and the package for `hearth/tests/builder`, whose memory pin should hold (`cargo metadata` names it). `cargo fmt`; `(cd hearth && python3 -I -B scripts/old_forms.py)` if the tree has one; `heavy cargo clippy` naming each, `--all-targets -- -D warnings`; `python3 -I -B tools/citations/citations.py --all`. Commit, push.

## Report

As your text says, with the search's reading and the net lines.
