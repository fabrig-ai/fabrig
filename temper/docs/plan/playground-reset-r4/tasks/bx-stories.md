+++
id = "bx-stories"
part = "box-groups"
kind = "feature"
needs = ["bx-wire", "bx-commands", "bx-script-forge", "bx-init", "rm-remove-forge", "fr-make-forge", "pd-delete", "pg-reset"]
files = [
  "temper/crates/temper/tests/box_stories/**",
]
effort = "high"
minutes = 60
+++

## Goal

The box's process-tier stories of the draft's section 8, first bullet:
a box killed during the playground's reset ends the reset whole at its next
start; an owned Forgejo set up killed at each of its steps is found
unfinished, started over by its removal, and set up afresh with Forgejo's
binaries taken from the cache. Written to build and to the design, **not
run** (common.md: implementers never run the slow tiers); the report says
so.

Acceptance:

- `temper/crates/temper/tests/box_stories/` holds the stories, reached from
  `tests/process.rs` by `#[path = "box_stories/mod.rs"] mod box_stories;`;
  `heavy cargo check -p temper --tests` builds them; `heavy cargo nextest
  list -p temper --test process` lists each test below (a listing builds
  and runs nothing of the stories).
- Each story asserts what the draft's sections 4, 6.2 and 6.4 say the end
  state is (below), from the store as `temper ctl` reads it, the box's
  terminal lines and the data and cache directories' files.

## Inputs

- `temper/docs/plan/playground-reset-r4/notes/common.md` and
  `temper/docs/plan/playground-reset-r4/notes/contracts.md` (sections 1, 3).
- `temper/docs/design/draft/playground-reset.md`, sections 4, 6.2, 6.3, 6.4
  and 8.
- `temper/docs/design/domain/testing.md`, the process tier's section (find
  "process").
- `temper/docs/plan/playground-reset-r4/notes/surveys/s2-playground.md`,
  section 6, and `s4-box.md`, section 5 (the process tier's harness and its
  nearest stories).
- The harness and the stories it models: `temper/crates/temper/tests/support/boxed.rs`
  (`run_story`, `run_ctl`, `Plan`, `dying_with_test`), `support/ctl.rs`
  (`Story`, `Step`, `Call`), `support/forge_land.rs` (`cache_forgejo_binaries`,
  `forgejo_setup_steps`, `assert_cached_binary_unchanged`), and in
  `tests/process.rs`: `a_second_forge_ends_the_playground_for_good_closing_its_chat_and_account`
  (box runs one after another on one data directory),
  `a_first_run_plays_the_demo_then_sets_up_a_forgejo_here_with_no_restart`
  (an owned set up on a first run, the cache asserted unchanged),
  `a_restart_after_a_cut_at_listening_answers_ctl` (a cut and a restart).
- The reports of the tasks this one needs, for the asks' ctl tools and
  answers: `pg-reset` (`EndPlayground` and its page), `rm-remove-forge`
  (`RemoveForge`), `fr-make-forge` (`SetUp` making the forge), `pd-delete`,
  `bx-init` (the first run's document naming no forge), `bx-wire`.

## Owns and may change

- Owns `temper/crates/temper/tests/box_stories/**` (new).
- Neighbours:
  - `temper/crates/temper/tests/process.rs`: the two lines that reach the
    module, beside `mod fake_backends;`, and nothing else.
  - `temper/crates/temper/tests/support/**`: a helper the stories need that
    two of them share (a kill on a terminal line, below) may go in a new
    file `support/cut.rs` with its `mod` line in `support/mod.rs`; no
    existing helper's behaviour changes.
- Options:
  - Driving the asks: through `temper ctl` where its tools have them (the
    playground's start and end, a forge's set up and removal: the tools the
    playground and engine parts add, named in their reports), default; else
    through the client's person scripts (`support/person.rs`), as
    `first_run` drives the welcome. Use what the base has; say which.
  - The kill: the box run with `--echo steps` (`support::boxed::echo_steps`),
    its stderr read line by line, and the box's whole tree killed with
    SIGKILL at the first line that marks the cut (default; as
    `a_story_box_and_its_steps_die_when_the_test_process_is_killed` kills a
    tree). If a line cannot mark a cut deterministically, cut at the next
    line that can and say which in the report.
  - One `#[test]` per cut (default), each within the process tier's
    per-test CPU budget (60 s CPU, `tools/tiers/budgets.toml`); if a story
    plainly cannot fit (several Forgejo set ups), keep it whole and say in
    the report what it is expected to cost: the budget is the user's
    question, carried by the lead.

## Steps

1. `box_stories/mod.rs` with `mod reset;` and `mod set_up;`, and a shared
   `fn restart_until(...)`: start the box again on the same directories and
   wait, with `support::boxed::BOX_DEADLINE`, until a ctl read says the
   state named is reached.
2. `box_stories/reset.rs`, on a first run's directories
   (`first_run_directories`, as the playground stories take them):
   - `a_box_killed_during_the_playgrounds_reset_ends_it_whole_at_its_next_start`:
     run 1: sign in, start the playground, wait for it running; send its
     end (`EndPlayground`); kill the box's tree at its terminal's
     `Playground: removed` (the box's half done, the engine's later kept acts
     not yet answered, or answered: either is a cut the store must survive).
     Run 2, the same directories: wait until the playground reads none;
     assert the store read as empty (draft 6.3): the `forges` subject lists
     no forge, no account of the scripted provider, `me` names no
     playground project; `<data>/box/groups` holds no file, the playground
     fake forge's directory (`application.demo.fake`'s `directory`) is gone;
     the welcome's state is the Forge step's (draft 6.2: what a person who
     pressed "Set up temper" finds). Then start the playground again: it
     runs on a forge numbered past the first, and its scripted CSV change
     lands (bx-script-forge: the scripts' forge follows the forge's number).
   - `a_box_killed_as_the_reset_is_asked_ends_it_whole_at_its_next_start`:
     the same, killed at the first line after the end is sent (the box's
     pipe's `remove`, or the serve's reply to the ask, whichever the
     terminal shows first).
