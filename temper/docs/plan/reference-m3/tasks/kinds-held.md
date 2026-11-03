+++
id = "kinds-held"
part = "kinds-forge"
kind = "feature"
needs = ["kinds-store"]
files = ["temper/crates/temper-engine-domain-forge/src/kinds.rs", "temper/crates/temper-engine-domain-forge/src/tests/kinds.rs", "temper/crates/temper-engine-domain-forge/src/settings.rs", "temper/crates/temper-engine-domain-forge/src/domain.rs", "temper/crates/temper-engine-domain-forge/src/adoption.rs", "temper/crates/temper-engine-domain-forge/src/lib.rs", "temper/crates/temper-engine-domain-forge/src/tests.rs"]
effort = "high"
minutes = 120
+++

# kinds-held: the forge judges, keeps and holds its kinds 1 to 3

## Goal

The forge's side of contract C2, on its own typed values (`kinds-values`) and its family
`Setting` (`kinds-store`): it judges a change's items of kinds 1 to 3 from its own state,
keeps a committed change's items, a proposal's items and the stock defaults in its rows,
erases a deleted project's, holds the current rules per scope and its two charters (C3's
`charters()`), and wakes each live change a changed scope covers. Methods only: the root calls
none of them yet (`kinds-route`, `kinds-start`), and no change's behaviour moves.

Acceptance:

- A new module `src/kinds.rs` (`mod kinds;` in `lib.rs`), module doc citing
  `temper/docs/design/protocol/configuration.md`, 13.2 and `domain/forge.md`, 8.9 and
  section 16, holding an `impl Domain` block with:
  - `pub fn judge_settings(&self, bounds: &settings::Bounds, items: &[settings::Asked]) ->
    Box<[settings::Judged]>`, one answer per item in order, reading nothing but the forge's
    rows, its stock and its arguments. A `Reset` is judged as the default it would leave (the
    stock's value, or a removal where the stock has none); `Set` and `Remove` as they are;
    each through `settings::judge`. A refusal is `Refuse(refusal)`. An accepted item that
    leaves its setting as it is (13.2: a `Set` whose value and size equal the stored change's,
    a `Remove` where the stored change is a removal, a `Reset` where no change is stored) is
    `Same`; any other accepted item is `Accept { size, charter }`, `size` the value's (0 for a
    removal) and `charter` the charter a kind 3 value names.
  - `pub fn keep_settings(&mut self, env: &Env<Limits>, proposal: Option<u64>, items:
    &[settings::Asked], out: &mut Queue<Request>)`: per item, the edit is the item's, or with
    `proposal` the one kept under `SettingSlot::Proposal(proposal)` for that setting where
    there is one; `Set` and `Remove` save the setting's `Change` row
    (`Request::Save { record: Stored::Setting(..), release: None, read_afresh: None }`),
    `Reset` erases it where there is one. Then every row of `Proposal(proposal)` is erased.
    The held state is rebuilt; each live change into a repository whose project's rules, or
    the deployment's, changed is woken now (`change_due` at `env.wall`, as a restored change
    is); a change of kind 3 alone wakes none.
  - `pub fn keep_proposal_settings(&mut self, proposal: u64, items: &[settings::Asked], out:
    &mut Queue<Request>)`: saves each item as a `Proposal(proposal)` row with its edit;
    `pub fn drop_proposal_settings(&mut self, proposal: u64, out: &mut Queue<Request>)`:
    erases every row of that proposal.
  - `pub fn give_settings_defaults(&mut self, stock: Box<[settings::Stock]>) ->
    Box<[settings::DefaultState]>`: keeps the stock (replacing any given before), answers one
    state per stock entry in the stock's order, `differs` where the stored `Default` row of
    that setting is absent or holds another value or size, and rebuilds the held state.
  - `pub fn save_settings_defaults(&mut self, out: &mut Queue<Request>)`: saves a `Default` row
    for each stock entry that differs from its stored one, and erases each `Default` row the
    stock no longer gives.
  - `pub fn erase_project_settings(&mut self, project: u32, out: &mut Queue<Request>)`: erases
    every row of kind 2 at scope `project`, every slot, and drops the project's held rules; the
    deployment's kinds 1 and 3 stay (13.2, **Erased with their project**). Called from
    `adoption::release` (`Event::ReleaseProject`), one line, before its answer.
  - `pub fn charters(&self) -> ForgeCharters` (C3): kind 3's numbers 1 and 2 as held; a number
    with no value held is 0, which no charter is.
  - `pub fn landing_rules(&self, project: u32) -> (&[settings::Held], &[settings::Held])`: the
    deployment's held rules and the project's, each in number order (for `kinds-requirements`).
  - `pub fn setting(&self, kind: u32, scope: u32, number: u32) -> Option<&settings::SizedValue>`:
    the current value (the stored change over the default; none where removed or unset), and
    `pub fn proposal_settings(&self, proposal: u64) -> Box<[(settings::SettingKey,
    settings::Edit)]>`: a pending proposal's items as kept (for the clients' reads, later).
