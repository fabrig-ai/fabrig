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

## Answers

### 2026-11-05: Who repairs `roles_two_parties_decide_one_proposal`, this task or `protocol-parties`? (debt-24/seed-projects-1, foreman debt-24/seed-projects/foreman-1)

`protocol-parties`: it's that task's acceptance, and its spec now holds fabrig/debt's answer (either winner, both clients naming the same one, in `confirm_two_parties`). Don't change that body here, though `stories/shared/**` is in this header. This task's own work stands on the pushed branch (`bdd54520d0`), with its break already reported.

The next run merges `origin/debt-24/protocol-parties` first, keeping the `#[ignore]` removals where both branches made them, and runs what the last one didn't: both packages' whole suites and the checks after them. If that branch doesn't yet hold the repair, this one failure is known and not this task's: don't change it or ask about it again; report it under `## Left`, naming `protocol-parties`, with every other test passing.

## Rebase (2026-11-05, step merge, foreman debt-24/seed-projects/foreman-2)

Rebase onto the landing trunk, preserving both seed paths and adopting jig-core-kit.
The merge refused content conflicts in `jig/tests/client-protocol/src/front.rs` and `jig/tests/client/src/joint/role_fixture.rs`.
The overlapping landing changes are `397ff41ae4` (shared role setup and protocol identities) and `91cdbfa3f4` (core harness moved into jig-core-kit).
In protocol `Front::seed`, retain both `TwoProjects` and `Roles` branches, including their returns; preserve role-specific construction and identity minting.
In `role_fixture.rs`, retain the shared `setup`/`install` refactor while using `jig_core_kit` for `Peers`, `peers::fixture`, `peers::journal`, and `fixtures::ask_as`.
Carry the kit migration into this task's new code: change `a_projects::setup` in `jig/tests/client/src/stories/shared/a_projects.rs` to call `jig_core_kit::fixtures::two_projects`.
Also update the new `jig_core_world::peers::Peers::new` call in `jig/tests/client/src/joint/front.rs` to `jig_core_kit`; retain the landing's other migrated references and dependencies.
Keep the ignore removals, D029 deletion, project story registrations, and protocol-parties' either-winner assertions.
After resolution, run the spec's focused project story on all three fronts, both complete package suites, and the remaining prescribed checks through the required tools.
