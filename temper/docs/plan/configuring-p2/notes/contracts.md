# configuring-p2: contracts

The shapes that cross this plan's parts, as its binding calls. Each entry names the design that
holds it in full; the design wins on detail, this note on the names. Parts: **policy-item** (a
project's own policy as an item of a change and of a proposal) and **restore** (restoring a
version). Both build on the reference's M3 (`temper/docs/plan/reference-m3/notes/contracts.md`,
C1 to C3): an application's kinds as settings, and temper's kinds 1 to 3 on them. Where this
note names a type M3 creates, M3's name holds.

## P1. A project's policy, an item of a change (policy-item, restore)

- jig's `domain/engine.md`, 17.2 and 17.6; `domain/people.md`, 5.2.
- **The setting's name:** `jig_domain_people::Setting::ProjectPolicy { project: u32 }`, beside
  `Charter(u32)`, `Template(u32)`, `Policy(u32)`, `Rules` and M3's `Application { kind, scope,
  number }`. It is no setting of 17.1's: no default, no state in the core's current settings,
  never in a charter version.
- **Its edit:** `Edit::Set(Value::Policy(..))` alone, the value a policy in a seed's format, as
  a policy projects are made from (`jig_domain_core::settings::Value::Policy`). `Removed` and
  `Reset` are refused `SettingFault::Shape`, naming the item; so is a second `ProjectPolicy` in
  one change or proposal. A project not live is refused as the people child refuses every
  request naming an unknown project.
- **Who:** an item of the project's scope, decided by its owners: `ChangeSettings` from a party
  who is not its owner is *not theirs*; beside any of the deployment's items it needs one party
  who owns both. A proposal naming it waits for those parties, and is refused at its entrance
  where none is.
- **Its check:** 17.1's check of a policy projects are made from, against the settings the
  change leaves, and the people child's checks of an owner's edit of the policy (every part a
  `PolicyChange` would check, whole); a policy equal to the project's is the same, counted
  toward *empty*.
- **Its commit:** the project's policy replaced whole in the change's decision, each part as
  the people child applies an owner's `ChangePolicy` (the open period's budget and the pools
  resized), and the revision's record keeping the item, its value whole and the project's
  policy it replaced (P2).
- **On the wire and in the store:** jig's client wire's `Setting` and the store's
  `settings-v1` `Setting` each gain `project_policy`, a record `{ project: u32 }`; its set value
  is the `policy` arm's. The review's item (`ReviewItem`) carries the policy now and as it
  would be, whole.
- **In temper ctl's settings file** (temper's `client/tools.md`, 5.3):
  `{"setting": {"kind": "project-policy", "project": P}, "set": POLICY}`, POLICY in the form
  `settings show policy NUMBER --json` prints; `temper-settings-form` decodes it to `Setting::
  ProjectPolicy { project: P }`, refusing `edit` and a second one before anything is sent.

## P2. What an item replaced (restore)

- jig's `domain/engine.md`, 17.10; `protocol/store.md`, 4.4 (the record's items "with the
  values they set and what each replaced").
- A revision's record keeps, for each item, beside its edit, what it replaced: the value whole
  (a charter, template, policy, the rules, a project's policy), that it followed its default,
  that it was removed, or that it was not offered; for an application's item, the state and the
  size alone, its value kept by its connector. A release's revision records the defaults it
  replaced the same way. Names are the restore part's first task's.
- **`Restore { revision: u32, reason: Box<[u8]> }`,** a party's request in the deployment's
  scope (jig's `domain/people.md`, 5.1; `client/domain/contract.md`, 3.2's `Restore`), answered
  a proposal of settings and the items it leaves out (a deleted project's settings and
  policy), or refused forgotten (naming the oldest kept), empty, stale or too large. On temper
  ctl, `revision restore REVISION [REASON]` (temper's `client/tools.md`, 5.3).
- What a connector keeps of what its kinds' items replaced, and the ask that hands it a
  restore's items, are jig's `domain/engine.md`, 17.5 and `domain/connectors.md`, section 17,
  M3's sections, written once M3 lands; until then no task builds them.
