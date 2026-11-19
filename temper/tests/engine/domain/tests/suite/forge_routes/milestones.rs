//! The forge keeps milestone rows, never milestone issues (domain/milestones.md, 8).
use super::goal_landing::{ended, finish_goal, mine, row};
use super::*;
const STEPS: u32 = 1000;

fn ask(world: &mut World, key: u8, ask: people::Ask) -> people::Outcome {
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: world.signed_in.expect("owner"),
        key: [key; 16],
        ask,
    }));
    world.until(Until::Ready);
    world
        .store
        .rows
        .values()
        .find_map(|record| match record {
            Record::Core(jig_domain_core::Record::People(people::Stored::Answer { key: request, outcome, .. }))
                if request.key == [key; 16] =>
            {
                Some(outcome.clone())
            }
            _ => None,
        })
        .expect("party request answered")
}
fn created(outcome: &people::Outcome) -> u64 {
    let people::Outcome::Created { first, last, .. } = outcome else { panic!("created: {outcome:?}") };
    assert_eq!(first, last);
    *first
}
fn milestone(world: &mut World, words: &[u8]) -> u64 {
    let people::Ask::Create { project, mut batch } = world.chat(1, words) else { panic!("creation") };
    let member = &mut batch[0];
    member.executor =
        people::TaskExecutor::Procedure { connector: jig_domain_core::RECURRING, code: jig_domain_core::GATHERING };
    member.contract = people::Contract::Report { words: 128 };
    member.flags = people::Flags { listed: true, ended_by: people::EndedBy::Requester, ..people::Flags::PLAIN };
    member.tracked = Some(1);
    member.label = 4;
    member.authority.delegation.depth = 3;
    member.budget = people::Budget::AsAllowed { cap: Some(400) };
    created(&ask(world, 180, people::Ask::Create { project, batch }))
}
fn stored(world: &World, task: u64) -> &forge_top::MilestoneRow {
    let Some(Record::Forge { row }) = world.store.rows.get(&Key::Forge(forge_top::Key::Milestone(task))) else {
        panic!("milestone {task} stored")
    };
    let forge_top::Stored::Milestone(row) = row.as_ref() else { panic!("milestone row") };
    row
}
fn no_issue(world: &World, task: u64) {
    assert!(!world.store.rows.contains_key(&Key::Forge(forge_top::Key::Issue(task))), "milestones have no issues");
    assert!(!world.store.rows.values().any(|record| matches!(record, Record::Forge { row } if matches!(row.as_ref(), forge_top::Stored::Subscription(subscription) if subscription.task == task))), "milestones have no topics");
}
fn goal(world: &mut World, milestone: u64, own_branch: bool) -> engine::Assignment {
    let people::Ask::Create { project, mut batch } = goal_branch::goal_request(world, true) else { panic!("goal") };
    if !own_branch {
        batch[0].symbols = Box::new([]);
    }
    batch[0].authority.delegation.depth = 2;
    batch[0].authority.delegation.tasks = 10;
    batch[0].budget = people::Budget::AsAllowed { cap: Some(300) };
    let task = created(&ask(world, 181, people::Ask::Delegate { project, task: milestone, batch }));
    world.assigned.iter().find(|run| run.task == task).expect("goal assigned").clone()
}
fn world() -> World {
    let mut world = World::configured_limits(
        true,
        true,
        true,
        STEPS,
        Gate::default(),
        None,
        StoryLimits { slots: Some(2), spec_bytes: Some(1024), ..StoryLimits::default() },
    );
    world.adopt();
    world.send(protocol::hello_slots(Token::new(7), 2, Box::new([])));
    world
}
#[test]
fn a_milestones_dependent_goal_starts_when_the_first_ends_at_its_landing() {
    use temper_engine_domain_world::tier_runs::{self, LANDING_TICKS, Tier, task, tier, until};
    let mut world = World::configured_limits(
        true,
        true,
        false,
        STEPS,
        Gate::default(),
        None,
        StoryLimits {
            tier: Some(Tier { wait: temper_engine_domain_forge_change::Wait::Landed, ..Tier::default() }),
            slots: Some(4),
            tasks: Some(32),
            tree_tasks: Some(64),
            task_delegates: Some(16),
            authority_batch: Some(8),
            tasks_batch: Some(8),
            result_bytes: Some(1024),
            ..StoryLimits::default()
        },
    )
    .with_goal_closer();
    world.env.limits.tasks.project_goals = 3; // The milestone and its two goals coexist.
    let client = world.env.limits.forge.client;
    let mut world = world.with_forge_client(client);
    world.adopt();
    world.send(protocol::hello_slots(Token::new(7), 4, Box::new([])));
    let milestone = milestone(&mut world, b"Two goals");
    let people::Ask::Create { project, mut batch } = goal_branch::goal_request(&world, true) else { panic!("goal") };
    batch[0].symbols = Box::new([]);
    batch[0].flags.ended_by = people::EndedBy::Closer(people::Closer { connector: 0, code: 6 });
    batch[0].authority.delegation.depth = 2;
    batch[0].authority.delegation.tasks = 16;
    batch[0].budget = people::Budget::AsAllowed { cap: Some(200) };
    let first =
        created(&ask(&mut world, 181, people::Ask::Delegate { project, task: milestone, batch: batch.clone() }));
    batch[0].dependencies = Box::new([people::Dependency::Existing(first)]);
    batch[0].words = b"Wait for the first goal".as_slice().into();
    let second = created(&ask(&mut world, 182, people::Ask::Delegate { project, task: milestone, batch }));
    assert_eq!(task(&world, first).requester, tasks::Party::Task(milestone));
    assert_eq!(task(&world, second).requester, tasks::Party::Task(milestone));
    assert_eq!(task(&world, second).dependencies.as_ref(), &[first]);
    assert!(!world.assigned.iter().any(|run| run.task == second));
    let goal = world.assigned.iter().find(|run| run.task == first).unwrap().clone();
    let change = tiers::produce(&mut world, &goal, 93, b"file", b"first goal's work");
    let event =
        world.protocol.answer(Token::new(7), goal.task, goal.attempt, 0, tasks::End::Parked, engine::Parts::none());
    world.send(event);
    until(&mut world, LANDING_TICKS, "dependent starts at the first goal's landing", |world| {
        world.assigned.iter().any(|run| run.task == second)
    });
    assert!(matches!(task(&world, first).phase, tasks::Phase::Ended(tasks::Ending::Done(_))));
    assert!(matches!(task(&world, change).phase, tasks::Phase::Ended(tasks::Ending::Done(_))));
    assert!(tiers::result(&world, change).contains("Ended at landing; after landing judges it later."));
    assert_eq!(tier(&world).record.sequence, 0, "no tier judgement releases the dependent");
    assert_eq!(tier_runs::red_news(&world, second).len(), 0);
}

