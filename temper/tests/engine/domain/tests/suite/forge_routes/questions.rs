//! A change relays its agents' questions (temper/docs/design/domain/forge/change/questions.md, 1).
use super::*;
use temper_engine_domain_forge_change::{Route, Stands, State};

const TICKS: u32 = 200;
const CHANNEL: Token = Token::new(7);
const QUESTION: &[u8] = b"Which behavior should hold?\nChoose the documented behavior or widen the change.\nI recommend keeping the documented behavior.";
const ANSWER: &[u8] = b"Keep the documented behavior.\nLeave the wider work for later.";
const REPORT_IN_FINISH: &[u8] =
    b"Your report goes in finish; this change takes no words. To ask: message target=this change, form=question.";
const CALL: u64 = 93;
// A requester run reserves 60; the change and its producer share the remainder.
const BUDGET: u64 = 200;
// Keep ordinary relay stories within the stall; the stall story advances the clock.

pub(super) fn plan_world(bound: Option<u32>) -> (World, engine::Assignment) {
    let mut world = World::configured_limits(
        true,
        true,
        false,
        1000,
        Gate { approval: None, agent: false, owner: false },
        None,
        StoryLimits {
            slots: Some(3),
            questions: bound,
            stall: Some(Duration::from_secs(3600)),
            ..StoryLimits::default()
        },
    );
    // The question and its contract share the required task section.
    world.env.limits.brief.budgets.task = 512;
    let client = world.env.limits.forge.client;
    world = world.with_forge_client(client);
    world.adopt();
    world.send(protocol::hello_slots(CHANNEL, 3, Box::new([])));
    let people::Ask::Create { project, mut batch } = adoption_chat(&world) else { unreachable!() };
    batch[0].budget = people::Budget::AsAllowed { cap: Some(BUDGET * 2) };
    batch[0].authority.tools |= jig_domain_core::ToolFamilies::JIG.control.0;
    let parent = assigned_chat(&mut world, 92, people::Ask::Create { project, batch });
    let mut authority = live_task(&world, parent.task).authority.clone();
    authority.budget.spend = BUDGET + 100;
    authority.delegation.depth = 2;
    authority.delegation.tasks = 8;
    call(
        &mut world,
        &parent,
        90,
        engine::Tool::Delegate {
            batch: Box::new([engine::Delegate {
                authority,
                ..temper_engine_domain_world::fixtures::report_delegate(b"Coordinate the agreed work", Box::new([]))
            }]),
        },
    );
    world.until(Until::SecondAssignment);
    let requester = world.assigned.iter().find(|run| run.task != parent.task).expect("requester assigned").clone();
    let turns = live_task(&world, parent.task).turn;
    park_after_turn(&mut world, &parent, None, turns);
    world.until(Until::Ready);
    (world, requester)
}

pub(super) fn call(world: &mut World, run: &engine::Assignment, number: u64, tool: engine::Tool) {
    let before = world.answers.len() + world.call_errors.len();
    let event = world.protocol.call(
        CHANNEL,
        run.task,
        run.attempt,
        number,
        engine::Call { completion: live_task(world, run.task).turn + 1, position: 1, tool },
    );
    world.send(event);
    for _ in 0..TICKS {
        world.tick();
        if world.answers.len() + world.call_errors.len() > before && world.store.pending.is_empty() {
            let turn = live_task(world, run.task).turn + 1;
            let read = match world
                .store
                .rows
                .get(&Key::Core(jig_domain_core::Key::Core(jig_domain_core::CoreKey::RunProof(run.task))))
            {
                Some(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::RunProof(proof)))) => {
                    proof.turn.as_ref().and_then(|turn| turn.read)
                }
                _ => None,
            };
            world.send(protocol::turn(CHANNEL, run.task, run.attempt, turn, 0, read, b"Continue the agreed work"));
            for _ in 0..TICKS {
                world.tick();
                if live_task(world, run.task).turn >= turn && world.store.pending.is_empty() {
                    return;
                }
            }
            panic!(
                "call {number} evidence committed: wanted turn {turn}; row turn {}; phase {:?}; answers {:?}",
                live_task(world, run.task).turn,
                live_task(world, run.task).phase,
                world.answers
            );
        }
    }
    panic!("call {number} answered: answers={:?}; errors={:?}", world.answers, world.call_errors);
}

pub(super) fn make_change(
    world: &mut World,
    requester: &engine::Assignment,
    number: u64,
    questions: engine::ChangeQuestions,
) -> u64 {
    let mut change = engine::ChangeCall::words(b"Make the agreed fix".as_slice().into(), None);
    change.questions = Some(questions);
    call(world, requester, number, engine::Tool::Change { changes: Box::new([change]) });
    let Some(engine::CallAnswer::Delegated(made)) = world.answers.last() else {
        panic!("change made: {:?} {:?}", world.answers, world.call_errors)
    };
    made[0].task
}

pub(super) fn next_producer(world: &mut World, change: u64) -> engine::Assignment {
    for _ in 0..TICKS {
        if let Some(run) =
            world.assigned.iter().rev().find(|run| live_task(world, run.task).requester == tasks::Party::Task(change))
        {
            return run.clone();
        }
        world.tick();
    }
    panic!(
        "producer assigned: {:?}",
        world
            .store
            .rows
            .values()
            .filter_map(|record| match record {
                Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(row))) =>
                    Some((row.number, &row.phase, &row.numbers)),
                Record::Core(_) | Record::Forge { .. } => None,
            })
            .collect::<Vec<_>>()
    );
}

