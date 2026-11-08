+++
id = "copy-ctl"
part = "copies"
kind = "feature"
needs = []
files = ["temper/tests/one-process/src/copy.rs", "temper/tests/one-process/tests/suite/copy.rs"]
effort = "high"
minutes = 100
+++

# A copy beside a repository joins its project with the source's workflow kept, in the one-process world

## Goal

The process walk `copy::a_copy_is_made_and_adopted_through_ctl_on_the_fake_box`
(`temper/crates/temper/tests/fake_backends/walk/copy.rs`) is not covered
by `temper/docs/design/domain/forge.md`, 4.5's stories: they hold the
copy's rows, its act and its recovery, but none a copy joining a project
that already holds a repository, the copy's provider identity in that
project, `home` false on the finished resource, or the source's own
workflow kept beside the starter. The request named the joint world; it
has no setup driver (its copies stop at Waiting, and making them there
would inject `MakingMade`, a stub of what the walk proves). The
one-process world (`temper/tests/one-process`, package
`temper-one-process-world`) runs the service's copies through ctl, as
`a_restart_mid_copy_finishes_and_adopts_once` does: the story goes there,
beside it. The walk itself is `walks-go`'s; you never touch it.

Acceptance:
**`a_copy_beside_a_repository_joins_its_project_keeping_the_sources_workflow`**,
new, in `temper/tests/one-process/tests/suite/copy.rs`, its world in
`temper/tests/one-process/src/copy.rs` beside `restart` and `cut_short`:

1. The catalogue source carries, beside its README, a
   `.github/workflows/ci.yml` of its own bytes.
2. Through ctl, as the walk: the forge is connected with a first
   repository adopted into project 1 (`--first`) and a runner label
   (`--label copy-runner`); then `repository copy --project 1 --forge 1
   SOURCE --as OWNER/NAME --no-home --until adopted --json` answers
   adopted.
3. `forge list --json` holds one forge and no making rows.
4. Project 1 (`project show 1 --json`, as ctl reads it) holds two
   resources; the copy's, found by its forge and the provider id the
   forge gives the copied repository, is owned and not home.
5. On the forge: the copy's default branch is the source's; its head is
   one commit whose parent is the source's head; that commit's
   `.forgejo/workflows/ci.yml` is the starter for the forge's runner
   label (the expected bytes from the starter's own function, never
   written by hand); its `.github/workflows/ci.yml` is the source's,
   unchanged.

- **Bite:** make the copy keep `home` true (drop `--no-home`) or drop the
  source's workflow from the expected tree by hand, see step 4 or 5 fail
  for it, restore.
- `restart` and `cut_short` unchanged in what they check: share the
  world's `run` with a parameter or a sibling function.
- Clippy clean, fmt clean, `temper-one-process-world`'s focused suite
  green.

## Inputs

- `temper/docs/plan/walks-lower/notes/common.md` (read first).
- `temper/docs/plan/walks-lower/inputs/request.md`, the
  `copy::a_copy_is_made_and_adopted_through_ctl…` ruling.
- `temper/docs/design/domain/testing.md`, 2.5 (the one-process world)
  and 2.6, the bullet **The fake box keeps two walks**, its **Copies made
  and adopted** item.
- `temper/docs/design/domain/forge.md`, 4.5 (*A repository made or
  copied*).
- The walk (read only), for its calls and assertions:
  `copy.rs`'s `a_copy_is_made_and_adopted_through_ctl_on_the_fake_box`
  and `assert_copy`.

What the survey found, for your bearings (verify before relying):

- `src/copy.rs`: `source()` seeds `acme/app` on `temper_fake_git_hosted`
  with only a README; `Judge::caller` signs in, connects the forge and
  copies; the referee checks one adopted repository, owned, its default
  branch, and the copy's parent.
- The starter's bytes and its label: `temper-service`'s
  `setup_making.rs` tests `a_copy_runs_on_a_connected_forge_and_is_made`
  and `the_label_is_the_records_or_the_modes`
  (`temper/crates/temper-service/src/tests/setup_making.rs`) name the
  function that writes it.
- `"adopted": true` in ctl's answer only says the making row went
  (`temper/crates/temper-client-tools/src/making.rs`): the project's
  resources are the proof.

## Owns and may change

Owns:

- `temper/tests/one-process/src/copy.rs`: the source variant with its
  workflow, the ctl calls, and the new referee checks, beside the
  existing ones.
- `temper/tests/one-process/tests/suite/copy.rs`: the new story.

Reader and neighbour files (edits allowed as stated):

- `temper/tests/one-process/src/lib.rs`: a re-export, if needed.

Pre-approved options:

- **The first repository:** if `--first` can't adopt one in this world,
  adopt it with `repository adopt` before the copy, and say so.
- **The label:** if `--label` isn't accepted on this forge's mode, take
  the label the record has and expect its starter; say so.
- **A check the world can't make without a stub** (for instance the
  forge's compare route): read the same fact from the fake forge's
  domain, as the existing referee reads the copy's parent, and say so.

## Steps

1. Read the inputs and `src/copy.rs` whole.
2. Write the world's variant and the story; run it alone:
   `heavy cargo nextest run -p temper-one-process-world --test suite -E
   'test(=copy::a_copy_beside_a_repository_joins_its_project_keeping_the_sources_workflow)'`.
3. Bite it as above; restore; run the file's three stories.
4. Once, at the end: `cargo fmt`; `python3 -I -B
   temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-one-process-world --all-targets -- -D warnings`; `heavy cargo
   nextest run -p temper-one-process-world` (check the count). Commit
   (`walks-lower: a copy beside a repository in the one-process world`,
   the body saying why it is the one-process world's and not the joint
   world's) and push.

## Report

As the implementer's text says: the first line; the commit pushed; each
run with its count; the bite and its failure; which of the walk's checks
are read through ctl and which from the fake forge; the story's CPU
seconds; the minutes.
