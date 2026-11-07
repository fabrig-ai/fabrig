+++
id = "story"
part = "story"
kind = "feature"
needs = ["form", "reference"]
files = ["temper/tests/engine/domain/tests/suite/forge_routes/per_wake.rs", "temper/tests/engine/domain/tests/suite/forge_routes.rs"]
effort = "high"
minutes = 150
+++

# per-wake-temper: the story, in the engine's domain world: the goal's cut and its answer read their parts

## Goal

The request's first item, in temper's lowest world that holds temper's settings and a goal
(`temper/tests/engine/domain`, the real root, core and forge with a scripted worker): the
reference's settings, decoded as `temper ctl settings apply` decodes a file; a goal started
from template 4; its first run reads charter 5's shared text and its cut part at Opus 5.5
xhigh; a question reaching it during that run is kept until the run after, which reads the
shared text and the questions part at Opus 5.5 high, the goal never amended
(`temper/docs/design/protocol/configuration.md`, 13.9, "Charter 5 in parts" and template
4's bullet; jig's `jig/docs/design/domain/tasks.md`, 8.6). Everything it needs has merged;
this task writes the story and fixes nothing outside its files.

Acceptance:

- `temper/tests/engine/domain/tests/suite/forge_routes/per_wake.rs`, one `#[test]`,
  `the_reference_goal_reads_its_cut_then_its_questions_part`, registered beside
  `goal_branch` in `tests/suite/forge_routes.rs`, built as
  `reference_template_four_starts_on_the_configured_forge` (`forge_routes/goal_branch.rs`)
  is:
  - its settings: `temper/docs/examples/reference/settings.json` decoded by
    `temper_client_tools::jig::settings_file::decode` and carried to the core by ctl's
    translation (`jig_client_protocol_link::translate::asks::ask`, then
    `jig_protocol_engine_clients::translate::settings::items`, as `reference_template` does),
    given the world's catalogue models so template 4's `llm` resolves. Charter 5 and
    template 4 reach the world whole but for the substitutions the scripted world needs,
    as `goal_branch.rs` makes them: charter 5's `model` (and `alternatives`) the world's
    catalogue model named `claude-opus-5-5`, which declares `xhigh` and `high`; its
    `documents` empty; template 4's report bound the world's. Charter 5's instructions and
    parts and template 4's `llm` are the file's, unchanged;
  - the goal: made from template 4 as `reference_request` makes it, its member's `llm`
    the template's (not `None`);
  - row 1, the cut: the first assignment's `run.policy.instructions` is the shared text,
    `\n\n` and the cut part's instructions, each taken from the decoded file (never a
    literal of the charter's words); its model `claude-opus-5-5` at `xhigh`; the goal's
    stored task (`Driver::task` or the world's `live_task`) has `reasons` holding
    `Reasons::FIRST`;
  - row 2, kept: during that run a question reaches the goal, from a delegate it made in
    that run asking its requester with a question-form `message`
    (`engine::Tool::Message { form: engine::MessageForm::Question, .. }`, as
    `question_target.rs` sends one); it is not delivered to the live run (the world's
    delivered messages to the goal's run hold none of it) and keeps its place in the goal's
    inbox;
  - row 3, the answer: the first run parks with `wait`; the next run is assigned, its
    instructions the shared text, `\n\n` and the questions part's; its model
    `claude-opus-5-5` at `high`; its start messages carry the question; the goal's stored
    `llm` is the template's, main at `xhigh` and `later` at `high`, and no amendment was
    made;
  - the world's restart oracle (`world.close()`) passes at the end.
- The story bites: drop the parts from the decoded charter 5 before it reaches the world,
  and see rows 1 and 3 fail on the instructions; drop the template's `later`, and see row 3
  fail on the effort; restore each. The report names each failure seen.
- Its cost: one test in an in-memory world, no sleeps, the clock driven. Report its CPU
  time beside `reference_template_four_starts_on_the_configured_forge`'s, both measured on
  the same run; it stays within temper's focused budget
  (`temper/docs/design/domain/testing.md`, the focused and fuzzy figures).
- `heavy cargo nextest run -p temper-engine-domain-world --test suite -E
  'test(/forge_routes::per_wake/)'` passes, selecting one test, and the world's focused suite
  with it.

## Inputs

- `temper/docs/plan/per-wake-temper/notes/common.md`; `notes/contracts.md`, sections 4 and 5.
- `temper/docs/design/protocol/configuration.md`, 13.9: template 4's bullet and "Charter 5 in
  parts"; `temper/docs/design/draft/per-wake.md`, sections 2, 4 and 5.
- jig's `jig/docs/design/domain/tasks.md`, section 3 (a later choice) and 8.6 (a run's
  reasons, its instructions, what it is offered); jig's own story of the same,
  `jig/tests/core/tests/suite/per_wake.rs`, for what it observes.

## Owns and may change

- Owns the files in the header.
- Neighbours: `forge_routes/goal_branch.rs`'s `reference_authority` and `reference_request`
  may become `pub(super)` for this file, with no other change there; any other helper the
  story needs is written in `per_wake.rs`.
- Pre-approved: if the world applies a settings change (`tests/suite/configuring/fixture.rs`,
  `World::change`, or the service's `temper_service::clients::translation::to_root`), put the
  decoded items in by a change; otherwise seed them as the world's defaults as
  `reference_defaults` does; say which. If the cheapest question is a change's producer's,
  relayed (`World::producer_question`), use it in place of a plain delegate's and say so. If
  the reference's rules don't let charter 5 delegate the charter the story's delegate runs
  under, widen the world's rules and policy for it as `reference_authority` widens them for
  charter 5, and say so.
- Not yours: the reference's settings, the settings form, temper's or jig's crates. A fault
  the story finds in them is a `Left` item with the failure seen, the row marked ignored
  with that reason (`AGENTS.md`'s rule for a story failing outside its files), and the
  report's first line still `Done:` if the rest holds.

## Steps

1. The story, its rows failing first where they can (row 3's effort before `later` is
   read), then passing.
2. Bite it as above.
3. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo clippy -p
   temper-engine-domain-world --all-targets -- -D warnings`; `heavy cargo nextest run -p
   temper-engine-domain-world`.
4. Commit (why: the reference goal's cut and its answer each read their part at their
   effort, in temper's lowest world that holds both); push your branch.

## Report

As the implementer's text says, and also: how the settings reached the world, the question's
source, the substitutions made, the story's CPU time beside its neighbour's.
