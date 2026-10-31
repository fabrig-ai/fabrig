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

## Answers

2026-10-31. Does the face keep its two-slot gate, or take the shared
stage's per-entry-point gate? The shared stage's. The face's `room() < 2`
is its private copy of the kernel-edge check that
`skein/docs/design/draft/component-kit.md, 3.1` moves into
`skein_io::stage` (`io::MAX_OUT_*`), a uniform upper bound of the four
`MAX_OUT`s, not a behaviour of the face's own: no peer depends on it, and
the order of events and submissions is unchanged. The exact bound is the
kit's contract (3.3: every site checks the kernel's submission room before
taking work), so this is the migration, not a change of behaviour under
`notes/common.md`. Add no guard beside the call. The deleted test's
blocked assertion is io's gate, covered by tests/io's
`resume_waits_for_its_submission_room_and_keeps_later_passes_queued` (no
room) and `completions_wait_for_their_submission_room_after_ready_results`
(one slot): name both in the report, saying the face now resumes at one
free slot, as every site on the stage does. Keep the `forwarding_host`
recount (8 to 6) that `old_forms.py --write` gives: fab recounts where
merges meet. Commit and push the branch as it stands.
