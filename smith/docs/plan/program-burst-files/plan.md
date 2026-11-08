# program-burst-files: plan

A program-backed main that asks several `sub_agent` calls in one message aborted the agent at `smith-protocol-program/src/backend/files.rs:325` (`one pending private file request`): a private file answer reached a program backend with nothing outstanding. The request ranks three wants: the failing rows, the fix, the design.

## Design

Written by this cut: `smith/docs/design/protocol/programs.md`, 5.1, the bullets *Each program's files are its own* (each program's files in its own directory, each answer to the program that asked) and *An answer not asked for* (fails that program's run with a reason; dropped once it is ending). `programs.status.md` carries their **Not yet** entries; each task deletes its own.

## Parts and tasks

- **`unasked`** (slice 1): `unasked-answer`. The backend's own guard: `file_event` and `native_event` fail the run on an unasked answer, drop it once ending, never panic. Needs nothing; released first. Its rows are unit rows of the backend (`backend/files/tests.rs`).
- **The routing and the burst rows:** to follow, once the survey of which answer is misdelivered reports (a child's answer routed to main by a shared owner token, a second answer for one request, or one after the backend ended).

## Critical path

`unasked-answer` (45 min) now; the burst rows and routing fix after it.

## Overlaps with drafts beside this one

None: the other drafts in flight change temper's and jig's files only; this plan stays in smith.
