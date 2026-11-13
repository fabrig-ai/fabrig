//! Explicit no-change completion and production failures in the engine domain world.
use super::questions::row as change_row;
use super::*;
use jig_domain_core::reads;
use jig_domain_link::EndpointBound;
use temper_engine_domain_forge_change::{Hold, State};

const CHANNEL: Token = Token::new(7);
const STEPS: u32 = 200;
const REASON: &[u8] = b"the repository has no search UI: it is a table-printing command-line program; no files changed";

pub(super) fn failed_production(reason: &[u8]) -> (World, engine::Assignment, u64) {
    // The person decides the held change while the chat reports it and waits.
    production(tasks::TaskResult::Failure { reason: reason.into() }, people::EndedBy::Requester)
}

fn production(result: tasks::TaskResult, ended_by: people::EndedBy) -> (World, engine::Assignment, u64) {
    let mut world = World::configured_limits(
        true,
        true,
        false,
        1000,
        Gate::default(),
        None,
        StoryLimits { result_bytes: Some(tasks::HOLD_ACCOUNT_BYTES), ..StoryLimits::default() },
    );
    world.adopt();
    world.send(protocol::hello_slots(CHANNEL, 2, Box::new([])));
    let people::Ask::Create { project, mut batch } = world.chat(1, b"Fix widget's empty search state") else {
        panic!("chat creation fixture")
    };
    for member in &mut batch {
        member.flags.ended_by = ended_by;
    }
    let ask = people::Ask::Create { project, batch };
    let chat = assigned_chat(&mut world, 92, ask);
    let engine::CallAnswer::Delegated(made) =
        super::change_tool::change(&mut world, &chat, 93, None, b"Fix widget's empty search state")
    else {
        panic!("change delegated")
    };
    let producer = super::reviews::next_assignment(&mut world, 1);
    let event = world.protocol.answer(CHANNEL, chat.task, chat.attempt, 1, tasks::End::Parked, engine::Parts::none());
    world.send(event);
    let event = world.protocol.answer(
        CHANNEL,
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished { result, cancel_delegates: false },
        engine::Parts::none(),
    );
    world.send(event);
    let change = made[0].task;
    for _ in 0..STEPS {
        world.tick();
        if world.store.rows.contains_key(&Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(change))))
            || matches!(live_task(&world, change).phase, tasks::Phase::Held { .. })
        {
            return (world, chat, change);
        }
    }
    panic!("production did not settle its change");
}

#[test]
fn an_inapplicable_change_finishes_without_a_hold() {
    let (mut world, chat, change) =
        production(tasks::TaskResult::NoChange { words: REASON.into() }, people::EndedBy::Result);
    assert_no_change(&world, change);
    let resumed = super::reviews::next_assignment(&mut world, 2);
    assert_eq!(resumed.task, chat.task);
    let notice = resumed
        .messages
        .iter()
        .find(|word| {
            word.from == tasks::Party::Task(change)
                && word.kind == tasks::MessageKind::Result(tasks::ResultKind::NoChange)
        })
        .expect("requester hears the successful no-change result");
    assert_eq!(notice.words.as_ref(), REASON);
    let read = resumed.messages.iter().map(|word| word.number).max();
    let reply = b"No changes were made: the repository has no search UI.";
    world.send(protocol::turn(CHANNEL, resumed.task, resumed.attempt, 1, 1, read, reply));
    for _ in 0..STEPS {
        world.tick();
    }
    let reads::Answer::Ended(past) =
        world.party_read(reads::Read::Turn { task: chat.task, attempt: resumed.attempt, turn: 1 })
    else {
        panic!("person reads the reply")
    };
    let reads::EndedValue::Turn(turn) = past.value else { panic!("chat turn") };
    assert_eq!(turn.transcript.as_ref(), reply);
}

fn assert_no_change(world: &World, change: u64) {
    let key = Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(change)));
    let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(row)))) = world.store.rows.get(&key)
    else {
        panic!("no-change task ended")
    };
    assert!(matches!(&row.phase, tasks::Phase::Ended(tasks::Ending::Done(tasks::TaskResult::NoChange { words }))
        if words.as_ref() == REASON));
    assert!(!world.store.rows.values().any(|row| matches!(row,
        Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task)))
        if matches!(task.escalation, tasks::Escalation::Waiting { .. }))));
    let Some(Record::Forge { row }) = world.store.rows.get(&Key::Forge(forge_top::Key::Change(change))) else {
        panic!("durable change")
    };
    let forge_top::Stored::Change(row) = row.as_ref() else { panic!("durable change") };
    assert!(row.pull.is_none());
    assert_eq!(world.merges, 0);
}

