//! A standing triager's named repair call makes one funded goal (domain/red-repairs.md, 4).
use super::*;
use forge_top::{self as forge, settings as s};
use skein_lib::Wall;
use temper_client_protocol_wire::v1::Record as CodecRecord;

fn finding(number: u32) -> forge::Finding {
    forge::Finding {
        project: 1,
        number,
        kind: forge::FindingKind::Red,
        severity: forge::Severity::High,
        area: None,
        state: forge::FindingState::New,
        title: format!("Broken {number}").into_bytes().into(),
        body: b"reproduce".as_slice().into(),
        filer: forge::Filer::Task(99),
        at: Wall::from_nanos(u64::from(number)),
        why: None,
        hander: None,
        goal: None,
        reason: None,
        seen: None,
        cleared: None,
        repair: None,
        signature: Some(forge::FindingSignature {
            repository: forge_world::REPO,
            source: b"tier-a".as_slice().into(),
            ids: Box::new([format!("test {number}").into_bytes().into()]),
            first: Box::new([]),
            many: false,
        }),
        blame: Some(forge::FindingBlame {
            strength: forge::BlameStrength::Unblamed,
            landings: Box::new([]),
            moved: None,
        }),
    }
}
fn landing() -> s::ProjectLanding {
    s::ProjectLanding {
        wait: temper_engine_domain_forge_change::Wait::Landed,
        repairs: Some(s::RepairSettings {
            goals: 2,
            attempts: 2,
            deadline_ms: 86_400_000,
            goal: s::RepairGoalSettings {
                charter: 1,
                priority: 7,
                budget: 20,
                llm: Some(s::NamedLlm {
                    account: None,
                    main: s::NamedModel { model: b"scripted".as_slice().into(), effort: None },
                    subagents: Box::new([]),
                    later: None,
                }),
            },
        }),
        ..s::ProjectLanding::default()
    }
}
fn world() -> (World, engine::Assignment) {
    world_with(true, false)
}
fn world_with(closer_kind: bool, prior: bool) -> (World, engine::Assignment) {
    let mut world = World::configured_limits(
        true,
        true,
        false,
        1000,
        Gate::default(),
        None,
        StoryLimits {
            kinds: true,
            goal_closer: true,
            tier: Some(tier_runs::Tier {
                wait: temper_engine_domain_forge_change::Wait::Landed,
                ..tier_runs::Tier::default()
            }),
            tasks: Some(12),
            slots: Some(3),
            spec_bytes: Some(8192),
            message_bytes: Some(1024),
            message_result_bytes: Some(512),
            message_inbox_bytes: Some(8192),
            inbox_messages: Some(16),
            inbox_recipients: Some(37),
            call_records: Some(5),
            party_requests: Some(16),
            ..StoryLimits::default()
        },
    );
    let mut landing = landing();
    if prior {
        landing.repairs.as_mut().expect("repair story fixture").goal.llm = None;
    }
    let setting = s::SettingRow {
        key: s::SettingKey { kind: 4, scope: 1, number: 1, slot: s::SettingSlot::Change },
        edit: s::Edit::Set(s::SizedValue {
            size: s::landing_record(&landing)
                .measure(&temper_client_protocol_wire::v1::CEILINGS)
                .expect("repair story fixture"),
            value: s::Value::Landing(landing),
        }),
    };
    let mut rows = vec![Record::Forge { row: Box::new(forge::Stored::Setting(setting)) }];
    rows.extend((1..=4).map(|number| {
        let mut row = finding(number);
        if prior && number == 1 {
            row.repair = Some(forge::FindingRepair {
                goal: 999,
                made: true,
                attempts: 1,
                cancelled: false,
                failing: Box::new([b"test 1".as_slice().into()]),
                until: None,
                judging: None,
            });
        }
        Record::Forge { row: Box::new(forge::Stored::Finding(row)) }
    }));
    world.store.seed(rows.into(), Box::new([]));
    world.adopt();
    let mut hello = protocol::hello_slots(Token::new(7), 3, Box::new([]));
    let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello: worker, .. }, .. }) =
        &mut hello
    else {
        panic!("worker hello")
    };
    worker.bounds.message_bytes = world.env.limits.tasks.message_bytes;
    world.send(hello);
    let people::Ask::Create { project, mut batch } = world.chat(1, b"Triage the project's findings") else {
        panic!("triager")
    };
    if !closer_kind {
        batch[0].authority.delegation.kinds = batch[0]
            .authority
            .delegation
            .kinds
            .iter()
            .copied()
            .filter(|kind| *kind != people::Executor::Procedure(6))
            .collect();
    }
    batch[0].label = 3;
    batch[0].flags.conversation = false;
    batch[0].flags.ended_by = people::EndedBy::Standing;
    batch[0].budget = people::Budget::AsAllowed { cap: Some(100) };
    let triager = assigned_chat(&mut world, 92, people::Ask::Create { project, batch });
    tiers::baseline(&mut world);
    save(&mut world, 15, &s::Value::ProjectTriager(s::ProjectTriager { owner: s::AreaOwner::Charter(1) }), 30);
    (world, triager)
}
fn save(world: &mut World, kind: u32, value: &s::Value, key: u8) {
    let items = Box::new([kinds::item(kind, 1, 1, value)]);
    let against = kinds::revision(&world.store.rows);
    world.send(engine::Event::Party(engine::PartyInput::ChangeSettings {
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: world.signed_in.expect("repair story fixture"),
        key: [key; 16],
        digest: [key; 32],
        kinds: kinds::decoded(items.as_ref()),
        change: Box::new(jig_domain_core::settings::Change {
            against,
            items,
            running: jig_domain_core::settings::Running::Stays,
        }),
    }));
    world.until(Until::Ready);
    assert_eq!(kinds::revision(&world.store.rows), against + 1, "kind {kind}: {:?}", world.web_replies.last());
}
fn call(run: &engine::Assignment, input: &str) -> engine::Call {
    let input = smith_domain_run::HostInput::attested(input.as_bytes().into()).expect("repair story fixture");
    let temper_engine_smith::Decoded::Call(call) = temper_engine_smith::call(
        smith_domain_run::CallName { activation: run.attempt, completion: 1, position: 1 },
        b"finding",
        &input,
    )
    .expect("repair story fixture") else {
        panic!("finding call")
    };
    call
}
fn made(answer: &engine::CallAnswer) -> u64 {
    let engine::CallAnswer::Finding(answer) = answer else { panic!("finding answer {answer:?}") };
    let engine::FindingToolAnswer::Write(reply) = answer.as_ref() else { panic!("finding write") };
    let Ok(forge::FindingDone::Repaired(goal)) = reply.result else { panic!("repair refused: {:?}", reply.result) };
    goal
}
fn row(world: &World, number: u32) -> &forge::Finding {
    findings(world).into_iter().find(|row| row.number == number).expect("repair story fixture")
}
fn settle(world: &mut World) {
    world.until(Until::Ready);
}

