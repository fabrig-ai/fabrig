//! A target tier's red belongs to the follow owner's first standing area (follows.md, 6).
use super::*;
use forge_top::{Holder, settings as s};
use skein_forge::api::{Commit, Repository};

const PROJECT: u32 = 2;
const SOURCE: Repository = Repository { forge: 1, repository: 3 };
const TARGET: Repository = Repository { forge: 1, repository: 4 };

fn until(world: &mut World, description: &str, met: impl Fn(&World) -> bool) {
    for _ in 0..RUN_TICKS * 2 {
        if met(world) {
            return;
        }
        world.tick();
    }
    assert!(met(world), "{description}; tier {:?}; calls {:?}", fuzz(world), world.call_errors);
}
fn fuzz(world: &World) -> &forge_top::TierRow {
    let Some(Record::Forge { row }) =
        world.store.rows.get(&Key::Forge(forge_top::Key::Tier { repository: TARGET, tier: b"fuzz".as_slice().into() }))
    else {
        panic!("target tier")
    };
    let forge_top::Stored::Tier(row) = row.as_ref() else { panic!("tier row") };
    row
}
fn follow(world: &World) -> &forge_top::FollowRow {
    let Some(Record::Forge { row }) =
        world.store.rows.get(&Key::Forge(forge_top::Key::Follow { project: PROJECT, follow: 1 }))
    else {
        panic!("follow record")
    };
    let forge_top::Stored::Follow(row) = row.as_ref() else { panic!("follow row") };
    row
}
fn save(world: &mut World, items: Box<[jig_domain_core::settings::Item]>, key: u8) {
    tiers::sign_in(world);
    let against = kinds::revision(&world.store.rows);
    world.send(engine::Event::Party(engine::PartyInput::ChangeSettings {
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: world.signed_in.expect("project owner"),
        key: [key; 16],
        digest: [key; 32],
        kinds: kinds::decoded(&items),
        change: Box::new(jig_domain_core::settings::Change {
            against,
            items,
            running: jig_domain_core::settings::Running::Stays,
        }),
    }));
    world.until(Until::Ready);
    assert_eq!(kinds::revision(&world.store.rows), against + 1, "routing settings kept");
}
fn area(name: &[u8], owner: s::AreaOwner) -> s::Area {
    s::Area { name: name.into(), words: b"Ports to the target".as_slice().into(), owner }
}
fn world(person: bool, has_area: bool) -> (World, engine::Assignment, u64) {
    world_with_bound(person, has_area, 7_200_000)
}
fn configured(verification_bound_ms: u64) -> World {
    let mut world = World::configured_limits(
        true,
        true,
        false,
        1000,
        Gate::default(),
        None,
        StoryLimits {
            kinds: true,
            // A role's follow inherits its owner's grant to dispatch the follow check.
            workflow_dispatch: true,
            tier: Some(tier_runs::Tier::default()),
            person_executor: true,
            spec_bytes: Some(4096),
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
    .with_second_project()
    .with_project_landing(
        PROJECT,
        engine::ProjectLanding {
            gate_retries: 2,
            gate_timeout: skein_lib::Duration::from_millis(3_600_000),
            combine_main: engine::Combine::Merge,
            combine_other: engine::Combine::Squash,
            check_timeout: None,
            close_job: None,
            merge_job: None,
            by: engine::Way::Queue,
            repositories: Box::new([engine::RepositoryChecks {
                repository: TARGET,
                gates: Box::new([]),
                tiers: Box::new([engine::TierCheck {
                    name: b"fuzz".as_slice().into(),
                    workflow: b"fuzz.yml".as_slice().into(),
                    artifact: b"results".as_slice().into(),
                    format: temper_engine_domain_forge_tiers::Format::Lines,
                    follows: forge_top::Follows::Landing,
                    ignored: Box::new([]),
                    span_ms: 0,
                    description: Box::new([]),
                }]),
            }]),
            after: engine::AfterLandingCheck { tier_retries: 2, verification_bound_ms },
            landing_attempts: engine::LANDING_ATTEMPTS,
        },
    );
    // Keep the automatic red's full header and test id within the existing message bound.
    world.env.limits.tasks.message_bytes = 1024;
    let mut limits = temper_engine_domain_world::store::widest();
    limits.tasks = world.env.limits.tasks;
    limits.authority = world.env.limits.authority;
    let mut store = temper_engine_domain_world::store::Store::with_limits(&limits);
    store.seed(world.store.rows.values().cloned().collect(), Box::new([]));
    world.store = store;
    let client = world.env.limits.forge.client;
    world = world.with_forge_client(client);
    world.fake = fake::Domain::new(&world.fake_env.limits, 71);
    world
}
fn world_with_bound(person: bool, has_area: bool, verification_bound_ms: u64) -> (World, engine::Assignment, u64) {
    let mut world = configured(verification_bound_ms);
    world.sign_in();
    world.send(engine::Event::Party(engine::PartyInput::Period {
        project: PROJECT,
        period: 1,
        budget: 1000,
        next: None,
    }));
    world.until(Until::Ready);
    context_projects::adopt_pair_with_red_tier(&mut world);
    let mut hello = protocol::hello_slots(Token::new(7), 4, Box::new([]));
    let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello: worker, .. }, .. }) =
        &mut hello
    else {
        panic!("worker hello")
    };
    worker.bounds.message_bytes = world.env.limits.tasks.message_bytes;
    world.send(hello);
    let people::Ask::Create { project, mut batch } = world.chat(PROJECT, b"Triage findings") else {
        panic!("triager ask")
    };
    batch[0].budget = people::Budget::AsAllowed { cap: Some(100) };
    let triager = assigned_chat(&mut world, 120, people::Ask::Create { project, batch });
    let people::Ask::Create { project, mut batch } = world.chat(PROJECT, b"Own ports") else { panic!("role ask") };
    batch[0].label = 3;
    batch[0].tracked = None;
    batch[0].flags.conversation = false;
    batch[0].flags.ended_by = people::EndedBy::Standing;
    batch[0].budget = people::Budget::Named;
    batch[0].authority.spend = 100;
    let role = assigned_chat(&mut world, 121, people::Ask::Create { project, batch });
    let owner = if person { s::FollowOwner::Person(1) } else { s::FollowOwner::Task(role.task) };
    let area_owner = if person { s::AreaOwner::Party(1) } else { s::AreaOwner::Task(role.task) };
    let areas: Box<[_]> = if has_area {
        // A nonmatching holder first; matching names are deliberately not in byte order.
        Box::new([area(b"other", s::AreaOwner::Role(0)), area(b"port", area_owner), area(b"a-port", area_owner)])
    } else {
        Box::new([])
    };
    let setting = s::Follow {
        source: s::FollowSource {
            repository: SOURCE,
            branch: b"main".as_slice().into(),
            paths: Box::new([b"src".as_slice().into()]),
        },
        target: s::FollowTarget { repository: TARGET, branch: b"main".as_slice().into() },
        owner,
        start: s::FollowStart::Owner,
        producer: None,
        check: s::FollowCheck {
            workflow: b"test.yml".as_slice().into(),
            artifact: b"results".as_slice().into(),
            format: temper_engine_domain_forge_tiers::Format::Lines,
        },
    };
    save(
        &mut world,
        Box::new([
            kinds::item(12, PROJECT, 1, &s::Value::Follow(setting)),
            kinds::item(14, PROJECT, 1, &s::Value::ProjectAreas(s::ProjectAreas { areas })),
            kinds::item(
                15,
                PROJECT,
                1,
                &s::Value::ProjectTriager(s::ProjectTriager { owner: s::AreaOwner::Task(triager.task) }),
            ),
        ]),
        122,
    );
    until(&mut world, "target baseline and source observation", |world| {
        fuzz(world).record.sequence == 1 && follow(world).heard.is_some()
    });
    (world, role, triager.task)
}

