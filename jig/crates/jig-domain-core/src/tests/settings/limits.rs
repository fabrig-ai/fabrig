//! Limit overrides are shape alone, not resolved figures (domain/engine/settings.md, 1).
use crate::settings::{
    OVERRIDE_NAME_BYTES, OVERRIDE_REASON_BYTES, OVERRIDES, Override, OverrideFault, check_overrides,
};
fn bytes(byte: u8, count: u32) -> Box<[u8]> {
    let mut bytes = skein_lib::bytes::zeroed(skein_lib::usize_of(count));
    bytes.fill(byte);
    bytes
}
fn entry(name: &[u8], reason: &[u8]) -> Override {
    Override { name: name.into(), value: u64::MAX, reason: reason.into() }
}
#[test]
fn empty_limits_are_well_shaped() {
    assert_eq!(check_overrides(&[]), Ok(()));
}
#[test]
fn overrides_at_their_bounds_need_no_known_names_or_figures() {
    let mut values = skein_lib::List::with_capacity(OVERRIDES);
    for at in 0..OVERRIDES {
        let mut name = bytes(b'_', OVERRIDE_NAME_BYTES);
        *name.get_mut(0).unwrap() = b'a'.checked_add(u8::try_from(at / 16).unwrap()).unwrap();
        *name.get_mut(1).unwrap() = b'a'.checked_add(u8::try_from(at % 16).unwrap()).unwrap();
        values.push(entry(&name, &bytes(b'r', OVERRIDE_REASON_BYTES))).unwrap();
    }
    assert_eq!(check_overrides(values.as_slice()), Ok(()));
    assert_eq!(check_overrides(&[entry(b"a.0_z", "理由".as_bytes())]), Ok(()));
}
#[test]
fn too_many_overrides_fail_by_count() {
    let mut values = skein_lib::List::with_capacity(OVERRIDES + 1);
    for _ in 0..=OVERRIDES {
        values.push(entry(b"a", b"reason")).unwrap();
    }
    assert_eq!(check_overrides(values.as_slice()), Err(OverrideFault::Count));
}
#[test]
fn malformed_names_fail_at_the_first_bad_override() {
    for name in [
        Box::default(),
        bytes(b'b', OVERRIDE_NAME_BYTES + 1),
        Box::from(b"Bad".as_slice()),
        Box::from(b"b-c".as_slice()),
        Box::from([0xff]),
    ] {
        assert_eq!(
            check_overrides(&[entry(b"a", b"reason"), entry(&name, b"reason"), entry(b"", b"")]),
            Err(OverrideFault::Name { at: 1 })
        );
    }
}
#[test]
fn unordered_or_repeated_names_fail_at_the_later_override() {
    for name in [b"tasks.depth".as_slice(), b"tasks.tasks"] {
        assert_eq!(
            check_overrides(&[entry(b"tasks.tasks", b"reason"), entry(name, b"reason")]),
            Err(OverrideFault::Order { at: 1 })
        );
    }
}
#[test]
fn malformed_reasons_fail_at_the_first_bad_override() {
    for reason in [Box::default(), bytes(b'r', OVERRIDE_REASON_BYTES + 1), Box::from([0xff])] {
        assert_eq!(
            check_overrides(&[entry(b"a", b"reason"), entry(b"b", &reason), entry(b"", b"")]),
            Err(OverrideFault::Reason { at: 1 })
        );
    }
}
#[test]
fn ill_shaped_limits_cannot_be_restored() {
    use crate::settings::{self as s, Failure, Fault, Setting, Value};
    let value = Value::Limits(Box::new([entry(b"tasks.tasks", b"reason"), entry(b"tasks.depth", b"reason")]));
    assert_eq!(
        s::check::restore_fit(Setting::Limits, &value, &crate::tests::limits()),
        Err(Failure { setting: Setting::Limits, fault: Fault::Shape, detail: None })
    );
}
