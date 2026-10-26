# Session 09: templates, policies' overrides, and operations per request

The configuration's half of jig's plumbing (jig's `docs/design/STATUS.md`,
section 3, step 2: "templates in the configuration and policies'
overrides"; `protocol/configuration.md`'s row: "the `templates` member
and policies' overrides ... operations per request checked at start").
jig's configuration gains a `templates` member, the application's shapes
of work as data, with jig's chat and goal among them; it replaces the
`chats` member, whose charter and grace the engine's chat code then takes
from template 1 until jig 16 removes that code. A project's policy
overrides a template's executor, budget, deadline and default priority;
the core keeps the templates and the overrides only to serve each
project's merged templates in `me`. The configuration refuses a start
whose operations per request cannot close a conversation or reorder a
project's goals in one request. Four increments; the first needs jig 02.1
and jig 04.1, as the session's start does.

## Read first

- jig's `docs/design/protocol/configuration.md`: 1 (the "Templates are
  data" item), 3 (the `templates`, `projects` and `charters` rows), 3.1,
  6 (the operations, compound and grace items), 8, 9 (the last item).
- jig's `docs/design/domain/authority.md`, 6 (the overrides item);
  `domain/people.md`, 5.2 (its last sentences) and 7; `domain/core.md`,
  7.3 (the template and jig's defaults items); `domain/engine.md`, 11.1
  and 13 (the templates item).
- jig's `docs/design/client/domain/contract.md`, 3.6 and 5.1 (`me`, a
  project's document).
- `findings/engine-agents-store-config.md`, sections 1 (the templates
  and operations items), 3.
- jig 04's file, 4.1 (`tasks::Limits::grace`, `charters::chats`) and
  4.6 (`people::Limits::operations`, the core's `operations` ≤ `tasks`).
- skein's `programming-model.md`, 6.3; `testing-strategy.md`, 2.2.

## What the design asks

- **A template** (configuration.md 3.1): "a name, for showing; the
  executor: a charter ... or 'a charter or procedure the role may create
  with', the creator's choice; the result contract; authority: the
  creator's role narrowed, by default one level shallower in
  delegation; the budget: an amount; `AsAllowed`, with an optional cap;
  or the creator's to give ...; a deadline: none; a span from creation;
  or the creator's to give; wake and park: the charter's; tracked, with
  a default priority; the flags ...; the label ...; the grace its helper
  cancels with, or none." "Numbered, never renumbered."
- **Checked at start** (3.1): "numbers unique; each charter an executor
  names configured; each grace within the tasks child's limit on a
  grace's span; each seeded policy's overrides as above."
- **Overrides** (3.1; authority.md 6): "A project's policy may override,
  per template number, the executor, the budget and its cap, the
  deadline and the default priority ... Never the flags, the label or
  tracked ... An override naming a template the configuration does not
  define is refused when the policy is changed, and a seeded policy's
  refuses the start."
- **Served, never used** (engine.md 11.1): "The core keeps the
  configuration's templates ... and each project's policy's overrides of
  them ... only to merge them, per project, into `me`. No decision reads
  one".
- **Operations per request** (configuration.md 6): "at least the larger
  of *delegates per task* plus one and *goals per project*, both the
  tasks child's ..., or the start is refused, naming the three."

## Choices this plan makes

- **Where the types live.** The overridable fields' types
  (`TemplateExecutor`, `TemplateBudget`, `TemplateDeadline`) and
  `TemplateOverride` are the people child's, beside `PolicyValue`;
  `Template` is the core's, since it holds a `tasks::Contract` and jig
  02's flags.
- **The JSON an operator writes** (configuration.md 10, which writes
  this form, jig's chat and goal among it, since the design answer to
  jig 09.1's question of 2026-10-16T14:57:59Z): a
  template is
  `{"number": 1, "name": "chat", "executor": {"charter": 1}, "contract":
  {"report": {"words": 512}}, "shallower": 1, "budget": {"as_allowed":
  null}, "deadline": "none", "tracked": null, "conversation": true,
  "listed": false, "retention": "kept", "label": 1, "grace_ms": 5000}`,
  each member required, `null` where none; the forms: executor
  `{"charter": n}` or `"chosen"`; contract `{"report": {"words": n}}`,
  `{"verdict": {"choices": [{"code": c, "words": n, "followups": n,
  "fields": [{"name": s, "max": n}]}]}}` (`tasks::Verdict`,
  `jig-domain-tasks/src/boundary.rs`) or `{"change":
  {"connector": c, "kind": k, "words": n}}` (an approval names one
  change, so no template holds one); budget `{"amount": n}`,
  `{"as_allowed": cap or null}` or `"given"`; deadline `"none"`,
  `{"span_ms": n}` or `"given"`; tracked `null` or `{"priority": n}`;
  retention `"kept"` or `"forgotten_once_read"`. An override, in a
  project's policy's `templates` list: `{"template": 2, "executor": null,
  "budget": null, "deadline": null, "priority": 5}`.
- **jig's defaults are written, not implied.** Each fixture writes the
  chat and goal templates of 3.1's table (the chat: charter 1, a report,
  one level shallower, `AsAllowed` uncapped, no deadline, untracked,
  conversation, kept, label 1, grace 5000 ms; the goal: chosen, a report,
  one level shallower, given, given, tracked at priority 1, listed, kept,
  label 2, no grace); an application drops one by leaving it out.
- **Transitional until jig 16:** the engine's chat code reads
  `Settings.chat` and `Settings.close_grace` (9 readers in
  `jig-domain-core`), which 9.1 fills from template 1's charter and
  grace; a configuration whose template 1 names no charter or no grace is
  refused, `What::Refers` at `templates`. jig 16 removes both settings,
  their readers and this check with the engine's chat code (jig 04's
  file, "Rules": "jig 09 replaces `close_grace` with the chat template's
  grace and jig 16 removes the rest").
- **Limits:** a new core limit `templates` (templates configured, and
  overrides per policy): 8 in the small profile, 16 in the standard one;
  a template's name within `documents.text_bytes`.
- **No format version.** jig's configuration has none
  (configuration.md 9: "A configuration's format version, once a
  deployment's configuration must survive a release that changes it" is
  open), so no increment bumps one. People v3 (jig 02.3's) carries the
  overrides.
- **An override's priority on an untracked template** serves nothing:
  `merged` leaves the template untracked (a question below).

## Rules that bind it

- **Format versions** (README, Rules): people v3 is jig 02.3's; 9.2
  edits it in place. Regenerate with `SKEIN_CODEGEN_WRITE=1 cargo
  nextest run -p jig-store-codecs-world --test drift`, once.
- **No shape of work grows** (code-organization.md, 6): `Settings.chat`,
  `Settings.close_grace`, `OfferedCharter` and `FrameProjectRole.chat_budget`
  keep their readers as they are; nothing new reads a template in a
  decision (engine.md 11.1: "No decision reads one").
- **Files over 800 lines at the base get at most 40 lines each**
  (code-organization.md, 7): here `jig-domain-core/src/tests.rs` (2987),
  `jig-domain-people/src/domain.rs` (1884). New code goes in
  `jig-protocol-config/src/templates.rs`,
  `jig-domain-people/src/templates.rs` and
  `jig-domain-core/src/templates.rs`.

## Increments

### 09.1 `templates` in jig's configuration, in place of `chats`

Scope: configuration.md 3, 3.1, 6 (a template's grace), 8: the
`templates` member parsed into `jig_domain_core::Template` values and
checked at start (numbers nonzero and unique, a charter executor
configured, the contract valid and not an approval, the grace within the
tasks child's `grace`, the name within `documents.text_bytes`, at most
`templates`); `Settings.chat` and `Settings.close_grace` taken from
template 1; the `chats` member and `Chats` removed.
Needs: jig 02.1's label and flags (`tasks::Flags`, the retention class)
and jig 04.1's `tasks::Limits::grace`, both at the session's start.
Files:
- `crates/jig-protocol-config/src/`: `templates.rs` (new), `lib.rs`
  (`Engine.templates` in `chats`'s place; the member), `charters.rs`
  (`Chats`, `chats` go, as jig 04.1 left them), `core.rs` (:54-59),
  `problem.rs` (`What::Grace { template: u32 }`), `profile/small.rs`
  and `profile/standard.rs` (core `templates`, and their doc lines),
  `overrides.rs` (`templates` by name), `tests.rs`, `tests/templates.rs`
  (new), `tests/fixtures.rs` (:58), `tests/charters.rs` (:30's test
  moves to templates.rs), `tests/domain.rs` (:69-74; jig 04.1's
  `a_chat_close_grace_past_the_tasks_limit_refuses_the_start` moves to
  templates.rs), `tests/overrides.rs` (the names table).
- `crates/jig-domain-people/src/templates.rs` (new), `lib.rs`.
- `crates/jig-domain-core/src/templates.rs` (new: `Template`),
  `routing/vocabulary.rs` (`Limits.templates`), `memory.rs`
  (`worst_case` is `None` when `templates` is 0), `lib.rs`.
- The `chats` member's users, 10 files in jig (from `git grep -l -E
  '"chats"|b"chats"|\bChats\b|charters::chats|close_grace_ms|engine\.chats'
  -- '*.rs'`): the 6 above in `jig-protocol-config` (`tests/domain.rs:248`,
  jig 04.1's test, writes `close_grace_ms`), and the JSON fixtures
  `testing/jig-test-service/src/tests/fixtures.rs:49`,
  `tests/channel/src/fixtures.rs:78`,
  `tests/client-protocol/src/native/config.rs:43` (each writes the chat
  and goal templates in `chats`'s place, their reports' `words` 32, within
  these fixtures' `"tasks.result_bytes":32`), and
  `tests/client-protocol/src/front.rs:263` (`chats_made`, a name only:
  unchanged).
- The core `Limits` literals (6 files: `jig-domain-core/src/tests.rs:117`,
  `jig-protocol-store/src/tests/fixtures.rs:504`,
  `tests/core/src/world.rs:116`, `tests/store/src/root.rs:101`, and the
  two profiles), each gaining `templates: 8`, the small profile's value,
  but the standard profile, whose `CORE` spreads `..small::CORE`
  (`profile/standard.rs:98-115`) and writes `templates: 16`, as Choices
  gives it.
- Downstream: the repository downstream of jig writes `chats` in one
  fixture and has one core `Limits` literal, and matches every
  `config::What` twice (its service's `startup/problems.rs:81-103` and
  `:128-170`: an arm for `What::Grace` in each), absorbed at its repin.
Shape:
- `jig_domain_core::Template` as sketched; `Limits.templates: u32`.
- `jig_domain_people::{TemplateExecutor, TemplateBudget,
  TemplateDeadline}` as sketched.
- `jig_protocol_config::Engine.templates: Box<[jig_domain_core::Template]>`.
- `jig_protocol_config::templates::templates(document, at, path,
  charters, limits, problems) -> Option<Box<[Template]>>` (crate-private),
  as sketched; problems: `What::Bound` (number 0, a name past
  `documents.text_bytes`, more than `templates`), `What::Unique`
  (number), `What::Refers` (a charter not configured), `What::Form` (an
  approval contract, an unknown form), `What::Grace { template }` (a
  grace past `tasks::Limits::grace`).
- `templates::chat(&[Template]) -> Option<(u32, Duration)>`: template
  1's `Charter(n)` executor and its grace; `core.rs` sets
  `Settings.chat` and `Settings.close_grace` from it. Transitional:
  removed by jig 16.
Test shape:
- `crates/jig-protocol-config/src/tests/templates.rs` (new; `mod
  templates;` in tests.rs). Fixtures: `fixtures::{ENGINE_SMALL, with,
  engine, path}` (`tests/fixtures.rs:17` and the helpers above it),
  whose `ENGINE_SMALL` writes the two templates. Limits: the small
  profile (`"limits": {"profile": "small"}`): `grace` 600 s, `templates`
  8. Reference test:

  ```rust
  use crate::{Segment, What, tests::fixtures};

  #[test]
  fn a_template_grace_past_the_tasks_limit_refuses_the_start() {
      let document = fixtures::with(fixtures::ENGINE_SMALL, br#""grace_ms": 5000"#, br#""grace_ms": 600001"#);
      let problems = fixtures::engine(&document).expect_err("refused template");
      let problem = problems.problems.get(0).expect("one problem");
      assert_eq!(problem.what, What::Grace { template: 1 });
      assert_eq!(problem.path, fixtures::path(&[b"templates"]).child(Segment::Index(0)).member(b"grace_ms"));
  }
  ```
- Also there: `the_chat_and_goal_templates_parse_with_their_defaults`
  (template 2: `Chosen`, `Given`, `Given`, `tracked: Some(1)`, listed,
  label 2, no grace), `a_template_numbered_twice_is_refused`,
  `a_template_naming_an_unknown_charter_is_refused`,
  `an_approval_contract_in_a_template_is_refused`,
  `a_configuration_whose_template_one_names_no_charter_is_refused`.
- `tests/domain.rs`: `the_cores_chat_settings_take_the_provider_charter_and_grace`
  keeps its assertions (charter 1, 5 s), now from template 1.
- `testing/jig-test-service/src/tests/startup.rs` (the pattern of
  `an_in_process_engine_address_naming_no_listener_is_refused`, :5):
  `a_template_grace_past_the_tasks_limit_refuses_the_service_start`.
- Ceilings: unit under 20 ms.
Proof: world. `cargo nextest run -p jig-one-process-world -E
'test(a_person_s_chat_is_answered_by_the_in_process_worker)'` once (it
passes at the base in 0.28 s): the reference service starts from the
channel world's configuration, which writes `templates` and no `chats`,
and a person's chat runs under template 1's charter and is answered;
under 1 s. jig-test's process tier is not the proof: its
`a_killed_jig_test_restarts_and_the_work_is_found` fails at the base
("configuration JSON: missing field `prices`", in its agent's
configuration).
Don't: read a template in a core decision; add a template field to a
wire record (jig 14's for requests, jig 22's for documents); keep
`chats` beside `templates`.
Size: at most 260 lines of production code.
Removes: the `chats` member, `jig_protocol_config::Chats` and
`charters::chats`.

### 09.2 Policies' overrides of a template's executor, budget, deadline and priority

Scope: authority.md 6, configuration.md 3 and 3.1: a project's policy
holds its overrides of templates (`PolicyValue.templates`), changed as
one list by `PolicyChange::Templates`, kept by the core beside the
permissions (`permission_roles`' pattern), saved in people v3, restored
at start, shown in the project's document's policy (contract.md 5.1);
a change naming a template not configured is refused; a stored override
naming a template the configuration no longer defines is restored and
serves nothing (configuration.md 3.1 and 8); a seeded policy's overrides
are parsed and checked at start. `Settings.templates` keeps the
configured templates.
Needs: jig 02.3's people family at version 3.
Files:
- `crates/jig-domain-people/src/`: `templates.rs` (`TemplateOverride`),
  `policy_value.rs` (`PolicyValue.templates`, `PolicyChange::Templates`,
  `policy_bytes`, `policy_change_bytes`), `lib.rs`.
- `crates/jig-domain-core/src/`: `policy.rs` (`apply`, `snapshot`,
  `restore`, `change_policy`, `restore_policy` carry the overrides; a
  `Templates` change naming a template not configured is
  `Refusal::Unknown`, more than `templates` is `Refusal::Limit { which:
  LimitKind::Policy }`; `restore_policy` admits an override naming a
  template not configured), `lib.rs`
  (`Config.template_overrides`, `Settings.templates`, the core's map;
  at most 40 lines), `documents/project_note.rs` (`Policy.templates`),
  `memory.rs` (the templates and overrides in `worst_case`).
- `crates/jig-protocol-config/src/`: `projects.rs` (`Project.templates`;
  the policy's `templates` member), `templates.rs` (the override's
  parse), `domain.rs` (`whole`: an override naming no configured
  template, `What::Refers`), `core.rs` (`Config.template_overrides`,
  `Settings.templates`), `tests/domain.rs`, `tests/templates.rs`.
- `crates/jig-protocol-store/`: `schema/people-v3.schema` (records for
  the override and its fields; `templates: list 16 TemplateOverride` in
  `PolicyValue`; `templates` in `PolicyChange`), `src/codec/people/policy.rs`,
  the generated family, the goldens, `src/tests/fixtures.rs:369`, and
  `src/tests/format.rs`'s five expected values of the fixture store's
  value and page bounds, which the policy's `templates` raises (`:76`,
  `:81`, `:90`, `:201`: value `80_949`; `:86`: page `81_061`, at jig
  `e1b483760`), each set to what the new code computes (ruling 147):
  derived values of the Shape, not "the case the Shape changes"'s four
  assertions.
- Users: `Settings` literals (5 in jig: `jig-domain-core/src/tests.rs:263`,
  `jig-protocol-config/src/core.rs:54`, `testing/jig-test-domain/src/tests/fixtures.rs:117`,
  `tests/core/src/world.rs:301`, `tests/engine-clients/src/fixtures/receiving.rs:33`),
  each gaining `templates` (the core world's: `fixtures::templates()`,
  new in `tests/core/src/fixtures.rs`, the chat and goal of 9.1;
  elsewhere `Box::new([])`); `Config` literals (4 besides `core.rs`'s: `jig-domain-core/src/tests.rs:248`,
  `testing/jig-test-domain/src/tests/fixtures.rs:115`,
  `tests/core/src/world.rs:286`, `tests/engine-clients/src/fixtures/receiving.rs:31`),
  each gaining `template_overrides: Map::with_capacity(0)` or the
  world's; `Project` literals (2: `projects.rs:72`,
  `testing/jig-test-service/src/tests/first_period.rs:17`); the JSON
  project policies of 9.1's four fixtures (`"templates": []`); the
  matches over `PolicyChange` (`tests/core/src/observations.rs:447`, an
  arm; `tests/client/src/joint/wire/asks.rs:714`, `Templates { .. }`
  joining the panic arm at :718); the `documents::Policy` literal at
  `crates/jig-protocol-engine-clients/src/tests/fixtures/objects.rs:76`
  (`templates: List::with_capacity(0)`).
- Downstream: one `Settings` literal, one `Config` literal, one
  `Project` literal and one JSON policy, absorbed at its repin.
Shape:
- `people::TemplateOverride` and `PolicyChange::Templates { overrides }`
  as sketched; `PolicyValue.templates: Box<[TemplateOverride]>`.
- `jig_domain_core::Settings.templates: Box<[Template]>`;
  `Config.template_overrides: Map<u32, Box<[people::TemplateOverride]>>`,
  as `permission_roles`.
- `documents::Policy.templates: List<people::TemplateOverride>` (on the
  wire: jig 14's `PolicyEdit` for the change, jig 22's `Policy` document
  for the field).
- At restart, `restore_policy` keeps a stored override whose template
  the configuration no longer defines ("a policy's override naming a
  template the configuration does not define at that start serves
  nothing, and the next change of that policy must leave it out",
  configuration.md 3.1); a later `Templates` change naming it is refused
  as any unknown template is.
- `jig_protocol_config::Project.templates: Box<[people::TemplateOverride]>`.
Test shape:
- `crates/jig-domain-core/src/tests.rs` is over 800 lines: unit tests
  go in `crates/jig-domain-core/src/tests/templates.rs` (new):
  `a_policy_change_naming_an_unknown_template_is_refused`,
  `a_policy_change_past_the_templates_limit_is_refused`.
- `crates/jig-protocol-config/src/tests/templates.rs`:
  `a_seeded_override_naming_no_template_refuses_the_start`,
  `a_seeded_override_reaches_the_cores_config`.
- `tests/core/tests/templates.rs` (new; fixtures as 8.1's in jig 08:
  `stories::reads::{world, request}`, `reads::{ask, latest}`,
  `World::restart`): `a_template_override_survives_a_restart_in_the_projects_policy`
  (an owner's `ChangePolicy { change: Templates { [override of template
  2, priority 5] } }`; after `world.restart`, the project's document's
  policy holds that override),
  `a_stored_override_of_a_dropped_template_is_restored_at_start` (the
  same override; `world.restart_with` a configuration whose
  `Settings.templates` holds template 1 alone: the start goes on, and
  the project's document's policy still holds the override),
  `a_policy_change_naming_a_dropped_template_is_refused` (after that
  restart, a `Templates` change naming template 2 is
  `Refusal::Unknown`).
- Ceilings: unit under 20 ms, world under 0.5 s.
Proof: world. `cargo nextest run -p jig-core-world -E
'test(a_template_override_survives_a_restart_in_the_projects_policy)'`
once: an owner's override is committed in the real store's policy row,
restored, and read back in the project's document; under 0.5 s.
Don't: put the overrides in `authority::Policy` (no check reads them);
override a template's flags, label or tracked; check an override's
values against the role's authority.
Size: at most 230 lines of production code (a codec increment).
Removes: nothing.

### 09.3 `me` serves each project's templates, merged with its overrides

Scope: engine.md 11.1 and 13, contract.md 3.6 and 5.1: each project
entry of `me` carries the configured templates with that project's
overrides applied (`merged`, the sketch); a change of a project's
overrides marks `me` as any policy change does (jig 05.4:
`touch_project_parties`).
Needs: jig 05's `me`: `documents::Frame` and its `FrameProjectRole`
(names jig 05 keeps).
Files:
- `crates/jig-domain-core/src/`: `templates.rs` (`merged`),
  `documents.rs` and `documents/small.rs` (`FrameProjectRole.templates`;
  `me`'s worst case grows by `documents.lines` (the frame's projects,
  `documents/small.rs:91`) times a `List<Template>` of `templates`, with
  their names and contracts' heap, added in
  `document_with_context_worst_case(&Limits)` (`documents.rs:459`), since
  `document_worst_case(&DocumentLimits)` (:425) cannot see `templates`;
  `src/tests/frame_deployment_role.rs:68-74`'s `held` counts it too).
- The values derived from that worst case (rulings 110 and 112):
  `What::Memory { over }` at
  `crates/jig-protocol-config/src/tests/domain.rs:169`, and the
  profiles' bounds and their copies where they move.
- `FrameProjectRole`'s users, as jig 05.4 lists them for `creating`
  (11 in 8 files at the base), among them
  `crates/jig-protocol-engine-clients/src/translate/owner.rs`, whose
  translation drops `templates` until jig 22.
- `tests/core/tests/templates.rs`.
Shape:
- `FrameProjectRole.templates: List<Template>`, the
  merged templates in configuration order.
- `jig_domain_core::templates::merged(templates, overrides) -> Box<[Template]>`
  as sketched.
Test shape:
- `crates/jig-domain-core/src/tests/templates.rs`:
  `an_override_replaces_only_the_fields_it_names`,
  `an_override_naming_no_configured_template_serves_nothing`,
  `a_priority_override_leaves_an_untracked_template_untracked`.
- `tests/core/tests/templates.rs`:
  `me_serves_the_projects_templates_merged_with_its_overrides`
  (template 2 with priority 5 after 9.2's override, template 1
  unchanged), `a_template_override_marks_the_owners_me`.
- Ceilings: unit under 20 ms, world under 0.5 s.
Proof: world. `cargo nextest run -p jig-core-world -E
'test(me_serves_the_projects_templates_merged_with_its_overrides)'`
once: the owner's `me`, read from the core on the real store, holds
template 1 as configured and template 2 at the overridden priority;
under 0.5 s. The wire carries `me`'s templates from jig 22, whose
client story reads them at the joint layer.
Don't: expand a template into a request in the engine; key an offer, a
check or a list on a template; drop `OfferedCharter` (jig 22 and jig 16
decide it).
Size: at most 120 lines of production code.
Removes: nothing.

### 09.4 Operations per request checked at start

Scope: configuration.md 6: a configuration whose people `operations` is
below the larger of the tasks child's `delegates` plus one and
`project_goals` refuses the start, naming the three; the shipped
profiles hold it.
Needs: jig 04.6's `people::Limits::operations` and jig 04.2's
`tasks::Limits::project_goals`.
Files:
- `crates/jig-protocol-config/src/`: `domain.rs` (`whole`: the check,
  at `limits`), `problem.rs` (`What::Operations { operations, delegates,
  goals }`), `tests/domain.rs`.
- `testing/jig-test-service/src/tests/startup.rs`.
- `tests/channel/src/fixtures.rs:29-38`: its limits (`"tasks.tasks":1`,
  `"tasks.delegates":1`) cannot hold `operations >= delegates + 1` beside
  jig 04.6's `operations <= tasks`; it takes `"tasks.tasks":2`,
  `"people.operations":2`, `"tasks.project_goals":1` and
  `"people.inbox_tasks":4` (the core's `worst_case` needs `inbox_tasks`
  at least twice `tasks`, `memory.rs:99`), the smallest values that
  hold them, each test of the worlds on it
  (`jig-channel-world`, `jig-one-process-world`, `jig-test`) kept as it
  is. A different value is a question.
- Downstream: its service's two matches over every `config::What`
  (`startup/problems.rs:81-103` and `:128-170`) gain an arm for
  `What::Operations`, absorbed at its repin.
Shape:
- `What::Operations { operations: u32, delegates: u32, goals: u32 }`,
  pushed at `Path::root().member(b"limits")` when `people.operations <
  max(tasks.delegates + 1, tasks.project_goals)`. jig 04.6's
  `worst_case` already refuses `operations` above `tasks` (`What::Bound`
  at `limits`); this increment adds no term to `route_room_checked`.
Test shape:
- `crates/jig-protocol-config/src/tests/domain.rs` (its `refuses`, :11):
  `operations_below_delegates_plus_one_refuse_the_start`,
  `operations_below_goals_per_project_refuse_the_start`,
  `the_shipped_profiles_hold_operations_per_request` (small: 8 against
  max(5, 8); standard: 16 against max(9, 16)).
- `testing/jig-test-service/src/tests/startup.rs`:
  `a_service_whose_operations_cannot_close_a_conversation_is_refused`
  (overrides `"people.operations": 4` with `"tasks.delegates": 4`:
  `What::Operations { operations: 4, delegates: 4, goals: 8 }`).
- Ceilings: unit under 20 ms.
Proof: world. `cargo nextest run -p jig-test-service -E
'test(a_service_whose_operations_cannot_close_a_conversation_is_refused)'`
once: the reference service's start is refused, naming the three
numbers, before its loop runs; under 0.5 s.
Don't: lower a profile's value to fit; check operations against `tasks`
again (jig 04.6 does).
Size: at most 60 lines of production code.
Removes: nothing.

## Interface sketches

Compiled at the base under `cargo clippy -p <scratch crate>
--all-targets -- -D warnings`, in a scratch crate of jig with jig 02's
`Flags` stubbed: checked at the ready check. Bodies are
`unreachable!("sketch")` but for the algorithm in comments.

```rust
// crates/jig-domain-people/src/templates.rs (9.1; TemplateOverride 9.2)
/// Who executes a task made from a template (configuration.md 3.1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TemplateExecutor {
    /// An agent under this charter, by its number among the configured charters.
    Charter(u32),
    /// A charter or procedure the creator's role may create with, the creator's choice.
    Chosen,
}
/// A template's budget (configuration.md 3.1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TemplateBudget {
    /// This amount.
    Amount(u64),
    /// As much as the creator's role and pool allow, capped where `cap` says.
    AsAllowed { cap: Option<u64> },
    /// The creator's to give, within their allotment.
    Given,
}
/// A template's deadline (configuration.md 3.1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum TemplateDeadline {
    /// No deadline.
    None,
    /// This span from creation.
    Span(Duration),
    /// The creator's to give.
    Given,
}
/// A project's policy's override of one template (authority.md 6), each
/// field `None` where the template's own value stands.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TemplateOverride {
    pub template: u32,
    pub executor: Option<TemplateExecutor>,
    pub budget: Option<TemplateBudget>,
    pub deadline: Option<TemplateDeadline>,
    pub priority: Option<u32>,
}
// crates/jig-domain-people/src/policy_value.rs
#[derive(Clone, PartialEq, Eq, Hash, Debug)] // the base's
pub enum PolicyChange {
    // ... the base's variants, then:
    /// The project's overrides replaced as one list, each naming a configured template.
    Templates { overrides: Box<[TemplateOverride]> },
}
```

```rust
// crates/jig-domain-core/src/templates.rs (9.1; merged 9.3)
/// A template: the application's defaults for a shape of work
/// (configuration.md 3.1), served in `me`, read by no decision.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Template {
    /// Its number, never reused (0 is refused).
    pub number: u32,
    /// Its name, for showing, within `documents.text_bytes`.
    pub name: Box<[u8]>,
    pub executor: people::TemplateExecutor,
    /// The result contract; never `Approval`, which names one change.
    pub contract: tasks::Contract,
    /// The creator's role narrowed by this many levels of delegation.
    pub shallower: u32,
    pub budget: people::TemplateBudget,
    pub deadline: people::TemplateDeadline,
    /// Tracked, with its default priority; `None` when not tracked.
    pub tracked: Option<u32>,
    /// jig 02's flags: conversation, listed, the retention class.
    pub flags: tasks::Flags,
    /// Opaque to jig.
    pub label: u32,
    /// The grace its helper cancels with, within the tasks child's grace span.
    pub grace: Option<Duration>,
}
/// Each template with the project's override for its number applied.
pub fn merged(templates: &[Template], overrides: &[people::TemplateOverride]) -> Box<[Template]> {
    // for each template t, in configuration order:
    //   m = t.clone(); o = the override whose `template == t.number`, if any;
    //   if let Some(o) = o {
    //       if let Some(e) = o.executor { m.executor = e }
    //       if let Some(b) = o.budget { m.budget = b }
    //       if let Some(d) = o.deadline { m.deadline = d }
    //       if let (Some(p), Some(_)) = (o.priority, m.tracked) { m.tracked = Some(p) }
    //   }
    //   push m (an override naming no configured template serves nothing).
}
```

```rust
// crates/jig-protocol-config/src/templates.rs (9.1)
/// Parse and check the `templates` member (configuration.md 3.1).
pub(crate) fn templates(
    document: &Document,
    at: u32,
    path: &Path,
    charters: &[jig_domain_core::Charter],
    limits: &Limits,
    problems: &mut Problems,
) -> Option<Box<[Template]>>;
/// Transitional, until jig 16: the chat charter and grace the engine's
/// chat code reads, from template 1; `None` refuses the start.
pub(crate) fn chat(templates: &[Template]) -> Option<(u32, Duration)>;
// crates/jig-protocol-config/src/problem.rs: What gains
//   Grace { template: u32 } (9.1), Operations { operations: u32, delegates: u32, goals: u32 } (9.4)
```

Who owns what: the configuration parses and checks templates and seeded
overrides; the people child's policy value carries a project's overrides
and the store saves them; the core keeps both and merges them only into
`me`; the client fills a merged template into an ordinary request (jig
15's `shapes`).

## Questions to raise when reached

- **A priority override on an untracked template** (9.3): the design
  lets a policy override "the default priority" and never tracked; this
  plan serves the template untracked and the priority unused. The other
  reading refuses such an override when the policy is changed. Raise it
  when 9.2 lands.
- **The templates' JSON** (9.1): answered (the design answer to jig
  09.1's question of 2026-10-16T14:57:59Z): configuration.md 10 takes
  "Choices"' form, as 9.1 built it; nothing in 9.1 changes.

## Done when

- Each of `jig-09.1` to `jig-09.4` has landed: for k in 1 to 4, `git -C
  ~/src/rust/jig fetch origin && git -C ~/src/rust/jig log origin/main
  --all-match --grep='^Plan: client-api$' --grep="^Step: jig-09.$k$"
  --format=%H -1` prints a commit.
- `git -C ~/src/rust/jig grep -n -E '"chats"|b"chats"|\bChats\b|close_grace_ms'
  origin/main -- crates testing tests` prints nothing.
- In a worktree of jig's `origin/main`, once: `cargo nextest run -p
  jig-protocol-config -p jig-core-world -p jig-test-service -E
  'test(=tests::templates::a_template_grace_past_the_tasks_limit_refuses_the_start) |
  test(=tests::templates::the_chat_and_goal_templates_parse_with_their_defaults) |
  test(=tests::templates::a_seeded_override_naming_no_template_refuses_the_start) |
  test(=a_template_override_survives_a_restart_in_the_projects_policy) |
  test(=a_stored_override_of_a_dropped_template_is_restored_at_start) |
  test(=me_serves_the_projects_templates_merged_with_its_overrides) |
  test(=tests::startup::a_service_whose_operations_cannot_close_a_conversation_is_refused)'`
  passes these seven tests and runs no other.
