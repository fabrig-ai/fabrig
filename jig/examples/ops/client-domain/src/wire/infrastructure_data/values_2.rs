//! Whole-value mappings for infrastructure/values.
use crate::infrastructure::values as d;
use crate::wire::w;
pub(crate) fn entry_encode(value: &d::Entry) -> w::InfrastructureEntry {
    w::InfrastructureEntry {
        project: match &(value.project) {
            Some(item) => Some(*item),
            None => None,
        },
        number: (value.number),
        key: crate::wire::infrastructure_data::key_encode(&(value.key)),
        effect: crate::wire::infrastructure_data::effect_encode(&(value.effect)),
        attempt: (value.attempt),
        deadline: (value.deadline),
        phase: crate::wire::infrastructure_data::phase_encode(value.phase),
    }
}
pub(crate) fn entry_decode(value: w::InfrastructureEntry) -> d::Entry {
    d::Entry {
        project: match value.project {
            Some(item) => Some(item),
            None => None,
        },
        number: value.number,
        key: crate::wire::infrastructure_data::key_decode(value.key),
        effect: crate::wire::infrastructure_data::effect_decode(value.effect),
        attempt: value.attempt,
        deadline: value.deadline,
        phase: crate::wire::infrastructure_data::phase_decode(value.phase),
    }
}

pub(crate) fn service_fact_encode(value: &d::ServiceFact) -> w::InfrastructureServiceFact {
    w::InfrastructureServiceFact {
        version: (value.version).clone(),
        replicas: (value.replicas),
        healthy: (value.healthy),
        revision: (value.revision),
        observed: (value.observed),
    }
}
pub(crate) fn service_fact_decode(value: w::InfrastructureServiceFact) -> d::ServiceFact {
    d::ServiceFact {
        version: value.version,
        replicas: value.replicas,
        healthy: value.healthy,
        revision: value.revision,
        observed: value.observed,
    }
}

pub(crate) fn environment_fact_encode(value: &d::EnvironmentFact) -> w::InfrastructureEnvironmentFact {
    w::InfrastructureEnvironmentFact {
        created_by: crate::wire::infrastructure_data::key_encode(&(value.created_by)),
        ready_at: (value.ready_at),
        until: (value.until),
        price: (value.price),
    }
}
pub(crate) fn environment_fact_decode(value: w::InfrastructureEnvironmentFact) -> d::EnvironmentFact {
    d::EnvironmentFact {
        created_by: crate::wire::infrastructure_data::key_decode(value.created_by),
        ready_at: value.ready_at,
        until: value.until,
        price: value.price,
    }
}

pub(crate) fn procedure_encode(value: &d::Procedure) -> w::InfrastructureProcedure {
    match value {
        d::Procedure::Remediate { service, operation, deadline } => {
            let value = w::InfrastructureProcedureRemediate {
                service: crate::wire::infrastructure_data::service_encode(service),
                operation: (*operation),
                deadline: (*deadline),
            };
            w::InfrastructureProcedure::Remediate(value)
        }
        d::Procedure::Scale { service, replicas, deadline } => {
            let value = w::InfrastructureProcedureScale {
                service: crate::wire::infrastructure_data::service_encode(service),
                replicas: (*replicas),
                deadline: (*deadline),
            };
            w::InfrastructureProcedure::Scale(value)
        }
        d::Procedure::Provision { environment, until, price, deadline } => {
            let value = w::InfrastructureProcedureProvision {
                environment: crate::wire::infrastructure_data::environment_encode(environment),
                until: (*until),
                price: (*price),
                deadline: (*deadline),
            };
            w::InfrastructureProcedure::Provision(value)
        }
        d::Procedure::TearDown { environment, created_by, deadline } => {
            let value = w::InfrastructureProcedureTearDown {
                environment: crate::wire::infrastructure_data::environment_encode(environment),
                created_by: crate::wire::infrastructure_data::key_encode(created_by),
                deadline: (*deadline),
            };
            w::InfrastructureProcedure::TearDown(value)
        }
    }
}
pub(crate) fn procedure_decode(value: w::InfrastructureProcedure) -> d::Procedure {
    match value {
        w::InfrastructureProcedure::Remediate(value) => d::Procedure::Remediate {
            service: crate::wire::infrastructure_data::service_decode(value.service),
            operation: value.operation,
            deadline: value.deadline,
        },
        w::InfrastructureProcedure::Scale(value) => d::Procedure::Scale {
            service: crate::wire::infrastructure_data::service_decode(value.service),
            replicas: value.replicas,
            deadline: value.deadline,
        },
        w::InfrastructureProcedure::Provision(value) => d::Procedure::Provision {
            environment: crate::wire::infrastructure_data::environment_decode(value.environment),
            until: value.until,
            price: value.price,
            deadline: value.deadline,
        },
        w::InfrastructureProcedure::TearDown(value) => d::Procedure::TearDown {
            environment: crate::wire::infrastructure_data::environment_decode(value.environment),
            created_by: crate::wire::infrastructure_data::key_decode(value.created_by),
            deadline: value.deadline,
        },
    }
}