fn land(world: &mut World) -> (u64, Commit) {
    let after = translate::commit(
        fake::advance(&mut world.fake, &world.fake_env, b"org/vsr", b"main", b"src/a.c", b"Fix handoff", 1)
            .expect("source landing"),
    );
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Hint {
        hint: skein_forge::api::Hint {
            repository: SOURCE,
            change: skein_forge::api::Change::Branch(b"main".as_slice().into()),
            key: None,
        },
    })));
    until(world, "follow change made", |world| {
        follow(world).queue.iter().any(|entry| entry.after == after && entry.change.is_some())
    });
    let change = follow(world)
        .queue
        .iter()
        .find(|entry| entry.after == after)
        .expect("source landing queued")
        .change
        .expect("follow made its change");
    world.root.follow_start(PROJECT, 1, after).expect("owner starts the port");
    world.send(engine::Event::Resume);
    until(world, "producer assigned", |world| {
        world.assigned.iter().any(|run| task(world, run.task).requester == tasks::Party::Task(change))
    });
    let producer = world
        .assigned
        .iter()
        .find(|run| task(world, run.task).requester == tasks::Party::Task(change))
        .expect("follow producer assigned")
        .clone();
    let branch = questions::row(world, change).branch.clone();
    let base = world.fake.branch(b"org/vsr-rs", b"main").expect("target base");
    world.external_at(b"org/vsr-rs", 1, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: base }));
    fake::advance(&mut world.fake, &world.fake_env, b"org/vsr-rs", &branch, b"failures", b"suite.alpha\n", 1)
        .expect("producer push");
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
                resource: forge_top::tag(TARGET.forge, TARGET.repository),
                words: b"pushed".as_slice().into(),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    until(world, "port landed", |world| {
        matches!(questions::row(world, change).change.state, temper_engine_domain_forge_change::State::Landed { .. })
    });
    assert_eq!(questions::row(world, change).goal, None);
    assert_eq!(questions::row(world, change).follows.as_ref().expect("landed follow member").follow, 1);
    (change, tier_runs::merge(world, change))
}
fn judged(world: &mut World, commit: Commit, area: Option<&[u8]>, holder: Holder) {
    let runs = world.assigned.len();
    until(world, "target landing reddens", |world| fuzz(world).record.sequence == 2);
    let rows = findings(world);
    let [finding] = rows.as_slice() else { panic!("one red finding: {rows:?}") };
    assert_eq!(finding.project, PROJECT);
    assert_eq!(finding.area.as_deref(), area);
    assert_eq!(finding.kind, forge_top::FindingKind::Red);
    assert_eq!(finding.severity, forge_top::Severity::High);
    assert_eq!(finding.title.as_ref(), b"Red after landing on main: tier fuzz");
    assert_eq!(engine::finding_holder(&world.root, PROJECT, finding.area.as_deref()), holder);
    let body = String::from_utf8_lossy(&finding.body);
    assert!(
        body.starts_with(&format!(
            "Red after landing: tier fuzz run 2 blames landing {} on main: 1 failing.\nsuite.alpha\n",
            String::from_utf8_lossy(&forge_top::commit_text(&commit))
        )),
        "{body}"
    );
    assert_eq!(world.assigned.len(), runs, "filing starts no agent's run");
    for _ in 0..LANDING_TICKS {
        world.tick();
    }
    assert_eq!(findings(world).len(), 1, "once per landing");
}