pub(super) fn row(world: &World, change: u64) -> &forge_top::ChangeRow {
    let Some(Record::Forge { row }) = world.store.rows.get(&Key::Forge(forge_top::Key::Change(change))) else {
        panic!("change row")
    };
    let forge_top::Stored::Change(row) = row.as_ref() else { panic!("change row") };
    row
}

pub(super) fn replies(world: &World, target: u64, from: u64) -> Vec<&tasks::Word> {
    live_task(world, target).inbox.iter().filter(|word| word.from == tasks::Party::Task(from)).collect()
}

fn up_world(bound: Option<u32>) -> (World, engine::Assignment, u64, engine::Assignment) {
    let (mut world, requester) = plan_world(bound);
    let change = make_change(&mut world, &requester, CALL, engine::ChangeQuestions::Up);
    let producer = next_producer(&mut world, change);
    world.until(Until::Ready);
    (world, requester, change, producer)
}

pub(super) fn ask(world: &mut World, producer: &engine::Assignment, change: u64) -> u64 {
    let number = world.restore_delegate_question(producer.task, QUESTION);
    for _ in 0..TICKS {
        world.tick();
        if row(world, change).questions.open.first().is_some_and(|q| q.stands == Stands::Asked)
            && world.store.pending.is_empty()
        {
            return number;
        }
    }
    panic!("question passed up: {:?}", row(world, change));
}

pub(super) fn answer(world: &mut World, requester: &engine::Assignment, change: u64, number: u64) {
    call(
        world,
        requester,
        number,
        engine::Tool::Message {
            target: change,
            form: engine::MessageForm::Answer { question: live_task(world, change).questions[0].number },
            words: ANSWER.into(),
        },
    );
}

pub(super) fn finish_producing(world: &mut World, change: u64, producer: &engine::Assignment) {
    temper_engine_domain_world::kinds::produce(world, change, producer);
    for _ in 0..TICKS {
        world.tick();
        if !matches!(row(world, change).change.state, State::Producing { .. }) {
            break;
        }
    }
    assert!(!matches!(row(world, change).change.state, State::Producing { .. }));
}

#[test]
fn a_producers_words_are_refused_by_the_call_and_its_finish_goes_through() {
    let (mut world, _, change, producer) = up_world(None);
    call(
        &mut world,
        &producer,
        CALL + 1,
        engine::Tool::Message {
            target: change,
            form: engine::MessageForm::Words,
            words: b"My report".as_slice().into(),
        },
    );
    assert_eq!(
        world.call_errors.last().map(|error| error.text.as_ref()),
        Some(REPORT_IN_FINISH),
        "the call refuses the report: {:?}",
        world.answers
    );
    assert_eq!(replies(&world, producer.task, change).len(), 0);
    assert_eq!(replies(&world, change, producer.task).len(), 0);
    finish_producing(&mut world, change, &producer);
}

#[test]
fn a_producers_words_found_at_a_step_hear_the_same_words_and_its_finish_goes_through_once_read() {
    let (mut world, _, change, producer) = up_world(None);
    world.restore_delegate_words(producer.task, b"My report");
    for _ in 0..TICKS {
        world.tick();
        if !replies(&world, producer.task, change).is_empty() && world.store.pending.is_empty() {
            break;
        }
    }
    let words = replies(&world, producer.task, change);
    assert_eq!(words.len(), 1);
    assert_eq!(words[0].kind, tasks::MessageKind::Words);
    assert_eq!(words[0].words.as_ref(), REPORT_IN_FINISH);
    let read = words[0].number;
    for _ in 0..TICKS {
        world.tick();
        if world
            .delivered_messages
            .get(&(producer.task, producer.attempt))
            .is_some_and(|messages| messages.contains(&read))
            && world.store.pending.is_empty()
        {
            break;
        }
    }
    assert!(
        world
            .delivered_messages
            .get(&(producer.task, producer.attempt))
            .is_some_and(|messages| messages.contains(&read)),
        "feedback delivered before its reading turn"
    );
    let turn = live_task(&world, producer.task).turn + 1;
    world.send(protocol::turn(CHANNEL, producer.task, producer.attempt, turn, 0, Some(read), b"Read the feedback"));
    for _ in 0..TICKS {
        world.tick();
        if live_task(&world, producer.task).inbox.is_empty() && world.store.pending.is_empty() {
            break;
        }
    }
    assert!(live_task(&world, producer.task).inbox.is_empty());
    finish_producing(&mut world, change, &producer);
}

#[test]
fn a_change_with_questions_up_keeps_its_agents_message_family() {
    let (world, _, change, producer) = up_world(None);
    assert_eq!(row(&world, change).questions.route, Route::Up);
    assert_ne!(live_task(&world, producer.task).authority.tools.0 & jig_domain_core::ToolFamilies::JIG.message.0, 0);
}

