+++
id = "jig-delegate-story"
part = "jig-review"
kind = "feature"
needs = ["jig-changed-story"]
files = ["jig/tests/client/src/stories/shared/**", "jig/tests/client/src/joint/role_fixture.rs", "jig/tests/client/src/joint/role_stories.rs", "jig/tests/client/src/scripted/stories.rs", "jig/tests/client/src/harness/seed.rs", "jig/docs/design/client/domain/testing.status.md"]
effort = "high"
minutes = 90
+++

# jig: the delegate record's story goes red on `requester: by`

## Goal

The review's Part B, finding 2. In `a_delegate_s_escalation_record_names_the_person_who_asked`,
`REQUESTER`, `PARENT` and the decider are all 1, so `requester: by` in
`jig/testing/jig-client-scripted-engine/src/requests/decisions.rs` stays green, and so does recording the
parent task's number. The rule is held only by that crate's unit test
`escalation_records_walk_to_the_person_who_asked_instead_of_the_decider`.

The request wants a story that goes red when `requester: by` comes back, which the review's own fix
does not give. That needs a decider who is **not** the person who asked, and the cut read how:

- In the core an escalation waits on `Person(requester)` only if that person's role covers it;
  otherwise it waits on the policy's `escalation_role` (`jig-domain-core/src/admission/escalation.rs`,
  `recipient_after`). `jig_test_profile::config`'s policy names role 0, and `people::Role::Owner.number()` is 0.
- `jig_test_profile::config` holds one role, so `Facts::DelegateEscalation`'s world cannot part them.
  `Seed::Roles` can: `joint/role_fixture.rs` holds owner 1, maintainer 2, member 3 and observer 4 as real
  holdings, and `scripted/stories.rs`'s `roles` mirrors them.

So **a member's chat delegates, the delegate is held, and the owner decides:** the record's person who
asked is 3, its decider 1, and the parent is task 1. Both hand-breaks read `Requester task: 1` where the
story wants `3`.

Acceptance, on the story path:

- The story runs on both fronts, and reads `Requester task: 3` and `Decision person: 1`.
- With `requester: by` in `decisions.rs`, the scripted story goes red. Recording the parent task's number
  there does too.
- `Facts::DelegateEscalation` is gone if the story no longer uses it.

Acceptance, on the deletion path, which the request pre-approves and Step 4 bounds:

- The story, `Facts::DelegateEscalation`, its arms and its fixtures are gone; the unit test stays.
- `testing.status.md`, 4 records the gap, as Step 4 words it.

## Inputs

- `temper/docs/plan/debt-6/notes/common.md`, `AGENTS.md`, `jig/AGENTS.md`.
- `jig/docs/design/client/domain/testing.md`, 4 to 6; `jig/docs/design/client/domain/testing.status.md`.

## Owns and may change

- Owns: the story and its fixtures: `b_held.rs`, `b_scenarios.rs`'s `DelegateEscalation` arms, `b_config.rs`'s,
  `b_seed.rs`'s, `b_runner.rs`'s line, `a_roles.rs`, `a_tests.rs`.
- May add to `joint/role_fixture.rs`, `joint/role_stories.rs`, `harness/seed.rs`'s `RoleStory` and
  `scripted/stories.rs`'s `roles` one story's arm each.
- `testing.status.md`: this departs from the implementer's text, on the deletion path only.
- Not this task's: `jig-client-scripted-engine` and any crate outside `jig/tests/client`, which are
  only hand-broken. A story that needs one changed takes the deletion path, and says what it needed.

## Steps

1. Add `RoleStory::DelegateHold`, built from what `Facts::DelegateEscalation` configures today:
   the worker's `Call::Delegate` then `Script::Park`, the chat's `ended_by` requester, and the authority,
   tree and budget limits `b_config::setup` sets for it. The role fixture's `configure` and its
   `Seed::Conversation { party: 3, .. }` are the seams. On the joint, the chat is created as party 3's
   sign-in.
2. The scripted side: `chat.requester` is `Person(3)`, the delegate's `Task(1)`, and the hold's holder is
   the owner. `requests.rs`'s `recipient` matches a `Role` only against `HeldRole::Policy`, and `roles`
   gives party 1 `HeldRole::Owner`: use `EscalationHolder::Person(1)` here, and say so. The rule under
   test is the record's requester, not the holder's form.
3. Move the story to `a_roles.rs` as `a_delegate_s_escalation_record_names_the_person_who_asked`, run from
   `a_tests.rs` (scripted) and `joint/role_stories.rs` (joint). The owner's client opens the inbox, leaves
   it held and cancels the delegate as today; it then sees `Requester task: 3` and `Decision person: 1`.
4. **Stop and delete if it costs too much:** that is, if the story and its fixtures would end more than
   40 lines longer than today's, or the joint front cannot hold the delegate on a member's chat with
   fixture changes alone, or a change outside `jig/tests/client` is needed. Then delete the story,
   `Facts::DelegateEscalation` and every arm and helper only it uses, and add to `testing.status.md`'s
   **Departs:** `4: an escalation record's person who asked is held only by the scripted engine's unit
   test (`escalation_records_walk_to_the_person_who_asked_instead_of_the_decider`): no shared story has
   a decider other than the person who asked; <what stopped it, in one clause>.` Update its date.
5. Run the hand-breaks. `heavy cargo nextest run -p jig-client-world --test b_shared`, and the binary that
   runs `a_tests.rs`; `cargo fmt`; `heavy cargo clippy -p jig-client-world --all-targets -- -D warnings`;
   `heavy cargo nextest run -p jig-client-world`; the citations check. Commit and push.

## Report

Which path, and why. The common note's table with both hand-breaks on the story path, or on the deletion
path that the unit test goes red on `requester: by`. Net lines, and the holder's form on each front.
