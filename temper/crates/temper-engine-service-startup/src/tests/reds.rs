//! A goal's red bounds (domain/forge/reds.md, 7).
#[test]
fn red_limit_overrides_outside_the_range_refuse_start_naming_the_limit() {
    for name in [b"forge.branch_reds".as_slice(), b"forge.red_key_bytes"] {
        for value in [0_u64, 1025, u64::MAX] {
            assert_eq!(
                crate::limits::apply(&mut crate::limits::stock(), name, value),
                Err(crate::limits::Refused::Range)
            );
        }
    }
}
