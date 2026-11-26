use skein_shell::{Set, Variable, Way};

use crate::test_env::{EnvironmentEntry, FUZZY_CAMPAIGN, FUZZY_CASE, Mode, TESTS, mode_in};

#[test]
fn table_is_skeins_fuzzy_test_set() {
    assert_eq!(TESTS.tree, "skein");
    let names: Vec<_> = TESTS.variables.iter().map(|variable| variable.name).collect();
    assert_eq!(names, ["SKEIN_TEST_FUZZY_CAMPAIGN", "SKEIN_TEST_FUZZY_CASE"]);
    for variable in TESTS.variables {
        assert_eq!(variable.set, Set::Test, "{}", variable.name);
        assert_eq!(variable.way, Way::Reads, "{}", variable.name);
    }
}

#[test]
#[expect(clippy::disallowed_methods, reason = "a test harness: reads SKEIN_CODEGEN_WRITE, the codegen rewrite switch")]
fn document_is_the_table() {
    let path = skein_scratch::paths::tree().join("docs/design/cli/test-env.md");
    let rendered = TESTS.document();
    if std::env::var_os("SKEIN_CODEGEN_WRITE").is_some_and(|value| value == "1") {
        std::fs::create_dir_all(path.parent().expect("test table document has a parent"))
            .expect("test table directory");
        std::fs::write(path, rendered).expect("write test table document");
    } else {
        assert!(
            std::fs::read(path).expect("test table document") == rendered.as_bytes(),
            "test-env.md: regenerate with SKEIN_CODEGEN_WRITE=1 and this test"
        );
    }
}

fn pair(variable: Variable, value: &[u8]) -> EnvironmentEntry {
    (Box::from(variable.name.as_bytes()), Box::from(value))
}

#[test]
fn absent_and_empty_are_unset_and_hex_accepts_both_cases() {
    assert_eq!(mode_in(&[]), Ok(Mode::Gate));
    assert_eq!(mode_in(&[pair(FUZZY_CAMPAIGN, b""), pair(FUZZY_CASE, b"")]), Ok(Mode::Gate));
    for bytes in [b"0123456789abcdef", b"0123456789ABCDEF"] {
        let campaign = pair(FUZZY_CAMPAIGN, bytes);
        let replay = pair(FUZZY_CASE, bytes);
        let seed = 0x0123_4567_89ab_cdef;
        assert_eq!(mode_in(core::slice::from_ref(&campaign)), Ok(Mode::Campaign(seed)));
        assert_eq!(mode_in(core::slice::from_ref(&replay)), Ok(Mode::Replay(seed)));
        assert_eq!(mode_in(&[campaign, pair(FUZZY_CASE, b"")]), Ok(Mode::Campaign(seed)));
        assert_eq!(mode_in(&[pair(FUZZY_CAMPAIGN, b""), replay]), Ok(Mode::Replay(seed)));
    }
}

#[test]
fn malformed_values_name_the_variable_and_bytes() {
    for variable in [FUZZY_CAMPAIGN, FUZZY_CASE] {
        for bytes in [
            b"0123456789abcde".as_slice(),
            b"0123456789abcdef0",
            b"0x0123456789abcd",
            b"+0123456789abcde",
            b"0123456789abcdeg",
            b"0123456789abcde\xff",
        ] {
            let error = mode_in(&[pair(variable, bytes)]).expect_err("not sixteen ASCII hex digits");
            assert_eq!(
                error,
                format!("fuzzy setup: {}=\"{}\" must be sixteen hex digits", variable.name, bytes.escape_ascii())
            );
        }
    }
}

#[test]
fn both_nonempty_values_name_both_variables_and_values() {
    let pairs = [pair(FUZZY_CAMPAIGN, b"0123456789abcdef"), pair(FUZZY_CASE, b"000000000000002a")];
    assert_eq!(
        mode_in(&pairs),
        Err(concat!(
            "fuzzy setup: SKEIN_TEST_FUZZY_CAMPAIGN=\"0123456789abcdef\" and ",
            "SKEIN_TEST_FUZZY_CASE=\"000000000000002a\" cannot both be set"
        )
        .into())
    );
}
