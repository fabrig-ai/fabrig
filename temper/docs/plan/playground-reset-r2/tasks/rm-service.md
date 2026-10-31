+++
id = "rm-service"
part = "forge-remove"
kind = "feature"
needs = ["rm-domain", "fr-service", "bx-serve"]
files = [
  "temper/crates/temper-service/src/forge/removal.rs",
  "temper/crates/temper-service/src/tests/removal.rs",
]
effort = "high"
minutes = 90
+++

## Goal

The service's half of a forge's removal: the client's `RemoveForge`
admitted and answered; the root's released `RemoveGroup` sent to the
box and its `removed N` answering the kept act; the root's released
`EraseSecrets` erasing every secret item of the forge and answering the
kept act; and the removed forge retired from the service's runtime
(its connection, setup host run, tokens, worker identities, client
rows), and never built again at a start. With it, a set up cut short is
started over on the service side (draft 6.4): removing a forge the setup
host is setting up stops the run.

Acceptance (new tests, `temper-service`'s `src/tests/removal.rs`, on
the crate's in-process fixtures):

- `the_clients_remove_forge_is_admitted_and_answered_removing`: a
  deployment owner's wire `RemoveForge` reaches the engine as
  `forge::Ask::RemoveForge`, of the deployment's scope, and is answered
  `removing`; a forge not kept, `remove_refused(unknown)`; a person not
  an owner, refused as other deployment requests are.
- `remove_group_is_sent_and_removed_answers_the_kept_act`: under the
  box, `Request::RemoveGroup { forge }` writes `remove N` on the pipe
  (bx-serve's `remove_group`); the box's `removed N` reaches the engine
  as `forge::Event::GroupRemoved { forge }`; `failed N WHY` sends
  nothing and logs one note.
- `erase_secrets_erases_every_item_then_answers`: each of the forge's
  ten secret numbers (`temper_protocol_records::number(forge, item)`
  for every `Item`) is erased through the secrets store under the
  application owner, absent ones included, and only once each answered
  `Erased` does the engine get `forge::Event::SecretsErased { forge }`;
  the loaded secret rows of the forge are dropped.
- `a_removed_forge_leaves_the_runtime`: after the removal's commit the
  forge's connection is closed and not reconnected, the setup host holds
  no run or state for it, its GitHub tokens and worker identities are
  gone, and the forges watch lists it not.
- `removing_a_forge_being_set_up_stops_its_run`: an owned Forgejo
  mid-setup removed: its setup run ends without a `SetupEnd` that would
  mark it connected, and a new `SetUp` makes a new forge (a new number).
- `a_start_builds_no_removing_forge_and_resends_its_acts`: a store
  holding a `removing` record and its kept acts starts with the forge
  not built (no connection, no setup host entry) and the kept requests
  sent again.

## Inputs

- `temper/docs/plan/playground-reset-r2/notes/common.md` and
  `temper/docs/plan/playground-reset-r2/notes/scope.md`.
- `temper/docs/plan/playground-reset-r2/notes/contracts.md`, 2, 3 (the
  pipe's `remove N`) and 5.
- `temper/docs/design/draft/playground-reset.md`, 4 and 6.4.
- `temper/docs/design/protocol/credentials.md`, the records section
  (the secret numbers of a forge).
- `temper/docs/design/protocol/setup.md`, 6.6 (the cache) and 7.1.
- `temper/docs/design/domain/root.md`, 9.1.
- Names from other tasks, as merged in the trunk: D2's `crate::forge::registry::{Registry,
  Built, restored, made}` at `engine.registry` and the consumers' `add`
  (`crate::forge::add`, `crate::workers::add`, `crate::setup::add`,
  `crate::clients::add`); D3's `crate::box_groups::{remove_group,
  take_removed, GroupRemoved}`; rm-domain's root `Request::{RemoveGroup,
  EraseSecrets}` (a no-op arm in `src/route.rs` today) and forge events
  `GroupRemoved`, `SecretsErased`; cl-wire's `w::Request::RemoveForge`
  bridge in `src/family.rs` (answered `Undecodable`).
- The code: `src/forge/records.rs` (`Rows`, `Loader`), `src/setup/public.rs`
  (`erase`: the carried-row erase, whose owner routing this follows),
  `src/setup.rs` (`answer`), `src/setup/lifecycle.rs`, `src/forge.rs`
  (`ended`, the reconnect gate), `src/forge/tokens.rs`, `src/workers.rs`.

## Owns and may change

Owns the new module `src/forge/removal.rs` and its tests
`src/tests/removal.rs`.

May change, as the work needs (the service is fr-'s, then this part's):
`src/route.rs` (the two no-op arms become calls; drain
`box_groups::take_removed` wherever bx-serve's box outputs are drained),
`src/family.rs` (the `RemoveForge` request arm: cl-wire's bridge
replaced), `src/forge.rs`, `src/forge/records.rs`, `src/forge/registry.rs`
(skip `removing` records at `restored`; a `remove`), `src/forge/tokens.rs`,
`src/setup.rs`, `src/setup/*.rs`, `src/workers.rs`, `src/clients.rs` and
`src/clients/*.rs` (each consumer's `remove`), `src/engine.rs` (owner
routing of secrets answers), `src/box_groups.rs` (call sites only),
`src/tests.rs` and `src/tests/fixtures*.rs` (helpers). `notes/scope.md`
applies. Runs beside `pg-start` (domains, `settings.rs`, `startup/`) and
`pg-models` (`stock.rs`): do not edit those files but for forced arms.

## Steps

1. **Admission** (`family.rs`): `w::Request::RemoveForge(r)` becomes
   `root::Application::Forge(forge::Ask::RemoveForge { forge: r.forge })`
   as the other deployment-scoped forge requests are built there; the
   answer arm is rm-domain's.
2. **Retiring** (`src/forge/removal.rs`):

   ```rust
   /// playground-reset.md, 4: the service's half of a forge's removal.
   pub(crate) fn erase_secrets(engine: &mut Engine, forge: u16);   // root::Request::EraseSecrets
   pub(crate) fn erased(engine: &mut Engine, owner: Token, event: StoreEvent) -> bool; // true when it was ours
   pub(crate) fn remove_group(engine: &mut Engine, forge: u16);    // root::Request::RemoveGroup
   pub(crate) fn drain_removed(engine: &mut Engine);              // take_removed -> GroupRemoved
   pub(crate) fn retire(engine: &mut Engine, forge: u16);         // the runtime's inverse of registry `add`
   ```

   `erase_secrets` first calls `retire` (idempotent: the request comes
   after the removal's commit, and again at each start), then erases
   the ten numbers under one owner token of its own
   (`crate::tokens::FORGE`'s space, a number distinct from `LOAD` and
   the setup runs'), counting answers in a small map keyed by forge;
   the tenth `Erased` pushes `SecretsErased`. A store refusal or `Full`
   leaves it unanswered (resent at the next start) with a note.
   `retire` drops the forge from `engine.registry` and calls each
   consumer's `remove(engine, forge)`, added beside its `add`
   (`forge`: close and forget the connection and its installation
   tokens; `setup`: stop and forget any run, intent and pasted values
   of the forge; `workers`: drop its git identity from the workers'
   credentials; `clients`: drop its client rows and sign-in metadata).
   The service's ended gates (`crate::forge::ended` and its callers)
   stay as they are: they read the root's general `forge_ended`.
3. **At a start:** `registry::restored` builds no `removing` record;
   the kept acts are resent by the domain (rm-domain), and arrive here
   as requests again.
4. **Tests** as the acceptance names. Then, one at a time:
   - `heavy cargo nextest run -p temper-service --lib -E 'test(/removal::/)'`;
   - `heavy cargo nextest run -p temper-service`;
   - `heavy cargo nextest run -p temper-system-world -p temper-one-process-world -p temper-real-loop-world -p temper`
     (the service's dependents, focused profile);
   - `cargo fmt`; `heavy cargo clippy -p temper-service --all-targets -- -D warnings`
     (and any other package edited);
   - `python3 -I -B scripts/old_forms.py` from `temper/`.
5. Commit and push.

## Report

The standard report, plus: each consumer's `remove` and what it drops;
the owner token used for the erase; whether the secrets store answers an
absent number `Erased`; how a setup run is stopped (and whether the box
group's removal and the run's stop race).
