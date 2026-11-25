//! Two reviewed goals released by one person form one green turn
//! (temper/docs/design/domain/landing.md, 16).
use super::super::{goal_landing, reviews, tiers};
use super::*;
use jig_domain_core::{documents, reads};
use temper_engine_domain_forge_change as change;
use temper_protocol_worker_wire::v1::Record as WorkerRecord;

const SLOTS: u32 = 3; // Two goal runs and one producer, review or prepare job.

fn world() -> World {
    let mut world = World::configured_limits(
        true,
        true,
        false,
        1000,
        Gate { agent: true, freshness: Some(engine::Freshness::Clean), ..Gate::default() },
        None,
        StoryLimits {
            turn: true,
            batch: Some(2),
            landing_members: Some(2),
            person_executor: true,
            slots: Some(SLOTS),
            tasks: Some(6), // The two goals, their changes, latch and one worker.
            spec_bytes: Some(1024),
            inbox_messages: Some(4),
            ..StoryLimits::default()
        },
    );
    // Immediate peer replies and deadline-driven waits avoid polling between decisions.
    world.tick_interval = Duration::ZERO;
    world.fake_env.limits.latency_min = Duration::ZERO;
    world.fake_env.limits.latency_max = Duration::ZERO;
    world.adopt();
    world.send(hello_prepare_with_bounds(SLOTS, world.env.limits.tasks.message_bytes));
    world.manual_prepare_jobs();
    world
}

fn hello_prepare_with_bounds(slots: u32, message_bytes: u32) -> engine::Event {
    let mut event = hello_prepare(slots);
    let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello, .. }, .. }) = &mut event
    else {
        panic!("worker hello")
    };
    hello.bounds.message_bytes = message_bytes;
    event
}

fn goal(world: &mut World, key: u8, words: &[u8]) -> engine::Assignment {
    let ask = walking::goal_from(
        world.chat(1, words),
        people::TaskExecutor::Agent { charter: 1 },
        jig_test_profile::fixtures::Goal { budget: 200, priority: 3, deadline: None },
    );
    let replies = world.web_replies.len();
    let run = assigned_chat(world, key, ask);
    let (first, last) = world.web_replies[replies..]
        .iter()
        .find_map(|reply| match reply {
            people::Reply::Outcome(people::Outcome::Created { first, last, .. }) => Some((*first, *last)),
            _ => None,
        })
        .expect("goal start receipt");
    assert_eq!((first, last), (run.task, run.task), "goal start names the assigned goal");
    assert_eq!(live_task(world, run.task).root, run.task, "a root goal");
    run
}

fn person(world: &mut World, key: u8, ask: people::Ask) {
    tiers::sign_in(world);
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: world.signed_in.expect("owner"),
        key: [key; 16],
        ask,
    }));
    world.until(Until::Ready);
}

fn wait_on(world: &mut World, task: u64, latch: u64) {
    person(
        world,
        u8::try_from(100 + task).expect("small task"),
        people::Ask::Amend {
            project: 1,
            task,
            revision: live_task(world, task).revisions.spec,
            release: None,
            amendment: people::Amendment {
                tracked: None,
                addressed: None,
                instructions: None,
                spec: None,
                wake: None,
                dependencies: Some(Box::new([latch])),
                authority: None,
                llm: None,
                reason: b"Ready together".as_slice().into(),
            },
        },
    );
    assert_eq!(live_task(world, task).waiting_on.as_ref(), [latch]);
}

fn approve(world: &mut World, number: u64) {
    let assigned = world.assigned.len();
    for _ in 0..PAIR_TICKS {
        restarts::tick(world);
        if world.assigned.len() > assigned {
            break;
        }
        if world.events.is_empty()
            && world.store.pending.is_empty()
            && world.pending.is_empty()
            && let Some(next) = world.root.next_deadline(&world.env)
        {
            world.advance_to(world.env.now.max(next));
        }
    }
    let reviewer = world.assigned.get(assigned).expect("review assigned").clone();
    assert_eq!(live_task(world, reviewer.task).requester, tasks::Party::Task(number));
    reviews::finish_review(
        world,
        &reviewer,
        tasks::TaskResult::Verdict { code: 1, fields: Box::new([]), words: b"approved".as_slice().into() },
    );
    for _ in 0..PAIR_TICKS {
        restarts::tick(world);
        if copied::reviewed(world, number) {
            break;
        }
    }
    let row = goal_landing::row(world, number);
    assert!(copied::reviewed(world, number), "approved: {row:?}");
    assert!(matches!(row.change.state, change::State::Gating { .. }), "gating: {row:?}");
}

