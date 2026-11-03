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

## C2. A charter's brief room (room)

temper's `domain/agent.md`, 4.3: a charter is refused at a start and at a change's three
entrances when it leaves its brief less than 2,048 bytes of smith's host part.

- `temper/crates/temper-engine-domain/src/room.rs`, `pub mod room;` in the crate's `lib.rs`:
  ```rust
  /// A run's least brief room (domain/agent.md, 4.3).
  pub const BRIEF_ROOM: u64 = 2_048;

  /// What a charter's non-brief host part costs, by its parts, in smith's bytes: built once at a
  /// start by temper-engine-smith from its declarations and smith's measures; the domain reads it.
  #[derive(Clone, Copy, PartialEq, Eq, Debug)]
  pub struct HostPart {
      /// smith's host part in the receiving profile (51,717 in small).
      pub bound: u64,
      /// What every run pays: smith's own text and the largest finish and deliver schemas.
      pub fixed: u64,
      /// Every host tool declared whatever the charter's `tools`, at the largest offer.
      pub declared: u64,
      /// The settings family's tools, paid where the charter's `tools` names the family.
      pub settings: u64,
      /// smith's own families' tools, each paid where the charter's policy turns it on.
      pub inspect: u64,
      pub modify: u64,
      pub shell: u64,
      pub agents: u64,
  }

  /// A charter that leaves its brief less than `BRIEF_ROOM`.
  #[derive(Clone, Copy, PartialEq, Eq, Debug)]
  pub struct Short {
      pub charter: u32,
      /// Its brief's room, 0 where its non-brief part alone passes the bound.
      pub room: u64,
      pub bound: u64,
  }

  impl HostPart {
      /// `fixed + declared + instructions`, with `settings` where
      /// `policy.tools.0 & ToolFamilies::JIG.settings != 0` and each smith family the policy
      /// turns on.
      pub fn non_brief(&self, policy: &jig_domain_core::RunPolicy) -> u64;
      /// `bound - non_brief`, 0 at or past the bound.
      pub fn room(&self, policy: &jig_domain_core::RunPolicy) -> u64;
      pub fn check(&self, charter: u32, policy: &jig_domain_core::RunPolicy) -> Result<(), Short>;
  }

  impl Short {
      /// "charter N leaves its brief R bytes of its host part's B; it needs at least 2,048",
      /// the figures with thousands' commas.
      pub fn words(&self) -> Box<[u8]>;
  }
  ```
- `temper_engine_smith::room::host_part(limits: &smith_domain_run::Limits) -> temper_engine_domain::room::HostPart`,
  `bound` = `limits.host_part`.
- **Where it is checked** (`room-checks`): at a start, each charter of the current settings
  (the defaults with the stored changes over them); at the engine's `ChangeSettings` and
  `ProposeSettings` routes, each charter an item sets, refused as
  `jig_domain_people::Refusal::Judged { at, setting, why: short.words() }`, `at` the item's
  index; at a run's `propose_settings`, answered as a host error,
  `SettledAnswer::Host { error: true, body: short.words() }`, the run going on. A stored
  proposal's acceptance, a restore and a reset are not checked. An engine given no
  `HostPart` (no agent configured) checks nothing.
