+++
id = "change-batch"
part = "change"
kind = "feature"
needs = ["change-call", "change-held-release"]
files = ["temper/crates/temper-engine-domain/src/route/forge_route.rs", "temper/crates/temper-engine-domain/src/route/tools.rs", "temper/tests/engine/domain/tests/suite/forge_routes/change_tool.rs", "temper/docs/design/domain/agent.status.md", "temper/docs/design/domain/forge.status.md"]
effort = "high"
minutes = 100
+++

# change-batch: the root builds a plan's changes

## Goal

The root builds each change of a `change` call (task `change-call`'s `Tool::Change {
changes }`) into its delegate, with its landing branch, its dependencies and the whole call as
one batch (contract C4), and refuses the plan's members from a run whose authority lacks
jig's control family. A call of one change naming no plan member is decided exactly as today.

Acceptance:

- **Who may:** a call naming `name`, `landing` or `after` on any change, or more than one
  change, from a run whose task's authority lacks `ToolFamilies::JIG.control` is answered at
  once, before any delegate is built, as `goal::change_refusal`'s words are answered in
  `route/tools.rs` (`SettledAnswer::Host { error: true, body }`), the words: `change's
  landing, name, after and changes are a plan's: this task's authority lacks the control
  family (amend, cancel, release).`
- **Each change** is built as `chat_change` builds one today, from its own `words` and
  `repository`, with:
  - its landing branch, parameter 2 and the branch `configured_gates` reads: the
    repository's default branch with no `landing`; `Branch(name)` as given, refused
    `Refusal::Spec` when it does not split into branch parts (`branch_parts`); `Goal`, the
    branch `tree_branch(repository, root, b'g', task, None, name_bytes)` of the run's task
    (`<prefix><root>/g<task>`, C1), refused with the host words `landing goal: this task
    holds no goal branch in OWNER/NAME.` unless the forge's hold on that branch's resource
    (`domain.forge.hold(&name)`) is the run's task's;
  - its dependencies: `ChangeAfter::Task(n)` as `engine::Dependency::Existing(n)` (jig
    checks it is a live delegate, as for any batch); `ChangeAfter::Name(x)` as
    `engine::Dependency::Batch(i)`, `i` the index in the call of the earlier change named
    `x`. A name given twice is refused `Refusal::Duplicate`; a name no earlier change gives,
    `Refusal::Dependencies`.
  - its numbers from what the run has left after the changes before it in the call: tasks
    (`tasks_left`, less 1 + the earlier changes' delegation tasks; refused `Refusal::Tree`
    as today when that leaves 1 or less), and spend: half of what the run has left, shared
    equally among the call's changes, refused `Refusal::Funding` when a share is no more
    than a run's minimum.
- **One batch:** the call becomes one `Tool::Delegate { batch }` of every change's delegate in
  the call's order, decided as today's one (`delegate_call`), so jig makes all or none and
  its answer names each task.
- The stories below pass; every story of `forge_routes` passes unchanged.

## Inputs

- `temper/docs/design/domain/agent.md`, 4.3.1 (the delegate's bullets; "What a plan
  decides" and its bullets).
- `temper/docs/design/domain/forge.md`, 8.1 (the paragraph "A run asks for one with
  `change`").
- `temper/docs/design/domain/agent.status.md`: its **Not yet** entry for 4.3.1, which this
  task removes but for "a change's contract bounds its words at 512 bytes" (task
  `change-words`'s); `forge.status.md`: its **Not yet** entry for 8.1 ("a change's landing
  branch other than its repository's default ..."), which this task removes.
- `temper/docs/plan/reference-m1/notes/common.md`; `notes/contracts.md`, C1 (the goal
  branch's name) and C4 with its line under "Names the families write here".

## Owns and may change

Owns the files in the header.

- `forge_route.rs`: `chat_change` takes the change's landing branch (or the `ChangeCall`)
  and its numbers' share; a new `pub(crate) fn change_batch(domain, env, task, context,
  changes: &[ChangeCall]) -> Result<Box<[super::Delegate]>, ChangeRefused>`, with `enum
  ChangeRefused { Refusal(tasks::Refusal), Words(Box<[u8]>) }`, beside it. Keep the hunks to
  `chat_change`, `change_parameters` and the new code: the file is changed on plan
  reference-m8's trunk too, and `change_decision` is task `change-held-release`'s.
- `tools.rs`: the `Tool::Change` arm calls `change_batch`, answering `Words` as host error
  words and `Refusal` as today's refused expansion; no other hunk.
- Reader files: `chat_change`'s other callers, if any (`rg 'chat_change\('`), call sites
  only.
- Options: if the forge holds no hold for a goal branch the goal family's code has not made
  yet, the `Goal` refusal stands; do not make the hold here. If `Dependency::Batch` indexes
  count from another origin than the call's order, map them as the delegate decoder does
  (`temper-engine-smith/src/nested.rs`) and say so.

## Steps

1. The batch, the members, the arm.
2. Stories in `temper/tests/engine/domain/tests/suite/forge_routes/change_tool.rs`, a run's
   authority given the control family where named (`assignment.run.authority.tools` or the
   world's policy, as the stories set tools):
   - `a_plan_member_from_a_task_without_control_is_refused`: the stock chat calls `change`
     with `landing: "main"`: the host words above, no delegate.
   - `two_changes_in_one_call_are_one_batch_the_second_after_the_first`: with control,
     `changes: [{name: "a", words}, {words, after: ["a"]}]`: two change tasks made in one
     decision, the second's dependencies the first's task; the first's producer starts and
     the second's does not until the first has landed (drive the first to landing as
     `plain_change.rs` does), then the second's starts.
   - `a_change_names_its_landing_branch`: with control, `landing: "release"` on a fake
     forge holding a branch `release`: the change's parameter 2 is `release` and its pull
     request's base is `release`.
   - `landing_goal_without_a_held_goal_branch_is_refused`: with control, `landing: "goal"`
     from the chat: the host words above.
   - `an_after_naming_no_earlier_change_is_refused`: `after: ["b"]` with `b` later in the
     call: `Refusal::Dependencies`, no delegate.
3. Narrow: `heavy cargo nextest run -p temper-engine-domain-world --test suite -E
   'test(/change_tool::/)'` (check the five ran). Then `heavy cargo nextest run -p
   temper-engine-domain-world --test suite -E 'test(/forge_routes::/) | test(/settings::/)'`
   and `heavy cargo nextest run -p temper-engine-domain --lib`.
4. The status edits. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-engine-domain -p temper-engine-domain-world --all-targets -- -D
   warnings`. Commit ("Build a plan's changes as one batch") and push.

## Report

As the implementer's text says, naming how a run's authority got the control family in the
stories.
