+++
id = "person-task"
part = "person-task"
kind = "feature"
needs = []
files = [
  "jig/crates/jig-domain-core/src/effects/connector.rs",
  "jig/crates/jig-domain-core/src/funding.rs",
  "jig/crates/jig-domain-core/src/funding/pools.rs",
  "jig/crates/jig-domain-core/src/routing/task.rs",
  "jig/crates/jig-domain-core/src/routing/mod.rs",
  "jig/crates/jig-domain-core/src/routing/stopping.rs",
  "jig/crates/jig-domain-core/src/vocabulary.rs",
  "jig/crates/jig-domain-core/src/documents/task_tree.rs",
  "jig/crates/jig-domain-core/src/documents/chip_phase.rs",
  "jig/crates/jig-domain-core/src/tests/person_task.rs",
  "jig/crates/jig-domain-core/src/tests.rs",
]
effort = "high"
minutes = 120
+++

# A person's task in no project

## Goal

jig's new mechanism: an application makes one agent task for a person who may have no project: in project 0, funded by
the person's pool there, so it takes what a chat of theirs takes, and in no list. temper's trial is its first reader, but
jig names no temper.

Acceptance, each a story, each failing first, in `jig/crates/jig-domain-core/src/tests/person_task.rs`, and in a world
of `jig/tests/core` where a run must reach a host:

- **It runs:** with a deployment and no project, `person_seed(person)` and `Event::PersonTask` with `llm: None` make a
  task whose run is claimed on a worker, taking the first model with a usable account of the person's, by
  `jig/docs/design/domain/accounts.md`, 4.3; a charter's first model with none of its account waits naming that, as a
  chat's does.
- **Who pays:** its `payer` is `Payer::Party(person)`; a plan and a key of the person's each serve it; a key whose `uses`
  name a project alone doesn't.
- **Money:** the first opens period 0 at the rules' `period_spend` and the person's pool at the ceiling's spend; a second
  opens neither; the budget is `maximum_run_spend`; `person_seed` is `None` past the pool's room. A later `Period`
  for a real project moves nothing of it.
- **Authority:** its authority holds no tools, grants, notes or delegation.
- **In no list:** with the task live, a `Chats`-style query for the person (`requester = Person`), the inbox, results and
  updates hold nothing of it; `me` names no project 0.
- **Its document:** the person who pays may watch its Task document: `run_wait`, `kept_account`, `chip.work` are as a
  chat's. Another person may not.
- **A hold reaches nobody:** an escalation past its tries reaches no inbox, and the task stays held until cancelled.
- **A restart** restores it live, its links checked; `Cancel` by `Party::Deployment { project: 0 }` ends it.
- `run_past_starting`: false with no run or while `Preparing` or `Starting`, true from `Active` on, false again after the
  run ends.

## Inputs

- `temper/docs/plan/model-check/notes/common.md`, `temper/docs/plan/model-check/notes/contracts.md`, *jig*. Its surface
  binds. This task's own words are jig's: no "trial".
- `jig/docs/design/domain/authority.md`, 6.1, 7.
- `jig/docs/design/domain/accounts.md`, 4.2, 4.3.
- `jig/docs/design/domain/core.md`, 6.1; `people.md`, 3.
- Precedent: `RepairSeed`, `repair_seed`, `route_connector_repair` (`effects/connector.rs`, `routing/task.rs`).

## Owns and may change

Owns, as `files` lists them.

May also change:

- `jig/tests/core/**`, a story file and its `mod` line.
- `jig/crates/jig-domain-core/src/lib.rs`: a re-export.
- `jig/crates/jig-domain-core/src/start/links.rs`, `admission/escalation.rs`, `inbox.rs`, `documents/inbox_deployment.rs`:
  only where a story shows project 0 reaching a wrong place; a line each, and say which.
- `temper/**`: arms the compiler forces for the new `Event` variant, and only those (`AGENTS.md`'s required field rule).
- `jig/docs/design/domain/authority.status.md`: from **Not yet**, remove "6.1: whole.", and only that.

Leave the wire: no `jig-client-protocol-wire`, no contract, no store schema. `Funder::Pool` and project 0 are values the
store already holds.

Options, each pre-approved:

- **A check in the tasks child refuses project 0 at `Make`:** let it pass for this funder, in that one check, and say which.
- **A reader panics on project 0** (`.expect` on a policy, an `open_project` assert) on the path of a run: return as for
  a closed project where it's a read; where it's the run's own path, take the rules' in place of the policy, a line. Name
  each in your report.
- **The worker's capacity counts by project:** let project 0's attempt count as any attempt. It needs no row of its own.

## Steps

```rust
// funding.rs: a period and pool of project 0 take the rules' figures.
pub(crate) fn period_budget(&self, project: u32) -> u64 { /* DEPLOYMENT, no row: rules().period_spend */ }
pub(crate) fn pool_budget(&self, project: u32, person: u64) -> u64 { /* DEPLOYMENT: rules().ceiling.budget.spend */ }
```

`person_seed` is `repair_seed`'s sibling: `self.start_input()`, the person known by `people.identity`, the authority the
rules' ceiling with `tools` 0, `grants` none, `notes` none, delegation none and its spend `maximum_run_spend` bounded by
the pool's room (none: `None`), a fresh `Family::Task` number, `open_person_pool(DEPLOYMENT, person)`'s two opens held in
the seed. Authority passes through `translate::task_authority`.

`Event::PersonTask` routes beside `ConnectorRepair`, and joins `routing/stopping.rs`'s refusal as `ConnectorRepair` does:
the opens, then `tasks::Event::Make`, creator `Party::Deployment { project: 0 }`, `deadline: Named`, `label: 0`,
`Flags::PLAIN`, `llm: None`, `recurring: None`, `tracked: None`, `wake: DEFAULT`, the funder the pool's.

`task_tree.rs`'s two checks of `open_project(row.project) && role` admit a party where
`row.project == DEPLOYMENT && Funder::Pool { person == party }`.

`run_past_starting` reads `core.fleet.observation` as `chip_phase.rs` does, with no copy of its `current` logic: draw a
helper there.

**This changes what no project means in code a lot of readers walk:** run `jig-domain-core`'s whole `--lib`, `jig/tests/core`,
`jig/tests/tasks`, `jig/tests/people` and `jig/tests/one-process`, and `heavy cargo check -p temper-engine-domain --tests`.

The checks: fmt, `old_forms.py` for `jig`, clippy on `jig-domain-core`, the tests above. Citations. Push.

## Report

As the implementer's text asks: each story's bite; every reader you had to touch for project 0, with its reason; the
options taken.
