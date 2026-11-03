+++
id = "policy-change"
part = "policy-item"
kind = "feature"
needs = ["restore-surface"]
files = ["jig/crates/jig-domain-core/src/settings.rs", "jig/crates/jig-domain-core/src/settings/**", "jig/crates/jig-domain-core/src/routing/settings/**", "jig/crates/jig-domain-core/src/admission/policy.rs", "jig/crates/jig-domain-core/src/documents/settings_review.rs", "jig/crates/jig-domain-core/src/documents/settings.rs", "jig/crates/jig-domain-core/src/reads/settings.rs", "jig/crates/jig-domain-core/src/tests/settings/project_policy.rs", "jig/crates/jig-domain-people/src/domain/applications.rs"]
effort = "high"
minutes = 150
+++

# policy-change: a project's policy changed by a change of settings, and proposed

## Goal

The core gives `Setting::ProjectPolicy { project }` its check, its commit, its proposal and
its review (jig's `domain/engine.md`, 17.2 and 17.6; contracts B1, B2 and configuring-p2's
P1): checked as a policy projects are made from and as an owner's edit of the policy is,
decided by the project's owners and, beside the deployment's items, by a party who owns both,
replacing the project's policy whole in the change's decision, recorded with the policy it
replaced.

Acceptance:

- A `ChangeSettings` holding one `ProjectPolicy` item set to a valid policy, from an owner of
  that project (and of the deployment, if any item is the deployment's), commits: the
  project's policy is the one set, as an owner's `ChangePolicy` would leave it (the open
  period's budget and the pools resized, the authority child told), and the revision's item
  records `Replaced::Value(Value::Policy(<the policy before>))`.
- Refused: `Removed` or `Reset`, a second `ProjectPolicy`, project 0 (all `Shape`, naming
  the item); a party who does not own the project (*not theirs*, as the people child refuses
  `ChangeSettings` today: `Refusal::Role`); a policy that fails 17.1's check of a policy
  projects are made from, or any check an owner's `ChangePolicy` makes (that refusal's
  fault, naming the item); a project not live (the people child's refusal of an unknown
  project). A policy equal to the project's is the same: alone, the change is *empty*.
- A `Propose` naming one: refused at its entrance where no party owns the deployment and the
  project; otherwise kept, offered as a settings proposal is, accepted only by a party who owns
  both (another refused as *not theirs*), withdrawn by the project's deletion (the deletion's
  decision withdraws every pending proposal naming that project's policy, as
  `jig/docs/design/domain/people.md`, 5.2 says). Its review shows the project's policy now and
  as it would be, whole.
- The unit tests below pass with `jig-domain-core`'s and `jig-domain-people`'s focused tests.

## Inputs

