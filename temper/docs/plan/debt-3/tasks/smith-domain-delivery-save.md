+++
id = "smith-domain-delivery-save"
part = "smith-lifecycles"
kind = "feature"
needs = []
files = [
  "smith/crates/smith-local-domain/src/domain.rs",
  "smith/crates/smith-local-domain/src/delivery/machine.rs",
  "smith/crates/smith-local-domain/src/reconcile.rs",
  "smith/crates/smith-local-domain/src/chat/machine.rs",
  "smith/crates/smith-local-domain/src/parent_tests.rs",
]
effort = "high"
minutes = 150
+++

# smith D002, the local Domain: one delivery, one state enum

## Goal

`Domain` holds the life of its one delivery in four `Option` fields: `delivery`, `saving`,
`delivery_owner` and `reconciling`. Together they name six states and allow more: a stale
`delivery_owner` outlives its delivery. Replace the four with one enum, `Delivering`, with its
transitions and `follow`, beside the machine it extends, after moving the one test that writes
them off private state (`skein/docs/foundation/programming-model.md`, 5.4). This is the local
`Domain`'s delivery share of `smith/docs/development/debt/D002-lifecycles-in-flags.md`; the item
stays open: leave its file.

Behaviour stays unchanged, with the two cells the table below names as changed (a reachable abort
each). Both follow the design: `smith/docs/design/protocol/hosts.md`, 5.5 ("the next start
settles it") and `smith/docs/design/domain/host.md`, 8 ("Late events are expected").

Acceptance:

- `rg -n 'delivery_owner|domain\.saving|domain\.reconciling|domain\.delivery\b|running_delivery'
  smith/crates/smith-local-domain/src` prints nothing.
- `Domain` has 24 fields (27 at base) and 5 `Option` fields (9 at base): `effort`, `choice`,
  `closing_last`, `transcript`, `answer_wall`. `interactive` stays the one `bool`.
- `rg -n 'State::Saving' smith/crates/smith-local-domain/src/delivery/machine.rs` prints nothing.
- `rg -n 'domain\.delivering = ' smith/crates/smith-local-domain/src` hits only the entrances
  and `follow` of Steps, 3, all in `delivery/machine.rs`: every other handler returns its target.
- `smith-local-domain --lib` runs 85 tests (81 at base): one rewritten, four new. Every other
  test passes unchanged.
- `smith-local-world --test suite`, `smith-local-service --lib` and
  `smith-local-process-world --test suite` pass unchanged.
- No public item, wire shape, stored format or design text moves.

## Inputs

- `temper/docs/plan/debt-3/notes/common.md`.
- `smith/AGENTS.md`.
- `smith/docs/development/debt/D002-lifecycles-in-flags.md`, whole.
- `skein/docs/foundation/programming-model.md`, 1 and 5.4, whole.
- `smith/docs/design/protocol/hosts.md`, 5.5; `smith/docs/design/domain/host.md`, 2, 8 and 9.2;
  `smith/docs/design/domain/run.md`, 8.2 and 8.4.
- The form: `smith/crates/smith-local-domain/src/delivery/machine.rs` and `reconcile.rs` as they
  stand; `smith/crates/smith-local-domain/src/chat/machine.rs`, `follow_chat`.

## Owns and may change

Owned: `delivery/machine.rs` and `reconcile.rs`, whole. Edits elsewhere, and no others:

- `domain.rs`: the four fields become `delivering: Delivering`; `Domain::new`; the `delivery` and
  `reconcile` imports; `git_result` loses its `reconciling` branch; `running_delivery` goes.
- `chat/machine.rs`: the `saving` condition in `select_chat`; the intent arm of `resume_loading`;
  the `if let Some(record) = domain.saving.take()` block of `store_failed` becomes one call; the
  `use super::{..}` list.
- `parent_tests.rs`: one test rewritten, four added, a shared config helper.

Not yours: `boundary.rs`, `delivery.rs`, `chat.rs`, `person.rs`, `smith/docs/design/**`, any status
file, the debt item. No design text changes: the design names none of these states.

Options, each with its default:

- `delivery_saved` cannot keep the `domain.delivering = match` form for `ResumeLoading`, since
  `resume_loading` itself writes `domain.delivering`: use a plain `match` there, as sketched.
- The git test needs a head the harness does not survey: insert it with
  `domain.chat.live_mut().heads.insert(..)` in the test; `Live` is the chat's, not this machine's.
- A test outside `parent_tests.rs` fails: none pins an abort and none reaches the two changed
  cells, so read its log and ask.

## Steps

1. **Tests off private state.** `told_and_withdrawn_do_not_abandon_a_durable_delivery` reads
   `domain.delivery_owner`, and `Domain::running_delivery` writes it. Move the plain-workspace
   config of `a_delivery_call_keeps_its_fields_owner_and_deadline_in_local_policy` into a helper
   and rewrite the test on real events: `parent::Request::Called` (call 55); `Request::PlainStatus`;
   `Event::PlainStatus { owner, changed: true }`; `Request::SaveDelivery` with an intent;
   `parent::Request::Told` and `Withdrawn`, with `out` staying empty; `Event::DeliverySaved`;
   `Request::SaveDelivery` with the answer. Delete `running_delivery`. It passes on the old fields.
   Commit on its own.

   `heavy cargo nextest run -p smith-local-domain --lib -E 'test(told_and_withdrawn)'`

2. **The enum,** in `delivery/machine.rs` (no new file: a new `#[path]` would raise
   `path_attribute` in `smith/scripts/old-forms.txt`):

   ```rust
   /// The one delivery this host holds, live or loaded, and its one store operation.
   pub(super) enum Delivering {
       /// No delivery in the host's hands.
       Idle,
       /// A live call, awaiting one git or filesystem terminal.
       InPlace(InPlace),
       /// A loaded intent, awaiting one git terminal.
       Reconciling(Reconcile),
       /// One record awaiting its store terminal.
       Saving { record: DeliveryRecord, after: AfterSave },
   }

   /// What a record's store terminal continues to.
   pub(super) enum AfterSave {
       /// The intent is durable: the call's git effects run.
       Execute(InPlace),
       /// The answer is durable: it returns to this call.
       Return { owner: Token },
       /// The reconciled answer is durable: loading goes on.
       ResumeLoading,
   }

   impl Delivering {
       /// Whether a record awaits its store terminal.
       pub(super) fn saving(&self) -> bool;
   }
   ```

   `State::Saving` goes: the outer state says it. `InPlace.owner` is the one owner of a live call.
   `Delivering` is one lifecycle, not a product: a live delivery and a reconcile never overlap,
   since a loaded intent needs `Gone`, and `Gone` needs the delivery call's terminal
   (`smith/crates/smith-inline-agent/src/stop.rs`, `settle`;
   `smith/crates/smith-host-domain/src/domain.rs`, `agent.calls.is_empty()`).

3. **Transitions,** in the idiom of 5.4: move the state out leaving `Delivering::Idle`, assign the
   match's result, then `follow`.

   ```rust
   // Entrances, one per event, each ending in `follow`:
   pub(super) fn begin_delivery(domain, env, name, owner, change, deadline, out);
   pub(super) fn plain_status(domain, env, owner, changed, out);
   pub(super) fn git_result(domain, env, owner, result, out);       // InPlace or Reconciling
   pub(super) fn delivery_saved(domain, env, name, out);
   pub(super) fn reconcile_intent(domain, env, record: DeliveryRecord, out); // from resume_loading
   pub(super) fn save_failed(domain, env, out);                     // from store_failed

   /// What the state implies, in one place: a transient inner state moves on.
   fn follow(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
       let state = core::mem::replace(&mut domain.delivering, Delivering::Idle);
       domain.delivering = match state {
           Delivering::InPlace(call) => follow_in_place(domain, env, call, out),
           Delivering::Reconciling(reconcile) => reconcile::follow(domain, env, reconcile, out),
           state @ (Delivering::Idle | Delivering::Saving { .. }) => state,
       };
   }

   fn follow_in_place(domain, env, call: InPlace, out) -> Delivering;   // today's `follow`
   fn advance_delivery(domain, env, call: InPlace, out) -> Delivering;
   fn save_intent(domain, call: InPlace, out) -> Delivering;
   fn save_answer(name, owner, delivery, landed, out) -> Delivering;    // today's `save_delivery`
   fn deliver_unknown(domain, env, owner, directory) -> Delivering;     // returns Idle
   /// The one place a record goes to the store.
   pub(super) fn save(record: DeliveryRecord, after: AfterSave, out) -> Delivering;

   // reconcile.rs: `Reconcile::new` starts in `State::Advance`, so `follow` runs `reconcile_next`.
   pub(super) fn reconcile_git(domain, env, reconcile: Reconcile, owner, result, out) -> Reconcile;
   pub(super) fn follow(domain, env, reconcile: Reconcile, out) -> Delivering;
   fn reconcile_next(domain, env, reconcile: Reconcile, out) -> Delivering;
   fn recovery_unknown(domain, out) -> Delivering;                      // returns Idle
   ```

   `InPlace::new` starts in `State::Advance`, and `delivery_saved` sets `State::Advance`, stage
   `Execute` and `next = 0` for `AfterSave::Execute`. Handlers keep their bodies and their order of
   effects; those that write `domain.delivery`, `domain.saving` or `domain.reconciling` return the
   state instead.

   The table. "Abort" is `unreachable!` with today's words, or the words the row gives. Every cell
   but the two marked **changed** does what it does at base.

   | state | `Called` | `Git` | `PlainStatus` | `DeliverySaved` | first `StoreFailed` | intent loaded |
   |---|---|---|---|---|---|---|
   | `Idle` | cached: answer the call, `Idle`. Saved intent: unknown, `Idle`. No message: `Saving`, `Return`. Otherwise `InPlace` | abort | abort | after a store failure ignore, **changed**; otherwise abort | stays | `Reconciling` |
   | `InPlace` | abort, "one delivery in flight" | today's cells: `InPlace`, `Saving` or `Idle` (unknown) | same | abort | stays: a delivery in flight is never abandoned | abort |
   | `Reconciling` | abort | today's cells: `Reconciling`, `Saving` (`ResumeLoading`), or `Idle` after `recovery_unknown` | abort | abort | stays | abort |
   | `Saving`, `Execute` | abort | abort | abort | record into `records`; `InPlace`, execute stage | answer `Failed { directory: 0, Broken }` to `call.owner`; `Idle` | abort |
   | `Saving`, `Return` | abort | abort | abort | record into `records`; `Fact::DeliveryReturned`; answer the call; `Idle` | `DeliveryUnsaved` if `Delivered`; answer the call; `Idle` | abort |
   | `Saving`, `ResumeLoading` | abort | abort | abort | record into `records`; `Idle`; `resume_loading`, **changed** (see below) | `DeliveryUnsaved` if `Delivered`; `Idle` | abort |

   - **Changed, the stale owner.** At base `delivery_saved` keeps `delivery_owner` when an intent
     is saved, and `deliver_unknown` never clears it. After an unknown in the execute stage, in a
     chat, the next `Load` reconciles the intent and `DeliverySaved` finds the stale owner:
     `deliver_to_agent` aborts on `child.run().expect(..)`, the child being gone. Here a reconciled
     save has no owner by construction, so loading goes on.
   - **Changed, the late terminal.** The store runs its queue on after a failure
     (`smith/crates/smith-local-service/src/host_io/store.rs`, `store_pass`), so a `SaveDelivery`
     queued behind a failed `SaveTurn` can land after `StoreFailed` has settled the save. At base
     `delivery_saved` aborts on `saving.take().expect(..)`. Here `Idle` × `DeliverySaved` is
     `assert!(domain.ending.store_failed(), "DeliverySaved answers one SaveDelivery")`, as
     `turn_saved` ignores its late terminal.
   - **A later `StoreFailed` stays ignored:** `store_failed`'s guard returns first, `save_failed`
     is not reached, and `Saving` stands. This is base behaviour, so keep it and do not widen it.
   - `Saving`, `Execute` × `StoreFailed` drops `InPlace` where base leaves it on `domain.delivery`
     unread; nothing outside sees it.

   The order inside `store_failed` stands: `Notice::Failure(Failure::Store { .. })`, then
   `save_failed`, then `Drop` or `Exit`. Inside `save_failed`: the notice, then the answer.

4. **The readers.**
   - `domain.rs`: `git_result` keeps the head survey's branch first, and sends the rest to
     `delivery::git_result`.
   - `select_chat`: `|| domain.delivering.saving()`.
   - `resume_loading`: the intent arm is `reconcile_intent(domain, env, record.clone(), out);
     return;`.
   - `store_failed`: `save_failed(domain, env, out);` where the block stood.
   - The module docs of `machine.rs` and `reconcile.rs` say what `Delivering` keeps
     (`smith/AGENTS.md`). `lib.rs`'s table stands as it is.

   The strict subset holds: no own-enum `if let` or `let else`, no `matches!`, no `_` arm.

5. **Tests,** in `parent_tests.rs`, on `running(config)`, whose child swallows a parent answer for
   an unknown handle:
   - `an_unknown_delivery_settles_at_the_next_load_of_the_same_invocation`: a git directory, with
     its head inserted; `Called`; `Git` status; `DeliverySaved` of the intent; `Git`
     `Uncertain` on the commit; `Git` `Failed` on the inspection; `Answered` and `gone`; `Loaded`
     with the intent record; `Git` `Inspected { named: true }` under owner 0; `DeliverySaved`.
     `domain.chat.phase.is_idle()`. It aborts at base.
   - `a_delivery_save_landing_after_a_store_failure_is_ignored`: the intent's `SaveDelivery`,
     `StoreFailed { turn: Some(2), .. }`, then `DeliverySaved`; `out` stays empty after the last.
     It aborts at base.
   - `a_store_failure_while_a_delivered_answer_saves_tells_the_person`:
     `Failure::Store`, then `Failure::DeliveryUnsaved`.
   - `a_store_failure_while_a_reconciled_answer_saves_exits_with_its_notice`: `Domain::new`,
     `resume`, `Loaded` with a plain intent, `StoreFailed`: `Failure::Store`,
     `Failure::DeliveryUnsaved`, `Exit`.

   ```
   heavy cargo nextest run -p smith-local-domain --lib
   heavy cargo nextest run -p smith-local-world --test suite
   heavy cargo nextest run -p smith-local-service --lib
   heavy cargo nextest run -p smith-local-process-world --test suite
   ```

6. `cargo fmt -p smith-local-domain`; `python3 -I -B scripts/old_forms.py` from `smith/`;
   `heavy cargo clippy -p smith-local-domain --all-targets -- -D warnings`. Commit and push.

## Report

- `Domain`'s fields and `Option`s before and after, by count.
- The two changed cells, with the two tests that abort at base: say whether you ran them at base.
- Any cell you placed otherwise than the table.
- How `Delivering`'s size was handled, if clippy spoke.
