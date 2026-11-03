+++
id = "system-running-revision"
part = "running-revision"
kind = "feature"
needs = []
files = ["temper/tests/system/tests/suite/running_revision.rs"]
effort = "high"
minutes = 45
+++

# temper: the system world shows a run keeping its revision, a chat moving, an advance

## Goal

Each run reads its charter at its task's revision (jig's `jig/docs/design/domain/engine.md`,
17.3 and 17.4; temper's `temper/docs/design/domain/agent.md`, 4.1 and 4.6). That is built:
jig prepares each run's `RunCharter` from the task's revision and temper's root and
`temper-engine-smith` read the charter only from the assignment. What is missing is the proof
that a run *keeps* it: today's system-world story shows only the task row's revision before
the advance. Make its one story (`temper/docs/design/client/tools.md`, 11.4, as this plan
revised it; `temper/docs/design/domain/testing.md`, 6.4's system-world row) show a run on the
old revision, a conversation moving at its next run, and the owner's advance.

Acceptance, in `temper/tests/system/tests/suite/running_revision.rs`, one story, renamed
`running_work_keeps_its_revision_a_chat_moves_and_an_advance_moves_the_rest`:

- Two parked agent tasks on revision 1, as today, but the first made with
  `flags.conversation = false` (its batch member's flags edited after `root.chat_ask`, as
  `direct.rs`'s `chat_ending` edits `ended_by`), the second a chat as today
  (`conversation: true`). Both park awaiting input.
- A change of charter 1's instructions with `Running::Stays` makes revision 2; both task rows
  still record `settings.task == 1`.
- The first task woken (a message, then its timer, as the story wakes tasks today): its next
  assignment's `run.revision` is 1, `run.policy.instructions` are revision 1's, and the fake
  model's system prompt holds revision 1's instructions and not `NEW_INSTRUCTIONS`; its row's
  `settings.task` stays 1.
- The second task woken: it moves at its next run (`run.rs`'s conversation move): its next
  assignment's `run.revision` is 2, its instructions and the fake model's system prompt hold
  `NEW_INSTRUCTIONS`; its row records 2.
- `people::Ask::MoveToNewest { revision: 1 }`: the first task's row records 2, the answer
  counts one task moved (assert the counts the answer carries, however the driver exposes
  them); its next run's `run.revision` is 2 and smith's system prompt holds `NEW_INSTRUCTIONS`.
- The story keeps its 2 s aim (a world story's, `testing.md`, 6.1): its time in the report.
  `heavy cargo nextest run -p temper-system-world --test suite -E 'test(/running_revision::/)'`
  passes, its count in the report (check the test binary's name in
  `temper/tests/system/Cargo.toml`), then the package's focused tests.

## Inputs

- `temper/docs/plan/configuring-p1/notes/common.md`.
- jig's `jig/docs/design/domain/engine.md`, 17.3 (a run records the revision it is prepared
  under; what it is given is that revision's) and 17.4 (a conversation moves at its next run;
  `MoveToNewest`); `temper/docs/design/client/tools.md`, 11.4 (the story);
  `temper/docs/design/domain/testing.md`, 6.4 (the system world's row) and 6.1 (a story's
  aim).
- Where it is built, for reading: `jig/crates/jig-domain-tasks/src/run.rs`, `prepare` (the
  conversation move, then `settings.run = settings.task`);
  `jig/crates/jig-domain-core/src/routing/run/preparation.rs` (`RunCharter.revision`);
  `temper/crates/temper-engine-smith/src/start.rs`, `charter_with_history` (smith's charter
  from `assignment.run` alone).

## Owns and may change

- Owns `temper/tests/system/tests/suite/running_revision.rs`.
- Read only: `temper/tests/engine/domain/src/` (the `direct::Driver` and its helpers) and
  `temper/tests/system/src/` (the world). Pre-approved: if a helper the story needs is
  missing there (an answer's counts, a task's flags at creation), build it inside the story
  file from what the driver exposes, and say so; adding to either crate is a question.
- No code outside the test changes: if the first task does not keep revision 1 at its run, or
  the chat does not move, stop and report it as `Question:` with the assignment's `run.revision`
  and the task row you saw; never bend the assertions to fit.

## Steps

1. Rework the story as above; run it alone.
2. `cargo fmt`; `heavy cargo clippy -p temper-system-world --all-targets -- -D warnings`;
   `heavy cargo nextest run -p temper-system-world`; the citations check.
3. Commit (why: the system world proves a run keeps its revision, not only its task row);
   push your branch.

## Report

As the implementer's text says, and also: the story's time, the answer's counts as asserted,
and the test count.
