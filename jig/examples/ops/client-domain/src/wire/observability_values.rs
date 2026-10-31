//! Family envelopes and subjects (jig/docs/design/examples.md, 8).
use crate::observability::documents as d;
use crate::wire::{Box, Invalid, w};
pub(crate) fn scope_encode(value: d::Scope) -> Result<w::ObservabilityScope, Invalid> {
    Ok(match value {
        d::Scope::Project(value) => {
            let parts = w::ObservabilityScopeProjectParts { value };
            let Ok(value) = w::ObservabilityScopeProject::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::ObservabilityScope::Project(value)
        }
        d::Scope::Deployment => w::ObservabilityScope::Deployment,
    })
}

pub(crate) fn scope_decode(value: w::ObservabilityScope) -> d::Scope {
    match value {
        w::ObservabilityScope::Project(value) => {
            let value = value.into_parts();
            d::Scope::Project(value.value)
        }
        w::ObservabilityScope::Deployment => d::Scope::Deployment,
    }
}

pub(crate) fn kind_encode(value: &d::Kind) -> Result<w::ObservabilityKind, Invalid> {
    Ok(match value {
        d::Kind::Incidents { open } => {
            let parts = w::ObservabilityKindIncidentsParts { open: (*open) };
            let Ok(value) = w::ObservabilityKindIncidents::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::ObservabilityKind::Incidents(value)
        }
        d::Kind::Incident(value) => {
            let parts = w::ObservabilityKindIncidentParts { value: (*value) };
            let Ok(value) = w::ObservabilityKindIncident::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::ObservabilityKind::Incident(value)
        }
        d::Kind::Panel(value) => {
            let parts = w::ObservabilityKindPanelParts { value: (*value) };
            let Ok(value) = w::ObservabilityKindPanel::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::ObservabilityKind::Panel(value)
        }
        d::Kind::Result(value) => {
            let parts = w::ObservabilityKindResultParts { value: (*value) };
            let Ok(value) = w::ObservabilityKindResult::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::ObservabilityKind::Result(value)
        }
        d::Kind::Effect(value) => {
            let parts = w::ObservabilityKindEffectParts { value: crate::wire::proposal_encode(value)? };
            let Ok(value) = w::ObservabilityKindEffect::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::ObservabilityKind::Effect(value)
        }
        d::Kind::Outbox(value) => {
            let parts = w::ObservabilityKindOutboxParts { value: (*value) };
            let Ok(value) = w::ObservabilityKindOutbox::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::ObservabilityKind::Outbox(value)
        }
        d::Kind::Hold(value) => {
            let parts = w::ObservabilityKindHoldParts { value: (*value) };
            let Ok(value) = w::ObservabilityKindHold::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::ObservabilityKind::Hold(value)
        }
        d::Kind::Project(value) => {
            let parts = w::ObservabilityKindProjectParts { value: (*value) };
            let Ok(value) = w::ObservabilityKindProject::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::ObservabilityKind::Project(value)
        }
        d::Kind::Deployment => w::ObservabilityKind::Deployment,
    })
}

pub(crate) fn kind_decode(value: w::ObservabilityKind) -> d::Kind {
    match value {
        w::ObservabilityKind::Incidents(value) => {
            let value = value.into_parts();
            d::Kind::Incidents { open: value.open }
        }
        w::ObservabilityKind::Incident(value) => {
            let value = value.into_parts();
            d::Kind::Incident(value.value)
        }
        w::ObservabilityKind::Panel(value) => {
            let value = value.into_parts();
            d::Kind::Panel(value.value)
        }
        w::ObservabilityKind::Result(value) => {
            let value = value.into_parts();
            d::Kind::Result(value.value)
        }
        w::ObservabilityKind::Effect(value) => {
            let value = value.into_parts();
            d::Kind::Effect(crate::wire::proposal_decode(value.value))
        }
        w::ObservabilityKind::Outbox(value) => {
            let value = value.into_parts();
            d::Kind::Outbox(value.value)
        }
        w::ObservabilityKind::Hold(value) => {
            let value = value.into_parts();
            d::Kind::Hold(value.value)
        }
        w::ObservabilityKind::Project(value) => {
            let value = value.into_parts();
            d::Kind::Project(value.value)
        }
        w::ObservabilityKind::Deployment => d::Kind::Deployment,
    }
}

pub(crate) fn subject_encode(value: &d::Subject) -> Result<w::ObservabilitySubject, Invalid> {
    let parts = w::ObservabilitySubjectParts { scope: scope_encode(value.scope)?, kind: kind_encode(&(value.kind))? };
    let Ok(value) = w::ObservabilitySubject::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}

pub(crate) fn subject_decode(value: w::ObservabilitySubject) -> d::Subject {
    let value = value.into_parts();
    d::Subject { scope: scope_decode(value.scope), kind: kind_decode(value.kind) }
}

pub(crate) fn service_encode(value: &d::Service) -> Result<w::ObservabilityService, Invalid> {
    let parts = w::ObservabilityServiceParts { environment: (value.environment).clone(), name: (value.name).clone() };
    let Ok(value) = w::ObservabilityService::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}

pub(crate) fn service_decode(value: w::ObservabilityService) -> d::Service {
    let value = value.into_parts();
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
                w::ObservabilityDocumentValueParts { value: crate::wire::observability_data::document_encode(value)? };
            let Ok(value) = w::ObservabilityDocumentValue::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::ObservabilityDocument::Value(value)
        }
    })
}

pub(crate) fn document_decode(value: w::ObservabilityDocument) -> d::Document {
    match value {
        w::ObservabilityDocument::Value(value) => {
            let value = value.into_parts();
            d::Document::Value(Box::new(crate::wire::observability_data::document_decode(value.value)))
        }
    }
}
