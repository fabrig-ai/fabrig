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
