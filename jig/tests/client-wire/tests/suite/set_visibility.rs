//! Project visibility records (jig/docs/design/domain/people/public-reading.md, 2 and 8).
use jig_client_protocol_wire::Record as _;
use jig_client_protocol_wire as w;
use skein_lib::{List, Reader, Writer};

#[test]
fn a_set_visibility_ask_round_trips() {
    for public in [false, true] {
        let value = w::Ask::SetVisibility(
            w::SetVisibility { project: u32::MAX, public },
        );
        let mut out = Writer::new(skein_lib::length::usize_of(value.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")));
        value.write(&jig_client_protocol_wire::CEILINGS, &mut out).expect("ask bytes");
        let bytes = out.finish();
        let mut input = Reader::new(&bytes);
        assert_eq!(w::Ask::decode(&w::CEILINGS, &mut input), Ok(value));
        assert!(input.is_empty());
    }
}

#[test]
fn a_visibility_set_outcome_round_trips() {
    for public in [false, true] {
        let value = w::DocumentOutcome::VisibilitySet(
            w::OutcomeVisibilitySet { project: u32::MAX, public },
        );
        let mut out = Writer::new(skein_lib::length::usize_of(value.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")));
        value.write(&jig_client_protocol_wire::CEILINGS, &mut out).expect("outcome bytes");
        let bytes = out.finish();
        let mut input = Reader::new(&bytes);
        assert_eq!(w::DocumentOutcome::decode(&w::CEILINGS, &mut input), Ok(value));
        assert!(input.is_empty());
    }
}

#[test]
fn a_me_project_round_trips_with_and_without_its_public_identifier() {
    for public in [Some(u64::MAX), Some(0), None] {
        let value = w::MeProject {
                left_out: 0,
                defaults: List::with_capacity(0),
                project: u32::MAX,
                name: w::DocumentText { bytes: Box::default(), cut: false },
                role: w::HeldRole::Owner,
                pool: None,
                templates: Box::default(),
                creating: None,
                models: List::with_capacity(0),
                public,
            };
        let mut out = Writer::new(skein_lib::length::usize_of(value.measure(&jig_client_protocol_wire::CEILINGS).expect("bounded fixture")));
        value.write(&jig_client_protocol_wire::CEILINGS, &mut out).expect("project bytes");
        let bytes = out.finish();
        let mut input = Reader::new(&bytes);
        assert_eq!(w::MeProject::decode(&w::CEILINGS, &mut input), Ok(value));
        assert!(input.is_empty());
    }
}