// temper/docs/design/domain/agent/tools.md, 1: a successful no-change result needs a chat reply.
const NO_CHANGE_REPORT: &str = "For successful NoChange, tell the person nothing changed and why from the delegate's conclusion; claim no landing, implementation or tests.";

fn stock_search_chat() -> (World, engine::Assignment) {
    let model = stock::box_model();
    let llm = jig_protocol_config::Llm {
        models: (Box::new([model.clone()]))
            .into_iter()
            .enumerate()
            .map(|(at, model)| jig_domain_core::settings::CatalogueModel {
                first_read: None,
                number: u32::try_from(at).expect("bounded fixture models") + 1,
                model,
                shown: None,
                description: None,
                picker: true,
                withdrawn: false,
                source: jig_domain_core::settings::ModelSource::Stock,
                estimated: false,
            })
            .collect(),

        providers: Box::default(),
        endpoints: Box::default(),
    };
    let limits = temper_engine_domain_world::forge_routes::box_limits();
    let defaults = temper_engine_service_stock::stock::with_models(llm.models.clone(), limits.tasks.result_bytes);
    let mut world = World::configured_stock_limits(&defaults, &model, &limits);
    world.adopt();
    let mut event = protocol::hello_slots(CHANNEL, 2, Box::default());
    let engine::Event::Worker(engine::WorkerInput::Link { link: engine::Link::Hello { hello, .. }, .. }) = &mut event
    else {
        panic!("scripted worker hello")
    };
    hello.bounds.turns = 800;
    hello.bounds.time = Duration::from_secs(28_800);
    hello.bounds.spend = defaults.rules.maximum_run_spend;
    hello.bounds.endpoints =
        Box::new([EndpointBound { endpoint: model.endpoint, window: model.window, output: model.output }]);
    hello.bounds.message_bytes = world.env.limits.tasks.result_bytes.max(world.env.limits.tasks.message_bytes);
    hello.bounds.inbox = world.env.limits.tasks.run_inbox;
    world.send(event);
    let people::Ask::Create { project, mut batch } = world.chat(1, b"Fix widget's empty search state") else {
        panic!("chat request")
    };
    batch[0].executor = people::TaskExecutor::Agent { charter: 1 };
    batch[0].contract = people::Contract::Report { words: limits.tasks.result_bytes };
    batch[0].flags.ended_by = people::EndedBy::Requester;
    let chat = assigned_chat(&mut world, 92, people::Ask::Create { project, batch });
    assert_eq!(chat.charter, 1);
    (world, chat)
}

