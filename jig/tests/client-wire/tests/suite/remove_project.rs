//! Project deletion records (jig/docs/design/domain/people.md, 5.2).
use jig_client_protocol_wire::v19 as w;
use skein_lib::{Reader, Writer};

#[test]
fn a_remove_project_ask_round_trips() {
    for project in [0, 3, u32::MAX] {
        let value = w::Ask::RemoveProject(
            w::RemoveProject::new(&w::CEILINGS, w::RemoveProjectParts { project }).expect("project"),
        );
        let mut out = Writer::new(usize::try_from(value.measure()).expect("ask size"));
        value.encode(&mut out).expect("ask bytes");
        let bytes = out.finish();
        let mut input = Reader::new(&bytes);
        assert_eq!(w::Ask::decode(&w::CEILINGS, &mut input), Ok(value));
        assert!(input.is_empty());
    }
}

#[test]
fn a_project_removed_outcome_round_trips() {
    for project in [0, 3, u32::MAX] {
        let value = w::DocumentOutcome::ProjectRemoved(
            w::OutcomeProjectRemoved::new(&w::CEILINGS, w::OutcomeProjectRemovedParts { project }).expect("project"),
        );
        let mut out = Writer::new(usize::try_from(value.measure()).expect("outcome size"));
        value.encode(&mut out).expect("outcome bytes");
        let bytes = out.finish();
        let mut input = Reader::new(&bytes);
        assert_eq!(w::DocumentOutcome::decode(&w::CEILINGS, &mut input), Ok(value));
        assert!(input.is_empty());
    }
}
