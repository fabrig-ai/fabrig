+++
id = "away-story"
part = "away"
kind = "feature"
needs = []
files = ["temper/tests/client/tests/suite/away.rs", "temper/tests/client/src/front/away.rs"]
effort = "high"
minutes = 120
+++

# Away's whole sequence on Home, in the joint world

## Goal

The process walk `away::a_goal_decides_while_away_then_waits_on_home_after_overturn_and_off`
(`temper/crates/temper/tests/fake_backends/walk/away.rs`) moves to the
joint world (`temper/tests/client`, package `temper-client-world`) as a
story of the same name in `temper/tests/client/tests/suite/away.rs`. The
five stories there hold the switch, a decision and the Ask answer apart;
none runs the walk's sequence through Overturn and off. The walk itself
is `walks-go`'s; you never touch it.

Acceptance: **`a_goal_decides_while_away_then_waits_on_home_after_overturn_and_off`**,
new, in this order, the person's presses made on the built view by the
visible names the walk presses (`Temper::act` on a node found by name in
the client's tree, as `suite::task` and `suite::chat_draft` press), the
goal's run played from the worker side through the real host-tool path
(`temper_engine_domain_world::host_tool::host_tool`, which decodes and
renders as smith does; never a typed answer rendered by the story):

1. **On.** Project 1's ways have goals on and Away available (as the
   walk's `project-ways` item: drafts off, goals on, changes off, away
   on). On Settings the person presses "I turn it off" and sees it
   selected; on Home presses "Away" and sees "agents decide within".
2. **A goal decides.** A tracked goal's run calls `decided` (question
   "Which local implementation?", options Small and Broad, took 0,
   reason "The small implementation fits the design.", kind
   implementation); its answer, as the run reads it, says carry on.
3. **Decided carries on.** The run's next call is its `note` with the
   walk's sentence ("I carried on with the small implementation before
   asking anyone."); the goal's page shows it; Home shows "One thing was
   decided for you.", the reason and "Small", and no "Try the other
   implementation".
4. **Overturn asks again.** The person presses "Overturn": the engine
   marks the decision overturned and the client's message reaches the
   goal's run (observe it as the run's input: "I overturn your
   decision…"). The run calls `message` asking the person ("Which local
   implementation after overturn?", choices "Try the other
   implementation" and "Keep the chosen implementation"). Home shows
   "Overturned: it asks you again", the button "Try the other
   implementation", and "Still needs you".
5. **Off and it waits.** The person presses the Away switch off and sees
   "Away is off: questions wait for you again."; answers "Try the other
   implementation" (the answer's hold is the clock's: advance past
   `needs::HOLD`, never a wait). The run's `decided` again is answered
   Ask, and no second decision is saved; its `message` asks "Which local
   implementation while here?" and Home shows it with "Use the small
   implementation", "One thing was decided for you." still there. The
   person answers it (the hold advanced again); the run finishes with its
   report; Home shows the Chats region and no Goals region.
6. The goal's committed calls start with `decided`, `note`.

- **Bite:** make the root answer `decided` with Ask while Away is on (or
  skip the overturn's saved mark) by hand, see the story fail at step 2
  or 4 for that reason, restore.
- Clippy clean, fmt clean, `temper-client-world`'s focused suite green.

## Inputs

- `temper/docs/plan/walks-lower/notes/common.md` (read first).
- `temper/docs/plan/walks-lower/inputs/request.md`, the `away::` ruling.
- `temper/docs/design/domain/testing.md`, 2.6, the bullet **The fake box
  keeps two walks**, its **Away** item; and 6.3.
- `temper/docs/design/client/README.md`, 3.5 (Home's Away and its
  holds), as the walk's module comment cites.
- The walk, for its words and order (read only):
  `temper/crates/temper/tests/fake_backends/walk/away.rs`, and its fake
  model's script for these cues in `temper/crates/temper-fake/src/backends.rs`
  (search "Walk the local away decision").

What the survey found, for your bearings (verify before relying):

- `suite/away.rs`'s own `Story` (`act`, `toggle`, `after`, `home`,
  `facts`, `decide`, `decisions`) and `Joint::with_away_people`,
  `Joint::begin_away_goal` (`temper/tests/client/src/front/away.rs`).
- Overturn is two requests from the client: the forge family's Overturn,
  then a message to the goal with the overturn's words
  (`temper/crates/temper-client-domain/src/pages/away.rs`, about 153 and
  204); the engine saves the mark
  (`temper/crates/temper-engine-domain-forge/src/away.rs`).
- A choice's answer is held `needs::HOLD`, five seconds
  (`temper/crates/temper-client-domain/src/pages/needs.rs`); advance the
  story's clock past it and pump, as `suite::chat_draft` does.
- `Temper::build` renders at `fixtures::NOW`; build at the story's wall
  for words that depend on time ("agents decide within").

## Owns and may change

Owns:

- `temper/tests/client/tests/suite/away.rs`: the new story and its
  private helpers.
- `temper/tests/client/src/front/away.rs`: one constructor or helper
  appended, if the story can't do without it (for instance a fixture
  whose goal's run is assigned and answers through `host_tool`). Add,
  never reshape: another draft also edits this tree.

Reader and neighbour files (edits allowed as stated):

- `temper/tests/client/src/front.rs`, `src/fixtures.rs`, `src/app.rs`:
  only a visibility widening or one small function appended at the end,
  named in the report.

Pre-approved options:

- **A press the view can't make:** if a step's button isn't on the
  built view in this world, send the same act the button sends and say
  so in the report; the step's words are still read from the view.
- **A step the world can't hold without a stub** (a seeded decision row,
  a forged Home document, a typed answer the story renders): write the
  rest, stop that step, and report it under `## Left` with what the
  world lacks; the walk then stays for it.

## Steps

1. Read the inputs, `suite/away.rs` whole and `front/away.rs`.
2. Write the story step by step, running it alone each time:
   `heavy cargo nextest run -p temper-client-world --test suite -E
   'test(=away::a_goal_decides_while_away_then_waits_on_home_after_overturn_and_off)'`.
3. Bite it as above; restore.
4. Once, at the end: `cargo fmt`; `python3 -I -B
   temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-client-world --all-targets -- -D warnings`; `heavy cargo
   nextest run -p temper-client-world` (check the count). Commit
   (`walks-lower: Away's sequence through overturn and off in the joint
   world`, the body saying why) and push.

## Report

As the implementer's text says: the first line; the commit pushed; each
run with its count; the bite and its failure; which presses went through
the view and which as acts; each step held, or under `## Left`; the
story's CPU seconds as nextest reports them; the minutes.
