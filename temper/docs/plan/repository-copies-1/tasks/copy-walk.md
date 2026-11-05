+++
id = "copy-walk"
part = "copy-end-to-end"
kind = "feature"
needs = ["making-run", "repository-tools", "fake-box", "copy-story"]
files = ["temper/crates/temper/tests/fake_backends/walk/copy.rs", "temper/crates/temper/tests/fake_backends/walk.rs", "temper/tests/one-process/tests/suite/copy.rs", "temper/tests/one-process/tests/suite/main.rs", "temper/tests/one-process/src/ctl.rs"]
effort = "high"
minutes = 60
+++

# A copy made and adopted, end to end

## Goal

`temper ctl` copies a repository and has it adopted, on the real box and the fake backends, and a restart in
the middle finishes it. This is the plan's slice that a walk then repeats by hand
(`temper/docs/plan/repository-copies-1/walk.md`).

Acceptance:

- `temper/crates/temper/tests/fake_backends/walk/copy.rs`: `a_copy_is_made_and_adopted_through_ctl_on_the_fake_box`:
  `forge connect forgejo-admin ... --first ORG/REPOSITORY`, so a project exists; the source's address
  from the retained `forgejo.json`; `repository copy --no-home --until adopted`; the repository is adopted,
  its default branch the source's, and `.forgejo/workflows/ci.yml` on top of the source's commits,
  read through the fake's API; `.github/workflows/ci.yml` still there.
- `temper/tests/one-process/tests/suite/copy.rs`:
  - `a_restart_mid_copy_finishes_and_adopts_once`: a `Slow` source, the engine stopped under it and
    started again: `copying` set, the plan reruns, the repository is adopted once. This is the story
    `walk.md` leaves out.
  - `a_copy_cut_short_is_deleted_and_copied_again`, by a `Cut` source then a `Whole` one.
- Each runs within its tier's budget; a test over it is the implementer's text's question, not a raise.
- `heavy cargo nextest run -p temper --test process -E 'test(a_copy_is_made)'`; `heavy cargo nextest run -p
  temper-one-process-world -E 'test(copy)'` pass, and `heavy cargo nextest run -p temper-one-process-world`.

## Inputs

- `temper/docs/plan/repository-copies-1/notes/common.md`.
- `temper/docs/plan/repository-copies-1/walk.md`.
- `temper/docs/design/draft/repository-copies.md`, 10; `temper/docs/design/domain/forge.md`, 4.5.
- The frame: `walk/here_key.rs`, 37; `temper/crates/temper/tests/support/forgejo_elsewhere.rs`, 200; and
  `temper/tests/one-process/src/ctl.rs`, 136.

## Owns and may change

- The files of the header. `Walk` reads sandbox files, and `outcome.stderr` comes after the box stops, so read
  addresses from `forgejo.json`, as `add_key` does. `open_default.rs` is `look-round-3`'s.
- `temper/tests/one-process/src/**`: a helper that gives the world's fake a `Source`.
- Pre-approved:
  - A failure a lower tier should show is reproduced there, in the owner's crate, and fixed there, if
    the fix is a line; else `Left`, with its file.
  - If `--first`'s project is not what the walk needs, `forge setup` is not available on a peer: use the
    project `--first` makes.

## Steps

1. The process test. 2. The one-process stories. 3. Each biting: skip the mark's keeping once, and see the
restart's test fail. 4. The implementer's checks; commit and push.

## Report

As the implementer's text asks; how long each test takes, and any finding the walk should look at.
