//! Project deletion records (jig/docs/design/domain/people/acts.md, 2).
use jig_client_protocol_wire::v22 as w;
use skein_lib::{Reader, Writer};

#[test]
fn a_remove_project_ask_round_trips() {
    for project in [0, 3, u32::MAX] {
        for grace in [None, Some(skein_lib::Duration::from_millis(5000))] {
            let value = w::Ask::RemoveProject(
                w::RemoveProject::new(&w::CEILINGS, w::RemoveProjectParts { project, grace }).expect("project"),
            );
            let mut out = Writer::new(skein_lib::length::usize_of(value.measure()));
            value.encode(&mut out).expect("ask bytes");
            let bytes = out.finish();
            let mut input = Reader::new(&bytes);
            assert_eq!(w::Ask::decode(&w::CEILINGS, &mut input), Ok(value));
            assert!(input.is_empty());
        }
    }
}

#[test]
fn a_project_removed_outcome_round_trips() {
    for project in [0, 3, u32::MAX] {
        let value = w::DocumentOutcome::ProjectRemoved(
            w::OutcomeProjectRemoved::new(&w::CEILINGS, w::OutcomeProjectRemovedParts { project }).expect("project"),
        );
        let mut out = Writer::new(skein_lib::length::usize_of(value.measure()));
        value.encode(&mut out).expect("outcome bytes");
        let bytes = out.finish();
        let mut input = Reader::new(&bytes);
        assert_eq!(w::DocumentOutcome::decode(&w::CEILINGS, &mut input), Ok(value));
        assert!(input.is_empty());
    }
}
