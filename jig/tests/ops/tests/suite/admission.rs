//! Admitted continuations and mixed facts at the root's watermark (domain/root.md, 3).
use jig_domain_core as core;
use jig_domain_fleet as fleet;
use jig_domain_host as host;
use jig_domain_tasks as tasks;
use jig_ops_domain as root;
use jig_ops_domain_infrastructure as infra;
use jig_ops_domain_observability as obs;
use jig_ops_world::direct::Driver;
use skein_lib::{Duration, ReplyTo, Token};
use std::collections::BTreeSet;

const SEED: u64 = 9205;
const TASK: u64 = 7;
const FIRST: Token = Token::new(960);
const SECOND: Token = Token::new(961);
const THIRD: Token = Token::new(962);
const READ_BYTES: u32 = 128;
const ONCE: usize = 1;
const ONE_REFUSAL: u64 = 1;
// One activation u64, one completion u32 and one position u32.
const CALL_NAME_BYTES: usize = 16;
// A held head, the mixed remainder and its following completion.
const HELD_INPUTS: usize = 3;
const CHAT: u64 = 1;

fn new_work() -> root::Event {
    root::Event::Observability(obs::Event::Subscribe {
        task: TASK,
        topic: obs::Topic::Alerts(jig_ops_world::service()),
        wake_at: 5,
        keep_at: 1,
    })
}

fn read(token: Token) -> root::Event {
    root::Event::Observability(obs::Event::Read {
        token,
        read: obs::Read::Logs {
            service: jig_ops_world::service(),
            window: obs::Window { from: 0, through: 1 },
            filter: Box::new([]),
            max_bytes: READ_BYTES,
        },
    })
}

fn read_done(token: Token, bytes: &[u8]) -> root::Event {
    root::Event::Observability(obs::Event::System(obs::SystemEvent::ReadDone { token, bytes: bytes.into() }))
}

fn replies(driver: &Driver) -> Vec<(Token, &[u8])> {
    driver
        .released
        .iter()
        .filter_map(|output| {
            if let root::Output::Read { owner, bytes } = output { Some((*owner, bytes.as_ref())) } else { None }
        })
        .collect()
}

fn health_fact(replicas: u32) -> root::Event {
    root::Event::Observability(obs::Event::System(obs::SystemEvent::Fact {
        service: jig_ops_world::service(),
        fact: obs::Fact {
            observed: 0,
            healthy_replicas: replicas,
            errors: 0,
            error_rate_percent: 0,
            load_percent: 10,
            load: Box::new([]),
        },
    }))
}

fn service_fact(revision: u64) -> root::Event {
    root::Event::Infrastructure(infra::Event::System(infra::SystemEvent::Service {
        service: jig_ops_world::infra_service(),
        fact: infra::ServiceFact {
            version: b"v1".as_slice().into(),
            replicas: 2,
            healthy: true,
            revision,
            observed: 0,
        },
        other_hand: true,
    }))
}

fn absence() -> root::Event {
    root::Event::Infrastructure(infra::Event::System(infra::SystemEvent::Environment {
        environment: jig_ops_world::infra_environment(),
        fact: None,
        other_hand: true,
    }))
}

fn drifts(driver: &Driver) -> usize {
    driver
        .transactions
        .iter()
        .flat_map(|(_, writes)| writes)
        .filter(|write| matches!(write, root::Write::Save(root::Record::Infrastructure(infra::Record::Drift(_)))))
        .count()
}

fn scale_driver() -> Driver {
    let mut driver = Driver::started(SEED);
    driver.send(service_fact(1));
    driver.send(root::Event::Infrastructure(infra::Event::Names {
        task: TASK,
        project: 1,
        resources: Box::new([infra::Resource::Service(jig_ops_world::infra_service())]),
    }));
    driver.send(root::Event::Infrastructure(infra::Event::Restore {
        record: infra::Record::Procedure(infra::ProcedureState {
            task: TASK,
            chat: None,
            phase: infra::ProcedurePhase::ReadingFacts,
            procedure: infra::Procedure::Scale { service: jig_ops_world::infra_service(), replicas: 3, deadline: 100 },
        }),
    }));
    driver.settle();
    driver.transactions.clear();
    driver
}

fn ready_readers(driver: &Driver) -> usize {
    driver
        .transactions
        .iter()
        .flat_map(|(_, writes)| writes)
        .filter(|write| {
            matches!(write,
        root::Write::Save(root::Record::Infrastructure(infra::Record::Procedure(state)))
            if state.phase == infra::ProcedurePhase::FactsReady)
        })
        .count()
}