fn read_search_code(world: &mut World, chat: &engine::Assignment) {
    const CODE: &[u8] = b"fn main() { println!(\"name count\"); }\n";
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", b"main", b"src/main.rs", CODE, 1)
        .expect("table printer");
    let branch = super::change_tool::named_read(world, chat, 93, b"read_branch", br#"{"branch":"main"}"#);
    let digits = engine::commit_text(&translate::commit(head));
    let rendered = temper_engine_smith::answer(&branch);
    assert!(!rendered.error(), "{branch:?}");
    assert!(rendered.text().windows(digits.len()).any(|part| part == digits.as_ref()));
    let input = format!(r#"{{"head":"{}","path":"src/main.rs"}}"#, core::str::from_utf8(&digits).expect("head"));
    let file = super::change_tool::named_read(world, chat, 94, b"read_file", input.as_bytes());
    let rendered = temper_engine_smith::answer(&file);
    assert!(!rendered.error(), "{file:?}");
    assert!(rendered.text().windows(CODE.len()).any(|part| part == CODE));
}

fn person_reply(world: &mut World, chat: &engine::Assignment, reply: &[u8]) {
    let read = chat.messages.iter().map(|word| word.number).max();
    world.send(protocol::turn(CHANNEL, chat.task, chat.attempt, 1, 0, read, reply));
    park_after_turn(world, chat, read, 1);
    for _ in 0..STEPS {
        world.tick();
    }
    let reads::Answer::Ended(past) =
        world.party_read(reads::Read::Turn { task: chat.task, attempt: chat.attempt, turn: 1 })
    else {
        panic!("person reads the chat's reply")
    };
    let reads::EndedValue::Turn(turn) = past.value else { panic!("chat turn") };
    assert_eq!(turn.transcript.as_ref(), reply);
}

#[test]
fn a_stock_chat_reports_its_delegates_no_change_conclusion() {
    let (mut world, chat) = stock_search_chat();
    read_search_code(&mut world, &chat);
    let engine::CallAnswer::Delegated(made) = super::change_tool::change(
        &mut world,
        &chat,
        95,
        None,
        b"Fix widget's empty search state. The code prints a table; it has no search implementation.",
    ) else {
        panic!("change delegated")
    };
    let change = made[0].task;
    let producer = super::reviews::next_assignment(&mut world, 1);
    person_reply(&mut world, &chat, b"Work started.");
    let event = world.protocol.answer(
        CHANNEL,
        producer.task,
        producer.attempt,
        5,
        tasks::End::Finished { result: tasks::TaskResult::NoChange { words: REASON.into() }, cancel_delegates: false },
        engine::Parts::none(),
    );
    world.send(event);
    let resumed = super::reviews::next_assignment(&mut world, 2);
    assert_eq!(resumed.task, chat.task);
    assert!(resumed.attempt > chat.attempt);
    assert_no_change(&world, change);
    let notice = resumed
        .messages
        .iter()
        .find(|word| {
            word.from == tasks::Party::Task(change)
                && word.kind == tasks::MessageKind::Result(tasks::ResultKind::NoChange)
        })
        .expect("successful no-change conclusion wakes the chat");
    assert_eq!(notice.words.as_ref(), REASON);
    let instructions = core::str::from_utf8(&resumed.run.policy.instructions).expect("stock instructions");
    assert!(instructions.contains(NO_CHANGE_REPORT), "stock chat must report successful NoChange to the person");
    assert!(instructions.contains("After calling change, say only that work started until the delegate reports."));
    // The scripted worker relays the delivered conclusion under the assigned stock instructions.
    let reply = format!("Nothing changed: {}", core::str::from_utf8(&notice.words).expect("conclusion"));
    person_reply(&mut world, &resumed, reply.as_bytes());
    assert_eq!(world.assigned.len(), 3, "the chat parks after consuming its result");
    assert!(matches!(live_task(&world, chat.task).phase, tasks::Phase::Active(tasks::Active::Idle)));
    world.close();
}

fn account(world: &World, change: u64) -> &[u8] {
    let tasks::Phase::Held { why: tasks::Hold::EffectFailed, source: Some(source), .. } =
        &live_task(world, change).phase
    else {
        panic!("retained failure hold: {:?}", live_task(world, change).phase)
    };
    let tasks::HoldSource::Account { words, .. } = source.as_ref() else { panic!("producer's account") };
    words
}

#[test]
fn a_failed_producers_words_reach_its_hold_and_its_chats_reply() {
    let (mut world, chat, change) = failed_production(REASON);
    assert_eq!(account(&world, change), REASON);
    let Some(Record::Forge { row }) = world.store.rows.get(&Key::Forge(forge_top::Key::Change(change))) else {
        panic!("durable change")
    };
    let forge_top::Stored::Change(row) = row.as_ref() else { panic!("durable change") };
    assert!(matches!(&row.change.state, State::Held { was, why: Hold::Failed }
        if matches!(was.as_ref(), State::Producing { .. })));
    assert!(row.pull.is_none());
    assert_eq!(world.merges, 0);
    let resumed = super::reviews::next_assignment(&mut world, 2);
    assert_eq!(resumed.task, chat.task);
    let notice = resumed
        .messages
        .iter()
        .find(|word| word.from == tasks::Party::Task(change) && word.kind == tasks::MessageKind::Words)
        .expect("chat hears its change's reason");
    assert_eq!(notice.words.as_ref(), REASON);
    // The scripted worker plays the chat's response to the words it received.
    let reply = format!("I could not make the change: {}", core::str::from_utf8(&notice.words).expect("explanation"));
    let read = resumed.messages.iter().map(|word| word.number).max();
    world.send(protocol::turn(CHANNEL, resumed.task, resumed.attempt, 1, 1, read, reply.as_bytes()));
    for _ in 0..STEPS {
        world.tick();
    }
    let reads::Answer::Ended(past) =
        world.party_read(reads::Read::Turn { task: resumed.task, attempt: resumed.attempt, turn: 1 })
    else {
        panic!("person reads the chat's reply")
    };
    let reads::EndedValue::Turn(turn) = past.value else { panic!("chat turn") };
    assert_eq!(turn.transcript.as_ref(), reply.as_bytes());
    assert!(turn.transcript.windows(REASON.len()).any(|bytes| bytes == REASON));
}

#[test]
fn a_producer_failure_uses_the_hold_account_bound_even_with_a_smaller_message_bound() {
    let bound = usize::try_from(tasks::HOLD_ACCOUNT_BYTES).expect("hold bound");
    let mut reason = vec![b'x'; bound];
    reason[..REASON.len()].copy_from_slice(REASON);
    let (world, chat, change) = failed_production(&reason);
    assert_eq!(world.env.limits.tasks.message_bytes, 128);
    assert_eq!(account(&world, change), reason, "a whole hold account is never cut to the inbox bound");
    let notice = live_task(&world, chat.task)
        .inbox
        .iter()
        .find(|word| word.from == tasks::Party::Task(change) && word.kind == tasks::MessageKind::Words)
        .expect("requester notified");
    assert_eq!(notice.words.len(), usize::try_from(world.env.limits.tasks.message_bytes).expect("message bound"));
    assert!(notice.words.starts_with(REASON));
    assert!(notice.words.ends_with(b"..."));
}

#[test]
fn a_refused_failure_notice_still_holds_and_escalates_the_change() {
    let (mut world, requester) = super::questions::plan_world(None);
    let change = super::questions::make_change(&mut world, &requester, 93, engine::ChangeQuestions::Up);
    let producer = super::questions::next_producer(&mut world, change);
    world.drain_cut();
    super::questions::fill_requester_inbox(&mut world, &requester);
    world.until(Until::Ready);
    super::reviews::finish_review(&mut world, &producer, tasks::TaskResult::Failure { reason: REASON.into() });
    for _ in 0..STEPS {
        world.tick();
        if matches!(live_task(&world, change).escalation, tasks::Escalation::Waiting { .. })
            && world.store.pending.is_empty()
        {
            break;
        }
    }
    assert_eq!(account(&world, change), REASON);
    assert!(matches!(change_row(&world, change).change.state, State::Held { why: Hold::Failed, .. }));
    assert!(
        std::iter::from_fn(|| world.root.pop_fact())
            .any(|fact| matches!(fact, engine::Fact::StepRefused { task, .. } if task == change)),
        "the failure notice's step was refused whole"
    );
    assert!(!live_task(&world, requester.task).inbox.iter().any(|word| word.from == tasks::Party::Task(change)
        && word.kind == tasks::MessageKind::Words
        && word.words.as_ref() == REASON));
    let tasks::Escalation::Waiting { holder, entry, .. } = live_task(&world, change).escalation else {
        panic!("the refused notice cannot lose the escalation")
    };
    assert_eq!(holder, tasks::EscalationHolder::Task(requester.task));
    // The holder's run must read its full offered prefix before more can be delivered.
    world.scripted_read_inbox(&requester);
    for _ in 0..STEPS {
        world.tick();
        if world
            .delivered_messages
            .get(&(requester.task, requester.attempt))
            .is_some_and(|words| words.contains(&entry))
        {
            break;
        }
    }
    assert!(
        world.delivered_messages.get(&(requester.task, requester.attempt)).is_some_and(|words| words.contains(&entry)),
        "the escalation reaches the requester without its Words copy"
    );
    world.close();
}

#[test]
fn a_restored_held_row_holds_its_idle_task_again() {
    let (mut world, _, change) = failed_production(REASON);
    world.drain_cut();
    let mut task = Box::new(live_task(&world, change).clone());
    task.phase = tasks::Phase::Active(tasks::Active::Idle);
    task.escalation = tasks::Escalation::Unheld { revision: task.revisions.hold };
    let mut header = world.store.header();
    header.commits += 1;
    world.store.pending.push_back((
        header.commits,
        Box::new([
            engine::Write::Save(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Live(task)))),
            engine::Write::Save(Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Deployment(
                header,
            )))),
        ]),
    ));
    world.store.apply();
    world.restart(true, true);
    for _ in 0..STEPS {
        world.tick();
        if matches!(live_task(&world, change).phase, tasks::Phase::Held { .. }) && world.store.pending.is_empty() {
            break;
        }
    }
    assert_eq!(account(&world, change), REASON);
    assert!(matches!(change_row(&world, change).change.state, State::Held { why: Hold::Failed, .. }));
    world.close();
}

