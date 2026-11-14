//! A delivered head moves on despite an answered question and a sibling's landing
//! (temper/docs/design/domain/forge/change/README.md, 2).
use super::goal_questions::{ask_up, finish_story, independent, land, next_producer, resumed};
use super::stall_hold::{assert_stalled, jump, preparing, waiting};
use super::questions::row;
use super::*;
use temper_engine_domain_forge_change::State;

const STEPS: u32 = 1000; // A read, durable hold and escalation through the root.

fn held_reads(ci: bool) -> (World, u64) {
    let (mut world, _, task, producer) = preparing(false);
    if ci {
        waiting(&mut world, task, &producer, true);
    } else {
        let branch = row(&world, task).branch.clone();
        world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 }));
        fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"fixed", 1)
            .expect("producer pushed");
    }
    world.hold_reads_at_any_step = true;
    world.held_reads = Some(Vec::new());
    if ci {
        let poll = world.env.limits.forge.client.poll_max;
        jump(&mut world, poll);
    } else {
        super::ci_poll::finish(&mut world, &producer);
    }
    for _ in 0..STEPS {
        world.tick();
        if world.held_reads.as_ref().is_some_and(|reads| !reads.is_empty())
            && world.store.rows.contains_key(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(producer.task))))
            && world.store.pending.is_empty()
        {
            break;
        }
    }
    assert!(world.held_reads.as_ref().is_some_and(|reads| !reads.is_empty()));
    assert!(matches!(row(&world, task).change.state, State::Producing { .. } | State::Checking { .. }));
    (world, task)
}

fn stalled_words(world: &World, task: u64, step: &str, what: &str) {
    assert_stalled(world, task);
    let tasks::Phase::Held { source: Some(source), .. } = &live_task(world, task).phase else { panic!("hold source") };
    let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("stall account") };
    let minutes = world.env.limits.forge.change_policy.stall.as_nanos() / 60_000_000_000;
    assert_eq!(String::from_utf8_lossy(words), format!("no progress at {step} for {minutes} minutes: waiting for {what}"));
}

#[test]
fn an_unanswered_producing_gather_holds_its_row_and_task() {
    let (mut world, task) = held_reads(false);
    let bound = world.env.limits.forge.change_policy.stall;
    jump(&mut world, bound.saturating_add(Duration::from_secs(1)));
    stalled_words(&world, task, "producing", "the forge to answer its reads");
}

fn release_reads(world: &mut World) {
    for read in world.held_reads.take().expect("held reads") {
        world.out.push(read);
    }
    world.collect();
}

#[test]
fn a_busy_client_retries_then_holds_at_producing() {
    let (mut world, task) = held_reads(false);
    world.busy_reads = true;
    world.forge_reads = Some(Vec::new());
    release_reads(&mut world);
    for _ in 0..100 {
        world.tick();
    }
    assert!(matches!(row(&world, task).change.state, State::Producing { .. }));
    assert!(world.forge_reads.as_ref().expect("read count").len() > 1, "busy reads are retried before the bound");
    let bound = world.env.limits.forge.change_policy.stall;
    jump(&mut world, bound.saturating_add(Duration::from_secs(1)));
    stalled_words(&world, task, "producing", "the forge to answer its reads");
}

#[test]
fn an_unanswered_checking_gather_names_its_ci_head() {
    let (mut world, task) = held_reads(true);
    let State::Checking { head } = row(&world, task).change.state else { panic!("checking") };
    let short: String = head.iter().take(6).map(|byte| format!("{byte:02x}")).collect();
    let what = format!("CI at {short}");
    let bound = world.env.limits.forge.change_policy.stall;
    jump(&mut world, bound.saturating_add(Duration::from_secs(1)));
    stalled_words(&world, task, "checking", &what);
}


#[test]
fn an_answered_producer_opens_its_head_after_a_sibling_moves_the_goal_branch() {
    let mut story = independent();
    let (question, up) = ask_up(&mut story.world, &story.producer, story.a);
    story.world.goal_answer(&story.goal, 105, story.a, up, b"Keep the design.");
    let producer = resumed(&mut story.world, &story.producer, story.a, question, b"Keep the design.");
    let sibling = next_producer(&mut story.world, story.b);
    let tip = land(&mut story.world, story.goal.task, story.b, &sibling, story.main, b"B", b"Done: B.");
    assert_ne!(tip, story.main);
    // A's pushed head starts from the old base, not B's landed head.
    land(&mut story.world, story.goal.task, story.a, &producer, story.main, b"A", b"Done: A.");
    assert_eq!(story.world.merges, 2);
    finish_story(&mut story);
}

fn ended_producer(world: &World, task: u64) -> Box<tasks::TaskRecord> {
    world.store.rows.values().find_map(|record| match record {
        Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))
            if row.requester == tasks::Party::Task(task) => Some(row.clone()),
        _ => None,
    }).expect("producer's stored end")
}

fn party(world: &mut World, number: u8, ask: people::Ask) {
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]), reply_to: ReplyTo::new(Token::new(u64::from(number))),
        sign_in: world.signed_in.expect("owner"), key: [number; 16], ask,
    }));
}

