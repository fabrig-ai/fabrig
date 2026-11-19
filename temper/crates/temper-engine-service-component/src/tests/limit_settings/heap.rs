//! The deployment store's admission bounds do not reserve heap at rest.
use super::{build, changed, fixtures, inputs, item};
use crate::{limit_settings::LimitInputs, store::Store};
use skein_lib::{List, Time};
use skein_world::domain::heap::{Measured, Meter};

fn start(directory: &mut fixtures::directory::Directory, input: LimitInputs) -> Box<crate::Service> {
    let mut store = Store::for_limits(&input, &[]).unwrap();
    directory.open_limits(&mut store);
    let state = store.resolve_settings(input, None).unwrap();
    let mut service = build(store, state);
    for _ in 0_u32..10_000 {
        directory.full_turn(&mut service);
        assert_eq!(service.end(), None, "start succeeds");
        let engine = service.engine.as_ref().unwrap();
        if engine.domain.ready() && engine.store.idle() && !service.work_pending(Time::from_nanos(1)) {
            break;
        }
    }
    let engine = service.engine.as_ref().unwrap();
    assert!(engine.domain.ready(), "engine ready");
    assert!(engine.store.idle(), "store idle");
    assert!(
        !service.work_pending(Time::from_nanos(1)),
        "queues drained and nothing due: {:?}",
        service.pending(Time::from_nanos(1))
    );
    assert!(service.submissions().is_empty(), "all submissions handed to the kernel");
    assert!(service.completions().is_empty(), "all kernel completions consumed");
    service
}
fn measure(typed: bool, hundred: bool) -> Measured {
    let meter = Meter::new();
    meter.start();
    let mut directory = fixtures::directory::Directory::new();
    if hundred {
        let mut previous = start(&mut directory, inputs());
        let engine = previous.engine.as_mut().unwrap();
        let mut header = engine.domain.deployment();
        header.commits = header.commits.checked_add(1).unwrap();
        let mut writes = List::with_capacity(3);
        writes
            .push(jig_protocol_store::Write::Save(jig_domain_core::Record::Core(
                jig_domain_core::CoreRecord::Deployment(header),
            )))
            .unwrap();
        for write in changed(Box::new([]), Box::new([item(b"live_goals", 100)])) {
            writes.push(write).unwrap();
        }
        directory.keep(&mut engine.store, header.commits, writes.into_boxed());
        directory.reopen(&mut engine.store);
        drop(previous);
    }
    let mut input = inputs();
    if typed {
        input.base = Some(input.stock().unwrap());
    }
    let service = start(&mut directory, input);
    assert_eq!(
        service.engine.as_ref().unwrap().limits_in_force().unwrap().in_force.temper.live_goals,
        if hundred { 100 } else { 40 }
    );
    // The fake directory and kernel are the harness, not heap retained by the engine.
    drop(directory);
    let measured = meter.end();
    drop(service);
    measured
}

#[test]
#[expect(clippy::print_stderr, reason = "the at-rest test reports each case's held heap and startup peak")]
fn the_engine_at_rest_keeps_the_store_bounds_off_the_heap() {
    // Held / startup peak, in bytes: store for 40 = 770,860 / 1,839,612;
    // stock = 770,860 / 1,839,612; live_goals 100 = 771,230 / 2,102,719.
    // The unchanged base's store for 40 held 770,860, peaking at 1,839,612.
    // A quarter above the largest held figure, rounded up to two significant
    // digits (testing-strategy.md, 6). Peaks include the fake directory/kernel
    // and, for 100 live goals, planting its setting before the measured start.
    const AT_REST_HEAP_CEILING: u64 = 970_000;
    let before = measure(true, false);
    let stock = measure(false, false);
    let hundred = measure(false, true);
    for (case, measured) in [("store for 40", before), ("stock", stock), ("live_goals 100", hundred)] {
        std::eprintln!("{case}: held={}, peak={}", measured.held(), measured.peak());
        assert!(measured.held() <= 38_000_000, "engine at rest: {measured:?}");
        assert!(measured.held() <= AT_REST_HEAP_CEILING, "engine at rest: {measured:?}");
    }
    assert!(stock.held().abs_diff(before.held()) * 100 <= before.held(), "store bounds cost no more than 1% at rest");
}
