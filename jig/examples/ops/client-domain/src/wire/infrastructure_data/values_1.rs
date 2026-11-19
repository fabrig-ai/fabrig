//! Whole-value mappings for infrastructure/values.
use crate::infrastructure::values as d;
use crate::wire::{Invalid, w};
use skein_lib::List;
pub(crate) fn service_encode(value: &d::Service) -> w::InfrastructureService {
    w::InfrastructureService { environment: (value.environment).clone(), name: (value.name).clone() }
}
pub(crate) fn service_decode(value: w::InfrastructureService) -> d::Service {
    d::Service { environment: value.environment, name: value.name }
}

pub(crate) fn environment_encode(value: &d::Environment) -> w::InfrastructureEnvironment {
    w::InfrastructureEnvironment { pool: (value.pool).clone(), name: (value.name).clone() }
}
pub(crate) fn environment_decode(value: w::InfrastructureEnvironment) -> d::Environment {
    d::Environment { pool: value.pool, name: value.name }
}

pub(crate) fn pool_encode(value: &d::Pool) -> w::InfrastructurePool {
    w::InfrastructurePool { value: (value.0).clone() }
}
pub(crate) fn pool_decode(value: w::InfrastructurePool) -> d::Pool {
    d::Pool(value.value)
}

pub(crate) fn resource_encode(value: &d::Resource) -> w::InfrastructureResource {
    match value {
        d::Resource::Service(value) => {
            let value = w::InfrastructureResourceService { value: service_encode(value) };
            w::InfrastructureResource::Service(value)
        }
        d::Resource::Environment(value) => {
            let value = w::InfrastructureResourceEnvironment { value: environment_encode(value) };
            w::InfrastructureResource::Environment(value)
        }
        d::Resource::Pool(value) => {
            let value = w::InfrastructureResourcePool { value: pool_encode(value) };
            w::InfrastructureResource::Pool(value)
        }
        d::Resource::Quota(value) => {
            let value = w::InfrastructureResourceQuota { value: pool_encode(value) };
            w::InfrastructureResource::Quota(value)
        }
    }
}
pub(crate) fn resource_decode(value: w::InfrastructureResource) -> d::Resource {
    match value {
        w::InfrastructureResource::Service(value) => d::Resource::Service(service_decode(value.value)),
        w::InfrastructureResource::Environment(value) => d::Resource::Environment(environment_decode(value.value)),
        w::InfrastructureResource::Pool(value) => d::Resource::Pool(pool_decode(value.value)),
        w::InfrastructureResource::Quota(value) => d::Resource::Quota(pool_decode(value.value)),
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

pub(crate) fn named_encode(value: &d::Named) -> w::InfrastructureNamed {
    w::InfrastructureNamed {
        resource: resource_encode(&(value.resource)),
        role: resource_role_encode(value.role),
        hold: hold_encode(value.hold),
    }
}
pub(crate) fn named_decode(value: w::InfrastructureNamed) -> d::Named {
    d::Named {
        resource: resource_decode(value.resource),
        role: resource_role_decode(value.role),
        hold: hold_decode(value.hold),
    }
}

pub(crate) fn purpose_encode(value: &d::Purpose) -> w::InfrastructurePurpose {
    match value {
        d::Purpose::Call { attempt, completion, position } => {
            let value =
                w::InfrastructurePurposeCall { attempt: (*attempt), completion: (*completion), position: (*position) };
            w::InfrastructurePurpose::Call(value)
        }
        d::Purpose::Procedure { purpose } => {
            let value = w::InfrastructurePurposeProcedure { purpose: (*purpose) };
            w::InfrastructurePurpose::Procedure(value)
        }
        d::Purpose::Projection { purpose } => {
            let value = w::InfrastructurePurposeProjection { purpose: (*purpose) };
            w::InfrastructurePurpose::Projection(value)
        }
    }
}
pub(crate) fn purpose_decode(value: w::InfrastructurePurpose) -> d::Purpose {
    match value {
        w::InfrastructurePurpose::Call(value) => {
            d::Purpose::Call { attempt: value.attempt, completion: value.completion, position: value.position }
        }
        w::InfrastructurePurpose::Procedure(value) => d::Purpose::Procedure { purpose: value.purpose },
        w::InfrastructurePurpose::Projection(value) => d::Purpose::Projection { purpose: value.purpose },
    }
}

pub(crate) fn key_encode(value: &d::Key) -> w::InfrastructureKey {
    w::InfrastructureKey {
        deployment: (value.deployment),
        task: (value.task),
        origin: purpose_encode(&(value.origin)),
        purpose: (value.purpose),
    }
}
pub(crate) fn key_decode(value: w::InfrastructureKey) -> d::Key {
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

pub(crate) fn effect_encode(value: &d::Effect) -> w::InfrastructureEffect {
    match value {
        d::Effect::Restart { service, operation } => {
            let parts = w::InfrastructureEffectRestart { service: service_encode(service), operation: (*operation) };
            let value = parts;
            w::InfrastructureEffect::Restart(value)
        }
        d::Effect::Scale { service, from, to } => {
            let parts = w::InfrastructureEffectScale { service: service_encode(service), from: (*from), to: (*to) };
            let value = parts;
            w::InfrastructureEffect::Scale(value)
        }
        d::Effect::Rollback { service, from, to } => {
            let value = w::InfrastructureEffectRollback {
                service: service_encode(service),
                from: (*from).clone(),
                to: (*to).clone(),
            };
            w::InfrastructureEffect::Rollback(value)
        }
        d::Effect::CreateEnvironment { environment, until, price } => {
            let value = w::InfrastructureEffectCreateEnvironment {
                environment: environment_encode(environment),
                until: (*until),
                price: (*price),
            };
            w::InfrastructureEffect::CreateEnvironment(value)
        }
        d::Effect::TearDown { environment, created_by } => {
            let value = w::InfrastructureEffectTearDown {
                environment: environment_encode(environment),
                created_by: key_encode(created_by),
            };
            w::InfrastructureEffect::TearDown(value)
        }
    }
}
pub(crate) fn effect_decode(value: w::InfrastructureEffect) -> d::Effect {
    match value {
        w::InfrastructureEffect::Restart(value) => {
            d::Effect::Restart { service: service_decode(value.service), operation: value.operation }
        }
        w::InfrastructureEffect::Scale(value) => {
            d::Effect::Scale { service: service_decode(value.service), from: value.from, to: value.to }
        }
        w::InfrastructureEffect::Rollback(value) => {
            d::Effect::Rollback { service: service_decode(value.service), from: value.from, to: value.to }
        }
        w::InfrastructureEffect::CreateEnvironment(value) => d::Effect::CreateEnvironment {
            environment: environment_decode(value.environment),
            until: value.until,
            price: value.price,
        },
        w::InfrastructureEffect::TearDown(value) => d::Effect::TearDown {
            environment: environment_decode(value.environment),
            created_by: key_decode(value.created_by),
        },
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
pub(crate) fn description_encode(value: &d::Description) -> Result<w::InfrastructureDescription, Invalid> {
    let value = w::InfrastructureDescription {
        kind: (value.kind),
        resources: {
            let source = &(value.resources);
            let Ok(capacity) = u32::try_from(source.len()) else { return Err(Invalid) };
            let mut values = List::with_capacity(capacity);
            for item in source {
                let value = resource_encode(item);
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
    Ok(value)
}
pub(crate) fn description_decode(value: w::InfrastructureDescription) -> d::Description {
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
