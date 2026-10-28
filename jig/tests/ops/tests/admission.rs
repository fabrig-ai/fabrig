//! Admitted continuations and mixed facts at the root's watermark (domain/root.md, 3).
use jig_domain_host as host;
use jig_ops_domain as root;
use jig_ops_domain_infrastructure as infra;
use jig_ops_domain_observability as obs;
use jig_ops_world::direct::Driver;
use skein_lib::Token;

const SEED: u64 = 9205;
const TASK: u64 = 7;
const FIRST: Token = Token::new(960);
const SECOND: Token = Token::new(961);
const THIRD: Token = Token::new(962);
const READ_BYTES: u32 = 128;
const ONCE: usize = 1;

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
    driver.released.iter().filter_map(|output| {
        if let root::Output::Read { owner, bytes } = output { Some((*owner, bytes.as_ref())) } else { None }
    }).collect()
}

fn health_fact(replicas: u32) -> root::Event {
    root::Event::Observability(obs::Event::System(obs::SystemEvent::Fact {
        service: jig_ops_world::service(),
        fact: obs::Fact { observed: 0, healthy_replicas: replicas, errors: 0, error_rate_percent: 0,
            load_percent: 10, load: Box::new([]) },
    }))
}

fn service_fact(revision: u64) -> root::Event {
    root::Event::Infrastructure(infra::Event::System(infra::SystemEvent::Service {
        service: jig_ops_world::infra_service(),
        fact: infra::ServiceFact { version: b"v1".as_slice().into(), replicas: 2, healthy: true, revision, observed: 0 },
        other_hand: true,
    }))
}

fn absence() -> root::Event {
    root::Event::Infrastructure(infra::Event::System(infra::SystemEvent::Environment {
        environment: jig_ops_world::infra_environment(), fact: None, other_hand: true,
    }))
}

fn drifts(driver: &Driver) -> usize {
    driver.transactions.iter().flat_map(|(_, writes)| writes).filter(|write| matches!(write,
        root::Write::Save(root::Record::Infrastructure(infra::Record::Drift(_))))).count()
}

#[test]
fn admitted_work_is_taken_at_the_watermark() {
    let mut driver = Driver::started(SEED);
    driver.chat();
    driver.fill_to_watermark();
    let returned = driver.returned;
    driver.send(root::Event::AgentContent { owner: driver.owner, content: root::AgentContent::Text { turn: 1, retry: 0, offset: 0, text: b"partial".as_slice().into() } });
    driver.send(root::Event::Agent(host::FromAgent::Finished {
        owner: driver.owner, turns: 0, spent: 0, read: None, finish: host::Finish::Parked,
    }));
    driver.send(root::Event::Agent(host::FromAgent::Gone { owner: driver.owner, detail: Box::new([]) }));
    driver.send(root::Event::InfrastructureTimer);
    driver.send(root::Event::TurnTimer);
    assert_eq!(driver.returned, returned, "records and retries of admitted work enter at the watermark");
    driver.settle();
    assert_eq!(driver.transactions.iter().flat_map(|(_, writes)| writes).filter(|write| matches!(write,
        root::Write::Save(root::Record::Core(jig_domain_core::Record::Core(jig_domain_core::CoreRecord::Terminal(_)))))).count(), ONCE);
}

#[test]
fn a_health_fact_settles_under_load_and_its_news_waits() {
    let mut driver = Driver::started(SEED);
    driver.send(health_fact(2));
    driver.settle();
    driver.fill_to_watermark();
    let rest = root::try_step(&mut driver.domain, &driver.env, health_fact(1)).expect_err("health news waits");
    assert!(!matches!(rest, root::Event::Observability(obs::Event::System(obs::SystemEvent::Fact { .. }))),
        "the admitted fact must never be replayed");
    driver.settle();
    driver.send(rest);
    driver.settle();
}

#[test]
fn a_service_fact_settles_under_load_and_its_drift_waits() {
    let mut driver = Driver::started(SEED);
    driver.send(service_fact(1));
    driver.settle();
    driver.fill_to_watermark();
    let rest = root::try_step(&mut driver.domain, &driver.env, service_fact(2)).expect_err("service drift waits");
    assert!(!matches!(rest, root::Event::Infrastructure(infra::Event::System(infra::SystemEvent::Service { .. }))),
        "the admitted service fact must never be replayed");
    driver.collect();
    assert_eq!(drifts(&driver), 0, "drift has not run");
    driver.settle();
    driver.send(rest);
    driver.settle();
    assert_eq!(drifts(&driver), ONCE);
}

#[test]
fn an_environment_absence_settles_under_load_and_its_drift_waits() {
    let mut driver = Driver::started(SEED);
    driver.send(root::Event::Infrastructure(infra::Event::Restore { record: infra::Record::Made {
        environment: jig_ops_world::infra_environment(), key: infra::Key::procedure([1; 16], TASK, 3), project: Some(1),
    }}));
    driver.settle();
    driver.fill_to_watermark();
    let rest = root::try_step(&mut driver.domain, &driver.env, absence()).expect_err("deletion drift waits");
    assert!(!matches!(rest, root::Event::Infrastructure(infra::Event::System(infra::SystemEvent::Environment { .. }))),
        "the admitted absence must never be replayed");
    driver.settle();
    driver.send(rest);
    driver.settle();
    assert_eq!(drifts(&driver), ONCE);
}

#[test]
fn completions_pass_a_held_new_event_in_order() {
    let mut driver = Driver::started(SEED);
    for token in [FIRST, SECOND, THIRD] { driver.send(read(token)); }
    driver.fill_to_watermark();
    for event in [new_work(), read_done(SECOND, b"second"), read_done(FIRST, b"first"), new_work(), read_done(THIRD, b"third")] {
        driver.enqueue(event);
    }
    driver.drive_inputs_once();
    assert_eq!(replies(&driver), [(SECOND, b"second".as_slice()), (FIRST, b"first".as_slice())]);
    driver.settle();
    assert_eq!(replies(&driver), [(SECOND, b"second".as_slice()), (FIRST, b"first".as_slice()), (THIRD, b"third".as_slice())]);
}