fn reviewed_batch(world: &mut World) -> (u64, [(u64, u64); 2]) {
    let a = goal(world, 92, b"Make turn-a");
    let mut latch = temper_engine_domain_world::fixtures::report_delegate(b"Wait for the batch", Box::new([]));
    latch.executor = tasks::Executor::Person(tasks::PersonAddress::Person(1));
    let engine::CallAnswer::Delegated(made) =
        world.scripted_call(&a, 95, engine::Tool::Delegate { batch: Box::new([latch]) })
    else {
        panic!("person latch delegated")
    };
    let latch = made[0].task;
    assert_eq!(live_task(world, latch).executor, tasks::Executor::Person(tasks::PersonAddress::Person(1)));
    let (c1, p1) = restarts::pushed_change(world, &a, 96);
    // Amend before finishing the producer so no readiness decision can race the person.
    wait_on(world, c1, latch);
    finish_producer(world, &p1, forge_world::REPO);
    approve(world, c1);
    let b = goal(world, 93, b"Make turn-b");
    let (c2, p2) = restarts::pushed_change(world, &b, 97);
    wait_on(world, c2, latch);
    finish_producer(world, &p2, forge_world::REPO);
    approve(world, c2);
    (latch, [(a.task, c1), (b.task, c2)])
}

fn release(world: &mut World, latch: u64, revision: u64) {
    person(
        world,
        120,
        people::Ask::AnswerPerson {
            project: 1,
            task: latch,
            revision,
            result: people::PersonResult::Report { words: b"Ready together".as_slice().into() },
        },
    );
}

fn task_document(world: &mut World, task: u64) -> Box<documents::Task> {
    let reads::Answer::Document(document) = world.party_read(reads::Read::Task { task }) else {
        panic!("live task read: {task}")
    };
    let documents::Document::Task(document) = *document else { panic!("task document: {task}") };
    assert_eq!(document.chip.number, task);
    document
}

#[test]
fn two_reviewed_changes_show_one_live_person_latch() {
    let mut world = world();
    let (latch, members) = reviewed_batch(&mut world);
    tiers::sign_in(&mut world);
    let sign_in = world.signed_in.expect("reader");
    for (_, change) in members {
        let Some(forge_top::Document::Change(panel)) = world.oracle_document(sign_in, 1, change) else {
            panic!("change panel: {change}")
        };
        assert_eq!(panel.task, change);
        assert_eq!(panel.step, forge_top::PanelStep::Gating);
        let [review] = panel.gates.as_ref() else { panic!("one review gate: {panel:?}") };
        assert_eq!(review.kind, change::GateKind::Agent);
        assert_eq!(review.verdict, change::Status::Passed);

        let task = task_document(&mut world, change);
        let [documents::TaskLink::Live(dependency)] = task.dependencies.as_slice() else {
            panic!("one live dependency: {:?}", task.dependencies)
        };
        assert_eq!(dependency.number, latch);
    }
    let Some(forge_top::Document::Landings(list)) =
        world.oracle_document(sign_in, 1, forge_top::landings_subject(1, true))
    else {
        panic!("project landings")
    };
    assert!(list.landings.is_empty(), "no forming decision before release: {list:?}");

    let revision = task_document(&mut world, latch).person_revision.expect("answerable person revision");
    let replies = world.web_replies.len();
    release(&mut world, latch, revision);
    assert!(
        world.web_replies[replies..].contains(&people::Reply::Outcome(people::Outcome::PersonAnswered { task: latch }))
    );
    assert!(ended(&world, latch), "person answered the latch: {:?}", world.web_replies);
}