fn refresh(world: &mut World, task: u64) {
    let branch = change_row(world, task).branch.clone();
    world.send(engine::Event::Forge(Box::new(forge_top::Event::Hint {
        hint: skein_forge::api::Hint {
            repository: forge_world::REPO,
            change: skein_forge::api::Change::Branch(branch),
            key: None,
        },
    })));
}

fn pending_reads(world: &mut World, task: u64) -> Vec<engine::Event> {
    refresh(world, task);
    let mut held = Vec::new();
    for _ in 0..STEPS {
        while let Some(index) = world.events.iter().position(|event| {
            matches!(event,
            engine::Event::Forge(event) if matches!(event.as_ref(),
                forge_top::Event::Client(skein_forge_client::Event::Answered { .. })))
        }) {
            held.push(world.events.remove(index).expect("selected forge read"));
        }
        if !held.is_empty()
            && world.root.quiescent()
            && world.store.pending.is_empty()
            && matches!(
                live_task(world, task).phase,
                tasks::Phase::Active(tasks::Active::Due) | tasks::Phase::Held { .. }
            )
        {
            return held;
        }
        world.tick();
    }
    panic!("forge reads did not pause");
}

fn answer_reads(world: &mut World, reads: Vec<engine::Event>) {
    for read in reads {
        world.send(read);
    }
}

