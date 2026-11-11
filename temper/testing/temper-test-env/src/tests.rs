use super::*;

#[test]
fn table_is_tempers_test_set() {
    let names: Vec<&str> = TESTS.variables.iter().map(|variable| variable.name).collect();
    assert_eq!(TESTS.tree, "temper");
    assert_eq!(
        names,
        [
            "TEMPER_TEST_BOX_PARENT_TEST",
            "TEMPER_TEST_BUNDLE",
            "TEMPER_TEST_E4_CAPTURE",
            "TEMPER_TEST_E4_LIFE",
            "TEMPER_TEST_E4_RESTART",
            "TEMPER_TEST_FORGEJO",
            "TEMPER_TEST_FORGEJO_RUNNER",
            "TEMPER_TEST_FORGE_PARENT_HELPER",
            "TEMPER_TEST_LIVE_ACCOUNT",
            "TEMPER_TEST_LIVE_CLAUDE",
            "TEMPER_TEST_LIVE_GITHUB_FIXTURES",
            "TEMPER_TEST_LIVE_GITHUB_KEEP",
            "TEMPER_TEST_LIVE_GITHUB_OWNER",
            "TEMPER_TEST_LIVE_KEEP",
            "TEMPER_TEST_LIVE_NOTICE",
            "TEMPER_TEST_LIVE_STORE",
            "TEMPER_TEST_MANAGED_CLIENT",
            "TEMPER_TEST_MANAGED_EVIDENCE",
            "TEMPER_TEST_MANAGED_MODE",
            "TEMPER_TEST_MANAGED_PERSON",
            "TEMPER_TEST_MANAGED_PREFIX",
            "TEMPER_TEST_MANAGED_SCRATCH",
            "TEMPER_TEST_MANAGED_UPSTREAM",
            "TEMPER_TEST_PLAYGROUND",
            "TEMPER_TEST_RATE",
            "TEMPER_TEST_RECORD_RELEASE",
            "TEMPER_TEST_SITTING_WAIT_SECS",
            "TEMPER_TEST_WRAPPER",
        ]
    );
    for variable in TESTS.variables {
        assert_eq!(variable.set, Set::Test, "{}", variable.name);
        assert_eq!(variable.way, Way::Reads, "{}", variable.name);
        assert!(variable.name.starts_with(concat!("TEMPER", "_TEST_")));
    }
    assert!(names.windows(2).all(|pair| pair[0].as_bytes() < pair[1].as_bytes()));
}

#[test]
fn document_is_the_table() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../docs/design/cli/test-env.md");
    let rendered = TESTS.document();
    if rewrite() {
        std::fs::write(path, rendered).expect("write test table document");
    } else {
        assert!(
            std::fs::read(path).expect("test table document") == rendered.as_bytes(),
            "test-env.md: regenerate with SKEIN_CODEGEN_WRITE=1 and this test"
        );
    }
}

#[test]
fn lookup_is_declared_nonempty_bytes() {
    use std::os::unix::ffi::OsStrExt;

    let unrelated = [(Box::from(b"OTHER".as_slice()), Box::from(b"1".as_slice()))];
    assert_eq!(value_in(&RATE, &[]), None);
    assert_eq!(value_in(&RATE, &unrelated), None);
    for bytes in [b"".as_slice(), b"1", b"value-\xff"] {
        let pairs = [(Box::from(RATE.name.as_bytes()), Box::from(bytes))];
        let value = value_in(&RATE, &pairs);
        let expected = if bytes.is_empty() { None } else { Some(bytes) };
        assert_eq!(value.as_deref().map(std::ffi::OsStr::as_bytes), expected);
    }
}

#[test]
fn rewrite_requires_exactly_one() {
    assert!(!rewrite_in(&[]));
    for (bytes, expected) in [(b"1".as_slice(), true), (b"", false), (b"0", false), (b"yes", false)] {
        let pairs = [(Box::from(SKEIN_CODEGEN_WRITE.name.as_bytes()), Box::from(bytes))];
        assert_eq!(rewrite_in(&pairs), expected, "{bytes:?}");
    }
}