fn release_with_result_in_record(placed: bool) {
    let (mut world, task) = held_reads(false);
    for _ in 0..100 { world.tick(); }
    let ended = ended_producer(&world, task);
    let mut projection = row(&world, task).clone();
    projection.delegate = Some((ended.number, temper_engine_domain_forge_change::Delegate::Produce));
    projection.delegate_status = temper_engine_domain_forge_change::Status::Unknown;
    projection.title = Box::new([]);
    projection.report = Box::new([]);
    if placed {
        party(&mut world, 170, people::Ask::Stop { project: 1, task });
    } else {
        projection.change.state = State::Held {
            why: temper_engine_domain_forge_change::Hold::Stalled,
            was: Box::new(State::Producing { requested: true }),
        };
        let tasks::Executor::Procedure { connector, code } = live_task(&world, task).executor else { panic!("procedure") };
        world.send(engine::Event::Party(engine::PartyInput::ProcedureStep {
            task, step: live_task(&world, task).attempt + 1, connector, code,
            decision: engine::ProcedureStep::new(engine::StepEnd::Hold(tasks::Hold::Stalled)),
        }));
    }
    world.store.seed(Box::new([Record::Forge { row: Box::new(forge_top::Stored::Change(Box::new(projection.clone()))) }]), Box::new([]));
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Restore { record: forge_top::Stored::Change(Box::new(projection)) })));
    for _ in 0..STEPS {
        world.tick();
        if matches!(live_task(&world, task).phase, tasks::Phase::Held { .. }) && world.store.pending.is_empty() { break; }
    }
    assert!(matches!(live_task(&world, task).phase, tasks::Phase::Held { .. }));
    assert_eq!(matches!(row(&world, task).change.state, State::Held { .. }), !placed);
    // Preserve the old unanswered reads; only newly requested reads may answer.
    let old = world.held_reads.take().expect("old gather");
    world.forge_reads = Some(Vec::new());
    let revision = live_task(&world, task).revisions.hold;
    party(&mut world, 171, people::Ask::Release { project: 1, task, revision, again: None });
    for _ in 0..STEPS {
        world.tick();
        if row(&world, task).pull.is_some() && world.store.pending.is_empty() { break; }
    }
    let after = row(&world, task).clone();
    assert!(after.pull.is_some(), "a release abandons its gather and takes the stored result: {after:?}");
    assert_eq!(after.title.as_ref(), b"Change");
    assert_eq!(after.report.as_ref(), b"pushed");
    let reads = world.forge_reads.as_ref().expect("fresh reads");
    assert!(reads.iter().any(|(_, read)| matches!(read, skein_forge::api::Read::Branch { branch } if *branch == after.branch)));
    assert!(reads.iter().any(|(_, read)| matches!(read, skein_forge::api::Read::Branch { branch } if *branch == after.base)));
    assert!(reads.iter().any(|(_, read)| matches!(read, skein_forge::api::Read::PullFor { .. } | skein_forge::api::Read::Pull { .. })));
    for read in old { world.out.push(read); }
    world.collect();
    for _ in 0..100 { world.tick(); }
    assert_eq!(row(&world, task).title, after.title, "late answers cannot replace the result");
    assert_eq!(row(&world, task).pull, after.pull, "late answers cannot open another pull");
    assert!(!matches!(row(&world, task).change.state, State::Held { .. }));
}

#[test]
fn a_release_abandons_the_old_gather_and_reads_the_producers_record() {
    release_with_result_in_record(false);
}

#[test]
fn a_partys_release_refreshes_an_unheld_change_too() {
    release_with_result_in_record(true);
}

#[test]
fn a_stored_question_of_an_ended_asker_no_longer_exempts_the_stall() {
    let (mut world, _, task, producer) = preparing(false);
    waiting(&mut world, task, &producer, false);
    for _ in 0..100 { world.tick(); }
    let ended = ended_producer(&world, task);
    let mut projection = row(&world, task).clone();
    projection.title = Box::new([]);
    projection.delegate = Some((ended.number, temper_engine_domain_forge_change::Delegate::Produce));
    projection.questions.open = Box::new([temper_engine_domain_forge_change::Open {
        asker: ended.number, number: 910, line: b"Old question".as_slice().into(), words: b"Old question".as_slice().into(),
        stands: temper_engine_domain_forge_change::Stands::Asked,
    }]);
    world.store.seed(Box::new([Record::Forge { row: Box::new(forge_top::Stored::Change(Box::new(projection.clone()))) }]), Box::new([]));
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Restore { record: forge_top::Stored::Change(Box::new(projection)) })));
    let bound = world.env.limits.forge.change_policy.stall;
    let half = Duration::from_nanos(bound.as_nanos() / 2);
    jump(&mut world, half);
    assert!(!matches!(row(&world, task).change.state, State::Held { .. }));
    jump(&mut world, half.saturating_add(Duration::from_secs(1)));
    assert_stalled(&world, task);
    assert_eq!(row(&world, task).change.since, ended.ended_at.expect("asker's end"));
}
