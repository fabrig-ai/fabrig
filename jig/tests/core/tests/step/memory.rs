use jig_domain_core as core;
use skein_lib::{Env, Time, Wall};
use skein_world::domain::heap::Meter;

/// Empty closure: 992 measured additional bytes plus a quarter, rounded up
/// to two significant digits.
const EMPTY_CLOSURE_HEAP_CEILING: u64 = 1_300;

// Peak 4,255 bytes measured 10-29; testing-strategy.md 6.
const FINISHING_QUERIES_HEAP_CEILING: u64 = 5_400;

#[test]
fn settled_finishing_queries_allocate_nothing_and_empty_closure_stays_under_its_heap_ceiling() {
    let limits = jig_test_profile::limits().core;
    let meter = Meter::new();
    meter.start();
    let mut domain = jig_test_profile::new_core(jig_test_profile::config(192).core, &limits).expect("fixture core");
    domain.counters = core::Counters::new(domain.counters.deployment());
    let setup = meter.end();
    // The current catalogue adds a boxed default and the resolved charter map;
    // two run-byte bounds cover their owned model and policy bytes in this fixture.
    let bound = FINISHING_QUERIES_HEAP_CEILING
        + skein_lib::Map::<u32, core::Charter>::price(limits.tasks.charters).expect("resolved charter map")
        + u64::from(limits.run_bytes) * 2
        + u64::try_from(size_of::<core::settings::CatalogueModel>()).expect("catalogue record")
        + u64::try_from(size_of::<core::EndpointFacts>()).expect("endpoint facts");
    meter.check(setup, bound, &limits);
    let held = meter.held();
    meter.start();
    for _ in 0..100 {
        assert!(!domain.finish_owed());
    }
    let queries = meter.end();
    assert_eq!((queries.held(), queries.peak()), (held, held));
    meter.check(queries, bound, &limits);
    meter.start();
    let out = core::finish_decision(&mut domain, &Env { now: Time::ZERO, wall: Wall::EPOCH, limits });
    let closed = meter.end();
    assert!(out.is_empty());

    assert!(closed.peak() - held <= EMPTY_CLOSURE_HEAP_CEILING, "empty closure stays under its heap ceiling");
    drop(out);
    meter.check(closed, bound, &limits);
    meter.start();
    let mut quiet = skein_lib::Queue::with_capacity(0);
    jig_domain_views::step(
        &mut domain.views,
        &Env { now: Time::ZERO, wall: Wall::EPOCH, limits: limits.views },
        jig_domain_views::Event::Text {
            run: skein_lib::Token::new(41),
            turn: jig_domain_views::TurnName { attempt: 1, turn: 1, retry: 0 },
            offset: 0,
            text: Box::new(*b"x"),
            call: None,
        },
        &mut quiet,
    );
    let retained = meter.end();
    assert!(quiet.is_empty());
    meter.check(retained, bound, &limits);
    let held = meter.held();
    meter.start();
    for _ in 0..100 {
        assert!(domain.finish_owed());
    }
    let queries = meter.end();
    assert_eq!((queries.held(), queries.peak()), (held, held), "retained metadata inspection also allocates nothing");
    meter.check(queries, bound, &limits);
}
