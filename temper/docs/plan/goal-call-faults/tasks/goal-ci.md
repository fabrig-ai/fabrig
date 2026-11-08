+++
id = "goal-ci"
part = "subscribe-room"
kind = "feature"
needs = []
files = ["temper/crates/temper-engine-domain-forge/src/topics.rs", "temper/crates/temper-engine-domain-forge/src/domain.rs", "temper/crates/temper-engine-domain-forge/src/boundary.rs", "temper/crates/temper-engine-domain-forge/src/tests.rs", "temper/crates/temper-engine-domain/src/route/forge_route.rs", "temper/tests/engine/domain/tests/suite/forge_routes.rs", "temper/tests/engine/forge/**", "temper/tests/engine/protocol-forge/**", "temper/tests/real-loop/**", "temper/docs/design/domain/forge.status.md"]
effort = "high"
minutes = 110
+++

## Goal

The core stops subscribing a goal to CI on each of its live changes'
heads, which no design section asks for, and drops any such
subscription it finds at restart. Each one took one of the goal's
`tasks.subscriptions` (4 in the profiles), so a goal with a few changes
pushed had its room full of the core's topics, and its run's own
`subscribe` to a child or a timer was refused `Refused: task 4 busy`, its
own task busy to itself (the request, item 2). A goal's topics from the
core are then its landing branches' alone (`forge.md`, 7.2).

Acceptance:

- No `Request::GoalTopic` for a `Topic::Ci` is made any more:
  `topics.rs`' `subscribe` and `head_changed` make no goal CI
  subscription, and nothing a goal's CI subscriptions alone needed stays
  (their old-head `GoalUntopic` in `head_changed`, `goal_members`' `Ci`
  arm in `forge_route.rs`, a refresh only they wanted). What a change's
  own procedure reads, and a task's own CI subscription (a run's
  `subscribe_forge`), work as today: `subscribed_ci_and_unknown_change_facts_keep_their_wakes`
  (`temper/tests/engine/domain/tests/suite/forge_routes/hints.rs`) and
  the change tool's stories pass unchanged.
- At restart (`domain.rs`, `restored`), each held subscriber whose topic
  is `Topic::Ci` and whose `goal_tasks` is `Some` is dropped through the
  existing `Request::GoalUntopic { goal, topic }`, which removes the
  forge's row and the goal's jig row (`forge_route.rs`' `GoalUntopic`
  arm); a task's own CI subscriber (`goal_tasks` `None`) stays.
