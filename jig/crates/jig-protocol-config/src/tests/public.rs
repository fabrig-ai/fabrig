//! Public reading's startup contract (jig/docs/design/protocol/limits.md).
#![expect(
    clippy::disallowed_types,
    clippy::disallowed_methods,
    clippy::disallowed_macros,
    reason = "ordinary Rust test fixtures build bounded JSON documents and tables of overrides"
)]
use super::fixtures;
use crate::{Limits, Path, Problems, Profile, What, overrides};
use jig_domain_core_vocabulary::PublicIndexLimits;
use jig_protocol_engine_clients::PublicLimits;

// Example deployment figures from domain/people/public-reading.md, 5 and 9; off in profiles.
const PUBLIC: PublicLimits = PublicLimits {
    reads: 8,
    rate: 20,
    rate_burst: 20,
    bytes: 2_097_152,
    bytes_burst: 2_097_152,
    answer_bytes: 262_144,
    max_age: 5,
};
const INDEX: PublicIndexLimits = PublicIndexLimits { goals: 20, ended: 20, lines: 50, depth: 16 };
const OVERRIDES: &[(&str, u64)] = &[
    ("clients.public.reads", 8),
    ("clients.public.rate", 20),
    ("clients.public.rate_burst", 20),
    ("clients.public.bytes", 2_097_152),
    ("clients.public.bytes_burst", 2_097_152),
    ("clients.public.answer_bytes", 262_144),
    ("clients.public.max_age", 5),
    ("clients_http.public_calls", 8),
    ("core.public.goals", 20),
    ("core.public.ended", 20),
    ("core.public.lines", 50),
    ("core.public.depth", 16),
];