fn health_messages(driver: &Driver) -> BTreeSet<u64> {
    let mut messages = BTreeSet::new();
    for (_, writes) in &driver.transactions {
        for write in writes {
            if let root::Write::Save(root::Record::Core(core::Record::Tasks(tasks::Stored::Live(row)))) = write {
                for word in &row.inbox {
                    if matches!(word.kind, tasks::MessageKind::News { subscription: 1, .. }) {
                        messages.insert(word.number);
                    }
                }
            }
        }
    }
    messages
}

fn pressured_chat() -> Driver {
    let mut driver = Driver::started(SEED);
    driver.chat();
    driver.fill_to_watermark();
    driver
}

fn terminal_count(driver: &Driver) -> usize {
    driver
        .transactions
        .iter()
        .flat_map(|(_, writes)| writes)
        .filter(|write| {
            matches!(write, root::Write::Save(root::Record::Core(core::Record::Core(core::CoreRecord::Terminal(_)))))
        })
        .count()
}

fn worker_call(owner: Token, writes: bool) -> root::Event {
    let mut name = [0; CALL_NAME_BYTES];
    name[..8].copy_from_slice(&1_u64.to_be_bytes());
    name[8..12].copy_from_slice(&1_u32.to_be_bytes());
    root::Event::Agent(host::FromAgent::Called {
        owner,
        call: Box::new(name),
        ask: host::Ask::Relay {
            tool: b"subscribe".as_slice().into(),
            writes,
            input: Box::new([]),
            deadline: Duration::from_secs(30),
        },
    })
}

#[test]
fn admitted_work_is_taken_at_the_watermark() {
    let mut driver = pressured_chat();
    let returned = driver.returned;
    driver.send(root::Event::AgentContent {
        owner: driver.owner,
        content: root::AgentContent::Text { turn: 1, retry: 0, offset: 0, text: b"partial".as_slice().into() },
    });
    driver.send(root::Event::Agent(host::FromAgent::Finished {
        owner: driver.owner,
        turns: 0,
        spent: 0,
        read: None,
        finish: host::Finish::Parked,
    }));
    driver.send(root::Event::Agent(host::FromAgent::Gone { owner: driver.owner, detail: Box::new([]) }));
    driver.send(root::Event::InfrastructureTimer);
    driver.send(root::Event::TurnTimer);
    assert_eq!(driver.returned, returned, "records and retries of admitted work enter at the watermark");
    driver.settle();
    assert_eq!(terminal_count(&driver), ONCE);
}

#[test]
fn a_health_fact_settles_under_load_and_its_news_waits() {
    let mut driver = Driver::started(SEED);
    driver.chat();
    driver.send(root::Event::Core(core::Event::Tasks(tasks::Event::SubscribeTopic {
        reply_to: ReplyTo::new(FIRST),
        task: CHAT,
        subscription: tasks::Subscription {
            number: 1,
            kind: tasks::SubscriptionKind::Topic { connector: 1, topic: 1 },
        },
    })));
    driver.send(root::Event::Observability(obs::Event::Subscribe {
        task: CHAT,
        topic: obs::Topic::Health(jig_ops_world::service()),
        wake_at: 5,
        keep_at: 1,
    }));
    driver.send(health_fact(2));
    driver.settle();
    driver.fill_to_watermark();
    let rest = root::try_step(&mut driver.domain, &driver.env, health_fact(1)).expect_err("health news waits");
    assert!(
        !matches!(rest, root::Event::Observability(obs::Event::System(obs::SystemEvent::Fact { .. }))),
        "the admitted fact must never be replayed"
    );
    driver.collect();
    assert!(health_messages(&driver).is_empty(), "settlement sends no health news");
    driver.settle();
    assert!(health_messages(&driver).is_empty());
    driver.send(rest);
    driver.settle();
    assert_eq!(health_messages(&driver).len(), ONCE, "the health rest sends one durable news message");
}

#[test]
fn a_service_fact_settles_under_load_and_its_drift_waits() {
    let mut driver = scale_driver();
    driver.fill_to_watermark();
    let rest = root::try_step(&mut driver.domain, &driver.env, service_fact(2)).expect_err("service drift waits");
    assert!(
        !matches!(rest, root::Event::Infrastructure(infra::Event::System(infra::SystemEvent::Service { .. }))),
        "the admitted service fact must never be replayed"
    );
    driver.collect();
    assert_eq!(drifts(&driver), 0, "drift has not run");
    assert_eq!(ready_readers(&driver), ONCE, "the scale reader advances in the admitted part");
    driver.settle();
    driver.send(rest);
    driver.settle();
    assert_eq!(drifts(&driver), ONCE);
    assert_eq!(ready_readers(&driver), ONCE, "drift does not replay settlement");
}

