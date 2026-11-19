//! Whole-value mappings for infrastructure/values.
use crate::infrastructure::values as d;
use crate::wire::{Invalid, w};
use skein_lib::List;
pub(crate) fn value_encode(value: &d::Value) -> Result<w::InfrastructureValue, Invalid> {
    Ok(match value {
        d::Value::Environment { environment, end_revision, fact, end, pool } => {
            let value = w::InfrastructureValueEnvironment {
                environment: crate::wire::infrastructure_data::environment_encode(environment),
                end_revision: (*end_revision),
                fact: crate::wire::infrastructure_data::environment_fact_encode(fact),
                end: match end {
                    Some(item) => Some(crate::wire::infrastructure_data::environment_end_encode(item)),
                    None => None,
                },
                pool: match pool {
                    Some(item) => Some(crate::wire::infrastructure_data::client_pool_encode(*item)),
                    None => None,
                },
            };
            w::InfrastructureValue::Environment(value)
        }
        d::Value::Panel { task, state, resources, effects } => {
            let value = w::InfrastructureValuePanel {
                task: (*task),
                state: crate::wire::infrastructure_data::procedure_state_encode(state),
                resources: resources_encode(resources)?,
                effects: effects_encode(effects)?,
            };
            w::InfrastructureValue::Panel(value)
        }
        d::Value::Result { task, state, resources } => {
            let value = w::InfrastructureValueResult {
                task: (*task),
                state: crate::wire::infrastructure_data::procedure_state_encode(state),
                resources: resources_encode(resources)?,
            };
            w::InfrastructureValue::Result(value)
        }
        d::Value::Effect { proposal, task, effect, description } => {
            let value = w::InfrastructureValueEffect {
                proposal: (*proposal),
                task: (*task),
                effect: crate::wire::infrastructure_data::effect_encode(effect),
                description: crate::wire::infrastructure_data::description_encode(description)?,
            };
            w::InfrastructureValue::Effect(value)
        }
        d::Value::Outbox { entry, description } => {
            let value = w::InfrastructureValueOutbox {
                entry: crate::wire::infrastructure_data::entry_encode(entry),
                description: crate::wire::infrastructure_data::description_encode(description)?,
            };
            w::InfrastructureValue::Outbox(value)
        }
        d::Value::Hold { task, resources, drift, effects } => {
            let value = w::InfrastructureValueHold {
                task: (*task),
                resources: resources_encode(resources)?,
                drift: drift_encode(drift)?,
                effects: effects_encode(effects)?,
            };
            w::InfrastructureValue::Hold(value)
        }
        d::Value::Project { project, resources, environments } => {
            let value = w::InfrastructureValueProject {
                project: (*project),
                resources: resources_encode(resources)?,
                environments: environments_encode(environments)?,
            };
            w::InfrastructureValue::Project(value)
        }
        d::Value::Deployment { backend, effects, made, proposal_limit, staged_limit } => {
            let value = w::InfrastructureValueDeployment {
                backend: crate::wire::infrastructure_data::backend_encode(*backend),
                effects: effects_encode(effects)?,
                made: (*made),
                proposal_limit: (*proposal_limit),
                staged_limit: (*staged_limit),
            };
            w::InfrastructureValue::Deployment(value)
        }
    })
}
pub(crate) fn value_decode(value: w::InfrastructureValue) -> d::Value {
    match value {
        w::InfrastructureValue::Environment(value) => d::Value::Environment {
            environment: crate::wire::infrastructure_data::environment_decode(value.environment),
            end_revision: value.end_revision,
            fact: crate::wire::infrastructure_data::environment_fact_decode(value.fact),
            end: match value.end {
                Some(item) => Some(crate::wire::infrastructure_data::environment_end_decode(item)),
                None => None,
            },
            pool: match value.pool {
                Some(item) => Some(crate::wire::infrastructure_data::client_pool_decode(item)),
                None => None,
            },
        },
        w::InfrastructureValue::Panel(value) => d::Value::Panel {
            task: value.task,
            state: crate::wire::infrastructure_data::procedure_state_decode(value.state),
            resources: resources_decode(value.resources),
            effects: effects_decode(value.effects),
        },
        w::InfrastructureValue::Result(value) => d::Value::Result {
            task: value.task,
            state: crate::wire::infrastructure_data::procedure_state_decode(value.state),
            resources: resources_decode(value.resources),
        },
        w::InfrastructureValue::Effect(value) => d::Value::Effect {
            proposal: value.proposal,
            task: value.task,
            effect: crate::wire::infrastructure_data::effect_decode(value.effect),
            description: crate::wire::infrastructure_data::description_decode(value.description),
        },
        w::InfrastructureValue::Outbox(value) => d::Value::Outbox {
            entry: crate::wire::infrastructure_data::entry_decode(value.entry),
            description: crate::wire::infrastructure_data::description_decode(value.description),
        },
        w::InfrastructureValue::Hold(value) => d::Value::Hold {
            task: value.task,
            resources: resources_decode(value.resources),
            drift: drift_decode(value.drift),
            effects: effects_decode(value.effects),
        },
        w::InfrastructureValue::Project(value) => d::Value::Project {
            project: value.project,
            resources: resources_decode(value.resources),
            environments: environments_decode(value.environments),
        },
        w::InfrastructureValue::Deployment(value) => d::Value::Deployment {
            backend: crate::wire::infrastructure_data::backend_decode(value.backend),
            effects: effects_decode(value.effects),
            made: value.made,
            proposal_limit: value.proposal_limit,
            staged_limit: value.staged_limit,
        },
    }
}

