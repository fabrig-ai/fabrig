//! Limit representation (programming-model.md, 6.3; io.md, 2).
use skein_io::{Limits, valid};
use skein_lib::{Duration, Rng};

// The smallest usable admission and byte limits; an empty entity slab is usable.
pub(super) const LIMITS: Limits = Limits {
    sockets: 0,
    names: 0,
    refusals: 1,
    intake: 1,
    receive: 1,
    output: 1,
    sends: 1,
    accepts: 1,
    backlog: 0,
    close_timeout: Duration::from_nanos(0),
    retry: Duration::from_nanos(1),
};

#[test]
fn valid_and_heap_bound_agree_at_representable_capacity_edges() {
    // Capacities are 8*sockets+4 flights, 4*sockets+2 operations, 2*sockets timers.
    const FLIGHT_LAST: u32 = 536_870_911;
    const OPERATION_LAST: u32 = 1_073_741_823;
    const TIMER_LAST: u32 = 2_147_483_647;
    let cases = [
        ("minimal", LIMITS, true),
        ("last flights", Limits { sockets: FLIGHT_LAST, ..LIMITS }, true),
        ("first flight overflow", Limits { sockets: FLIGHT_LAST + 1, ..LIMITS }, false),
        ("last operations", Limits { sockets: OPERATION_LAST, ..LIMITS }, false),
        ("first operation overflow", Limits { sockets: OPERATION_LAST + 1, ..LIMITS }, false),
        ("last timers", Limits { sockets: TIMER_LAST, ..LIMITS }, false),
        ("first timer overflow", Limits { sockets: TIMER_LAST + 1, ..LIMITS }, false),
        ("max sockets", Limits { sockets: u32::MAX, ..LIMITS }, false),
        ("max names", Limits { names: u32::MAX, ..LIMITS }, true),
        ("max refusals", Limits { refusals: u32::MAX, ..LIMITS }, true),
        ("max intake", Limits { intake: u32::MAX, ..LIMITS }, true),
        ("max receive", Limits { receive: u32::MAX, ..LIMITS }, true),
        ("max output", Limits { output: u32::MAX, ..LIMITS }, true),
        ("max sends", Limits { sends: u32::MAX, ..LIMITS }, true),
        ("max accepts", Limits { accepts: u32::MAX, ..LIMITS }, true),
        ("max backlog", Limits { backlog: u32::MAX, ..LIMITS }, true),
    ];
    for (name, limits, expected) in cases {
        assert_eq!(valid(&limits), expected, "{name}: {limits:?}");
        assert_eq!(valid(&limits), super::heap_bound(&limits).is_some(), "{name}: {limits:?}");
    }
}

#[test]
fn valid_refuses_unusable_io_limits_the_sum_accepted() {
    let cases = [
        Limits {
            refusals: 0,
            intake: 0,
            receive: 0,
            output: 0,
            sends: 0,
            accepts: 0,
            retry: Duration::from_nanos(0),
            ..LIMITS
        },
        Limits { refusals: 0, ..LIMITS },
        Limits { intake: 0, ..LIMITS },
        Limits { receive: 0, ..LIMITS },
        Limits { output: 0, ..LIMITS },
        Limits { sends: 0, ..LIMITS },
        Limits { accepts: 0, ..LIMITS },
        Limits { retry: Duration::from_nanos(0), ..LIMITS },
    ];
    for limits in cases {
        assert!(!valid(&limits), "{limits:?}");
        assert!(super::heap_bound(&limits).is_some(), "{limits:?}");
    }
}

#[test]
fn valid_accepts_limits_whose_heap_total_alone_overflows() {
    // Flights still fit, but every stream's maximum queue of boxed sends
    // makes the aggregate heap exceed u64 on native and wasm32 targets.
    const LAST_FLIGHT_SOCKETS: u32 = 536_870_911;
    let limits = Limits { sockets: LAST_FLIGHT_SOCKETS, sends: u32::MAX, ..LIMITS };
    assert!(valid(&limits));
    assert!(super::heap_bound(&limits).is_none());
}

pub(super) fn draw(rng: &mut Rng, maximum: u32) -> u32 {
    u32::try_from(rng.between(0, u64::from(maximum))).expect("draw fits u32")
}
