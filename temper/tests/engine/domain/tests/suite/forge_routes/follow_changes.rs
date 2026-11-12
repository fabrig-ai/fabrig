//! The standing follow and its target-project work, on the independent fake forge.
#[path = "follow_changes/proof.rs"]
pub(crate) mod proof;
#[path = "follow_changes/role.rs"]
pub(crate) mod role;
use super::*;
use forge_top::{FollowRow, FollowState, settings as s};
use jig_domain_core::settings as cs;
const SOURCE: skein_forge::api::Repository = skein_forge::api::Repository { forge: 1, repository: 3 };
const TARGET: skein_forge::api::Repository = skein_forge::api::Repository { forge: 1, repository: 4 };
const TICKS: u32 = 200;
fn roles(world: &mut World, session: u64, key: u8, holdings: Box<[people::Holding]>) {
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: session,
        key: [key; 16],
        ask: people::Ask::SetRoles { project: 2, holdings: holdings.clone() },
    }));
    settle(world);
    assert_eq!(
        world.store.rows.get(&Key::Core(jig_domain_core::Key::People(people::Key::Roles(2)))),
        Some(&Record::Core(jig_domain_core::Record::People(people::Stored::Roles { project: 2, holdings })))
    );
}
#[test]
fn follow_owner_and_project_owner_can_act_but_an_unrelated_member_cannot() {
    let mut world = world_with_gate(s::FollowStart::Owner, 1000, Gate { owner: true, ..Gate::default() });
    let a = push(&mut world, b"Fix handoff");
    let owner = world.signed_in.take().unwrap();
    world.send(engine::Event::Party(engine::PartyInput::SignedIn {
        held: None,
        reply_to: ReplyTo::new(Token::new(170)),
        identity: people::Identity {
            key: forge_top::user_key(1, 8),
            login: b"other".as_slice().into(),
            name: b"Other person".as_slice().into(),
        },
    }));
    world.until(Until::SignedIn);
    let other = world.signed_in.unwrap();
    let Some(Record::Core(jig_domain_core::Record::People(people::Stored::SignIn { person, .. }))) =
        world.store.rows.get(&Key::Core(jig_domain_core::Key::People(people::Key::SignIn(other))))
    else {
        panic!("other session")
    };
    let person = *person;
    roles(
        &mut world,
        owner,
        171,
        Box::new([
            people::Holding { person: 1, role: people::Role::Member },
            people::Holding { person, role: people::Role::Owner },
        ]),
    );
    assert_eq!(
        act(&mut world, 172, FollowAct::Hold { after: a, reason: b"project owner waits".as_slice().into() }),
        None
    );
    world.signed_in = Some(owner);
    assert_eq!(
        act(&mut world, 173, FollowAct::Hold { after: a, reason: b"follow owner waits".as_slice().into() }),
        None
    );
    roles(
        &mut world,
        other,
        174,
        Box::new([
            people::Holding { person: 1, role: people::Role::Owner },
            people::Holding { person, role: people::Role::Member },
        ]),
    );
    world.signed_in = Some(other);
    let reason = act(&mut world, 175, FollowAct::Skip { after: a, reason: b"not mine".as_slice().into() }).unwrap();
    assert!(
        reason.starts_with(&forge_top::commit_text(&a))
            && reason.ends_with(b"only the follow's owner or an owner of its project may act")
    );
    assert_eq!(row(&world).queue[0].state, FollowState::HeldByOwner(b"follow owner waits".as_slice().into()));
    world.close();
}
use forge_top::follows::FollowAct;
fn act(world: &mut World, key: u8, act: FollowAct) -> Option<Box<[u8]>> {
    let before = world.follow_answers.len();
    world.send(engine::Event::Party(engine::PartyInput::Application {
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: world.signed_in.expect("signed in"),
        key: [key; 16],
        digest: [key; 32],
        scope: people::ApplicationScope::Project(2),
        carried: None,
        request: engine::Application::Follow { project: 2, follow: 1, act },
    }));
    settle(world);
    assert_eq!(world.follow_answers.len(), before + 1, "act {key} answered; replies {:?}", world.web_replies);
    world.follow_answers.last().expect("follow act answered").clone()
}
fn no_holds(world: &World, changes: &[u64]) {
    for change in changes {
        for record in world.store.rows.values() {
            if let Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row) | tasks::Stored::Ended(row))) =
                record
                && row.number == *change
            {
                assert!(!matches!(row.phase, tasks::Phase::Held { .. }), "change {change} held: {:?}", row.phase);
                assert_eq!(row.hold, 0, "change {change} acquired a hold");
            }
        }
    }
}
#[test]
fn owner_acts_keep_the_chain_whole_and_merge_the_briefs() {
    let mut world = world(s::FollowStart::Owner);
    let a = push(&mut world, b"First fix");
    let b = push(&mut world, b"Second fix");
    let c = push(&mut world, b"Third fix");
    let d = push(&mut world, b"Fourth fix");
    let changes = [change(&world, a), change(&world, b), change(&world, c), change(&world, d)];
    assert_eq!(
        act(&mut world, 180, FollowAct::Hold { after: a, reason: b"waits for the handoff fix".as_slice().into() }),
        None
    );
    assert_eq!(row(&world).queue[0].state, FollowState::HeldByOwner(b"waits for the handoff fix".as_slice().into()));
    assert!(questions::row(&world, changes[0]).delegate.is_none());
    no_holds(&world, &changes);
    assert_eq!(act(&mut world, 181, FollowAct::Merge { afters: Box::new([b, c]) }), None);
    assert_eq!(change(&world, b), changes[1]);
    assert_eq!(
        change(&world, c),
        changes[1],
        "follow {:?}; task {:?}",
        row(&world),
        live_task(&world, row(&world).task.unwrap())
    );
    assert_eq!(live_task(&world, changes[3]).waiting_on.as_ref(), &[changes[1]]);
    no_holds(&world, &changes);
    let spec = &live_task(&world, changes[1]).spec;
    let b_text = String::from_utf8(forge_top::commit_text(&b).into_vec()).unwrap();
    let c_text = String::from_utf8(forge_top::commit_text(&c).into_vec()).unwrap();
    assert!(spec.parameters.iter().any(|p| matches!(p, tasks::Parameter::Bytes { name: 18, value } if value.as_ref() == format!("1:{b_text},{c_text}").as_bytes())));
    let words = std::str::from_utf8(&spec.words).unwrap();
    assert!(words.starts_with(&format!("Follow org/vsr {} and 1 more: Third fix", &c_text[..12])), "{words}");
    assert!(
        words.find(&format!("Port landing {b_text}")).unwrap() < words.find(&format!("Port landing {c_text}")).unwrap()
    );
    for _ in 0..TICKS {
        if world.store.rows.contains_key(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(changes[2])))) {
            break;
        }
        world.tick();
    }
    assert!(
        matches!(&ended_task(&world, changes[2]).phase, tasks::Phase::Ended(tasks::Ending::Cancelled { reason, .. }) if reason.as_ref() == format!("merged into change {}", changes[1]).as_bytes())
    );
    assert_eq!(questions::row(&world, changes[1]).follows.as_ref().unwrap().commits.as_ref(), &[b, c]);
    assert_eq!(act(&mut world, 182, FollowAct::Skip { after: d, reason: b"build only".as_slice().into() }), None);
    assert_eq!(row(&world).skips.last().unwrap().after, d);
    assert_eq!(row(&world).skips.last().unwrap().reason.as_ref(), b"build only");
    assert!(!row(&world).queue.iter().any(|e| e.after == d));
    no_holds(&world, &changes);
    // Skipping the head rewires the surviving merged change to no dependency.
    assert_eq!(act(&mut world, 183, FollowAct::Skip { after: a, reason: b"already ported".as_slice().into() }), None);
    assert!(live_task(&world, changes[1]).waiting_on.is_empty());
    no_holds(&world, &changes);
    world.drain_cut();
    world.restart_same();
    world.until(Until::Ready);
    let count = world.follow_answers.len();
    assert_eq!(act(&mut world, 183, FollowAct::Skip { after: a, reason: b"already ported".as_slice().into() }), None);
    assert_eq!(world.follow_answers.len(), count + 1, "the same retained answer is delivered once per request");
    assert_eq!(row(&world).skips.len(), 2, "the repeated key never applies the skip twice");
    world.close();
}
#[test]
fn owner_hold_is_lifted_by_start_and_started_changes_refuse_hold_and_merge() {
    let mut world = world(s::FollowStart::Owner);
    let a = push(&mut world, b"Fix handoff");
    let b = push(&mut world, b"Fix build");
    let first = change(&world, a);
    assert_eq!(
        act(&mut world, 180, FollowAct::Hold { after: a, reason: b"waits for the handoff fix".as_slice().into() }),
        None
    );
    assert!(questions::row(&world, first).delegate.is_none());
    assert_eq!(act(&mut world, 181, FollowAct::Start { after: a }), None);
    next_delegate(&mut world, first, 0);
    for (key, request) in [
        (182, FollowAct::Hold { after: a, reason: b"too late".as_slice().into() }),
        (183, FollowAct::Merge { afters: Box::new([a, b]) }),
    ] {
        let reason = act(&mut world, key, request).expect("started entry refused");
        assert!(
            reason.starts_with(&forge_top::commit_text(&a)) && reason.ends_with(b"the change has started producing"),
            "{}",
            String::from_utf8_lossy(&reason)
        );
    }
    world.close();
}
#[test]
fn follow_acts_refuse_unknown_skipped_nonconsecutive_and_over_bound_entries() {
    let mut world = world(s::FollowStart::Owner);
    let a = push(&mut world, b"A");
    let b = push(&mut world, b"B");
    let c = push(&mut world, b"C");
    let reason =
        act(&mut world, 180, FollowAct::Skip { after: [0xff; 32], reason: b"unknown".as_slice().into() }).unwrap();
    assert!(
        reason.starts_with(&forge_top::commit_text(&[0xff; 32]))
            && reason.ends_with(b"no queued entry has this commit; it may already be followed or skipped")
    );
    let reason = act(&mut world, 181, FollowAct::Merge { afters: Box::new([a, c]) }).unwrap();
    assert!(reason.ends_with(b"merge needs consecutive entries in queue order"));
    let merge_bound = usize::try_from(world.env.limits.forge.follow_merge).unwrap();
    let reason =
        act(&mut world, 182, FollowAct::Merge { afters: vec![a; merge_bound + 1].into_boxed_slice() }).unwrap();
    assert!(reason.ends_with(b"merge needs two to forge.follow_merge entries"));
    let length = usize::try_from(world.env.limits.forge.follow_reason_bytes).unwrap();
    assert_eq!(act(&mut world, 183, FollowAct::Hold { after: a, reason: vec![b'x'; length].into_boxed_slice() }), None);
    let reason =
        act(&mut world, 184, FollowAct::Hold { after: a, reason: vec![b'x'; length + 1].into_boxed_slice() }).unwrap();
    assert!(reason.ends_with(b"the reason exceeds forge.follow_reason_bytes"));
    assert_eq!(act(&mut world, 185, FollowAct::Skip { after: b, reason: b"build only".as_slice().into() }), None);
    let reason = act(&mut world, 186, FollowAct::Start { after: b }).unwrap();
    assert!(reason.starts_with(&forge_top::commit_text(&b)) && reason.ends_with(b"this entry was skipped"));
    no_holds(&world, &[change(&world, a), change(&world, c)]);
    world.close();
}
fn settle(world: &mut World) {
    for _ in 0..20 {
        world.tick();
    }
    world.until(Until::Ready);
}
fn row(world: &World) -> &FollowRow {
    let Some(Record::Forge { row }) =
        world.store.rows.get(&Key::Forge(forge_top::Key::Follow { project: 2, follow: 1 }))
    else {
        panic!("follow record")
    };
    let forge_top::Stored::Follow(row) = row.as_ref() else { panic!("follow row") };
    row
}
fn revision(world: &World) -> u32 {
    world
        .store
        .rows
        .values()
        .filter_map(|record| match record {
            Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Settings(
                cs::Record::Revision { history: false, revision },
            ))) => Some(revision.head.number),
            _ => None,
        })
        .max()
        .expect("settings head")
}
fn setting(start: s::FollowStart) -> s::Follow {
    s::Follow {
        source: s::FollowSource {
            repository: SOURCE,
            branch: b"main".as_slice().into(),
            paths: Box::new([b"src".as_slice().into(), b"include".as_slice().into()]),
        },
        target: s::FollowTarget { repository: TARGET, branch: b"main".as_slice().into() },
        owner: s::FollowOwner::Person(1),
        start,
        producer: None,
        check: s::FollowCheck {
            workflow: b"test.yml".as_slice().into(),
            artifact: b"results".as_slice().into(),
            format: temper_engine_domain_forge_tiers::Format::Lines,
        },
    }
}
fn keep(world: &mut World, start: Option<s::FollowStart>, key: u8) {
    let against = revision(world);
    change_setting(world, start.map(setting), key);
    assert_eq!(revision(world), against + 1, "{:?}; adoptions {:?}", world.web_replies, world.adopted);
}
fn change_setting(world: &mut World, value: Option<s::Follow>, key: u8) {
    let against = revision(world);
    let kinds = value
        .clone()
        .map(|value| engine::KindValue {
            kind: 12,
            scope: 2,
            number: 1,
            value: s::SizedValue {
                size: s::follow_record(&value).expect("valid follow setting").measure(),
                value: s::Value::Follow(value),
            },
        })
        .into_iter()
        .collect();
    world.send(engine::Event::Party(engine::PartyInput::ChangeSettings {
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: world.signed_in.expect("owner signed in"),
        key: [key; 16],
        digest: [key; 32],
        change: Box::new(cs::Change {
            against,
            items: Box::new([cs::Item {
                setting: cs::Setting::Application { kind: 12, scope: 2, number: 1 },
                edit: value.map_or(cs::Edit::Removed, |_| cs::Edit::Set(cs::Value::Application(Box::new([])))),
            }]),
            running: cs::Running::Stays,
        }),
        kinds,
    }));
    settle(world);
}
fn world(start: s::FollowStart) -> World {
    world_passing(start, 1000)
}
fn world_passing(start: s::FollowStart, passes: u32) -> World {
    world_with_gate(start, passes, Gate::default())
}
fn world_with_gate(start: s::FollowStart, passes: u32, gate: Gate) -> World {
    configured_follow(start, passes, gate, false)
}
fn configured_follow(start: s::FollowStart, passes: u32, gate: Gate, cued: bool) -> World {
    let mut world = unaccepted_follow(passes, gate, cued, true);
    keep(&mut world, Some(start), 110);
    assert!(row(&world).heard.is_some());
    world
}
fn unaccepted_follow(passes: u32, gate: Gate, cued: bool, context: bool) -> World {
    let mut world = World::configured_limits(
        true,
        true,
        false,
        1000,
        gate,
        None,
        StoryLimits {
            kinds: true,
            workflow_dispatch: true,
            // A seeded source stream and its ports retain more than the basic fake's 16 commits.
            forge_commits: cued.then_some(256),
            brief: cued.then(|| temper_service::limits::profile(temper_service::limits::Profile::Small).root.brief),
            person_executor: true,
            tier: Some(temper_engine_domain_world::tier_runs::Tier::default()),
            spec_bytes: Some(2048),
            applications: Some(24),
            inbox_messages: Some(16),
            inbox_bytes: Some(4096),
            tasks_batch: Some(8),
            authority_batch: Some(8),
            task_delegates: Some(16),
            tree_tasks: Some(64),
            slots: Some(4),
            stall: Some(Duration::from_secs(3600)),
            ..StoryLimits::default()
        },
    )
    .with_forge_pages(b"https://forge.example")
    .with_second_project();
    if cued {
        world = world.with_project_landing(
            2,
            engine::ProjectLanding {
                check_timeout: None,
                close_job: None,
                merge_job: None,
                by: engine::Way::Queue,
                repositories: Box::new([]),
                after: engine::AfterLandingCheck { tier_retries: 2, verification_bound_ms: 7_200_000 },
                landing_attempts: engine::LANDING_ATTEMPTS,
            },
        );
    }
    if cued {
        // At most two check runs per port, across the seed's source stream and drained tail.
        world.fake_env.limits.limits.workflow_runs = 64;
    }
    world.fake = fake::Domain::new(&world.fake_env.limits, 71);
    world.sign_in();
    if !context {
        super::context_projects::adopt_pair_without_context(&mut world);
    } else if cued {
        super::context_projects::adopt_pair_cued(&mut world);
    } else {
        super::context_projects::adopt_pair_passing(&mut world, b"vsr", b"vsr-rs", passes);
    }
    world.send(engine::Event::Party(engine::PartyInput::Period { project: 2, period: 1, budget: 1000, next: None }));
    world.send(protocol::hello_slots(Token::new(7), 4, Box::new([])));
    world
}
fn push(world: &mut World, message: &[u8]) -> skein_forge::api::Commit {
    let raw::Answer::Commit(parent) =
        world.external_at(b"org/vsr", 1, raw::Op::Read(raw::Read::Branch { branch: b"main".as_slice().into() }))
    else {
        panic!("source head")
    };
    let commit = fake::commit_tree(
        &mut world.fake,
        &world.fake_env.limits,
        parent,
        None,
        Box::new([raw::File { path: b"src/a.c".as_slice().into(), content: message.into() }]),
        message,
    )
    .expect("source commit fits");
    world.external_at(
        b"org/vsr",
        1,
        raw::Op::Git(raw::Git::Push { branch: b"main".as_slice().into(), commit, expected: Some(parent) }),
    );
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Hint {
        hint: skein_forge::api::Hint {
            repository: SOURCE,
            change: skein_forge::api::Change::Branch(b"main".as_slice().into()),
            key: None,
        },
    })));
    for _ in 0..TICKS {
        world.tick();
        if row(world).queue.iter().any(|entry| entry.after == translate::commit(commit) && entry.change.is_some())
            && world.store.pending.is_empty()
        {
            return translate::commit(commit);
        }
    }
    panic!(
        "follow change not made: {:?}; tasks {:?}",
        row(world),
        world
            .store
            .rows
            .values()
            .filter_map(|r| match r {
                Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row))) =>
                    Some((row.number, &row.phase)),
                _ => None,
            })
            .collect::<Vec<_>>()
    );
}
fn start(world: &mut World, after: skein_forge::api::Commit) {
    world.root.follow_start(2, 1, after).expect("entry may start");
    world.send(engine::Event::Resume);
    settle(world);
}
fn change(world: &World, after: skein_forge::api::Commit) -> u64 {
    row(world).queue.iter().find(|entry| entry.after == after).expect("queued commit").change.expect("made change")
}
fn cancel(world: &mut World, task: u64, key: u8) {
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: world.signed_in.expect("owner signed in"),
        key: [key; 16],
        models: Box::new([]),
        ask: people::Ask::Cancel { project: 2, task, reason: b"try again".as_slice().into(), grace: None },
    }));
    settle(world);
}
fn hint_target(world: &mut World) {
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Hint {
        hint: skein_forge::api::Hint {
            repository: TARGET,
            change: skein_forge::api::Change::Branch(b"main".as_slice().into()),
            key: None,
        },
    })));
    settle(world);
}
fn push_target(world: &mut World, change: u64) {
    let row = questions::row(world, change);
    let branch = row.branch.clone();
    let base = row.base.clone();
    let tip = world.fake.branch(b"org/vsr-rs", &base).expect("target base exists");
    if world.fake.branch(b"org/vsr-rs", &branch).is_none() {
        world.external_at(b"org/vsr-rs", 1, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: tip }));
    }
    fake::advance(&mut world.fake, &world.fake_env, b"org/vsr-rs", &branch, b"src/a.rs", b"port", 1)
        .expect("producer push fits");
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Hint {
        hint: skein_forge::api::Hint {
            repository: TARGET,
            change: skein_forge::api::Change::Branch(branch),
            key: None,
        },
    })));
}
fn produce(world: &mut World, change: u64, producer: &engine::Assignment) {
    push_target(world, change);
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        0,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: tasks::ChangeTitle::new(b"Port handoff".as_slice().into()),
                connector: 0,
                kind: 2,
                resource: forge_top::tag(1, 4),
                words: b"pushed".as_slice().into(),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
}
fn ended_task(world: &World, task: u64) -> &tasks::TaskRecord {
    let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))) =
        world.store.rows.get(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(task))))
    else {
        panic!("ended task {task}: {:?}", live_task(world, task).phase)
    };
    row
}
fn next_delegate(world: &mut World, change: u64, previous: u64) -> engine::Assignment {
    for _ in 0..temper_engine_domain_world::tier_runs::RUN_TICKS {
        world.tick();
        if let Some((child, _)) = questions::row(world, change).delegate
            && child != previous
            && let Some(run) = world.assigned.iter().find(|run| run.task == child)
        {
            return run.clone();
        }
    }
    panic!(
        "next delegate of {change}: {:?}; check {:?}",
        questions::row(world, change).change.state,
        questions::row(world, change).follow_check
    );
}
#[test]
fn accepted_follow_makes_owner_waiting_changes_with_code_words_and_chain() {
    let mut world = world(s::FollowStart::Owner);
    let task = row(&world).task.unwrap();
    let procedure = live_task(&world, task);
    assert_eq!(procedure.executor, tasks::Executor::Procedure { connector: 0, code: engine::FOLLOW_CODE });
    assert_eq!(procedure.requester, tasks::Party::Deployment { project: 2 });
    assert_eq!(procedure.spec.words.as_ref(), b"Follow 1 of project 2: org/vsr main into org/vsr-rs main");
    assert_eq!(procedure.numbers.budget, procedure.authority.budget.spend);
    assert!(procedure.numbers.budget > 0);
    let first = push(&mut world, b"Fix handoff\n\nThe body.");
    let a = change(&world, first);
    let spec = &live_task(&world, a).spec;
    let words = std::str::from_utf8(&spec.words).unwrap();
    let after = forge_top::commit_text(&first);
    let after = std::str::from_utf8(&after).unwrap();
    assert!(words.starts_with(&format!("Follow org/vsr {}: Fix handoff", &after[..12])), "{words}");
    assert!(words.contains("The body.") && words.contains("-- src include`") && words.contains("- src/a.c"), "{words}");
    assert!(
        words.contains(&format!(
            "Compare: https://forge.example/org/vsr/compare/{}...{after}",
            std::str::from_utf8(&forge_top::commit_text(&row(&world).queue[0].before)).unwrap()
        )),
        "{words}"
    );
    assert!(spec.parameters.iter().any(
        |p| matches!(p, tasks::Parameter::Resource { name: 1, resource, .. } if *resource == forge_top::tag(1, 4))
    ));
    assert!(
        spec.parameters
            .iter()
            .any(|p| matches!(p, tasks::Parameter::Bytes { name: 2, value } if value.as_ref() == b"main"))
    );
    assert!(spec.parameters.iter().any(|p| matches!(p, tasks::Parameter::Bytes { name: 18, value } if value.as_ref() == format!("1:{after}").as_bytes())));
    assert!(!spec.parameters.iter().any(|p| matches!(p, tasks::Parameter::Number { name: 14, .. })));
    for _ in 0..TICKS {
        world.tick();
    }
    assert_eq!(row(&world).queue[0].state, FollowState::WaitsWord);
    assert!(questions::row(&world, a).delegate.is_none(), "an owner change produces nothing even after reads settle");
    let second = push(&mut world, b"Second fix");
    let b = change(&world, second);
    assert_eq!(live_task(&world, b).waiting_on.as_ref(), &[a]);
    start(&mut world, first);
    let producer = next_delegate(&mut world, a, 0);
    assert_eq!(row(&world).queue[0].state, FollowState::Producing);
    produce(&mut world, a, &producer);
    for _ in 0..temper_engine_domain_world::tier_runs::RUN_TICKS {
        world.tick();
        if row(&world).queue.len() == 1 {
            break;
        }
    }
    settle(&mut world);
    assert_eq!(
        row(&world).queue.len(),
        1,
        "change {:?}; phase {:?}",
        questions::row(&world, a).change.state,
        live_task(&world, a).phase
    );
    assert_eq!(row(&world).queue[0].after, second);
    assert_eq!(row(&world).queue[0].state, FollowState::WaitsWord);
    start(&mut world, second);
    let producer = next_delegate(&mut world, b, 0);
    assert_eq!(live_task(&world, producer.task).project, 2);
    hint_target(&mut world);
    world.close();
}
#[test]
fn code_follow_produces_without_word_and_remakes_the_whole_held_tail() {
    let mut world = world(s::FollowStart::Code);
    let first = push(&mut world, b"First");
    let a = change(&world, first);
    let producer = next_delegate(&mut world, a, 0);
    keep(&mut world, Some(s::FollowStart::Owner), 119);
    let second = push(&mut world, b"Second");
    let b = change(&world, second);
    let third = push(&mut world, b"Third");
    let c = change(&world, third);
    start(&mut world, second);
    start(&mut world, third);
    assert!(questions::row(&world, b).delegate.is_none() && questions::row(&world, c).delegate.is_none());
    cancel(&mut world, a, 120);
    let event = world.protocol.answer(
        Token::new(7),
        producer.task,
        producer.attempt,
        0,
        tasks::End::Parked,
        engine::Parts::none(),
    );
    world.send(event);
    settle(&mut world);
    assert!(matches!(row(&world).queue[0].state, FollowState::Ended(_)));
    assert!(matches!(live_task(&world, b).phase, tasks::Phase::Held { .. }));
    start(&mut world, first);
    let made = [change(&world, first), change(&world, second), change(&world, third)];
    let mut facts = Vec::new();
    while let Some(fact) = world.root.pop_fact() {
        facts.push(fact);
    }
    assert!(
        made.iter().all(|m| ![a, b, c].contains(m)),
        "facts {facts:?}; phase {:?}",
        live_task(&world, row(&world).task.unwrap()).phase
    );
    assert_eq!(live_task(&world, made[1]).waiting_on.as_ref(), &[made[0]]);
    assert_eq!(live_task(&world, made[2]).waiting_on.as_ref(), &[made[1]]);
    assert!(matches!(ended_task(&world, b).phase, tasks::Phase::Ended(tasks::Ending::Cancelled { .. })));
    assert!(matches!(ended_task(&world, c).phase, tasks::Phase::Ended(tasks::Ending::Cancelled { .. })));
    assert!(!row(&world).queue.iter().any(|e| matches!(e.state, FollowState::Ended(_))));
    assert!(row(&world).queue.iter().all(|entry| entry.word));
    next_delegate(&mut world, made[0], 0);
    world.close();
}
#[test]
fn removing_and_cancelling_the_follow_stop_its_work_until_an_accepted_edit() {
    let mut world = world(s::FollowStart::Owner);
    let after = push(&mut world, b"First");
    let a = change(&world, after);
    let task = row(&world).task.unwrap();
    cancel(&mut world, task, 121);
    assert!(row(&world).stopped.is_some());
    assert_eq!(row(&world).task, Some(task));
    assert!(matches!(ended_task(&world, a).phase, tasks::Phase::Ended(_)));
    keep(&mut world, Some(s::FollowStart::Code), 122);
    assert_ne!(row(&world).task, Some(task));
    assert!(row(&world).stopped.is_none());
    for _ in 0..1300 {
        world.tick();
    }
    world.sign_in();
    keep(&mut world, None, 123);
    assert!(!world.store.rows.contains_key(&Key::Forge(forge_top::Key::Follow { project: 2, follow: 1 })));
    world.close();
    let mut second = self::world(s::FollowStart::Owner);
    let after = push(&mut second, b"Live removal");
    let child = change(&second, after);
    let task = row(&second).task.unwrap();
    keep(&mut second, None, 123);
    assert!(matches!(ended_task(&second, child).phase, tasks::Phase::Ended(tasks::Ending::Cancelled { .. })));
    assert!(matches!(ended_task(&second, task).phase, tasks::Phase::Ended(tasks::Ending::Cancelled { .. })));
    second.close();
}
#[test]
fn follow_questions_go_only_to_the_target_owner_and_answer_the_producer() {
    for answered in [true, false] {
        question_story(answered);
    }
}
fn question_story(answered: bool) {
    let mut world = world(s::FollowStart::Code);
    let after = push(&mut world, b"Questioned port");
    let a = change(&world, after);
    let producer = next_delegate(&mut world, a, 0);
    questions::call(
        &mut world,
        &producer,
        125,
        engine::Tool::Message {
            target: a,
            form: engine::MessageForm::Question,
            words: b"Keep the source behavior?".as_slice().into(),
        },
    );
    let Some(engine::CallAnswer::Sent { message: q }) = world.answers.last().cloned() else {
        panic!("question sent: {:?}", world.answers)
    };
    for _ in 0..TICKS {
        world.tick();
        if row(&world).questions.first().is_some_and(|q| q.person.is_some()) && world.store.pending.is_empty() {
            break;
        }
    }
    let question = row(&world).questions.first().expect("question forwarded").clone();
    let person = question.person.expect("owner person task");
    let person_row = live_task(&world, person);
    assert_eq!(person_row.project, 2);
    assert_eq!(person_row.executor, tasks::Executor::Person(tasks::PersonAddress::Person(1)));
    assert_eq!(
        person_row.spec.words.as_ref(),
        format!("Follow 1's change {a} asks (question {}): Keep the source behavior?", question.number).as_bytes()
    );
    assert!(!world.store.rows.values().any(|record| matches!(record, Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row))) if row.project == 1)));
    let revision = person_row.revisions.person;
    if answered {
        world.send(engine::Event::Party(engine::PartyInput::Ask {
            models: Box::new([]),
            reply_to: ReplyTo::new(Token::new(124)),
            sign_in: world.signed_in.expect("owner signed in"),
            key: [124; 16],
            ask: people::Ask::AnswerPerson {
                project: 2,
                task: person,
                revision,
                result: people::PersonResult::Report { words: b"Yes, keep it.".as_slice().into() },
            },
        }));
    } else {
        world.restart_same();
        world.sign_in();
        settle(&mut world);
        assert_eq!(row(&world).questions[0].person, Some(person), "restart retains the question's one person task");
        cancel(&mut world, person, 124);
    }
    let expected =
        if answered { b"Yes, keep it.".as_slice() } else { b"The follow's owner gave no answer.".as_slice() };
    for _ in 0..TICKS {
        world.tick();
        if questions::replies(&world, producer.task, a)
            .iter()
            .any(|word| word.kind == tasks::MessageKind::Answer { question: q })
        {
            break;
        }
    }
    assert!(
        questions::replies(&world, producer.task, a)
            .iter()
            .any(|word| word.kind == tasks::MessageKind::Answer { question: q } && word.words.as_ref() == expected),
        "replies {:?}; party {:?}; questions {:?}",
        questions::replies(&world, producer.task, a),
        world.web_replies,
        row(&world).questions
    );
    world.close();
}
#[test]
fn restart_keeps_the_made_change_and_makes_an_unmade_entry_once() {
    let mut world = world(s::FollowStart::Owner);
    let first = push(&mut world, b"First");
    let a = change(&world, first);
    let follow_task = row(&world).task.unwrap();
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(126)),
        sign_in: world.signed_in.expect("owner signed in"),
        key: [126; 16],
        ask: people::Ask::Stop { project: 2, task: follow_task },
    }));
    settle(&mut world);
    let raw::Answer::Commit(parent) =
        world.external_at(b"org/vsr", 1, raw::Op::Read(raw::Read::Branch { branch: b"main".as_slice().into() }))
    else {
        panic!("source head")
    };
    let second = fake::commit_tree(
        &mut world.fake,
        &world.fake_env.limits,
        parent,
        None,
        Box::new([raw::File { path: b"src/a.c".as_slice().into(), content: b"second".as_slice().into() }]),
        b"Second",
    )
    .expect("source commit fits");
    world.external_at(
        b"org/vsr",
        1,
        raw::Op::Git(raw::Git::Push { branch: b"main".as_slice().into(), commit: second, expected: Some(parent) }),
    );
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Hint {
        hint: skein_forge::api::Hint {
            repository: SOURCE,
            change: skein_forge::api::Change::Branch(b"main".as_slice().into()),
            key: None,
        },
    })));
    let mut cut = false;
    for _ in 0..TICKS {
        world.tick();
        if row(&world).queue.len() == 2 && row(&world).queue[1].change.is_none() && world.store.pending.is_empty() {
            cut = true;
            break;
        }
    }
    assert!(cut, "cut on the unmade entry");
    world.restart_same();
    world.sign_in();
    settle(&mut world);
    assert_eq!(row(&world).queue[1].change, None);
    let revision = live_task(&world, follow_task).revisions.hold;
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(127)),
        sign_in: world.signed_in.expect("owner signed in"),
        key: [127; 16],
        ask: people::Ask::Release { project: 2, task: follow_task, revision, again: None },
    }));
    for _ in 0..TICKS {
        world.tick();
        if row(&world).queue.len() == 2 && row(&world).queue[1].change.is_some() {
            break;
        }
    }
    assert_eq!(change(&world, first), a);
    let second = translate::commit(second);
    let b = change(&world, second);
    let changes: Vec<_> = world
        .store
        .rows
        .values()
        .filter_map(|record| match record {
            Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task)))
                if task.executor == (tasks::Executor::Procedure { connector: 0, code: 2 }) =>
            {
                Some(task.number)
            }
            _ => None,
        })
        .collect();
    assert_eq!(changes, [a, b]);
    assert_eq!(live_task(&world, b).waiting_on.as_ref(), &[a]);
    world.close();
}
#[test]
fn exhausted_follow_repairs_keep_the_hold_account_and_the_next_change_waits() {
    let mut world = world_passing(s::FollowStart::Code, 0);
    let first = push(&mut world, b"First");
    let a = change(&world, first);
    let mut run = next_delegate(&mut world, a, 0);
    let second = push(&mut world, b"Second");
    let b = change(&world, second);
    produce(&mut world, a, &run);
    for _ in 0..world.env.limits.forge.change_policy.repairs {
        run = next_delegate(&mut world, a, run.task);
        produce(&mut world, a, &run);
    }
    for _ in 0..TICKS {
        world.tick();
        if matches!(live_task(&world, a).phase, tasks::Phase::Held { .. }) {
            break;
        }
    }
    settle(&mut world);
    assert!(matches!(
        questions::row(&world, a).change.state,
        temper_engine_domain_forge_change::State::Held { why: temper_engine_domain_forge_change::Hold::Repairs, .. }
    ));
    let tasks::Phase::Held { source: Some(source), .. } = &live_task(&world, a).phase else { panic!("held change") };
    let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("hold account") };
    assert_eq!(row(&world).queue[0].state, FollowState::Held(words.clone()));
    assert_eq!(row(&world).queue.len(), 2);
    assert_eq!(live_task(&world, b).waiting_on.as_ref(), &[a]);
    assert!(questions::row(&world, b).delegate.is_none());
    world.close();
}
#[test]
fn code_follow_waits_for_landing_even_after_a_pushed_head() {
    let mut world = world(s::FollowStart::Code);
    let first = push(&mut world, b"First");
    let a = change(&world, first);
    let producer = next_delegate(&mut world, a, 0);
    let second = push(&mut world, b"Second");
    let b = change(&world, second);
    push_target(&mut world, a);
    for _ in 0..TICKS {
        world.tick();
    }
    let head = translate::commit(world.fake.branch(b"org/vsr-rs", &questions::row(&world, a).branch).unwrap());
    assert!(world.store.rows.values().any(|record| matches!(record, Record::Forge { row } if matches!(row.as_ref(), forge_top::Stored::Client(skein_forge_client::Stored::Live(row)) if row.cached.tip == Some(head)))), "the root has heard the pushed prerequisite");
    assert!(questions::row(&world, b).delegate.is_none());
    assert!(live_task(&world, b).started_from.is_empty());
    assert_eq!(questions::row(&world, b).change.early, temper_engine_domain_forge_change::Early::None);
    produce(&mut world, a, &producer);
    let next = next_delegate(&mut world, b, 0);
    assert_eq!(live_task(&world, next.task).project, 2);
    assert!(live_task(&world, b).waiting_on.is_empty());
    assert!(!row(&world).queue[0].word, "code needs no owner's word");
    hint_target(&mut world);
    world.close();
}

