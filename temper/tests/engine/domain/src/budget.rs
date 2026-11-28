//! Budget questions and effort changes, without a forge (testing/reference-case.md, checks.md step 5).
use crate::{direct::Driver, fixtures, protocol, store::Store, walking};
use jig_domain_core::{documents, reads};
use jig_domain_people as people;
use jig_domain_tasks as tasks;
use skein_lib::{ReplyTo, Token, Wall};
use temper_engine_domain as engine;

pub const CHANNEL: Token = Token::new(7);
pub const QUESTION: &[u8] = b"May the story take 70 seconds against the 60 second budget?";
pub const UP: &[u8] = b"May my investigation's story exceed the budget?";
pub const ANSWER: &[u8] = b"No raise; make it cheaper.";
pub const REASON: &[u8] = b"Try the investigation again at high effort.";
pub const FINDING: &[u8] = b"No cause found at medium effort.";
pub const REPORT: &[u8] = b"No cause found, including at high effort.";
pub const ADVICE: &[u8] = b"The repeated setup is the cause.";
pub const A_DEADLINE: Wall = Wall::from_nanos(900_000_000_000);
pub const A1_DEADLINE: Wall = Wall::from_nanos(800_000_000_000);
// Director, two owners, investigator and advisor; four are live at once.
const TASKS: u32 = 5;

/// Keep the caller's model transport, adding medium/high and an owner's model.
#[must_use]
pub fn configured(mut configs: [engine::Config; 2]) -> (Driver, engine::Assignment) {
    let mut bounds = walking::limits();
    bounds.tasks.tasks = TASKS;
    bounds.tasks.tree_tasks = TASKS;
    bounds.tasks.project_tasks = TASKS;
    bounds.tasks.depth = 4;
    bounds.tasks.delegates = 3;
    bounds.tasks.inbox_messages = 16;
    bounds.tasks.inbox_bytes = 4096;
    bounds.people.inbox_tasks = TASKS * 2;
    bounds.people.inbox_recipients = bounds.tasks.inbox_messages * 2 + bounds.authority.roles + 4;
    bounds.people.requests = 8;
    bounds.people.amendment_bytes = 4096;
    bounds.fleet.slots = 4;
    bounds.fleet.attempts = 8;
    bounds.call_records = 8;
    bounds.forge.named_calls = 8;
    bounds.brief.sections = 5;
    bounds.journal.writes = crate::journal_limits::WALKING_WRITES;
    bounds.journal.transcript_bytes = 131_072;
    for config in &mut configs {
        // These holders isolate delivery in fresh runs, not transcript-tail rendering.
        config.core.settings.tail_bytes = 0;
        let charter = &mut config.core.defaults.charters[0];
        charter.policy.resume = false;
        charter.policy.model.price_unit = 1_000_000;
        charter.policy.model.efforts = Box::new([b"medium".as_slice().into(), b"high".as_slice().into()]);
        charter.policy.model.effort = Some(b"medium".as_slice().into());
        let mut owner = charter.policy.model.clone();
        owner.name = b"owner".as_slice().into();
        charter.policy.alternatives = Box::new([owner.clone()]);
        config.core.settings.models = Box::new([charter.policy.model.clone(), owner]);
        config.core.defaults.rules.maximum_run_spend = 5;
        for authority in [
            &mut config.core.defaults.rules.ceiling,
            &mut config.core.seed_projects[0].policy.policy.ceiling,
            &mut config.core.seed_projects[0].policy.policy.roles[0].authority,
        ] {
            authority.delegation.depth = 4;
            authority.delegation.tasks = TASKS;
            authority.models = jig_domain_authority::Models::ALL;
        }
    }
    let model = &configs[0].core.defaults.charters[0].policy.model;
    let mut hosting = protocol::bounds();
    hosting.endpoints = Box::new([jig_domain_link::EndpointBound {
        endpoint: model.endpoint,
        window: model.window,
        output: model.output,
    }]);
    hosting.turns = configs[0].core.defaults.charters[0].policy.turns;
    hosting.time = configs[0].core.defaults.charters[0].policy.time;
    hosting.checks = configs[0].core.defaults.charters[0].policy.reserve_time;
    let mut driver = Driver::configured(Store::new(), configs, &bounds);
    driver.settle();
    driver.send(protocol::hello_with(CHANNEL, 0, 4, hosting, Box::new([])));
    driver.settle();
    driver.sign_in();
    driver.settle();
    let mut ask = driver.chat_ask(1, b"Direct the budget investigation.");
    let people::Ask::Create { batch, .. } = &mut ask else { panic!("chat") };
    batch[0].flags.ended_by = people::EndedBy::Result;
    ask_person(&mut driver, 31, ask);
    let director = driver.last_assignment();
    (driver, director)
}

pub fn ask_person(driver: &mut Driver, key: u8, ask: people::Ask) {
    driver.send(engine::Event::Party(engine::PartyInput::Ask {
        models: Box::new([]),
        reply_to: ReplyTo::new(Token::new(u64::from(key))),
        sign_in: driver.session(),
        key: [key; 16],
        ask,
    }));
    driver.settle();
}

#[must_use]
pub fn choice(model: u8, effort: u8) -> tasks::LlmChoice {
    tasks::LlmChoice {
        account: None,
        main: tasks::ModelEffort { model, effort: Some(effort), speed: None },
        subagents: Box::new([]),
        later: None,
    }
}