#[test]
fn two_goals_land_in_one_green_turn() {
    let mut world = world();
    let (latch, members @ [(_, c1), (_, c2)]) = reviewed_batch(&mut world);
    for (owner, number) in members {
        let row = goal_landing::row(&world, number);
        assert_eq!(row.goal, Some(owner));
        assert!(copied::reviewed(&world, number), "approved before release: {row:?}");
        assert!(matches!(row.change.state, change::State::Gating { .. }), "gating before release: {row:?}");
        assert_eq!(live_task(&world, number).waiting_on.as_ref(), [latch]);
    }
    assert!(landing_rows(&world).is_empty(), "no landing before the person's answer");
    let reviews = world.assigned.len();
    let revision = live_task(&world, latch).revisions.person;
    release(&mut world, latch, revision);
    assert!(ended(&world, latch), "person answered the latch: {:?}", world.web_replies);
    let landing = restarts::await_prepare(&mut world);
    let spec = temper_protocol_worker_wire::v1::JobSpec::decode(
        &temper_protocol_worker_wire::v1::CEILINGS,
        &mut skein_lib::Reader::new(&world.jobs[0].spec),
    )
    .expect("prepare spec");
    let request: serde_json::Value = serde_json::from_slice(spec.request.as_ref()).expect("prepare request");
    assert_eq!(request["members"].as_array().expect("stack").len(), 2);
    let mut queue = [c1, c2];
    queue.sort_by_key(|number| (goal_landing::row(&world, *number).change.ready_since.expect("ready"), *number));
    restarts::answer_prepare(&mut world);
    restarts::await_landed(&mut world, landing);
    assert_eq!(world.assigned.len(), reviews, "clean approvals carry to both prefixes without another review");

    let rows = landing_rows(&world);
    let [row] = rows.as_slice() else { panic!("one landing: {rows:?}") };
    assert_eq!(row.landing.task, landing);
    assert_eq!(row.landing.phase, forge_top::LandingPhase::Landed);
    assert_eq!(row.landing.members.iter().map(|member| member.change).collect::<Vec<_>>(), queue);
    let members = row.landing.members.clone();
    for member in &members {
        assert!(member.told && member.landed.is_some());
        assert_eq!(member.landed, member.candidate);
        assert!(ended(&world, member.change));
        let change::State::Landed { merge, .. } = goal_landing::row(&world, member.change).change.state else {
            panic!("change landed")
        };
        assert_eq!(Some(merge), member.candidate);
        let raw::Answer::Pull(pull) = world.external(1, raw::Op::Read(raw::Read::Pull { number: member.pull })) else {
            panic!("member pull")
        };
        assert_eq!(pull.merged.map(translate::commit), member.candidate, "member merged at its prefix");
    }
    let top = members.last().expect("top");
    let dispatches: Vec<_> = world.writes.iter().filter(|write| matches!(write, api::Write::Dispatch { .. })).collect();
    let [api::Write::Dispatch { workflow, branch, .. }] = dispatches.as_slice() else {
        panic!("one dispatch: {dispatches:?}")
    };
    assert_eq!(workflow.as_ref(), b"gate.yml");
    assert_eq!(*branch, top.branch);
    let merges: Vec<_> = world.writes.iter().filter(|write| matches!(write, api::Write::Merge { .. })).collect();
    assert!(matches!(merges.as_slice(), [api::Write::Merge { how: api::MergeHow::FastForwardOnly, .. }]), "{merges:?}");
    let raw::Answer::Runs(runs) = world.external(
        1,
        raw::Op::Read(raw::Read::Runs {
            workflow: Some(b"gate.yml".as_slice().into()),
            commit: None,
            number: None,
            page: 1,
            limit: 2,
        }),
    ) else {
        panic!("gate runs")
    };
    let [run] = runs.as_ref() else { panic!("one gate run: {runs:?}") };
    assert_eq!(run.event, raw::RunEvent::Dispatch);
    assert_eq!(run.state, raw::RunState::Completed);
    assert_eq!(run.conclusion, Some(raw::Conclusion::Success));
    assert_eq!(Some(translate::commit(run.commit)), top.candidate);
    assert_eq!(world.fake.branch(b"org/repo", b"main"), Some(run.commit));
    tiers::sign_in(&mut world);
    let sign_in = world.signed_in.expect("reader");
    let Some(forge_top::Document::Landing(panel)) = world.oracle_document(sign_in, 1, landing) else {
        panic!("person's landing panel")
    };
    assert_eq!(panel.task, landing);
    assert_eq!(panel.phase, forge_top::LandingPhase::Landed);
    assert_eq!(panel.members.iter().map(|member| member.change).collect::<Vec<_>>(), queue);
    assert_eq!(panel.preparation, 1, "one successful round");
    let member = panel.members.last().expect("top member");
    assert_eq!(member.candidate, top.candidate);
    let gate = &member.gates[0];
    assert_eq!(gate.name.as_ref(), b"gate");
    assert!(gate.blocking);
    let forge_top::GateProof::Passed(proof) = &gate.proof else { panic!("passed gate proof: {gate:?}") };
    assert_eq!(proof.id, run.id);
}