fn configuration(values: &[(&str, u64)]) -> String {
    let members = values.iter().map(|(name, value)| format!("\"{name}\":{value}")).collect::<Vec<_>>().join(",");
    format!("{{\"profile\":\"small\",\"overrides\":{{{members}}}}}")
}
fn parse(values: &[(&str, u64)], prefix: &Path) -> overrides::Parsed {
    let bytes = configuration(values);
    let document =
        crate::document(bytes.as_bytes(), &crate::DocumentLimits { bytes: 4096, tokens: 256, depth: 8 }).unwrap();
    overrides::parse(&document, 0, prefix)
}
fn start(values: &[(&str, u64)]) -> Result<crate::Engine, Problems> {
    let limits = format!("\"limits\": {}", configuration(values));
    let bytes = fixtures::with(fixtures::ENGINE_SMALL, br#""limits": {"profile": "small"}"#, limits.as_bytes());
    fixtures::engine(&bytes)
}
fn public_limits() -> Limits {
    let mut limits = crate::profile(Profile::Small);
    limits.clients.public = PUBLIC;
    limits.clients_http.public_calls = PUBLIC.reads;
    limits.core.public = INDEX;
    limits
}

fn changed(changes: &[(&'static str, u64)]) -> Vec<(&'static str, u64)> {
    let mut values = OVERRIDES.to_vec();
    for (name, value) in changes {
        let (_, configured) = values.iter_mut().find(|(key, _)| key == name).expect("known override");
        *configured = *value;
    }
    values
}

#[test]
fn every_profile_turns_public_reading_off() {
    for (profile, name) in [(Profile::Small, "small"), (Profile::Standard, "standard")] {
        let limits = crate::profile(profile);
        assert_eq!(limits.clients.public, PublicLimits::OFF);
        assert!(!limits.clients.public.on());
        assert_eq!(limits.clients_http.public_calls, 0);
        assert_eq!(limits.core.public, PublicIndexLimits::OFF);
        let replacement = format!("\"profile\": \"{name}\"");
        let bytes = fixtures::with(fixtures::ENGINE_SMALL, br#""profile": "small""#, replacement.as_bytes());
        assert!(fixtures::engine(&bytes).is_ok(), "{profile:?}");
    }
}

#[test]
fn public_overrides_parse_at_their_paths() {
    let prefix = Path::root().member(b"limits");
    // Distinct neighbouring values also catch crossed assignments.
    let values = changed(&[
        ("clients.public.rate_burst", 21),
        ("clients.public.bytes_burst", 3_145_728),
        ("clients_http.public_calls", 7),
        ("core.public.ended", 19),
    ]);
    let mut expected = public_limits();
    expected.clients.public.rate_burst = 21;
    expected.clients.public.bytes_burst = 3_145_728;
    expected.clients_http.public_calls = 7;
    expected.core.public.ended = 19;
    let parsed = parse(&values, &prefix);
    assert!(parsed.problems.is_empty(), "{:?}", parsed.problems);
    assert_eq!(parsed.limits, expected);
    assert!(parsed.limits.clients.public.on());
    // Zero is representable for every figure, including rate and HTTP capacity.
    let zero: Vec<_> = OVERRIDES.iter().map(|(name, _)| (*name, 0)).collect();
    let parsed = parse(&zero, &prefix);
    assert!(parsed.problems.is_empty(), "{:?}", parsed.problems);
    assert_eq!(parsed.limits, crate::profile(Profile::Small));
    for (name, maximum) in [
        ("clients.public.reads", u64::from(u32::MAX)),
        ("clients.public.rate", u64::from(u32::MAX)),
        ("clients.public.rate_burst", u64::from(u32::MAX)),
        ("clients.public.answer_bytes", u64::from(u32::MAX)),
        ("clients.public.max_age", u64::from(u32::MAX)),
        ("clients_http.public_calls", u64::from(u32::MAX)),
        ("core.public.goals", u64::from(u16::MAX)),
        ("core.public.ended", u64::from(u16::MAX)),
        ("core.public.lines", u64::from(u16::MAX)),
        ("core.public.depth", u64::from(u16::MAX)),
    ] {
        let mut limits = crate::profile(Profile::Small);
        assert_eq!(overrides::apply(&mut limits, name.as_bytes(), maximum), Ok(()), "{name}");
        let parsed = parse(&[(name, maximum + 1)], &prefix);
        assert_eq!(parsed.problems.problems.len(), 1, "{name}");
        let problem = parsed.problems.problems.get(0).unwrap();
        assert_eq!(problem.path, prefix.member(b"overrides").member(name.as_bytes()));
        assert_eq!(problem.what, What::Override);
    }
    let parsed = parse(&[("clients.public.bytes", u64::MAX), ("clients.public.bytes_burst", u64::MAX)], &prefix);
    assert!(parsed.problems.is_empty());
    assert_eq!((parsed.limits.clients.public.bytes, parsed.limits.clients.public.bytes_burst), (u64::MAX, u64::MAX));
}

#[test]
fn public_relations_refuse_the_start_at_their_paths() {
    assert!(start(OVERRIDES).is_ok(), "proposed deployment figures pass");
    let cases: &[(&str, &[(&str, u64)])] = &[
        ("clients.public.rate_burst", &[("clients.public.rate_burst", 19)]),
        ("clients.public.bytes_burst", &[("clients.public.bytes_burst", 2_097_151)]),
        ("clients.public.answer_bytes", &[("clients.public.answer_bytes", 2_097_153)]),
        ("clients_http.public_calls", &[("clients_http.public_calls", 9)]),
        // Isolate positive reads from the HTTP carrier relation.
        ("clients.public.reads", &[("clients.public.reads", 0), ("clients_http.public_calls", 0)]),
        ("core.public.goals", &[("core.public.goals", 0)]),
        ("core.public.ended", &[("core.public.ended", 0)]),
        ("core.public.lines", &[("core.public.lines", 0)]),
        ("core.public.depth", &[("core.public.depth", 0)]),
        ("clients.public.max_age", &[("clients.public.max_age", 61)]),
    ];
    for (name, changes) in cases {
        let values = changed(changes);
        let expected = crate::Problem {
            path: Path::root().member(b"limits").member(b"overrides").member(name.as_bytes()),
            what: What::Bound,
        };
        let problems = start(&values).unwrap_err();
        assert_eq!(problems.problems.as_slice(), &[expected], "{name}");
        let prefix = Path::root().member(b"engine").member(b"limits");
        let parsed = parse(&values, &prefix);
        assert_eq!(parsed.problems.problems.len(), 1, "{name}");
        assert_eq!(parsed.problems.problems.get(0).unwrap().path, prefix.member(b"overrides").member(name.as_bytes()));
    }
    // 2 * 65,535 goals * 65,535 lines * 128 bytes * three projects exceeds u32 bytes.
    let values = changed(&[
        ("core.public.goals", u64::from(u16::MAX)),
        ("core.public.ended", u64::from(u16::MAX)),
        ("core.public.lines", u64::from(u16::MAX)),
    ]);
    let problems = start(&values).unwrap_err();
    assert_eq!(
        problems.problems.as_slice(),
        &[crate::Problem {
            path: Path::root().member(b"limits").member(b"overrides").member(b"core.public.lines"),
            what: What::Bound,
        }]
    );
}

#[test]
fn a_zero_rate_checks_only_max_age() {
    let inconsistent = [
        ("clients.public.reads", 0),
        ("clients.public.rate", 0),
        ("clients.public.rate_burst", 1),
        ("clients.public.bytes", u64::MAX),
        ("clients.public.bytes_burst", 0),
        ("clients.public.answer_bytes", u64::from(u32::MAX)),
        ("clients_http.public_calls", u64::from(u32::MAX)),
        ("core.public.goals", u64::from(u16::MAX)),
        ("core.public.ended", u64::from(u16::MAX)),
        ("core.public.lines", u64::from(u16::MAX)),
        ("core.public.depth", 0),
        ("clients.public.max_age", 60),
    ];
    start(&inconsistent).expect("zero rate checks only max_age");
    let mut past = inconsistent.to_vec();
    for (name, value) in &mut past {
        if *name == "clients.public.max_age" {
            *value = 61;
        }
    }
    let problems = start(&past).unwrap_err();
    assert_eq!(
        problems.problems.as_slice(),
        &[crate::Problem {
            path: Path::root().member(b"limits").member(b"overrides").member(b"clients.public.max_age"),
            what: What::Bound,
        }]
    );
}

#[test]
fn public_index_memory_counts_both_goal_sets_and_every_project() {
    // Two lines of 128 bytes per project: 16,777,215 projects fit u32 bytes;
    // one more project needs exactly 2^32 bytes and refuses the start.
    for (projects, refused) in [(16_777_215, false), (16_777_216, true)] {
        let mut limits = public_limits();
        limits.core.public = PublicIndexLimits { goals: 1, ended: 1, lines: 1, depth: 1 };
        limits.core.people.projects = projects;
        let mut problems = Problems::empty();
        crate::clients::check_public(&limits, &mut problems);
        assert_eq!(!problems.is_empty(), refused, "{projects} projects");
        for problem in &problems.problems {
            assert_eq!(problem.path, crate::domain::codec_path(b"core.public.lines"));
            assert_eq!(problem.what, What::Bound);
        }
    }
}