fn producing_goal() -> (World, engine::Assignment, engine::Assignment, u64) {
    let mut world = World::configured_limits(
        true,
        true,
        false,
        1000,
        Gate::default(),
        None,
        StoryLimits { result_bytes: Some(tasks::HOLD_ACCOUNT_BYTES), ..StoryLimits::default() },
    );
    world.adopt();
    world.send(protocol::hello_slots(CHANNEL, 2, Box::new([])));
    let ask = super::goal_branch::goal_request(&world, true);
    let goal = assigned_chat(&mut world, 92, ask);
    let mut call = engine::ChangeCall::words(b"Small fix".as_slice().into(), None);
    call.producer = Some(2);
    call.landing = Some(engine::ChangeLanding::Goal);
    call.llm.producer = Some(tasks::LlmChoice {
        later: None,
        account: None,
        main: tasks::ModelEffort { model: 0, effort: None },
        subagents: Box::new([]),
    });
    let engine::CallAnswer::Delegated(made) =
        world.scripted_call(&goal, 93, engine::Tool::Change { changes: Box::new([call]) })
    else {
        panic!("goal's change admitted")
    };
    let producer = super::reviews::next_assignment(&mut world, 1);
    let event = world.protocol.answer(CHANNEL, goal.task, goal.attempt, 1, tasks::End::Parked, engine::Parts::none());
    world.send(event);
    world.until(Until::Ready);
    (world, goal, producer, made[0].task)
}

fn assert_production_hold(world: &mut World, task: u64, reason: &[u8]) {
    for _ in 0..STEPS {
        world.tick();
        if matches!(live_task(world, task).phase, tasks::Phase::Held { .. }) {
            assert_eq!(account(world, task), reason);
            assert!(matches!(&change_row(world, task).change.state,
                State::Held { was, why: Hold::Failed } if matches!(was.as_ref(), State::Producing { .. })));
            assert!(change_row(world, task).delegate.is_none());
            assert!(change_row(world, task).change.last_head.is_none());
            assert_eq!(world.merges, 0);
            return;
        }
    }
    panic!("producer failure did not hold: {:?}; phase={:?}", change_row(world, task), live_task(world, task).phase);
}

fn release_production(world: &mut World, task: u64) {
    let revision = live_task(world, task).revisions.hold;
    world.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(950)),
        sign_in: world.signed_in.expect("owner signed in"),
        key: [95; 16],
        ask: people::Ask::Release { project: 1, task, revision, again: None },
    }));
}