#[test]
fn a_question_open_past_the_stall_holds_nothing() {
    let (mut world, requester, change, producer) = up_world(None);
    // An indefinite wait's polling must not supply the missing answer wake.
    let stall = world.env.limits.forge.change_policy.stall;
    world.env.limits.forge.queue_window = stall.saturating_add(stall);
    ask(&mut world, &producer, change);
    park_after_turn(&mut world, &producer, None, 0);
    let before_question = row(&world, change).change.since;
    world.env.now = world.env.now.saturating_add(stall.saturating_add(stall));
    for _ in 0..TICKS {
        world.tick();
    }
    assert_eq!(row(&world, change).change.state, State::Producing { requested: true });
    assert!(!matches!(live_task(&world, change).phase, tasks::Phase::Held { .. }));
    assert!(!row(&world, change).questions.open.is_empty());
    let answering_at = world.env.wall;
    answer(&mut world, &requester, change, CALL + 1);
    wait_answer(&mut world, producer.task, change);
    for _ in 0..TICKS {
        world.tick();
    }
    let since = row(&world, change).change.since;
    assert!(since >= answering_at);
    assert!(since > before_question);
    let deadline = since.as_nanos().saturating_add(stall.as_nanos());
    // Leave a minute before the deadline, beyond these ticks' twenty seconds.
    world.env.now = Time::from_nanos(deadline - Duration::from_secs(60).as_nanos());
    for _ in 0..TICKS {
        world.tick();
    }
    assert_eq!(row(&world, change).change.state, State::Producing { requested: true });
    assert!(row(&world, change).change.since >= since);
    world.env.now = Time::from_nanos(deadline);
    for _ in 0..TICKS {
        world.tick();
    }
    assert_eq!(row(&world, change).change.state, State::Producing { requested: true });
    assert!(!matches!(live_task(&world, change).phase, tasks::Phase::Held { .. }));
}

#[test]
fn a_producers_question_goes_up_and_the_answer_comes_down() {
    let (mut world, requester, change, producer) = up_world(None);
    let question = ask(&mut world, &producer, change);
    let words = replies(&world, requester.task, change);
    assert_eq!(words.len(), 1);
    assert_eq!(words[0].kind, tasks::MessageKind::Question);
    assert_eq!(words[0].words.as_ref(), QUESTION);
    park_after_turn(&mut world, &producer, None, 0);
    answer(&mut world, &requester, change, CALL + 1);
    for _ in 0..TICKS {
        world.tick();
        if !replies(&world, producer.task, change).is_empty() {
            break;
        }
    }
    let words = replies(&world, producer.task, change);
    assert_eq!(words.len(), 1);
    assert_eq!(words[0].kind, tasks::MessageKind::Answer { question });
    assert_eq!(words[0].words.as_ref(), ANSWER);
    assert!(live_task(&world, producer.task).questions.is_empty());
    assert!(row(&world, change).questions.open.is_empty());
    for _ in 0..TICKS {
        world.tick();
        if world.assigned.iter().any(|run| run.task == producer.task && run.attempt > producer.attempt) {
            break;
        }
    }
    let resumed = world
        .assigned
        .iter()
        .find(|run| run.task == producer.task && run.attempt > producer.attempt)
        .expect("the producer's next run goes on");
    assert!(
        resumed
            .messages
            .iter()
            .any(|word| word.kind == tasks::MessageKind::Answer { question } && word.words.as_ref() == ANSWER)
    );
}

/// temper/docs/design/domain/agent/README.md, 5: an unchecked push is the next brief's head.
#[test]
fn a_resolution_that_saves_and_asks_reads_its_next_pull_brief_at_the_saved_head() {
    let (mut world, requester, change, producer) = up_world(None);
    // The producer and landing branch change the same file, requiring a resolution.
    temper_engine_domain_world::kinds::produce(&mut world, change, &producer);
    let branch = row(&world, change).branch.clone();
    let landing = row(&world, change).base.clone();
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &landing, b"file", b"base moved", 1)
        .expect("base conflicts with the producer");
    let resolution = super::reviews::next_assignment(&mut world, 3);
    assert_eq!(
        row(&world, change).delegate,
        Some((
            resolution.task,
            temper_engine_domain_forge_change::Delegate::Resolve {
                base: translate::commit(world.fake.branch(b"org/repo", &landing).expect("base head")),
            }
        ))
    );
    let previous = world.fake.branch(b"org/repo", &branch).expect("producer head");
    let base = world.fake.branch(b"org/repo", &landing).expect("base head");
    let saved = fake::merge_commit(
        &mut world.fake,
        &world.fake_env.limits,
        previous,
        base,
        Box::new([raw::File {
            path: b"file".as_slice().into(),
            content: b"resolved work in progress".as_slice().into(),
        }]),
        b"Unchecked: work in progress, saved before a question",
    )
    .expect("the worker commits the unchecked resolved tree");
    assert_eq!(
        world.external(
            2,
            raw::Op::Git(raw::Git::Push { branch: branch.clone(), commit: saved, expected: Some(previous) })
        ),
        raw::Answer::Pushed(raw::Pushed::Pushed)
    );
    let question = ask(&mut world, &resolution, change);
    let state = row(&world, change).change.state.clone();
    // A save returns the same pushed-head part as a delivery, without a checked result.
    let event = world.protocol.answer(
        CHANNEL,
        resolution.task,
        resolution.attempt,
        0,
        tasks::End::Parked,
        engine::Parts {
            pushed: Box::new([forge_top::Pushed {
                tag: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                commit: translate::commit(saved),
            }]),
            ..engine::Parts::none()
        },
    );
    world.send(event);
    world.until(Until::Ready);
    assert_eq!(row(&world, change).change.state, state, "a save leaves the resolution unfinished");
    answer(&mut world, &requester, change, CALL + 1);
    let resumed = super::reviews::next_assignment(&mut world, 4);
    assert_eq!(resumed.task, resolution.task, "the same resolution continues");
    assert!(resumed.attempt > resolution.attempt);
    assert!(resumed.messages.iter().any(|word| word.kind == tasks::MessageKind::Answer { question }));
    let pull = resumed
        .sections
        .iter()
        .find_map(|section| {
            if section.kind == engine::BriefKind::Forge(engine::ForgeBriefKind::Pull)
                && let engine::BriefBody::Text(words) = &section.body
            {
                Some(words.as_ref())
            } else {
                None
            }
        })
        .expect("resumed resolution reads its pull section");
    let head = forge_top::commit_text(&translate::commit(saved));
    assert!(contains(pull, &head), "{}", String::from_utf8_lossy(pull));
    assert!(contains(pull, b"resolved work in progress"));
    assert_eq!(row(&world, change).change.last_head, Some(translate::commit(saved)));
    assert_eq!(live_task(&world, resolution.task).tries, tasks::Tries::NONE);
    world.close();
}

