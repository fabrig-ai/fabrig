//! Project visibility records (jig/docs/design/domain/people/public-reading.md, 2 and 8).
use jig_client_protocol_wire::v21 as w;
use skein_lib::{List, Reader, Writer};

#[test]
fn a_set_visibility_ask_round_trips() {
    for public in [false, true] {
        let value = w::Ask::SetVisibility(
            w::SetVisibility::new(&w::CEILINGS, w::SetVisibilityParts { project: u32::MAX, public })
                .expect("visibility"),
        );
        let mut out = Writer::new(skein_lib::length::usize_of(value.measure()));
        value.encode(&mut out).expect("ask bytes");
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
            w::OutcomeVisibilitySet::new(&w::CEILINGS, w::OutcomeVisibilitySetParts { project: u32::MAX, public })
                .expect("visibility"),
        );
        let mut out = Writer::new(skein_lib::length::usize_of(value.measure()));
        value.encode(&mut out).expect("outcome bytes");
        let bytes = out.finish();
        let mut input = Reader::new(&bytes);
        assert_eq!(w::DocumentOutcome::decode(&w::CEILINGS, &mut input), Ok(value));
        assert!(input.is_empty());
    }
}

#[test]
fn a_me_project_round_trips_with_and_without_its_public_identifier() {
    for public in [Some(u64::MAX), Some(0), None] {
        let value = w::MeProject::new(
            &w::CEILINGS,
            w::MeProjectParts {
                left_out: 0,
                defaults: List::with_capacity(0),
                project: u32::MAX,
                name: w::DocumentText::new(&w::CEILINGS, w::DocumentTextParts { bytes: Box::default(), cut: false })
                    .expect("name"),
                role: w::HeldRole::Owner,
                pool: None,
                templates: Box::default(),
                creating: None,
                models: List::with_capacity(0),
                public,
            },
        )
        .expect("project");
        let mut out = Writer::new(skein_lib::length::usize_of(value.measure()));
        value.encode(&mut out).expect("project bytes");
        let bytes = out.finish();
        let mut input = Reader::new(&bytes);
        assert_eq!(w::MeProject::decode(&w::CEILINGS, &mut input), Ok(value));
        assert!(input.is_empty());
    }
}
