use super::fixtures;
use crate::{Graces, Path, Problem, What, stop_bound};
use skein_lib::Duration;

fn worker(bytes: &[u8]) -> Result<crate::Worker, crate::Problems> {
    let limits = crate::DocumentLimits { bytes: 8192, tokens: 128, depth: 8 };
    let document = crate::document(bytes, &limits).expect("worker fixture JSON");
    crate::worker(&document, 0, &crate::Directories::working(), &crate::FixedWorker::NONE)
}
fn small() -> crate::Worker {
    worker(fixtures::WORKER_SMALL).unwrap()
}
fn files() -> Box<[crate::File]> {
    crate::worker_files(&small())
}
fn refuses(from: &[u8], to: &[u8], path: Path, what: What) {
    let bytes = fixtures::with(fixtures::WORKER_SMALL, from, to);
    let problems = worker(&bytes).unwrap_err();
    assert_eq!(problems.problems.get(0), Some(&Problem { path, what }));
}
fn graces() -> Graces {
    Graces {
        grace: Duration::from_secs(30),
        drain: Duration::from_secs(10),
        agent_stop: Duration::from_secs(10),
        delivery: Duration::from_secs(20),
        save: Duration::from_secs(5),
    }
}
#[test]
fn the_stop_bound_sums_ping_silence_grace_the_longer_stop_and_a_save() {
    let bound = stop_bound(&graces(), Duration::from_secs(5), Duration::from_secs(15));
    assert_eq!(bound, Some(Duration::from_secs(75)));
    let draining = Graces { drain: Duration::from_secs(180), ..graces() };
    assert_eq!(stop_bound(&draining, Duration::from_secs(5), Duration::from_secs(15)), Some(Duration::from_secs(205)));
}
#[test]
fn the_stop_bound_uses_the_agents_grace_when_it_is_longer() {
    let graces = Graces { agent_stop: Duration::from_secs(40), ..graces() };
    assert_eq!(stop_bound(&graces, Duration::from_secs(5), Duration::from_secs(15)), Some(Duration::from_secs(95)));
}
#[test]
fn a_stop_bound_past_a_duration_is_refused() {
    let graces = Graces { save: Duration::from_nanos(u64::MAX), ..graces() };
    assert_eq!(stop_bound(&graces, Duration::from_secs(5), Duration::from_secs(15)), None);
}
#[test]
fn an_absent_drain_uses_three_minutes() {
    let bytes = fixtures::with(fixtures::WORKER_SMALL, br#""drain_ms": 10000, "#, b"");
    let worker = worker(&bytes).unwrap();
    assert_eq!(worker.graces.drain, Duration::from_millis(180_000));
    assert_eq!(worker.stop_bound, Duration::from_secs(205));
    let mut drain = None;
    for row in &crate::members::MEMBERS {
        if row.shape == "graces" {
            if row.name == "drain_ms" {
                drain = Some(row);
            } else {
                assert!(row.required);
            }
        }
    }
    assert!(!drain.expect("drain member").required);
}
#[test]
fn a_present_drain_accepts_unsigned_milliseconds_up_to_a_duration() {
    for millis in [0, 180_000, 18_446_744_073_709_u64] {
        let drain = fixtures::with(br#""drain_ms": MILLIS"#, b"MILLIS", skein_lib::Decimal::of(millis).as_bytes());
        let bytes = fixtures::with(fixtures::WORKER_SMALL, br#""drain_ms": 10000"#, &drain);
        // No cancellation tail: the largest drain fits the whole stop bound too.
        let bytes = fixtures::with(&bytes, b"\"agent_stop_ms\": 10000", b"\"agent_stop_ms\": 0");
        let bytes = fixtures::with(&bytes, b"\"delivery_ms\": 20000", b"\"delivery_ms\": 0");
        let bytes = fixtures::with(&bytes, b"\"save_ms\": 5000", b"\"save_ms\": 0");
        assert_eq!(worker(&bytes).unwrap().graces.drain, Duration::from_millis(millis));
    }
}
#[test]
fn invalid_drain_members_name_their_field() {
    for (drain, what) in [
        (br#""180000""#.as_slice(), What::Kind),
        (b"true", What::Kind),
        (b"-1", What::Bound),
        (b"1.5", What::Bound),
        (b"18446744073710", What::Bound),
        (b"10000, \"drain_ms\": 10", What::Duplicate),
    ] {
        let drain = fixtures::with(br#""drain_ms": VALUE"#, b"VALUE", drain);
        refuses(b"\"drain_ms\": 10000", &drain, fixtures::path(&[b"graces", b"drain_ms"]), what);
    }
}
fn a_silence_under_three_pings_is_refused() {
    refuses(
        br#""profile": "standard""#,
        br#""profile": "standard", "overrides": {"link.silence_ms": 14999}"#,
        fixtures::path(&[b"limits", b"link.silence_ms"]),
        What::Silence,
    );
}
#[test]
fn a_silence_of_exactly_three_pings_is_accepted() {
    let bytes = fixtures::with(
        fixtures::WORKER_SMALL,
        br#""profile": "standard""#,
        br#""profile": "standard", "overrides": {"link.ping_ms": 1000, "link.silence_ms": 3000}"#,
    );
    assert_eq!(worker(&bytes).unwrap().stop_bound, Duration::from_secs(59));
}
fn three_pings_past_a_duration_are_refused() {
    refuses(
        br#""profile": "standard""#,
        br#""profile": "standard", "overrides": {"link.ping_ms": 9223372036854}"#,
        fixtures::path(&[b"limits", b"link.silence_ms"]),
        What::Silence,
    );
}
#[test]
fn a_worker_names_its_secret_file_as_private() {
    let files = files();
    let secret = files.first().unwrap();
    assert_eq!(secret.at, fixtures::path(&[b"secret"]));
    assert_eq!(secret.path.as_ref(), b"secrets/worker-a.secret");
    assert_eq!(secret.max, 257);
    assert!(secret.private);
}
#[test]
fn the_agents_configuration_file_is_named_for_startup() {
    let files = files();
    let configuration = files.get(1).unwrap();
    assert_eq!(configuration.at, fixtures::path(&[b"agent", b"configuration"]));
    assert_eq!(configuration.path.as_ref(), b"agent.json");
    assert_eq!(configuration.max, 1_048_576);
    assert!(!configuration.private);
    assert_eq!(files.len(), 2);
}
fn an_engine_at_a_public_address_is_refused() {
    refuses(
        br#"{"in_process": "workers"}"#,
        br#"{"socket": "192.0.2.1:7100"}"#,
        fixtures::path(&[b"engine"]),
        What::Plaintext,
    );
}
#[test]
fn an_engine_at_a_loopback_address_is_accepted() {
    let bytes = fixtures::with(fixtures::WORKER_SMALL, br#"{"in_process": "workers"}"#, br#"{"socket": "[::1]:7100"}"#);
    worker(&bytes).unwrap();
}
#[test]
fn the_small_worker_document_parses_whole() {
    let worker = small();
    assert_eq!(
        worker.engine,
        crate::Address::Addr(skein_io::Addr::InProcess(skein_io::Name::new(b"workers").unwrap()))
    );
    assert_eq!(worker.name.as_ref(), b"worker-a");
    assert_eq!(worker.slots, 2);
    assert_eq!(worker.graces, graces());
    assert_eq!(worker.stop_bound, Duration::from_secs(75));
    assert_eq!(worker.agent.as_ref().expect("configured agent").program.as_ref(), b"jig-test");
    assert_eq!(worker.agent.as_ref().expect("configured agent").arguments.as_ref(), &[Box::from(&b"agent"[..])]);
    assert_eq!(worker.memory, Some(1_073_741_824));
}
fn an_agent_without_a_program_is_refused() {
    refuses(br#""program": "jig-test", "#, b"", fixtures::path(&[b"agent", b"program"]), What::Missing);
}
fn a_worker_name_past_the_profiles_bound_is_refused() {
    refuses(b"worker-a", b"012345678901234567890123456789012", fixtures::path(&[b"name"]), What::Bound);
}
#[test]
fn a_workers_hub_slots_follow_its_places_unless_overridden() {
    for slots in [12, 24] {
        let places =
            fixtures::with(br#""slots": SLOTS"#, b"SLOTS", skein_lib::Decimal::of(u64::from(slots)).as_bytes());
        let bytes = fixtures::with(fixtures::WORKER_SMALL, br#""slots": 2"#, &places);
        let bytes = fixtures::with(&bytes, br#""profile": "standard""#, br#""overrides": {}"#);
        let parsed = worker(&bytes).unwrap();
        assert_eq!(parsed.slots, slots);
        assert_eq!(parsed.limits.expect("configured limits").hub.slots, slots);
        let explicit = fixtures::with(
            br#""overrides": {"hub.slots": SLOTS}"#,
            b"SLOTS",
            skein_lib::Decimal::of(u64::from(slots)).as_bytes(),
        );
        let at = fixtures::with(&bytes, br#""overrides": {}"#, &explicit);
        assert_eq!(worker(&at).unwrap().limits.expect("configured limits").hub.slots, slots);
        let below = fixtures::with(&bytes, br#""overrides": {}"#, br#""overrides": {"hub.slots": 8}"#);
        let errors = worker(&below).unwrap_err();
        assert_eq!(
            errors.problems.as_slice(),
            &[Problem { path: fixtures::path(&[b"slots"]), what: What::SlotsPastHub }]
        );
    }
}
fn more_slots_than_the_hub_can_host_are_refused() {
    let bytes = fixtures::with(
        fixtures::WORKER_SMALL,
        br#""profile": "standard""#,
        br#""profile": "standard", "overrides": {"hub.slots": 2}"#,
    );
    let bytes = fixtures::with(&bytes, br#""slots": 2"#, br#""slots": 3"#);
    let errors = worker(&bytes).unwrap_err();
    assert_eq!(errors.problems.get(0), Some(&Problem { path: fixtures::path(&[b"slots"]), what: What::SlotsPastHub }));
}
fn a_worker_with_no_profile_adds_no_bound_of_small() {
    let bytes = fixtures::with(fixtures::WORKER_SMALL, br#""slots": 2"#, br#""slots": 3"#);
    let bytes = fixtures::with(&bytes, br#""profile": "standard""#, br#""profile": "standrd""#);
    let problems = worker(&bytes).unwrap_err();
    assert_eq!(problems.problems.len(), 1, "{problems:?}");
    assert_eq!(problems.past, 0, "{problems:?}");
    assert_eq!(problems.problems.get(0).unwrap().path, fixtures::path(&[b"limits", b"profile"]));
}
fn a_grace_past_a_duration_is_refused() {
    refuses(
        br#""save_ms": 5000"#,
        br#""save_ms": 18446744073710"#,
        fixtures::path(&[b"graces", b"save_ms"]),
        What::Bound,
    );
}
fn graces_whose_sum_overflows_refuse_the_start() {
    refuses(br#""save_ms": 5000"#, br#""save_ms": 18446744073709"#, fixtures::path(&[b"graces"]), What::StopBound);
}
fn a_worker_refuses_an_unknown_grace() {
    refuses(
        br#""save_ms": 5000"#,
        br#""save_ms": 5000, "unknown_ms": 10"#,
        fixtures::path(&[b"graces", b"unknown_ms"]),
        What::Unknown,
    );
}
fn a_worker_refuses_an_agent_member_named_twice() {
    refuses(
        br#""program": "jig-test""#,
        br#""program": "jig-test", "program": "jig-test""#,
        fixtures::path(&[b"agent", b"program"]),
        What::Duplicate,
    );
}
fn an_argument_that_is_not_text_is_refused_at_its_index() {
    refuses(
        br#"["agent"]"#,
        b"[17]",
        fixtures::path(&[b"agent", b"arguments"]).child(crate::Segment::Index(0)),
        What::Kind,
    );
}
#[test]
fn a_workers_memory_bytes_is_handed_on_unchecked() {
    let bytes = fixtures::with(fixtures::WORKER_SMALL, b"1073741824", b"1");
    assert_eq!(worker(&bytes).unwrap().memory, Some(1));
}
#[test]
fn a_worker_without_memory_bytes_parses() {
    let bytes = fixtures::with(fixtures::WORKER_SMALL, br#", "memory_bytes": 1073741824"#, b"");
    assert_eq!(worker(&bytes).unwrap().memory, None);
}
#[test]
fn a_workers_present_memory_bytes_must_be_an_unsigned_integer() {
    for (value, what) in [(br#""1""#.as_slice(), What::Kind), (b"18446744073709551616", What::Bound)] {
        refuses(b"1073741824", value, fixtures::path(&[b"memory_bytes"]), what);
    }
}
#[test]
fn a_workers_hub_guard_runs_without_memory_bytes() {
    let mut limits = crate::stock();
    limits.hub.source_timeout = Duration::ZERO;
    let mut problems = crate::Problems::empty();
    crate::check::worker(&limits, Some(graces()), &mut problems);
    assert_eq!(problems.problems.len(), 1);
    assert_eq!(problems.problems.get(0), Some(&Problem { path: fixtures::path(&[b"limits"]), what: What::Bound }));
}

#[test]
fn worker_configuration_problems_name_the_invalid_fields() {
    table([
        ("a_silence_under_three_pings_is_refused", a_silence_under_three_pings_is_refused),
        ("three_pings_past_a_duration_are_refused", three_pings_past_a_duration_are_refused),
        ("an_engine_at_a_public_address_is_refused", an_engine_at_a_public_address_is_refused),
        ("an_agent_without_a_program_is_refused", an_agent_without_a_program_is_refused),
        ("a_worker_name_past_the_profiles_bound_is_refused", a_worker_name_past_the_profiles_bound_is_refused),
        ("more_slots_than_the_hub_can_host_are_refused", more_slots_than_the_hub_can_host_are_refused),
        ("a_worker_with_no_profile_adds_no_bound_of_small", a_worker_with_no_profile_adds_no_bound_of_small),
        ("a_grace_past_a_duration_is_refused", a_grace_past_a_duration_is_refused),
        ("graces_whose_sum_overflows_refuse_the_start", graces_whose_sum_overflows_refuse_the_start),
        ("a_worker_refuses_an_unknown_grace", a_worker_refuses_an_unknown_grace),
        ("a_worker_refuses_an_agent_member_named_twice", a_worker_refuses_an_agent_member_named_twice),
        ("an_argument_that_is_not_text_is_refused_at_its_index", an_argument_that_is_not_text_is_refused_at_its_index),
    ]);
}

type Case = (&'static str, fn());

fn table<const N: usize>(cases: [Case; N]) {
    skein_rows::check(cases, |case| case());
}

#[test]
fn program_source_resources_have_positive_bounded_admission() {
    let limits = crate::stock().hub;
    assert_eq!(limits.program_sources, 12);
    assert!(jig_domain_host::valid_hub(&limits));
    assert!(!jig_domain_host::valid_hub(&jig_domain_host::Limits { source_interval: Duration::ZERO, ..limits }));
    assert!(!jig_domain_host::valid_hub(&jig_domain_host::Limits { source_timeout: Duration::ZERO, ..limits }));
}

#[test]
fn a_worker_configures_its_local_login_check_interval() {
    let bytes = fixtures::with(fixtures::WORKER_SMALL, b"\"check_interval_ms\":60000", b"\"check_interval_ms\":1250");
    let parsed = worker(&bytes).expect("positive source interval");
    assert_eq!(parsed.check_interval, Duration::from_millis(1_250));
    assert_eq!(parsed.limits.expect("configured limits").hub.source_interval, Duration::from_millis(1_250));
}
#[test]
fn a_zero_login_check_interval_names_the_invalid_field() {
    refuses(
        b"\"check_interval_ms\":60000",
        b"\"check_interval_ms\":0",
        fixtures::path(&[b"programs", b"check_interval_ms"]),
        What::Bound,
    );
}
#[test]
fn a_missing_login_check_interval_names_the_missing_field() {
    refuses(b"\"check_interval_ms\":60000", b"", fixtures::path(&[b"programs", b"check_interval_ms"]), What::Missing);
}

fn fixed_worker(bytes: &[u8], fixed: &crate::FixedWorker) -> Result<crate::Worker, crate::Problems> {
    let document = crate::document(bytes, &crate::DocumentLimits { bytes: 8192, tokens: 256, depth: 8 }).unwrap();
    crate::worker(&document, 0, &crate::Directories::working(), fixed)
}
fn without_fixed_members() -> Box<[u8]> {
    let bytes = fixtures::with(fixtures::WORKER_SMALL, br#""graces": {"grace_ms": 30000, "drain_ms": 10000, "agent_stop_ms": 10000, "delivery_ms": 20000, "save_ms": 5000},"#, b"");
    fixtures::with(&bytes, br#""programs":{"check_interval_ms":60000},"#, b"")
}
#[test]
fn fixed_worker_members_are_consumed_and_refused_whole() {
    for (fixed, name) in [
        (crate::FixedWorker { graces: Some(graces()), check_interval: None }, b"graces".as_slice()),
        (crate::FixedWorker { graces: None, check_interval: Some(Duration::from_secs(7)) }, b"programs".as_slice()),
    ] {
        let found = fixed_worker(fixtures::WORKER_SMALL, &fixed).unwrap_err();
        assert_eq!(found.problems.as_slice(), &[Problem { path: Path::root().member(name), what: What::Fixed }]);
    }
}
#[test]
fn fixed_worker_values_feed_the_stop_bound_and_source_interval() {
    let given = Graces {
        grace: Duration::from_secs(300),
        drain: Duration::from_secs(180),
        agent_stop: Duration::from_secs(5),
        delivery: Duration::from_secs(1),
        save: Duration::from_secs(1),
    };
    let interval = Duration::from_secs(7);
    let fixed = crate::FixedWorker { graces: Some(given), check_interval: Some(interval) };
    let worker = fixed_worker(&without_fixed_members(), &fixed).unwrap();
    assert_eq!(worker.graces, given);
    assert_eq!(worker.stop_bound, Duration::from_secs(326));
    assert_eq!(worker.check_interval, interval);
    assert_eq!(worker.limits.as_ref().expect("configured limits").hub.source_interval, interval);
    let zero = crate::FixedWorker { check_interval: Some(Duration::ZERO), ..fixed };
    let found = fixed_worker(&without_fixed_members(), &zero).unwrap_err();
    assert_eq!(found.problems.get(0).unwrap().path, Path::root().member(b"programs").member(b"check_interval_ms"));
    assert_eq!(found.problems.get(0).unwrap().what, What::Bound);
}

#[test]
fn worker_agent_and_limits_are_independently_optional() {
    for (agent, limits) in [(false, true), (true, false), (false, false)] {
        let mut input: Box<[u8]> = fixtures::WORKER_SMALL.into();
        if !agent {
            input = fixtures::with(
                &input,
                br#""agent": {"program": "jig-test", "arguments": ["agent"], "configuration": "agent.json"},"#,
                b"",
            );
        }
        if !limits {
            input = fixtures::with(&input, br#""limits": {"profile": "standard"},"#, b"");
        }
        let parsed = worker(&input).unwrap();
        assert_eq!(parsed.agent.is_some(), agent);
        assert_eq!(parsed.limits.is_some(), limits);
        let files = crate::worker_files(&parsed);
        assert_eq!(files.len(), if agent { 2 } else { 1 });
        assert_eq!(files.first().unwrap().path.as_ref(), b"secrets/worker-a.secret");
    }
}