pub(crate) fn document_encode(value: &d::Document) -> Result<w::InfrastructureDocumentData, Invalid> {
    let value = w::InfrastructureDocumentData {
        subject: crate::wire::infrastructure_values::subject_encode(&(value.subject)),
        value: value_encode(&(value.value))?,
    };
    Ok(value)
}
pub(crate) fn document_decode(value: w::InfrastructureDocumentData) -> d::Document {
    d::Document {
        subject: crate::wire::infrastructure_values::subject_decode(value.subject),
        value: value_decode(value.value),
    }
}

fn resources_encode(source: &List<d::ClientResource>) -> Result<List<w::InfrastructureClientResource>, Invalid> {
    let mut values = List::with_capacity(source.capacity());
    for item in source {
        let value = crate::wire::infrastructure_data::client_resource_encode(item);
        if values.push(value).is_err() {
            return Err(Invalid);
        }
    }
    Ok(values)
}

fn resources_decode(source: List<w::InfrastructureClientResource>) -> List<d::ClientResource> {
    let mut values = List::with_capacity(source.capacity());
    for item in source.into_boxed() {
        let value = crate::wire::infrastructure_data::client_resource_decode(item);
        assert!(values.push(value).is_ok(), "same list capacity");
    }
    values
}

fn effects_encode(source: &List<d::Entry>) -> Result<List<w::InfrastructureEntry>, Invalid> {
    let mut values = List::with_capacity(source.capacity());
    for item in source {
        let value = crate::wire::infrastructure_data::entry_encode(item);
        if values.push(value).is_err() {
            return Err(Invalid);
        }
    }
    Ok(values)
}

fn effects_decode(source: List<w::InfrastructureEntry>) -> List<d::Entry> {
    let mut values = List::with_capacity(source.capacity());
    for item in source.into_boxed() {
        let value = crate::wire::infrastructure_data::entry_decode(item);
        assert!(values.push(value).is_ok(), "same list capacity");
    }
    values
}

fn drift_encode(source: &List<d::DriftAccount>) -> Result<List<w::InfrastructureDriftAccount>, Invalid> {
    let mut values = List::with_capacity(source.capacity());
    for item in source {
        let value = crate::wire::infrastructure_data::drift_account_encode(item);
        if values.push(value).is_err() {
            return Err(Invalid);
        }
    }
    Ok(values)
}

fn drift_decode(source: List<w::InfrastructureDriftAccount>) -> List<d::DriftAccount> {
    let mut values = List::with_capacity(source.capacity());
    for item in source.into_boxed() {
        let value = crate::wire::infrastructure_data::drift_account_decode(item);
        assert!(values.push(value).is_ok(), "same list capacity");
    }
    values
}

fn environments_encode(
    source: &List<d::ClientEnvironment>,
) -> Result<List<w::InfrastructureClientEnvironment>, Invalid> {
    let mut values = List::with_capacity(source.capacity());
    for item in source {
        let value = crate::wire::infrastructure_data::client_environment_encode(item);
        if values.push(value).is_err() {
            return Err(Invalid);
        }
    }
    Ok(values)
}

fn environments_decode(source: List<w::InfrastructureClientEnvironment>) -> List<d::ClientEnvironment> {
    let mut values = List::with_capacity(source.capacity());
    for item in source.into_boxed() {
        let value = crate::wire::infrastructure_data::client_environment_decode(item);
        assert!(values.push(value).is_ok(), "same list capacity");
    }
    values
}
