+++
id = "pd-cancel"
part = "project-deletion"
kind = "feature"
needs = ["pd-store"]
files = [
  "jig/crates/jig-domain-core/src/routing/deployment_acts.rs",
  "jig/crates/jig-domain-people/src/domain/acts.rs",
  "jig/tests/core/tests/suite/deployment_acts.rs",
  "jig/tests/people/tests/suite/acts.rs",
]
effort = "high"
minutes = 35
+++

## Goal

`Act::Cancel`'s body (`notes/contracts.md`, section 4): the
deployment's compound cancel of one live task of the named project,
whoever requested it, its live delegates cancelled with it, as `Close`
cancels a conversation today. Answered as a compound is; a task already
ended answers *ended*, a task the store no longer holds *unknown*, as
`Close`'s absent subject does. The forge connector's removal sends it
for each live change of a removed forge (`rm-remove-forge`), and the
project's deletion uses its function for each live task (`pd-close`).

Acceptance: the tests below pass; `Act::Cancel` granted by `"cancel"`
is no longer refused *not offered*; `Close` behaves as before (its
tests unchanged and passing).

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md`,
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (section 4, D4).
- `jig/docs/design/domain/people.md`, 5.3 (`Close`'s row: what `Cancel`
  generalises); `jig/docs/design/domain/tasks.md`, section 7 (the
  compound request, a deployment's cancel).
- `jig/crates/jig-domain-core/src/routing/deployment_acts.rs` (`close`),
  `routing/result_reads.rs` (`closed_subject`, `ReadFor::DeploymentClose`).

## Owns and may change

Owned: the `files` above. Neighbour: `routing/result_reads.rs`, only to
let `closed_subject`'s answer serve `Cancel` too (a `ReadFor` variant
or a parameter; default: reuse `DeploymentClose` unchanged if its
answer is already *ended*/*unknown* for any task).

## Steps

1. In `deployment_acts.rs`, split `close` into the checks and a shared

   ```rust
   /// The deployment's compound cancel of one live task and its live
   /// delegates (tasks.md, section 7); `Close` and `Cancel` both end here.
   pub(super) fn cancel_live(
       core: &mut Core, work: &mut Queue<Event>, request: Token,
       project: u32, task: u64, grace: Option<skein_lib::Duration>, reason: Box<[u8]>,
   ) -> Result<(), people::Refusal>
   ```

   holding today's body from the capacity check on (delegates with no
   grace, then the task with the grace, the `PersonTaskRoute::Compound`
   continuation, `tasks::Party::Deployment { project }`). `close` keeps
   its conversation and person-requester checks, then calls it.
   `route`'s `Act::Cancel` arm: a task the core does not hold goes to
   `closed_subject` as `Close`'s does; a task of another project is
   `Unknown`; an ended or closing one is `refusals::ended`; else
   `cancel_live`. `start` gives it no digest. `pd-close` calls
   `cancel_live` for each live root task of a deleted project.
2. In `jig-domain-people`'s `domain/acts.rs`: drop jig-acts' *not
   offered* for `Act::Cancel`; its `valid` checks as `Close`'s (task
   non-zero, reason within `words`); `valid_outcome` takes the
   compound's outcomes as `Close`'s.
3. Tests, `jig-core-world` (`tests/suite/deployment_acts.rs`; add
   `ActKind::Cancel` to `configuration`'s grant):
   - `a_connector_cancels_any_live_task_of_a_project_with_its_delegates`
     (a live non-conversation task with one delegate: both cancelled,
     by the deployment, with the reason; answered `Compound`);
   - `a_cancel_of_a_task_requested_by_the_deployment_is_made` (the
     requester rule `Close` has is not `Cancel`'s);
   - `a_cancel_of_an_ended_task_answers_ended_and_an_unknown_one_unknown`;
   - `a_cancel_naming_another_project_is_unknown`.
   `jig-people-world` (`tests/suite/acts.rs`): replace jig-acts' *not
   offered* case for `Cancel` by `a_cancel_is_routed_and_its_answer_saved`.
4. Checks: `cargo fmt`; clippy on `jig-domain-core`,
   `jig-domain-people`; `heavy cargo nextest run -p jig-core-world
   --test suite -E 'test(/deployment_acts/)'`, `-p jig-people-world`,
   `-p jig-domain-people --lib`, `-p jig-domain-core --lib`; `heavy
   cargo check --tests -p temper-engine-domain`. Commit, push.

## Report

fab's sections; `## Interfaces`: `cancel_live`'s path and signature,
the `Act::Cancel` answers by case.