pub(crate) fn procedure_phase_encode(value: d::ProcedurePhase) -> w::InfrastructureProcedurePhase {
    match value {
        d::ProcedurePhase::Active => w::InfrastructureProcedurePhase::Active,
        d::ProcedurePhase::ReadingFacts => w::InfrastructureProcedurePhase::ReadingFacts,
        d::ProcedurePhase::FactsReady => w::InfrastructureProcedurePhase::FactsReady,
        d::ProcedurePhase::EffectOutstanding => w::InfrastructureProcedurePhase::EffectOutstanding,
        d::ProcedurePhase::WaitingCondition => w::InfrastructureProcedurePhase::WaitingCondition,
        d::ProcedurePhase::Holding => w::InfrastructureProcedurePhase::Holding,
        d::ProcedurePhase::Done => w::InfrastructureProcedurePhase::Done,
        d::ProcedurePhase::Held => w::InfrastructureProcedurePhase::Held,
    }
}
pub(crate) fn procedure_phase_decode(value: w::InfrastructureProcedurePhase) -> d::ProcedurePhase {
    match value {
        w::InfrastructureProcedurePhase::Active => d::ProcedurePhase::Active,
        w::InfrastructureProcedurePhase::ReadingFacts => d::ProcedurePhase::ReadingFacts,
        w::InfrastructureProcedurePhase::FactsReady => d::ProcedurePhase::FactsReady,
        w::InfrastructureProcedurePhase::EffectOutstanding => d::ProcedurePhase::EffectOutstanding,
        w::InfrastructureProcedurePhase::WaitingCondition => d::ProcedurePhase::WaitingCondition,
        w::InfrastructureProcedurePhase::Holding => d::ProcedurePhase::Holding,
        w::InfrastructureProcedurePhase::Done => d::ProcedurePhase::Done,
        w::InfrastructureProcedurePhase::Held => d::ProcedurePhase::Held,
    }
}

pub(crate) fn procedure_state_encode(value: &d::ProcedureState) -> w::InfrastructureProcedureState {
    w::InfrastructureProcedureState {
        task: (value.task),
        procedure: procedure_encode(&(value.procedure)),
        phase: procedure_phase_encode(value.phase),
    }
}
pub(crate) fn procedure_state_decode(value: w::InfrastructureProcedureState) -> d::ProcedureState {
    d::ProcedureState {
        task: value.task,
        procedure: procedure_decode(value.procedure),
        phase: procedure_phase_decode(value.phase),
    }
}

pub(crate) fn drift_kind_encode(value: d::DriftKind) -> w::InfrastructureDriftKind {
    match value {
        d::DriftKind::Changed => w::InfrastructureDriftKind::Changed,
        d::DriftKind::Deleted => w::InfrastructureDriftKind::Deleted,
    }
}
pub(crate) fn drift_kind_decode(value: w::InfrastructureDriftKind) -> d::DriftKind {
    match value {
        w::InfrastructureDriftKind::Changed => d::DriftKind::Changed,
        w::InfrastructureDriftKind::Deleted => d::DriftKind::Deleted,
    }
}
pub(crate) fn drift_account_encode(value: &d::DriftAccount) -> w::InfrastructureDriftAccount {
    w::InfrastructureDriftAccount {
        resource: crate::wire::infrastructure_data::resource_encode(&(value.resource)),
        kind: drift_kind_encode(value.kind),
        at: (value.at).as_nanos(),
        by: match &(value.by) {
            Some(item) => Some((*item).clone()),
            None => None,
        },
    }
}
pub(crate) fn drift_account_decode(value: w::InfrastructureDriftAccount) -> d::DriftAccount {
    d::DriftAccount {
        resource: crate::wire::infrastructure_data::resource_decode(value.resource),
        kind: drift_kind_decode(value.kind),
        at: skein_lib::Wall::from_nanos(value.at),
        by: match value.by {
            Some(item) => Some(item),
            None => None,
        },
    }
}

