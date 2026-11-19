//! Lent-terminal account forms at the client boundary (domain/terminal-login.md, 5).
use super::{c, fixtures, translate, w};
use jig_client_protocol_wire::Record as _;
use skein_lib::{Reader, Writer};
fn exits() -> [(c::LoginExit, w::LoginExit); 3] {
    [
        (c::LoginExit::Exited { code: 17 }, w::LoginExit::Exited(w::LoginExitExited { code: 17 })),
        (c::LoginExit::Killed { signal: 9 }, w::LoginExit::Killed(w::LoginExitKilled { signal: 9 })),
        (c::LoginExit::NotStarted, w::LoginExit::NotStarted),
    ]
}
fn login(cancelled: bool) -> (c::TerminalLogin, w::TerminalLogin) {
    (
        c::TerminalLogin { number: 1, provider: 2, name: b"work".as_slice().into(), cancelled },
        w::TerminalLogin { number: 1, provider: 2, name: b"work".as_slice().into(), cancelled },
    )
}
fn delivery(value: &w::DeliveryValue) -> c::Delivery {
    let mut out = Writer::new(skein_lib::length::usize_of(
        value.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"),
    ));
    value.write(&jig_client_protocol_wire::CEILINGS, &mut out).unwrap();
    translate::watches::delivery(&out.finish(), &fixtures::limits()).unwrap()
}
#[test]
fn login_changes_and_the_asked_answer_translate() {
    let limits = fixtures::limits();
    let mut changes = skein_lib::List::with_capacity(5);
    for row in [
        (
            c::AccountChange::LoginProgram { provider: 2, name: b"work".as_slice().into() },
            w::AccountChange::LoginProgram(w::AccountChangeLoginProgram {
                provider: 2,
                name: b"work".as_slice().into(),
            }),
        ),
        (
            c::AccountChange::CancelLogin { number: 1 },
            w::AccountChange::CancelLogin(w::AccountChangeCancelLogin { number: 1 }),
        ),
    ] {
        changes.push(row).unwrap();
    }
    for (exit, wire) in exits() {
        changes
            .push((
                c::AccountChange::LoginEnded { number: 1, exit },
                w::AccountChange::LoginEnded(w::AccountChangeLoginEnded { number: 1, exit: wire }),
            ))
            .unwrap();
    }
    for (holder, wire_holder) in [
        (c::AskAccountHolder::Own, w::AskAccountHolder::Own),
        (c::AskAccountHolder::Deployment, w::AskAccountHolder::Deployment),
    ] {
        for (change, expected) in &changes {
            let ask = c::Ask::AccountChange { holder, change: change.clone() };
            let wire = translate::asks::ask(&ask, limits.wire()).unwrap();
            let w::Ask::AccountChange(row) = &wire else { panic!("account change") };
            assert_eq!((&row.change), expected);
            assert_eq!((&row.holder), &wire_holder);
            let mut out = Writer::new(skein_lib::length::usize_of(
                wire.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture"),
            ));
            wire.write(&jig_client_protocol_wire::CEILINGS, &mut out).unwrap();
            let restored = w::Ask::decode(limits.wire(), &mut Reader::new(&out.finish())).unwrap();
            assert_eq!(translate::store::restore_ask(restored, &limits), Ok(ask));
        }
    }
    let answer = w::DocumentOutcome::AccountLoginAsked(w::OutcomeAccountLoginAsked { number: 1 });
    assert_eq!(
        translate::answers::document_outcome(&answer, &limits),
        Ok(c::DocumentOutcome::AccountLoginAsked { number: 1 })
    );
    for (wire, native) in [
        (w::Refusal::NoTerminal, c::Refusal::NoTerminal),
        (w::Refusal::LoginUnderWay, c::Refusal::LoginUnderWay),
        (w::Refusal::NotLending, c::Refusal::NotLending),
    ] {
        assert_eq!(translate::answers::refusal(&wire, &limits), Ok(native));
    }
}
#[test]
fn terminal_and_accounts_documents_preserve_login_facts() {
    let limits = fixtures::limits();
    for terminal in [0, u64::MAX] {
        assert_eq!(
            translate::watches::subject(&c::Subject::Terminal { terminal }, limits.wire()),
            Ok(w::Subject::Terminal(w::SubjectTerminal { terminal }))
        );
    }
    for cancelled in [false, true] {
        let (native, wire) = login(cancelled);
        for pair in [None, Some((native, wire))] {
            let (native, wire) = match pair {
                None => (None, None),
                Some((n, w)) => (Some(n), Some(w)),
            };
            let row = w::Terminal { login: wire.clone() };
            assert_eq!(
                delivery(&w::DeliveryValue::Terminal(row)),
                c::Delivery::Terminal(Box::new(c::Terminal { login: native.clone() }))
            );
            let mut parts = jig_client_test_records::accounts_record(None);
            parts.lendable = native.is_some();
            parts.lending = wire;
            let value = parts;
            let c::Delivery::Accounts(doc) = delivery(&w::DeliveryValue::Accounts(value)) else { panic!("accounts") };
            assert_eq!(doc.lendable, native.is_some());
            assert_eq!(doc.lending, native);
            assert_eq!(doc.lent_end, None);
        }
    }
    let mut ends = skein_lib::List::with_capacity(7);
    for row in [
        (c::LoginEnd::SignedIn { source: 19 }, w::LoginEnd::SignedIn(w::LoginEndSignedIn { source: 19 })),
        (c::LoginEnd::NotFinished { exit: None }, w::LoginEnd::NotFinished(w::LoginEndNotFinished { exit: None })),
        (c::LoginEnd::Cancelled, w::LoginEnd::Cancelled),
        (
            c::LoginEnd::Refused { refusal: c::AccountRefusal::Numbers },
            w::LoginEnd::Refused(w::LoginEndRefused { refusal: w::AccountRefusal::Numbers }),
        ),
    ] {
        ends.push(row).unwrap();
    }
    for (exit, wire) in exits() {
        ends.push((
            c::LoginEnd::NotFinished { exit: Some(exit) },
            w::LoginEnd::NotFinished(w::LoginEndNotFinished { exit: Some(wire) }),
        ))
        .unwrap();
    }
    for (end, wire) in ends.into_boxed() {
        let mut parts = jig_client_test_records::accounts_record(None);
        parts.lent_end = Some(w::AccountLentEnd { number: 1, end: wire });
        let doc = parts;
        let c::Delivery::Accounts(doc) = delivery(&w::DeliveryValue::Accounts(doc)) else { panic!("accounts") };
        assert_eq!(doc.lent_end, Some(c::AccountLentEnd { number: 1, end }));
    }
}
#[test]
fn terminal_names_are_validated_and_charged_to_the_document() {
    let (_, row) = login(false);
    let mut parts = row.clone();
    parts.name = b"../bad".as_slice().into();
    let invalid = parts;
    let mut room = 4;
    assert_eq!(
        translate::documents::terminal_login(Some(&invalid), &mut room).unwrap_err().reason,
        skein_codec::Reason::Malformed
    );
    translate::documents::terminal_login(Some(&row), &mut room).unwrap();
    assert_eq!(room, 0);
    let mut room = 3;
    assert_eq!(
        translate::documents::terminal_login(Some(&row), &mut room).unwrap_err().reason,
        skein_codec::Reason::Bound
    );
}
