//! Whole-value mappings for infrastructure/values.
use crate::infrastructure::values as d;
use crate::wire::{Invalid, w};
use skein_lib::List;
pub(crate) fn service_encode(value: &d::Service) -> Result<w::InfrastructureService, Invalid> {
    let parts = w::InfrastructureServiceParts { environment: (value.environment).clone(), name: (value.name).clone() };
    let Ok(value) = w::InfrastructureService::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
pub(crate) fn service_decode(value: w::InfrastructureService) -> d::Service {
    let value = value.into_parts();
    d::Service { environment: value.environment, name: value.name }
}

pub(crate) fn environment_encode(value: &d::Environment) -> Result<w::InfrastructureEnvironment, Invalid> {
    let parts = w::InfrastructureEnvironmentParts { pool: (value.pool).clone(), name: (value.name).clone() };
    let Ok(value) = w::InfrastructureEnvironment::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
pub(crate) fn environment_decode(value: w::InfrastructureEnvironment) -> d::Environment {
    let value = value.into_parts();
    d::Environment { pool: value.pool, name: value.name }
}

pub(crate) fn pool_encode(value: &d::Pool) -> Result<w::InfrastructurePool, Invalid> {
    let parts = w::InfrastructurePoolParts { value: (value.0).clone() };
    let Ok(value) = w::InfrastructurePool::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
pub(crate) fn pool_decode(value: w::InfrastructurePool) -> d::Pool {
    let value = value.into_parts();
    d::Pool(value.value)
}

pub(crate) fn resource_encode(value: &d::Resource) -> Result<w::InfrastructureResource, Invalid> {
    Ok(match value {
        d::Resource::Service(value) => {
            let parts = w::InfrastructureResourceServiceParts { value: service_encode(value)? };
            let Ok(value) = w::InfrastructureResourceService::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::InfrastructureResource::Service(value)
        }
        d::Resource::Environment(value) => {
            let parts = w::InfrastructureResourceEnvironmentParts { value: environment_encode(value)? };
            let Ok(value) = w::InfrastructureResourceEnvironment::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::InfrastructureResource::Environment(value)
        }
        d::Resource::Pool(value) => {
            let parts = w::InfrastructureResourcePoolParts { value: pool_encode(value)? };
            let Ok(value) = w::InfrastructureResourcePool::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureResource::Pool(value)
        }
        d::Resource::Quota(value) => {
            let parts = w::InfrastructureResourceQuotaParts { value: pool_encode(value)? };
            let Ok(value) = w::InfrastructureResourceQuota::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureResource::Quota(value)
        }
    })
}
pub(crate) fn resource_decode(value: w::InfrastructureResource) -> d::Resource {
    match value {
        w::InfrastructureResource::Service(value) => {
            let value = value.into_parts();
            d::Resource::Service(service_decode(value.value))
        }
        w::InfrastructureResource::Environment(value) => {
            let value = value.into_parts();
            d::Resource::Environment(environment_decode(value.value))
        }
        w::InfrastructureResource::Pool(value) => {
            let value = value.into_parts();
            d::Resource::Pool(pool_decode(value.value))
        }
        w::InfrastructureResource::Quota(value) => {
            let value = value.into_parts();
            d::Resource::Quota(pool_decode(value.value))
        }
    }
}

pub(crate) fn hold_encode(value: d::Hold) -> w::InfrastructureHold {
    match value {
        d::Hold::ExclusiveWait => w::InfrastructureHold::ExclusiveWait,
        d::Hold::PooledWait => w::InfrastructureHold::PooledWait,
        d::Hold::None => w::InfrastructureHold::None,
    }
}
pub(crate) fn hold_decode(value: w::InfrastructureHold) -> d::Hold {
    match value {
        w::InfrastructureHold::ExclusiveWait => d::Hold::ExclusiveWait,
        w::InfrastructureHold::PooledWait => d::Hold::PooledWait,
        w::InfrastructureHold::None => d::Hold::None,
    }
}

pub(crate) fn resource_role_encode(value: d::ResourceRole) -> w::InfrastructureResourceRole {
    match value {
        d::ResourceRole::Owned => w::InfrastructureResourceRole::Owned,
        d::ResourceRole::Participant => w::InfrastructureResourceRole::Participant,
        d::ResourceRole::Context => w::InfrastructureResourceRole::Context,
        d::ResourceRole::Unavailable => w::InfrastructureResourceRole::Unavailable,
    }
}
pub(crate) fn resource_role_decode(value: w::InfrastructureResourceRole) -> d::ResourceRole {
    match value {
        w::InfrastructureResourceRole::Owned => d::ResourceRole::Owned,
        w::InfrastructureResourceRole::Participant => d::ResourceRole::Participant,
        w::InfrastructureResourceRole::Context => d::ResourceRole::Context,
        w::InfrastructureResourceRole::Unavailable => d::ResourceRole::Unavailable,
    }
}

pub(crate) fn named_encode(value: &d::Named) -> Result<w::InfrastructureNamed, Invalid> {
    let parts = w::InfrastructureNamedParts {
        resource: resource_encode(&(value.resource))?,
        role: resource_role_encode(value.role),
        hold: hold_encode(value.hold),
    };
    let Ok(value) = w::InfrastructureNamed::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
pub(crate) fn named_decode(value: w::InfrastructureNamed) -> d::Named {
    let value = value.into_parts();
    d::Named {
        resource: resource_decode(value.resource),
        role: resource_role_decode(value.role),
        hold: hold_decode(value.hold),
    }
}

pub(crate) fn purpose_encode(value: &d::Purpose) -> Result<w::InfrastructurePurpose, Invalid> {
    Ok(match value {
        d::Purpose::Call { attempt, completion, position } => {
            let parts = w::InfrastructurePurposeCallParts {
                attempt: (*attempt),
                completion: (*completion),
                position: (*position),
            };
            let Ok(value) = w::InfrastructurePurposeCall::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructurePurpose::Call(value)
        }
        d::Purpose::Procedure { purpose } => {
            let parts = w::InfrastructurePurposeProcedureParts { purpose: (*purpose) };
            let Ok(value) = w::InfrastructurePurposeProcedure::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::InfrastructurePurpose::Procedure(value)
        }
        d::Purpose::Projection { purpose } => {
            let parts = w::InfrastructurePurposeProjectionParts { purpose: (*purpose) };
            let Ok(value) = w::InfrastructurePurposeProjection::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::InfrastructurePurpose::Projection(value)
        }
    })
}
pub(crate) fn purpose_decode(value: w::InfrastructurePurpose) -> d::Purpose {
    match value {
        w::InfrastructurePurpose::Call(value) => {
            let value = value.into_parts();
            d::Purpose::Call { attempt: value.attempt, completion: value.completion, position: value.position }
        }
        w::InfrastructurePurpose::Procedure(value) => {
            let value = value.into_parts();
            d::Purpose::Procedure { purpose: value.purpose }
        }
        w::InfrastructurePurpose::Projection(value) => {
            let value = value.into_parts();
            d::Purpose::Projection { purpose: value.purpose }
        }
    }
}

pub(crate) fn key_encode(value: &d::Key) -> Result<w::InfrastructureKey, Invalid> {
    let parts = w::InfrastructureKeyParts {
        deployment: (value.deployment),
        task: (value.task),
        origin: purpose_encode(&(value.origin))?,
        purpose: (value.purpose),
    };
    let Ok(value) = w::InfrastructureKey::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
pub(crate) fn key_decode(value: w::InfrastructureKey) -> d::Key {
    let value = value.into_parts();
    d::Key {
        deployment: value.deployment,
        task: value.task,
        origin: purpose_decode(value.origin),
        purpose: value.purpose,
    }
}

pub(crate) fn backend_encode(value: d::Backend) -> w::InfrastructureBackend {
    match value {
        d::Backend::OperationIds => w::InfrastructureBackend::OperationIds,
        d::Backend::NoOperationIds => w::InfrastructureBackend::NoOperationIds,
    }
}
pub(crate) fn backend_decode(value: w::InfrastructureBackend) -> d::Backend {
    match value {
        w::InfrastructureBackend::OperationIds => d::Backend::OperationIds,
        w::InfrastructureBackend::NoOperationIds => d::Backend::NoOperationIds,
    }
}

pub(crate) fn effect_encode(value: &d::Effect) -> Result<w::InfrastructureEffect, Invalid> {
    Ok(match value {
        d::Effect::Restart { service, operation } => {
            let parts =
                w::InfrastructureEffectRestartParts { service: service_encode(service)?, operation: (*operation) };
            let Ok(value) = w::InfrastructureEffectRestart::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureEffect::Restart(value)
        }
        d::Effect::Scale { service, from, to } => {
            let parts =
                w::InfrastructureEffectScaleParts { service: service_encode(service)?, from: (*from), to: (*to) };
            let Ok(value) = w::InfrastructureEffectScale::new(&w::CEILINGS, parts) else { return Err(Invalid) };
            w::InfrastructureEffect::Scale(value)
        }
        d::Effect::Rollback { service, from, to } => {
            let parts = w::InfrastructureEffectRollbackParts {
                service: service_encode(service)?,
                from: (*from).clone(),
                to: (*to).clone(),
            };
            let Ok(value) = w::InfrastructureEffectRollback::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::InfrastructureEffect::Rollback(value)
        }
        d::Effect::CreateEnvironment { environment, until, price } => {
            let parts = w::InfrastructureEffectCreateEnvironmentParts {
                environment: environment_encode(environment)?,
                until: (*until),
                price: (*price),
            };
            let Ok(value) = w::InfrastructureEffectCreateEnvironment::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::InfrastructureEffect::CreateEnvironment(value)
        }
        d::Effect::TearDown { environment, created_by } => {
            let parts = w::InfrastructureEffectTearDownParts {
                environment: environment_encode(environment)?,
                created_by: key_encode(created_by)?,
            };
            let Ok(value) = w::InfrastructureEffectTearDown::new(&w::CEILINGS, parts) else {
                return Err(Invalid);
            };
            w::InfrastructureEffect::TearDown(value)
        }
    })
}
pub(crate) fn effect_decode(value: w::InfrastructureEffect) -> d::Effect {
    match value {
        w::InfrastructureEffect::Restart(value) => {
            let value = value.into_parts();
            d::Effect::Restart { service: service_decode(value.service), operation: value.operation }
        }
        w::InfrastructureEffect::Scale(value) => {
            let value = value.into_parts();
            d::Effect::Scale { service: service_decode(value.service), from: value.from, to: value.to }
        }
        w::InfrastructureEffect::Rollback(value) => {
            let value = value.into_parts();
            d::Effect::Rollback { service: service_decode(value.service), from: value.from, to: value.to }
        }
        w::InfrastructureEffect::CreateEnvironment(value) => {
            let value = value.into_parts();
            d::Effect::CreateEnvironment {
                environment: environment_decode(value.environment),
                until: value.until,
                price: value.price,
            }
        }
        w::InfrastructureEffect::TearDown(value) => {
            let value = value.into_parts();
            d::Effect::TearDown {
                environment: environment_decode(value.environment),
                created_by: key_decode(value.created_by),
            }
        }
    }
}

pub(crate) fn form_encode(value: d::Form) -> w::InfrastructureForm {
    match value {
        d::Form::Creation => w::InfrastructureForm::Creation,
        d::Form::Transition => w::InfrastructureForm::Transition,
        d::Form::Set => w::InfrastructureForm::Set,
    }
}
pub(crate) fn form_decode(value: w::InfrastructureForm) -> d::Form {
    match value {
        w::InfrastructureForm::Creation => d::Form::Creation,
        w::InfrastructureForm::Transition => d::Form::Transition,
        w::InfrastructureForm::Set => d::Form::Set,
    }
}

pub(crate) fn recovery_encode(value: d::Recovery) -> w::InfrastructureRecovery {
    match value {
        d::Recovery::Keyed => w::InfrastructureRecovery::Keyed,
        d::Recovery::Conditional => w::InfrastructureRecovery::Conditional,
        d::Recovery::Unrecoverable => w::InfrastructureRecovery::Unrecoverable,
    }
}
pub(crate) fn recovery_decode(value: w::InfrastructureRecovery) -> d::Recovery {
    match value {
        w::InfrastructureRecovery::Keyed => d::Recovery::Keyed,
        w::InfrastructureRecovery::Conditional => d::Recovery::Conditional,
        w::InfrastructureRecovery::Unrecoverable => d::Recovery::Unrecoverable,
    }
}

#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn description_encode(value: &d::Description) -> Result<w::InfrastructureDescription, Invalid> {
    let parts = w::InfrastructureDescriptionParts {
        kind: (value.kind),
        resources: {
            let source = &(value.resources);
            let Ok(capacity) = u32::try_from(source.len()) else { return Err(Invalid) };
            let mut values = List::with_capacity(capacity);
            for item in source {
                let value = resource_encode(item)?;
                if values.push(value).is_err() {
                    return Err(Invalid);
                }
            }
            values
        },
        condition: (value.condition),
        form: form_encode(value.form),
        recovery: recovery_encode(value.recovery),
        price: match &(value.price) {
            Some(item) => Some(*item),
            None => None,
        },
        state: (value.state),
    };
    let Ok(value) = w::InfrastructureDescription::new(&w::CEILINGS, parts) else { return Err(Invalid) };
    Ok(value)
}
#[expect(clippy::manual_map, reason = "step code matches optional fields without closures")]
pub(crate) fn description_decode(value: w::InfrastructureDescription) -> d::Description {
    let value = value.into_parts();
    d::Description {
        kind: value.kind,
        resources: {
            let source = value.resources;
            let mut values = List::with_capacity(source.capacity());
            for item in source.into_boxed() {
                let value = resource_decode(item);
                assert!(values.push(value).is_ok(), "same list capacity");
            }
            values.into_boxed()
        },
        condition: value.condition,
        form: form_decode(value.form),
        recovery: recovery_decode(value.recovery),
        price: match value.price {
            Some(item) => Some(item),
            None => None,
        },
        state: value.state,
    }
}

pub(crate) fn phase_encode(value: d::Phase) -> w::InfrastructurePhase {
    match value {
        d::Phase::Kept => w::InfrastructurePhase::Kept,
        d::Phase::Sent => w::InfrastructurePhase::Sent,
        d::Phase::Uncertain => w::InfrastructurePhase::Uncertain,
        d::Phase::Held => w::InfrastructurePhase::Held,
    }
}
pub(crate) fn phase_decode(value: w::InfrastructurePhase) -> d::Phase {
    match value {
        w::InfrastructurePhase::Kept => d::Phase::Kept,
        w::InfrastructurePhase::Sent => d::Phase::Sent,
        w::InfrastructurePhase::Uncertain => d::Phase::Uncertain,
        w::InfrastructurePhase::Held => d::Phase::Held,
    }
}
