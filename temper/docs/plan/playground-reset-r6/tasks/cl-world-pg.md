+++
id = "cl-world-pg"
part = "pg"
kind = "feature"
needs = ["cl-world-forges", "fe-playground-forge", "fe-playground-root", "pd-core-release"]
files = [
  "temper/tests/client/**",
  "temper/crates/temper-engine-domain/src/route/playground.rs",
  "temper/crates/temper-engine-domain/src/tests/playground.rs",
  "temper/crates/temper-client-domain/src/forge/demo.rs",
  "temper/crates/temper-client-domain/src/tests/demo.rs",
]
effort = "high"
minutes = 120
+++

## Goal

The joined client world proves the playground on the real engine, start to reset, and its
scripted stand-ins go (draft `temper/docs/design/draft/playground-reset.md`, 6.1 to 6.3;
`notes/contracts.md` T6, J1, J8): `front.rs`'s `start_demo`, `demo_read` and
`demo_forges` intercepts, `demo_starts`/`demo_outcomes` and the `demo` fixture answer are
removed; `start_playground`, `end_playground` and `Read::playground` reach the engine.

- The bridge translates `start_playground` → `forge::Ask::StartPlayground`,
  `end_playground` → `forge::Ask::EndPlayground`, and their answers as `notes/cl-wire.md`
  3 reads them.
- `Read::playground` is answered by the world as serve answers it (`notes/cl-wire.md` 3):
  from the engine's `Playground` row (`forge::Domain::playground()` or the store's row) and
  the configuration's `application.demo` members (provider, owner, name, cues), no record
  when the configuration has no playground.
- The playground's fake forge connects through the engine world's fake forge stand, so the
  start's adoption of `playground/widget` is the engine's.
- New acceptance cases pass (step 2); the `demo_start` suite's cases are rewritten onto the
  engine, each keeping what it proves where the draft keeps it, those of removed behaviour
  already gone (cl-pg-goes).

## Inputs

