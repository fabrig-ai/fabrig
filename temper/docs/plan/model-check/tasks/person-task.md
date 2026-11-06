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
  "jig/crates/jig-domain-authority/src/check.rs",
  "jig/crates/jig-domain-authority/src/lib.rs",
  "jig/crates/jig-domain-authority/src/tests_policy.rs",
  "jig/testing/jig-conformance/src/referee.rs",
  "jig/testing/jig-conformance/src/referee/tests.rs",
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

## Answers

2026-11-06. **`check_run` refuses project 0 with `Finding::UnknownProject`, outside this task's files: widen the task, or
a prerequisite task?** Widen: "It runs" cannot hold without it, and `jig/docs/design/domain/authority.md`, 6.1 already
decides it ("Project 0 names no project and has no policy"; "its authority: the rules' ceiling"). `files` now hold
`jig/crates/jig-domain-authority/src/check.rs`, its `lib.rs` for the re-export and `tests_policy.rs`. Resume from the
pushed `4ad4220b01`. No synthetic policy, no substitute project, no bypass in the core.

- **The name:** `jig_domain_authority::NO_PROJECT: u32 = 0`, citing 6.1; the authority child names no other child, so it
  doesn't take `jig_domain_people::DEPLOYMENT`. Beside `person_seed`, the core holds them equal:
  `const _: () = assert!(authority::NO_PROJECT == people::DEPLOYMENT);`.
- **`check_run` only:** with no policy, a project other than `NO_PROJECT` still refuses `UnknownProject`. At `NO_PROJECT`
  the rules' ceiling stands in the policy's place for the run cap and the deadline, and a write refuses with
  `Finding::Grant { source: Source::Project }`, since nothing grants it (6.1: no grants).
- **Every other check keeps its refusal:** `check_call`, `check_effect`, `check_batch`, `check_projection`, `check_job`
  and `check_request` on project 0 refuse, as is right for a task with no tools, grants or delegation. If a story shows
  one on the run's own path, that is a new question.
- **Its story, failing first, in `tests_policy.rs`:** project 0 with no policy allows a good run; past the rules' spend
  or deadline it refuses; a write refuses; project 99 with no policy still refuses `UnknownProject`. Run
  `jig-domain-authority`'s `--lib` with the checks above.

2026-11-06. **The referee's authority predicate (`referee.rs:622`) requires a project policy for project 0: widen the task,
or a prerequisite task?** Widen: the host story in `jig/tests/core` is this task's acceptance and cannot pass without it,
and `jig/docs/design/domain/authority.md`, 6.1 already decides what the rule is. `files` now hold `referee.rs` and
`referee/tests.rs`. Resume from the pushed `0c38bd8a04`; the failing world story stays as it is. No synthetic policy, no
substitute project, no observation disabled.

- **The rule is the referee's own:** it states 6.1 itself and calls no authority decision, as the referee does elsewhere.
  A constant of its own, `NO_PROJECT: u32 = 0`, citing 6.1; it doesn't take `jig_domain_authority::NO_PROJECT`.
- **At `task.project == NO_PROJECT`,** in the place of `project.is_some_and(..)`: `task.source` is `Source::Deployment`,
  and the scope holds no tools, grants, notes, note resources, descendants or depth. The two checks beside it stay as they
  are: within `policy.deployment`, within its source. Every other project with no policy stays a violation.
- **Its story, failing first, in `referee/tests.rs`:** a stripped task from the deployment at project 0 passes; one with a
  tool, a grant, a note or delegation each breaks `Promise::Authority`; past the deployment's budget breaks it; a person as
  its source breaks it; project 99 with no policy still breaks it. Each refusal names what it expects; run
  `jig-conformance`'s `--lib` with the checks above.
- **Another referee read of a project's policy on this task's path** (`policy.projects[..]` at the effect decision, say):
  pre-approved, the same rule, a line each, named in your report. A decision on a project 0 task has no grant to stand on:
  where the story reaches that index, it's a `Promise::Authority` violation, not a pass and not a panic. Change it only if
  a story reaches it.
- **Any other violation** (spend, a pool, delivery, a receipt) is the referee finding what it's there to find: fix the
  code, not the referee. If the fix is outside this task, that is a new question.
