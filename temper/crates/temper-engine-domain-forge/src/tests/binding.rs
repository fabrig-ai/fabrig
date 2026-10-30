use super::*;
const ADDRESS: &[u8] = b"sam@example.com";
const SUBJECT: &[u8] = b"first-google-account";
fn bind(d: &mut Domain, address: &[u8], subject: &[u8]) -> Box<[Request]> {
    outputs(d, Event::Bind { request: Token::new(1), address: address.into(), subject: subject.into() })
}
fn row() -> Stored {
    Stored::Binding { address: ADDRESS.into(), subject: SUBJECT.into() }
}
#[test]
fn first_binding_saves_the_folded_address_and_answers() {
    assert_eq!(
        bind(&mut domain(), b"Sam@EXAMPLE.com", SUBJECT).as_ref(),
        [
            Request::Save { record: row(), read_afresh: None, release: None },
            Request::Bound { request: Token::new(1), answer: BindAnswer::Bound },
        ]
    );
}
#[test]
fn same_subject_again_is_bound_without_a_write() {
    let mut d = domain();
    bind(&mut d, ADDRESS, SUBJECT);
    assert_eq!(
        bind(&mut d, b"SAM@example.com", SUBJECT).as_ref(),
        [Request::Bound { request: Token::new(1), answer: BindAnswer::Bound },]
    );
}
#[test]
fn another_subject_is_refused_without_a_write() {
    let mut d = domain();
    bind(&mut d, ADDRESS, SUBJECT);
    assert_eq!(
        bind(&mut d, ADDRESS, b"another-google-account").as_ref(),
        [Request::Bound { request: Token::new(1), answer: BindAnswer::Another },]
    );
}
#[test]
fn restored_binding_keeps_refusing_another_subject() {
    let mut d = domain();
    outputs(&mut d, Event::Restore { record: row() });
    outputs(&mut d, Event::Restored { clock: skein_forge_client::RecoveryClock::Monotonic });
    assert_eq!(
        bind(&mut d, ADDRESS, b"another-google-account").as_ref(),
        [Request::Bound { request: Token::new(1), answer: BindAnswer::Another },]
    );
}
#[expect(clippy::disallowed_macros, reason = "test member addresses are numbered fixtures")]
fn restore_rows(count: u32) -> Domain {
    let mut d = domain();
    for number in 0..count {
        let address = alloc::format!("member{number}@example.com").into_bytes().into_boxed_slice();
        outputs(&mut d, Event::Restore { record: Stored::Binding { address, subject: SUBJECT.into() } });
    }
    d
}
#[test]
fn bindings_restore_at_the_capacity() {
    assert_eq!(restore_rows(LIMITS.bindings).restore_fit_failure(), None);
}
#[test]
fn a_binding_past_restart_capacity_is_refused() {
    let d = restore_rows(LIMITS.bindings.checked_add(1).unwrap());
    assert_eq!(
        d.restore_fit_failure(),
        Some(&RestoreFit {
            family: Some(RecordKind::Binding),
            set: RestoreSet::Rows(RecordKind::Binding),
            held: 4,
            additional: 1,
            limit: 4
        })
    );
}
#[test]
fn a_new_address_past_the_table_is_full_without_a_write() {
    let mut d = domain();
    for address in [b"a@example.com".as_slice(), b"b@example.com", b"c@example.com", b"d@example.com"] {
        bind(&mut d, address, SUBJECT);
    }
    assert_eq!(
        bind(&mut d, ADDRESS, SUBJECT).as_ref(),
        [Request::Bound { request: Token::new(1), answer: BindAnswer::Full }]
    );
}
