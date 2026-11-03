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
