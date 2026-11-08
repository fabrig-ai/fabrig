+++
id = "unasked-answer"
part = "unasked"
kind = "feature"
needs = []
files = [
  "smith/crates/smith-protocol-program/src/backend/files.rs",
  "smith/crates/smith-protocol-program/src/backend/native.rs",
  "smith/crates/smith-protocol-program/src/backend/files/tests.rs",
  "smith/crates/smith-protocol-program/src/backend/builtin_tests.rs",
  "smith/docs/design/protocol/programs.status.md",
]
effort = "high"
minutes = 45
+++

# A program backend fails its run on a file answer it did not ask for

## Goal

A program backend aborts the agent process when a file answer reaches it with nothing outstanding: `Backend::file_event` takes `self.files.pending` with `expect("one pending private file request")`, and `Backend::native_event` takes `self.native` with `expect("one native file terminal")`. A deployment hit the first with several `sub_agent` children on program backends. This task makes such an answer fail that backend's run with a reason, or drop it once the program is ending, never a panic (`smith/docs/design/protocol/programs.md`, 5.1, *An answer not asked for*). Which answer was misdelivered, and the routing that delivers each answer to the backend that asked, is a later task of this plan; this one is the backend's own guard.

Acceptance:

1. **The rows,** one table test `an_answer_not_asked_for_fails_its_run_never_the_agent` in `smith/crates/smith-protocol-program/src/backend/files/tests.rs`, each row a backend fixture, an owner (`Owner::Files` or `Owner::Native`) and a `file::Event` delivered through `Backend::from_io` as `BelowEvent::File` with nothing of that owner outstanding (`files.pending` is `None`; `native` is `None`, or `Some` and not yet `submitted`):

   | fixture | owner | event | expected |
   |---|---|---|---|
   | `fixtures::starting()` | Files | `Stored` | failed |
   | `fixtures::active()` | Files | `Opened` (with a descriptor) | failed |
   | `fixtures::active()` | Files | `Failed { error: NotFound, .. }` | failed |
   | `fixtures::running_child()` with a claimed completion | Files | `Closed` | failed |
   | `fixtures::active()` | Native | `Loaded` | failed |
   | `fixtures::active()` after `Backend::close` (phase `Ending`) | Files | `Removed` | dropped |
   | `fixtures::active()` after `Backend::close` (phase `Ending`) | Native | `Loaded` | dropped |

   *failed*: `up` holds exactly one `ToDomain::Failed` for the fixture's completion owner, `failure: llm::Failure::Program { exit: None }`, its `detail` containing `smith: a private file answer the program did not ask for`; no panic. *dropped*: the event adds nothing to `up`, and the backend's file state is as before it (`files.pending`, the created-file flags, `settings.directory`); no panic.
2. **It fails first:** run on the task's base, the table's first row panics with `one pending private file request`; the report quotes it (`notes/common.md`). Then the guard, and the table passes.
3. **The guard,** in `files.rs` and `native.rs`, as sketched below.
4. **Status:** delete the **Not yet** entry `5.1: an answer not asked for aborts the agent` from `smith/docs/design/protocol/programs.status.md`; no other design line.

## Inputs

- `smith/docs/plan/program-burst-files/notes/common.md`.
- `smith/docs/plan/program-burst-files/inputs/request.md`, item 2.
- `smith/docs/design/protocol/programs.md`, 5.1: the bullets *Each program's files are its own* and *An answer not asked for* (written by this cut); 5.6, *A start that fails*; 5.10, *Its failures are its call's*.
- `smith/crates/smith-protocol-program/src/backend.rs`: `from_io` (routes `Owner::Native` to `native_event`, every other file answer to `file_event`), `fail`, `token`, `Owner`.
- `smith/crates/smith-protocol-program/src/backend/files.rs`: `file_event`, `file`, `cleanup`; `backend/native.rs`: `load_native`, `native_event`.
- `smith/crates/smith-protocol-program/src/backend/fixtures.rs` (`starting`, `active`, `running_child`, `claim_completion`, `env`, `COMPLETION`) and `backend/resume_tests.rs` (`running_history`) for driving `from_io` with `BelowEvent::File`.

## Owns and may change

Owns `file_event`'s and `native_event`'s first lines and the new guard beside them, and `backend/files/tests.rs`.

Reader and neighbour files, each edited only so:

- `smith/crates/smith-protocol-program/src/backend.rs`: `from_io`'s call of `native_event`, if its signature gains `env`.
- `smith/docs/design/protocol/programs.status.md`: the one entry of acceptance 4.

Options, each decided:

- **Where the guard lives:** one private helper on `Backend` in `files.rs`, called by both events, is the default; two inline `match`es are fine if shorter.
- **`native_event` needs `env`** for `fail`: add the parameter and pass it from `from_io`.
- **A native snapshot held but not yet `submitted`** counts as nothing outstanding: the answer fails the run and the snapshot stays held. If keeping it held trips a later assertion in a row, clear it (`self.native = None`) and say so in the report.
- **`fail` in phase `Idle`** (no completion claimed) records `pending_failure` for the next completion: no row is required for it. If a row you add shows `fail` asserts there, drop that row and say so.
- **An unasked `Opened`** leaves its file open in the file component: drop it with the event; closing it is not this task's, and the report says so.

## Steps

The guard's shape (the names inside are yours):

```rust
// files.rs
const UNASKED: &[u8] = b"smith: a private file answer the program did not ask for";

pub(super) fn file_event(&mut self, env: ..., event: file::Event, descriptor: Option<kernel::Fd>, up: ..., below: ...) {
    let Some(step) = self.files.pending.take() else {
        self.unasked(env, up, below);
        return;
    };
    // unchanged from here
}

fn unasked(&mut self, env: &Env<protocol::ComponentLimits>, up: &mut Queue<ToDomain>, below: &mut Queue<Below>) {
    match self.startup.phase() {
        State::Ending | State::Ended => {}
        State::Idle | State::Checking | State::Starting | State::Running => {
            self.fail(env, llm::Failure::Program { exit: None }, UNASKED, up, below);
        }
    }
}
```

`native_event` takes `self.native` only when it is `Some` and `submitted`; otherwise it calls the same guard and returns.

1. Write the table and run it alone; see the first row panic (acceptance 2):
   `heavy cargo nextest run -p smith-protocol-program --lib -E 'test(=backend::files::tests::an_answer_not_asked_for_fails_its_run_never_the_agent)'`.
2. Add the guard; the same command passes, one test selected.
3. Delete the status entry (acceptance 4).
4. `cargo fmt`; `python3 -I -B smith/scripts/old_forms.py`; `python3 -I -B tools/citations/citations.py --base origin/main`; `heavy cargo clippy -p smith-protocol-program --all-targets -- -D warnings`; `heavy cargo nextest run -p smith-protocol-program`; `heavy cargo nextest run -p smith-agent-service`; `heavy cargo nextest run -p smith-agent-process-world` (the program worlds that drive file answers through the service). A departure from the implementer text's every direct dependent: no public item changes, and a package whose tests delivered an unasked answer would panic on the base, so the other trees' dependents can't see the change; the trunk's end check runs them.
5. Commit (why: an answer a backend did not ask for fails its run instead of aborting the agent) and push with `git push origin HEAD:refs/heads/<your branch>`.

## Report

fab's implementer report, with: the panic seen in step 1, each row's outcome after the guard, the counts of the three package runs, and the choices of the options above.

## Rebase (2026-11-08, step merge, foreman program-burst-files/unasked-answer/foreman-1)

resolve the status conflict by retaining the new routing gap and deleting only the unasked-answer abort gap.
The merge failed in `smith/docs/design/protocol/programs.status.md`.
Integration commit `00bb0beb809ac09bc1a817f80bf2a96211abe46e` added the adjacent **Not yet** entry beginning “5.1: an answer's program is told by any token the program owns”.
Task commit `55ab3b6aee1d4d8179d6cd15c2c2a1efb4c3e328` deletes the preceding entry beginning “5.1: an answer not asked for aborts the agent”.
Rebase onto the landing's current integration tip. Resolve the conflict by preserving the routing entry in full and removing only the seven-line unasked-answer abort entry, as acceptance 4 requires.
Preserve all other status text and the task's code changes.

## Answers

2026-11-08. *May the task repair `builtin_tests::several_native_reads_in_one_turn_allow_read_only_reviewer_mounts`, which delivers `Loaded` to a snapshot held but not submitted?* Yes: the guard's held-but-not-submitted rule is this task's, so the fixture that broke under it is too, and the package check is its acceptance. In that test, insert `backend.load_native(&mut below);` immediately before `backend.native_event(...)`, so the snapshot is submitted as production submits it. Do not call the `native_load` helper: it asserts the single load's path is `a`, and this test reads `ci.yml`, `CONTRIBUTING.md` and `changes/*`. The `Below::File { Load }` the call pushes stays in `below`; `fixtures::server_reply` (in `allow_native`) skips file effects, so nothing else changes. No other line of `builtin_tests.rs` beyond this and the existing `env` wiring; no production change. Then finish step 4's three package runs and report their counts.