#[test]
fn a_named_repair_makes_one_funded_goal_and_routes_two_findings_in_its_kept_decision() {
    let (mut world, triager) = world();
    let parent = task(&world, triager.task).clone();
    let input = call(&triager, r#"{"act":"repair","numbers":[2,1],"words":"Fix both from their common cause."}"#);
    world.protocol.defer_render = true;
    let event = world.protocol.call(Token::new(7), triager.task, triager.attempt, 180, input.clone());
    world.send(event);
    let mut atomic = false;
    for _ in 0..LANDING_TICKS {
        world.tick();
        for (_, writes) in &world.store.pending {
            let findings = writes.iter().filter(|write| matches!(write, engine::Write::Save(Record::Forge { row }) if matches!(row.as_ref(), forge::Stored::Finding(row) if row.state == forge::FindingState::Routed))).count();
            if findings != 0 {
                assert_eq!(findings, 2);
                assert!(writes.iter().any(|write| matches!(write, engine::Write::Save(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row)))) if row.requester == tasks::Party::Task(triager.task) && row.tracked == Some(7))));
                assert!(writes.iter().any(|write| matches!(write, engine::Write::Save(Record::Forge { row }) if matches!(row.as_ref(), forge::Stored::Call { answer: forge::CallPayload::Finding(_), .. }))));
                atomic = true;
            }
        }
        if !world.protocol.pending_renders.is_empty() && row(&world, 1).goal.is_some() && world.store.pending.is_empty()
        {
            break;
        }
    }
    assert!(atomic, "making, both findings and keyed answer share a transaction");
    let answer = world.protocol.answers.last().expect("repair story fixture").1.clone();
    let goal = made(&answer);
    assert_eq!(
        temper_engine_smith::answer(&answer).text(),
        format!("Repair goal {goal} made for findings 1/2, 1/1.").as_bytes()
    );
    let goal_row = task(&world, goal);
    assert_eq!(goal_row.executor, tasks::Executor::Agent { charter: 1 });
    assert_eq!(goal_row.requester, tasks::Party::Task(triager.task));
    assert_eq!(goal_row.funder, tasks::Funder::Task(triager.task));
    assert_eq!(goal_row.numbers.budget, 20);
    assert_eq!(task(&world, triager.task).numbers.reserved, parent.numbers.reserved + 20);
    assert_eq!((goal_row.tracked, goal_row.label), (Some(7), 2));
    assert_eq!(goal_row.llm.as_ref().expect("repair story fixture").main.model, 0);
    assert_eq!(
        goal_row.flags,
        tasks::Flags {
            ended_by: tasks::EndedBy::Closer(tasks::Closer { connector: 0, code: 6 }),
            conversation: false,
            listed: true,
            retention: tasks::Retention::Kept
        }
    );
    assert_eq!(goal_row.contract, tasks::Contract::Report { words: world.env.limits.tasks.result_bytes });
    assert_eq!(goal_row.authority.delegation.depth, parent.authority.delegation.depth - 1);
    assert!(!goal_row.authority.delegation.kinds.iter().any(|kind| matches!(kind, tasks::AuthorityExecutor::Role(_))));
    assert!(
        goal_row.holdings.iter().any(|holding| match holding {
            tasks::Holding::Write { resource, .. } =>
                resource.connector == 0
                    && resource.path.last().is_some_and(|branch| branch.ends_with(format!("/g{goal}").as_bytes())),
            tasks::Holding::Slot { .. } => false,
        }),
        "the goal holds its own goal branch, not just an implicit run branch"
    );
    assert!(goal_row.spec.words.starts_with(
        b"Repair 2 tests failing in tier-a\n\nFix both from their common cause.\n\nRepository: org/repo. Tier: tier-a."
    ));
    assert!(
        goal_row
            .spec
            .words
            .windows(b"Finding 1/1: Broken 1\nAttempt 1 of 2.\n".len())
            .any(|bytes| bytes == b"Finding 1/1: Broken 1\nAttempt 1 of 2.\n")
    );
    assert_routed(&world, goal);
    world.drain_cut();
    world.protocol.pending_renders.clear();
    world.protocol.answers.clear();
    world.protocol.defer_render = false;
    world.restart_same();
    settle(&mut world);
    let before = task(&world, triager.task).delegates.clone();
    let event = world.protocol.call(Token::new(7), triager.task, triager.attempt, 180, input);
    world.send(event);
    settle(&mut world);
    assert_eq!(world.answers.last(), Some(&answer));
    assert_eq!(task(&world, triager.task).delegates, before, "key replay makes no second goal");
    world.close();
}