#[test]
fn a_follow_red_is_the_standing_roles_first_area_and_news_without_a_run() {
    let (mut world, role, triager) = world(false, true);
    let (_, commit) = land(&mut world);
    judged(&mut world, commit, Some(b"port"), Holder::Task(role.task));
    assert_eq!(task(&world, role.task).label, 3);
    let news = news(&world, role.task);
    assert_eq!(news.len(), 1);
    assert!(
        news[0]
            .words
            .starts_with(b"Finding 2/1 is yours: Red after landing on main: tier fuzz\nred, high, area port\n")
    );
    assert!(super::news(&world, triager).is_empty());
    world.close();
}
#[test]
fn a_persons_follow_red_lists_under_mine_with_their_area() {
    let (mut world, _, triager) = world(true, true);
    let (_, commit) = land(&mut world);
    judged(&mut world, commit, Some(b"port"), Holder::Party(1));
    tiers::sign_in(&mut world);
    let session = world.signed_in.expect("person signed in");
    let document = world
        .oracle_document(
            session,
            PROJECT,
            forge_top::findings_subject(PROJECT, Some(forge_top::FindingState::New), true),
        )
        .expect("mine document");
    let forge_top::Document::Findings(document) = document else { panic!("findings read") };
    assert_eq!(document.rows.len(), 1);
    assert_eq!(document.rows[0].holder, Holder::Party(1));
    assert_eq!(document.rows[0].area.as_deref(), Some(b"port".as_slice()));
    assert!(news(&world, triager).is_empty());
    world.close();
}
#[test]
fn a_follow_red_without_an_owned_area_falls_back_to_the_triager() {
    let (mut world, _, triager) = world(false, false);
    let (_, commit) = land(&mut world);
    judged(&mut world, commit, None, Holder::Task(triager));
    assert!(
        news(&world, triager)[0]
            .words
            .starts_with(b"Finding 2/1 is yours: Red after landing on main: tier fuzz\nred, high, no area\n")
    );
    world.close();
}
#[test]
fn a_forgotten_follow_change_leaves_its_red_with_the_triager() {
    // Its verification bound expires before the slower tier, so the member can leave kept history.
    let (mut world, role, triager) = world_with_bound(false, true, 10_000);
    let (change, commit) = land(&mut world);
    until(&mut world, "uncleared change ended", |world| matches!(task(world, change).phase, tasks::Phase::Ended(_)));
    world.drain_cut();
    assert!(follow(&world).queue.is_empty(), "follow already counted its landing");
    world.store.seed(Box::new([]), Box::new([Key::Forge(forge_top::Key::Change(change))]));
    world.restart_same();
    until(&mut world, "root restored while the tier runs", |world| world.root.ready());
    judged(&mut world, commit, None, Holder::Task(triager));
    assert_eq!(news(&world, triager).len(), 1);
    assert!(news(&world, role.task).is_empty());
    world.close();
}
#[test]
fn a_follow_red_with_an_ended_area_owner_falls_back_to_the_triager() {
    let (mut world, role, triager) = world(false, true);
    let (_, commit) = land(&mut world);
    tiers::sign_in(&mut world);
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(130)),
        sign_in: world.signed_in.expect("project owner signed in"),
        key: [130; 16],
        ask: people::Ask::Cancel {
            project: PROJECT,
            task: role.task,
            reason: b"role ended".as_slice().into(),
            grace: None,
        },
    }));
    goal_landing::finish_run(&mut world, &role, tasks::TaskResult::Report { words: b"Done".as_slice().into() });
    until(&mut world, "owner ended", |world| matches!(task(world, role.task).phase, tasks::Phase::Ended(_)));
    judged(&mut world, commit, None, Holder::Task(triager));
    assert_eq!(news(&world, triager).len(), 1);
    world.close();
}