/// temper/docs/design/domain/agent/README.md, 5: an unchecked push is the next brief's head.
#[test]
fn a_resolution_whose_save_is_not_reported_rebriefs_at_the_reread_head() {
    let (mut world, requester, change, producer) = up_world(None);
    // The producer and landing branch change the same file, requiring a resolution.
    temper_engine_domain_world::kinds::produce(&mut world, change, &producer);
    let branch = row(&world, change).branch.clone();
    let landing = row(&world, change).base.clone();
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &landing, b"file", b"base moved", 1)
        .expect("base conflicts with the producer");
    let resolution = super::reviews::next_assignment(&mut world, 3);
    assert_eq!(
        row(&world, change).delegate,
        Some((
            resolution.task,
            temper_engine_domain_forge_change::Delegate::Resolve {
                base: translate::commit(world.fake.branch(b"org/repo", &landing).expect("base head")),
            }
        ))
    );
    let previous = world.fake.branch(b"org/repo", &branch).expect("producer head");
    let base = world.fake.branch(b"org/repo", &landing).expect("base head");
    let saved = fake::merge_commit(
        &mut world.fake,
        &world.fake_env.limits,
        previous,
        base,
        Box::new([raw::File {
            path: b"file".as_slice().into(),
            content: b"resolved work in progress".as_slice().into(),
        }]),
        b"Unchecked: work in progress, saved before a question",
    )
    .expect("the worker commits the unchecked resolved tree");
    assert_eq!(
        world.external(
            2,
            raw::Op::Git(raw::Git::Push { branch: branch.clone(), commit: saved, expected: Some(previous) })
        ),
        raw::Answer::Pushed(raw::Pushed::Pushed)
    );
    let question = ask(&mut world, &resolution, change);
    let state = row(&world, change).change.state.clone();
    let delegate = row(&world, change).delegate;
    let pull_number = row(&world, change).pull;
    let writes = world.writes.len();
    let merges = world.merges;
    // Only the authenticated read that frees this writer can report the save.
    let event = world.protocol.answer(
        CHANNEL,
        resolution.task,
        resolution.attempt,
        0,
        tasks::End::Parked,
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..TICKS {
        world.tick();
        if super::goal_branch::writer(&world, &branch).is_none() && world.store.pending.is_empty() {
            break;
        }
    }
    assert_eq!(super::goal_branch::writer(&world, &branch), None, "the reread frees the writer");
    assert_eq!(row(&world, change).change.state, state, "a save leaves the resolution unfinished");
    assert_eq!(row(&world, change).delegate, delegate);
    assert_eq!(row(&world, change).pull, pull_number);
    assert_eq!(world.writes.len(), writes, "a save makes no forge effect");
    assert_eq!(world.merges, merges);
    answer(&mut world, &requester, change, CALL + 1);
    assert_eq!(
        row(&world, change).change.last_head,
        Some(translate::commit(saved)),
        "the accepted save pins the next brief"
    );
    let resumed = super::reviews::next_assignment(&mut world, 4);
    assert_eq!(resumed.task, resolution.task, "the same resolution continues");
    assert!(resumed.attempt > resolution.attempt);
    assert!(resumed.messages.iter().any(|word| word.kind == tasks::MessageKind::Answer { question }));
    let pull = resumed
        .sections
        .iter()
        .find_map(|section| {
            if section.kind == engine::BriefKind::Forge(engine::ForgeBriefKind::Pull)
                && let engine::BriefBody::Text(words) = &section.body
            {
                Some(words.as_ref())
            } else {
                None
            }
        })
        .expect("resumed resolution reads its pull section");
    let head = forge_top::commit_text(&translate::commit(saved));
    assert!(contains(pull, &head), "{}", String::from_utf8_lossy(pull));
    assert!(contains(pull, b"resolved work in progress"));
    assert_eq!(row(&world, change).change.last_head, Some(translate::commit(saved)));
    assert_eq!(live_task(&world, resolution.task).tries, tasks::Tries::NONE);
    assert_eq!(row(&world, change).change.state, state);
    assert_eq!(row(&world, change).delegate, delegate);
    assert_eq!(row(&world, change).pull, pull_number);
    assert_eq!(world.writes.len(), writes);
    assert_eq!(world.merges, merges);
    world.close();
}

