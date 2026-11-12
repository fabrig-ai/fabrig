//! Lent-terminal account forms at the client boundary (domain/terminal-login.md, 5).
use super::{c, fixtures, translate, w};
use skein_lib::{Reader, Writer};
fn exits() -> [(c::LoginExit, w::LoginExit); 3] {
    [
        (c::LoginExit::Exited { code: 17 }, w::LoginExit::Exited(w::LoginExitExited::new(&w::CEILINGS, w::LoginExitExitedParts { code: 17 }).unwrap())),
        (c::LoginExit::Killed { signal: 9 }, w::LoginExit::Killed(w::LoginExitKilled::new(&w::CEILINGS, w::LoginExitKilledParts { signal: 9 }).unwrap())),
        (c::LoginExit::NotStarted, w::LoginExit::NotStarted),
    ]
}
fn login(cancelled: bool) -> (c::TerminalLogin, w::TerminalLogin) {
    (c::TerminalLogin { number: 1, provider: 2, name: b"work".as_slice().into(), cancelled },
    w::TerminalLogin::new(&w::CEILINGS, w::TerminalLoginParts { number: 1, provider: 2, name: b"work".as_slice().into(), cancelled }).unwrap())
}
fn delivery(value: &w::DeliveryValue) -> c::Delivery {
    let mut out = Writer::new(skein_lib::length::usize_of(value.measure()));
    value.encode(&mut out).unwrap();
    translate::watches::delivery(&out.finish(), &fixtures::limits()).unwrap()
}
#[test]
fn login_changes_and_the_asked_answer_translate() {
    let limits = fixtures::limits();
    let mut changes = skein_lib::List::with_capacity(5);
    for row in [
        (c::AccountChange::LoginProgram { provider: 2, name: b"work".as_slice().into() },
        w::AccountChange::LoginProgram(w::AccountChangeLoginProgram::new(limits.wire(), w::AccountChangeLoginProgramParts { provider: 2, name: b"work".as_slice().into() }).unwrap())),
        (c::AccountChange::CancelLogin { number: 1 },
        w::AccountChange::CancelLogin(w::AccountChangeCancelLogin::new(limits.wire(), w::AccountChangeCancelLoginParts { number: 1 }).unwrap())),
    ] { changes.push(row).unwrap(); }
    for (exit, wire) in exits() {
        changes.push((c::AccountChange::LoginEnded { number: 1, exit }, w::AccountChange::LoginEnded(w::AccountChangeLoginEnded::new(limits.wire(), w::AccountChangeLoginEndedParts { number: 1, exit: wire }).unwrap()))).unwrap();
    }
    for (holder, wire_holder) in [(c::AskAccountHolder::Own, w::AskAccountHolder::Own), (c::AskAccountHolder::Deployment, w::AskAccountHolder::Deployment)] {
        for (change, expected) in &changes {
            let ask = c::Ask::AccountChange { holder, change: change.clone() };
            let wire = translate::asks::ask(&ask, limits.wire()).unwrap();
            let w::Ask::AccountChange(row) = &wire else { panic!("account change") };
            assert_eq!(row.change(), expected);
            assert_eq!(row.holder(), &wire_holder);
            let mut out = Writer::new(skein_lib::length::usize_of(wire.measure()));
            wire.encode(&mut out).unwrap();
            let restored = w::Ask::decode(limits.wire(), &mut Reader::new(&out.finish())).unwrap();
            assert_eq!(translate::store::restore_ask(restored, &limits), Ok(ask));
        }
    }
    let answer = w::DocumentOutcome::AccountLoginAsked(w::OutcomeAccountLoginAsked::new(limits.wire(), w::OutcomeAccountLoginAskedParts { number: 1 }).unwrap());
    assert_eq!(translate::answers::document_outcome(&answer, &limits), Ok(c::DocumentOutcome::AccountLoginAsked { number: 1 }));
    for (wire, native) in [(w::Refusal::NoTerminal, c::Refusal::NoTerminal), (w::Refusal::LoginUnderWay, c::Refusal::LoginUnderWay), (w::Refusal::NotLending, c::Refusal::NotLending)] {
        assert_eq!(translate::answers::refusal(&wire, &limits), Ok(native));
    }
}
#[test]
fn terminal_and_accounts_documents_preserve_login_facts() {
    let limits = fixtures::limits();
    for terminal in [0, u64::MAX] {
        assert_eq!(translate::watches::subject(&c::Subject::Terminal { terminal }, limits.wire()), Ok(w::Subject::Terminal(w::SubjectTerminal::new(limits.wire(), w::SubjectTerminalParts { terminal }).unwrap())));
    }
    for cancelled in [false, true] {
        let (native, wire) = login(cancelled);
        for pair in [None, Some((native, wire))] {
            let (native, wire) = match pair { None => (None, None), Some((n, w)) => (Some(n), Some(w)) };
            let row = w::Terminal::new(limits.wire(), w::TerminalParts { login: wire.clone() }).unwrap();
            assert_eq!(delivery(&w::DeliveryValue::Terminal(row)), c::Delivery::Terminal(Box::new(c::Terminal { login: native.clone() })));
            let mut parts = jig_client_test_records::accounts_record(None).into_parts();
            parts.skein_bools[0] = native.is_some();
            parts.lending = wire;
            let value = w::Accounts::new(limits.wire(), parts).unwrap();
            let c::Delivery::Accounts(doc) = delivery(&w::DeliveryValue::Accounts(value)) else { panic!("accounts") };
            assert_eq!(doc.lendable, native.is_some());
            assert_eq!(doc.lending, native);
            assert_eq!(doc.lent_end, None);
        }
    }
    let mut ends = skein_lib::List::with_capacity(7);
    for row in [
        (c::LoginEnd::SignedIn { source: 19 }, w::LoginEnd::SignedIn(w::LoginEndSignedIn::new(limits.wire(), w::LoginEndSignedInParts { source: 19 }).unwrap())),
        (c::LoginEnd::NotFinished { exit: None }, w::LoginEnd::NotFinished(w::LoginEndNotFinished::new(limits.wire(), w::LoginEndNotFinishedParts { exit: None }).unwrap())),
        (c::LoginEnd::Cancelled, w::LoginEnd::Cancelled),
        (c::LoginEnd::Refused { refusal: c::AccountRefusal::Numbers }, w::LoginEnd::Refused(w::LoginEndRefused::new(limits.wire(), w::LoginEndRefusedParts { refusal: w::AccountRefusal::Numbers }).unwrap())),
    ] { ends.push(row).unwrap(); }
    for (exit, wire) in exits() {
        ends.push((c::LoginEnd::NotFinished { exit: Some(exit) }, w::LoginEnd::NotFinished(w::LoginEndNotFinished::new(limits.wire(), w::LoginEndNotFinishedParts { exit: Some(wire) }).unwrap()))).unwrap();
    }
    for (end, wire) in ends.into_boxed() {
        let mut parts = jig_client_test_records::accounts_record(None).into_parts();
        parts.lent_end = Some(w::AccountLentEnd::new(limits.wire(), w::AccountLentEndParts { number: 1, end: wire }).unwrap());
        let doc = w::Accounts::new(limits.wire(), parts).unwrap();
        let c::Delivery::Accounts(doc) = delivery(&w::DeliveryValue::Accounts(doc)) else { panic!("accounts") };
        assert_eq!(doc.lent_end, Some(c::AccountLentEnd { number: 1, end }));
    }
}
#[test]
fn terminal_names_are_validated_and_charged_to_the_document() {
    let (_, row) = login(false);
    let mut parts = row.clone().into_parts();
    parts.name = b"../bad".as_slice().into();
    let invalid = w::TerminalLogin::new(&w::CEILINGS, parts).unwrap();
    let mut room = 4;
    assert_eq!(translate::documents::terminal_login(Some(&invalid), &mut room).unwrap_err().reason, skein_codec::Reason::Malformed);
    translate::documents::terminal_login(Some(&row), &mut room).unwrap();
    assert_eq!(room, 0);
    let mut room = 3;
    assert_eq!(translate::documents::terminal_login(Some(&row), &mut room).unwrap_err().reason, skein_codec::Reason::Bound);
}
