+++
id = "fr-first-intent"
part = "forge-records"
kind = "feature"
needs = ["fr-setup", "fr-connect"]
files = [
  "temper/crates/temper-engine-domain-forge/src/first.rs",
  "temper/crates/temper-engine-domain-forge/src/asks.rs",
  "temper/crates/temper-engine-domain-forge/src/connect.rs",
  "temper/crates/temper-engine-domain-forge/src/lib.rs",
  "temper/crates/temper-engine-domain-forge/src/tests/first_intent.rs",
  "temper/crates/temper-engine-domain-forge/src/tests.rs",
  "temper/crates/temper-service/src/setup/lifecycle.rs",
  "temper/crates/temper-service/src/forge/table.rs",
  "temper/crates/temper-service/src/tests/first_intent.rs",
  "temper/tests/engine/domain/tests/qa_all/qa_first_repository.rs",
  "temper/crates/temper-engine-domain/src/route/applications.rs",
  "temper/crates/temper-engine-domain/src/route.rs",
  "jig/crates/jig-domain-core/src/projects.rs",
  "jig/crates/jig-domain-core/src/tests/project_names.rs",
]
effort = "high"
minutes = 55
+++

## Goal

The first repository's intent is kept in its forge's record (draft 3,
"The intent is kept"; contracts 2, `Forge.first`), so a restart before
the first repository is made makes it at the next connect instead of
losing it (`temper/docs/design/protocol/setup.md`, 6.7). And the
welcome's Default project, made already by a set up started over, is
reused by the next set up's first repository (contracts, D5).

Acceptance:

- An admitted `SetUp`, and an admitted `Connect` with
  `How::ForgejoAdmin { first: Some(..) }`, save the forge's row with
  `first` set (`FirstIntent`: org, repository, project, label) in their
  decision, beside the domain's volatile `first` entry as today.
- `first_made` (and the service's success of the first repository)
  saves the row with `first` none; `first_failed` keeps it (the state
  is volatile, the intent stays).
- At a start, a restored record with `first` gives the domain its
  `first` entry, state `Making`, and the service's lifecycle its
  `Intent` when it holds the forge (`forge::table::kept`); the next
  connect of the forge makes the repository as the in-memory intent
  makes it today (`setup/lifecycle.rs`, `create_local`, and the
  managed path, `admin_create`).
- `SetUpProject::New(name)` whose name a project already has, made by
  this deployment for a set up before, is taken as
  `SetUpProject::Existing(that project)` (D5); a name taken by any
  other project is refused as today (`SetUpRefusal::NewProject`). The
  connector learns "made for a set up" from its own records: the
  project the removed forge's `first` named, kept while that forge is
  removing, or (default, simpler) any project whose name equals the
  intent's and that holds no repository; say which.
- `qa_first_repository.rs`' `first_repository_row_is_volatile_across_restart`
  is renamed `first_repository_intent_survives_a_restart` and asserts
  the row is back after the forked store's start (a qa-tier test: built,
  not run).

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md` and
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (section 2,
  `first`; D5).
- `temper/docs/design/protocol/setup.md`, 6.7.
- `temper/docs/plan/playground-reset-r4/notes/surveys/s1-forges-document.md`,
  section 4 (both copies of the intent and their readers).
- The merged `fr-setup`, `fr-connect` and `fr-forge-added` reports.

## Owns and may change

Owned: the files in the header (`first.rs` and both `tests/first_intent.rs`
new).

Neighbours: `temper-engine-domain-forge/src/domain.rs`'s `first_made`
and `first_failed` and the `restore` `Forge` arm: the call into
`first.rs` only.

Pre-approved options:

- If the service's intent can be rebuilt from the domain's `Row.first`
  instead of the record (it reads the domain's rows already), do that
  and leave `forge::table::kept` alone; say so.

## Steps

1. `first.rs`: `pub(crate) fn keep(d, forge, intent, out)`, `made(d,
   forge, out)`, `restored(d, record)`; the D5 project rule as
   `pub(crate) fn project(d_or_core_view, project: SetUpProject) ->
   SetUpProject` (the root may need to answer it: if the connector
   cannot see projects' names, do the rule in the root's admission of
   `SetUp`, `temper-engine-domain/src/route/applications.rs`, and say so).
2. `asks.rs`/`connect.rs`: the saves at admission.
3. Service: the lifecycle's intent from the record at `kept`.
4. **Tests**: connector `tests/first_intent.rs`:
   `set_up_keeps_its_intent_in_the_record`,
   `a_made_first_repository_clears_the_intent`,
   `a_failed_first_repository_keeps_the_intent`,
   `a_restored_intent_is_making_again`,
   `a_default_project_made_before_is_reused`;
   service `tests/first_intent.rs`:
   `an_intent_restored_with_its_forge_is_made_at_the_next_connect`
   (fork the store after `SetUp` admits, restart, connect: `Create`
   plan with the intent's org and repository).
5. `cargo fmt`; clippy on the changed packages; their focused tests;
   `heavy cargo check --tests -p temper-engine-domain-world -p
   temper-client-world`. Commit and push.

## Report

fab's sections, and `## Interfaces`: `first::*`, where the D5 rule
lives, and the qa test's change.

