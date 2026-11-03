# configuring-p3: contracts

The shapes that cross this plan's tasks, as its binding calls, checked against this trunk
(main at the cut, 0c28865613's parent). The design wins on detail, these notes on the names.
Each batch adds entries; none changes.

## C1. A charter's host families (charter-tools)

jig's `domain/engine.md`, 7.1: a run is offered the tools of each host family its charter
names and its authority holds; the charter's `tools` is a `ToolFamilies` mask, the same type
as an authority's tools.

- `jig_domain_core::RunPolicy` (`jig/crates/jig-domain-core/src/runs/state.rs`) gains, as its
  last member, after `call_timeout`:
  ```rust
  /// The host families its runs are offered, a mask of the configured `ToolFamilies` as a
  /// task's authority's tools are; a family it or the task's authority leaves out is not
  /// offered (domain/engine.md, 7.1).
  pub tools: jig_domain_tasks::Tools,
  ```
  It travels wherever `RunPolicy` travels: a charter setting's `policy`, a task's run, the
  run's assignment.
- `impl RunPolicy { pub const EVERY_FAMILY: jig_domain_tasks::Tools = jig_domain_tasks::Tools(u64::MAX); }`,
  documented "Every family: the identity under a task's authority, and what a decoder whose
  record carries no mask fills in."
- **The core decides nothing by it.** A call is decided by its task's authority alone, as now
  (jig's `domain/authority.md`, sections 3 and 8.5); the mask decides only what a run is
  offered, which is the host's (temper-engine-smith's) to apply.
- **Until each format carries it** (C2), a decoder of a `RunPolicy` that has no field for it
  fills `RunPolicy::EVERY_FAMILY`, and an encoder ignores it: no stored, wired or golden byte
  moves in `tools-surface`.
- **In temper's settings file** (`temper-settings-form`), a charter's value carries `"tools"`,
  an integer, the bits as an authority's `tools` has them (temper's
  `docs/help/reference/settings-fields.md`, `## authority`'s `tools` line); left out, it is
  `1023`, every jig family but settings (`temper_settings_form`'s constant
  `CHARTER_TOOLS_DEFAULT`), so that a file written before names no settings family.
- **temper offers by it the settings family alone** (temper's `docs/design/domain/agent.md`,
  4.3): `settings` and `propose_settings` are declared where
  `policy.tools.0 & authority.tools.0 & ToolFamilies::JIG.settings != 0`; every other host
  tool is declared as now, by the task's authority where it is today (message, plan).
