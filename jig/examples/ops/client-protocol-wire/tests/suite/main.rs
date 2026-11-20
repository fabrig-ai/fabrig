#![cfg(test)]

use jig_ops_client_protocol_wire::{
    codec::{self, infrastructure as infra, observability as obs},
    v1 as w,
};
use skein_lib::List;

mod fixtures;
use fixtures::{PROJECT, REVISION, UNTIL, request};

#[test]
fn an_environment_request_round_trips_through_its_bytes() {
    let value = request();
    assert_eq!(infra::ask_value(&infra::ask(&value).expect("record fits schema")), Ok(value));
}

#[test]
fn an_environment_answer_round_trips_through_its_bytes() {
    let value =
        w::InfrastructureAnswerMovedEnd { environment: fixtures::environment(), revision: REVISION, until: UNTIL };
    let value = w::InfrastructureAnswerRecord { value: w::InfrastructureAnswer::MovedEnd(value) };
    assert_eq!(infra::answer(&infra::answer_record(&value).expect("record fits schema")), Ok(value));
}

#[test]
fn an_infrastructure_subject_round_trips_through_its_bytes() {
    let subject = fixtures::infrastructure_subject();
    let value = w::InfrastructureSubjectRecord { value: subject.kind };
    assert_eq!(infra::subject_value(&infra::subject(&value).expect("record fits schema")), Ok(value));
}

#[test]
fn an_observability_subject_round_trips_through_its_bytes() {
    let value = w::ObservabilitySubjectRecord { value: w::ObservabilityKind::Deployment };
    assert_eq!(obs::subject_value(&obs::subject(&value).expect("record fits schema")), Ok(value));
}

#[test]
fn an_environment_document_round_trips_through_its_bytes() {
    let project = w::InfrastructureValueProject {
        project: PROJECT,
        resources: List::with_capacity(0),
        environments: List::with_capacity(0),
    };
    let value = w::InfrastructureDocumentData {
        subject: w::InfrastructureSubject {
            scope: w::InfrastructureScope::Project(w::InfrastructureScopeProject { value: PROJECT }),
            kind: w::InfrastructureKind::Environments(w::InfrastructureKindEnvironments { pool: None }),
        },
        value: w::InfrastructureValue::Project(project),
    };
    let value = w::InfrastructureDocumentValue { value };
    let value = w::InfrastructureDocumentRecord { value: w::InfrastructureDocument::Value(value) };
    assert_eq!(infra::document(&infra::document_record(&value).expect("record fits schema")), Ok(value));
}

#[test]
fn an_incident_document_round_trips_through_its_bytes() {
    let incident = fixtures::incident();
    assert_eq!(incident.project, PROJECT);
    let number = incident.number;
    let value = w::ObservabilityValueIncident { incident, fact: None };
    let value = w::ObservabilityDocumentData {
        subject: w::ObservabilitySubject {
            scope: w::ObservabilityScope::Project(w::ObservabilityScopeProject { value: PROJECT }),
            kind: w::ObservabilityKind::Incident(w::ObservabilityKindIncident { value: number }),
        },
        value: w::ObservabilityValue::Incident(value),
    };
    let value = w::ObservabilityDocumentValue { value };
    let value = w::ObservabilityDocumentRecord { value: w::ObservabilityDocument::Value(value) };
    assert_eq!(obs::document(&obs::document_record(&value).expect("record fits schema")), Ok(value));
}

#[test]
fn a_record_of_another_version_is_undecodable() {
    let mut bytes = infra::ask(&request()).expect("record fits schema");
    bytes[1] = 2;
    assert_eq!(infra::ask_value(&bytes), Err(codec::Undecodable));
}

#[test]
fn trailing_bytes_are_undecodable() {
    let bytes = infra::ask(&request()).expect("record fits schema");
    let mut longer = skein_lib::Writer::new(bytes.len() + 1);
    longer.put(&bytes).expect("request");
    longer.put(&[0]).expect("trailing byte");
    assert_eq!(infra::ask_value(&longer.finish()), Err(codec::Undecodable));
}

#[test]
fn a_truncated_record_is_undecodable() {
    let bytes = infra::ask(&request()).expect("record fits schema");
    assert_eq!(infra::ask_value(&bytes[..bytes.len() - 1]), Err(codec::Undecodable));
}

#[test]
fn encoding_reports_a_nested_bound_refusal() {
    let mut value = request();
    let w::InfrastructureAsk::MoveEnd(ask) = &mut value.value;
    ask.environment.pool = Box::new([b'p'; 4097]);
    let problem = infra::ask(&value).expect_err("a public field may exceed its wire bound");
    assert_eq!(problem.path, w::paths::INFRASTRUCTURE_ENVIRONMENT_POOL);
    assert_eq!(problem.reason, skein_codec::Reason::Bound);
}
