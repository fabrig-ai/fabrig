+++
id = "person-answers-chats"
part = "relay"
kind = "feature"
needs = []
files = ["jig/crates/jig-domain-core/src/routing/task.rs", "jig/crates/jig-domain-core/src/tests/**", "jig/tests/core/tests/suite/reads.rs", "jig/docs/design/domain/people.status.md"]
effort = "medium"
minutes = 40
+++

# person-answers-chats: a person answers the questions to their conversations, not to every task they asked for

## Goal

`Core::person_answers()` (jig's `domain/people.status.md`, Departs 6, 8) routes a question
to a task its person asked for into that person's inbox and refuses the task's own answer
(`Refusal::PersonAnswers`). jig's `domain/people.md`, section 6, gives the rule its reason
for a conversation alone ("so a conversation never answers for its person"), and temper's
`domain/agent.md`, 8.2 uses it for the chat's goal. Applied to every person-requested
task, it takes a tracked goal's questions from it: the reference's coordinator, a goal its
person starts from the template (`temper/docs/design/draft/reference-code-production.md`,
sections 4 and 8; `temper/docs/design/domain/forge.md`, 8.11), can never answer its
changes. With the switch on, a question to a task that is no conversation is the task's to
answer, as with it off.

Acceptance:

- `question_event` (`jig-domain-core/src/routing/task.rs`) makes a `PersonQuestion` only
  where the asked task's row is flagged conversation (`row.flags.conversation`, as
  `Core::conversation_requester` reads it) and its requester is a person; every other
  question stays a `Message`. Nothing else changes: the person's answer path
  (`sibling_routes.rs`) and the refusal in `routing/run/calls.rs` read the question's
  recorded person (`question_person`), which a question to a non-conversation no longer
  has.
- Tests, beside the existing person-answers ones in `jig/tests/core/tests/suite/reads.rs`:
  `a_question_to_a_persons_tracked_task_is_the_tasks_to_answer` (switch on; a person's
  task, no conversation, delegates a child that asks it: the question is not in the
  person's inbox, the task's `message` answer is admitted and reaches the child) and the
  existing conversation case still refused with `PersonAnswers`. Where an existing
  person-answers test's asked task is no conversation, flag it one (the rule's case), and
  name each in the report.
- Status: `people.status.md`'s Departs entry for 6, 8 says the switch covers a question to
  a conversation, its person answering it; a question to any other task its person asked
  for is the task's.

## Inputs

- jig's `docs/design/domain/people.md`, section 6; `people.status.md`, Departs 6, 8.
- `temper/docs/design/domain/agent.md`, 8.2 (the chat's goal, which keeps its route).
- The question and answer in `relay-story.md`, Answers.

## Owns and may change

The files in the header. temper's `temper-engine-domain/src/domain.rs` keeps
`core.person_answers()` as it is.

## Steps

1. The condition; the tests; the status line.
2. Narrow: `heavy cargo nextest run -p jig-core-world --test suite -E 'test(/reads::/)'`
   (check the count), then
   `heavy cargo nextest run -p jig-domain-core --lib`. The rule's temper readers:
   `heavy cargo nextest run -p temper-engine-domain-world --test suite -E
   'test(/forge_routes::b2_tree::/)'` (the chat's forged answer is still refused).
3. `cargo fmt`; `python3 -I -B scripts/old_forms.py` from `jig/`; `heavy cargo clippy -p
   jig-domain-core --all-targets -- -D warnings`. Commit ("A person answers their
   conversations' questions, not every task's") and push.

## Report

As the implementer's text says.