#[test]
fn the_change_merges_nothing_while_its_question_is_up() {
    let (mut world, _, change, producer) = up_world(None);
    ask(&mut world, &producer, change);
    let branch = row(&world, change).branch.clone();
    world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 }));
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"file", b"agreed fix", 1)
        .expect("producer pushed");
    super::delegates::fixtures::answer_producer(&mut world, &producer);
    for _ in 0..TICKS {
        world.tick();
    }
    assert_eq!(world.merges, 0);
    assert_eq!(live_task(&world, change).questions.len(), 1);
}

#[test]
fn a_restart_keeps_an_open_question_where_it_stood() {
    let (mut world, requester, change, producer) = up_world(None);
    ask(&mut world, &producer, change);
    world.until(Until::Ready);
    let questions = row(&world, change).questions.clone();
    world.restart(true, true);
    world.until(Until::Ready);
    assert_eq!(row(&world, change).questions, questions);
    assert_eq!(replies(&world, requester.task, change).len(), 1);
}

#[test]
fn questions_none_and_up_are_offered_with_control() {
    let (_, requester) = plan_world(None);
    let charter = temper_engine_smith::charter(&requester);
    let tool = charter.grants.host_tools.iter().find(|tool| tool.name.as_ref() == b"change").expect("change offered");
    let schema: serde_json::Value = serde_json::from_slice(&tool.schema).expect("change schema");
    for properties in [&schema["properties"], &schema["properties"]["changes"]["items"]["properties"]] {
        assert_eq!(properties["questions"]["enum"], serde_json::json!(["none", "up"]));
    }
}

pub(super) fn wait_answer(world: &mut World, producer: u64, change: u64) {
    for _ in 0..TICKS {
        world.tick();
        if !replies(world, producer, change).is_empty()
            && row(world, change).questions.open.is_empty()
            && world.store.pending.is_empty()
        {
            return;
        }
    }
    panic!(
        "answer relayed: merges={}; questions={:?}; waiting={:?}; changes={:?}",
        world.merges,
        row(world, change).questions,
        live_task(world, change).waiting_on,
        world
            .store
            .rows
            .values()
            .filter_map(|record| match record {
                Record::Forge { row } => match row.as_ref() {
                    forge_top::Stored::Change(row) => Some((row.task, &row.change.state)),
                    _ => None,
                },
                Record::Core(_) => None,
            })
            .collect::<Vec<_>>()
    );
}

fn bound_world() -> (World, engine::Assignment, u64, engine::Assignment, u64) {
    let (mut world, requester, change, producer) = up_world(Some(1));
    ask(&mut world, &producer, change);
    answer(&mut world, &requester, change, CALL + 1);
    wait_answer(&mut world, producer.task, change);
    let question = world.restore_delegate_question(producer.task, b"Second question: should the scope widen?");
    for _ in 0..TICKS {
        world.tick();
        if matches!(live_task(&world, change).phase, tasks::Phase::Held { why: tasks::Hold::Procedure, .. })
            && world.store.pending.is_empty()
        {
            break;
        }
    }
    (world, requester, change, producer, question)
}

#[test]
fn past_its_bound_the_change_holds_naming_the_question() {
    let (world, requester, change, producer, question) = bound_world();
    let tasks::Phase::Held { why: tasks::Hold::Procedure, source: Some(source), .. } = &live_task(&world, change).phase
    else {
        panic!("held past its bound")
    };
    let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("hold account") };
    let expected = format!(
        "its questions are spent: 1 of 1 taken; question {question} of task {} waits; release it to pass the question up",
        producer.task
    );
    assert_eq!(words.as_ref(), expected.as_bytes());
    assert!(matches!(
        row(&world, change).change.state,
        temper_engine_domain_forge_change::State::Held { why: temper_engine_domain_forge_change::Hold::Questions, .. }
    ));
    assert!(replies(&world, requester.task, change).iter().any(|word| word.words.as_ref() == expected.as_bytes()));
    assert_eq!(row(&world, change).questions.open[0].stands, Stands::Taken);
}

#[test]
fn released_it_passes_the_question_up() {
    let (mut world, requester, change, _, question) = bound_world();
    let revision = live_task(&world, change).revisions.hold;
    call(&mut world, &requester, CALL + 2, engine::Tool::Release { target: change, revision, again: None });
    for _ in 0..TICKS {
        world.tick();
        if row(&world, change).questions.open.first().is_some_and(|q| q.stands == Stands::Asked) {
            break;
        }
    }
    assert_eq!(row(&world, change).questions.open[0].number, question);
    assert_eq!(
        row(&world, change).questions.open[0].stands,
        Stands::Asked,
        "answers={:?}; why={:?}; phase={:?}",
        world.answers,
        live_task(&world, change).due_reason,
        live_task(&world, change).phase
    );
    assert_eq!(row(&world, change).questions.taken, 1);
    assert!(replies(&world, requester.task, change).iter().any(|word| word.kind == tasks::MessageKind::Question
        && word.words.as_ref() == b"Second question: should the scope widen?"));
}

