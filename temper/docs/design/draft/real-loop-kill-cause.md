# A real-loop world's unrequested kill names its cause

Draft, 2026-11-16, on the user's word. On 11-16, in two gate runs in a row, the fuzzy real-loop story `case_5_4::a_new_direction_cancels_x3_…` failed its check that every kill was one the story asked for: `only requested kills: [Killed { host: 158, exit: Signal(9), peak: None }]` (`temper/tests/real-loop/src/lib.rs`). It then passed alone. The record says a hosted child was killed inside the world, but not which child, by whom or why. So nobody can tell whether a cancellation escalated to a kill under load, a process closed early, or a deadline fired. This draft makes the next one readable; a fix for the cause follows once it shows.

## 1. A kill records who asked and why

- **Today:**
  - In skein-world's real world, every hosted child runs on the world's one loop. `Running::kill` turns an `Op::Signal { signal: Kill }` into exit `Signal(9)` and records `Killed { host, exit, peak }` (`skein/testing/skein-world/src/real.rs`).
  - `host` is an admission index, which can't be mapped back to a program afterwards. The record keeps no requester, target program or reason.
  - Several paths send that kill:
    - a story's own `Step::Kill`;
    - smith's supervision escalating TERM to KILL once a cancellation's grace has passed (`smith/crates/smith-host-domain/src/domain.rs`);
    - a process closed or aborted while running (`skein/crates/skein-io/src/process.rs`, `begin_close`);
    - a worker's stop bound (`temper/crates/temper-worker-domain/src/stopping.rs`);
    - a job's deadline or cancel (`temper/crates/temper-worker-service/src/worker/jobs.rs`).
  - The real-loop service's notes are dropped when the assertion fails (`temper/tests/real-loop/src/service.rs`).
- **Wanted:**
  - `Killed` also holds:
    - the time on the world's clock;
    - the target's program and arguments' first word, as admitted;
    - the requesting host and its program;
    - the reason, where the sender knows it: a story kill, a cancellation's grace passed, a close or abort while running, a stop bound, a job's deadline or cancel.
  - Each kill path passes its reason down with the signal, where it can. Where it can't, the reason is named unknown.
  - The "only requested kills" assertion prints every unrequested record in full, and the service's notes from the story's last seconds, so a failing run names its cause.
  - The assertion stays as strict as it is.

## 2. Out

- Changing case_5_4, a cancellation's grace or any supervisor's escalation. Wait for the cause.
- The gate machine's CPU and memory holds, which another area runs.

## 3. Tests

Cheap: focused and fuzzy suites only; walk = false.
- skein-world's real world tests: a story kill, a close while running and a deadline kill each record their target, requester and reason.
- A focused real-loop test: a cancellation whose agent ignores its grace ends in a kill named as a forced kill after the grace, and the assertion's message carries it.
- case_5_4 passes as before.

Run only the new and affected tests, by name.
