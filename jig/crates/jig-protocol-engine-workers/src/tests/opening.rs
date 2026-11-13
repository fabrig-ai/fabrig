use super::fixtures::{self, APPLICATION_VERSION, WORKER_A};
use crate::opening::{Verdict, same};
use jig_protocol_worker_wire as wire;
use jig_protocol_worker_wire::Reason;
use skein_io::digest::Digest;
use skein_lib::List;

fn stated(name: &[u8], lowest: u16, layouts: &[[u8; 32]]) -> wire::Wire {
    let mut list = List::with_capacity(u32::try_from(layouts.len()).expect("layout count"));
    for layout in layouts {
        list.push(*layout).expect("uncapped");
    }
    wire::Wire::new(&wire::CEILINGS, wire::WireParts { name: Box::from(name), lowest, layouts: list }).expect("wire")
}

#[test]
fn wire_refusals_name_the_first_failure_and_its_values() {
    let component = fixtures::component();
    let mut layout = fixtures::statement(APPLICATION_VERSION);
    *layout.get_mut(4).expect("application wire") = stated(b"test-worker-wire", 3, &[[4; 32]]);
    let mut missing = fixtures::statement(APPLICATION_VERSION);
    missing.truncate(4);
    let mut twice = fixtures::statement(APPLICATION_VERSION);
    twice.push(twice.get(4).expect("application wire").clone()).expect("uncapped");
    let mut overflow = fixtures::statement(APPLICATION_VERSION);
    *overflow.get_mut(4).expect("application wire") = stated(b"test-worker-wire", u16::MAX, &[[3; 32], [3; 32]]);
    let mut empty = fixtures::statement(APPLICATION_VERSION);
    *empty.get_mut(4).expect("application wire") = stated(b"test-worker-wire", 3, &[]);
    for (statement, text) in [
        (
            layout,
            b"wire test-worker-wire version 3: the engine's layout 0303030303030303, the worker's 0404040404040404"
                .as_slice(),
        ),
        (fixtures::statement(4), b"wire test-worker-wire: the engine speaks 3..3, the worker 4..4"),
        (missing, b"wire test-worker-wire: the worker states none"),
        (twice, b"wire test-worker-wire: stated twice"),
        (overflow, b"wire test-worker-wire: no such range"),
        (empty, b"wire test-worker-wire: no such range"),
    ] {
        let credential = fixtures::credential_with(WORKER_A, fixtures::SECRET_A, statement);
        assert_eq!(
            component.judge(&credential),
            Verdict::Refuse { worker: Some(0), reason: Reason::Wires, text: Box::from(text) }
        );
    }
}

#[test]
fn agreement_checks_common_layouts_and_selects_the_highest_version() {
    let mut component = fixtures::component();
    let layouts = List::from_boxed(Box::new([[3; 32], [4; 32]]));
    component.wires.get_mut(4).expect("application wire").layouts = layouts;
    let mut statement = fixtures::statement(APPLICATION_VERSION);
    *statement.get_mut(4).expect("application wire") = stated(b"test-worker-wire", 3, &[[3; 32], [4; 32]]);
    let credential = fixtures::credential_with(WORKER_A, fixtures::SECRET_A, statement.clone());
    let Verdict::Accept { wires, .. } = component.judge(&credential) else { panic!("two layouts agree") };
    assert_eq!(wires.get(4), Some(&wire::Agreed { name: "test-worker-wire", version: 4 }));
    *statement.get_mut(4).expect("application wire") = stated(b"test-worker-wire", 3, &[[9; 32], [4; 32]]);
    let credential = fixtures::credential_with(WORKER_A, fixtures::SECRET_A, statement);
    assert_eq!(
        component.judge(&credential),
        Verdict::Refuse {
            worker: Some(0),
            reason: Reason::Wires,
            text: Box::from(
                b"wire test-worker-wire version 3: the engine's layout 0303030303030303, the worker's 0909090909090909"
                    .as_slice()
            )
        }
    );
}

#[test]
fn agreement_keeps_statement_order_and_passes_over_unknown_wires() {
    let component = fixtures::component();
    let mut statement = fixtures::statement(APPLICATION_VERSION);
    let first = statement.swap_remove(0).expect("first wire");
    statement.push(first).expect("uncapped");
    statement.push(stated(b"unknown-wire", 1, &[[7; 32]])).expect("uncapped");
    let credential = fixtures::credential_with(WORKER_A, fixtures::SECRET_A, statement.clone());
    let Verdict::Accept { worker, wires, .. } = component.judge(&credential) else { panic!("accepted") };
    assert_eq!(worker, 0);
    assert_eq!(wires.len(), 5);
    for (agreed, stated) in wires.iter().zip(statement.iter()) {
        assert_eq!(agreed.name.as_bytes(), stated.name());
        assert_eq!(agreed.version, stated.lowest());
    }
}
#[test]
fn a_wrong_secret_and_an_unknown_name_refuse_alike() {
    let component = fixtures::component();
    for name in [WORKER_A, b"unknown"] {
        let bytes = fixtures::credential(name, b"wrong", APPLICATION_VERSION);
        assert_eq!(
            component.judge(&bytes),
            Verdict::Refuse { worker: None, reason: Reason::Unauthorized, text: Box::new([]) }
        );
    }
}
#[test]
fn a_wrong_secret_with_other_wires_is_unauthorized() {
    let component = fixtures::component();
    let bytes = fixtures::credential(WORKER_A, b"wrong", 4);
    assert_eq!(
        component.judge(&bytes),
        Verdict::Refuse { worker: None, reason: Reason::Unauthorized, text: Box::new([]) }
    );
}
#[test]
fn digests_differing_in_the_last_word_are_not_the_same() {
    assert!(!same(&Digest([1, 2, 3, 4]), &Digest([1, 2, 3, 5])), "the last word participates");
}
#[test]
fn an_undecodable_credential_is_unauthorized() {
    let component = fixtures::component();
    assert_eq!(
        component.judge(b"bad"),
        Verdict::Refuse { worker: None, reason: Reason::Unauthorized, text: Box::new([]) }
    );
}

#[test]
fn worker_names_are_bounded_independently_of_channels() {
    let limits = crate::Limits { names: 1, ..fixtures::LIMITS };
    let result = crate::Component::new(&limits, fixtures::workers(), fixtures::application());
    assert_eq!(result.err(), Some(crate::Unusable::Workers));
}

#[test]
fn a_configured_worker_authenticates_with_its_secret() {
    let component = fixtures::component();
    let bytes = fixtures::credential(WORKER_A, fixtures::SECRET_A, APPLICATION_VERSION);
    let Verdict::Accept { worker, wires, .. } = component.judge(&bytes) else { panic!("accepted") };
    assert_eq!(worker, 0);
    assert_eq!(wires.len(), 5);
    for (agreed, stated) in wires.iter().zip(fixtures::statement(APPLICATION_VERSION).iter()) {
        assert_eq!(agreed.name.as_bytes(), stated.name());
        assert_eq!(agreed.version, stated.lowest());
    }
}
