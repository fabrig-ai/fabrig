+++
id = "public-index"
part = "jig-public"
kind = "feature"
needs = ["public-limits", "public-vocabulary", "visibility-core"]
files = ["jig/crates/jig-domain-core/src/public/index.rs", "jig/crates/jig-domain-core/src/public/index/**", "jig/crates/jig-domain-core/src/public/line.rs", "jig/crates/jig-domain-core/src/tests/public/index.rs", "jig/crates/jig-domain-core/src/tests/public/index/**"]
effort = "high"
minutes = 150
+++

# The core's public index: which tasks are public, kept with every decision, bounded

## Goal

The core keeps, for each public project, its public goals and tasks by people.md 13.7's one
rule, within `core.public.*`, updated in the step of every decision that changes what is
public, so a public read examines it alone (`jig/docs/design/domain/people.md`, 13.7 and
13.9; `jig/docs/design/domain/engine.md`, section 11). Nothing reads it yet but its tests;
`public-reads-core` answers from it.

Acceptance:

- A module `index` of the core's `public` module (`src/public/index.rs`, crate-private), and
  `src/public/line.rs`, the one maker of a `PublicLine` from a live task:

  ```rust
  /// The public index (jig/docs/design/domain/people.md, 13.9).
  pub(crate) struct Index { /* per public project, within Limits.public */ }
  pub(crate) enum Entry<'a> { Busy, Ready(&'a Project) }
  impl Index {
      /// Build the project's entry from its live tasks; called when it is made public and,
      /// for each restored public project, once the start has restored the live tasks.
      pub(crate) fn publish(&mut self, core: &CoreTasks, limits: &PublicIndexLimits, project: u32);
      /// Forget the project: made private, or its deletion admitted.
      pub(crate) fn withdraw(&mut self, project: u32);
      /// Follow one task's change in the decision that commits it (13.7's last bullet).
      pub(crate) fn follow(&mut self, core: &CoreTasks, limits: &PublicIndexLimits, change: Change);
      /// The one path a read takes: None for a project not in the index, whatever the reason.
      pub(crate) fn project(&self, project: u32) -> Option<Entry<'_>>;
  }
  impl Project {
      pub(crate) fn live(&self) -> &[u64];           // live public goals, by priority, within `goals`
      pub(crate) fn live_folded(&self) -> u32;       // live public goals past the bound
      pub(crate) fn ended(&self) -> &[Frozen];       // most recent first, within `ended`
      pub(crate) fn goal_of(&self, task: u64) -> Option<u64>; // None: not a public task
      pub(crate) fn tree(&self, goal: u64) -> Option<(&[u64], u32)>; // lines in tree order, folded
      pub(crate) fn lineage(&self, task: u64) -> Option<&[u64]>; // nearest first, ends at the goal
      pub(crate) fn delegates(&self, task: u64) -> (&[u64], u32); // public, within `lines`, folded
      pub(crate) fn counts(&self, task: u64) -> PublicCounts;     // public delegates by phase
      pub(crate) fn public_tasks(&self) -> Box<[u64]>;            // ascending, live and frozen
  }
  pub(crate) struct Frozen { pub line: PublicLine, pub tree: Box<[PublicLine]>, pub folded: u32 }
  pub(crate) enum Change { Made { task: u64 }, Tracked { task: u64 }, Untracked { task: u64 },
                           Moved { task: u64 }, Ended { task: u64 }, Priority { task: u64 } }
  ```

  The names inside are yours (`CoreTasks` stands for whatever borrow of the core's live tasks
  the module needs); the outer shape binds: one `project()` lookup is the only way in, a
  project absent and a project excluded are the same `None`, and nothing in the module reads
  the store. `Core` gains a field `public_index: public::Index`.
- **The rule (13.7), one function:** a public goal is a tracked task (`tracked.is_some()`) of a
  public project (`Core::public_project`, `visibility-core`) none of whose ancestors is tracked
  (its parent is in no goal's tree; a goal beneath a conversation is one, the conversation
  never in the index); a public task is a public goal or a task in its tree within the bounds
  (`lines` per goal in tree order, `depth` below the goal); nothing else (a conversation and
  its delegates outside every goal's tree, a recurring task, another project's, one past the
  bounds). A goal inside another goal's tree is a line of that tree. Ancestry stops at the
  public goal: its line's `parent` is `None`, a lineage ends at it.
- **Filtered first:** every count (`counts`, `live_folded`, a tree's or delegates' folded
  count) counts public tasks alone (a task past the bounds counts in its list's folded count,
  never anywhere else), made after the filter; nothing is counted or ordered before it.
- **Truncated by rule (13.9):** live goals by priority keep the first `goals`, the rest
  folded; lines in tree order keep the first `lines`; an ended goal's lines are frozen
  (`line.rs`) in the decision that ends it, and the ended list keeps the `ended` most recent,
  the oldest leaving (no longer public, its card unknown).
