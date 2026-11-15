# A checkout failure says where and why

Draft, 2026-11-15, on the user's word. On 11-15, a goal's first run failed six times in 11 minutes with "Unprepared(Transient): check out: TimedOut", and then went through. The forge was fine. Nobody could tell which worker had tried, which workspace, or what the checkout was waiting on: the workers log nothing for a checkout, and the engine's line names no host (P54). This draft makes the next case readable. A fix for the cause follows once one shows.

## 1. What a checkout failure carries

- **Today:**
  - A checkout runs in stages: workspace, clones, fetch, check out, merge of heads (`temper/crates/temper-worker-domain-checkout/src/hold.rs`; `temper/docs/design/domain/worker/checkouts.md`). Each operation has its own deadline: the remote timeout (60 s by default, `checkout.remote_timeout_ms`) for fetch, check out and merge, and the local one (10 s) for the rest (`temper/crates/temper-service-core/src/limits.rs`).
  - The stage "check out" covers both `git checkout --force --detach` and the `git clean -ffdxq` after it, under one deadline (`temper/crates/temper-worker-protocol-git/src/invocation.rs`, `sequence.rs`). The words can't tell a slow checkout, a lazy blob fetch or a long clean apart.
  - `preparation_detail` reports the stage and the fault alone. The engine's run-failure line (`run_failure_line`, `temper/crates/temper-engine-shell/src/lines.rs`) prints the task, run, attempt and those words, with no host, workspace or timing. The worker logs nothing for a failed preparation (`Worker::run_answer`, `temper/crates/temper-worker-service/src/worker.rs`).
- **Wanted:**
  - A failed preparation's detail names the worker's host, the workspace, the repository, and the stage and command it was in, with check out and clean told apart. It also gives that operation's elapsed time against its deadline, and the whole preparation's elapsed time, so time spent queued shows apart from the operation's own.
  - The detail travels in the failure's existing detail bytes (`Failed.detail`, at most 4,096 bytes, `jig/crates/jig-protocol-worker-wire/schema/worker-wire-v14.schema`), with room reserved so these fields survive truncation before any git output. No wire or stored format changes. If a structured field turns out to be needed instead, the wire moves its version and bridges, by the upgrade rules.
  - The engine's run-failure line prints that detail whole.
  - The worker logs a line for each failed preparation, with the same fields (`worker_alone`'s notes and the worker shell's `note_line`).
  - Revise `checkouts.md`, and `temper/docs/design/protocol/git.md` where it describes the check out stage, to say so.

## 2. Out

- Changing the deadlines, or how clean treats build output. Wait for the cause.
- The checkout's other facts, which have no consumer yet.

## 3. Tests

Cheap: focused and fuzzy suites only; walk = false.
- In the checkout domain's tests (`temper/crates/temper-worker-domain-checkout/src/tests.rs`, beside the typed preparation failures): a timed-out stage names its workspace, stage, command and timings, and queued time is counted apart.
- In the git component's tests (`temper/crates/temper-worker-protocol-git/src/tests/component.rs`): a deadline during check out and one during clean are told apart.
- A worker service test (`temper/crates/temper-worker-service/src/tests/`): the failure's detail carries the host, and a note is logged.
- The engine shell's line tests (`temper/crates/temper-engine-shell/src/lines/tests.rs`): the fields survive escaping and truncation.

Run only the new and affected tests, by name.
