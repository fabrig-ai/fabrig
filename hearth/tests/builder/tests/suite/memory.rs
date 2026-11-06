use super::bounds;
use hearth_builder_world::{fixtures, simulated};

#[test]
fn a_client_that_stops_reading_holds_render_under_its_ceiling() {
    // A quarter above the measured 237,107-byte peak, rounded up to two significant digits.
    const CEILING: u64 = 300_000;
    // Release at one second, before the shipped five-second append timeout.
    const RELEASE: skein_lib::Time = skein_lib::Time::from_nanos(1_000_000_000);
    let outcome = simulated::checked_stuck(&bounds::stuck(), bounds::SEED, bounds::stuck_limits(), CEILING, RELEASE);
    assert_eq!(outcome.machine.output_holds(), (1, 1), "the output was held and released once");
    let output = String::from_utf8(outcome.machine.contents(fixtures::OUTPUT).expect("printed specs")).expect("text");
    let targets: Vec<_> = output
        .lines()
        .filter_map(|line| line.strip_prefix("target       "))
        .map(|line| line.split_whitespace().next().expect("target name"))
        .collect();
    let mut expected: Vec<_> = (0..bounds::STUCK_TARGETS).map(|target| format!("b{target}-prod")).collect();
    expected.sort();
    assert_eq!(targets, expected, "every target prints once, in order, after release");
    assert_eq!(fixtures::client_exits(&outcome), [skein_io::kernel::Exit::Code(0)]);
}

#[test]
fn the_builder_holds_no_more_than_its_ceiling_at_every_iteration() {
    const CEILING: u64 = 250_000;
    let outcome = simulated::checked(fixtures::SIMULATION_SEED, CEILING);
    assert!(outcome.peaks().is_some(), "the simulator checked every iteration's peak");
    assert!(
        fixtures::lists(&outcome.machine) > fixtures::STORE_PAGE,
        "hearth/docs/design/domain/store.md, 8.2: the ledger contains more than one page of list operations"
    );
    assert_eq!(outcome.machine.contents(fixtures::OUTPUT).as_deref(), fixtures::THIN_OUTPUT);
}

#[test]
fn queued_specs_hold_no_more_than_their_ceiling() {
    const CEILING: u64 = 250_000;
    let outcome = simulated::checked_with(&bounds::queued(), bounds::SEED, bounds::limits(), CEILING);
    let output = outcome.machine.contents(fixtures::OUTPUT).expect("printed specs");
    assert_eq!(String::from_utf8_lossy(&output).matches("target       ").count(), 4);
}

#[test]
fn recording_resolutions_holds_no_more_than_their_ceiling() {
    const CEILING: u64 = 250_000;
    let outcome =
        simulated::checked_with(&fixtures::resolving(fixtures::THIN), bounds::SEED, fixtures::small_limits(), CEILING);
    assert_eq!(fixtures::simulation_records(&outcome.machine).len(), 1);
}

#[test]
fn building_holds_no_more_than_their_ceiling() {
    use hearth_lib::encode::Digest;
    const CEILING: u64 = 1_400_000;
    let laid = [1_u8, 2]
        .map(|byte| fixtures::Laid::Layer(Box::new(fixtures::build::config_over(Digest([byte; 32]), Digest([0; 32])))));
    let scenario = fixtures::Scenario { laid: &laid, ..fixtures::build::building() };
    let outcome = simulated::checked_with(&scenario, bounds::SEED, fixtures::small_limits(), CEILING);
    assert_eq!(fixtures::client_exits(&outcome), [skein_io::kernel::Exit::Code(69)], "the fetch is unavailable");
    let output = String::from_utf8(outcome.machine.contents(fixtures::OUTPUT).expect("printed layers")).expect("text");
    assert_eq!(output.matches("   to build\n").count(), 3, "{output}");
    assert!(output.ends_with("failed       fetch packages   unavailable\n"), "{output}");
    let error = String::from_utf8(outcome.machine.contents(fixtures::ERROR).expect("progress")).expect("text");
    assert!(error.starts_with("resolving ") && !error.contains("strays"), "the laid records are indexed: {error}");
}