- **Kept in every decision that changes what is public:** a task made, tracked or untracked,
  moved (`Ask::Move`, a task adopted as a person's own goal, leaves its old tree and becomes a
  public goal of its own), ended, a goal's priority changed, and the project made public,
  private or deleted (`withdraw` at the decision that admits its deletion). A task moved under
  another task (people.md 5.1's `Move` under another task) is not built
  (`jig/docs/design/domain/people.status.md`): no change for it, said in the module's doc.
- **Built before it is read:** at start, once the live tasks are restored, each public
  project's entry is built from them; a read of a project whose entry is not built is
  `Entry::Busy`. Ended goals: see the options below.
- **Bounded memory:** at most `people.projects` entries, each within `goals + ended` goals of
  `lines` lines (`PublicIndexLimits`); every bound zero (every profile) keeps nothing public.
- Unit tests in `src/tests/public/index.rs` (a module line in `src/tests/public.rs`), each a
  core driven through its steps as the core's other tests are, asserting the index's answers
  through `project()` alone:
  - `a_goal_beneath_a_conversation_is_a_public_goal_with_no_parent`: its line's parent is
    `None`, the conversation is in no list, count or `public_tasks`, and `goal_of` it is
    `None`;
  - `a_conversations_other_delegates_count_nowhere`: a conversation with a public goal and an
    untracked delegate beside it; every count and folded count is the same as with the
    delegate absent;
  - `a_nested_goal_is_a_line_of_the_outer_tree`;
  - `a_task_past_the_depth_or_the_lines_is_not_public`: `goal_of` `None`, counted only in its
    list's folded count;
  - `live_goals_keep_their_first_by_priority_and_fold_the_rest`, and after a priority change;
  - `an_ended_goal_is_frozen_and_the_oldest_past_the_bound_leaves`;
  - `untracking_a_goal_takes_its_tree_out` and `tracking_a_task_makes_it_public`;
  - `a_moved_task_leaves_the_tree_it_left`;
  - `a_private_a_deleting_and_an_unknown_project_are_the_same_miss` (`project()` is `None`
    for each, from the deciding step);
  - `a_project_is_busy_until_its_entry_is_built`;
  - `with_every_bound_zero_nothing_is_public`;
  - `the_index_stays_within_its_bounds` (entries, goals and lines at their limits, one more of
    each refused or folded, never held).

## Inputs

- `temper/docs/plan/public-reading-build/notes/common.md`; `notes/contracts.md`, sections 1
  (`PublicIndexLimits`) and 4 (`PublicLine`, `PublicCounts`, `PublicPhase`, `PublicHold`,
  `PublicExecutor`).
- `jig/docs/design/domain/people.md`, 13.7 and 13.9 (and 13.3's public line);
  `jig/docs/design/domain/engine.md`, section 11 (public reads);
  `jig/docs/design/client/domain/contract.md`, section 12 (each field's source);
  `jig/docs/design/domain/people.status.md` (what of `Move` is built).
- As they stand: `jig-domain-core/src/admission/goal_of.rs` (walking requesters);
  `jig-domain-tasks/src/control.rs` (`tracked`), `flags.rs` (`conversation`), `recurring.rs`;
  `jig-domain-core/src/routing/view/marks.rs` and `routing/step.rs` (where the core already
  follows tasks' changes for watches); `src/public.rs` (`public-vocabulary`); the core's
  start (`start.rs`, its Open phase).

## Owns and may change

- Owns the files in the header.
- `jig/crates/jig-domain-core/src/public.rs`: the module lines `mod index; mod line;` and
  nothing else (`public-vocabulary`'s, merged). `src/tests/public.rs`: the module line alone.
  `src/lib.rs`: the `Core` field and its constructor line.
- **Hooks, one call each,** where the core already sees the change: the site(s) where it
  follows a task made, tracked, untracked, moved, ended or reprioritized (the report names
  each); `routing/party/projects.rs`'s `visibility_request` (`publish` / `withdraw`);
  `admission/deletion.rs`'s `begin` (`withdraw`); the start's Open phase (`publish` for each
  restored public project).
- Not this task's: answering a read (`public-reads-core`), the root's admission, the store.
- **Ended goals at start and at publication** (people.md 13.9, "Built before it is read"):
  the store keeps no index of a project's ended goals (`jig/docs/design/protocol/store.md`,
  7.4: ended tasks are indexed by requester, and by project only when *listed*), so loading
  them would scan. Default, as people.md 13.9 says ("Ended goals since it was built"): the
  ended list starts empty at start and at publication, fills from the decisions that end
  public goals after, and the entry is built at once; the module's doc says so. Pre-approved
  instead, only if the trunk then holds an index that yields a project's ended goals newest
  first: load them once per project within `ended × lines`, the entry `Busy` until loaded,
  through the store's paged load as the ended reads use it, adding a `Held`/`Event` pair only
  if no existing load can carry it (appended last, routed in `jig-test-domain` and temper's
  root as `Held::ReadLoad` is), and say which.
- Pre-approved: if the core cannot follow a priority change without scanning the project's
  live goals, rescan them in that decision (bounded by the tasks' limits; never in a read) and
  say so.

## Steps

1. The rule and the index's shape; `publish`/`withdraw`; `heavy cargo check -p jig-domain-core --tests`.
2. `follow` and its hooks; the start; `line.rs`.
3. The unit tests above, each run alone first
   (`heavy cargo nextest run -p jig-domain-core --lib -E 'test(/public::index::/)'`).
4. `cargo fmt`; `python3 -I -B jig/scripts/old_forms.py`;
   `heavy cargo clippy -p jig-domain-core --all-targets -- -D warnings`; the focused tests of
   `jig-domain-core`, `jig-test-domain` and `jig-core-world` (the hooks run in every
   decision: a change there that breaks another story is yours); the citations check.
5. Commit (why: the public index, kept by the decisions, which public reads examine alone);
   push your branch.

## Report

As the implementer's text says, and also: each hook's site, how a priority change is
followed, what the ended list does at start, and the index's memory at the dev.fabrig.ai
figures (`temper/docs/design/draft/first-release.status.md`).
