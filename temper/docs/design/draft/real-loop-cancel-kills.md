# Real-loop stories count a cancel's kill as asked for, and pin their workers' order

Draft, 2026-11-16, on the user's word. With `real-loop-kill-cause.md` in, three real-loop stories that flaked on 11-16's gates now say why:
- **case_5_4 (run 1178):** a worker killed a running `git` while cancelling a coder's preparation, after the story's own cancel.
- **turn_landing's failing-member story (run 1185):** the standalone service killed a running `git` while cancelling a prepare job, right after the story cancelled its goals and landing.
- **case_4_1_4_3 (run 1185):** the worker named `ws` ran both chats, though the story assumed `s1` would take one.

Each passed alone. The first two are correct behaviour that the story's check counts as unrequested: git's design kills its child when a cancel wins the race (`temper/docs/design/protocol/git.md`), and whether the git is still running is timing. The third assumes an order the fleet doesn't promise.

## 1. A kill that follows the story's own cancel is accounted for

- **Today:** `check` (`temper/tests/real-loop/src/lib.rs`) accepts one recorded kill per explicit `Kill` step and refuses any other, and `temper/docs/design/testing/real-loop.md` says the same. A cancel the story requests travels through jig's host and the worker's workspace abort to git's `Operation::cancel`, which kills its child with reason `Cancelled` (`jig/crates/jig-domain-host/src/hosted.rs`; `temper/crates/temper-worker-domain/src/workspace.rs`; `temper/crates/temper-worker-protocol-git/src/operation.rs`).
- **Wanted:**
  - The world keeps, for each `Cancelled` kill, the task and attempt whose work it ended, using the service's observations (`temper/tests/real-loop/src/service.rs`).
  - `check` accepts such a kill when that task is one the story cancelled, or one of its delegates, after the story's cancel step.
  - It still refuses a root's cancel of work the story didn't cancel, a deadline kill, and a supervisor's forced kill of an agent. The stubborn-agent teardown story's refusal stays.
  - Its message, on refusal, keeps naming the kill's target, requester and reason.
  - Revise `real-loop.md` where it says only `Kill` steps may kill.

## 2. case_4_1_4_3 doesn't assume which worker connects first

- **Today:** `the_opener_and_the_two_hour_check_run_on_three_workers` (`temper/tests/real-loop/tests/suite/fuzzy/case_4_1_4_3.rs`) starts `ws` and `s1` together, deferring only `s2`. It then asserts that `ws` ran one chat and `s1` another. The fleet chooses by free slots, workspace affinity and channel order (`jig/crates/jig-domain-fleet/src/channel.rs`, `choose`), not by name, so either can take the first chat.
- **Wanted:** the story defers `s1` as it does `s2`, and adds it once the director is placed on `ws` (`temper/tests/real-loop/src/case/acts.rs`, `step_4_1_4_3`). Its placement assertions stay as they are.

## 3. Out

- Changing git's cancel or the fleet's choice.

## 4. Tests

Cheap: focused and fuzzy suites only; walk = false.
- Focused tests of `check`: a `Cancelled` git kill of work the story cancelled is accepted. An unrelated root cancel, a deadline kill and a forced agent kill are refused.
- The teardown story still refuses a stubborn agent's kill.
- A focused test of the worker cancelling during preparation (`temper/crates/temper-worker-service/src/tests/worker_git.rs`) keeps its kill's reason.
- case_5_4, turn_landing's failing-member story and case_4_1_4_3 pass, each run alone by name.

Run only the new and affected tests, by name.