fn assert_routed(world: &World, goal: u64) {
    let routed = news(world, goal);
    assert_eq!(routed.len(), 1, "one ordinary coalesced findings-topic hint");
    assert_eq!(routed[0].hits, 2);
    let heard: Vec<_> = world
        .store
        .rows
        .values()
        .filter_map(|record| match record {
            Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Activity(tasks::ActivityLine {
                task,
                line: tasks::Line::Heard(message),
                ..
            }))) if *task == goal && message.words.starts_with(b"Finding ") => Some(message),
            _ => None,
        })
        .collect();
    assert_eq!(heard.len(), 2);
    let making_wall = heard[0].at;
    for (number, message) in [1, 2].into_iter().zip(heard) {
        assert!(message.words.starts_with(format!("Finding 1/{number} for you: Broken {number}\n").as_bytes()));
    }
    for number in [1, 2] {
        let repair = row(world, number).repair.as_ref().expect("repair story fixture");
        assert_eq!((row(world, number).state, row(world, number).goal), (forge::FindingState::Routed, Some(goal)));
        assert_eq!(
            (repair.goal, repair.made, repair.attempts, repair.cancelled, repair.judging),
            (goal, true, 1, false, None)
        );
        let deadline = repair.until.expect("repair story fixture").as_nanos();
        assert_eq!(deadline, making_wall.as_nanos() + Duration::from_millis(86_400_000).as_nanos());
    }
}

#[test]
fn an_owner_repairs_as_the_triagers_delegate_and_keeps_prior_failing_ids_until_judged() {
    let (mut world, triager) = world_with(true, true);
    let before = row(&world, 1).repair.as_ref().expect("repair story fixture").failing.clone();
    let project = 1;
    world.send(engine::Event::Party(engine::PartyInput::Application {
        reply_to: ReplyTo::new(Token::new(700)),
        sign_in: world.signed_in.expect("repair story fixture"),
        scope: people::ApplicationScope::Project(project),
        key: [70; 16],
        digest: [70; 32],
        carried: None,
        request: engine::Application::TriageFinding(forge::TriageFinding {
            project,
            number: 1,
            act: forge::Triage::Repair(forge::RepairAct {
                more: Box::new([]),
                words: b"A person approved this retry.".as_slice().into(),
            }),
        }),
    }));
    settle(&mut world);
    let Some(forge::TriageFindingAnswer::Repaired(goal)) = world.finding_triages.last() else {
        panic!("owner repair: {:?}", world.finding_triages)
    };
    let goal = *goal;
    assert_eq!(task(&world, goal).requester, tasks::Party::Task(triager.task));
    assert_eq!(task(&world, goal).funder, tasks::Funder::Task(triager.task));
    assert_eq!(task(&world, goal).llm, None, "none takes the charter's current default");
    let repair = row(&world, 1).repair.as_ref().expect("repair story fixture");
    assert_eq!((repair.goal, repair.attempts, repair.cancelled), (goal, 2, false));
    assert_eq!(repair.failing, before);
    assert!(
        task(&world, goal)
            .spec
            .words
            .windows(b"Attempt 2 of 2.\nStill failing after goal 999: 1 of its tests.\ntest 1\n".len())
            .any(|bytes| bytes == b"Attempt 2 of 2.\nStill failing after goal 999: 1 of its tests.\ntest 1\n")
    );
    world.close();
}

