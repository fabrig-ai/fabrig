+++
id = "jig-scripted-requester"
part = "jig-scripted-rules"
kind = "feature"
needs = ["jig-scripted-order"]
files = [
  "jig/testing/jig-client-scripted-engine/src/requests.rs",
  "jig/testing/jig-client-scripted-engine/src/requests/decisions.rs",
  "jig/tests/client/src/stories/shared/**",
  "jig/tests/client/src/stories/closing.rs",
  "jig/tests/client/src/stories/conversations.rs",
  "jig/tests/client/src/stories.rs",
  "jig/docs/development/debt/D028-scripted-engine-reimplements-core.md",
  "jig/docs/development/debt/README.md",
  "jig/docs/design/client/domain/testing.status.md",
]
effort = "high"
minutes = 90
+++

# jig D028, requester choice: the fake names who the core names, and the item goes

## Goal

The fake picks requesters itself. An escalation's record names the decider where the core walks
up to the person who asked, a task holder gives its requester standing where the core gives none,
and nothing holds a member to their own chats. The fake takes the core's answer in each, shared
stories hold the two a person can see, and this task, the last on
`jig/docs/development/debt/D028-scripted-engine-reimplements-core.md`, deletes the item.

Acceptance:

- `decisions.rs`'s record names the person the walk reaches, and `recipient` gives a task holder no
  standing for an escalation.
- Two new stories pass on the scripted and joint fronts, as sketched.
- `stories/closing.rs` and `conversations::without_chats` are gone: nothing calls them.
- The item's file and its README row are gone. `python3 -I -B tools/citations/citations.py --all`
  reads 0. D031's `D028` is skein's item, and stays.
- `jig/docs/design/client/domain/testing.status.md`'s **Not yet** has no entry for the shared
  stories, and its **Departs** stands.
- The note's `What a task runs` passes, with `-p jig-ops-world`.

## Inputs

- `temper/docs/plan/debt-4/notes/common.md`, `temper/docs/plan/debt-4/notes/jig-scripted.md`, `AGENTS.md`,
  `jig/AGENTS.md`.
- `jig/docs/development/debt/D028-scripted-engine-reimplements-core.md`, whole.
- `jig/docs/design/client/domain/testing.md`, 4 and 5.
- The core: `jig-domain-tasks/src/escalation.rs`, `context`; `jig-domain-core/src/admission/escalation.rs`,
  `escalation_standing`; `jig-domain-core/src/documents/task_query.rs`, 150, and `src/inbox.rs`,
  `conversation_requester`.

## Owns and may change

- Owns: the files above.
- May change: `E/tests/**`, where an assertion read the removed answer; each such edit is in your
  report.
- Pre-approved: if the joint front cannot reach a decider other than the requester in this task's
  size, the record's story has the requester decide, a delegate between them, and a unit test in
  `E/tests` holds a decider other than the requester. Say so in your report.
- Pre-approved: if a scripted story relied on a task holder's standing, it was holding the fake,
  so give its fixture the person's holder; if a proposal's holder is the case at hand, leave it:
  `recipient` changes for escalations only.
- Not here: the `Unread` the fake reads from its source, which is a fixture's count; the note's
  query admission and counts, which the status records.

## Steps

1. In `decisions.rs`, the record's `requester` is the walk:

   ```rust
   /// The person a task's requesters reach, or 0 for the deployment
   /// (jig-domain-tasks's `context`).
   fn person_who_asked(truth: &crate::Truth, task: &Task) -> u64;
   ```

   It follows `Requester::Task` through `truth.objects`, bounded by `lineage`'s length.
2. In `requests.rs`, the `waiting` check takes the escalation's `Task` holder as no standing; a
   `Proposal`'s `Recipient::Task` stays. A unit test in `E/tests` holds it.
3. Two stories:

   ```rust
   /// In `b_conversations.rs`: another member's chat is not in this person's list.
   pub fn a_person_s_chats_are_their_own<F: Front<Test>>(world: &mut World<Test, F>);
   /// In `b_held.rs`: the decision's record names the person who asked, through a delegate.
   pub fn a_delegate_s_escalation_record_names_the_person_who_asked<F: Front<Test>>(world: &mut World<Test, F>);
   ```

   The first seeds `Seed::Conversation` for party 3 beside party 1's. The second reads the ended
   escalation's page, whose `P::Requester` row `jig-client-view/src/tree/fragments/ended.rs` writes.
   One line each in `person_story_suite!`.
4. Delete `stories/closing.rs`, its `pub mod` line, and `without_chats`.
5. Status: remove the **Not yet** entry; date line today's. Delete the item and its row. Report
   `find jig/testing/jig-client-scripted-engine -name '*.rs' | xargs cat | wc -l`: the item has no
   target for it, and most of the fake is its codec and tests.
6. What the note's `What a task runs` names, then the citations check with `--all`. One commit for
   the code, one for the debt and status text. Push.

## Report

The fake's lines at the plan's base (15,420) and now; each story's fronts; each pre-approved option
you took; each `E/tests` edit.
