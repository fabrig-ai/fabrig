//! Program inventories cross bytes intact and share the document's native byte budget.
use super::{Limits, c, fixtures, w};
use crate::translate::programs as decode;
use skein_lib::{List, Writer};
fn login(name: &[u8], state: Option<w::ProgramState>) -> w::ProgramLogin {
    w::ProgramLogin::new(
        &w::CEILINGS,
        w::ProgramLoginParts { name: name.into(), state, sign_out: None, cleanup: false, cut: false },
    )
    .expect("literal login")
}
fn piece(row: &w::ProgramLogin) -> Box<[u8]> {
    let mut out = Writer::new(skein_lib::length::usize_of(row.measure()).checked_add(4).expect("piece prefix"));
    out.put(&row.measure().to_be_bytes()).expect("prefix");
    row.encode(&mut out).expect("measured login");
    out.finish()
}
#[test]
fn program_inventory_preserves_every_login_state() {
    let failed = w::ProgramFailed::new(
        &w::CEILINGS,
        w::ProgramFailedParts { reason: w::ProgramFailure::Deadline, detail: b"bounded".as_slice().into() },
    )
    .expect("failure");
    let method =
        w::ProgramOtherwise::new(&w::CEILINGS, w::ProgramOtherwiseParts { method: b"api-key".as_slice().into() })
            .expect("method");
    for (wire, native) in [
        (None, None),
        (Some(w::ProgramState::Usable), Some(c::ProgramState::Usable)),
        (Some(w::ProgramState::NotSignedIn), Some(c::ProgramState::NotSignedIn)),
        (
            Some(w::ProgramState::SignedInOtherwise(method)),
            Some(c::ProgramState::SignedInOtherwise { method: b"api-key".as_slice().into() }),
        ),
        (
            Some(w::ProgramState::Failed(failed)),
            Some(c::ProgramState::Failed { reason: c::ProgramFailure::Deadline, detail: b"bounded".as_slice().into() }),
        ),
    ] {
        let mut room = 4096;
        let rows =
            decode::logins(&piece(&login(b"personal", wire)), &fixtures::limits(), &mut room).expect("inventory");
        assert_eq!(rows.get(0).expect("one row").state, native);
    }
}
#[test]
fn malformed_program_names_are_refused() {
    let mut room = 4096;
    let result = decode::logins(&piece(&login(b"Bad Name", None)), &fixtures::limits(), &mut room);
    assert_eq!(result.err(), Some(w::Problem { path: w::Path::OwnedRecord, reason: skein_codec::Reason::Malformed }));
}
#[test]
fn malformed_payment_methods_are_refused() {
    let method =
        w::ProgramOtherwise::new(&w::CEILINGS, w::ProgramOtherwiseParts { method: b"bad method".as_slice().into() })
            .expect("wire bytes");
    let mut room = 4096;
    let result = decode::logins(
        &piece(&login(b"personal", Some(w::ProgramState::SignedInOtherwise(method)))),
        &fixtures::limits(),
        &mut room,
    );
    assert_eq!(result.err(), Some(w::Problem { path: w::Path::OwnedRecord, reason: skein_codec::Reason::Malformed }));
}
#[test]
fn native_program_budget_accepts_its_exact_bound() {
    let mut room = List::<c::ProgramLogin>::price(1).expect("one row").checked_add(1).expect("name");
    let rows =
        decode::logins(&piece(&login(b"p", Some(w::ProgramState::Usable))), &fixtures::limits(), &mut room).unwrap();
    assert_eq!(rows.len(), 1);
    assert_eq!(room, 0);
}
#[test]
fn native_program_budget_refuses_one_byte_less() {
    let mut room = List::<c::ProgramLogin>::price(1).expect("one row");
    let result = decode::logins(&piece(&login(b"p", Some(w::ProgramState::Usable))), &fixtures::limits(), &mut room);
    assert_eq!(result.err(), Some(w::Problem { path: w::Path::OwnedRecord, reason: skein_codec::Reason::Bound }));
}

#[test]
fn program_lists_over_the_native_budget_are_refused() {
    let one = piece(&login(b"p", Some(w::ProgramState::Usable)));
    let mut bytes = Writer::new(one.len().checked_mul(2).expect("two pieces"));
    bytes.put(&one).expect("first");
    bytes.put(&one).expect("second");
    let mut room = List::<c::ProgramLogin>::price(1).expect("one row").checked_add(1).expect("name");
    let result = decode::logins(&bytes.finish(), &fixtures::limits(), &mut room);
    assert_eq!(result.err(), Some(w::Problem { path: w::Path::OwnedRecord, reason: skein_codec::Reason::Bound }));
}
#[test]
fn configured_host_names_stay_whole_below_the_text_limit() {
    let row = w::ProgramHost::new(
        &w::CEILINGS,
        w::ProgramHostParts {
            name: b"configured-worker".as_slice().into(),
            channel: 17,
            connected: true,
            awaiting_logins: true,
            programs: Box::new([]),
            cut: false,
        },
    )
    .expect("host");
    let mut bytes = Writer::new(skein_lib::length::usize_of(row.measure()).checked_add(4).expect("prefix"));
    bytes.put(&row.measure().to_be_bytes()).expect("prefix");
    row.encode(&mut bytes).expect("row");
    let base = fixtures::limits();
    let limits =
        Limits::new(jig_client_domain::Limits { text: 1, turn_text: 1, ..*base.client() }, base.exchanges()).unwrap();
    let mut room = 4096;
    let rows = decode::hosts(&bytes.finish(), &limits, &mut room).expect("whole name");
    assert_eq!(rows.get(0).expect("host").name.as_ref(), b"configured-worker");
}

#[test]
fn a_host_rule_survives_wire_receipt() {
    let person = w::HostPerson::new(&w::CEILINGS, w::HostPersonParts { party: 7 }).expect("person");
    for (rule, expected) in [
        (w::HostRule::Open, c::HostRule::Open),
        (w::HostRule::Person(person), c::HostRule::Person(7)),
        (w::HostRule::Several, c::HostRule::Several),
        (w::HostRule::Unknown, c::HostRule::Unknown),
    ] {
        let row = w::Host::new(
            &w::CEILINGS,
            w::HostParts {
                name: None,
                skein_bools: [false, false, true, false],
                rule,
                programs: Box::new([]),
                channel: 17,
                kind: w::HostKind::Worker,
                slots: 1,
                used: 0,
                free: 1,
                grace_until: None,
                grace_remaining: None,
                runs: Box::new([]),
            },
        )
        .expect("host");
        let mut bytes = Writer::new(skein_lib::length::usize_of(row.measure()).checked_add(4).expect("prefix"));
        bytes.put(&row.measure().to_be_bytes()).expect("prefix");
        row.encode(&mut bytes).expect("row");
        let mut room = 4096;
        let limits = fixtures::limits();
        let rows = crate::translate::documents::host_piece(&bytes.finish(), &limits, limits.client().rows, &mut room)
            .expect("host receipt");
        assert_eq!(rows.get(0).expect("host").rule, expected);
    }
}