#[test]
fn an_environment_absence_settles_under_load_and_its_drift_waits() {
    let mut driver = Driver::started(SEED);
    driver.send(root::Event::Infrastructure(infra::Event::Restore {
        record: infra::Record::Made {
            environment: jig_ops_world::infra_environment(),
            key: infra::Key::procedure([1; 16], TASK, 3),
            project: Some(1),
        },
    }));
    driver.send(root::Event::Infrastructure(infra::Event::Names {
        task: TASK,
        project: 1,
        resources: Box::new([infra::Resource::Environment(jig_ops_world::infra_environment())]),
    }));
    driver.send(root::Event::Infrastructure(infra::Event::System(infra::SystemEvent::Environment {
        environment: jig_ops_world::infra_environment(),
        other_hand: false,
        fact: Some(infra::EnvironmentFact {
            created_by: infra::Key::procedure([1; 16], TASK, 3),
            ready_at: 0,
            until: 100,
            price: 20,
        }),
    })));
    driver.settle();
    driver.fill_to_watermark();
    let rest = root::try_step(&mut driver.domain, &driver.env, absence()).expect_err("deletion drift waits");
    assert!(
        !matches!(rest, root::Event::Infrastructure(infra::Event::System(infra::SystemEvent::Environment { .. }))),
        "the admitted absence must never be replayed"
    );
    driver.settle();
    driver.send(rest);
    driver.settle();
    assert_eq!(drifts(&driver), ONCE);
}

#[test]
fn completions_pass_a_held_new_event_in_order() {
    let mut driver = Driver::started(SEED);
    for token in [FIRST, SECOND, THIRD] {
        driver.send(read(token));
    }
    driver.fill_to_watermark();
    for event in
        [new_work(), read_done(SECOND, b"second"), read_done(FIRST, b"first"), new_work(), read_done(THIRD, b"third")]
    {
        driver.enqueue(event);
    }
    driver.drive_inputs_once();
    assert_eq!(replies(&driver), [(SECOND, b"second".as_slice()), (FIRST, b"first".as_slice())]);
    driver.settle();
    assert_eq!(
        replies(&driver),
        [(SECOND, b"second".as_slice()), (FIRST, b"first".as_slice()), (THIRD, b"third".as_slice())]
    );
}

#[test]
fn a_mixed_event_that_passes_leaves_its_rest_and_ends_the_pass() {
    let mut driver = scale_driver();
    driver.send(read(SECOND));
    driver.fill_to_watermark();
    for event in [new_work(), service_fact(2), read_done(SECOND, b"second")] {
        driver.enqueue(event);
    }
    driver.drive_inputs_once();
    assert_eq!(driver.events.len(), HELD_INPUTS, "the held head, mixed rest and completion retain their places");
    assert!(matches!(driver.events.front(), Some(root::Event::Observability(obs::Event::Subscribe { .. }))));
    assert!(matches!(
        driver.events.get(1),
        Some(root::Event::Infrastructure(infra::Event::System(infra::SystemEvent::ServiceDrift { from: 1, .. })))
    ));
    assert!(replies(&driver).is_empty(), "the pass ends at the mixed remainder");
    assert_eq!(ready_readers(&driver), ONCE);
    assert_eq!(drifts(&driver), 0);
    driver.settle();
    assert_eq!(ready_readers(&driver), ONCE, "settlement is never replayed");
    assert_eq!(drifts(&driver), ONCE);
    assert_eq!(replies(&driver), [(SECOND, b"second".as_slice())]);
}

#[test]
fn a_service_returning_to_its_baseline_cancels_deferred_drift() {
    let mut driver = scale_driver();
    driver.fill_to_watermark();
    let rest = root::try_step(&mut driver.domain, &driver.env, service_fact(2)).expect_err("new drift waits");
    driver.send(service_fact(1));
    driver.settle();
    driver.send(rest);
    driver.settle();
    assert_eq!(drifts(&driver), 0, "the rest compares with the fact kept when it runs");
    assert_eq!(ready_readers(&driver), ONCE);
}

#[test]
fn a_result_read_finishes_its_admitted_run_under_pressure() {
    let mut driver = pressured_chat();
    let returned = driver.returned;
    driver.send(root::Event::Agent(host::FromAgent::Finished {
        owner: driver.owner,
        turns: 0,
        spent: 0,
        read: None,
        finish: host::Finish::Ended { outcome: b"result".as_slice().into() },
    }));
    driver.send(root::Event::Agent(host::FromAgent::Gone { owner: driver.owner, detail: Box::new([]) }));
    let payload = driver
        .released
        .iter()
        .find_map(|output| if let root::Output::ReadResult { payload, .. } = output { Some(*payload) } else { None })
        .expect("the admitted run's fenced result needs reading");
    driver.send(root::Event::ResultRead { payload, end: tasks::End::Parked });
    assert_eq!(driver.returned, returned);
    assert!(driver.notices.is_empty(), "the terminal acknowledgement waits for its commit");
    driver.settle();
    assert_eq!(terminal_count(&driver), ONCE);
    assert_eq!(driver.notices.len(), ONCE, "one committed terminal acknowledgement");
}

