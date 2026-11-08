+++
id = "file-owners"
part = "routing"
kind = "feature"
needs = []
files = [
  "smith/crates/smith-agent-service/src/service/program.rs",
  "smith/crates/smith-agent-service/src/service/program/tests.rs",
  "smith/crates/smith-protocol-program/src/backend.rs",
  "smith/tests/agent-process/tests/suite/program_children.rs",
  "smith/docs/design/protocol/programs.status.md",
  "smith/docs/design/domain/run.status.md",
  "smith/docs/design/protocol/limits.status.md",
]
effort = "high"
minutes = 75
+++

# The agent routes a file answer to a program by its file owners alone

## Goal

A program-backed main that asked several `sub_agent` calls in one message aborted the agent at `smith-protocol-program/src/backend/files.rs:325` (`one pending private file request`). The cause, found by reading: the agent service's `program::file_event` (`smith-agent-service/src/service/program.rs`) hands a file answer to main or a child whenever `Backend::owns` matches its owner, and `owns` matches the program's process pipes and its status check's pipes too. Those are io tokens (`skein_lib::Id::token`: slot in the high 32 bits, generation in the low), of the same form as the domain's owners for its tools' file operations (`smith-domain-tools/src/flight.rs`, `id.token()`), which reach the shared file component unchanged (`service/down.rs`). A tool's file answer whose owner equals a live pipe token goes to that program, which has nothing outstanding, and the machine loses its answer. This task proves it with failing rows, then routes a file answer to a program only by that program's own file owners (`smith/docs/design/protocol/programs.md`, 5.1, *Each program's files are its own*).

Acceptance:

1. **The rows,** one table test `a_tools_file_answer_never_reaches_a_program` in `smith/crates/smith-agent-service/src/service/program/tests.rs`. Each row builds a service with `conversations` set to its children plus one (as `child_service` does for one child), starts main (`complete` with a main `Asker::Main` request) and its children (`complete` with `child_request`, each child's directory open answered by `file_event` with `file::Event::Opened` for a root adopted with `service.stages.files.adopt_root`), and answers each program's status-check spawn with `io_event` and `io::Event::Spawned`, its pipes tokens of `Id`'s form (`Token::new((slot << 32) | generation)`). Then it delivers, at the row's point, a tool's file answer whose owner equals one of a program's check pipes:

   | children | whose pipe | answer | delivered |
   |---|---|---|---|
   | 2 | main's | `Closed` | after every check spawned |
   | 2 | child 0's | `Stored` | after every check spawned |
   | 3 | child 1's | `Opened` | after every check spawned |
   | 3 | main's | `Failed { error: NotFound, .. }` | before any child's directory answers |
   | 5 | child 4's | `Removed` | after every check spawned |
   | 5 | child 0's | `Closed` | after child 0's check spawned, before children 1 to 4's directories answer |
   | 5 | main's | `Loaded` | after children 0 to 2's checks spawned, before 3 and 4's directories answer |

   Each row asserts: `file_event` returns `Some` of the same event (it goes on to the machine); no `llm::ToDomain::Failed` is pushed; and every child whose directory answers after it still reaches its check (its `Spawn` appears in `program.below`).
2. **It fails first:** run on the task's base, the first row fails: if `unasked-answer` has not merged, main panics with `one pending private file request`; if it has, `file_event` returns `None` and main's completion fails with `smith: a private file answer the program did not ask for`. The report quotes which (`notes/common.md`).
3. **The routing,** as sketched below; the table passes.
4. **The request's story, at the world tier:** rows `(3, false, 3, 0, 3)` and `(3, true, 3, 0, 0)` in `a_program_main_reads_a_burst_past_its_room`'s table (`smith/tests/agent-process/tests/suite/program_children.rs`), beside its 2, 4, 5 and 6 rows. They pass on the base as on the branch: they cover a burst of three, and the rows of acceptance 1 are the bite. Nothing else of the world changes.
5. **Status:** delete the **Not yet** entry `5.1: an answer's program is told by any token the program owns` from `smith/docs/design/protocol/programs.status.md`, the entry `5.3: the reported deployment abort on several sub_agent calls` from `smith/docs/design/domain/run.status.md` and the entry `6: the reported deployment abort on several sub_agent calls` from `smith/docs/design/protocol/limits.status.md`; no other design line.

## Inputs

- `smith/docs/plan/program-burst-files/notes/common.md`.
- `smith/docs/plan/program-burst-files/inputs/request.md`, items 1 and 2.
- `smith/docs/design/protocol/programs.md`, 5.1: *Each program's files are its own* and *An answer not asked for* (written by this cut); 5.10, *Each child a program of its own*.
- `smith/crates/smith-agent-service/src/service/program.rs`: `FIRST`, `STRIDE`, `Child::directory_owner`, `start_child`, `Program::io_index`, `Program::route_io`, `file_event`, `io_event`, `complete`; `service/up.rs` (the file answers' loop: `program::file_event`, then the machine); `service/down.rs` (the machine's file requests, owners unchanged).
- `smith/crates/smith-protocol-program/src/backend.rs`: `owns`, `token`, `Owner`, `from_io`; `src/command.rs`, `Command::owns` (the check's pipes).
- `smith/crates/smith-agent-service/src/service/program/tests.rs`: `child_service`, `child_request`, `a_closing_child_holds_its_slot_until_the_directory_terminal` (the idiom).
- `skein/crates/skein-lib/src/id.rs`, `Id::token`; `skein/crates/skein-io/src/process.rs` (`Spawned`'s pipe tokens).

## Owns and may change

Owns `program::file_event`'s routing and a new `Program::file_index` beside `io_index`; the new `Backend::owns_file`; the table of acceptance 1; the two rows of acceptance 4.

Reader and neighbour files, each edited only so:

- `smith/crates/smith-protocol-program/src/backend.rs`: the new `owns_file` beside `owns`, nothing else. `unasked-answer`, running beside this task, may change `from_io`'s call of `native_event` in the same file: leave that line as you find it.
- `smith/docs/design/protocol/programs.status.md`, `domain/run.status.md`, `protocol/limits.status.md`: the three entries of acceptance 5.

Options, each decided:

- **`child_service` takes one child:** add a parameter for `conversations`, or a sibling helper; keep the existing test's call reading as now.
- **A main request:** a `main_request` helper beside `child_request`, or a parameter of it for the `Asker`; either.
- **Where the check's spawn is:** read the backend's `Spawn` (or `SpawnAlone`) from `program.below` to learn its owner, and answer it with `io_event`. If main's or a child's check does not spawn from the state the row reaches, use the nearest io token that program's `owns` matches (its process pipes once spawned) and say so in the report.
- **A token in the private range no program owns** (`FIRST` upward, not a child's directory open nor a backend's `owns_file`): it goes on to the machine, as today. Add no assertion for it.
- **`io_index` and `route_io`** stay for io answers; if `file_event` no longer calls them, they stay as they are.

## Steps

The routing's shape (names inside are yours, the two public ones are binding):

```rust
// smith-protocol-program/src/backend.rs
/// Whether a file answer with this owner answers one of this backend's own file operations (5.1).
#[must_use]
pub fn owns_file(&self, owner: Token) -> bool {
    owner == self.token(Owner::Files) || owner == self.token(Owner::Native)
}

// smith-agent-service/src/service/program.rs, file_event, after the opening children
match program.file_index(owner) {
    Some(index) => { /* that child's backend.from_io(BelowEvent::File { event, descriptor }) */ None }
    None if program.backend.owns_file(owner) => { /* main's backend.from_io */ None }
    None => Some(event),
}
```

1. Write the table and run it alone; see the first row fail (acceptance 2):
   `heavy cargo nextest run -p smith-agent-service --lib -E 'test(=service::program::tests::a_tools_file_answer_never_reaches_a_program)'`.
2. Add `owns_file`, `file_index` and the routing; the same command passes, one test selected.
3. Add the world rows (acceptance 4) and run that test alone:
   `heavy cargo nextest run -p smith-agent-process-world --test suite -E 'test(=program_children::a_program_main_reads_a_burst_past_its_room)'`.
4. Delete the status entries (acceptance 5).
5. `cargo fmt`; `python3 -I -B smith/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main`; `heavy cargo clippy -p smith-protocol-program -p smith-agent-service -p smith-agent-process-world --all-targets -- -D warnings`; `heavy cargo nextest run -p smith-agent-service`; `heavy cargo nextest run -p smith-agent-process-world` (every program world: `file_event`'s meaning changes for the answers it routes); `heavy cargo nextest run -p smith-protocol-program`. The other trees' dependents of `smith-protocol-program` gain one method and nothing changes for them; the trunk's end check runs them.
6. Commit the rows and the fix apart (why: a tool's file answer whose owner equals a program's pipe token aborted the agent) and push with `git push origin HEAD:refs/heads/<your branch>`.

## Report

fab's implementer report, with: the failure seen in step 1 and which of acceptance 2's two forms it took, each row's outcome after the fix, the world test's result with its two new rows, the counts of the package runs, and the choices of the options above.
