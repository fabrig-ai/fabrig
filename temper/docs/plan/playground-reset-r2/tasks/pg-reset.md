+++
id = "pg-reset"
part = "playground"
kind = "feature"
needs = ["pg-start", "rm-project", "rm-service", "jp-delete"]
files = [
  "temper/crates/temper-engine-domain-forge/src/reset.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/reset.rs",
  "temper/crates/temper-engine-domain/src/route/reset.rs",
  "temper/crates/temper-engine-domain/src/tests/reset.rs",
  "temper/tests/engine/domain/tests/suite/playground_reset.rs",
]
effort = "high"
minutes = 90
+++

## Goal

The playground's reset whole (draft 6.2): `EndPlayground`, the
deployment's, answered `Resetting` in the decision that moves the
`Playground` row to `Resetting(Project)`; then the row's own steps, in
order, each act sent under a key derived from the step and the row
moved on at its answer: the playground's project deleted (jig's
`Act::RemoveProject`, its chats cancelled with it), the fake forge
removed (the connector's own removal, `rm-domain`), the scripted
account removed (`Act::RemoveAccount`); then the row erased. Cut by a
restart, the step is sent again after the core opens; a row restored
`Running` is reset at that start (draft 9's first default: the fake
forge kept nothing across the start). Afterwards the store reads as one
that never tried the playground: no playground row, no fake forge
record, no playground project, no scripted account.

Acceptance:

- `temper-engine-domain-forge`, `src/tests/reset.rs`:
  - `end_moves_the_row_to_resetting_and_sends_the_project_step`;
  - `end_without_a_playground_is_not_offered`; a second `EndPlayground`
    while resetting answers `Resetting` and sends nothing new;
  - `each_step_moves_on_at_its_answer_and_the_last_erases_the_row`
    (project answered → `Resetting(Forge)` and the forge's removal
    begun, with its kept acts; record gone → `Resetting(Account)`;
    account answered → `Key::Playground` erased);
  - `a_step_with_nothing_to_do_moves_on` (a row whose project, forge or
    account is none, or whose forge record is already gone);
  - `a_running_row_restored_is_reset_at_start` and
    `a_resetting_row_restored_sends_its_step_again`;
  - `start_while_resetting_is_not_offered` (stays as pg-start made it).
- `temper-engine-domain`, `src/tests/reset.rs`:
  `the_reset_steps_are_jigs_deployment_acts_under_their_keys`
  (`people::Act::RemoveProject { project }`, then
  `people::Act::RemoveAccount { account }`, the forge connector's
  number and the derived keys), and
  `reset_outcomes_move_on_or_keep_as_the_kept_acts_do`: `ProjectRemoved`,
  `AccountRemoved`, `Refused(Unknown)` move on; `Again`,
  `Refused(NotOffered)` (note naming the missing grant,
  `remove_project`/`remove_account`) and `Refused(Role { held: None })`
  keep the step.
- `temper-engine-domain-world`, `tests/suite/playground_reset.rs`:
  `a_started_playground_reset_reads_as_never_tried`: a playground
  started (pg-start's world helpers), a chat in its project; the reset
  answers `Resetting`; the service's answers to `RemoveGroup` and
  `EraseSecrets` given by the test; then the chat ended and erased with
  its project (jp-delete), no row of the playground, its forge or its
  account in the store, `StartPlayground` offered again and making a
  forge with the next number.
- `temper-service`: the wire `end_playground` reaches the engine as
  `forge::Ask::EndPlayground` (cl-wire's `Undecodable` bridge replaced)
  and answers `resetting` or `end_refused(not_offered)`; the read
  `playground` answers `resetting` until the row is none
  (`src/tests/family.rs`).

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, 1
  (`RemoveProject`, `ProjectRemoved` answered at the closing decision),
  2 ("The playground's reset") and 5 (`EndPlayground`).
- `temper/docs/design/draft/playground-reset.md`, 6.2, 9 (the first
  default).
- `jig/docs/design/domain/people.md`, 5.3 (the deployment's acts, keys,
  retained answers).
- `temper/docs/design/protocol/setup.md`, 8.4 (kept acts resent at
  start, the pattern).
- The code: `temper-engine-domain-forge/src/remove.rs` and
  `src/playground.rs`; `temper-engine-domain/src/route/kept.rs` and
  `src/route/playground.rs`; `restart.rs` (`SendKeptActs`).

Assumed: rm-domain's `remove.rs` (`ask`, its record-erase point),
`route/kept.rs` (reply tokens, notes, `Delivery::DemoActed` dispatch by
token); pg-start's `playground.rs`, `PlaygroundConfig`, the row's parts;
jp-delete's `Act::RemoveProject` built; forge-families'
`kept::{save_playground, erase_playground}`; cl-wire's
`w::Request::EndPlayground`, `w::Outcome::{Resetting, EndRefused}`.

## Owns and may change

Owns the new modules and tests in the header.

May change: the forge crate's `boundary.rs` (the vocabulary below,
placed after pg-start's playground items), `domain.rs` (dispatch),
`remove.rs` (split its decision into a `begin` the reset calls without
the playground's *not offered* check, and call the reset when a
record it erases is the row's forge), `playground.rs` (`restart`),
`lib.rs`; the root's `boundary.rs` (`ApplicationAnswer::EndPlayground`),
`domain.rs`, `route.rs`, `route/forge_route.rs` (the new request's
arm), `route/applications.rs`, `route/kept.rs` (token dispatch),
`restart.rs`; `temper-service/src/family.rs` (the `EndPlayground`
request and answer arms) and the service's read of the row; forced
arms per `notes/scope.md`.

## Steps

1. **Vocabulary** (forge crate):

   ```rust
   Ask::EndPlayground
   Answered::EndPlayground(EndAnswer)
   /// playground-reset.md, 6.2.
   pub enum EndAnswer { Resetting, NotOffered }
   pub struct ResetAct { pub key: [u8; 16], pub act: ResetActKind }
   pub enum ResetActKind { RemoveProject { project: u32 }, RemoveAccount { account: u32 } }
   Request::SendResetAct(ResetAct)
   Event::ResetActAnswered { key: [u8; 16], done: bool }
   ```

   Root: `ApplicationAnswer::EndPlayground(forge::EndAnswer)`;
   `Ask::EndPlayground` of the deployment's scope.
2. **The steps** (`src/reset.rs`): `end(d, env, request, token,
   retention, out)` (the answer held as rm-domain's removal answers
   are), `step(d, env, out)` (sends the current step's act, or moves
   on when it has nothing to do), `answered(d, env, key, done, out)`,
   `forge_gone(d, env, forge, out)` (from `remove.rs`), and
   `restart(d, env, out)`, called on `Event::SendKeptActs`: a `Running`
   row is saved `Resetting(Project)` and stepped; a `Resetting` row's
   step is sent again. Keys: `sha256("temper playground reset\0" ||
   step name || number as 8 bytes BE)`, first 16 bytes, step names
   `b"project"`, `b"account"`. A pending start is answered `NotOffered`
   when the reset begins.
3. **The root** (`src/route/reset.rs`): `SendResetAct` becomes
   `core::Event::Act` with `people::Act::RemoveProject { project }` or
   `people::Act::RemoveAccount { account }`, the forge connector's
   number and the row's key, its reply token held as `kept.rs`'s are;
   the outcome becomes `forge::Event::ResetActAnswered { key, done }` (done on
   `ProjectRemoved`, `AccountRemoved`, `Refused(Unknown)`,
   `Refused(Ended)`; kept, with a note prefixed `playground reset: `,
   on `Again`, `Refused(NotOffered)`, `Refused(Role { held: None })`;
   done with a note on any other refusal).
4. **The service:** `family.rs`'s `EndPlayground` arms; the read
   answers `resetting` from the row.
5. **Tests** as the acceptance names. Then, one at a time:
   - `heavy cargo nextest run -p temper-engine-domain-forge --lib`;
   - `heavy cargo nextest run -p temper-engine-domain --lib`;
   - `heavy cargo nextest run -p temper-engine-domain-world`;
   - `heavy cargo nextest run -p temper-service`;
   - `heavy cargo nextest run -p temper-system-world -p temper-client-world -p temper-one-process-world -p temper`
     (focused);
   - `cargo fmt`; `heavy cargo clippy` over every package changed,
     `--all-targets -- -D warnings`;
   - `python3 -I -B scripts/old_forms.py` from `temper/` (and `jig/`
     if you touched it).
6. Commit and push.

## Report

The standard report, plus: when exactly a restored `Running` row is
reset (which decision); how the reset learns the forge's record is
gone; whether jp-delete's `ProjectRemoved` came before the project's
tasks were erased (and so before rm-project's erase ran); the grants
the template must carry (`remove_project`, `remove_account`, `cancel`)
and whether fr-init's template already does.