#[test]
fn a_question_from_another_task_names_the_enabled_route() {
    let (mut world, requester, change, _) = up_world(None);
    call(
        &mut world,
        &requester,
        CALL + 1,
        engine::Tool::Message { target: change, form: engine::MessageForm::Question, words: QUESTION.into() },
    );
    let question = live_task(&world, requester.task).questions[0].number;
    for _ in 0..TICKS {
        world.tick();
        if !replies(&world, requester.task, change).is_empty() {
            break;
        }
    }
    let words = replies(&world, requester.task, change);
    assert_eq!(words.len(), 1);
    assert_eq!(words[0].kind, tasks::MessageKind::Answer { question });
    assert_eq!(
        words[0].words.as_ref(),
        b"Relays questions. Use read_task on this change; message target=this change, form=answer, question=number. Words do not answer."
    );
    assert!(row(&world, change).questions.open.is_empty());
}

#[test]
fn an_asker_that_ends_drops_its_question() {
    let (mut world, requester, change, producer) = up_world(None);
    let number = ask(&mut world, &producer, change);
    call(
        &mut world,
        &requester,
        CALL + 1,
        engine::Tool::Cancel { target: producer.task, reason: b"stop the asker".as_slice().into() },
    );
    park_after_turn(&mut world, &producer, None, 0);
    for _ in 0..TICKS {
        world.tick();
        if matches!(live_task(&world, change).phase, tasks::Phase::Held { .. }) && world.store.pending.is_empty() {
            break;
        }
    }
    assert!(matches!(live_task(&world, change).phase, tasks::Phase::Held { .. }));
    assert!(world.call_errors.is_empty(), "requester may cancel its descendant: {:?}", world.call_errors);
    assert_eq!(row(&world, change).questions.open[0].stands, Stands::Asked);
    assert_eq!(live_task(&world, change).questions.len(), 1, "the change still owes its own answer");
    let expected = format!(
        "Task {} ended; your answer to its question {number} goes no further: Which behavior should hold?",
        producer.task
    );
    assert!(!replies(&world, requester.task, change).iter().any(|word| word.words.as_ref() == expected.as_bytes()));
    world.restart(true, true);
    world.until(Until::Ready);
    answer(&mut world, &requester, change, CALL + 2);
    let revision = live_task(&world, change).revisions.hold;
    call(&mut world, &requester, CALL + 3, engine::Tool::Release { target: change, revision, again: None });
    for _ in 0..TICKS {
        world.tick();
    }
    assert!(row(&world, change).questions.open.is_empty());
    assert!(live_task(&world, change).questions.is_empty());
    let words: Vec<_> = replies(&world, requester.task, change)
        .into_iter()
        .filter(|word| word.kind == tasks::MessageKind::Words && word.words.as_ref() == expected.as_bytes())
        .collect();
    assert_eq!(words.len(), 1);
    assert!(!world.store.rows.values().any(|record| matches!(record, Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Activity(line))) if line.task == producer.task && matches!(line.line, tasks::Line::Words { kind: tasks::MessageKind::Answer { .. }, .. }))));
    world.restart(true, true);
    world.until(Until::Ready);
    assert_eq!(
        replies(&world, requester.task, change)
            .into_iter()
            .filter(|word| word.kind == tasks::MessageKind::Words && word.words.as_ref() == expected.as_bytes())
            .count(),
        1
    );
}

#[test]
fn a_resolution_that_ends_with_a_relayed_question_still_holds_its_change() {
    let (mut world, requester, change, producer) = up_world(None);
    temper_engine_domain_world::kinds::produce(&mut world, change, &producer);
    let landing = row(&world, change).base.clone();
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &landing, b"file", b"base moved", 1)
        .expect("base conflicts with the producer");
    let resolution = super::reviews::next_assignment(&mut world, 3);
    let number = ask(&mut world, &resolution, change);
    call(
        &mut world,
        &requester,
        CALL + 1,
        engine::Tool::Cancel { target: resolution.task, reason: b"stop the asker".as_slice().into() },
    );
    park_after_turn(&mut world, &resolution, None, 0);
    for _ in 0..TICKS {
        world.tick();
        if matches!(live_task(&world, change).phase, tasks::Phase::Held { source: Some(_), .. })
            && world.store.pending.is_empty()
        {
            break;
        }
    }
    assert!(
        matches!(&row(&world, change).change.state, State::Held { was, why: temper_engine_domain_forge_change::Hold::Failed }
        if matches!(was.as_ref(), State::Resolving { .. }))
    );
    let tasks::Phase::Held { why: tasks::Hold::Procedure, source: Some(source), .. } = &live_task(&world, change).phase
    else {
        panic!("the ended resolution holds its change with its account: {:?}", live_task(&world, change).phase)
    };
    let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("resolution's account") };
    assert_eq!(
        words.as_ref(),
        format!("its resolution, task {}, was cancelled: stop the asker", resolution.task).as_bytes()
    );
    assert_eq!(row(&world, change).questions.open[0].stands, Stands::Asked);
    assert_eq!(live_task(&world, change).questions.len(), 1, "the change still owes its own answer");
    assert!(matches!(live_task(&world, change).escalation,
        tasks::Escalation::Waiting { holder: tasks::EscalationHolder::Task(holder), .. } if holder == requester.task));
    world.drain_cut();
    world.restart(true, true);
    world.until(Until::Ready);
    answer(&mut world, &requester, change, CALL + 2);
    let revision = live_task(&world, change).revisions.hold;
    call(&mut world, &requester, CALL + 3, engine::Tool::Release { target: change, revision, again: None });
    for _ in 0..TICKS {
        world.tick();
    }
    assert!(row(&world, change).questions.open.is_empty());
    assert!(live_task(&world, change).questions.is_empty());
    let expected = format!(
        "Task {} ended; your answer to its question {number} goes no further: Which behavior should hold?",
        resolution.task
    );
    assert_eq!(
        replies(&world, requester.task, change)
            .into_iter()
            .filter(|word| word.kind == tasks::MessageKind::Words && word.words.as_ref() == expected.as_bytes())
            .count(),
        1
    );
    world.close();
}

