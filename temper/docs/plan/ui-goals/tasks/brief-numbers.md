+++
id = "brief-numbers"
part = "brief-numbers"
kind = "feature"
needs = []
files = [
  "jig/crates/jig-domain-core/src/runs/brief_text.rs",
  "jig/crates/jig-domain-core/src/tests/runs/brief_revisions.rs",
]
effort = "medium"
minutes = 30
+++

# A brief names its task

## Goal

A run can name its own task and project in a note's scope: no brief says them today, so a real model has to
guess. One line in the brief's task section.

Acceptance:

- A run's brief's task section holds `[This task: N, in project P]`, its own line before `[Requested by …]`
  (`temper/docs/plan/ui-goals/notes/contracts.md`, A brief names its task), for every task kind and requester.
- `a_briefs_task_section_names_its_number_and_project` passes, bitten once by hand.
- `jig/docs/design/domain/engine.status.md`'s *Not yet* for section 9's numbers is gone.

## Inputs

- `temper/docs/plan/ui-goals/notes/common.md`, `temper/docs/plan/ui-goals/notes/contracts.md`.
- `jig/docs/design/domain/engine.md`, 9.

## Owns and may change

Owns, as `files` lists them. `tasks::RunContext` has `task`; if it lacks the project, add `project` to it, and
to what fills it, each edited only for that (`jig-domain-tasks/src/boundary.rs`, the file that makes it).

Readers it may change, only for the new line: `temper/tests/real-loop/src/case/{oracle.rs,scripts.rs}`,
`temper/tests/engine/domain/tests/suite/walking.rs`, `temper/tests/engine/domain/src/walking_referee.rs`,
`jig/tests/client/src/stories/shared/b_proposals.rs`, `temper/crates/temper-playground/src/tests/stories.rs`,
`temper/crates/temper-fake/src/tests/demo.rs`, a brief's expected text wherever a test pins it whole.

Options, each pre-approved:

- If `task_read`'s `parts` allows too few for one more part, write the line in the same part as the requester's,
  or raise the count it asks; your call, and say which.
- A fake model's cue on `[Requested by` still matches: don't change it.

## Steps

1. **`task_read`:** the line, within its `bytes` as its neighbours are; cut as they are when there's no room.
2. **The test,** beside `brief_revisions.rs`'s neighbours.
3. **A change of meaning:** every brief grows a line. Run its readers:
   `heavy cargo nextest run -p jig-domain-core -p temper-engine-domain-world -p temper-playground -p temper-fake`,
   and read any failure for a whole brief or a measured byte count; fix those in place.
4. **Check,** the implementer's text, then push.

## Report

As `notes/common.md`, Your report.
