use jig_protocol_config::{self as config, DocumentLimits, Path, Profile};

#[test]
fn fixtures_small_is_the_small_profile() {
    let document = config::document(br#"{"profile":"small"}"#, &DocumentLimits { bytes: 256, tokens: 128, depth: 8 })
        .expect("fixture JSON");
    let parsed = config::overrides::parse(&document, 0, &Path::root().member(b"limits"));
    assert!(parsed.problems.is_empty());
    assert_eq!(crate::small().core.tasks.tasks, config::profile(Profile::Small).core.tasks.tasks);
    assert_eq!(crate::small(), config::profile(Profile::Small));
    let mut expected = crate::small();
    // The reader derives the carrier from two streams plus four ordinary exchanges
    // (jig/docs/design/protocol/limits.md, derived in the reader).
    expected.clients.exchanges = 6;
    assert_eq!(parsed.limits, expected);
}
