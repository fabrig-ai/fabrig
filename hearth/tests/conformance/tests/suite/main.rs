use hearth_conformance_world::resolve;

mod resolve_stanzas;

#[test]
fn no_allowed_difference_touches_test_min_or_test_boot_s_closure() {
    use hearth_conformance_world::fake_mmdebstrap::{ALLOWED, What};
    assert!(ALLOWED.len() <= 4);
    for row in ALLOWED {
        assert!(!row.reason.is_empty(), "{row:?}");
        if let Some(name) = row.case {
            assert!(resolve::CASES.iter().any(|case| case.name == name), "{row:?}");
            assert!(name != "test-min" && name != "test-boot", "{row:?}");
        } else {
            assert!(row.what == What::ListSizes || row.what == What::StanzaFields, "{row:?}");
        }
    }
}
