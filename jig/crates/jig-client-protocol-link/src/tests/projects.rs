use super::*;
use crate::translate::{answers, asks, store};
use skein_lib::Writer;

#[test]
fn project_requests_round_trip_and_restore_their_whole_fields() {
    let limits = fixtures::limits();
    for ask in [
        c::Ask::Rename { project: u32::MAX, name: c::Text::new(b"Tally".as_slice().into(), 64).unwrap() },
        c::Ask::MakeProject { name: c::Text::new(b"Tally".as_slice().into(), 64).unwrap(), policy: u32::MAX },
    ] {
        let record = asks::ask(&ask, limits.wire()).unwrap();
        let mut bytes = Writer::new(usize::try_from(record.measure()).unwrap());
        record.encode(&mut bytes).unwrap();
        let decoded = w::Ask::decode(limits.wire(), &mut Reader::new(&bytes.finish())).unwrap();
        assert_eq!(store::restore_ask(decoded, &limits), Ok(ask));
    }
}

#[test]
fn project_outcomes_round_trip_the_supplied_project_number() {
    let limits = fixtures::limits();
    for (wire, expected) in [
        (
            w::DocumentOutcome::Renamed(
                w::OutcomeRenamed::new(limits.wire(), w::OutcomeRenamedParts { project: 17 }).unwrap(),
            ),
            c::DocumentOutcome::Renamed { project: 17 },
        ),
        (
            w::DocumentOutcome::ProjectMade(
                w::OutcomeProjectMade::new(limits.wire(), w::OutcomeProjectMadeParts { project: 19 }).unwrap(),
            ),
            c::DocumentOutcome::ProjectMade { project: 19 },
        ),
    ] {
        let mut bytes = Writer::new(usize::try_from(wire.measure()).unwrap());
        wire.encode(&mut bytes).unwrap();
        let decoded = w::DocumentOutcome::decode(limits.wire(), &mut Reader::new(&bytes.finish())).unwrap();
        assert_eq!(answers::document_outcome(&decoded, &limits), Ok(expected));
    }
}

#[test]
fn project_refusals_keep_each_name_rule_and_the_deployment_limit() {
    let limits = fixtures::limits();
    assert_eq!(translate::documents::act_kind_fields(&w::ActKind::Rename), c::ActKind::Rename);
    assert_eq!(answers::refusal(&w::Refusal::NameTaken, &limits), Ok(c::Refusal::NameTaken));
    assert_eq!(answers::admission_limit(&w::AdmissionLimit::Projects), c::AdmissionLimit::Projects);
    for (wire, expected) in [
        (w::NameRule::Empty, c::NameRule::Empty),
        (w::NameRule::Long, c::NameRule::Long),
        (w::NameRule::Control, c::NameRule::Control),
        (w::NameRule::Space, c::NameRule::Space),
        (w::NameRule::Digits, c::NameRule::Digits),
    ] {
        let refusal = w::Refusal::Malformed(
            w::RefusalMalformed::new(
                limits.wire(),
                w::RefusalMalformedParts {
                    at: 0,
                    why: w::Malformed::Name(
                        w::MalformedName::new(limits.wire(), w::MalformedNameParts { rule: wire }).unwrap(),
                    ),
                },
            )
            .unwrap(),
        );
        assert_eq!(
            answers::refusal(&refusal, &limits),
            Ok(c::Refusal::Malformed { at: 0, why: c::Malformed::Name(expected) })
        );
    }
}

#[test]
fn project_requests_cannot_be_compound_operations() {
    let limits = fixtures::limits();
    for ask in [
        c::Ask::SetVisibility { project: u32::MAX, public: false },
        c::Ask::SetVisibility { project: u32::MAX, public: true },
        c::Ask::Rename { project: 1, name: fixtures::store_text() },
        c::Ask::MakeProject { name: fixtures::store_text(), policy: 1 },
    ] {
        let ask = asks::ask(&ask, limits.wire()).unwrap();
        let mut bytes = Writer::new(usize::try_from(ask.measure()).unwrap());
        ask.encode(&mut bytes).unwrap();
        let bytes = bytes.finish();
        let mut pieces = Writer::new(bytes.len().checked_add(4).unwrap());
        pieces.put(&u32::try_from(bytes.len()).unwrap().to_be_bytes()).unwrap();
        pieces.put(&bytes).unwrap();
        let compound =
            w::Compound::new(limits.wire(), w::CompoundParts { project: 1, operations: pieces.finish() }).unwrap();
        store::restore_ask(w::Ask::Compound(compound), &limits)
            .expect_err("a project request is not a compound operation");
    }
}