pub(crate) fn environment_end_encode(value: &d::EnvironmentEnd) -> w::InfrastructureEnvironmentEnd {
    w::InfrastructureEnvironmentEnd { revision: (value.revision), until: (value.until) }
}
pub(crate) fn environment_end_decode(value: w::InfrastructureEnvironmentEnd) -> d::EnvironmentEnd {
    d::EnvironmentEnd { revision: value.revision, until: value.until }
}

pub(crate) fn client_pool_encode(value: d::ClientPool) -> w::InfrastructureClientPool {
    w::InfrastructureClientPool { quota: (value.quota), used: (value.used) }
}
pub(crate) fn client_pool_decode(value: w::InfrastructureClientPool) -> d::ClientPool {
    d::ClientPool { quota: value.quota, used: value.used }
}

pub(crate) fn resource_fact_encode(value: &d::ResourceFact) -> w::InfrastructureResourceFact {
    match value {
        d::ResourceFact::Service(value) => {
            let value = w::InfrastructureResourceFactService {
                value: match value {
                    Some(item) => Some(service_fact_encode(item)),
                    None => None,
                },
            };
            w::InfrastructureResourceFact::Service(value)
        }
        d::ResourceFact::Environment { fact, end } => {
            let value = w::InfrastructureResourceFactEnvironment {
                fact: match fact {
                    Some(item) => Some(environment_fact_encode(item)),
                    None => None,
                },
                end: match end {
                    Some(item) => Some(environment_end_encode(item)),
                    None => None,
                },
            };
            w::InfrastructureResourceFact::Environment(value)
        }
        d::ResourceFact::Pool(value) => {
            let value = w::InfrastructureResourceFactPool {
                value: match value {
                    Some(item) => Some(client_pool_encode(*item)),
                    None => None,
                },
            };
            w::InfrastructureResourceFact::Pool(value)
        }
        d::ResourceFact::Quota(value) => {
            let value = w::InfrastructureResourceFactQuota {
                value: match value {
                    Some(item) => Some(client_pool_encode(*item)),
                    None => None,
                },
            };
            w::InfrastructureResourceFact::Quota(value)
        }
    }
}
pub(crate) fn resource_fact_decode(value: w::InfrastructureResourceFact) -> d::ResourceFact {
    match value {
        w::InfrastructureResourceFact::Service(value) => d::ResourceFact::Service(match value.value {
            Some(item) => Some(service_fact_decode(item)),
            None => None,
        }),
        w::InfrastructureResourceFact::Environment(value) => d::ResourceFact::Environment {
            fact: match value.fact {
                Some(item) => Some(environment_fact_decode(item)),
                None => None,
            },
            end: match value.end {
                Some(item) => Some(environment_end_decode(item)),
                None => None,
            },
        },
        w::InfrastructureResourceFact::Pool(value) => d::ResourceFact::Pool(match value.value {
            Some(item) => Some(client_pool_decode(item)),
            None => None,
        }),
        w::InfrastructureResourceFact::Quota(value) => d::ResourceFact::Quota(match value.value {
            Some(item) => Some(client_pool_decode(item)),
            None => None,
        }),
    }
}

pub(crate) fn client_resource_encode(value: &d::ClientResource) -> w::InfrastructureClientResource {
    w::InfrastructureClientResource {
        named: crate::wire::infrastructure_data::named_encode(&(value.named)),
        fact: resource_fact_encode(&(value.fact)),
    }
}
pub(crate) fn client_resource_decode(value: w::InfrastructureClientResource) -> d::ClientResource {
    d::ClientResource {
        named: crate::wire::infrastructure_data::named_decode(value.named),
        fact: resource_fact_decode(value.fact),
    }
}

pub(crate) fn client_environment_encode(value: &d::ClientEnvironment) -> w::InfrastructureClientEnvironment {
    w::InfrastructureClientEnvironment {
        environment: crate::wire::infrastructure_data::environment_encode(&(value.environment)),
        fact: environment_fact_encode(&(value.fact)),
        end: match &(value.end) {
            Some(item) => Some(environment_end_encode(item)),
            None => None,
        },
        pool: match &(value.pool) {
            Some(item) => Some(client_pool_encode(*item)),
            None => None,
        },
    }
}
pub(crate) fn client_environment_decode(value: w::InfrastructureClientEnvironment) -> d::ClientEnvironment {
    d::ClientEnvironment {
        environment: crate::wire::infrastructure_data::environment_decode(value.environment),
        fact: environment_fact_decode(value.fact),
        end: match value.end {
            Some(item) => Some(environment_end_decode(item)),
            None => None,
        },
        pool: match value.pool {
            Some(item) => Some(client_pool_decode(item)),
            None => None,
        },
    }
}
