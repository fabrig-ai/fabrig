//! Whole-value mappings for observability/values.
use crate::observability::values as d;
use crate::wire::{Invalid, w};
use skein_lib::List;
pub(crate) fn value_encode(value: &d::Value) -> Result<w::ObservabilityValue, Invalid> {
    Ok(match value {
        d::Value::Service { service, fact } => {
            let parts = w::ObservabilityValueServiceParts {
                service: crate::wire::observability_values::service_encode(service)?,
                fact: crate::wire::observability_data::fact_encode(fact)?,
            };
            let Ok(value) = w::ObservabilityValueService::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::ObservabilityValue::Service(value)
        }
        d::Value::Alert { alert, raised, state, fact } => {
            let parts = w::ObservabilityValueAlertParts {
                alert: crate::wire::observability_data::alert_encode(alert)?,
                raised: (*raised).as_nanos(),
                state: crate::wire::observability_values::state_encode(*state),
                fact: match fact {
                    Some(item) => Some(crate::wire::observability_data::fact_encode(item)?),
                    None => None,
                },
            };
            let Ok(value) = w::ObservabilityValueAlert::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::ObservabilityValue::Alert(value)
        }
        d::Value::Outbox { entry } => {
            let parts =
                w::ObservabilityValueOutboxParts { entry: crate::wire::observability_data::entry_encode(entry)? };
            let Ok(value) = w::ObservabilityValueOutbox::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::ObservabilityValue::Outbox(value)
        }
        d::Value::Deployment { watches, effects, read_limit, incident_limit } => {
            let parts = w::ObservabilityValueDeploymentParts {
                watches: (*watches),
                effects: {
                    let source = effects;
                    let mut values = List::with_capacity(source.capacity());
                    for item in source {
                        let value = crate::wire::observability_data::entry_encode(item)?;
                        if values.push(value).is_err() {
                            return Err(Invalid);
                        }
                    }
                    values
                },
                read_limit: (*read_limit),
                incident_limit: (*incident_limit),
            };
            let Ok(value) = w::ObservabilityValueDeployment::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::ObservabilityValue::Deployment(value)
        }
        d::Value::Incident { incident, fact } => {
            let parts = w::ObservabilityValueIncidentParts {
                incident: crate::wire::observability_data::incident_encode(incident)?,
                fact: match fact {
                    Some(item) => Some(crate::wire::observability_data::fact_encode(item)?),
                    None => None,
                },
            };
            let Ok(value) = w::ObservabilityValueIncident::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::ObservabilityValue::Incident(value)
        }
        d::Value::Project { project, services, incidents } => {
            let parts = w::ObservabilityValueProjectParts {
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
            let Ok(value) = w::ObservabilityValueProject::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::ObservabilityValue::Project(value)
        }
    })
}
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn value_decode(value: w::ObservabilityValue) -> d::Value {
    match value {
        w::ObservabilityValue::Service(value) => {
            let value = value.into_parts();
            d::Value::Service {
                service: crate::wire::observability_values::service_decode(value.service),
                fact: crate::wire::observability_data::fact_decode(value.fact),
            }
        }
        w::ObservabilityValue::Alert(value) => {
            let value = value.into_parts();
            d::Value::Alert {
                alert: crate::wire::observability_data::alert_decode(value.alert),
                raised: skein_lib::Wall::from_nanos(value.raised),
                state: crate::wire::observability_values::state_decode(value.state),
                fact: match value.fact {
                    Some(item) => Some(crate::wire::observability_data::fact_decode(item)),
                    None => None,
                },
            }
        }
        w::ObservabilityValue::Outbox(value) => {
            let value = value.into_parts();
            d::Value::Outbox { entry: crate::wire::observability_data::entry_decode(value.entry) }
        }
        w::ObservabilityValue::Deployment(value) => {
            let value = value.into_parts();
            d::Value::Deployment {
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
            }
        }
        w::ObservabilityValue::Incident(value) => {
            let value = value.into_parts();
            d::Value::Incident {
                incident: crate::wire::observability_data::incident_decode(value.incident),
                fact: match value.fact {
                    Some(item) => Some(crate::wire::observability_data::fact_decode(item)),
                    None => None,
                },
            }
        }
        w::ObservabilityValue::Project(value) => {
            let value = value.into_parts();
            d::Value::Project {
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
            }
        }
    }
}

pub(crate) fn document_encode(value: &d::Document) -> Result<w::ObservabilityDocumentData, Invalid> {
    let parts = w::ObservabilityDocumentDataParts {
        subject: crate::wire::observability_values::subject_encode(&(value.subject))?,
        value: value_encode(&(value.value))?,
    };
    let Ok(value) = w::ObservabilityDocumentData::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
pub(crate) fn document_decode(value: w::ObservabilityDocumentData) -> d::Document {
    let value = value.into_parts();
    d::Document {
        subject: crate::wire::observability_values::subject_decode(value.subject),
        value: value_decode(value.value),
    }
}