- The held state: `Domain` gains the stock given at this start and the held rules
  (`settings::HeldRules { deployment: Box<[Held]>, projects: Map<u32, Box<[Held]>> }`, in
  `settings.rs`, its project map sized by `judge_projects`) and charters. Current is: a
  setting's `Change` row where there is one (a removal: no value), else the stock given at
  this start, else (before the stock is given, during the restore) its stored `Default` row.
  It is rebuilt from `setting_rows` and the stock whenever either changes, and once at the end
  of the restore (`restored`), so a restored change is judged by the rules as they are
  (`domain/forge.md`, section 16).
- A kept row past `setting_rows`' room, or a project's rules past `judge_projects`, panics as a
  writer's bug: jig's check bounds the items before they reach the forge.
- Tests in `src/tests/kinds.rs` pass (Steps); the crate's focused tests pass unchanged.

## Inputs

- `temper/docs/design/protocol/configuration.md`, 13.2 (**Judged by the forge**, its bullets
  as this cut wrote them; **Kept by the forge**; **Erased with their project**; **The forge's
  charters are read when a change is made**); `temper/docs/design/domain/forge.md`, 8.9 and
  section 16 (the paragraph on loading its kinds' values before its records);
  `temper/docs/design/protocol/store.md`, section 2, family 20.
- `temper/docs/plan/reference-m3/notes/common.md`; `notes/contracts.md`, C2 (what each ask
  does: the forge's methods are its half, typed) and C3.
- The status entries stay: `kinds-route` and `kinds-start` close them.

## Owns and may change

- Owns `kinds.rs` and its tests, new. `settings.rs`: `HeldRules` alone. `domain.rs`: the new
  fields and their construction, the rebuild in `restored`, nothing else. `adoption.rs`: the
  one call in `release`. `lib.rs` and `tests.rs`: one `mod` line each.
- Options: if `restored` is not where every restored row has arrived, rebuild where the
  restore's last row is known (the restore's end) and say where. If a live change's project is
  read elsewhere than its repository's row, use what `requires` uses.
- Not this task's: requirements 5 and 6, and the gates from held rules (`kinds-requirements`);
  the root's routing and its restart order (`kinds-route`, `kinds-start`).
- Other plans: reference-m2, -m5, -m6, -m7b change `domain.rs` on their trunks: keep its hunks
  to the fields, construction and the rebuild's one call.

## Steps

1. `HeldRules`, the fields, the rebuild; the module's methods.
2. Tests in `src/tests/kinds.rs`, on the crate's test fixtures (`fixtures.rs`, as
   `src/tests/requires.rs` builds a domain and an adopted repository), the stock kind 1 rule
   1 (everywhere, CI and up to date), kind 3 number 1 charter 2 and number 2 charter 4:
   - `a_change_of_rules_is_judged_kept_and_held`: a project rule judged `Accept`, kept (one
     `Save` of a `Change` row), then in `landing_rules(1)`; `charters()` is `{2, 4}`.
   - `items_that_leave_a_setting_as_it_is_are_the_same`: a set equal to the stored change, a
     removal of a removed rule, a reset with no change.
   - `a_reset_is_judged_as_its_default`: kind 3 number 1 reset is `Accept` naming charter 2;
     a deployment rule with no stock default reset is an accepted removal.
   - `a_refused_item_names_what_fails`: kind 3 charter past `charters`: `Refuse(Charter)`.
   - `an_accepted_proposal_keeps_its_kept_values`: proposal 7's items kept, then
     `keep_settings(Some(7), ..)` saves their values as changes and erases proposal 7's rows.
   - `a_dropped_proposal_erases_its_items`.
   - `defaults_say_which_differ_and_are_saved`: restored `Default` rows, one equal and one
     not, and one the stock no longer gives: the states, then the saves and the erase.
   - `a_change_of_rules_wakes_the_live_changes_it_covers`: a live change in project 1; project
     1's rules kept wake it, project 2's do not, kind 3's do not.
   - `a_released_project_erases_its_rules`: `Event::ReleaseProject` erases project 1's kind 2
     rows and keeps kind 1's.
   - `restored_rows_are_held_at_the_end_of_the_restore`: `Setting` rows restored, then
     `Restored`: `landing_rules` and `charters()` read them.
   Narrow: `heavy cargo nextest run -p temper-engine-domain-forge --lib -E 'test(/kinds::/)'`
   (check the count: ten).
3. fmt, old forms, clippy on `temper-engine-domain-forge`; its focused tests; `heavy cargo
   check -p temper-engine-domain --tests`.
4. Commit ("Let the forge judge, keep and hold its kinds") and push.

## Report

The implementer's text's report, plus: where the rebuild runs at the restore's end.
