//! Whole-value mappings for observability/values.
use crate::observability::values as d;
use crate::wire::{Invalid, w};
use skein_lib::List;
pub(crate) fn load_point_encode(value: &d::LoadPoint) -> w::ObservabilityLoadPoint {
    w::ObservabilityLoadPoint { observed: (value.observed), load_percent: (value.load_percent) }
}
pub(crate) fn load_point_decode(value: w::ObservabilityLoadPoint) -> d::LoadPoint {
    d::LoadPoint { observed: value.observed, load_percent: value.load_percent }
}

pub(crate) fn fact_encode(value: &d::Fact) -> Result<w::ObservabilityFact, Invalid> {
    let value = w::ObservabilityFact {
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
                let value = load_point_encode(item);
                if values.push(value).is_err() {
                    return Err(Invalid);
                }
            }
            values
        },
    };
    Ok(value)
}
pub(crate) fn fact_decode(value: w::ObservabilityFact) -> d::Fact {
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

pub(crate) fn health_encode(value: &d::Health) -> w::ObservabilityHealth {
    w::ObservabilityHealth {
        observed: (value.observed),
        healthy_replicas: (value.healthy_replicas),
        errors: (value.errors),
        error_rate_percent: (value.error_rate_percent),
    }
}
pub(crate) fn health_decode(value: w::ObservabilityHealth) -> d::Health {
    d::Health {
        observed: value.observed,
        healthy_replicas: value.healthy_replicas,
        errors: value.errors,
        error_rate_percent: value.error_rate_percent,
    }
}

pub(crate) fn point_encode(value: &d::Point) -> w::ObservabilityPoint {
    w::ObservabilityPoint {
        at: (value.at).as_nanos(),
        health: match &(value.health) {
            Some(item) => Some(health_encode(item)),
            None => None,
        },
    }
}
pub(crate) fn point_decode(value: w::ObservabilityPoint) -> d::Point {
    d::Point {
        at: skein_lib::Wall::from_nanos(value.at),
        health: match value.health {
            Some(item) => Some(health_decode(item)),
            None => None,
        },
    }
}
pub(crate) fn incident_encode(value: &d::Incident) -> Result<w::ObservabilityIncident, Invalid> {
    let value = w::ObservabilityIncident {
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
        service: crate::wire::observability_values::service_encode(&(value.service)),
        severity: (value.severity),
        state: crate::wire::observability_values::state_encode(value.state),
        timeline: {
            let source = &(value.timeline);
            let mut values = List::with_capacity(source.capacity());
            for item in source {
                let value = point_encode(item);
                if values.push(value).is_err() {
                    return Err(Invalid);
                }
            }
            values
        },
        cut: (value.cut),
    };
    Ok(value)
}
pub(crate) fn incident_decode(value: w::ObservabilityIncident) -> d::Incident {
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
    let value = w::ObservabilityServiceState {
        service: crate::wire::observability_values::service_encode(&(value.service)),
        fact: match &(value.fact) {
            Some(item) => Some(fact_encode(item)?),
            None => None,
        },
    };
    Ok(value)
}
pub(crate) fn service_state_decode(value: w::ObservabilityServiceState) -> d::ServiceState {
    d::ServiceState {
        service: crate::wire::observability_values::service_decode(value.service),
        fact: match value.fact {
            Some(item) => Some(fact_decode(item)),
            None => None,
        },
    }
}

pub(crate) fn alert_encode(value: &d::Alert) -> w::ObservabilityAlert {
    w::ObservabilityAlert {
        number: (value.number),
        service: crate::wire::observability_values::service_encode(&(value.service)),
        severity: (value.severity),
    }
}
pub(crate) fn alert_decode(value: w::ObservabilityAlert) -> d::Alert {
    d::Alert {
        number: value.number,
        service: crate::wire::observability_values::service_decode(value.service),
        severity: value.severity,
    }
}

pub(crate) fn purpose_encode(value: &d::Purpose) -> w::ObservabilityPurpose {
    match value {
        d::Purpose::Call { attempt, completion, position } => {
            let value =
                w::ObservabilityPurposeCall { attempt: (*attempt), completion: (*completion), position: (*position) };
            w::ObservabilityPurpose::Call(value)
        }
        d::Purpose::Procedure { purpose } => {
            let value = w::ObservabilityPurposeProcedure { purpose: (*purpose) };
            w::ObservabilityPurpose::Procedure(value)
        }
        d::Purpose::Projection { purpose } => {
            let value = w::ObservabilityPurposeProjection { purpose: (*purpose) };
            w::ObservabilityPurpose::Projection(value)
        }
    }
}
pub(crate) fn purpose_decode(value: w::ObservabilityPurpose) -> d::Purpose {
    match value {
        w::ObservabilityPurpose::Call(value) => {
            d::Purpose::Call { attempt: value.attempt, completion: value.completion, position: value.position }
        }
        w::ObservabilityPurpose::Procedure(value) => d::Purpose::Procedure { purpose: value.purpose },
        w::ObservabilityPurpose::Projection(value) => d::Purpose::Projection { purpose: value.purpose },
    }
}

pub(crate) fn key_encode(value: &d::Key) -> w::ObservabilityKey {
    w::ObservabilityKey {
        deployment: (value.deployment),
        task: (value.task),
        origin: purpose_encode(&(value.origin)),
        purpose: (value.purpose),
    }
}
pub(crate) fn key_decode(value: w::ObservabilityKey) -> d::Key {
    d::Key {
        deployment: value.deployment,
        task: value.task,
        origin: purpose_decode(value.origin),
        purpose: value.purpose,
    }
}

pub(crate) fn effect_encode(value: &d::Effect) -> w::ObservabilityEffect {
    w::ObservabilityEffect { rule: (value.rule).clone(), until: (value.until) }
}
pub(crate) fn effect_decode(value: w::ObservabilityEffect) -> d::Effect {
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
pub(crate) fn entry_encode(value: &d::Entry) -> w::ObservabilityEntry {
    w::ObservabilityEntry {
        project: match &(value.project) {
            Some(item) => Some(*item),
            None => None,
        },
        number: (value.number),
        key: key_encode(&(value.key)),
        effect: effect_encode(&(value.effect)),
        attempt: (value.attempt),
        deadline: (value.deadline),
        phase: phase_encode(value.phase),
    }
}
pub(crate) fn entry_decode(value: w::ObservabilityEntry) -> d::Entry {
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