fn replacement_lands(world: &mut World, task: u64, first: &engine::Assignment) {
    let fresh = world
        .assigned
        .iter()
        .find(|run| run.task != first.task && live_task(world, run.task).requester == tasks::Party::Task(task))
        .expect("fresh producer")
        .clone();
    assert_eq!(fresh.charter, first.charter);
    assert_eq!(fresh.charter, 2, "the chosen charter survives release");
    assert_eq!(fresh.run.policy.model, first.run.policy.model);
    let ended = Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(first.task)));
    let Some(Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Ended(prior)))) = world.store.rows.get(&ended)
    else {
        panic!("failed producer retained")
    };
    assert!(prior.llm.is_some(), "explicit producer LLM choice");
    assert_eq!(live_task(world, fresh.task).llm, prior.llm);
    assert_eq!(live_task(world, fresh.task).spec, prior.spec);
    assert_eq!(live_task(world, task).made, 3, "change and exactly two producers");
    let branch = change_row(world, task).branch.clone();
    super::reviews::produce(world, &fresh, &branch);
    for _ in 0..STEPS {
        world.tick();
        if matches!(change_row(world, task).change.state, State::Landed { .. }) {
            assert_eq!(world.merges, 1);
            assert_eq!(live_task(world, task).made, 3);
            return;
        }
    }
    panic!("replacement did not land: {:?}", change_row(world, task));
}

fn finish_failed_producer(world: &mut World, first: &engine::Assignment, task: u64, reason: &[u8]) {
    let event = world.protocol.answer(
        CHANNEL,
        first.task,
        first.attempt,
        5,
        tasks::End::Finished { result: tasks::TaskResult::Failure { reason: reason.into() }, cancel_delegates: false },
        engine::Parts::none(),
    );
    world.send(event);
    let ended = Key::Core(jig_domain_core::Key::Tasks(tasks::Key::Ended(first.task)));
    for _ in 0..STEPS {
        if world.store.rows.contains_key(&ended) {
            break;
        }
        world.tick();
    }
    assert!(world.store.rows.contains_key(&ended), "failed worker releases its writer");
    // The normalized terminal and an already-started read may overlap.
    producer_terminal(world, task, first.task, reason);
}

fn producer_terminal(world: &mut World, task: u64, child: u64, words: &[u8]) {
    world.send(engine::Event::Forge(Box::new(forge_top::Event::DelegateResult {
        task,
        child,
        status: temper_engine_domain_forge_change::Status::Failed,
        no_change: false,
        failed: true,
        cancelled: false,
        words: words.into(),
    })));
}

fn resume_production(world: &mut World, task: u64, first: u64, reason: &[u8]) {
    let reads = pending_reads(world, task);
    release_production(world, task);
    refresh(world, task);
    answer_reads(world, reads);
    for _ in 0..STEPS {
        world.tick();
        if change_row(world, task).delegate.is_some() {
            break;
        }
    }
    assert!(change_row(world, task).delegate.is_some(), "release admits a replacement");
    let reads = pending_reads(world, task);
    producer_terminal(world, task, first, reason);
    refresh(world, task);
    answer_reads(world, reads);
    for _ in 0..20 {
        world.tick();
    }
}

#[test]
fn a_failed_producer_holds_notifies_and_restarts_on_release() {
    // The finding records these causes, but supplies no model transcript.
    for reason in [b"out of turns".as_slice(), b"missing cargo".as_slice(), b"".as_slice()] {
        let words = if reason.is_empty() { b"producer failed for good without an account".as_slice() } else { reason };
        let (mut world, goal, first, task) = producing_goal();
        let reads = pending_reads(&mut world, task);
        finish_failed_producer(&mut world, &first, task, reason);
        answer_reads(&mut world, reads);
        assert_production_hold(&mut world, task, words);
        let resumed = super::reviews::next_assignment(&mut world, 2);
        assert_eq!(resumed.task, goal.task);
        assert!(resumed.messages.iter().any(|word| word.from == tasks::Party::Task(task)
            && word.kind == tasks::MessageKind::Words
            && word.words.as_ref() == words));
        resume_production(&mut world, task, first.task, reason);
        replacement_lands(&mut world, task, &first);
        world.close();
    }
}

const PUSHED_FAILURE: &[u8] = b"out of turns after pushing partial work";

fn pushed_failure() -> (World, engine::Assignment, u64, u64) {
    pushed_failure_with_message(b"keep this work")
}