#[test]
fn remove_project_round_trips_through_the_link() {
    let limits = fixtures::limits();
    for grace in [None, Some(skein_lib::Duration::from_millis(5000))] {
        let ask = c::Ask::RemoveProject { project: u32::MAX, grace };
        let record = asks::ask(&ask, limits.wire()).unwrap();
        let w::Ask::RemoveProject(row) = &record else { panic!("remove_project wire record") };
        assert_eq!(row.project(), u32::MAX);
        assert_eq!(*row.grace(), grace);
        let mut bytes = Writer::new(usize::try_from(record.measure()).unwrap());
        record.encode(&mut bytes).unwrap();
        let decoded = w::Ask::decode(limits.wire(), &mut Reader::new(&bytes.finish())).unwrap();
        assert_eq!(store::restore_ask(decoded, &limits), Ok(ask));
    }
    let outcome = w::DocumentOutcome::ProjectRemoved(
        w::OutcomeProjectRemoved::new(limits.wire(), w::OutcomeProjectRemovedParts { project: u32::MAX }).unwrap(),
    );
    let mut bytes = Writer::new(usize::try_from(outcome.measure()).unwrap());
    outcome.encode(&mut bytes).unwrap();
    let decoded = w::DocumentOutcome::decode(limits.wire(), &mut Reader::new(&bytes.finish())).unwrap();
    assert_eq!(
        answers::document_outcome(&decoded, &limits),
        Ok(c::DocumentOutcome::ProjectRemoved { project: u32::MAX })
    );
    assert_eq!(translate::documents::act_kind_fields(&w::ActKind::RemoveProject), c::ActKind::RemoveProject);
}

#[test]
fn ceiling_refusals_keep_holders_dimensions_and_scalar_values() {
    for (holder, wire_holder) in [
        (c::CeilingHolder::Rules, w::CeilingHolder::Rules),
        (c::CeilingHolder::Policy, w::CeilingHolder::Policy),
        (
            c::CeilingHolder::Role(0),
            w::CeilingHolder::Role(w::CeilingRole::new(&w::CEILINGS, w::CeilingRoleParts { role: 0 }).unwrap()),
        ),
    ] {
        for (dimension, wire_dimension, value, asked) in [
            (c::CeilingDimension::PeriodSpend, w::CeilingDimension::PeriodSpend, Some(1000), Some(1001)),
            (c::CeilingDimension::Spend, w::CeilingDimension::Spend, Some(500), Some(501)),
            (c::CeilingDimension::Tools, w::CeilingDimension::Tools, None, None),
            (c::CeilingDimension::Tasks, w::CeilingDimension::Tasks, Some(3), Some(4)),
            (c::CeilingDimension::Depth, w::CeilingDimension::Depth, Some(2), Some(3)),
            (c::CeilingDimension::Executors, w::CeilingDimension::Executors, None, None),
            (c::CeilingDimension::Grants, w::CeilingDimension::Grants, None, None),
            (c::CeilingDimension::Deadline, w::CeilingDimension::Deadline, Some(10), None),
            (c::CeilingDimension::Notes, w::CeilingDimension::Notes, None, None),
        ] {
            let wire = w::Lacking::Ceiling(
                w::LackingCeiling::new(
                    &w::CEILINGS,
                    w::LackingCeilingParts { holder: wire_holder.clone(), dimension: wire_dimension, value, asked },
                )
                .unwrap(),
            );
            let mut bytes = Writer::new(usize::try_from(wire.measure()).unwrap());
            wire.encode(&mut bytes).unwrap();
            let bytes = bytes.finish();
            let decoded = w::Lacking::decode(&w::CEILINGS, &mut Reader::new(&bytes)).unwrap();
            assert_eq!(answers::lacking(&decoded), c::Lacking::Ceiling { holder, dimension, value, asked });
        }
    }
}

#[test]
fn set_visibility_crosses_the_link_both_ways() {
    let limits = fixtures::limits();
    for public in [false, true] {
        let ask = c::Ask::SetVisibility { project: u32::MAX, public };
        let record = asks::ask(&ask, limits.wire()).expect("wire ask");
        let w::Ask::SetVisibility(row) = &record else { panic!("visibility ask") };
        assert_eq!((row.project(), row.public()), (u32::MAX, public));
        let mut bytes = Writer::new(skein_lib::length::usize_of(record.measure()));
        record.encode(&mut bytes).expect("ask bytes");
        let decoded = w::Ask::decode(limits.wire(), &mut Reader::new(&bytes.finish())).expect("decoded ask");
        assert_eq!(store::restore_ask(decoded, &limits), Ok(ask));
        let outcome = w::DocumentOutcome::VisibilitySet(
            w::OutcomeVisibilitySet::new(limits.wire(), w::OutcomeVisibilitySetParts { project: u32::MAX, public })
                .expect("outcome"),
        );
        assert_eq!(
            answers::document_outcome(&outcome, &limits),
            Ok(c::DocumentOutcome::VisibilitySet { project: u32::MAX, public })
        );
    }
}

#[test]
fn a_public_identifier_in_me_reaches_the_contract() {
    let limits = fixtures::limits();
    for public in [Some(u64::MAX), Some(0), None] {
        let project = w::MeProject::new(
            limits.wire(),
            w::MeProjectParts {
                project: u32::MAX,
                name: w::DocumentText::new(limits.wire(), w::DocumentTextParts { bytes: Box::default(), cut: false })
                    .expect("name"),
                role: w::HeldRole::Owner,
                pool: None,
                templates: Box::default(),
                creating: None,
                models: skein_lib::List::with_capacity(0),
                public,
            },
        )
        .expect("project");
        let contract = translate::documents::project_role_fields(&project, &limits).expect("contract project");
        assert_eq!(contract.project, u32::MAX);
        assert_eq!(contract.public, public);
    }
}
