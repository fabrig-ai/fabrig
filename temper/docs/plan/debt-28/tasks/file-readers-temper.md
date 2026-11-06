+++
id = "file-readers-temper"
part = "file-layer"
kind = "migration"
needs = []
files = ["temper/crates/temper-box/src/group_records.rs", "temper/crates/temper-client-shell-cli/src/directory.rs", "temper/crates/temper/src/fake/check.rs", "temper/crates/temper-service/src/tests/stop/pre_settings.rs", "temper/testing/temper-github-app/src/store.rs"]
effort = "medium"
minutes = 25
+++

# temper's file readers call `file_layer::step`

## Goal

First half of skein's D049, in temper: every reader moves to `FileIo::with_limits` and `file_layer::step`. The cut counted 16 thin calls (9 in production) and 5 constructors, in five files. Behaviour doesn't change.

Acceptance:

- `rg -n 'file_layer::(down|down_until|up|expire)\b|FileIo::(new|with_whole_limit)\(' --type rust temper` finds nothing.
- `temper-client-shell-cli`'s `Directory::{complete, progress, request}` and `temper-box`'s `GroupRecords::{complete, tick}` keep their signatures.
- Net at most +20 lines.

## Inputs

- `temper/docs/plan/debt-28/notes/file-readers.md`, the recipe.
- `skein/docs/design/io.md`, 5.
- `temper/docs/plan/debt-28/notes/common.md`.

## Owns and may change

Owns the files in the header, and no other. `temper-client-shell-cli/src/host.rs` is another draft's (`test-speed/ctl-stopped`) and calls `directory.complete(complete)`: that's why `Directory` keeps its signature, and `complete` gives `Time::ZERO`.

## Steps

The recipe. No new test: the note's move changes no behaviour.

`heavy cargo nextest run -p temper-box -p temper-client-shell-cli -p temper-github-app`, and `heavy cargo nextest run -p temper-service -E 'test(pre_settings)'`, checking the count isn't 0. `temper/src/fake/check.rs` is run by `temper`'s tests: `heavy cargo check -p temper --tests`, and `heavy cargo nextest run -p temper -E 'test(check)'` if the count isn't 0. `cargo fmt`; `(cd temper && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy` naming each, `--all-targets -- -D warnings`; `python3 -I -B tools/citations/citations.py --all`. Commit, push.

## Report

As your text says, with the search's reading and the net lines.
