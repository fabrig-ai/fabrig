+++
id = "fe-kept-root"
part = "rf"
kind = "feature"
needs = ["fe-records", "pd-vocabulary"]
files = ["temper/crates/temper-engine-domain/**"]
effort = "high"
minutes = 90
+++

## Goal

The root takes the connector's kept acts and the reset's acts to their takers, and their
answers back (draft `temper/docs/design/draft/playground-reset.md`, sections 4 and 6.2;
`notes/contracts.md` T5, T6, J1, J5, J8; `notes/fe-surface.md` section 5): a kept `Cancel`
goes to the core as the deployment's `Act::Cancel`, `RemoveGroup` and `EraseSecrets` go to
serve, the reset's `RemoveProject` and `RemoveAccount` go to the core as the deployment's
acts; each core answer comes back as `KeptActAnswered` or `ResetActed`, kept or done. At
each start, once the core is open, the connector is asked to send them again. The core's
`ReleaseProject` reaches the connector and its answer reaches the core.

Acceptance:
- `forge::Request::SendKeptAct(row)`:
  - `Cancel { project, task }` sends `core::Event::Act { reply_to, connector:
    forge_connector, key: row.key, act: people::Act::Cancel { project, task, reason:
    b"Forge removed" }, carried: None }`, with a fresh reply token mapped to the row.
  - `RemoveGroup` sends `root::Request::RemoveGroup { forge: row.forge }`; `EraseSecrets`
    sends `root::Request::EraseSecrets { forge: row.forge }`, each through the now door.
    Their answers (`GroupRemoved`, `SecretsErased`) reach the connector from serve as
    `root::Event::Forge`, unchanged by the root.
- `forge::Request::ResetAct(act)` sends `people::Act::RemoveProject { project }` or
  `people::Act::RemoveAccount { account }` as the deployment's act, keyed
  `reset_key(b"project", project)` / `reset_key(b"account", account)`: the first 16 bytes of
  SHA-256 over `b"temper playground reset\0"`, the number's eight big-endian bytes and the
  purpose, so a resend after a restart carries the same key and jig answers it from its
  retained answer.
- The core's answer (`Held::Acted`, routed today as `Delivery::DemoActed`) is matched to
  its map (demo end act, kept act or reset act) and classified:
  - kept (sent again at the next start): `Again(_)`; `Refused(NotOffered)` (no grant: a
    `DemoNote` line names the grant word, `cancel`, `remove_project` or `remove_account`,
    and `engine.acts`); `Refused(Role { held: None })` (no first owner, a line says so);
  - done: for `Cancel`, `Compound { .. }`, `Refused(Unknown)`, `Refused(Ended { .. })`; for
    `RemoveProject`, `ProjectRemoved { .. }` and `Refused(Unknown)`; for `RemoveAccount`,
    `AccountRemoved { .. }` and `Refused(Unknown)`; any other refusal is done with a line
    naming it (as the demo's `end_acted` does);
  - then `forge::Event::KeptActAnswered { key, keep }` or `forge::Event::ResetActed { act,
    keep }`.
- `RestartStep::Open` queues `forge::Event::SendKeptActs` after `SendDemoActs`.
- The core's `Ask::ReleaseProject { project }` for the forge connector becomes
  `forge::Event::ReleaseProject { project }` (replacing the arm jig's task added that
  answers at once); `forge::Request::ProjectReleased { project }` becomes the core's
  `Event::ProjectReleased { project, connector: forge_connector }`, in the same decision.
- `root::Domain::playground(&self) -> Option<forge::Playground>` (pub): the connector's
  row, for serve's playground read (sv's).
- Unit tests below pass; every test of the crate and of `temper-engine-domain-world`
  passes.

## Inputs

- `temper/docs/plan/playground-reset-r6/notes/common.md`; `notes/contracts.md` T5, T6,
  J1, J5, J8; `notes/fe-surface.md` sections 3 to 5.
- jig's `jig/docs/design/domain/people.md`, 5.3 (the deployment's acts, their keys and
  retained answers).
- The demo's end acts, the model to follow:
  `temper/crates/temper-engine-domain/src/route/demo.rs` (`send_end_act`, `end_acted`,
  `end_note`) and `src/restart.rs` (`RestartStep::Open`).

## Owns and may change

- Owns: `temper/crates/temper-engine-domain/`.
- May change: nothing else.
- Pre-approved options:
  - Module: default, a new `src/route/kept.rs` holding the two reply maps (sized
    `limits.forge.tasks + 2 * limits.forge.forges` and 2), `send_kept`, `send_reset`,
    `acted`, and the release routing; `domain.rs`'s `Delivery::DemoActed` arm dispatches
    to the demo's `end_acted` or to `kept::acted` by which map holds the token. If renaming
    `Delivery::DemoActed` to `Delivery::Acted` makes the dispatch clearer, do it inside this
    crate (its readers outside are the client and system worlds' exhaustive matches:
    compiler-forced) and say so.
  - Lines: default, `Delivery::DemoNote { line }` carries them, prefixed `forge removal: `
    or `playground reset: `, at most 256 bytes, secret-free, as `end_note` builds them.
  - `ReleaseProject` for a connector other than the forge's: answer it at once as jig's
    arm did.

## Steps

1. `src/route/kept.rs` as above; `forge_route.rs`'s `SendKeptAct`, `ResetAct` and
   `ProjectReleased` arms call it (they are no-ops since `fe-surface`/`fe-records`);
   `route/asks.rs`'s `ReleaseProject` arm; `restart.rs`'s `Open`.
2. `domain.rs`: `pub fn playground(&self) -> Option<forge::Playground>` delegating to
   `self.forge.playground().copied()`.
3. Tests in a new `src/tests/kept.rs`, driving `forge_route::outputs` with a crafted
   `Queue<forge::Request>` and the core's answers as the existing route tests do
   (`src/route/forge_route/tests.rs`):
   `a_kept_cancel_reaches_the_core_as_the_deployments_act_with_its_key`;
   `group_and_secret_acts_go_to_serve`;
   `reset_acts_carry_keys_derived_from_their_numbers`;
   `again_no_grant_and_no_owner_keep_the_act_and_name_why`;
   `ended_unknown_and_compound_answers_are_done`;
   `the_start_asks_for_kept_acts_after_the_core_opens`;
   `a_release_reaches_the_connector_and_its_answer_the_core`.
4. Run, one at a time: `heavy cargo nextest run -p temper-engine-domain --lib`; `heavy
   cargo nextest run -p temper-engine-domain-world --test suite`; `heavy cargo check
   --workspace --all-targets`; `cargo fmt --all`; `heavy cargo clippy -p
   temper-engine-domain --all-targets -- -D warnings`; `python3 -I -B
   scripts/old_forms.py` from `temper/`.
5. Commit, push your branch.

## Report

- How `Held::Acted` answers are told apart, and the names of the jig variants you used
  (`Act::Cancel`, `Act::RemoveProject`, `Outcome::ProjectRemoved`, `Ask::ReleaseProject`,
  `Event::ProjectReleased`) as they stand in jig.