#[test]
fn a_producers_finish_waits_for_its_requesters_answer() {
    let (mut world, requester, change, producer) = up_world(None);
    let number = ask(&mut world, &producer, change);
    super::reviews::finish_review(
        &mut world,
        &producer,
        tasks::TaskResult::Failure { reason: b"need a decision".as_slice().into() },
    );
    world.until(Until::Ready);
    assert_eq!(live_task(&world, producer.task).phase, tasks::Phase::Active(tasks::Active::Idle));
    assert_eq!(row(&world, change).questions.open[0].stands, Stands::Asked);
    assert_eq!(row(&world, change).delegate.map(|(task, _)| task), Some(producer.task));
    assert!(!matches!(live_task(&world, change).phase, tasks::Phase::Held { .. }));
    answer(&mut world, &requester, change, CALL + 1);
    wait_answer(&mut world, producer.task, change);
    world.until(Until::Ready);
    let resumed = next_producer(&mut world, change);
    assert_eq!(resumed.task, producer.task);
    assert!(resumed.attempt > producer.attempt);
    assert!(
        resumed
            .messages
            .iter()
            .any(|word| word.kind == tasks::MessageKind::Answer { question: number } && word.words.as_ref() == ANSWER)
    );
    finish_producing(&mut world, change, &resumed);
    assert!(row(&world, change).change.last_head.is_some(), "the resumed producer's change goes on");
    assert!(!matches!(live_task(&world, change).phase, tasks::Phase::Held { .. }));
}

#[test]
fn an_answer_with_a_new_dependency_waits_for_it_to_land() {
    let (mut world, requester, change, producer) = up_world(None);
    ask(&mut world, &producer, change);
    park_after_turn(&mut world, &producer, None, 0);
    let dependency = make_change(&mut world, &requester, CALL + 1, engine::ChangeQuestions::None);
    let upstream = next_producer(&mut world, dependency);
    let mut amendment =
        temper_engine_domain_world::fixtures::task_amendment(None, None, b"Land the prerequisite first");
    amendment.dependencies = Some(Box::new([dependency]));
    let revision = live_task(&world, change).revisions.spec;
    call(
        &mut world,
        &requester,
        CALL + 2,
        engine::Tool::Amend { keep_parameters: false, target: change, revision, amendment },
    );
    answer(&mut world, &requester, change, CALL + 3);
    for _ in 0..TICKS {
        world.tick();
        if row(&world, change).questions.open.first().is_some_and(|q| matches!(q.stands, Stands::Held { .. })) {
            break;
        }
    }
    assert_eq!(row(&world, change).questions.open[0].stands, Stands::Held { answer: ANSWER.into() });
    assert_eq!(replies(&world, producer.task, change).len(), 0);
    let branch = row(&world, dependency).branch.clone();
    world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 }));
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"prerequisite", b"ready", 1)
        .expect("prerequisite pushed");
    super::delegates::fixtures::answer_producer(&mut world, &upstream);
    wait_answer(&mut world, producer.task, change);
    assert_eq!(world.merges, 1);
    assert_eq!(replies(&world, producer.task, change)[0].words.as_ref(), ANSWER);
}

#[test]
fn a_batch_of_one_answers_then_asks_the_next_question() {
    let (mut world, requester, change, producer) = up_world(None);
    let first = ask(&mut world, &producer, change);
    let second = world.restore_delegate_question(producer.task, b"Another question, kept whole.");
    for _ in 0..TICKS {
        world.tick();
        if row(&world, change).questions.open.len() == 2 && world.store.pending.is_empty() {
            break;
        }
    }
    answer(&mut world, &requester, change, CALL + 1);
    for _ in 0..TICKS {
        world.tick();
        if row(&world, change).questions.open.len() == 1
            && row(&world, change).questions.open[0].stands == Stands::Asked
            && world.store.pending.is_empty()
        {
            break;
        }
    }
    assert_eq!(world.env.limits.tasks.batch, 1);
    assert_eq!(row(&world, change).questions.open[0].number, second);
    assert_eq!(row(&world, change).questions.open[0].stands, Stands::Asked);
    assert_eq!(row(&world, change).questions.taken, 2);
    assert_eq!(replies(&world, producer.task, change)[0].kind, tasks::MessageKind::Answer { question: first });
    assert_eq!(
        replies(&world, requester.task, change).iter().filter(|word| word.kind == tasks::MessageKind::Question).count(),
        2
    );
}

