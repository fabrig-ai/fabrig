+++
id = "goal-story"
part = "goal"
kind = "feature"
needs = []
files = ["temper/tests/client/tests/suite/task.rs"]
effort = "high"
minutes = 90
+++

# A chat's goal reached from its Goals row to its report, in the joint world

## Goal

The process walk `goal::a_chats_goal_on_the_fake_box_shows_its_report_in_the_goals_list`
(`temper/crates/temper/tests/fake_backends/walk/goal.rs`) moves to the
joint world as `a_chats_goal_shows_its_report_from_its_goals_row` in
`temper/tests/client/tests/suite/task.rs`, beside
`a_finished_goals_page_shows_its_report`. That story starts at a finished
goal's address, made by a fixture with no chat and no report note; the
new one keeps the route: the chat's run starts the goal, the goal's run
writes its report and finishes, and the person goes from Home's Goals
link to the row and from the row to the report. The walk itself is
`walks-go`'s; you never touch it.

Acceptance: **`a_chats_goal_shows_its_report_from_its_goals_row`**, new,
in this order, every run played from the worker side through the real
host-tool path (`temper_engine_domain_world::host_tool::host_tool`, as
`suite::chat_draft::write` uses it), never a seeded goal, report or page:

1. Project 1's ways have goals on. A chat's run calls `goal` with the
   walk's brief ("Write a report for this local demonstration."); the
   engine makes the goal; the chat's reply ("I started a goal to write
   its report.") is committed and shown on the chat.
2. The goal's run is assigned; it calls `note` in its own scope with its
   report (description `report: The local goal wrote its report.`, a
   body holding "I wrote this report on the local fake box." and the
   walk's Left and Risk lines), then `finish` with form `report` and the
   headline "The local goal wrote its report.".
3. The person on Home presses the "Goals" link (the walk only checked
   it; press it) and reaches Goals, all: the heading "1 goal, 0 under
   way.", the headline as the row's paragraph, and a row link starting
   "Write a report for this local demonstration." that says "landed".
4. The person presses that row and reaches the goal's page: its H1 is
   the headline, a "Report" region holds "I wrote this report on the
   local fake box.", and a "Landing" H2 is there.

- **Bite:** break the route once by hand (for instance make the Goals
  row's link address another task, or drop the report note's body from
  the goal page's read), see the story fail at step 3 or 4 for it,
  restore.
- Clippy clean, fmt clean, `temper-client-world`'s focused suite green.

## Inputs

- `temper/docs/plan/walks-lower/notes/common.md` (read first).
- `temper/docs/plan/walks-lower/inputs/request.md`, the `goal::` ruling.
- `temper/docs/design/domain/testing.md`, 2.6, the bullet **The fake box
  keeps two walks**, its **A chat's goal and its report** item; and 6.3.
- `temper/docs/design/client/README.md`, 5.3, 5.8 and 5.13, and
  `temper/docs/design/domain/agent.md`, 8.3 (a goal's report), as the
  walk's module comment cites.
- The walk, for its words (read only):
  `temper/crates/temper/tests/fake_backends/walk/goal.rs`, and its fake
  model's script for these cues in `temper/crates/temper-fake/src/backends.rs`
  (search "Write a report for this local demonstration.").

What the survey found, for your bearings (verify before relying):

- `fixtures::finished_goal` (`temper/tests/client/src/fixtures.rs`, about
  826) replaces the front with `Joint::with_questions` and ends a goal
  with `End::Finished(Report)`; it writes no note and has no chat, so it
  is not this story's fixture.
- `Joint::with_chat_draft` (`temper/tests/client/src/front.rs`, about
  632) is a two-slot chat and goal configuration with delegation and
  notes authority; `fixtures::chat`, `chat_page`, `task_page`, and
  `suite::chat_draft::write` show a chat's run calling a host tool and
  committing its transcript.
- Goals' address: `Address::Goals { project, filter: GoalsFilter::All }`;
  presses by name: `Step::Press` with `Find::named`, or `Temper::act` on
  the found node, as the stories of `task.rs` do.

## Owns and may change

Owns:

- `temper/tests/client/tests/suite/task.rs`: the new story and its
  private helpers.

Reader and neighbour files (edits allowed as stated):

- `temper/tests/client/src/fixtures.rs`: one small function appended at
  the end, if the story can't do without it (for instance a fixture
  with ways set for goals), named in the report. Add, never reshape:
  another draft also edits this tree.

Pre-approved options:

- **The fixture:** if `with_chat_draft` can't hold the goal's run and its
  note, compose one in the suite file from the helpers it uses; if no
  composition can make the goal from the chat's call without a stub,
  write steps 2 to 4 on a goal the chat's call makes in the nearest way
  the world allows, and report step 1 under `## Left`.

## Steps

1. Read the inputs, `suite/task.rs` around `a_finished_goals_page_shows_its_report`
   and `suite/chat_draft.rs`'s `write`.
2. Write the story, running it alone:
   `heavy cargo nextest run -p temper-client-world --test suite -E
   'test(=task::a_chats_goal_shows_its_report_from_its_goals_row)'`.
3. Bite it as above; restore.
4. Once, at the end: `cargo fmt`; `python3 -I -B
   temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-client-world --all-targets -- -D warnings`; `heavy cargo
   nextest run -p temper-client-world` (check the count). Commit
   (`walks-lower: a chat's goal reached from its Goals row in the joint
   world`, the body saying why) and push.

## Report

As the implementer's text says: the first line; the commit pushed; each
run with its count; the bite and its failure; each step held, or under
`## Left`; the story's CPU seconds; the minutes.
