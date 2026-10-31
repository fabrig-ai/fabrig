use crate::{
    codec::{self, infrastructure as infra, observability as obs},
    v1 as w,
};
use skein_lib::List;

mod fixtures;
use fixtures::{PROJECT, REVISION, UNTIL, request};

#[test]
fn an_environment_request_round_trips_through_its_bytes() {
    let value = request();
    assert_eq!(infra::ask_value(&infra::ask(&value)), Ok(value));
}

#[test]
fn an_environment_answer_round_trips_through_its_bytes() {
    let value = w::InfrastructureAnswerNativeMovedEnd::new(
        &w::CEILINGS,
        w::InfrastructureAnswerNativeMovedEndParts {
            environment: fixtures::environment(),
            revision: REVISION,
            until: UNTIL,
        },
    )
    .expect("answer");
    let value = w::InfrastructureAnswerRecord::new(
        &w::CEILINGS,
        w::InfrastructureAnswerRecordParts { value: w::InfrastructureAnswer::NativeMovedEnd(value) },
    )
    .expect("answer record");
    assert_eq!(infra::answer(&infra::answer_record(&value)), Ok(value));
}

#[test]
fn an_infrastructure_subject_round_trips_through_its_bytes() {
    let subject = fixtures::infrastructure_subject();
    let value = w::InfrastructureSubjectRecord::new(
        &w::CEILINGS,
        w::InfrastructureSubjectRecordParts { value: subject.into_parts().kind },
    )
    .expect("subject record");
    assert_eq!(infra::subject_value(&infra::subject(&value)), Ok(value));
}

#[test]
fn an_observability_subject_round_trips_through_its_bytes() {
    let value = w::ObservabilitySubjectRecord::new(
        &w::CEILINGS,
        w::ObservabilitySubjectRecordParts { value: w::ObservabilityKind::Deployment },
    )
    .expect("subject record");
    assert_eq!(obs::subject_value(&obs::subject(&value)), Ok(value));
}

#[test]
fn an_environment_document_round_trips_through_its_bytes() {
    let project = w::InfrastructureNativeValueProject::new(
        &w::CEILINGS,
        w::InfrastructureNativeValueProjectParts {
            project: PROJECT,
            resources: List::with_capacity(0),
            environments: List::with_capacity(0),
        },
    )
    .expect("project");
    let value = w::InfrastructureNativeDocument::new(
        &w::CEILINGS,
        w::InfrastructureNativeDocumentParts {
            subject: w::InfrastructureSubject::new(
                &w::CEILINGS,
                w::InfrastructureSubjectParts {
                    scope: w::InfrastructureScope::Project(
                        w::InfrastructureScopeProject::new(
                            &w::CEILINGS,
                            w::InfrastructureScopeProjectParts { value: PROJECT },
                        )
                        .expect("project scope"),
                    ),
                    kind: w::InfrastructureKind::NativeEnvironments(
                        w::InfrastructureKindNativeEnvironments::new(
                            &w::CEILINGS,
                            w::InfrastructureKindNativeEnvironmentsParts { pool: None },
                        )
                        .expect("environments"),
                    ),
                },
            )
            .expect("subject"),
            value: w::InfrastructureNativeValue::Project(project),
        },
    )
    .expect("environment list");
    let value = w::InfrastructureDocumentNative::new(&w::CEILINGS, w::InfrastructureDocumentNativeParts { value })
        .expect("native document");
    let value = w::InfrastructureDocumentRecord::new(
        &w::CEILINGS,
        w::InfrastructureDocumentRecordParts { value: w::InfrastructureDocument::Native(value) },
    )
    .expect("document record");
    assert_eq!(infra::document(&infra::document_record(&value)), Ok(value));
}

#[test]
fn an_incident_document_round_trips_through_its_bytes() {
    let incident = fixtures::incident();
    assert_eq!(incident.project(), PROJECT);
    let number = incident.number();
    let value = w::ObservabilityNativeValueIncident::new(
        &w::CEILINGS,
        w::ObservabilityNativeValueIncidentParts { incident, fact: None },
    )
    .expect("incident value");
    let value = w::ObservabilityNativeDocument::new(
        &w::CEILINGS,
        w::ObservabilityNativeDocumentParts {
            subject: w::ObservabilitySubject::new(
                &w::CEILINGS,
                w::ObservabilitySubjectParts {
                    scope: w::ObservabilityScope::Project(
                        w::ObservabilityScopeProject::new(
                            &w::CEILINGS,
                            w::ObservabilityScopeProjectParts { value: PROJECT },
                        )
                        .expect("project scope"),
                    ),
                    kind: w::ObservabilityKind::Incident(
                        w::ObservabilityKindIncident::new(
                            &w::CEILINGS,
                            w::ObservabilityKindIncidentParts { value: number },
                        )
                        .expect("incident kind"),
                    ),
                },
            )
            .expect("subject"),
            value: w::ObservabilityNativeValue::Incident(value),
        },
    )
    .expect("native incident");
    let value = w::ObservabilityDocumentNative::new(&w::CEILINGS, w::ObservabilityDocumentNativeParts { value })
        .expect("document");
    let value = w::ObservabilityDocumentRecord::new(
        &w::CEILINGS,
        w::ObservabilityDocumentRecordParts { value: w::ObservabilityDocument::Native(value) },
    )
    .expect("document record");
    assert_eq!(obs::document(&obs::document_record(&value)), Ok(value));
}

#[test]
fn a_record_of_another_version_is_undecodable() {
    let mut bytes = infra::ask(&request());
    bytes[1] = 2;
    assert_eq!(infra::ask_value(&bytes), Err(codec::Undecodable));
}

#[test]
fn trailing_bytes_are_undecodable() {
    let bytes = infra::ask(&request());
    let mut longer = skein_lib::Writer::new(bytes.len() + 1);
    longer.put(&bytes).expect("request");
    longer.put(&[0]).expect("trailing byte");
    assert_eq!(infra::ask_value(&longer.finish()), Err(codec::Undecodable));
}

#[test]
fn a_truncated_record_is_undecodable() {
    let bytes = infra::ask(&request());
    assert_eq!(infra::ask_value(&bytes[..bytes.len() - 1]), Err(codec::Undecodable));
}
