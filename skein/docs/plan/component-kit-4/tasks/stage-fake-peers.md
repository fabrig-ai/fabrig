+++
id = "stage-fake-peers"
part = "stage-sites"
kind = "migration"
needs = ["io-stage", "counts"]
files = ["skein/crates/skein-fake-peers/**", "skein/tests/fake-peers/tests/suite/stages.rs", "skein/tests/fake-peers/tests/suite/main.rs"]
effort = "high"
minutes = 40
+++

## Goal

The fake peers' shared face (`skein-fake-peers`, `src/face.rs`) runs its io
stage through `skein_io::stage`, and the world test that `#[path]`-included
its private source goes, covered by tests/io's stage test
(`skein/docs/design/draft/component-kit.md, 3.3`).

Acceptance:

- `Face::up` and `Face::down` call `skein_io::stage::up` and `down`; their
  public signatures stay.
- `tests/fake-peers/tests/suite/stages.rs` and its `mod` line are deleted:
  name in the report which assertion of tests/io's `tests/suite/stage.rs`
  covers each of its assertions. If one is not covered (it is about the
  face, not io's stage), keep that test, rewritten against the face's
  public surface without `#[path]`.
- The `io_resume` count falls by 1.
- `skein-fake-peers` and `skein-fake-peers-tests` pass their tests, and
  every package that depends on `skein-fake-peers` checks (`heavy cargo
  check --tests -p <each>`; find them with `cargo tree -i skein-fake-peers
  --workspace -e normal,dev --depth 1`).

## Inputs

- `skein/docs/plan/component-kit-4/notes/common.md` and `notes/migrations.md`
  (the io stage).
- `skein/crates/skein-io/src/stage.rs`; `skein/tests/io/tests/suite/stage.rs`.
- `crates/skein-fake-peers/src/face.rs`; `tests/fake-peers/tests/suite/stages.rs`.

## Owns and may change

- Owns `skein/crates/skein-fake-peers/`, and in `skein/tests/fake-peers/`
  only `tests/suite/stages.rs` and the suite's `mod` list.

## Steps

1. Move `Face::up`/`down` onto the stage (set the env's `now` and `wall`
   from the arguments first).
2. Delete or rewrite `stages.rs` as the Goal says.
3. `cargo fmt`; `heavy cargo clippy -p skein-fake-peers -p
   skein-fake-peers-tests --all-targets -- -D warnings`; `heavy cargo
   nextest run -p skein-fake-peers -p skein-fake-peers-tests`; the checks
   of the dependents.
4. `old_forms.py --write`; commit and push.

## Report

As common.md says.
