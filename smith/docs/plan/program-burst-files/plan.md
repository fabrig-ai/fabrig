# program-burst-files: plan

A program-backed main that asks several `sub_agent` calls in one message aborted the agent at `smith-protocol-program/src/backend/files.rs:325` (`one pending private file request`): a private file answer reached a program backend with nothing outstanding. The request ranks three wants: the failing rows, the fix, the design.

## Design

Written by this cut: `smith/docs/design/protocol/programs.md`, 5.1, the bullets *Each program's files are its own* (each program's files in its own directory, each answer to the program that asked) and *An answer not asked for* (fails that program's run with a reason; dropped once it is ending). `programs.status.md` carries their **Not yet** entries; each task deletes its own.

## Parts and tasks

- **`unasked`** (slice 1): `unasked-answer`. The backend's own guard: `file_event` and `native_event` fail the run on an unasked answer, drop it once ending, never panic. Needs nothing; released first. Its rows are unit rows of the backend (`backend/files/tests.rs`).
- **`routing`** (slice 2): `file-owners`. The cause, found by a read-only survey and checked by reading: `program::file_event` (`smith-agent-service/src/service/program.rs`) hands a file answer to main or a child whenever `Backend::owns` matches its owner, pipes and the status check's pipes included. Pipe tokens are io's `Id` tokens, of the same form as the domain's owners for its tools' file operations, which reach the shared file component unchanged; a tool's answer whose owner equals a live pipe token reaches a program with nothing outstanding. Of the request's three suspects it is the first, a shared owner token, but shared with a tool's operation, not a child's. A second answer for one request and an answer after the backend ended are ruled out by reading (`skein-io`'s file layer ends each request once; a backend ends only from `cleanup`, which waits for its outstanding operation). The task's failing rows are at the lowest tier holding both the backend and the routing, the service's own tests, with 2, 3 and 5 children and the stray answer delivered before, between and after the children's directory answers; the fix routes by a new `Backend::owns_file` (Files and Native owners). Needs nothing: it runs beside `unasked-answer`.

## Decisions

- **The request's "every order the file component may deliver them"** is the rows' `delivered` column at the service tier, not a seed or latency sweep of the agent-process world: the cause is a token's identity, not an order, and a sweep there would not bite. The world gains the request's burst of three as two rows of its existing table (`a_program_main_reads_a_burst_past_its_room`), as coverage; the bite is the service's table.
- **Two tasks, not three:** the request's three items are the guard (`unasked-answer`), the rows and routing (`file-owners`) and the design, which this cut wrote. Nothing was dropped.
- **Status entries:** `run.status.md`, 5.3 and `limits.status.md`, 6 said the burst abort was unexplained; they now point at `programs.status.md`, 5.1, and `file-owners` deletes them with its own entry.

## Critical path

`unasked-answer` (45 min) and `file-owners` (75 min) run side by side; the plan lands when both have merged.

## Overlaps

- **Within the plan:** both tasks touch `smith-protocol-program/src/backend.rs` (`unasked-answer` may change `from_io`'s call of `native_event`; `file-owners` adds `owns_file` beside `owns`) and `programs.status.md` (each deletes its own entry): distinct lines, each spec says so.
- **With the drafts beside this one:** none; they change temper's and jig's files, this plan smith's.
