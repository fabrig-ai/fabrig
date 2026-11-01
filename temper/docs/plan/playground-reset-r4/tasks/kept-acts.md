+++
id = "kept-acts"
part = "forge-removal"
kind = "feature"
needs = ["fr-seed", "jig-acts"]
files = [
  "temper/crates/temper-engine-domain-forge/src/kept.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/kept.rs",
  "temper/crates/temper-engine-domain-forge/src/tests.rs",
  "temper/crates/temper-engine-domain/src/route/kept.rs",
  "temper/crates/temper-engine-domain/src/domain.rs",
  "temper/crates/temper-engine-domain/src/route/held.rs",
  "temper/crates/temper-engine-domain/src/route/forge_route/tests/kept.rs",
]
effort = "high"
minutes = 55
+++

## Goal

The `Kept` family's mechanism (contracts, D4), generalised from the
demo end's `DemoAct` route: a decision writes `Kept` rows; each is sent
as its act, a jig deployment act (`Cancel`, `RemoveProject`,
`RemoveAccount`) or the box's `remove`, at once and again at each start
once the core is ready; each is erased on its saved answer by the
`DemoAct` rule as it stands; `KeptAct::RemoveForge` is the connector's
own and is answered by the removal (`rm-remove-forge`). The demo's own
end keeps working on `DemoAct` until the playground part removes it.

Acceptance:

- `kept::keep(d, acts: &[crate::Kept], out)` saves each row
  (`Stored::Kept`), holds it in `d.kept`, and sends it
  (`Request::SendKept`) unless it is a `RemoveForge`;
  `kept::key(kind: &[u8], numbers: &[u64]) -> [u8; 16]` derives a row's
  key deterministically (as the root's `end_key` does for the demo:
  `temper-engine-domain/src/route/demo.rs`), so a decision that keeps
  the same act twice keeps one row.
- `kept::send_all` (on `Event::SendKept`, each start) sends every row
  but `RemoveForge`s; `kept::answered(key, keep)` erases the row
  (`Request::Erase { key: Key::Kept(key) }`) when `keep` is false;
  `kept::box_answered(forge, removed)` erases the forge's `BoxRemove`
  row when `removed`, and keeps it otherwise (it goes again at the next
  start: `notes/progress-box.md`'s contract problem, accepted); and
  `pub(crate) fn erased_for(d, forge) -> bool` says whether a forge's
  own rows (`Cancel` rows of its changes, its `BoxRemove`) are all gone,
  for the removal to answer its `RemoveForge`. `kept::erase(d, key, out)`
  erases one row (the removal's own `RemoveForge`).
- The root (`route/kept.rs`): `send` makes the jig act and sends it as
  `send_end_act` does (a reply token held in the root, `Work::Core(
  core::Event::Act { .. })`, the forge connector's number, the row's
  key): `Cancel { project, task }` as `people::Act::Cancel { project,
  task, grace: the conversation template's grace (as the demo's close
  finds it), reason: "Removed with its forge" }`, `RemoveProject {
  project }`, `RemoveAccount { account }`; `BoxRemove { forge }` as the
  root's `Request::BoxRemove { forge }`; `acted` maps the act's outcome
  to `keep` by `end_acted`'s rule: `Again` keeps; `Refused(NotOffered)`
  keeps with a note naming the missing grant (`cancel`,
  `remove_project`, `remove_account`, `engine.acts`); `Refused(Role {
  held: None })` keeps; `Refused(Unknown | Ended { .. })`, the
  compound's and the removal's answers erase (contracts 4: Cancel's
  *ended* and *unknown* are done); any other refusal erases with a note.
  The note reuses the demo's `Delivery::DemoNote` line form.
- The root's held acts (`route/held.rs`, `core::Held::Acted`) go to
  `kept::acted` for a kept reply token and to the demo's path otherwise.
- The tests below pass; the demo's end tests pass unchanged.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md` and
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (D4, section 2's
  `KeptAct`, section 4's `Act::Cancel` answers).
- `temper/docs/design/protocol/setup.md`, 8.4 (the demo's end acts, the
  rule this generalises).
- The code this generalises: `temper-engine-domain-forge/src/demo.rs`
  (`acts`, `send_acts`, `act_answered`),
  `temper-engine-domain/src/route/demo.rs` (`send_end_act`,
  `end_acted`, `end_key`, `end_note`), `route/held.rs`.
- The merged `fr-surface` (the stubs and variants) and `jig-acts`
  (`Act::Cancel`, `Act::RemoveProject`, their outcomes) reports.

## Owns and may change

Owned: the files in the header. In the root's `domain.rs`: two fields
(the reply table `kept_replies: Map<Token, forge::Kept>`, capacity
`kept_bound`, and its sequence) and their construction only.

Neighbours: `temper-engine-domain/src/route/forge_route/tests.rs`: the
`mod kept;` line.

Pre-approved options:

- If the demo's `end_key` and `end_note` are private to `route/demo.rs`,
  copy their few lines into `route/kept.rs` rather than editing
  `route/demo.rs` (the playground part removes it).
- If a kept reply token must be told apart from the demo's by value,
  tag it with a high bit (`1 << 62`) and say so.

## Steps

1. Connector `kept.rs` (module comment cites D4 and setup.md, 8.4): the
   functions above, bodies of `fr-surface`'s stubs.
2. Root `route/kept.rs`: `send`, `acted`; the reply table; `held.rs`'s
   dispatch.
3. **Tests**: connector `tests/kept.rs`:
   `kept_rows_are_saved_and_sent_but_remove_forge`,
   `every_row_is_sent_again_at_a_start`,
   `a_kept_answer_erases_its_row_and_a_kept_one_stays`,
   `a_failed_box_remove_stays_kept`,
   `erased_for_tells_when_a_forges_rows_are_gone`,
   `the_same_act_kept_twice_is_one_row`;
   root `route/forge_route/tests/kept.rs`:
   `a_kept_cancel_becomes_the_deployment_cancel_and_its_ended_answer_erases`,
   `an_ungranted_kept_act_stays_and_notes_its_grant`,
   `a_kept_box_remove_asks_the_box_link`,
   `kept_acts_go_again_after_a_restart` (the root's restart test shape
   that `SendDemoActs` has).
4. `cargo fmt`; `heavy cargo clippy -p temper-engine-domain-forge -p
   temper-engine-domain --all-targets -- -D warnings`; their focused
   tests; `heavy cargo check --tests -p temper-service -p
   temper-engine-domain-world`. Commit and push.

## Report

fab's sections, and `## Interfaces`: `kept::{keep, key, send_all,
answered, box_answered, erased_for, erase}`, the root's `send`/`acted`,
and the reply-token rule.
