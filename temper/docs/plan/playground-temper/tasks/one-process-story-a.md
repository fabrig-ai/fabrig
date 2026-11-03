+++
id = "one-process-story-a"
part = "story"
kind = "feature"
needs = ["one-process-composed", "story-a"]
files = [
  "temper/tests/one-process/src/stories.rs",
  "temper/tests/one-process/src/stories/**",
  "temper/tests/one-process/src/world.rs",
  "temper/tests/one-process/src/world/**",
  "temper/tests/one-process/src/referee.rs",
  "temper/tests/one-process/src/referee/**",
  "temper/tests/one-process/tests/suite/one_process.rs",
  "temper/tests/one-process/tests/suite/story_a.rs",
  "temper/tests/one-process/tests/suite/main.rs",
]
effort = "high"
minutes = 120
+++

# Story A walked natively in the one-process world

## Goal

The gate runs the world the browser runs (`temper/docs/design/playground.md`, 3): the one-process
world, which composes `temper-playground` with its harness, walks story A from its documents
(`stories::A`), its fake person in front of the terminal client, under its referee: from the
welcome to the CSV change landed and the other two requests answered (`playground.md`, 5.2 and
9). This is story A walked natively; the web client's host walks it later, through skein's
bridge.

Acceptance:

- A test `story_a_is_walked_from_the_welcome_to_the_csv_change_landed` in
  `tests/suite/story_a.rs` (its `mod` line in `main.rs`): the one-process world started from
  `temper_playground::stories::A.start(fixtures::SEED, wall)`, its harness's edits applied through
  `Harness::configure` and its links kept (`Links::Keep`) for its referee as its other stories do;
  the person signs in by the printed link, presses "Try the playground" on the welcome, waits for
  the playground's project, then asks the three suggested requests in the playground's chat as
  their buttons or the composer offer them:
  - "Add CSV export to widget": delegated, edited, pushed, passed by CI and landed: the fake
    forge's `playground/widget` `main` holds the change's commit, its CI green at its head;
  - "Add a dark theme to widget" and "Fix widget's empty search state": each answered in text,
    the template's reply seen in the chat.
- The verdict is `Passed` under the one-process world's referee (jig's conformance referee and the
  engine's over the fake forge: nothing lands on `main` without green CI).
- The story runs at the documents' pace (story A's `reply_ms` and `ci_ms`) on the simulator's
  clock. It aims at 2 s of the test's own time (`temper/docs/design/domain/testing.md`, 6.1);
  report the time it takes and, if past 2 s, the step that costs it.
- Every other test of the crate passes with the trunk's count.

## Inputs

- `temper/docs/plan/playground-temper/notes/common.md` and `notes/contracts.md`, C3 and C4.
- `temper/docs/design/playground.md`, 2 (what the visitor sees), 5.2 and 9.
- `temper/docs/design/protocol/setup.md`, 8.4 ("Try the playground", the box's playground).
- The words the person finds: `temper/tests/client/tests/suite/playground.rs`
  (`try_run_and_leave_the_playground_through_its_banner`, ~73) and the client view's playground
  pages (`temper/crates/temper-client-view`), read only.

## Owns and may change

- `stories.rs` (or `stories/`): the story's person (its `Step`s, as `person` ~35 does) and its
  outcome's reads (the forge's `main`, the chat's answers), as the other stories' are.
- `world.rs`: a `Story::PlaygroundA` (or a run function beside `run`) that starts from
  `stories::A` in place of the harness's coded fakes; the `Outcome` fields the test reads (e.g.
  `landed`, `answered`), each set as today's are. Its other stories do not change.
- `referee.rs` and `referee/`: only if story A's fakes need the referee to read them where it reads
  the coded fakes today (the forge's records), edited for that alone.
- The test file and its `mod` line.
- If the walk cannot reach the landing because of the product (a refusal, a stall), that is a
  finding: keep the test, mark it `#[ignore = "<the finding, in one line>"]`, report it under
  `## Left` with the world's log lines that show it, and add nothing to work around it.
- If "Try the playground" asks the box for a group or the reset is needed, it is not this story's
  (`playground.status.md`, 4): the story ends at the third answer.

## Steps

1. The story's start and its person; `heavy cargo nextest run -p temper-one-process-world --test
   suite -E 'test(=story_a_is_walked_from_the_welcome_to_the_csv_change_landed)'`.
2. `heavy cargo nextest run -p temper-one-process-world`, count against the trunk's plus one.
3. `cargo fmt`; old forms for temper; `heavy cargo clippy -p temper-one-process-world
   --all-targets -- -D warnings`. Commit; push.

## Report

As the implementer's text says, and: the person's steps; the story's time; each step the world
took longest on.
