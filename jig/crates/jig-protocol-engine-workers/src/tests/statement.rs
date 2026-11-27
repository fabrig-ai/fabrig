use crate::{Gap, Site, Stated, Statement, Unfit, gaps, judge};
use jig_protocol_worker_wire as wire;
use skein_channel::{Direction, Term};
use skein_codec::{layout::Refused, schema};
use skein_lib::{List, Reader};

#[test]
fn judge_checks_wires_before_output_before_terms() {
    let component = super::fixtures::component();
    let limits = component.codec();
    let credential = super::fixtures::credential(super::fixtures::WORKER_A, super::fixtures::SECRET_A, 4);
    let credential =
        <wire::Credential as wire::Record>::decode(limits, &mut Reader::new(&credential)).expect("credential");
    let mut statement = Statement::credential(credential);
    let room = component.limits.room;
    statement.output = room.checked_sub(1).expect("room above one");
    statement.terms = List::uncapped();
    let roots = List::with_capacity(0);
    assert_eq!(
        judge(component.spoken(), &roots, room, limits, &statement),
        Err(Unfit::Wires(Box::from(b"wire test-worker-wire: the worker's text does not parse at line 1".as_slice())))
    );
    let app = component.spoken().get(4).expect("app wire");
    statement.wires.get_mut(4).expect("stated app wire").layout = app.layout;
    assert_eq!(
        judge(component.spoken(), &roots, room, limits, &statement),
        Err(Unfit::Limits(crate::opening::output_room_words(room, room.checked_sub(1).expect("room above one"))))
    );
    statement.output = room;
    assert_eq!(
        judge(component.spoken(), &roots, room, limits, &statement),
        Err(Unfit::Limits(Box::from(b"384".as_slice())))
    );
    let schema = wire::schema(limits).expect("valid table");
    for rule in &schema.kinds {
        if rule.direction == Direction::FromResponder {
            statement.terms.push(Term { kind: rule.kind, largest: rule.largest }).expect("term slot");
        }
    }
    assert_eq!(judge(component.spoken(), &roots, room, limits, &statement), Ok(List::with_capacity(0)));
}

#[test]
fn gaps_reuses_judges_wire_checks_without_needing_output_or_terms() {
    let component = super::fixtures::component();
    let spoken = component.spoken();
    let limits = component.codec();
    let credential = super::fixtures::credential(super::fixtures::WORKER_A, super::fixtures::SECRET_A, 3);
    let credential =
        <wire::Credential as wire::Record>::decode(limits, &mut Reader::new(&credential)).expect("credential");
    let mut statement = Statement::credential(credential);
    statement.output = 0;
    let roots = List::from_boxed(Box::new([wire::Root {
        carried: wire::Carried::JobSpec,
        wire: "test-worker-wire",
        declaration: "Test",
    }]));

    let app = statement.wires.get_mut(4).expect("stated application wire");
    app.text = Box::from(b"not a schema".as_slice());
    assert_eq!(gaps(spoken, &roots, limits, &statement.wires), Ok(List::with_capacity(0)));

    let app = statement.wires.get_mut(4).expect("stated application wire");
    state_text(app, b"family test-worker-wire 3\nrecord Test { value: u32 }\n");
    assert_eq!(
        gaps(spoken, &roots, limits, &statement.wires).expect("the layout fits").as_slice(),
        &[Gap {
            wire: "test-worker-wire".into(),
            site: Site::Carried(wire::Carried::JobSpec),
            why: Refused::Field,
            declaration: "Test".into(),
            part: "note".into(),
        }]
    );

    let app = statement.wires.get_mut(4).expect("stated application wire");
    state_text(app, b"family test-worker-wire 3\nrecord Test { value: bool }\n");
    let error = judge(spoken, &roots, component.limits.room, limits, &statement).expect_err("retype needs a plan");
    assert_eq!(
        error,
        Unfit::Wires(Box::from(b"wire test-worker-wire Test: no plan to the worker: retyped at Test.value".as_slice()))
    );
    assert_eq!(gaps(spoken, &roots, limits, &statement.wires), Err(error));
}

fn state_text(wire: &mut Stated, text: &[u8]) {
    wire.layout = schema::parse(text).expect("fixture schema").fingerprints().family().digest();
    wire.text = Box::from(text);
}
