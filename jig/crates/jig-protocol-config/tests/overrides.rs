//! Named stock and fixture limits (jig/docs/design/protocol/limits.md).
use jig_protocol_config::{self as config, DocumentLimits, Limits, Path, Problems, What, overrides};

fn path(segments: &[&[u8]]) -> Path {
    let mut path = Path::root();
    for segment in segments {
        path = path.member(segment);
    }
    path
}

fn parse(bytes: &[u8]) -> Result<Limits, Problems> {
    let document =
        config::document(bytes, &DocumentLimits { bytes: 256, tokens: 128, depth: 8 }).expect("fixture JSON");
    let parsed = overrides::parse(&document, 0, &path(&[b"limits"]));
    if parsed.problems.is_empty() { Ok(parsed.limits) } else { Err(parsed.problems) }
}

fn problem(bytes: &[u8], segments: &[&[u8]], what: What) {
    let errors = parse(bytes).unwrap_err();
    assert_eq!(errors.problems.len(), 1);
    assert_eq!(errors.problems.get(0).unwrap().path, path(segments));
    assert_eq!(errors.problems.get(0).unwrap().what, what);
}

#[test]
fn named_limits_round_trip_between_both_profiles() {
    for (from, to) in [
        (jig_protocol_config_fixtures::small(), config::stock()),
        (config::stock(), jig_protocol_config_fixtures::small()),
    ] {
        let wanted = from;
        let mut got = to;
        let values = overrides::list(&wanted);
        assert_eq!(values.len(), u32::try_from(config::names::NAMES.len()).unwrap());
        for (index, (name, value)) in values.iter().enumerate() {
            for (prior, _) in values.iter().take(index) {
                assert_ne!(prior, name, "once: {name}");
            }
            overrides::apply(&mut got, name.as_bytes(), *value).unwrap();
        }
        for name in &config::names::NAMES {
            assert_eq!((name.value)(&got), (name.value)(&wanted), "{}", name.name);
        }
    }
}

#[test]
fn checkpoints_keep_defaults_to_two_and_accepts_only_one_through_sixty_four() {
    for limits in [jig_protocol_config_fixtures::small(), config::stock()] {
        assert_eq!(limits.store.checkpoints.keep, 2);
    }
    for (json, keep) in [
        (br#"{"overrides":{"store.checkpoints.keep":1}}"#.as_slice(), 1),
        (br#"{"overrides":{"store.checkpoints.keep":3}}"#, 3),
        (br#"{"overrides":{"store.checkpoints.keep":64}}"#, 64),
    ] {
        assert_eq!(parse(json).unwrap().store.checkpoints.keep, keep);
    }
    for json in
        [br#"{"overrides":{"store.checkpoints.keep":0}}"#.as_slice(), br#"{"overrides":{"store.checkpoints.keep":65}}"#]
    {
        problem(json, &[b"limits", b"overrides", b"store.checkpoints.keep"], What::Override);
    }
}
