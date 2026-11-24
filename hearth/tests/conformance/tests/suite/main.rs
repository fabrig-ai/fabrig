use hearth_conformance_world::resolve;
use hearth_protocol_mmdebstrap::{
    limits::{self, Limits},
    vector,
};
use hearth_test_mirror::Pin;

mod resolve_stanzas;

#[test]
fn a_small_include_limit_splits_the_vector() {
    let invocation = vector::simulate(
        &resolve::inputs(&Pin::committed(), resolve::case("test-boot")),
        b"/scratch",
        None,
        &Limits { include: 32, ..limits::SHIPPED },
    )
    .expect("split request");
    assert!(invocation.args.iter().filter(|arg| arg.starts_with(b"--include=")).count() > 1);
}

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