#[test]
fn repair_refuses_no_standing_triager_permission_funding_and_jigs_making_without_finding_writes() {
    let (mut world, triager) = world();
    let before = row(&world, 1).clone();
    save(&mut world, 15, &s::Value::ProjectTriager(s::ProjectTriager { owner: s::AreaOwner::Charter(4) }), 31);
    let answer =
        world.scripted_call(&triager, 180, call(&triager, r#"{"act":"repair","numbers":[1],"words":"fix"}"#).tool);
    assert_eq!(temper_engine_smith::answer(&answer).text(), b"no triager stands in project 1");
    assert_eq!(row(&world, 1), &before);
    save(&mut world, 15, &s::Value::ProjectTriager(s::ProjectTriager { owner: s::AreaOwner::Charter(1) }), 32);
    let ask = world.chat(1, b"Another agent");
    let other = assigned_chat(&mut world, 93, ask);
    let answer = world.scripted_call(&other, 181, call(&other, r#"{"act":"repair","numbers":[1],"words":"fix"}"#).tool);
    assert_eq!(
        temper_engine_smith::answer(&answer).text(),
        format!("finding 1/1 is not yours: its owner is task {}", triager.task).as_bytes()
    );
    assert_eq!(row(&world, 1), &before);
    let mut settings = landing();
    settings.repairs.as_mut().expect("repair story fixture").goal.budget = 101;
    save(&mut world, 4, &s::Value::Landing(settings), 33);
    world.restart_same();
    settle(&mut world);
    let numbers = task(&world, triager.task).numbers;
    let left = numbers.budget - numbers.spent - numbers.spent_below - numbers.reserved;
    let answer =
        world.scripted_call(&triager, 182, call(&triager, r#"{"act":"repair","numbers":[1],"words":"fix"}"#).tool);
    assert_eq!(
        temper_engine_smith::answer(&answer).text(),
        format!("the triager's period has {left} left, short of the repair goal's budget 101").as_bytes()
    );
    assert_eq!(row(&world, 1), &before);
    assert!(task(&world, triager.task).delegates.is_empty());
    world.close();
    let (mut world, triager) = world_with(false, false);
    let before = row(&world, 1).clone();
    let numbers = task(&world, triager.task).numbers;
    let answer =
        world.scripted_call(&triager, 180, call(&triager, r#"{"act":"repair","numbers":[1],"words":"fix"}"#).tool);
    assert_eq!(temper_engine_smith::answer(&answer).text(), b"Refused: executor");
    assert_eq!(row(&world, 1), &before);
    assert!(task(&world, triager.task).delegates.is_empty());
    assert_eq!(task(&world, triager.task).numbers.reserved, numbers.reserved);
    world.close();
}

#[test]
fn two_live_repair_goals_refuse_a_third_call_naming_their_distinct_numbers() {
    let (mut world, triager) = world();
    let first =
        world.scripted_call(&triager, 180, call(&triager, r#"{"act":"repair","numbers":[1],"words":"fix one"}"#).tool);
    let first = made(&first);
    let second =
        world.scripted_call(&triager, 181, call(&triager, r#"{"act":"repair","numbers":[2],"words":"fix two"}"#).tool);
    let second = made(&second);
    let before = row(&world, 3).clone();
    let answer = world.scripted_call(
        &triager,
        182,
        call(&triager, r#"{"act":"repair","numbers":[3],"words":"fix three"}"#).tool,
    );
    assert_eq!(
        temper_engine_smith::answer(&answer).text(),
        format!("project 1 has its 2 repair goals live: goal {first}, goal {second}").as_bytes()
    );
    assert_eq!(row(&world, 3), &before);
    assert_eq!(task(&world, triager.task).delegates.as_ref(), &[first, second]);
    world.close();
}
