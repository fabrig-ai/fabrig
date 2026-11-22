use skein_lib::{Time, Wall};

#[test]
fn an_iteration_environment_keeps_the_clocks_and_limits() {
    let now = Time::from_nanos(17);
    let wall = Wall::from_nanos(29);
    let env = crate::Now { now, wall }.with(7_u32);
    assert_eq!(env.now, now, "the iteration's monotonic time");
    assert_eq!(env.wall, wall, "the iteration's wall time");
    assert_eq!(env.limits, 7, "the step's limits");
}

#[test]
fn a_fixed_clock_starts_at_the_supplied_times() {
    let start = crate::FixedStart { wall: Wall::from_nanos(1_767_690_000_000_000_000), seed: 7 };
    let clock = crate::Clock::fixed(&start);
    assert_eq!(clock.now(), crate::Now { now: crate::FixedStart::NOW, wall: start.wall });
}

#[test]
fn the_host_clock_passes_deadlines_through() {
    let clock = crate::Clock::new();
    let deadline = clock.now().now;
    assert_eq!(clock.until(Some(deadline)), Some(deadline));
    assert_eq!(clock.until(None), None);
}
