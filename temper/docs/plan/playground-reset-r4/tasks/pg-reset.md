+++
id = "pg-reset"
part = "playground"
kind = "feature"
needs = ["pg-start", "kept-acts", "rm-remove-forge", "pd-delete"]
files = [
  "temper/crates/temper-engine-domain-forge/src/playground.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/playground_reset.rs",
  "temper/crates/temper-engine-domain/src/route/playground.rs",
  "temper/tests/engine/domain/tests/suite/playground_reset.rs",
]
effort = "high"
minutes = 70
+++

## Goal

The playground's reset (draft
`temper/docs/design/draft/playground-reset.md`, 6.2) and serve's start
resetting a playground found running (contracts, D5): `EndPlayground`,
in one decision, reads the `Playground` row resetting and keeps the
reset's first act; its acts go in order, each kept (D4) and sent again
at each start until answered: the playground's project deleted (jig's
`RemoveProject`, its chats cancelled with it), the fake forge removed
(`removal::remove`, `rm-remove-forge`, its box group stopped and
removed), the scripted account removed (`RemoveAccount`); then the row
erased, and every `EndPlayground` waiting answered `reset`.

Acceptance:

- `EndPlayground`: with no `DemoConfig`, *not offered*; with no row,
  `reset` at once; with the row `Running`, in its decision: the row
  saved `Resetting`, the first step kept (below), any start of the
  playground still out answered *not offered* and its root route
  dropped; with the row `Resetting`, held; every held request answered
  `reset` when the row is erased. Held requests are volatile: a restart
  drops them (the client asks again: `pg-playground-pages`);
