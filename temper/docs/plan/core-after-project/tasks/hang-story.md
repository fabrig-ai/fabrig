+++
id = "hang-story"
part = "core-answers"
kind = "feature"
needs = []
files = ["temper/tests/engine/domain/tests/suite/core_after_project.rs", "temper/tests/engine/domain/tests/suite/main.rs", "temper/crates/temper-service/src/tests/core_after_project.rs", "temper/crates/temper-service/src/tests.rs", "temper/tests/one-process/tests/suite/core_after_project.rs", "temper/tests/one-process/tests/suite/main.rs", "temper/docs/design/domain/root.status.md"]
effort = "high"
minutes = 360
+++

## Goal

A real deployment's core stopped answering every call right after `project make`, and again after a restart on the same store (`temper/docs/plan/core-after-project/inputs/request.md`, the acts 1 to 7 and what was seen). Reproduce it as a failing story in the lowest world that hangs, cut it to the smallest sequence that still hangs, find what the core waits on, and fix that cause, so the core answers, before and after a restart, and a store already in that state opens and answers.

What reading found, which the story must confirm or overturn (the cut's surveys; see `## Inputs`): the evidence fits one state alone, the service's gate shut because the root is no longer `ready()`. A public read of a private project reaching jig answers 404, not 503; 503 at once is `temper-service/src/clients/translation.rs`, `not_ready`, which the gate (`temper-service/src/engine.rs`, `iterate`, `open`) applies to every party's input while it is shut, with the clients' component `starting`, so no link comes up. At runtime the gate's other terms are startup's alone (`first_periods`, `forge_records`, `forge.resolving`), and `domain.ready()` is `opened && start_stop().is_none() && !journal.stopped()`. A start stop or a failed commit ends the engine loudly. The one way to a shut gate with nothing logged and an idle loop is a journal stopped by a store answer it refuses (`skein-lib/src/journal.rs`, `committed`: `number <= durable || number > sent`), taken in `temper-engine-domain/src/route.rs`'s `StoreInput::Committed` arm, which emits no `Stop`. No producer of such an answer was found by reading. Step 6 is the likeliest trigger and steps 1 to 3 its likeliest preconditions; the forget and the refused declare (steps 2 and 4) leave no global wait that reading can find.

Acceptance:

- **The story, red first,** `a_core_answers_after_a_project_is_made`: the acts in order, then the core calls of act 7 (`project show 3`, `me`, `account list`, `settings show`, a fresh sign-in), each of which must answer within the world's bound; then a restart on the same store, and the same calls again, each answering within the bound. It is committed failing first, in the lowest world of the ladder below that hangs on it, and its failure names the gate's state (the root's `ready()`, its journal's `stopped()`, its start phase, the service's `open` terms, as that world holds them).
- **The ladder of worlds,** lowest first, each tried with the whole sequence before the next:
  1. `temper-engine-domain-world` (`temper/tests/engine/domain`): the root on the world's store, `direct::Driver`, a restart by `Driver::reopen` on the same `Store`;
  2. `temper-service`'s own tests (`temper/crates/temper-service/src/tests/`): the service's gate and jig's store over in-memory maps (`fixtures::ready`, `fixtures::restarting_from`, which reopens the same maps), started from the stock configuration (`crate::stock`) as `temper serve` is;
  3. `temper-one-process-world` (`temper/tests/one-process`): the service with skein-kv's log files on the simulator's disk, restarted on the same disk, the person's terminal client in front of it.
  The story lives in the first that hangs, its file named `core_after_project.rs` there. A world that does not hang keeps no file of this task's.
- **The smallest sequence.** From the whole sequence, drop each act in turn (keeping act 6 and the calls), and keep it dropped where the story still hangs. The committed story holds that smallest sequence. A second test, `the_incident_s_whole_sequence_answers`, keeps the whole sequence, so the preconditions stay covered once fixed. The report lists each act and whether the story hung without it.
- **What the core waits on, named** in the design's terms (`temper/docs/design/domain/root.md`, sections 4, 5.7 and 9.1; `temper/docs/design/protocol/README.md`, 8.2's gate), with the code: the state, who put it there, and why the restart meets it again. The story's module doc comment says it in two or three lines, citing the design sections: that comment is how `why-log`, which runs after you and reads only the repository, learns the cause.
- **Fixed,** in the lowest crate that holds the cause, with a row in that crate's own tests first that fails on it. Then both stories pass, before and after the restart.
- **A store already in that state opens and answers:** a row of the story whose store was made by the sequence with the cause in place (that store captured before the fix's code runs, or its records seeded as the sequence left them) restarts under the fixed code and answers the calls.
- `temper/docs/design/domain/root.status.md`: the *Not yet* entry for 5.7 and 9.1 (the core stopped answering after `project make`) is removed. The entry for section 4 (a stopped journal ends the engine) stays: it is `why-log`'s.
- The packages you changed and their direct dependents pass their focused tests, and `temper-engine-domain-world`'s and `temper-service`'s focused suites pass.

## Inputs

- `temper/docs/plan/core-after-project/notes/common.md`, all of it.
- The request: `temper/docs/plan/core-after-project/inputs/request.md`.
- `temper/docs/design/domain/root.md`, section 4 (the journal, its stops, admission at the watermark), 5.7 (periods; a project made after the first start), 9.1 (restart).
- `temper/docs/design/protocol/README.md`, 8.2, first paragraph only: the gate and its terms.
- `temper/docs/design/domain/testing.md`, 2.3 and 2.5 (the one-process world), and 6.1 (focused tests' budgets).
- `skein/docs/design/lib.md`, section 11 (the journal: answers outside the order commits were sent stop it).
- The code the reading went through: `temper-service/src/engine.rs` (`iterate`'s `open`, `input_pass`, `release`, `step`'s `pending` and `blocked`), `temper-service/src/clients.rs` (`root`, `early`, `starting`) and `clients/translation.rs` (`not_ready`); `temper-engine-domain/src/route.rs` (the `Committed` arm), `domain.rs` (`ready`, `send_commit`, `finish_close`, `resume_routed`, `step_released`'s `ProjectMade`), `admission.rs` (`try_step`), `decision.rs` (`accept_routed`, `committed`); `temper-engine-domain/src/route/calendar.rs`; `jig-domain-core/src/routing/party/projects.rs` (`make_project_request`) and `routing/run/delegation.rs` (`route_period`); `jig-protocol-store/src/directory.rs` (`events_up`, the `Committed` answer); `skein-kv/src/store.rs` (`commit_memory`, `batch_synced`).
- Fixtures to start from: in the engine domain world, `tests/suite/settings.rs` (`MakeProject` after a settings change), `accounts_restart.rs` (a declared program account; restart on the same store), `forge_routes/goal_changes.rs` and `kept_change.rs` (a goal's change landed), `periods.rs`; in `temper-service`, `tests/first_model.rs` (`MakeProject` on the engine world's `Driver`), `tests/first_period.rs` (a later start on the same store); in the one-process world, `tests/suite/story_a.rs` and its upgrade story's landed change.

## Owns and may change

- Owned: the story's file in the world where it lives, and its `mod` line in that suite's `main.rs` (or `temper-service`'s `src/tests.rs`); `root.status.md`'s entry for 5.7 and 9.1.
- The fix: the lowest crate that holds the cause. That may be `temper-engine-domain`, `temper-service`'s engine, store or clients modules, `jig-domain-core`, `jig-protocol-store`, `skein-lib`'s journal or `skein-kv`, under `notes/common.md`'s rule for skein and jig. You own those files for the fix, and that crate's test row beside its siblings.
- Fixtures: a helper the story needs in its world's `src/` (the engine domain world's `direct.rs` or `fixtures.rs`; `temper-service`'s `tests/fixtures.rs`; the one-process world's `src/`) may be added to, additions only.
- Not yours: `temper-service`'s gate words and the log's lines (`why-log` writes them after you); the `StoreInput::Committed` arm's `Stop` on a refused answer, unless the cause is that arm, in which case fix the cause and leave the `Stop` and its reason to `why-log`; the CLI (`sign-in-line`).
- Options, each pre-approved:
  - **No world of the ladder hangs:** land the whole sequence as `the_incident_s_whole_sequence_answers` in the one-process world, passing, before and after a restart. Report `Done:`, with the gate's state as each world held it after act 6 and what was ruled out; put the unreproduced hang under `## Left`.
  - **The cause is a store answer the journal refuses:** fix its producer. If the answer is a repeat or an older number the store may lawfully send (by jig's `protocol/store.md` or skein-kv's contract), make the root take it as inert in `temper-engine-domain/src/decision.rs`'s `committed` wrapper, as its doc comment says, leaving `skein-lib`'s journal as its design says. Say which in the report.
  - **The cause is a start step that never ends** (a load, a continuation, `ReadAfresh`, `SettleOutbox`, the calendar's start): fix the step, in the crate that holds it.
  - **The fix needs a stored format, a protocol, or a skein or jig public signature changed:** commit and push the story red, and ask (`Question:`), with the options and your recommendation.
  - **The acts need a fixture the world lacks** (a goal run whole, a program account's forget, a settings revision): build it from the world's existing pieces in its `src/`; or, where a goal run whole costs more than its world's budget, seed the store with the records an ended goal leaves (`Store::seed` in the engine domain world), say so in the report, and keep the acts 2 to 7 run.

## Steps

1. Read the inputs.
2. Write the story with the whole sequence in the engine domain world; run it by name. If it does not hang, step up the ladder.
3. Commit it red where it hangs. Cut the sequence down, act by act; commit the smallest-sequence story red, and the whole-sequence one beside it.
4. Name what the core waits on. Write the failing row in the crate that holds the cause, then the fix; the store-in-that-state row; both stories green.
5. Break the fix once by hand: the story and the row fail for the cause's reason. Restore.
6. Remove `root.status.md`'s entry for 5.7 and 9.1.
7. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py` (and `skein/scripts/old_forms.py` or `jig/scripts/old_forms.py` for a tree you touched); `heavy cargo clippy -p PKG ... --all-targets -- -D warnings` for every package you changed; the focused tests named in the acceptance.
8. Commit and push.

## Report

- Which world hung, and the story's red failure: its assertion and the gate's state.
- Each act and whether the story hung without it: the smallest sequence.
- What the core waits on, in the design's words, with the code (file and function): the state, its producer, and why the restart meets it again. Say which of the request's suspects it is (a load that never completes, a startup continuation, a held lane, an admission that never frees, a record the core cannot decode) or that it is none of them.
- The fix, its crate, the row that pins it and its broken-by-hand failure; the store-in-that-state row; any skein or jig crate touched, as the heads-up.
- What `why-log`'s line should name for this cause (a reason of `temper/docs/design/protocol/README.md`, 8.2, or a new one), in a sentence.
- The new tests' times.
