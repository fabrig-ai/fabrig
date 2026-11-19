//! Family envelopes and subjects (jig/docs/design/examples.md, 8).
use crate::observability::documents as d;
use crate::wire::{Box, Invalid, w};
pub(crate) fn scope_encode(value: d::Scope) -> w::ObservabilityScope {
    match value {
        d::Scope::Project(value) => {
            let value = w::ObservabilityScopeProject { value };
            w::ObservabilityScope::Project(value)
        }
        d::Scope::Deployment => w::ObservabilityScope::Deployment,
    }
}

pub(crate) fn scope_decode(value: w::ObservabilityScope) -> d::Scope {
    match value {
        w::ObservabilityScope::Project(value) => d::Scope::Project(value.value),
        w::ObservabilityScope::Deployment => d::Scope::Deployment,
    }
}

pub(crate) fn kind_encode(value: &d::Kind) -> w::ObservabilityKind {
    match value {
        d::Kind::Incidents { open } => {
            let value = w::ObservabilityKindIncidents { open: (*open) };
            w::ObservabilityKind::Incidents(value)
        }
        d::Kind::Incident(value) => {
            let value = w::ObservabilityKindIncident { value: (*value) };
            w::ObservabilityKind::Incident(value)
        }
        d::Kind::Panel(value) => {
            let value = w::ObservabilityKindPanel { value: (*value) };
            w::ObservabilityKind::Panel(value)
        }
        d::Kind::Result(value) => {
            let value = w::ObservabilityKindResult { value: (*value) };
            w::ObservabilityKind::Result(value)
        }
        d::Kind::Effect(value) => {
            let value = w::ObservabilityKindEffect { value: crate::wire::proposal_encode(value) };
            w::ObservabilityKind::Effect(value)
        }
        d::Kind::Outbox(value) => {
            let value = w::ObservabilityKindOutbox { value: (*value) };
            w::ObservabilityKind::Outbox(value)
        }
        d::Kind::Hold(value) => {
            let value = w::ObservabilityKindHold { value: (*value) };
            w::ObservabilityKind::Hold(value)
        }
        d::Kind::Project(value) => {
            let value = w::ObservabilityKindProject { value: (*value) };
            w::ObservabilityKind::Project(value)
        }
        d::Kind::Deployment => w::ObservabilityKind::Deployment,
    }
}

pub(crate) fn kind_decode(value: w::ObservabilityKind) -> d::Kind {
    match value {
        w::ObservabilityKind::Incidents(value) => d::Kind::Incidents { open: value.open },
        w::ObservabilityKind::Incident(value) => d::Kind::Incident(value.value),
        w::ObservabilityKind::Panel(value) => d::Kind::Panel(value.value),
        w::ObservabilityKind::Result(value) => d::Kind::Result(value.value),
        w::ObservabilityKind::Effect(value) => d::Kind::Effect(crate::wire::proposal_decode(value.value)),
        w::ObservabilityKind::Outbox(value) => d::Kind::Outbox(value.value),
        w::ObservabilityKind::Hold(value) => d::Kind::Hold(value.value),
        w::ObservabilityKind::Project(value) => d::Kind::Project(value.value),
        w::ObservabilityKind::Deployment => d::Kind::Deployment,
    }
}

pub(crate) fn subject_encode(value: &d::Subject) -> w::ObservabilitySubject {
    w::ObservabilitySubject { scope: scope_encode(value.scope), kind: kind_encode(&(value.kind)) }
}

pub(crate) fn subject_decode(value: w::ObservabilitySubject) -> d::Subject {
    d::Subject { scope: scope_decode(value.scope), kind: kind_decode(value.kind) }
}

pub(crate) fn service_encode(value: &d::Service) -> w::ObservabilityService {
    w::ObservabilityService { environment: (value.environment).clone(), name: (value.name).clone() }
}

pub(crate) fn service_decode(value: w::ObservabilityService) -> d::Service {
    d::Service { environment: value.environment, name: value.name }
}

pub(crate) fn state_encode(value: d::State) -> w::ObservabilityState {
    match value {
        d::State::Alerting => w::ObservabilityState::Alerting,
        d::State::Resolved => w::ObservabilityState::Resolved,
    }
}

pub(crate) fn state_decode(value: w::ObservabilityState) -> d::State {
    match value {
        w::ObservabilityState::Alerting => d::State::Alerting,
        w::ObservabilityState::Resolved => d::State::Resolved,
    }
}

pub(crate) fn document_encode(value: &d::Document) -> Result<w::ObservabilityDocument, Invalid> {
    Ok(match value {
        d::Document::Value(value) => {
            let parts =
                w::ObservabilityDocumentValue { value: crate::wire::observability_data::document_encode(value)? };
            let value = parts;
            w::ObservabilityDocument::Value(value)
        }
    })
}

pub(crate) fn document_decode(value: w::ObservabilityDocument) -> d::Document {
    match value {
        w::ObservabilityDocument::Value(value) => {
            d::Document::Value(Box::new(crate::wire::observability_data::document_decode(value.value)))
        }
    }
}