- the steps, one kept row at a time, in this order, each skipped when
  its field of the row is none: (1) `KeptAct::RemoveProject { project }`;
  (2) `KeptAct::RemoveForge { forge }` kept and `removal::remove(d,
  env, forge, out)` decided with it (a `NoSuchForge` refusal: the step
  is done); (3) `KeptAct::RemoveAccount { account }`; (4) the row
  erased (`Key::Playground`). Each row's key is `kept::key(b"playground
  reset", &[number])` with the step's number. When a step's row is
  erased (its saved answer, or the removal's finish for step 2), in that
  decision the row is saved with the step's field none and the next
  step kept;
- at each start, once the core is ready (`Event::SendKept`): a row
  `Running` is reset as `EndPlayground` resets it, with no request (D5);
  a row `Resetting` goes on from its kept row, which `kept::send_all`
  sends again (a `Resetting` row with no kept row of its steps keeps
  its first step not done);
- `StartPlayground` while `Resetting` stays *not offered* (`pg-start`),
  and after the row is erased is offered again on a store with no real
  forge; its forge is numbered past the removed one;
- the engine's world: after a reset, nothing of the playground is read
  but what draft 4 and 5 keep: no `Playground` row, no `Forge` row of
  its forge, no `Kept` row, no deployment account of the scripted
  provider, no project named `Playground` in `me`, its chats ended;
- the tests below pass, and every existing test of the packages
  changed.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md`,
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (D3, D4,
  D5; section 2's `Playground` and `KeptAct`; section 4).
- `temper/docs/design/draft/playground-reset.md`, 4, 5, 6.2 and 9
  (the first open question: D5).
- `temper/docs/plan/playground-reset-r4/tasks/kept-acts.md`
  (`kept::{keep, key, answered, erase, send_all}`, the root's send),
  `tasks/rm-remove-forge.md` (`removal::{remove, finish}`, the finish
  erasing the reset's `RemoveForge` row), `tasks/pg-start.md` (the
  module this extends), `tasks/forge-asks.md` (`EndAnswer`),
  `tasks/pd-delete.md` (what the project's deletion does).

## Owns and may change

Owned: the `files` above; `playground.rs` (both crates) is
`pg-start`'s, extended here; the two test files are new.

Neighbour files, edits allowed:

- `temper-engine-domain-forge/src/kept.rs`: one call, after a row is
  erased in `answered` and in `erase`:
  `crate::playground::kept_erased(d, env, act, out)` with the erased
  row's act (thread `env` if the functions lack it), nothing else;
  `src/asks.rs`: the `EndPlayground` arm calls `playground::end`;
  `src/domain.rs`: the `SendKept` dispatch also calls
  `playground::restarted` after `kept::send_all`, and the held ends'
  field on `playground::State` needs no `domain.rs` edit;
  `src/tests.rs`: one `mod` line.
- `temper-engine-domain/src/route/playground.rs` is owned: its start
  drops itself when the connector answers its pending starts *not
  offered* on a reset.
- `temper/tests/engine/domain/tests/suite/main.rs`: one `mod` line;
  the world's config used by these tests grants `cancel`,
  `remove_project` and `remove_account` (`people::ActKind::{Cancel,
  RemoveProject, RemoveAccount}`) as `suite/demo.rs`' config grants
  the demo's.
- Compiler-forced arms anywhere in temper.

Pre-approved:

- if `kept::answered` already reports the erased act to a dispatcher
  (`rm-remove-forge`'s finish hook), add the playground's call beside
  it there instead;
- if a step's jig act is refused for good (`Unknown`, `Ended`: the
  project or account already gone), the erasure is the step's end, as
  D4's rule erases the row: no special case;
- if `removal::remove` must not run in the decision that keeps the
  `RemoveForge` row (it saves the `Forge` row), run it first, then
  keep the row, and say so.

## Steps

1. **Connector** (`playground.rs`):

   ```rust
   pub(crate) fn end(d: &mut Domain, env: &Env<Limits>, request: Token, token: Token, out: &mut Queue<Request>);
   /// A kept act of the reset was erased: the next step (draft 6.2).
   pub(crate) fn kept_erased(d: &mut Domain, env: &Env<Limits>, act: crate::KeptAct, out: &mut Queue<Request>);
   /// At each start, once the core is ready: a running playground is reset (D5).
   pub(crate) fn restarted(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>);
   fn next_step(d: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>);
   ```

   The held ends and their answers (`EndAnswer::Reset`) through the
   connect answer table, as `forge-asks`' stubs answer.
2. **Root** (`route/playground.rs`): the start's route stops on a
   resetting row.
3. **Tests:**
   - connector unit tests (`tests/playground_reset.rs`):
     `end_with_no_row_is_reset_at_once`;
     `end_keeps_the_project_first_and_reads_resetting` (saves:
     `Playground` resetting, `Kept` `RemoveProject`; sent;
     no answer yet);
     `each_erased_step_keeps_the_next_in_order` (erase `RemoveProject`:
     row without project, `RemoveForge` kept and the forge removing;
     the removal's finish erases it: row without forge, `RemoveAccount`
     kept; its erasure: `Key::Playground` erased and the held end
     answered `reset`);
     `a_step_with_no_field_is_skipped`;
     `a_second_end_waits_for_the_first`;
     `a_running_playground_found_at_a_start_is_reset` (restore a
     `Running` row: `restarted` keeps `RemoveProject`);
     `a_resetting_playground_found_at_a_start_goes_on` (restore
     `Resetting` and its `RemoveForge` row: nothing kept anew; the
     row sent again by `send_all`);
     `a_start_during_the_reset_is_not_offered`.
   - engine world (`tests/suite/playground_reset.rs`, config as
     `suite/playground.rs` builds it, with the grants):
     `a_reset_deletes_the_project_removes_the_forge_and_the_account`
     (start, a chat in the project, end: the answer `reset`; the store
     as the acceptance's last-but-one bullet reads it; the chat ended);
     `the_playground_starts_again_after_its_reset_on_a_new_forge`
     (number past the removed one);
     `an_ungranted_remove_project_keeps_the_reset_waiting` (no
     `remove_project` grant: the row stays `Resetting`, its
     `RemoveProject` kept, a note naming the grant; granted at the next
     start, it ends).
   - Narrow runs: `-p temper-engine-domain-forge --lib -E
     'test(playground_reset::) | test(playground::)'`; `-p
     temper-engine-domain-world --test suite -E 'test(playground_reset::)
     | test(playground::)'`.
4. `cargo fmt`; clippy on `temper-engine-domain-forge`,
   `temper-engine-domain`, `temper-engine-domain-world`; their focused
   tests and `temper-service`'s; `heavy cargo check --tests` for
   `temper`, `temper-system-world`, `temper-one-process-world`,
   `temper-engine-forge-world`, one at a time. Commit, push.

## Report

fab's sections; `## Interfaces`: `end`, `kept_erased`, `restarted`;
the kept keys' derivation; the grants the reset needs in `engine.acts`
(`cancel`, `remove_project`, `remove_account`, for `bx-init`).