#[test]
fn a_milestones_first_projection_keeps_its_project_and_utf8_name_without_an_issue() {
    for (words, name) in [
        (b"Store shape\n\nIts words".to_vec(), b"Store shape".to_vec()),
        (format!("{}€\nWords", "é".repeat(127)).into_bytes(), "é".repeat(127).into_bytes()),
    ] {
        let mut world = world();
        let milestone = milestone(&mut world, &words);
        assert_eq!(
            stored(&world, milestone),
            &forge_top::MilestoneRow {
                project: 1,
                task: milestone,
                name: name.into_boxed_slice(),
                last_landing: None,
                ended: None
            }
        );
        no_issue(&world, milestone);
        assert!(world.writes.is_empty(), "no milestone issue is made or updated");
        world.close();
    }
}
fn land(world: &mut World, goal: &engine::Assignment, call: u64, landing: Option<&str>) -> u64 {
    let before = world.assigned.len();
    let input = match landing {
        Some(landing) => format!("{{\"words\":\"Small fix\",\"landing\":\"{landing}\"}}"),
        None => r#"{"words":"Small fix"}"#.into(),
    };
    let answer = mine(world, goal, call, input.as_bytes());
    let Some(engine::CallAnswer::Delegated(made)) = answer else {
        panic!(
            "change {call}: {answer:?}; errors={:?}; replies={:?}; goal={:?}",
            world.call_errors,
            world.answers,
            live_task(world, goal.task).phase
        )
    };
    let task = made[0].task;
    let producer = reviews::next_assignment(world, before);
    let base = goal_branch::branch_tip(world, &row(world, task).base);
    let head = early_start::push(world, &producer, base, format!("file-{call}").as_bytes());
    goal_landing::status(world, head, raw::Check::Passed);
    early_start::finish(world, &producer, b"Small fix");
    await_landed(world, task);
    task
}
fn await_landed(world: &mut World, task: u64) {
    for _ in 0..STEPS {
        world.tick();
        if ended(world, task).is_some() {
            break;
        }
    }
    assert!(
        matches!(row(world, task).change.state, temper_engine_domain_forge_change::State::Landed { .. }),
        "{:?}",
        row(world, task)
    );
    world.until(Until::Ready);
}
#[test]
fn a_milestones_goal_lands_its_changes_on_its_tree_branch_then_lands_it_whole() {
    let mut world = world();
    let milestone = milestone(&mut world, b"Store shape");
    let main = goal_branch::branch_tip(&mut world, b"main");
    let goal = goal(&mut world, milestone, true);
    let branch = format!("temper/{milestone}/g{}", goal.task).into_bytes();
    let old_branch = format!("temper/{}/g{}", goal.task, goal.task).into_bytes();
    assert_eq!(goal_branch::forge_hold(&world, &branch).expect("goal holds tree branch").task, goal.task);
    assert!(goal_branch::forge_hold(&world, &old_branch).is_none());
    assert_eq!(goal_branch::branch_tip(&mut world, &branch), main);
    assert_eq!(stored(&world, milestone).last_landing, None);

    let first = land(&mut world, &goal, 101, None);
    assert_eq!(row(&world, first).base.as_ref(), branch);
    let first_tip = goal_branch::branch_tip(&mut world, &branch);
    assert_ne!(first_tip, main);
    assert_eq!(goal_branch::branch_tip(&mut world, b"main"), main);
    assert_eq!(stored(&world, milestone).last_landing, None);

    let second = land(&mut world, &goal, 102, Some("goal"));
    assert_eq!(row(&world, second).base.as_ref(), branch);
    let second_tip = goal_branch::branch_tip(&mut world, &branch);
    assert_ne!(second_tip, first_tip);
    assert_eq!(goal_branch::branch_tip(&mut world, b"main"), main);
    assert_eq!(stored(&world, milestone).last_landing, None);

    let assigned = world.assigned.len();
    let answer = mine(&mut world, &goal, 103, br#"{"words":"Land the goal","branch":"mine"}"#);
    let Some(engine::CallAnswer::Delegated(made)) = answer else { panic!("goal landing: {answer:?}") };
    let landing = made[0].task;
    assert_eq!(row(&world, landing).branch.as_ref(), branch);
    assert_eq!(row(&world, landing).base.as_ref(), b"main");
    assert_eq!(world.assigned.len(), assigned, "landing the whole branch needs no producer");
    goal_landing::status(&mut world, second_tip, raw::Check::Passed);
    await_landed(&mut world, landing);
    let landed = goal_branch::branch_tip(&mut world, b"main");
    assert_ne!(landed, main);
    for path in [b"file-101", b"file-102"] {
        assert_eq!(
            world.external(1, raw::Op::Read(raw::Read::File { commit: landed, path: path.as_slice().into() })),
            raw::Answer::File(b"added".as_slice().into()),
        );
    }
    let temper_engine_domain_forge_change::State::Landed { since, .. } = row(&world, landing).change.state else {
        panic!("whole goal landed")
    };
    assert_eq!(
        stored(&world, milestone).last_landing,
        Some(forge_top::MilestoneLanding { at: since, goal: goal.task, change: landing }),
    );
    ask(&mut world, 182, people::Ask::Conclude { project: 1, task: milestone });
    finish_goal(&mut world, &goal);
    world.close();
}

#[test]
fn the_changes_subject_enriches_landed_rows_after_the_forge_makes_them() {
    let mut world = world();
    let milestone = milestone(&mut world, b"Store shape");
    let goal = goal(&mut world, milestone, false);
    let change = land(&mut world, &goal, 101, Some("main"));
    let session = world.signed_in.expect("owner");
    // The first read starts any ended-row load; a fresh watch sees its answer.
    let _ = world.oracle_document(session, 1, forge_top::changes_subject(1));
    let Some(forge_top::Document::Changes(view)) = world.oracle_document(session, 1, forge_top::changes_subject(1))
    else {
        panic!("changes subject")
    };
    for rows in [&view.landed, &view.landed_default] {
        let row = rows.iter().find(|row| row.task == change).expect("landed change");
        assert_eq!(row.goal.as_ref().map(|from| from.task), Some(goal.task));
        assert_eq!(
            row.goal.as_ref().unwrap().title.as_ref(),
            live_task(&world, goal.task).spec.words.split(|byte| *byte == b'\n').next().unwrap()
        );
        assert_eq!(
            row.milestone,
            Some(forge_top::ChangeFrom { task: milestone, title: b"Store shape".as_slice().into() })
        );
        assert_eq!(row.chat, None);
    }
    ask(&mut world, 182, people::Ask::Conclude { project: 1, task: milestone });
    finish_goal(&mut world, &goal);
    world.close();
}

#[test]
fn only_a_goals_default_branch_landings_replace_its_milestones_landing_across_restart() {
    let mut world = world();
    let milestone = milestone(&mut world, b"Store shape");
    let goal = goal(&mut world, milestone, false);
    assert!(world.store.rows.contains_key(&Key::Forge(forge_top::Key::Issue(goal.task))), "the goal has its issue");
    let first = land(&mut world, &goal, 101, Some("main"));
    assert_eq!((row(&world, first).goal, row(&world, first).milestone), (Some(goal.task), Some(milestone)));
    let landing = stored(&world, milestone).last_landing.clone().expect("first landing");
    let temper_engine_domain_forge_change::State::Landed { since, .. } = row(&world, first).change.state else {
        panic!("landed")
    };
    assert_eq!(landing, forge_top::MilestoneLanding { at: since, goal: goal.task, change: first });
    let keys = [
        Key::Forge(forge_top::Key::Milestone(milestone)),
        Key::Forge(forge_top::Key::ChangeCold(first)),
        Key::Forge(forge_top::Key::ChangeHot(first)),
        Key::Forge(forge_top::Key::ProjectFacts { project: 1 }),
    ];
    let rows = keys.each_ref().map(|key| world.store.rows.get(key).expect("coexisting restart rows").clone());
    restart(&mut world);
    world.until(Until::Ready);
    assert!(world.root.ready(), "the mixed-family startup walk completes");
    for (key, expected) in keys.iter().zip(&rows) {
        assert_eq!(world.store.rows.get(key), Some(expected), "restart keeps {key:?}");
    }
    let second = land(&mut world, &goal, 102, Some("main"));
    let later = stored(&world, milestone).last_landing.clone().expect("later landing");
    assert!(later.at > landing.at);
    assert_eq!((later.goal, later.change), (goal.task, second));
    let tip = goal_branch::branch_tip(&mut world, b"main");
    assert_eq!(
        world.external(1, raw::Op::Git(raw::Git::Create { branch: b"side".as_slice().into(), commit: tip })),
        raw::Answer::Branch(raw::Created::Created)
    );
    let other = land(&mut world, &goal, 103, Some("side"));
    assert_eq!(row(&world, other).milestone, Some(milestone));
    assert_eq!(stored(&world, milestone).last_landing, Some(later));
    ask(&mut world, 182, people::Ask::Conclude { project: 1, task: milestone });
    finish_goal(&mut world, &goal);
    no_issue(&world, milestone);
    world.close();
}
fn restart(world: &mut World) {
    for _ in 0..STEPS {
        if world.pending.is_empty() && world.events.is_empty() && world.store.pending.is_empty() {
            break;
        }
        world.tick();
    }
    world.restart_same();
}
fn settled(world: &World, task: u64) {
    use jig_domain_core::{CoreKey, Key as CoreKeyKind, ProjectionKey};
    assert!(
        !world.store.rows.contains_key(&Key::Core(CoreKeyKind::Core(CoreKey::Projection(ProjectionKey::Goal(task))))),
        "milestone projection settled"
    );
}
#[test]
fn a_milestones_end_keeps_its_counts_outcome_spend_and_wall_without_an_issue() {
    for outcome in
        [forge_top::MilestoneOutcome::Complete, forge_top::MilestoneOutcome::Ended, forge_top::MilestoneOutcome::Closed]
    {
        let mut world = world();
        let milestone = milestone(&mut world, b"Store shape");
        let goal = goal(&mut world, milestone, false);
        assert!(matches!(
            ask(&mut world, 182, people::Ask::Conclude { project: 1, task: milestone }),
            people::Outcome::Concluded { .. }
        ));
        assert!(stored(&world, milestone).ended.is_none(), "concluded waits for its last goal");
        match outcome {
            forge_top::MilestoneOutcome::Complete => finish_goal(&mut world, &goal),
            forge_top::MilestoneOutcome::Ended => goal_landing::finish_run(
                &mut world,
                &goal,
                tasks::TaskResult::Failure { reason: b"cannot finish".as_slice().into() },
            ),
            forge_top::MilestoneOutcome::Closed => {
                ask(
                    &mut world,
                    183,
                    people::Ask::Cancel {
                        project: 1,
                        task: milestone,
                        reason: b"close".as_slice().into(),
                        grace: None,
                    },
                );
                goal_landing::finish_run(
                    &mut world,
                    &goal,
                    tasks::TaskResult::Report { words: b"stopped".as_slice().into() },
                );
            }
        }
        world.until(Until::Ready);
        let task = ended(&world, milestone).expect("milestone ended");
        let end = stored(&world, milestone).ended.as_ref().expect("forge end");
        assert_eq!(
            end,
            &forge_top::MilestoneEnd {
                at: task.ended_at.expect("end wall"),
                outcome,
                done: u32::from(outcome == forge_top::MilestoneOutcome::Complete),
                failed: u32::from(outcome == forge_top::MilestoneOutcome::Ended),
                closed: u32::from(outcome == forge_top::MilestoneOutcome::Closed),
                spent: task.numbers.spent + task.numbers.spent_below
            }
        );
        assert!(end.spent > 0, "settled goal spend is retained");
        settled(&world, milestone);
        no_issue(&world, milestone);
        let retained = stored(&world, milestone).clone();
        restart(&mut world);
        world.until(Until::Ready);
        assert_eq!(stored(&world, milestone), &retained);
        settled(&world, milestone);
        world.close();
    }
}
