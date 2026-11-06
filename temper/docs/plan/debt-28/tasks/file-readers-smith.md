+++
id = "file-readers-smith"
part = "file-layer"
kind = "migration"
needs = []
files = ["smith/crates/smith-agent-service/src/service/**", "smith/crates/smith-agent-service/src/service.rs", "smith/crates/smith-agent-service/src/profile/tests.rs", "smith/crates/smith-local-service/src/credentials.rs", "smith/crates/smith-local-service/src/credentials/**", "smith/crates/smith-local-service/src/host_io.rs", "smith/crates/smith-local-service/src/host_io/**", "smith/crates/smith-local-shell/src/compose/{cleanup,files}.rs", "smith/crates/smith-protocol-machine/src/boundary.rs", "smith/crates/smith-local-protocol/tests/suite/disk.rs", "smith/tests/fake-claude/src/fake.rs", "smith/tests/local-process/tests/suite/fuzzy/store.rs", "smith/tests/machine/src/lib.rs", "smith/tests/machine/tests/suite/removal.rs"]
effort = "medium"
minutes = 75
+++

# smith's file readers call `file_layer::step`

## Goal

First half of skein's D049, in smith: every reader moves to `FileIo::with_limits` and `file_layer::step`. The cut counted 47 thin calls (24 in production; 2 doc comments) and 13 constructors. Behaviour doesn't change.

Acceptance:

- `rg -n 'file_layer::(down|down_until|up|expire)\b|FileIo::(new|with_whole_limit)\(' --type rust smith` finds nothing.
- The doc comments in `smith-protocol-machine/src/boundary.rs` read true.
- Net at most +60 lines.

## Inputs

- `temper/docs/plan/debt-28/notes/file-readers.md`, the recipe.
- `skein/docs/design/io.md`, 5.
- `temper/docs/plan/debt-28/notes/common.md`.

## Owns and may change

Owns the files in the header, and `smith/crates/smith-agent-service/src/stages.rs`, where `io_env` is set, for one field. Nothing beyond the file layer's use moves.

Where the thought is:

- `smith-agent-service`'s `up`, `down`, `roots` and `program` have `io_env` on the struct, set in `stages.rs`: a file env beside it.
- `smith-local-service`'s `HostIo::at(now, wall)` (`host_io.rs`) already builds an `Env` for each iteration: follow it. `split` and `credentials.rs`'s `complete` hold no clock.
- `smith-local-shell/src/compose/files.rs` builds a `FileIo` and hands it to `ResultFile::new`, so its limits travel with it.
- `smith/tests/machine`'s `pass` has `machine_env`.

`smith/benchmarks/` and `smith/docs/design/**` are read by tests, and this task has no reason to touch them.

## Steps

The recipe. No new test: the note's move changes no behaviour.

`heavy cargo nextest run -p smith-agent-service -p smith-local-service -p smith-local-shell -p smith-protocol-machine -p smith-local-protocol`, then the world packages for `smith/tests/fake-claude`, `smith/tests/local-process` and `smith/tests/machine` (`cargo metadata` names them). `cargo fmt`; `(cd smith && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy` naming each, `--all-targets -- -D warnings`; `python3 -I -B tools/citations/citations.py --all`. Commit, push.

## Report

As your text says, with the search's reading and the net lines.

## Answers

2026-11-06. How do the recipe's `file_layer::Limits` literals pass the old-forms ratchet (`full_limits` 116 against 104)? By the targeted exemption, already on the trunk with this answer: the shared `full_limits` (`tools/old_forms/old_forms.py`) skips a literal spelled `file_layer::Limits`, because `skein/docs/design/io.md`, 5, makes that literal the new form. No baseline moves, and no constructor is added. Keep your literals spelled `file_layer::Limits`, never a local `Limits` import. Merge the trunk, see `(cd smith && python3 -I -B scripts/old_forms.py)` pass at 104, then run the focused tests your report left pending for the packages that depend on what you changed, and finish.