3. `box_stories/set_up.rs`, the owned Forgejo of a first run (the cache
   prepared by `cache_forgejo_binaries`, the person's set up through ctl as
   `forgejo_setup_steps` drives it, now making a forge record and its group
   file): one test per cut, each named for its step:
   `a_set_up_killed_at_fetch_is_started_over_from_the_cache`,
   `..._at_prepare_...`, `..._at_run_...` (at the box's terminal line
   starting that step: `Starting fetch N`, `Starting prepare N`,
   `Starting forgejo N`), `..._after_started_...` (at the terminal's
   `Forge N (Forgejo): started`, before the engine reconciles: the record
   and part of the secrets written). No story cuts between the forge
   connected and its first repository adopted: the terminal marks no such
   line (see Answers). Each: run 1 cut; run 2: the `forges` subject
   lists the forge unfinished (contracts, 5: not connected with its first
   repository adopted); the forge is removed (`RemoveForge`, as Start over
   sends it) and the removal answers; assert `<data>/box/groups/N`,
   `box/started/N` and the forge's directory gone, and the cached Forgejo
   and runner binaries unchanged (`assert_cached_binary_unchanged`) with no
   fetch download (the box's terminal shows no fetch of a new file, or the
   cache's files keep their inode and mtime); then set up again: a new
   forge, numbered past N, sets up whole (connected, its first repository
   adopted), its binaries the cache's.
4. `cargo fmt`; `heavy cargo clippy -p temper --all-targets -- -D
   warnings`; `heavy cargo check -p temper --tests`; `heavy cargo nextest
   list -p temper --test process` (lists, runs nothing). Do not run the
   stories. Commit, push.

## Report

Fab's sections; that the stories were not run; for each story, its cut's
marking line, the asks it drives and through what (ctl tool or client
script), and its expected cost against the process tier's budget.

## Answers

2026-11-01. Question: for the connected-before-first-repository cut, may
the setup owner add a deterministic pause or terminal marker before
adoption, or may the story use the group-started cut? Neither: the fifth
story (`..._before_the_first_repository_...`) is dropped, and no production
pause or diagnostic is added. The draft's section 8 asks for a set up killed
at each of its steps, and the box's steps are fetch, prepare, run and the
group's started; the gap between connected and adopted is the engine's, with
no terminal line to cut on, and a cut timed from a ctl snapshot does not
replay (`AGENTS.md`, 2: a seed replays). That state is checked in the lowest
tier that shows it (`AGENTS.md`, 6): `fr-first-intent`'s service test
`an_intent_restored_with_its_forge_is_made_at_the_next_connect` and its qa
test `first_repository_intent_survives_a_restart`. The four step stories cut
at `Starting fetch N`, `Starting prepare N`, `Starting forgejo N` and
`Forge N (Forgejo): started`, the terminal lines the run found; step 3 now
says so. The report names the dropped story and where its state is checked.