#[test]
fn a_call_answer_and_its_released_callback_enter_under_pressure() {
    let mut driver = Driver::started(SEED);
    driver.chat();
    driver.send(worker_call(driver.owner, true));
    let index = driver
        .released
        .iter()
        .position(|output| matches!(output, root::Output::Now(core::Now::Call { .. })))
        .expect("the admitted worker call leaves for decoding");
    let root::Output::Now(core::Now::Call { to, .. }) = driver.released.remove(index) else { unreachable!() };
    driver.fill_to_watermark();
    let returned = driver.returned;
    driver.send(root::Event::CallAnswered {
        to,
        call: core::SettledCall {
            serial: 0,
            tool: b"subscribe".as_slice().into(),
            answer: core::SettledAnswer::Host { error: true, body: b"invalid call".as_slice().into() },
        },
    });
    assert_eq!(driver.returned, returned);
    assert!(
        !driver.released.iter().any(|output| matches!(output, root::Output::Agent(host::ToAgent::Answer { .. }))),
        "the answer waits for its commit"
    );
    let commits = driver.pending.len();
    for _ in 0..commits {
        driver.commit_one();
        driver.fill_to_watermark();
    }
    assert!(!driver.events.is_empty(), "the committed answer released its callback");
    let returned = driver.returned;
    driver.drive_inputs_once();
    assert_eq!(driver.returned, returned, "the released callback enters at the watermark");
    driver.settle();
    assert_eq!(
        driver
            .released
            .iter()
            .filter(|output| matches!(output, root::Output::Agent(host::ToAgent::Answer { .. })))
            .count(),
        ONCE
    );
}

#[test]
fn an_applied_effect_settles_once_at_the_watermark() {
    const ENTRY: u64 = 3;
    const COPIES: usize = 2; // The same terminal twice.
    let key = obs::Key::procedure([1; 16], TASK, ENTRY);
    let mut driver = Driver::started(SEED);
    driver.send(root::Event::Observability(obs::Event::Restore {
        record: obs::Record::Outbox(obs::Entry {
            project: Some(1),
            number: ENTRY,
            key,
            effect: obs::Effect { rule: b"rule".as_slice().into(), until: 100 },
            attempt: 1,
            deadline: 100,
            phase: obs::Phase::Sent,
        }),
    }));
    driver.settle();
    driver.fill_to_watermark();
    let returned = driver.returned;
    for _ in 0..COPIES {
        driver.send(root::Event::Observability(obs::Event::System(obs::SystemEvent::Applied {
            key,
            attempt: 1,
            outcome: obs::Outcome::Made,
        })));
    }
    assert_eq!(driver.returned, returned);
    driver.settle();
    assert_eq!(
        driver
            .transactions
            .iter()
            .flat_map(|(_, writes)| writes)
            .filter(|write| matches!(write,
        root::Write::Erase(root::Key::Observability(obs::RecordKey::Outbox(erased))) if *erased == key))
            .count(),
        ONCE
    );
}

#[test]
fn read_only_worker_calls_still_wait_as_new_work() {
    let mut driver = pressured_chat();
    let returned = driver.returned;
    driver.send_retaining(worker_call(driver.owner, false));
    assert_eq!(driver.returned, returned + ONE_REFUSAL);
    assert!(matches!(
        driver.events.front(),
        Some(root::Event::Agent(host::FromAgent::Called { ask: host::Ask::Relay { writes: false, .. }, .. }))
    ));
    driver.settle();
    assert_eq!(
        driver.released.iter().filter(|output| matches!(output, root::Output::Now(core::Now::Call { .. }))).count(),
        ONCE
    );
}

#[test]
fn every_worker_call_entrance_classes_read_only_calls_as_new() {
    let root::Event::Agent(agent) = worker_call(FIRST, false) else { unreachable!() };
    for input in [
        worker_call(FIRST, false),
        root::Event::Host(host::Event::from_agent(agent)),
        root::Event::Core(core::Event::Fleet(fleet::Event::Relay {
            channel: FIRST,
            run: FIRST,
            attempt: FIRST,
            call: fleet::Call {
                name: Box::new([]),
                tool: b"logs".as_slice().into(),
                writes: false,
                input: Box::new([]),
                deadline: Duration::from_secs(30),
            },
        })),
    ] {
        assert_eq!(root::class(&input), root::Class::New);
    }
}

#[test]
fn mixed_facts_without_new_work_leave_no_rest_under_pressure() {
    let mut driver = Driver::started(SEED);
    driver.fill_to_watermark();
    let returned = driver.returned;
    // No prior facts or creation binding exists, so none of these facts has news.
    for input in [health_fact(2), service_fact(1), absence()] {
        driver.send(input);
    }
    assert_eq!(driver.returned, returned);
    driver.settle();
    assert_eq!(drifts(&driver), 0);
}