fn pushed_failure_with_message(message: &[u8]) -> (World, engine::Assignment, u64, u64) {
    let (mut world, _, first, task) = producing_goal();
    let branch = change_row(&world, task).branch.clone();
    world.external(2, raw::Op::Git(raw::Git::Create { branch: branch.clone(), commit: 1 }));
    let head = fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"partial", message, 1)
        .expect("producer pushes partial work");
    // Read the pushed branch before the terminal result reaches the connector.
    refresh(&mut world, task);
    for _ in 0..20 {
        world.tick();
    }
    let reads = pending_reads(&mut world, task);
    finish_failed_producer(&mut world, &first, task, PUSHED_FAILURE);
    answer_reads(&mut world, reads);
    refresh(&mut world, task);
    assert_production_hold(&mut world, task, PUSHED_FAILURE);
    (world, first, task, head)
}

#[test]
fn a_producer_that_delivers_then_fails_holds_at_producing() {
    let (mut world, _, task, head) = pushed_failure();
    let row = change_row(&world, task);
    assert!(row.pull.is_none());
    assert!(row.title.is_empty());
    let branch = row.branch.clone();
    assert_eq!(world.external(1, raw::Op::Read(raw::Read::Branch { branch })), raw::Answer::Commit(head));
    assert!(!world.writes.iter().any(|write| matches!(write, skein_forge::api::Write::OpenPull { .. })));
    world.close();
}

#[test]
fn a_released_failed_producer_with_a_head_makes_a_fresh_one() {
    let (mut world, first, task, _) = pushed_failure();
    resume_production(&mut world, task, first.task, PUSHED_FAILURE);
    for _ in 0..STEPS {
        world.tick();
        if world
            .assigned
            .iter()
            .any(|run| run.task != first.task && live_task(&world, run.task).requester == tasks::Party::Task(task))
        {
            break;
        }
    }
    let fresh = world
        .assigned
        .iter()
        .find(|run| run.task != first.task && live_task(&world, run.task).requester == tasks::Party::Task(task))
        .expect("fresh producer assigned")
        .clone();
    let branch = change_row(&world, task).branch.clone();
    assert_eq!(fresh.workspace.repositories[0].start, engine::ForgeStart::Branch(branch.clone()));
    assert_eq!(live_task(&world, task).made, 3, "change and exactly two producers");
    fake::advance(&mut world.fake, &world.fake_env, b"org/repo", &branch, b"finished", b"complete", 1)
        .expect("replacement pushes atop partial work");
    let title = b"Finish the partial update";
    let event = world.protocol.answer(
        CHANNEL,
        fresh.task,
        fresh.attempt,
        5,
        tasks::End::Finished {
            result: tasks::TaskResult::Change {
                title: tasks::ChangeTitle::new(title.as_slice().into()),
                connector: 0,
                kind: 2,
                resource: forge_top::tag(forge_world::REPO.forge, forge_world::REPO.repository),
                words: b"finished".as_slice().into(),
            },
            cancel_delegates: false,
        },
        engine::Parts::none(),
    );
    world.send(event);
    for _ in 0..STEPS {
        world.tick();
        if change_row(&world, task).pull.is_some() {
            break;
        }
    }
    assert!(change_row(&world, task).pull.is_some(), "replacement opens the pull");
    let opened: Vec<_> = world
        .writes
        .iter()
        .filter_map(|write| match write {
            skein_forge::api::Write::OpenPull { title, .. } => Some(title.as_ref()),
            _ => None,
        })
        .collect();
    assert_eq!(opened, [title.as_slice()]);
    assert_eq!(live_task(&world, task).made, 3);
    world.close();
}

#[test]
fn an_unchecked_producer_head_is_retained_without_a_merge_or_pull_request() {
    let (mut world, _, task, head) =
        pushed_failure_with_message(b"Unchecked: work in progress, saved before a question");
    let row = change_row(&world, task);
    assert!(row.pull.is_none());
    assert!(row.title.is_empty());
    let branch = row.branch.clone();
    assert_eq!(world.external(1, raw::Op::Read(raw::Read::Branch { branch })), raw::Answer::Commit(head));
    assert_eq!(world.merges, 0);
    assert!(!world.writes.iter().any(|write| matches!(write, skein_forge::api::Write::OpenPull { .. })));
    world.close();
}
