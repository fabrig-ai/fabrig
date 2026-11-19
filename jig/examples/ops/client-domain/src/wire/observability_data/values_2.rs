//! Whole-value mappings for observability/values.
use crate::observability::values as d;
use crate::wire::{Invalid, w};
use skein_lib::List;
pub(crate) fn value_encode(value: &d::Value) -> Result<w::ObservabilityValue, Invalid> {
    Ok(match value {
        d::Value::Service { service, fact } => {
            let value = w::ObservabilityValueService {
                service: crate::wire::observability_values::service_encode(service),
                fact: crate::wire::observability_data::fact_encode(fact)?,
            };
            w::ObservabilityValue::Service(value)
        }
        d::Value::Alert { alert, raised, state, fact } => {
            let value = w::ObservabilityValueAlert {
                alert: crate::wire::observability_data::alert_encode(alert),
                raised: (*raised).as_nanos(),
                state: crate::wire::observability_values::state_encode(*state),
                fact: match fact {
                    Some(item) => Some(crate::wire::observability_data::fact_encode(item)?),
                    None => None,
                },
            };
            w::ObservabilityValue::Alert(value)
        }
        d::Value::Outbox { entry } => {
            let parts = w::ObservabilityValueOutbox { entry: crate::wire::observability_data::entry_encode(entry) };
            let value = parts;
            w::ObservabilityValue::Outbox(value)
        }
        d::Value::Deployment { watches, effects, read_limit, incident_limit } => {
            let value = w::ObservabilityValueDeployment {
                watches: (*watches),
                effects: {
                    let source = effects;
                    let mut values = List::with_capacity(source.capacity());
                    for item in source {
                        let value = crate::wire::observability_data::entry_encode(item);
                        if values.push(value).is_err() {
                            return Err(Invalid);
                        }
                    }
                    values
                },
                read_limit: (*read_limit),
                incident_limit: (*incident_limit),
            };
            w::ObservabilityValue::Deployment(value)
        }
        d::Value::Incident { incident, fact } => {
            let value = w::ObservabilityValueIncident {
                incident: crate::wire::observability_data::incident_encode(incident)?,
                fact: match fact {
                    Some(item) => Some(crate::wire::observability_data::fact_encode(item)?),
                    None => None,
                },
            };
            w::ObservabilityValue::Incident(value)
        }
        d::Value::Project { project, services, incidents } => {
            let value = w::ObservabilityValueProject {
                project: (*project),
                services: {
                    let source = services;
                    let mut values = List::with_capacity(source.capacity());
                    for item in source {
                        let value = crate::wire::observability_data::service_state_encode(item)?;
                        if values.push(value).is_err() {
                            return Err(Invalid);
                        }
                    }
                    values
                },
                incidents: {
                    let source = incidents;
                    let mut values = List::with_capacity(source.capacity());
                    for item in source {
                        let value = crate::wire::observability_data::incident_encode(item)?;
                        if values.push(value).is_err() {
                            return Err(Invalid);
                        }
                    }
                    values
                },
            };
            w::ObservabilityValue::Project(value)
        }
    })
}
pub(crate) fn value_decode(value: w::ObservabilityValue) -> d::Value {
    match value {
        w::ObservabilityValue::Service(value) => d::Value::Service {
            service: crate::wire::observability_values::service_decode(value.service),
            fact: crate::wire::observability_data::fact_decode(value.fact),
        },
        w::ObservabilityValue::Alert(value) => d::Value::Alert {
            alert: crate::wire::observability_data::alert_decode(value.alert),
            raised: skein_lib::Wall::from_nanos(value.raised),
            state: crate::wire::observability_values::state_decode(value.state),
            fact: match value.fact {
                Some(item) => Some(crate::wire::observability_data::fact_decode(item)),
                None => None,
            },
        },
        w::ObservabilityValue::Outbox(value) => {
            d::Value::Outbox { entry: crate::wire::observability_data::entry_decode(value.entry) }
        }
        w::ObservabilityValue::Deployment(value) => d::Value::Deployment {
            watches: value.watches,
            effects: {
                let source = value.effects;
                let mut values = List::with_capacity(source.capacity());
                for item in source.into_boxed() {
                    let value = crate::wire::observability_data::entry_decode(item);
                    assert!(values.push(value).is_ok(), "same list capacity");
                }
                values
            },
            read_limit: value.read_limit,
            incident_limit: value.incident_limit,
        },
        w::ObservabilityValue::Incident(value) => d::Value::Incident {
            incident: crate::wire::observability_data::incident_decode(value.incident),
            fact: match value.fact {
                Some(item) => Some(crate::wire::observability_data::fact_decode(item)),
                None => None,
            },
        },
        w::ObservabilityValue::Project(value) => d::Value::Project {
            project: value.project,
            services: {
                let source = value.services;
                let mut values = List::with_capacity(source.capacity());
                for item in source.into_boxed() {
                    let value = crate::wire::observability_data::service_state_decode(item);
                    assert!(values.push(value).is_ok(), "same list capacity");
                }
                values
            },
            incidents: {
                let source = value.incidents;
                let mut values = List::with_capacity(source.capacity());
                for item in source.into_boxed() {
                    let value = crate::wire::observability_data::incident_decode(item);
                    assert!(values.push(value).is_ok(), "same list capacity");
                }
                values
            },
        },
    }
}

pub(crate) fn document_encode(value: &d::Document) -> Result<w::ObservabilityDocumentData, Invalid> {
    let value = w::ObservabilityDocumentData {
        subject: crate::wire::observability_values::subject_encode(&(value.subject)),
        value: value_encode(&(value.value))?,
    };
    Ok(value)
}
pub(crate) fn document_decode(value: w::ObservabilityDocumentData) -> d::Document {
    d::Document {
        subject: crate::wire::observability_values::subject_decode(value.subject),
        value: value_decode(value.value),
    }
}
