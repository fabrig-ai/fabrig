# configuring-p2-build: contracts

The shapes that cross this plan's tasks, as its binding calls. configuring-p2's
`temper/docs/plan/configuring-p2/notes/contracts.md`, P1 and P2, hold whole; this note adds the
names that one left to this cut, checked against this trunk (main, M3 landed, merged in). The
design wins on detail, these notes on the names. Each batch adds entries; none changes.

## B1. A project's policy, the setting (policy-item)

- `jig_domain_people::Setting::ProjectPolicy { project: u32 }`, declared last, after
  `Application { .. }` (`jig/crates/jig-domain-people/src/settings.rs`). Its only edit is
  `jig_domain_core::settings::Edit::Set(Value::Policy(..))`; the value is the
  `ConfiguredPolicy` a `Setting::Policy(n)` sets, a policy projects are made from.
- No new `SettingFault`: a `Removed`, `Reset`, a second `ProjectPolicy` in one change or
  proposal, and project 0 are `SettingFault::Shape`, naming the item; a party not the
  project's owner is refused as the people child refuses any request of a project's scope from
  a party who does not own it (*not theirs*), never by a new fault; a project not live is the
  people child's refusal of an unknown project.
- It never enters the core's current settings (`Current`'s defaults or changes), a charter
  version, a release's revision or a task's settings: the core's matches that walk the current
  settings treat it as absent.
- The tasks child's settings heads (`jig_domain_tasks::Setting`) gain no arm: the core leaves
  it out of a proposal's heads, as it leaves out an application's item
  (`jig_domain_core::settings::proposal_heads`).

## B2. What an item replaced, in the core (restore)

- `jig_domain_core::settings::Replaced { Value(Value), Default, Removed, NotOffered }`, beside
  `Edit` in `jig/crates/jig-domain-core/src/settings.rs`: the setting as it was before the
  item: a change's value whole, that it followed its default, that it was removed (a change
  with no value), or that it was not offered (no default and no change).
- `jig_domain_core::settings::RevisionItem { setting: Setting, edit: Edit, replaced: Replaced }`;
  `Revision.items` becomes `Box<[RevisionItem]>`. A party's `Item { setting, edit }` and a
  proposal's items stay as they are.
- `Current::replaced(&self, setting: Setting) -> Replaced`, read before the change applies.
  A release's revision records each item `Replaced::Default` (engine.md, 17.10).
- A project's policy item records `Replaced::Value(Value::Policy(..))`, the project's policy
  before the change, whole.
- An application's item of a revision: `Revision.applications` becomes
  `Box<[application::RevisionRecorded]>`, `RevisionRecorded { kind: u32, scope: u32, number:
  u32, edit: SizedEdit, replaced: application::Replaced }`, `enum Replaced { Changed { size:
  u32 }, Default, Removed, NotOffered }`, from the state the core keeps of it
  (`application::State`'s `follows` and `size`); the value is its connector's (B3). A
  proposal's `application::Recorded` stays as it is.

## B3. What a connector keeps, and a restore's items (restore)

- On `Ask::KeepSettings { revision, .. }` (unchanged), a connector keeps, beside each change
  it saves, the value the setting held before, when it held one (a change with a value),
  keyed by kind, scope, number and `revision`. A release's `SaveSettingsDefaults` keeps none.
- `Ask::ForgetReplaced { before: u64 }`: erase every value kept as replaced by a revision below
  `before`, in the decision that erases those revisions' records from the history.
- `Ask::RestoreSettings { owner: Token, proposal: u64, items: Box<[settings::application::Restored]> }`,
  `Restored { kind: u32, scope: u32, number: u32, edit: RestoredEdit }`,
  `RestoredEdit { Set { revision: u64 }, Remove, Reset }`: keep each item as the proposal's,
  a `Set` taking the value kept as replaced by `revision`, as `KeepProposalSettings` keeps a
  proposal's items; answered `Event::SettingsRestored { owner: Token, judged: Box<[Judged]> }`,
  one `Judged` per item in order: `Accept { size, charters }`, or `Refuse { why }` past the
  kind's bounds (the core then refuses the restore with it and sends `DropProposalSettings`).
- temper's forge keeps them as `SettingRow`s of a new slot `SettingSlot::Replaced(u64)` (the
  revision), in temper's store `SettingSlot { .., replaced: Number }`; jig's test connector in
  a record of its own beside `SettingChange`.

## B4. A restore, on the people boundary (restore)

- `jig_domain_people::Ask::Restore { revision: u32, reason: Box<[u8]> }`, in the deployment's
  scope, admitted as a party's `Propose` of settings is.
- `Outcome::Restored { proposal: u64, to: u32, left_out: Box<[Setting]> }`: the proposal, the
  role it went to, as `Outcome::Proposed`, and each setting left out (a deleted project's
  settings and its `ProjectPolicy`).
- Refusals: forgotten, a new `Refusal::Forgotten { oldest: u32 }`; empty,
  `Malformed::SettingsUnchanged`; stale, `Refusal::Stale { newest }`; too large,
  `Malformed::Settings { setting, fault: SettingFault::Count }`, `setting` the first item past
  the bound; a revision 0 or past the newest, `Refusal::Stale { newest }`.
- The reason: `restores revision R: ` then the party's reason, cut to the reason's bound at a
  character's boundary.
- **Too large is counted before anything is left out:** every item the records after R give,
  a deleted project's included, counts toward a whole set and one project's policy, so
  `left_out` holds at most a whole set's items and one policy (the store's `list` bound,
  `SETTINGS_ITEMS + 1`).

## B5. On the client wire and in temper ctl (policy-item, restore)

- jig's client wire, `client-v19` in place (as M3's `apps-wire`): `Setting.project_policy:
  ProjectPolicySetting { project: u32 }`, its set value the `policy` arm's; `ReviewItem`'s
  item of a project's policy carrying the policy now and as it would be, whole, in the
  existing `ReviewValue`; `Ask.restore: Restore { revision: u32 reason: bytes }`;
  `Outcome.restored: OutcomeRestored { proposal: u64 to: u32 left_out: list Setting }`;
  `Refusal.forgotten: RefusalForgotten { oldest: u32 }`; `RevisionItem` gains `replaced:
  RevisionReplaced { value: SettingValue, too_large: u32, default, removed, not_offered,
  application: u32 }` (`application` the size of an application's replaced value).
- jig-client-domain's contract: `Setting::ProjectPolicy { project }`, `Ask::Restore { revision,
  reason }`, `Outcome::Restored { proposal, to, left_out }`, `Refusal::Forgotten { oldest }`,
  `RevisionItem.replaced: RevisionReplaced` (the arms above).
- temper ctl: a settings file's item `{"setting": {"kind": "project-policy", "project": P},
  "set": POLICY}` (configuring-p2's P1); `revision restore REVISION [REASON]`, keyed
  `Required` as `revision advance` is, answered `proposed restoring revision R as proposal
  P to role K`, then a line per item left out; `revision show`'s items with `was: ...`.

## Task names in arms

`policy-surface`'s arms name `policy-store` and `policy-wire`: those are `settings-store` and
`settings-wire`, which carry both parts' store and wire.