pub(super) fn fill_requester_inbox(world: &mut World, requester: &engine::Assignment) -> u32 {
    let mut target = Box::new(live_task(world, requester.task).clone());
    let mut header = world.store.header();
    // The change's terminal keeps one inbox slot; fill the ordinary slots.
    let ordinary = world.env.limits.tasks.inbox_messages
        - u32::try_from(target.delegates.len()).expect("the live delegates fit u32");
    let mut inbox = target.inbox.into_vec();
    for _ in 0..ordinary {
        header.messages += 1;
        inbox.push(tasks::Word {
            number: header.messages,
            from: target.requester,
            kind: tasks::MessageKind::Words,
            words: b"Pending instruction".as_slice().into(),
            at: world.env.wall,
            hits: 1,
            eligible: true,
        });
    }
    let Some(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::RunProof(mut proof)))) = world
        .store
        .rows
        .get(&Key::Core(jig_domain_core::Key::Core(jig_domain_core::CoreKey::RunProof(requester.task))))
        .cloned()
    else {
        panic!("requester run proof")
    };
    let offered: Vec<u64> = inbox
        .iter()
        .take(usize::try_from(world.env.limits.tasks.run_inbox).expect("run inbox bound"))
        .map(|word| word.number)
        .collect();
    proof.offered = offered.last().copied();
    proof.outstanding = offered.into_boxed_slice();
    target.inbox = inbox.into_boxed_slice();
    target.last_message = header.messages;
    header.commits += 1;
    world.store.pending.push_back((
        header.commits,
        Box::new([
            engine::Write::Save(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(target)))),
            engine::Write::Save(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::RunProof(
                proof,
            )))),
            engine::Write::Save(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Deployment(
                header,
            )))),
        ]),
    ));
    world.store.apply();
    world.restart(true, true);
    ordinary
}

#[test]
fn a_full_requester_inbox_holds_the_question_for_release() {
    let (mut world, requester, change, producer) = up_world(None);
    world.restore_delegate_question(producer.task, QUESTION);
    let ordinary = fill_requester_inbox(&mut world, &requester);
    for _ in 0..TICKS {
        world.tick();
        if matches!(live_task(&world, change).phase, tasks::Phase::Held { why: tasks::Hold::EffectFailed, .. })
            && world.store.pending.is_empty()
        {
            break;
        }
    }
    let tasks::Phase::Held { why: tasks::Hold::EffectFailed, source: Some(source), .. } =
        &live_task(&world, change).phase
    else {
        panic!("question refusal holds: {:?}", live_task(&world, change).phase)
    };
    let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("jig's refusal account") };
    assert!(words.windows(b"busy: its inbox is full".len()).any(|part| part == b"busy: its inbox is full"));
    assert_eq!(row(&world, change).questions.open[0].stands, Stands::Taken);
    assert_eq!(row(&world, change).questions.open[0].words.as_ref(), QUESTION);
    assert!(live_task(&world, change).questions.is_empty());
    let turn = live_task(&world, requester.task).turn + 1;
    let Some(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::RunProof(proof)))) = world
        .store
        .rows
        .get(&Key::Core(jig_domain_core::Key::Core(jig_domain_core::CoreKey::RunProof(requester.task))))
    else {
        panic!("requester proof")
    };
    let read = proof.offered.expect("the full inbox's offered prefix");
    world.send(protocol::turn(
        CHANNEL,
        requester.task,
        requester.attempt,
        turn,
        0,
        Some(read),
        b"Read the pending instructions",
    ));
    for _ in 0..TICKS {
        world.tick();
        if live_task(&world, requester.task).inbox.len() < usize::try_from(ordinary).expect("ordinary inbox bound")
            && world.store.pending.is_empty()
        {
            break;
        }
    }
    let revision = live_task(&world, change).revisions.hold;
    call(&mut world, &requester, CALL + 1, engine::Tool::Release { target: change, revision, again: None });
    for _ in 0..TICKS {
        world.tick();
        if row(&world, change).questions.open[0].stands == Stands::Asked && world.store.pending.is_empty() {
            break;
        }
    }
    assert_eq!(
        row(&world, change).questions.open[0].stands,
        Stands::Asked,
        "answers={:?}; requester inbox={}; requester turn={}; change phase={:?}; change state={:?}",
        world.answers,
        live_task(&world, requester.task).inbox.len(),
        live_task(&world, requester.task).turn,
        live_task(&world, change).phase,
        row(&world, change).change.state
    );
    assert_eq!(
        row(&world, change).delegate,
        Some((producer.task, temper_engine_domain_forge_change::Delegate::Produce))
    );
    assert_eq!(
        world.assigned.iter().filter(|run| live_task(&world, run.task).requester == tasks::Party::Task(change)).count(),
        1
    );
    answer(&mut world, &requester, change, CALL + 2);
    wait_answer(&mut world, producer.task, change);
}

#[test]
fn a_retried_step_relays_the_question_once() {
    let (mut world, requester, change, producer) = up_world(None);
    world.restore_delegate_question(producer.task, QUESTION);
    let mut cut = false;
    for _ in 0..TICKS {
        if world.tick_losing_reply_commit(requester.task) {
            cut = true;
            break;
        }
    }
    assert!(cut, "lose the acknowledgement of the atomic relay step");
    world.restart(true, true);
    for _ in 0..TICKS {
        world.tick();
    }
    assert_eq!(row(&world, change).questions.open.len(), 1);
    assert_eq!(row(&world, change).questions.open[0].stands, Stands::Asked);
    let words = replies(&world, requester.task, change);
    assert_eq!(words.len(), 1);
    assert_eq!(words[0].kind, tasks::MessageKind::Question);
    assert_eq!(words[0].words.as_ref(), QUESTION);
}