- Failing rows first, in existing tests:
  - `a_tracked_goals_changes_install_landing_and_current_head_ci_topics_without_agent_calls`
    (`temper/tests/engine/domain/tests/suite/forge_routes.rs`), renamed
    `a_tracked_goals_changes_install_their_landing_topic_alone_without_agent_calls`:
    after the producer's push, the goal's forge subscriptions hold its
    landings topic and no `Topic::Ci`, and the goal's jig row
    (`live_task(&world, goal.task)`'s subscriptions) holds one. Then the
    goal's run calls `subscribe` with a timer, with
    `world.env.limits.tasks.subscriptions` set to 2 before the push, and
    is answered subscribed (today: refused busy, its landings and its
    change's CI filling the 2).
  - `goals_classify_landings_from_their_subtree_heads_and_planned_paths_after_restart`
    (`temper/tests/engine/forge/tests/suite/simulation.rs`): its
    `GoalTopic` assertion becomes that no `GoalTopic` is seen; and a row
    beside it: a goal subscriber to `Topic::Ci { repository: REPO, head }`
    with `goal_tasks` `Some`, made by `top::Event::Subscribe` before
    `world.restart()`, is answered after the restart by a `GoalUntopic`
    for it, while the same topic held by a task with `goal_tasks` `None`
    is not.
  - Each fails before the change for the fault's reason; break the
    restart drop once and see its row fail.
- The landings' classification for goals (7.3), red tier news under the
  landings subscription, and every other test of the touched packages
  pass. The real loop's fixtures whose message numbers or `forge ci`
  cue counts shift because the goal makes fewer subscriptions
  (`temper/tests/real-loop/src/case.rs`' numbers, `case/scripts.rs`,
  `oracle.rs`) are updated to what the stories now see, each named in
  the report with the reason.
- Close the entry: remove the **Not yet** 7.2 item this plan added to
  `temper/docs/design/domain/forge.status.md` ("the core still subscribes
  a goal to CI on each of its live changes' heads"), its lines alone.
  This departs from the implementer's text, which closes `docs/design/`
  to you: the entry is this task's, and the design it closes is written.

## Inputs

- `temper/docs/design/domain/forge.md`, 7.1 (the topics' table: CI on
  a head is read by changes and tasks that subscribe), 7.2 (the bullet
  *Those are a goal's only topics from the core*) and 7.3.
- `temper/docs/design/domain/agent.md`, 4.3, the bullet *A run's
  subscriptions take its task's room*.
- jig's `jig/docs/design/domain/tasks.md`, 8.4 (the per-task bound) and
  8.2 (the merged inbox, unchanged here).
- `temper/docs/design/domain/testing.md`, the table's row 5.4 (A hears
  no CI news of its changes' heads).
- `temper/docs/plan/goal-call-faults/notes/common.md`.

## Owns and may change

- Owns the files in `files`. In `forge_route.rs`, `subscribe_goal`,
  `goal_members` and the `GoalTopic`/`GoalUntopic` arms alone; in
  `forge_routes.rs`, the test above alone (task `busy-room` follows you
  in the same file's subscription-room table).
- Pre-approved: `Request::GoalTopic` goes when nothing makes it, with its
  arms in every matching file (`temper-engine-domain-forge/src/tests.rs`,
  `temper/tests/engine/forge/src/world.rs`, its `simulation.rs`, and
  `temper/tests/engine/protocol-forge/src/protocol.rs` and
  `tests/suite/protocol.rs`); if a landings path still makes it, keep it
  for that and say so. `subscribe_goal` keeps its signature.
- No stored format changes: `Subscriber`'s encoding, the store's keys
  and jig's rows stay as they are; only which rows are written changes.
- Not yours: jig (its bound and its inbox), the profiles' figures,
  `temper-engine-smith` (task `batch-options` and then `busy-room`).
- If a test outside `files` asserts a goal's CI topic, it is a reader:
  change it to the landings-only behaviour and name it; if it is a slow
  tier's (a `process`, `live` or browser binary), change it and compile
  it alone (`heavy cargo check -p PKG --tests`).

## Steps

1. The failing rows; see them fail
   (`heavy cargo nextest run -p temper-engine-domain-world --test suite
   -E 'test(=forge_routes::a_tracked_goals_changes_install_their_landing_topic_alone_without_agent_calls)'`;
   `heavy cargo nextest run -p temper-engine-forge-world --test suite -E
   'test(=simulation::goals_classify_landings_from_their_subtree_heads_and_planned_paths_after_restart)'`,
   and the new row's name; read the count, and find the exact path with
   `--list` if one selects nothing). Commit.
2. `topics.rs`, `domain.rs` (the drop at `restored`), `forge_route.rs`,
   `boundary.rs`; the rows pass; break the drop once, see its row fail,
   restore.
3. The real loop: `heavy cargo nextest run -p temper-real-loop-world`,
   then `--profile fuzzy` on the same package; read each failure's log,
   and update what the fewer subscriptions shift, as above.
4. The status entry closed.
5. `cargo fmt`; `python3 -I -B temper/scripts/old_forms.py`; `heavy cargo
   clippy -p temper-engine-domain-forge -p temper-engine-domain -p
   temper-engine-domain-world -p temper-engine-forge-world -p
   temper-engine-protocol-forge-world -p temper-real-loop-world
   --all-targets -- -D warnings`; `heavy cargo nextest run -p
   temper-engine-domain-forge -p temper-engine-domain -p
   temper-engine-forge-world -p temper-engine-protocol-forge-world`;
   `heavy cargo nextest run -p temper-engine-domain-world`; `heavy cargo
   check -p temper-service -p temper -p temper-system-world -p
   temper-client-world --tests`.
6. Commit and push.

## Report

As the implementer's text says, with the failure each new row showed
before the fix, every test whose expectation changed and why, and what
a goal's inbox no longer hears.