## Answers

2026-11-01. Question: may the task add a public project-name reader to
`jig_domain_core::Core` and do D5 in the root's admission of `SetUp`?

Yes, with this shape: `pub fn project_named(&self, name: &str) ->
Option<u32>` in `jig/crates/jig-domain-core/src/projects.rs`, the project
whose name is the same as `name` under `people::same_name` (the rule
`rename` and `make_project` already apply over `Core.names`), so the
root never compares names itself; doc comment citing jig's
`domain/people.md`, 5.1. It reads state only: no stored format, act or
document changes. Its test goes in
`jig/crates/jig-domain-core/src/tests/project_names.rs`:
`a_project_is_found_by_its_name_folded`, one name found under another
case, an unknown name none. The D5 rule lives in the root's admission of
`SetUp` (`temper/crates/temper-engine-domain/src/route/applications.rs`),
the default, simpler rule: `SetUpProject::New(name)` with
`project_named(name) == Some(p)` and `p` holding no repository in the
forge connector's rows becomes `SetUpProject::Existing(p)`; holding one,
refused `SetUpRefusal::NewProject` as today; none, unchanged. The
connector's `first::project` step goes; the connector test
`a_default_project_made_before_is_reused` moves to the root's tests of
that admission if the connector cannot reach it, said in the report.
The header's `files` now name these three files.

2026-11-01. Question: may the task add `Domain::project_has_repository`
and `Domain::refuse_set_up` on the forge connector for D5?

Yes, both, in `first.rs` (an `impl Domain` block there), with this shape:

- `pub fn project_has_repository(&self, project: u32) -> bool`: any row
  of the connector's `repositories` with that project, ended and removing
  forges included (not `readable_repositories`, which drops ended
  forges); doc comment citing contracts, D5.
- `pub fn refuse_set_up(&mut self, env: &Env<Limits>, request: Token,
  token: Token, retention: Duration, why: SetUpRefusal, out: &mut
  Queue<Request>)`: the `SetUp` arm's own path, not `connect.answers`:
  the expired answer dropped, the room check (`ApplicationAgain` Busy
  when `setup_pending` and `setup_answers` fill `applications`), then
  `setup_pending` and `setup_answer`, so the refusal is retained and
  read by `application_answer` as every `SetUp` answer is. Factor that
  prologue out of the `SetUp` arm of `domain.rs` into one private
  function both call (the arm's change: the call only).

The root applies D5 where it can route the connector's outputs: in
`route.rs`'s `forge_event`, before `forge::step`, one call into an
`applications.rs` function given the `Event::Application` whose ask is
`SetUp` with `SetUpProject::New(name)`: `core.project_named(name)` none,
the event unchanged; `Some(p)` and `!project_has_repository(p)`, the
event with `SetUpProject::Existing(p)`; `Some(p)` holding one,
`refuse_set_up(.., NewProject, &mut out)` and no step, `out` routed by
`forge_route::outputs` as the step's are. `route.rs` joins the header's
`files` for that call only. The test `a_default_project_made_before_is_reused`
and a `a_named_project_with_a_repository_is_refused` go in the root's
tests of that admission. No stored format, protocol or act changes.