#[test]
fn a_landing_in_a_dependency_makes_no_task_or_change_in_the_dependent() {
    let mut world = unaccepted_follow(1000, Gate::default(), false, true);
    world.forge_reads = Some(Vec::new());
    let against = revision(&world);
    world.send(engine::Event::Party(engine::PartyInput::ChangeSettings {
        reply_to: ReplyTo::new(Token::new(110)),
        sign_in: world.signed_in.unwrap(),
        key: [110; 16],
        digest: [110; 32],
        change: Box::new(cs::Change {
            against,
            items: Box::new([cs::Item {
                setting: cs::Setting::Application { kind: 13, scope: 2, number: 1 },
                edit: cs::Edit::Set(cs::Value::Application(Box::new([]))),
            }]),
            running: cs::Running::Stays,
        }),
        kinds: Box::new([engine::KindValue {
            kind: 13,
            scope: 2,
            number: 1,
            value: s::SizedValue { value: s::Value::Dependency(s::Dependency { project: 1 }), size: 4 },
        }]),
    }));
    settle(&mut world);
    assert_eq!(revision(&world), against + 1);
    let tasks_before: Vec<_> = world
        .store
        .rows
        .values()
        .filter_map(|record| match record {
            Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row) | tasks::Stored::Ended(row))) => {
                Some(row.number)
            }
            _ => None,
        })
        .collect();
    assert!(!world.store.rows.values().any(|record| matches!(record, Record::Forge { row } if matches!(row.as_ref(), forge_top::Stored::Follow(_) | forge_top::Stored::Change(_)))), "dependency makes no follow or change row");
    let parent = world.fake.branch(b"org/vsr", b"main").unwrap();
    let commit = fake::commit_tree(
        &mut world.fake,
        &world.fake_env.limits,
        parent,
        None,
        Box::new([raw::File { path: b"src/a.c".as_slice().into(), content: b"Fix handoff".as_slice().into() }]),
        b"Fix handoff",
    )
    .unwrap();
    world.external_at(
        b"org/vsr",
        1,
        raw::Op::Git(raw::Git::Push { branch: b"main".as_slice().into(), commit, expected: Some(parent) }),
    );
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Hint {
        hint: skein_forge::api::Hint {
            repository: SOURCE,
            change: skein_forge::api::Change::Branch(b"main".as_slice().into()),
            key: None,
        },
    })));
    for _ in 0..TICKS {
        world.tick();
    }
    let tasks_after: Vec<_> = world
        .store
        .rows
        .values()
        .filter_map(|record| match record {
            Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row) | tasks::Stored::Ended(row))) => {
                Some(row.number)
            }
            _ => None,
        })
        .collect();
    assert_eq!(tasks_after, tasks_before, "no task comes from a dependency's landing");
    assert!(
        !world
            .store
            .rows
            .values()
            .any(|record| matches!(record, Record::Forge {row} if matches!(row.as_ref(),forge_top::Stored::Change(_))))
    );
    assert_eq!(revision(&world), against + 1);
    assert!(
        !world
            .forge_reads
            .as_ref()
            .unwrap()
            .iter()
            .any(|(_, read)| matches!(read, skein_forge::api::Read::Branch { .. })),
        "dependencies add no branch polling"
    );
    world.close();
}