- `jig/docs/design/domain/engine.md`, 17.1 (the policy's check), 17.2 (a project's own
  policy), 17.6 (a proposal of a project's policy), 17.10 ("Each item records what it
  replaced").
- `jig/docs/design/domain/people.md`, 5.2.
- `jig/docs/design/client/domain/contract.md`, 5.1 (the review's item).
- `temper/docs/plan/configuring-p2-build/notes/contracts.md`, B1, B2;
  `temper/docs/plan/configuring-p2/notes/contracts.md`, P1; `notes/common.md`.

## Owns and may change

- Owned: `jig-domain-core`'s `src/settings.rs` and `src/settings/**` (the check, `projects`,
  `change_shape`'s arm, the commit, `proposal_heads`), `src/routing/settings/**`
  (`preflight`, `decide_with`, `propose_admitted`, `proposal_check`, the proposal's
  acceptance), `src/admission/policy.rs` (a whole replacement reusing `change_policy`'s
  checks, `resize` and `Event::Policy`), `src/documents/settings_review.rs` and
  `src/documents/settings.rs` (the review's item), `src/reads/settings.rs` (its arms);
  `jig-domain-people/src/domain/applications.rs` (the admission of a proposal's acceptance
  naming a project's policy); the unit test file
  `jig-domain-core/src/tests/settings/project_policy.rs`.
- Every `unreachable!("a project's policy: policy-change")` arm in `jig-domain-core` gets its
  body; the others stay (`notes/common.md`).
- Status: remove from `jig/docs/design/domain/engine.status.md`'s **Not yet** the text "17.2,
  17.6: a project's own policy as an item of a change or a proposal: jig's `Setting` names none
  (`jig-domain-people/src/settings.rs`), and no change replaces a project's policy."
- Options, pre-approved:
  - **Where the replacement runs:** reuse `admission/policy.rs`' `change_policy` tail (the
    checks from `apply` on, `snapshot`, `check_policy_ceilings`, `resize`,
    `step_authority(authority::Event::Policy { .. })`) with the whole policy in place of a
    `PolicyChange`'s result; if that tail cannot run inside the settings decision as it is,
    split it into a function both call and say so.
  - **The entrance check of a proposal** reads `core.people.project_members` for the
    deployment and the project; if a party's holdings are not reachable there, keep the
    proposal and refuse its acceptance by a party who does not own both, and say so in the
    report (the entrance check is then left, listed under `## Left`).
  - **An application's items of a project's scope** keep M3's rule (a proposal waits for the
    deployment's owners): this task changes the rule for a project's policy alone, and leaves
    engine.status.md's 17.6 entry about a project's items as it is.
  - If `ReviewItem`'s `now` and `would_be` cannot carry a project's policy as they carry a
    `Value`, add an arm or a field for it, documented, and say so (settings-wire carries it).
  - If a fixed byte price in a test fails only because a type grew, correct it as
    `policy-surface` did, listing each.

## Steps

1. `settings/application.rs`' `projects`: `ProjectPolicy { project } => project`, so the people
   child's `ChangeSettings` admission (an owner of every scope named) decides *not theirs* and
   *owns both*.
2. `settings/change.rs`' `change_shape`: `ProjectPolicy` accepted only as `Edit::Set(
   Value::Policy(_))`, project nonzero, at most one per change; `check.rs`' `shape` accepts
   `Value::Policy` for it.
3. The check (`check_change` / `preflight`): 17.1's `policy_fault` against the settings the
   change leaves, and the checks an owner's `ChangePolicy` makes of the result
   (`admission/policy.rs`); a policy equal to the project's counts toward *empty*
   (`changes_current`).
4. The commit (`decide_with`): the project's policy replaced whole (step's option above); the
   revision's `RevisionItem` for it with `replaced` the policy before. The item never enters
   `Current`'s defaults or changes.
5. Proposals: `propose_admitted` and `proposal_check` refuse a proposal naming a project's
   policy where no party owns both (`people::Refusal::Authority { lacking:
   people::Lack::Approver, proposable: Some(people::Accepter::Nobody) }`, as a proposal no
   owner can accept is refused today); the acceptance admitted only for a party who owns both;
   `proposal_heads` leaves the item out as it leaves out an application's; the project's
   deletion withdraws a pending proposal naming its policy, in the deletion's decision, as the
   core withdraws proposals elsewhere.
6. The review (`documents/settings_review.rs`): the item's `now` the project's policy, its
   `would_be` the policy set.
7. Tests, `jig-domain-core/src/tests/settings/project_policy.rs` (beside policy-surface's
   test, which you replace: its refusal no longer holds):
   - `an_owner_of_both_sets_charters_and_a_project_policy_in_one_change`
   - `a_project_policy_change_records_the_policy_it_replaced`
   - `a_deployment_only_owner_is_refused_a_project_policy_item`
   - `a_project_policy_removed_reset_or_twice_is_refused_its_shape`
   - `a_project_policy_past_its_bounds_is_refused_as_an_owners_edit_is`
   - `the_same_project_policy_alone_is_empty`
   - `a_proposal_of_a_project_policy_no_party_owns_both_of_is_refused_at_entrance`
   - `a_proposal_of_a_project_policy_is_accepted_only_by_an_owner_of_both`
   - `deleting_a_project_withdraws_a_proposal_naming_its_policy`
   - `a_review_shows_the_project_policy_now_and_as_it_would_be`
8. Narrow tests: `heavy cargo nextest run -p jig-domain-core --lib -E
   'test(/settings::project_policy/)'` (check the count), then `-E 'test(/settings/)'` and
   `-E 'test(/policy/)'`; `heavy cargo nextest run -p jig-domain-people --lib`. A change of
   meaning (the people child's admission, a proposal's acceptance): also the focused tests of
   `jig-domain-core`, `jig-domain-people` and the core world package `jig/tests/core`
   (`heavy cargo nextest run -p <its package> --test <its binary> -E 'test(/settings/)'`).
   Then `cargo fmt`; `heavy cargo clippy -p jig-domain-core -p jig-domain-people
   --all-targets -- -D warnings`. Commit and push.

## Report

How the replacement runs (the option taken); how a proposal's entrance and acceptance decide
*owns both*; what the review carries; the tests and their counts.
