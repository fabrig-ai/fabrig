+++
id = "copy-machine"
part = "copy-plan"
kind = "feature"
needs = ["copy-surface"]
files = ["skein/crates/skein-forge-setup/src/copy.rs", "skein/crates/skein-forge-setup/src/create.rs", "skein/crates/skein-forge-setup/src/lib.rs", "skein/crates/skein-forge-setup/src/tests/copy.rs", "skein/crates/skein-forge-setup/src/tests.rs"]
effort = "high"
minutes = 60
+++

# The plan `Copy`

## Goal

`Copy` runs as `skein/docs/design/forge-setup.md`, 4.10 says, step by step, each event in and at most
one need out, reading first and writing only what is missing. It is tested on hand-given replies here;
`copy-story` runs it on the fake.

Acceptance, each a test in the new `src/tests/copy.rs`, named for its rule:

- `a_copy_reads_marks_migrates_grants_and_puts_its_files`: an absent repository: the needs are, in
  order, the admin's token, `User`, `Org`, `Repository`, `Write(Copying)`, `Migrate`, `Branch`, then
  the access's and the files', ending `Done(Copied { id, branch })` with the source's branch.
- `a_repository_with_no_mark_is_taken_and_nothing_is_written`: `Read(Copying)` none: stop at
  `Repository`, `Taken`, the subject `org/name`, and no write after the org.
- `a_marked_repository_with_its_branch_is_the_copy_finished`: no `Migrate`, no delete.
- `a_marked_repository_with_no_branch_is_deleted_and_copied_again`, and with the delete's 404.
- `a_migrate_answered_409_is_taken`; `a_migrate_refused_stops_at_repository_with_its_status`.
- `a_source_with_no_branch_stops_at_empty`.
- `a_users_name_stops_at_org`: `Org` 404, `CreateOrg` 422: stop at `Org`, `Refused(422)`, the name.
- `a_file_the_source_has_is_left_as_it_is`.
- `a_halt_at_each_state_stops_at_its_stage_and_drops_the_token`, on `create.rs`'s `halt`.
- `Machine::Copy` runs under the host: `Plan::Copy` is no longer `Unexpected`.
- `heavy cargo nextest run -p skein-forge-setup -p skein-forge-setup-world` pass.

## Inputs

- `temper/docs/plan/repository-copies-1/notes/common.md`; `notes/contracts.md`, S1 and S2.
- `skein/docs/design/forge-setup.md`, 4.2, 4.5, 4.9 and 4.10.

## Owns and may change

- The files of the header; `skein/crates/skein-forge-setup/src/host/run.rs`, `Machine`'s arms and
  `forgejo_machine`'s `Plan::Copy` arm alone.
- **No second copy of `Create`'s body.** `Copy` shares its org, access and file steps with `Create`.
  Inside, the shape is yours: either one machine with a `Source` of `Empty` or `Copy { from }`, in
  `create.rs`, or the steps drawn into a shared helper as `access.rs` shares the bot's access, with
  `copy.rs` holding only what differs. If `create.rs` is reshaped, its tests stay as they are and pass.
- Pre-approved:
  - `Outcome::Copied`'s `id` zero, or an empty branch, is `Unexpected`, as `Create`'s are.
  - After `Migrate`'s `201` the id and branch are `CreatedRepository`'s; the `Branch` read that follows
    is `Empty`'s check.
  - A `DeleteRepository` answered 204 is `Reply::Empty`, 404 `Reply::Status(404)`; each goes on.

## Steps

1. The states of 4.10, step 3, beside `Create`'s. 2. `step` and `halt`, on `create.rs`'s pattern. 3.
`Machine::Copy` and its `step` and `halt` arms. 4. The tests, each biting: skip the `Write(Copying)`, and
treat a read `None` as finished, each once. 5. The implementer's checks; commit and push.

## Report

As the implementer's text asks; which shape you chose for sharing, and why.