- `temper/docs/plan/playground-reset-r6/notes/common.md`; `notes/contracts.md` T6, J1, J8,
  T8; `notes/cl-wire.md`; `notes/fe-surface.md` (the playground's `Ask`s, answers and row).
- `temper/docs/design/domain/testing.md`, 6.3.
- `tasks/cl-pg.md`, `tasks/cl-pg-goes.md` (the client behaviour proved);
  `tasks/fe-playground-forge.md`, `tasks/fe-playground-root.md` (the engine's).

## Owns and may change

Owned: `temper/tests/client/`, and the two fixes in this spec's Answers, in the files they
name. Not yours: the engine world library (as `cl-world-forges` says: a seam you lack is
written over its public API in `front.rs`, or a question).

Options, pre-approved:

- If jig's project deletion (contracts J1 to J4) is not in the trunk this task starts
  from, the reset stops at its `Project` step: stop and report it as a question naming the
  missing jig task, rather than scripting the step.

## Steps

1. Remove the intercepts and their fixtures; route the three through the bridge; the
   world's playground read as the Goal says; the start's fake forge stand.
2. **Cases** (`tests/suite/demo_start.rs`, renamed `playground.rs` if you like):
   - `try_the_playground_starts_it_on_the_engine_in_its_project` (the row `Running`, the
     fake forge's record, the project made, the scripted account's uses exactly the
     starting person and that project, `playground/widget` adopted, the chats page in the project);
   - `set_up_temper_resets_the_playground_and_opens_the_forge_step` (the banner's link;
     "Leaving the playground…" until `reset`; then: no `Playground` row, no forge record,
     the project gone, the scripted account gone; the welcome at Forge reads Empty and
     offers "Try the playground" again);
   - `a_reload_while_resetting_shows_leaving_then_the_forge_step` (the reset held at its
     `Forge` step: a fresh client reads `resetting`, shows leaving, then the Forge step);
   - `a_real_account_before_the_playground_stays_and_shows_ready_at_model`;
   - `forges_and_accounts_offer_nothing_to_add_while_it_runs`;
   - `a_begun_real_forge_refuses_the_playground_not_offered` (a forge set up first: the
     welcome draws no playground card; a start sent anyway answers `start_refused`).
3. `heavy cargo nextest run -p temper-client-world --test suite`; `heavy cargo check -p
   temper-client-world --tests`; the `qa_all` cases that named the demo's intercepts
   (`qa_first_run_hooks.rs`, `qa_welcome_m3.rs`, `qa_welcome_m4.rs`) rewritten onto the
   engine or, where their subject is gone, removed, each listed; run them:
   `heavy cargo nextest run -p temper-client-world --profile qa-temper --test qa_all -E
   'test(/welcome/) | test(/first_run/)'`. `cargo fmt`; clippy `--tests`; commit and push.

## Report

- Each `demo_start` and `qa_all` case rewritten or removed, and why.

## Answers

2026-11-01. Q: assign the funding and failed-start fixes and resume this branch, or retire
the two preserved `demo_start` cases?

A: Keep both cases and fix both here; this task's files widen to the four named (its
acceptance cannot hold without them). Merge the trunk into this branch first, then:

- **Funding: temper's, not jig's.** `coverage.rs` is right and stays: a person's chat is
  funded by their pool, so its payer is the person (jig's `domain/accounts.md`, 4.1), and a
  project in an account's uses covers that project's period-funded work alone. J8/T6's uses
  were wrong, and are amended: the scripted account is made with
  `Some([Use::Party(starting person), Use::Project(project)])`. In
  `route/playground.rs`, `begin` keeps the sign-in's person for every start, not only while
  the row has no project, so that a retry's `AddKey` can name them, and `prepare` sends those
  uses. Update `tests/playground.rs`'s expected act and stored account. The world never
  broadens the uses itself. `try_the_playground_...` asserts these uses exactly, and
  `enter_waits_for_adoption_then_the_chat_can_delegate` stays as it is. Should that chat stop
  on anything but its account, that is a new question.
- **Failed start: the client's.** In `forge/demo.rs`'s start answer, the selection moves
  from `Starting` to `Reading` on `StartAnswer::Failed(_)` as on `Running`, so the following
  read selects the project the failed start made and `chats.rs::demo_start` draws the alert
  and Try again. Add `a_failed_start_selects_the_project_it_made` to `src/tests/demo.rs`.
  `failed_start_survives_navigation_and_try_again_uses_a_new_key` stays as it is. A start
  that failed before making a project has no project to select: leave it, under `## Left`.
- **Left:** `names.rs::client_crates_carry_the_client_prefix` and the seven `qa_all` cases in
  your `## Left` are outside this task: leave them there, unweakened. This task is accepted
  with the `demo_start` cases and its rewritten `qa_all` cases passing.
- Add to your checks: `heavy cargo nextest run -p temper-engine-domain --lib` and `-p
  temper-client-domain --lib`, with clippy `--tests` on both.

2026-11-01. Q: the preserved chat case now stops on `TaskRunWait::Credential`: whose is it?

A: This task's, in the world: the fake setup host never grants the playground forge's git
identity. A run's workspace names each repository's git identity as an application
credential (`temper-engine-domain/src/route/now.rs::workspace`); a record forge's identity is
its own number (`temper-engine-domain-forge/src/domain.rs::git_identity`), and jig waits on
`Credential` until that number is granted. serve grants a forge's identity when its setup
keeps the git token (`temper-service/src/setup/lifecycle.rs`, `root::Event::Credential`); the
world's stand does serve's part, as `fixtures.rs::adopt_change_repository` already does.

- In `front/playground.rs::connect_playground`, before the `Connected` event, send
  `engine::Event::Credential { number: u32::from(row.forge), generation: 1, valid:
  Duration::from_secs(3600) }`. Every connect grants it, so a playground started again after
  a reset, on a new forge number, is granted too.
- `enter_waits_for_adoption_then_the_chat_can_delegate` stays as it is. No account, use or
  engine change. Should the chat still stop after this grant, stop and report what it then
  waits on, with the task document's `run_wait`.
- Left, unchanged: `forge_route/tests.rs::writable_repositories_exclude_an_ended_forge`
  (outside the task) and the cases already left.
