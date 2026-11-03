+++
id = "policy-story"
part = "policy-item"
kind = "feature"
needs = ["policy-change", "settings-store", "settings-wire", "policy-ctl"]
files = ["jig/tests/core/tests/suite/settings_project_policy.rs", "jig/tests/core/tests/suite/main.rs", "jig/tests/core/tests/settings/support.rs", "temper/tests/client/tests/suite/settings_tools.rs", "temper/tests/client/tests/suite/main.rs", "temper/tests/client/src/fixtures.rs"]
effort = "high"
minutes = 90
+++

# policy-story: a project's own policy changed and proposed, told end to end

## Goal

The worlds tell the policy-item part's story (jig's `domain/engine.md`, 17.2 and 17.6; the
slice's story: "An owner of both the deployment and a project changes charters and the
project's policy in one change of settings, through temper ctl's settings file"): through
jig's core on its real store and commit barrier, and through temper ctl's tools' root joined
to temper's engine root.

Acceptance:

- jig's core world, a new `jig/tests/core/tests/suite/settings_project_policy.rs`, each story
  one thread, one loop, its seed replaying, through `settings_support`'s helpers:
  - `an_owner_of_both_sets_charters_and_a_project_policy_in_one_change`: one `ChangeSettings`
    sets charter 1 and project 1's policy (the fixture's policy 1 with its period spend
    changed); one revision; the project's policy is the one set (the project document's
    spend per period and its open period's budget), the charter's value the new one, the
    revision's record holding both items, the policy item's `replaced` the policy before
    (`settings::Replaced::Value(Value::Policy(..))`, read from the stored record).
  - `a_deployment_only_owner_is_refused_a_project_policy_item`: a party owning the deployment
    (roster 0) and not project 1: the same change refused `people::Refusal::Role { .. }`
    (*not theirs*, policy-change's refusal), no row of the settings or the project changed.
  - `a_proposal_naming_a_project_policy_waits_for_an_owner_of_both`: a project member
    proposes charter 1 and project 1's policy; the review shows the policy now and as it
    would be, whole; a project-only owner and a deployment-only owner each accepting are
    refused, the proposal pending; the owner of both accepts it, one revision, the policy set.
  - `a_proposal_no_party_owns_both_of_is_refused_at_its_entrance`: with no party owning both
    (the deployment's roster emptied of project 1's owners, as
    `a_proposal_without_an_owner_is_refused_at_admission` empties it), the proposal is refused
    `people::Refusal::Authority { proposable: Some(people::Accepter::Nobody), .. }` and nothing
    is kept.
  - `deleting_a_project_withdraws_a_proposal_naming_its_policy`: a pending proposal naming
    project 2's policy; the project removed (`people::Act::RemoveProject`, as
    `suite/deletion.rs` removes one); the proposal withdrawn in the deletion's decision, its
    proposer told, its kept set erased; a proposal naming project 1 alone stays pending.
  - `a_restart_keeps_a_pending_project_policy_proposal`: pending, then `restart`; the review
    after the restart is the same, and its acceptance commits the policy.
- temper's client world joined to its engine root, a new
  `temper/tests/client/tests/suite/settings_tools.rs`:
  `a_settings_file_sets_a_project_policy_through_ctl_proposed_shown_and_accepted`: the agent
  (the world's party 1, the deployment's and project 1's owner) reads `settings show policy
  1 --json`, writes a file with charter 1 and `{"kind": "project-policy", "project": 1}` set
  to that policy with its period spend changed, `settings propose FILE TEXT --against N --key
  K` (`proposed: p1/K'`), `proposal show p1/K'` (the `project-policy 1 · set · now ... ·
  would be ...` line, each policy whole), `proposal decide p1/K' accept --running stays --key
  K2` (`done: decide`), then `project show 1` prints the new spend per period and `settings
  show` the next revision; and the same file with a second `project-policy` item exits 1
  before anything is sent.
- The narrow runs below pass with their counts; the core world's and the client world's
  focused suites pass.

## Inputs

- `jig/docs/design/domain/engine.md`, 17.2 (a project's own policy), 17.6 (its proposal, who
  decides, a deletion); `jig/docs/design/domain/people.md`, 5.2; `jig/docs/design/domain/
  testing.md` (the core world's rules).
- temper's `temper/docs/design/client/tools.md`, 5.3 (the settings file's item, `proposal
  show`) and 11.2 (`tests/client`'s stories); `temper/docs/design/domain/testing.md`, 6.3
  and 6.4 (the client and the engine joined).
- `temper/docs/plan/configuring-p2-build/notes/contracts.md`, B1 and B5; `notes/common.md`.
- `temper/docs/plan/configuring-p2-build/tasks/policy-change.md` (the refusals and the
  deletion's withdrawal it builds) and `policy-ctl.md` (ctl's words).

## Owns and may change

- Owned: the two new story files and their `mod` lines (`jig/tests/core/tests/suite/main.rs`,
  `temper/tests/client/tests/suite/main.rs`).
- Neighbour files, helpers alone, no existing helper's behaviour changed:
  `jig/tests/core/tests/settings/support.rs` (a `project_policy_item(world, project,
  period_spend)` building the item from the fixture's policy 1, a `project_document` read, as
  `inventory` reads the settings); `temper/tests/client/src/fixtures.rs` (a
  `give_with_file(world, agent, words, bytes)` that sets the call's FILE slot to
  `Value::Text(bytes)`, as `call` does for `task amend`, so no story writes a file).
- Options, pre-approved:
  - **The core world's owners:** if `settings_support::start`'s owner (`OWNER`) is not project
    1's owner, add it to project 1's roster with `party`'s `people::Event::Roles` path in the
    story's own setup and say so; the deployment-only owner is `party(world, b"deployment
    owner", Some((0, people::Role::Owner)))`, and if the deployment's roster row is not
    `people::Key::Roles(0)`, send `people::Event::Roles { project: 0, .. }` as
    `settings_proposal_edges.rs` does.
  - **Project 2** for the deletion: if the fixture seeds one project, make the second with
    `people::Ask::MakeProject` from policy 1 first.
  - **The client world's party:** if party 1 is not the owner of both there, give the story
    its own `Joint` roster in the story file (as `project_fund.rs`' fixtures set funding) and
    say so; if the world cannot make it the owner of both without changing a shared
    constructor, keep the ctl story in `temper-client-tools/src/tests/jig/settings.rs`'s
    harness instead (the engine's answers scripted from the core world's), and say so.
  - If a story finds the code departs from the design, keep the story as the design says,
    mark it `#[ignore = "..."]` naming the departure, and report it (never change the design
    to fit). If a story needs larger limits, raise them in its own setup, never the world's
    defaults.
- Not this task's: the ctl's unit tests (`policy-ctl`); restoring (`restore-story`).

## Steps

1. The helpers named above.
2. The core world's stories, one at a time: `heavy cargo nextest run -p jig-core-world --test
   suite -E 'test(/settings_project_policy::/)'` (check the count: six).
3. The client world's story: `heavy cargo nextest run -p temper-client-world --test suite -E
   'test(/settings_tools::/)'` (check the count: one).
4. The worlds' focused suites: `heavy cargo nextest run -p jig-core-world --test suite -E
   'test(/settings/)'`, `heavy cargo nextest run -p temper-client-world --test suite`.
5. `cargo fmt`; `heavy cargo clippy -p jig-core-world -p temper-client-world --all-targets --
   -D warnings`. Commit ("Tell a project's own policy changed and proposed, end to end") and
   push.

## Report

The implementer's text's report, plus: the options taken (owners, project 2, the client
world's party); any story ignored, and why; each story's time from the run's log.
