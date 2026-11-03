+++
id = "relay-story"
part = "relay"
kind = "feature"
needs = ["relay-foreman", "questions-panel", "person-answers-chats"]
files = ["temper/tests/engine/domain/tests/suite/forge_routes/goal_questions.rs", "temper/tests/engine/domain/tests/suite/forge_routes.rs", "temper/tests/engine/domain/src/forge_routes.rs"]
effort = "high"
minutes = 70
+++

# relay-story: a goal's changes ask, and their questions are settled

## Goal

The milestone's story, on the engine's domain world (fakes only): a goal holding its own
branch g makes its changes into g by one `change` call with questions `foreman`; their
producers ask, a foreman settles one question and escalates another, the goal answers, the
producers go on, and both changes land into g; the change's panel shows each open question
while it waits. It runs every real side of the relay together, as reference-m1's
`goal_changes.rs` runs a goal's changes.

Acceptance, one story each, in a new
`temper/tests/engine/domain/tests/suite/forge_routes/goal_questions.rs`, set up as
`goal_changes.rs` is (its world, its goal request, its landing rules for main and for goal
branches):

- `a_foreman_settles_a_question_and_the_change_lands`: the goal's call makes changes `a`
  and `b` (`b` after `a`) into g with `"questions": "foreman"`; `a`'s producer asks; the
  change's panel (the forge's `change` subject) shows the question `with foreman F`; F's
  verdict `answer` reaches the producer; the goal hears nothing of it; the producer
  finishes and `a` lands into g, its result carrying its producer's report.
- `an_escalated_question_reaches_the_goal_and_its_answer_the_producer`: `b`'s producer asks;
  its foreman escalates; the goal's inbox holds the question, then `Foreman:` and the
  foreman's reasons; the panel shows it `asked of requester`; the goal answers; the answer
  reaches `b`'s producer, which finishes, and `b` lands into g.
- `an_answer_that_adds_a_dependency_waits_for_it`: the goal answers `b`'s escalated
  question after making a change `c` and amending `b`'s `after` to it; the panel shows the
  question `answered, held for dependencies`; `b`'s producer hears the answer only once `c`
  has landed into g.
- `a_change_past_its_questions_holds_and_the_goal_releases_it`: with the world's
  `forge.change_policy.questions` at 1, `a`'s second question holds `a`, the hold's words
  naming the bound and the question; the goal hears the escalation, releases it, and the
  question reaches the goal with no foreman; its answer reaches the producer.

## Inputs

- `temper/docs/design/domain/forge.md`, 8.11; `temper/docs/design/draft/reference-code-production.md`,
  section 8 (the story it executes); `temper/docs/design/domain/forge.md`, 11.1 (g).
- `temper/docs/plan/reference-m5/notes/common.md`; `notes/contracts.md`, C5 to C8.
- `temper/tests/engine/domain/tests/suite/forge_routes/goal_changes.rs`, the model.

## Owns and may change

Owns the files in the header: the new story file, its `mod` line, and helpers it needs in
the world's `src/forge_routes.rs` (a scripted producer asking with `message`, a scripted
foreman's verdict, the goal's answer), added beside the existing helpers without changing
them.

- If a story fails for code outside the header (the relay, the panel), never fix that code
  here: push the stories that pass, and ask, naming the failing story, the file and line
  where the fault shows, and its log.
- If the world's `tasks.batch` makes an answer and the next question go over two steps,
  the story steps twice.

## Steps

1. The helpers; the four stories.
2. Narrow: `heavy cargo nextest run -p temper-engine-domain-world --test suite -E
   'test(/forge_routes::goal_questions::/)'` (check the count: 4), then `-E
   'test(/forge_routes::goal/)'`.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-engine-domain-world --all-targets -- -D warnings`. Commit ("Story: a goal's
   changes ask and land") and push.

## Report

As the implementer's text says, with each story's steps and wall time.

## Answers

2026-11-03. Q (run `relay-story-1`): keep the person-owned goal of `goal_changes.rs` and have
a jig change let it answer, or delegate the story's goal beneath a chat? A: keep the setup
as it is: the reference's coordinator is a goal its person starts from the template
(`reference-code-production.md`, sections 4 and 8), so its answering its changes is the
story. The fault is jig's: `Core::person_answers()` puts a question to any task its person
asked for in the person's inbox, where jig's `domain/people.md`, section 6 gives it a
conversation's reason alone. Task `person-answers-chats` (now a need of this one) narrows it
to conversations; this task changes no jig or root file. Once it lands, take the
`#[ignore]`s off the three blocked stories, run them, and finish as the Steps say. The
`change_tool.rs:147` failure (`questions_foreman_is_not_offered_yet`) is not this task's:
keep it under Left.
