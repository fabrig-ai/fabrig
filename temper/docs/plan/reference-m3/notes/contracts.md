# reference-m3: contracts

The shapes that cross this plan's families, as its binding calls. A task builds its half to
this note and needs no other family's task for it, except where its spec's `needs` says so.
Each entry names the design that holds it in full; the design wins on detail, this note on
the names. Families: **appsettings** (jig's application settings), **kinds** (temper's kinds
1 to 3 on them), **change** (`change` naming its producer's charter and an LLM choice per
role; jig's check of a batch's LLM choices against its charter's models), **subagents**
(M4's sub-agents reachable from temper), **reference** (the reference's configuration).

## C1. An application's kind, declared (appsettings, kinds)

- jig's `domain/engine.md`, 17.5; `domain/connectors.md`, section 17.
- `jig_domain_core::settings::Kind { number: u32, scope: KindScope, connector: u16,
  value_bytes: u32, per_scope: u32, charters: u8 }` and `KindScope { Deployment, Project }`,
  derived `Clone, Copy, PartialEq, Eq, Debug`. `charters` is the most charter numbers one
  value names (0: its values name none). The application gives them as `kinds: Box<[Kind]>`
  beside its defaults when it builds the core (`Config`); its kinds' default values it gives
  the owning connector, never the core.
- A setting of an application's kind is `jig_domain_people::Setting::Application { kind: u32,
  scope: u32, number: u32 }`, scope 0 the deployment's, otherwise a project's number. jig's
  own kinds keep their arms.
- A connector's refusal of an item is `jig_domain_people::SettingFault::Judged`, beside the
  closed list of 17.1; the refusal carries the connector's words (bytes, UTF-8) and names the
  setting.

## C2. Judging and keeping, between the core and the owning connector (appsettings, kinds)

- jig's `domain/engine.md`, 17.5 and 17.6; `domain/connectors.md`, section 17. Asks go out on
  `jig_domain_core::effects::connector::Ask`, answers come back on
  `jig_domain_core::effects::connector::Event`, routed by the root to the connector the kind
  names, as a requirement's `Judge` and `Verdict` are.
- `jig_domain_core::settings::application::Item { kind: u32, scope: u32, number: u32, edit:
  Edit }`, `jig_domain_core::settings::application::Edit { Set(Box<[u8]>), Remove, Reset }`
  (in `settings::application`, since `settings::{Item, Edit}` name the core's own change
  items; `Kind` and `KindScope` are re-exported at `settings::`). A reset is judged by the connector as the default
  it keeps (a setting with no default: as a removal).
- `Ask::JudgeSettings { owner: Token, revision: u64, proposal: Option<u64>, items: Box<[Item]> }`,
  `revision` the one the change would make, `proposal` the accepted proposal whose kept values
  are judged; answered within the same decision's step by
  `Event::SettingsJudged { owner: Token, judged: Box<[Judged]> }`, one per item in the ask's
  order: `Judged::Accept { size: u32, charters: Box<[u32]> }` (the size of the value it would
  leave, and the charters it names), `Judged::Same` (it leaves the setting as it is) or
  `Judged::Refuse { why: Box<[u8]> }`.
- `Ask::KeepSettings { revision: u64, proposal: Option<u64>, items: Box<[Item]> }`, once the
  change commits, in the same decision; no answer. The connector saves `Set` and `Remove` as
  the settings' current changes (a removal as a change with no value) and erases the changes
  of those `Reset`; `proposal` names the accepted proposal whose kept values become them.
- `Ask::KeepProposalSettings { proposal: u64, items: Box<[Item]> }` in the decision that makes
  a proposal of settings (checked against the kinds' bounds alone);
  `Ask::DropProposalSettings { proposal: u64 }` when it is withdrawn or rejected.
- At a start, before its other records, the connector is given its kinds' defaults by the
  application and answers `Event::SettingsDefaults { defaults: Box<[DefaultState]> }`,
  `DefaultState { kind: u32, scope: u32, number: u32, size: u32, charters: Box<[u32]>,
  differs: bool }`, for every default it holds; the core sends
  `Ask::SaveSettingsDefaults { revision: Option<u64> }` in the decision that makes the
  release's revision (none: no revision was needed), where the connector saves them.
- A project's settings of a project-scoped kind are erased by their connector as part of the
  existing `ReleaseProject` ask (jig's `domain/people.md`, 5.2); the core erases its states
  of them in the same deletion.
- The appsettings family may add fields it finds the core needs (a key, a request token);
  it may not rename these or change their meaning. Anything more is a question.

## C3. temper's kinds 1 to 3 (kinds, change, reference)

- temper's `protocol/configuration.md`, 13.2; `domain/forge.md`, 8.9. Owned by the forge's
  connector. M3 declares kinds 1 to 3 alone; kinds 4 and 5 stay Not yet.
- **Kind 1,** the deployment's landing rules: scope 0, numbers 1 to `forge.landing_rules`, a
  value the encoding of `temper_engine_domain_forge::settings::LandingRule` in temper's codec.
  **Kind 2,** a project's landing rules: scope a project, the same value. **Kind 3,** the
  forge's charters: scope 0, number 1 the change's (its producing, repairs, resolutions and
  checks' gates, unless `change` names a producer, C4), number 2 its reviews'; the value a
  charter's number, a `u32` in temper's codec; declares `charters` 1. M5 adds number 3, the
  foreman's.
- **The forge's held charters:** `temper_engine_domain_forge::Domain::charters(&self) ->
  ForgeCharters`, `ForgeCharters { change: u32, review: u32 }`. Until the kinds family
  replaces its source, the root's change start keeps reading `Config`'s `forge_charter` and
  review charter 4; the kinds family moves both reads to `charters()` in one task.

## C4. `change`'s producer and LLM choices (change, reference)

- temper's `domain/agent.md`, 4.3.1; `domain/forge.md`, 8.1 and 8.9. Members of each change's
  object, beside M1's, both optional, both a plan's (only a run whose authority holds jig's
  control family sets them, as `landing` and `after`):
  ```
  {"words": "...", "landing": "goal", "producer": 6,
   "llm": {"producer": {"main": {"model": "gpt-6.1-sol", "effort": "high"}},
           "review": {"main": {"model": "gpt-6.1-sol", "effort": "medium"}}}}
  ```
- **`producer`:** a charter's number, the charter the change's producing task, its repairs,
  its resolutions and its checks' gates run under; left out, kind 3's number 1 (C3). M7b adds
  `"merge_job"` as a producer that is a job.
- **`llm`:** an LLM choice per role, each in jig's form (`{"main": {"model", "effort"},
  "sub_agents": [...]}`, jig's `protocol/configuration.md`, 3.1): `producer` for the producer,
  its repairs, its resolutions and its checks' gates; `review` for its agents' gates. A role
  left out takes its charter's default (jig's `domain/tasks.md`, section 3). M5 adds
  `foreman`.
- Rust: `temper_engine_domain::ChangeCall` gains `producer: Option<u32>` and
  `llm: ChangeLlm`, `ChangeLlm { producer: Option<jig_domain_tasks::LlmChoice>, review:
  Option<jig_domain_tasks::LlmChoice> }` (`Default`: both none). The change's row
  (`temper_engine_domain_forge::boundary`'s change row, temper's store) keeps
  `producing_charter` and `review_charter` as now, and gains `producing_llm` and `review_llm`,
  each an `Option<LlmChoice>`, given as each delegate's batch entry's `llm`.
- **Checked by the root** as it decides the change's delegate: the producer's charter one the
  newest settings offer and the run's authority may delegate to; each LLM choice as jig checks
  a batch's (C5), against the charter its role runs under. The procedure's authority grants
  delegation to the producer's charter and the review charter.

## C5. A batch's LLM choice against its charter's models (change)

- jig's `domain/tasks.md`, section 4; `protocol/configuration.md`, 3.1; `domain/engine.md`,
  17.4. jig's batch check refuses an agent task whose LLM choice names a model, main or
  sub-agent, that its charter's models (at the newest revision) do not allow, with
  `jig_domain_tasks::LlmFault::Charter`, as a template's and a move's are checked today.
  Every creator: a party's batch, a run's delegation, a procedure's step.

## C6. Sub-agents from temper (subagents, reference)

- temper's `protocol/setup.md`, 8.1; `client/tools.md`, 5.5; smith's `protocol/programs.md`,
  5.10.
- The box's agent profile (`temper-init`'s `template/agent.json`, `docs/examples/owned/agent.json`)
  declares `conversations` 3: a run and two sub-agents side by side.
- `temper ctl chat start` and `goal start` take sub-agents in their LLM choice:
  `--sub-agent MODEL[:EFFORT]`, repeatable, after `--model` and `--effort`, each becoming one
  of the LLM choice's `sub_agents`.
