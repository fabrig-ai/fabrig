use skein_lib::{Env, Queue, Time, Wall};
use skein_world::domain::heap::Meter;
use temper_engine_domain_forge as top;
use temper_engine_forge_world::{LIMITS, REPO};

fn drain(out: &mut Queue<top::Request>) {
    for _ in 0..out.len() {
        out.pop();
    }
}

// Measured peak: 3,626 bytes; 25% headroom, rounded up to two significant digits.
const CONNECTOR_HEAP_CEILING: u64 = 4_600;

#[test]
fn bounded_connector_tables_and_client_fit_the_declared_heap() {
    for limits in
        [LIMITS, top::Limits { facts: 0, client: skein_forge_client::Limits { facts: 0, ..LIMITS.client }, ..LIMITS }]
    {
        let meter = Meter::new();
        let mut domain = connector(&limits);
        let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits };
        let mut out = Queue::with_capacity(top::max_out(&limits));
        top::step(
            &mut domain,
            &env,
            top::Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic },
            &mut out,
        );
        drain(&mut out);
        for task in 1..=u64::from(limits.tasks) {
            let name = top::Name { forge: REPO.forge, repository: REPO.repository, what: top::What::Issue(task) };
            meter.start();
            top::step(&mut domain, &env, top::Event::Names { task, resources: Box::new([name.clone()]) }, &mut out);
            let measured = meter.end();
            drain(&mut out);
            meter.check(measured, CONNECTOR_HEAP_CEILING, &limits);
            domain.reclaim();

            meter.start();
            top::step(&mut domain, &env, top::Event::Hold { task, resource: name, from: None }, &mut out);
            let measured = meter.end();
            drain(&mut out);
            meter.check(measured, CONNECTOR_HEAP_CEILING, &limits);
            domain.reclaim();
        }
    }
}

fn connector(limits: &top::Limits) -> top::Domain {
    top::Domain::new(
        limits,
        9,
        skein_forge_client::Config {
            namespace: Box::from(&b"world"[..]),
            writers: Box::new([skein_forge_client::Writer { forge: REPO.forge, author: 1 }]),
        },
        &[top::GitIdentity { forge: REPO.forge, credential: 1 }],
        &[],
        false,
    )
    .expect("admitted limits")
}

// Measured peak: 576 bytes; 25% headroom, rounded up to two significant digits.
const RESTORED_FLIGHTS_HEAP_CEILING: u64 = 720;

#[test]
fn restored_flight_nodes_fit_the_counted_entry_bound() {
    const TASK: u64 = 7;
    let mut domain = connector(&LIMITS);
    let env = Env { now: Time::ZERO, wall: Wall::EPOCH, limits: LIMITS };
    let mut out = Queue::with_capacity(top::max_out(&LIMITS));
    let meter = Meter::new();
    meter.start();
    for entry in 0..u64::from(LIMITS.entries) {
        top::step(
            &mut domain,
            &env,
            top::Event::Restore { record: top::Stored::Flight { task: TASK, entry } },
            &mut out,
        );
    }
    let measured = meter.end();
    meter.check(measured, RESTORED_FLIGHTS_HEAP_CEILING, &LIMITS);
    assert!(out.is_empty(), "restoration emits no writes");
}