#[must_use]
pub fn delegate(words: &[u8], spend: u64, depth: u32) -> engine::Delegate {
    let mut member = fixtures::report_delegate(words, Box::new([]));
    member.authority.budget.spend = spend;
    member.authority.models = tasks::Models::Named(3);
    member.authority.delegation = tasks::Delegation {
        kinds: if depth == 0 { Box::new([]) } else { Box::new([tasks::AuthorityExecutor::Charter(1)]) },
        tasks: if depth == 0 { 0 } else { depth + 1 },
        depth,
    };
    member
}

pub fn make(
    driver: &mut Driver,
    owner: &engine::Assignment,
    call: u64,
    member: engine::Delegate,
) -> engine::Assignment {
    fixtures::tool_call(driver, owner, call, engine::Tool::Delegate { batch: Box::new([member]) });
    let Some(engine::CallAnswer::Delegated(made)) = driver.answered(call) else {
        panic!("delegate: {:?}", driver.answered(call))
    };
    latest(driver, made[0].task)
}

/// A tracked owner with a shorter-lived subowner who can delegate one level.
pub fn owners(driver: &mut Driver, director: &engine::Assignment) -> [engine::Assignment; 2] {
    let mut a = delegate(b"Own the work.", 100, 2);
    a.tracked = Some(1);
    a.authority.budget.deadline = Some(A_DEADLINE);
    let a = make(driver, director, 101, a);
    let mut a1 = delegate(b"Own the investigation.", 60, 1);
    a1.authority.budget.deadline = Some(A1_DEADLINE);
    let a1 = make(driver, &a, 102, a1);
    [a, a1]
}

#[must_use]
pub fn latest(driver: &Driver, task: u64) -> engine::Assignment {
    driver
        .delivered
        .iter()
        .rev()
        .find_map(|d| match d {
            engine::Delivery::Assigned { assignment, .. } if assignment.task == task => Some(assignment.clone()),
            _ => None,
        })
        .expect("task assigned")
}

pub fn end(driver: &mut Driver, run: &engine::Assignment, end: tasks::End) {
    driver.answer(CHANNEL, run.task, run.attempt, 0, end, engine::Parts::none());
    driver.settle();
}

pub fn report(driver: &mut Driver, run: &engine::Assignment, words: &[u8]) {
    end(
        driver,
        run,
        tasks::End::Finished { result: tasks::TaskResult::Report { words: words.into() }, cancel_delegates: false },
    );
}

pub fn message(
    driver: &mut Driver,
    run: &engine::Assignment,
    target: u64,
    call: u64,
    form: engine::MessageForm,
    words: &[u8],
) -> u64 {
    fixtures::tool_call(driver, run, call, engine::Tool::Message { target, form, words: words.into() });
    let Some(engine::CallAnswer::Sent { message }) = driver.answered(call) else {
        panic!("message: {:?}", driver.answered(call))
    };
    *message
}

pub fn read(driver: &mut Driver, query: reads::Read) -> reads::Answer {
    driver.send(engine::Event::Party(engine::PartyInput::Read {
        reply_to: ReplyTo::new(Token::new(900)),
        sign_in: driver.session(),
        read: Box::new(query),
    }));
    driver.settle();
    driver
        .delivered
        .iter()
        .rev()
        .find_map(|d| match d {
            engine::Delivery::Read { answer, .. } => Some(answer.clone()),
            _ => None,
        })
        .expect("read answered")
}

pub fn document(driver: &mut Driver, task: u64) -> Box<documents::Task> {
    let answer = read(driver, reads::Read::Task { task });
    match answer {
        reads::Answer::Document(doc) => {
            let documents::Document::Task(doc) = *doc else { panic!("task document") };
            doc
        }
        reads::Answer::Ended(ended) => {
            let reads::EndedValue::Task { document, .. } = ended.value else { panic!("ended task document") };
            document
        }
        other => panic!("task {task} document: {other:?}"),
    }
}

/// Every admitted attempt has one terminal, and no failure is hidden by a retry.
pub fn healthy(driver: &Driver) {
    for delivery in &driver.delivered {
        if let engine::Delivery::Assigned { assignment, .. } = delivery {
            let terminal: Vec<_> =
                driver
                    .store
                    .rows
                    .values()
                    .filter_map(|row| match row {
                        engine::Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Terminal(
                            t,
                        ))) if t.task == assignment.task && t.attempt == assignment.attempt => Some(t),
                        _ => None,
                    })
                    .collect();
            let mut positions: Vec<_> = driver
                .store
                .rows
                .values()
                .filter_map(|row| match row {
                    engine::Record::Core(jig_domain_core::Record::Tasks(tasks::Stored::Activity(line)))
                        if line.task == assignment.task =>
                    {
                        Some(line.position)
                    }
                    _ => None,
                })
                .collect();
            positions.sort_unstable();
            assert_eq!(positions.first(), Some(&1), "journal starts at one");
            assert!(positions.windows(2).all(|pair| pair[0] + 1 == pair[1]), "whole ordered journal");
            assert_eq!(terminal.len(), 1, "task {} attempt {}", assignment.task, assignment.attempt);
            assert!(matches!(terminal[0].end, tasks::End::Parked | tasks::End::Finished { .. }), "{:?}", terminal[0]);
        }
    }
}
