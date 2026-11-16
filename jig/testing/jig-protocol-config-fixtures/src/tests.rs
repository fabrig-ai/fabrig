use jig_protocol_config::{self as config, DocumentLimits, Path, Profile};

#[test]
fn fixtures_small_is_the_small_profile() {
    let document = config::document(br#"{"profile":"small"}"#, &DocumentLimits { bytes: 256, tokens: 128, depth: 8 })
        .expect("fixture JSON");
    let parsed = config::overrides::parse(&document, 0, &Path::root().member(b"limits"));
    assert!(parsed.problems.is_empty());
    assert_eq!(crate::small().core.tasks.tasks, config::profile(Profile::Small).core.tasks.tasks);
    assert_eq!(crate::small(), config::profile(Profile::Small));
    assert_eq!(parsed.limits, crate::small());
}
