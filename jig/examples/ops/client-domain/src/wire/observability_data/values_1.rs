//! Whole-value mappings for observability/values.
use crate::observability::values as d;
use crate::wire::{Invalid, w};
use skein_lib::List;
pub(crate) fn load_point_encode(value: &d::LoadPoint) -> Result<w::ObservabilityLoadPoint, Invalid> {
    let parts = w::ObservabilityLoadPointParts { observed: (value.observed), load_percent: (value.load_percent) };
    let Ok(value) = w::ObservabilityLoadPoint::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
pub(crate) fn load_point_decode(value: w::ObservabilityLoadPoint) -> d::LoadPoint {
    let value = value.into_parts();
    d::LoadPoint { observed: value.observed, load_percent: value.load_percent }
}

pub(crate) fn fact_encode(value: &d::Fact) -> Result<w::ObservabilityFact, Invalid> {
    let parts = w::ObservabilityFactParts {
        observed: (value.observed),
        healthy_replicas: (value.healthy_replicas),
        errors: (value.errors),
        error_rate_percent: (value.error_rate_percent),
        load_percent: (value.load_percent),
        load: {
            let source = &(value.load);
            let Ok(capacity) = u32::try_from(source.len()) else { return Err(Invalid) };
            let mut values = List::with_capacity(capacity);
            for item in source {
                let value = load_point_encode(item)?;
                if values.push(value).is_err() {
                    return Err(Invalid);
                }
            }
            values
        },
    };
    let Ok(value) = w::ObservabilityFact::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
pub(crate) fn fact_decode(value: w::ObservabilityFact) -> d::Fact {
    let value = value.into_parts();
    d::Fact {
        observed: value.observed,
        healthy_replicas: value.healthy_replicas,
        errors: value.errors,
        error_rate_percent: value.error_rate_percent,
        load_percent: value.load_percent,
        load: {
            let source = value.load;
            let mut values = List::with_capacity(source.capacity());
            for item in source.into_boxed() {
                let value = load_point_decode(item);
                assert!(values.push(value).is_ok(), "same list capacity");
            }
            values.into_boxed()
        },
    }
}

pub(crate) fn health_encode(value: &d::Health) -> Result<w::ObservabilityHealth, Invalid> {
    let parts = w::ObservabilityHealthParts {
        observed: (value.observed),
        healthy_replicas: (value.healthy_replicas),
        errors: (value.errors),
        error_rate_percent: (value.error_rate_percent),
    };
    let Ok(value) = w::ObservabilityHealth::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
pub(crate) fn health_decode(value: w::ObservabilityHealth) -> d::Health {
    let value = value.into_parts();
    d::Health {
        observed: value.observed,
        healthy_replicas: value.healthy_replicas,
        errors: value.errors,
        error_rate_percent: value.error_rate_percent,
    }
}

pub(crate) fn point_encode(value: &d::Point) -> Result<w::ObservabilityPoint, Invalid> {
    let parts = w::ObservabilityPointParts {
        at: (value.at).as_nanos(),
        health: match &(value.health) {
            Some(item) => Some(health_encode(item)?),
            None => None,
        },
    };
    let Ok(value) = w::ObservabilityPoint::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn point_decode(value: w::ObservabilityPoint) -> d::Point {
    let value = value.into_parts();
    d::Point {
        at: skein_lib::Wall::from_nanos(value.at),
        health: match value.health {
            Some(item) => Some(health_decode(item)),
            None => None,
        },
    }
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn incident_encode(value: &d::Incident) -> Result<w::ObservabilityIncident, Invalid> {
    let parts = w::ObservabilityIncidentParts {
        project: (value.project),
        number: (value.number),
        watch: (value.watch),
        lead: match &(value.lead) {
            Some(item) => Some(*item),
            None => None,
        },
        alerts: {
            let source = &(value.alerts);
            let Ok(capacity) = u32::try_from(source.len()) else { return Err(Invalid) };
            let mut values = List::with_capacity(capacity);
            for item in source {
                let value = *item;
                if values.push(value).is_err() {
                    return Err(Invalid);
                }
            }
            values
        },
        raised: (value.raised).as_nanos(),
        service: crate::wire::observability_values::service_encode(&(value.service))?,
        severity: (value.severity),
        state: crate::wire::observability_values::state_encode(value.state),
        timeline: {
            let source = &(value.timeline);
            let mut values = List::with_capacity(source.capacity());
            for item in source {
                let value = point_encode(item)?;
                if values.push(value).is_err() {
                    return Err(Invalid);
                }
            }
            values
        },
        cut: (value.cut),
    };
    let Ok(value) = w::ObservabilityIncident::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn incident_decode(value: w::ObservabilityIncident) -> d::Incident {
    let value = value.into_parts();
    d::Incident {
        project: value.project,
        number: value.number,
        watch: value.watch,
        lead: match value.lead {
            Some(item) => Some(item),
            None => None,
        },
        alerts: {
            let source = value.alerts;
            let mut values = List::with_capacity(source.capacity());
            for item in source.into_boxed() {
                let value = item;
                assert!(values.push(value).is_ok(), "same list capacity");
            }
            values.into_boxed()
        },
        raised: skein_lib::Wall::from_nanos(value.raised),
        service: crate::wire::observability_values::service_decode(value.service),
        severity: value.severity,
        state: crate::wire::observability_values::state_decode(value.state),
        timeline: {
            let source = value.timeline;
            let mut values = List::with_capacity(source.capacity());
            for item in source.into_boxed() {
                let value = point_decode(item);
                assert!(values.push(value).is_ok(), "same list capacity");
            }
            values
        },
        cut: value.cut,
    }
}

pub(crate) fn service_state_encode(value: &d::ServiceState) -> Result<w::ObservabilityServiceState, Invalid> {
    let parts = w::ObservabilityServiceStateParts {
        service: crate::wire::observability_values::service_encode(&(value.service))?,
        fact: match &(value.fact) {
            Some(item) => Some(fact_encode(item)?),
            None => None,
        },
    };
    let Ok(value) = w::ObservabilityServiceState::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn service_state_decode(value: w::ObservabilityServiceState) -> d::ServiceState {
    let value = value.into_parts();
    d::ServiceState {
        service: crate::wire::observability_values::service_decode(value.service),
        fact: match value.fact {
            Some(item) => Some(fact_decode(item)),
            None => None,
        },
    }
}

pub(crate) fn alert_encode(value: &d::Alert) -> Result<w::ObservabilityAlert, Invalid> {
    let parts = w::ObservabilityAlertParts {
        number: (value.number),
        service: crate::wire::observability_values::service_encode(&(value.service))?,
        severity: (value.severity),
    };
    let Ok(value) = w::ObservabilityAlert::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
pub(crate) fn alert_decode(value: w::ObservabilityAlert) -> d::Alert {
    let value = value.into_parts();
    d::Alert {
        number: value.number,
        service: crate::wire::observability_values::service_decode(value.service),
        severity: value.severity,
    }
}

pub(crate) fn purpose_encode(value: &d::Purpose) -> Result<w::ObservabilityPurpose, Invalid> {
    Ok(match value {
        d::Purpose::Call { attempt, completion, position } => {
            let parts = w::ObservabilityPurposeCallParts {
                attempt: (*attempt),
                completion: (*completion),
                position: (*position),
            };
            let Ok(value) = w::ObservabilityPurposeCall::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::ObservabilityPurpose::Call(value)
        }
        d::Purpose::Procedure { purpose } => {
            let parts = w::ObservabilityPurposeProcedureParts { purpose: (*purpose) };
            let Ok(value) = w::ObservabilityPurposeProcedure::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::ObservabilityPurpose::Procedure(value)
        }
        d::Purpose::Projection { purpose } => {
            let parts = w::ObservabilityPurposeProjectionParts { purpose: (*purpose) };
            let Ok(value) = w::ObservabilityPurposeProjection::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::ObservabilityPurpose::Projection(value)
        }
    })
}
pub(crate) fn purpose_decode(value: w::ObservabilityPurpose) -> d::Purpose {
    match value {
        w::ObservabilityPurpose::Call(value) => {
            let value = value.into_parts();
            d::Purpose::Call { attempt: value.attempt, completion: value.completion, position: value.position }
        }
        w::ObservabilityPurpose::Procedure(value) => {
            let value = value.into_parts();
            d::Purpose::Procedure { purpose: value.purpose }
        }
        w::ObservabilityPurpose::Projection(value) => {
            let value = value.into_parts();
            d::Purpose::Projection { purpose: value.purpose }
        }
    }
}

pub(crate) fn key_encode(value: &d::Key) -> Result<w::ObservabilityKey, Invalid> {
    let parts = w::ObservabilityKeyParts {
        deployment: (value.deployment),
        task: (value.task),
        origin: purpose_encode(&(value.origin))?,
        purpose: (value.purpose),
    };
    let Ok(value) = w::ObservabilityKey::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
pub(crate) fn key_decode(value: w::ObservabilityKey) -> d::Key {
    let value = value.into_parts();
    d::Key {
        deployment: value.deployment,
        task: value.task,
        origin: purpose_decode(value.origin),
        purpose: value.purpose,
    }
}

pub(crate) fn effect_encode(value: &d::Effect) -> Result<w::ObservabilityEffect, Invalid> {
    let parts = w::ObservabilityEffectParts { rule: (value.rule).clone(), until: (value.until) };
    let Ok(value) = w::ObservabilityEffect::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
pub(crate) fn effect_decode(value: w::ObservabilityEffect) -> d::Effect {
    let value = value.into_parts();
    d::Effect { rule: value.rule, until: value.until }
}

pub(crate) fn phase_encode(value: d::Phase) -> w::ObservabilityPhase {
    match value {
        d::Phase::Kept => w::ObservabilityPhase::Kept,
        d::Phase::Sent => w::ObservabilityPhase::Sent,
        d::Phase::Uncertain => w::ObservabilityPhase::Uncertain,
        d::Phase::Held => w::ObservabilityPhase::Held,
    }
}
pub(crate) fn phase_decode(value: w::ObservabilityPhase) -> d::Phase {
    match value {
        w::ObservabilityPhase::Kept => d::Phase::Kept,
        w::ObservabilityPhase::Sent => d::Phase::Sent,
        w::ObservabilityPhase::Uncertain => d::Phase::Uncertain,
        w::ObservabilityPhase::Held => d::Phase::Held,
    }
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn entry_encode(value: &d::Entry) -> Result<w::ObservabilityEntry, Invalid> {
    let parts = w::ObservabilityEntryParts {
        project: match &(value.project) {
            Some(item) => Some(*item),
            None => None,
        },
        number: (value.number),
        key: key_encode(&(value.key))?,
        effect: effect_encode(&(value.effect))?,
        attempt: (value.attempt),
        deadline: (value.deadline),
        phase: phase_encode(value.phase),
    };
    let Ok(value) = w::ObservabilityEntry::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn entry_decode(value: w::ObservabilityEntry) -> d::Entry {
    let value = value.into_parts();
    d::Entry {
        project: match value.project {
            Some(item) => Some(item),
            None => None,
        },
        number: value.number,
        key: key_decode(value.key),
        effect: effect_decode(value.effect),
        attempt: value.attempt,
        deadline: value.deadline,
        phase: phase_decode(value.phase),
    }
}
