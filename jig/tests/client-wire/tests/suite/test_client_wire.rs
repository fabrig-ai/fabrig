use super::fixtures;
use jig_client_test_domain::family;

#[test]
fn a_connector_document_round_trips_with_its_supplied_subject() {
    use jig_client_test_domain::connector_values::{ApplicationObject, ClientDocument, ConnectorDocument};
    let value = ConnectorDocument {
        subject: family::Subject {
            scope: family::Scope::Project(fixtures::PROJECT),
            kind: family::Kind::Item(fixtures::ITEM),
        },
        value: Box::new(ClientDocument::Object {
            object: fixtures::ITEM,
            now: ApplicationObject { revision: fixtures::REVISION.value, value: 42 },
        }),
    };
    fixtures::assert_document(family::Document::Connector(value));
}

#[test]
fn a_connector_answer_round_trips() {
    use jig_client_test_domain::connector_values::{ApplicationObject, ApplicationOutcome, ConnectorAnswer};
    fixtures::assert_answer(family::Answer::Connector(ConnectorAnswer {
        value: ApplicationOutcome::Moved {
            object: fixtures::ITEM,
            now: ApplicationObject { revision: fixtures::REVISION.value, value: 42 },
        },
    }));
}
#[test]
fn an_entrance_refusal_has_no_application_wire_record() {
    use jig_client_domain::contract::{EntranceRefusal, Refusal};
    let refusal = EntranceRefusal::new(Refusal::Unknown).expect("fixed entrance refusal");
    assert_eq!(
        family::wire::encode_answer(&family::Answer::Refused(refusal)),
        Err(family::wire::Invalid::EntranceRefusal)
    );
}

#[test]
fn a_connector_change_round_trips_with_an_explicit_zero() {
    fixtures::assert_change(0);
}
#[test]
fn every_project_subject_round_trips_through_reads_and_opens() {
    for kind in fixtures::SUBJECT_KINDS {
        fixtures::assert_subject(family::Subject { scope: family::Scope::Project(fixtures::PROJECT), kind });
    }
}
#[test]
fn every_deployment_subject_round_trips_through_reads_and_opens() {
    for kind in fixtures::SUBJECT_KINDS {
        fixtures::assert_subject(family::Subject { scope: family::Scope::Deployment, kind });
    }
}
