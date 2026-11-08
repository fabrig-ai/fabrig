+++
id = "busy-room"
part = "subscribe-room"
kind = "feature"
needs = ["batch-options", "goal-ci"]
files = ["temper/crates/temper-engine-smith/src/answers.rs", "temper/crates/temper-engine-smith/src/tests.rs", "temper/tests/engine/domain/tests/suite/forge_routes.rs", "temper/docs/design/domain/agent.status.md"]
effort = "medium"
minutes = 45
+++

## Goal

A run's `subscribe` refused because its task holds its most
subscriptions says so, naming the bound, where today it answers
`Refused: task 4 busy` and no more. A goal's coordinator read that as
its own task busy to itself (the request, item 2): its room of 4
(`tasks.subscriptions`) was full of the topics the core keeps for the
goal (its landings, and CI on each of its changes' heads), and the
rendering dropped the cause jig gave.

Acceptance:

- `answers.rs`, `task_capacity`: `CapacityCause::Subscriptions { limit }`
  renders `: it holds its most subscriptions (at most <limit>, counting
  the topics the engine keeps for a goal); unsubscribe one it made to
  make room`, so the whole answer reads `Refused: task 4 busy: it holds
  its most subscriptions (at most 4, counting the topics the engine
  keeps for a goal); unsubscribe one it made to make room`. Every other
  cause renders as today.
- Failing rows first:
  - `temper-engine-smith/src/tests.rs`: a test beside
    `a_second_proposal_is_refused_naming_the_one_pending`, in its shape,
    `a_subscription_refused_for_a_full_room_names_its_bound`:
    `engine::CallAnswer::SubscriptionRefused(busy_problem(
    tasks::CapacityAction::Subscription { number: 23 },
    tasks::CapacityCause::Subscriptions { limit: 4 }))` renders the
    words above for task 2, as an error. Before the fix it renders
    `Refused: task 2 busy`.
  - `forge_subscription_rooms_name_their_limits`
    (`temper/tests/engine/domain/tests/suite/forge_routes.rs`): rows in
    which, with `tasks.subscriptions` 1 and the room filled by a first
    `subscribe_forge` (as the `"task"` row does), the goal's own run
    calls `subscribe` with a timer, and with `kind: task` naming a live
    delegate of its own, each answered with `it holds its most
    subscriptions (at most 1`. Give `subscription_json` the tool's name
    as a parameter (its callers pass `b"subscribe_forge"`) and set the
    bound by the existing `subscription_goal_bound("task")`. For the
    task row, the goal delegates one task first through the world's
    existing helpers; if no helper in `forge_routes.rs` or its siblings
    makes a delegate cheaply, keep the timer row alone and say so.
  - Break the new arm once (render nothing for `Subscriptions` again)
    and see the rows fail.
- Close the entry: remove the **Not yet** 4.3 entry this plan added to
  `temper/docs/design/domain/agent.status.md` ("a run's `subscribe`
  refused for its task's full subscriptions answers `Refused: task N
  busy` and no more"), its lines alone. This departs from the
  implementer's text, which closes `docs/design/` to you: the entry is
  this task's, and the design it closes is written.

## Inputs

- `temper/docs/design/domain/agent.md`, 4.3, the bullet *A run's
  subscriptions take its task's room*.
- jig's `jig/docs/design/domain/tasks.md`, 8.4 and section 11 (the
  bound), and `jig/docs/design/domain/engine.md`, 7.3 (`subscribe`
  answered "refused, and why").
- `temper/docs/plan/goal-call-faults/notes/common.md`.

## Owns and may change

- Owns the files in `files`; in `answers.rs`, `task_capacity` alone; in
  `forge_routes.rs`, the table, `subscription_json` and its callers; in
  `agent.status.md`, the one entry above. You start after `goal-ci`,
  which stops the core subscribing a goal to its changes' CI
  (`forge.md`, 7.2), so a goal's room holds its landings topics alone;
  your rows fill it by the run's own `subscribe_forge`, which stays.
- Not yours: how many subscriptions a task may hold or what counts in
  them (jig's `subscriptions.rs`, the profiles, the store), which this
  plan leaves to the user; jig's own renderer
  (`jig/crates/jig-protocol-tools/src/render.rs`), which drops the same
  cause for jig's applications: report it under `## Left`.
- Pre-approved: if the timer row's call needs a `{"kind":"timer", ...}`
  shape other than the one `temper-engine-smith`'s decoder tests use,
  take theirs.

## Steps

1. The rows above; see them fail
   (`heavy cargo nextest run -p temper-engine-smith --lib -E
   'test(=tests::a_subscription_refused_for_a_full_room_names_its_bound)'`;
   `heavy cargo nextest run -p temper-engine-domain-world --test suite
   -E 'test(=forge_routes::forge_subscription_rooms_name_their_limits)'`;
   read the count, and find the exact path with `--list` if either
   selects nothing). Commit.
2. The arm; both pass; break it once, see them fail, restore.
3. The status entry closed.
4. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-engine-smith -p temper-engine-domain-world
   --all-targets -- -D warnings`; `heavy cargo nextest run -p
   temper-engine-smith`; `heavy cargo nextest run -p
   temper-engine-domain-world --test suite -E 'test(/forge_routes::/)'`;
   `heavy cargo check -p temper-service -p temper-system-world --tests`.
5. Commit and push.

## Report

As the implementer's text says, with the failure each new row showed
before the fix.
