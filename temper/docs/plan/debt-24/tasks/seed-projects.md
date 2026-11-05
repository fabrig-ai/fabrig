+++
id = "seed-projects"
part = "front-identities"
kind = "feature"
needs = ["protocol-parties"]
files = ["jig/tests/client/src/harness/seed.rs", "jig/tests/client/src/scripted.rs", "jig/tests/client/src/scripted/stories.rs", "jig/tests/client/src/stories.rs", "jig/tests/client/src/stories/shared/**", "jig/tests/client/src/joint/front.rs", "jig/tests/client-protocol/src/front.rs", "jig/tests/client-protocol/tests/suite/shared/a.rs", "jig/docs/development/debt/D029-protocol-front-one-identity.md"]
effort = "high"
minutes = 40
+++

# `Seed` places a story on two projects, and D029 goes

## Goal

The item's second half. Every `Seed` is on project 1, because a front's projects are configuration fixed when it's built. One shared story now runs on two, on all three fronts that seed.

Acceptance:

- `Seed::TwoProjects { party: u64 }`, which rebuilds the front with two projects, as `Seed::Roles` rebuilds it. Its arms: `seed.rs`'s match; `scripted.rs` with a two-project frame (`scripted/stories.rs`); the Joint front and the protocol front from `jig_core_world::fixtures::two_projects`, or the same shape.
- One new story in `stories/shared/`, run on all three (`a_tests.rs`, the Joint suite, `client-protocol/tests/suite/shared/a.rs`): a person in two projects sees each named in the frame, and a move to the second reads the second's name. `joint/session/tests/project_names.rs`'s `the_frame_names_each_project` shows the words. **It must bite:** build the front with one project, and watch it fail on all three.
- `rg -o '#\[ignore' jig/tests/client-protocol | wc -l` reads 0, and `jig/docs/development/debt/D029-protocol-front-one-identity.md` is deleted. `python3 -I -B tools/citations/citations.py --all` reads 0, and `python3 -I -B tools/debt/debt.py | rg 'D029'` shows no jig row.
- `heavy cargo nextest run -p jig-client-protocol-world` and `-p jig-client-world` pass, with no other story's assertion changed.

## Inputs

- `temper/docs/plan/debt-24/notes/common.md`
- `jig/docs/development/debt/D029-protocol-front-one-identity.md`
- `jig/tests/client/src/joint/session/tests/project_names.rs`
- `jig/tests/core/src/fixtures.rs`, `two_projects`

## Owns and may change

- **Owns:** the files in the header.
- **Other fronts** (`ops`, temper, the referee's and navigation's stubs) take the default `seed`, which panics: no arm is needed. The story isn't registered on them.
- **`ChatProject` stays as it is,** asserting project 1.

## Steps

1. The variant and its arms; the story, on Scripted first, then Joint, then Protocol.
2. The break; the deletion; the checks.
3. `cargo fmt`; `(cd jig && python3 -I -B scripts/old_forms.py)`; `heavy cargo clippy -p jig-client-protocol-world -p jig-client-world --all-targets -- -D warnings`; the tests above. Commit and push.

## Report

The common note's items; the break's failure on each front. The story is newly held, so its lines are named as such.
