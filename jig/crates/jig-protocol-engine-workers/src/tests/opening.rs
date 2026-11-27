use super::fixtures::{self, APPLICATION_VERSION, WORKER_A};
use crate::opening::{Verdict, same};
use jig_protocol_worker_wire as wire;
use jig_protocol_worker_wire::Reason;

fn refusal(verdict: Verdict) -> (Option<u32>, Reason, Box<[u8]>) {
    let Verdict::Refuse { worker, reason, text } = verdict else { panic!("refusal") };
    (worker, reason, text)
}

#[test]
fn an_engine_rejects_a_root_of_an_unspoken_wire_or_missing_declaration() {
    for (name, declaration) in [("unspoken", "Test"), ("test-worker-wire", "Missing")] {
        let mut application = wire::Application::opaque(fixtures::application());
        application
            .roots
            .push(wire::Root { carried: wire::Carried::Items, wire: name, declaration })
            .expect("one root");
        assert_eq!(
            crate::Component::new(&fixtures::LIMITS, fixtures::workers(), wire::spoken(), application).err(),
            Some(crate::Unusable::Application),
        );
    }
}

#[test]
fn a_wrong_secret_and_an_unknown_name_refuse_alike() {
    let mut component = fixtures::component();
    for name in [WORKER_A, b"unknown"] {
        let bytes = fixtures::credential(name, b"wrong", APPLICATION_VERSION);
        assert_eq!(refusal(component.judge(&bytes, false)), (None, Reason::NotAdmitted, Box::from([])));
    }
}
#[test]
fn a_wrong_secret_with_other_wires_is_not_admitted() {
    let mut component = fixtures::component();
    let bytes = fixtures::credential(WORKER_A, b"wrong", 4);
    assert_eq!(refusal(component.judge(&bytes, false)), (None, Reason::NotAdmitted, Box::from([])));
}
#[test]
fn digests_differing_in_the_last_word_are_not_the_same() {
    let mut other = [1; 32];
    other[31] = 2;
    assert!(!same(&[1; 32], &other), "the last byte participates");
}
#[test]
fn an_undecodable_credential_is_unauthorized() {
    let mut component = fixtures::component();
    assert_eq!(refusal(component.judge(b"bad", false)), (None, Reason::Unauthorized, Box::from([])));
}

#[test]
fn the_current_table_admits_numbers_and_in_process_workers() {
    use crate::{Component, Event};
    use skein_lib::{Queue, Time};
    let mut component = Component::new(
        &fixtures::LIMITS,
        Box::new([]),
        wire::spoken(),
        wire::Application::opaque(fixtures::application()),
    )
    .expect("empty admission table");
    let first = fixtures::credential(WORKER_A, fixtures::SECRET_A, APPLICATION_VERSION);
    let refused = (None, Reason::NotAdmitted, Box::from([]));
    assert_eq!(refusal(component.judge(&first, false)), refused, "no configured name");
    component.down(Time::ZERO, Event::Workers(fixtures::workers()), &mut Queue::uncapped());
    let Verdict::Accept { worker, .. } = component.judge(&first, false) else { panic!("table supplied") };
    assert_eq!(worker, 0);
    let mut workers = fixtures::workers();
    workers[0].withdrawn = true;
    workers[1].number = 3;
    workers[1].digest = None;
    component.down(Time::ZERO, Event::Workers(workers), &mut Queue::uncapped());
    assert_eq!(refusal(component.judge(&first, false)), refused, "withdrawn name");
    let local = fixtures::credential(fixtures::WORKER_B, b"anything", APPLICATION_VERSION);
    assert_eq!(refusal(component.judge(&local, false)), refused, "no digest on a socket");
    let Verdict::Accept { worker, .. } = component.judge(&local, true) else { panic!("in process") };
    assert_eq!(worker, 2, "positions are numbers less one, not table offsets");
    component.down(Time::ZERO, Event::Workers(Box::new([])), &mut Queue::uncapped());
    assert_eq!(refusal(component.judge(&local, true)), refused, "replacement is whole");
    component.down(Time::ZERO, Event::Workers(fixtures::workers()), &mut Queue::uncapped());
    let second = fixtures::credential(fixtures::WORKER_B, fixtures::SECRET_B, APPLICATION_VERSION);
    let Verdict::Accept { worker, .. } = component.judge(&second, false) else { panic!("added worker") };
    assert_eq!(worker, 1);
}

#[test]
fn worker_names_are_bounded_independently_of_channels() {
    let limits = crate::Limits { names: 1, ..fixtures::LIMITS };
    let result = crate::Component::new(
        &limits,
        fixtures::workers(),
        jig_protocol_worker_wire::spoken(),
        jig_protocol_worker_wire::Application::opaque(fixtures::application()),
    );
    assert_eq!(result.err(), Some(crate::Unusable::Workers));
}

#[test]
fn a_configured_worker_authenticates_with_its_secret() {
    let mut component = fixtures::component();
    let bytes = fixtures::credential(WORKER_A, fixtures::SECRET_A, APPLICATION_VERSION);
    let Verdict::Accept { worker, fitted, statement, .. } = component.judge(&bytes, false) else { panic!("accepted") };
    assert_eq!(worker, 0);
    assert!(fitted.peer.same());
    assert!(fitted.converted.is_empty());
    assert_eq!(statement.wires.len(), 5);
}

#[test]
fn written_entries_must_name_a_spoken_declaration_and_be_unique_at_a_layout() {
    use skein_lib::List;
    let good = wire::Written { wire: "test-worker-wire", declaration: "Test", layout: [5; 32] };
    for (entries, accepted) in [
        (List::from_boxed(Box::new([wire::Written { wire: "unspoken", ..good }])), false),
        (List::from_boxed(Box::new([wire::Written { declaration: "Missing", ..good }])), false),
        (List::from_boxed(Box::new([good, good])), false),
        (List::from_boxed(Box::new([good, wire::Written { layout: [6; 32], ..good }])), true),
    ] {
        let application = wire::Application { written: entries, ..wire::Application::opaque(fixtures::application()) };
        let result = crate::Component::new(&fixtures::LIMITS, fixtures::workers(), wire::spoken(), application);
        assert_eq!(result.err(), if accepted { None } else { Some(crate::Unusable::Application) });
    }
}
