//! Infrastructure owner facts translated into the client family vocabulary.
use jig_ops_client_domain::infrastructure::values;
use jig_ops_domain_infrastructure as source;

#[must_use]
pub fn translate_service(value: source::Service) -> values::Service {
    values::Service { environment: value.environment, name: value.name }
}

#[must_use]
pub fn translate_environment(value: source::Environment) -> values::Environment {
    values::Environment { pool: value.pool, name: value.name }
}

#[must_use]
pub fn translate_pool(value: source::Pool) -> values::Pool {
    values::Pool(value.0)
}

#[must_use]
pub fn translate_resource(value: source::Resource) -> values::Resource {
    match value {
        source::Resource::Service(value) => values::Resource::Service(translate_service(value)),
        source::Resource::Environment(value) => values::Resource::Environment(translate_environment(value)),
        source::Resource::Pool(value) => values::Resource::Pool(translate_pool(value)),
        source::Resource::Quota(value) => values::Resource::Quota(translate_pool(value)),
    }
}

#[must_use]
pub fn translate_hold(value: source::Hold) -> values::Hold {
    match value {
        source::Hold::ExclusiveWait => values::Hold::ExclusiveWait,
        source::Hold::PooledWait => values::Hold::PooledWait,
        source::Hold::None => values::Hold::None,
    }
}

#[must_use]
pub fn translate_resource_role(value: source::ResourceRole) -> values::ResourceRole {
    match value {
        source::ResourceRole::Owned => values::ResourceRole::Owned,
        source::ResourceRole::Participant => values::ResourceRole::Participant,
        source::ResourceRole::Context => values::ResourceRole::Context,
        source::ResourceRole::Unavailable => values::ResourceRole::Unavailable,
    }
}

#[must_use]
pub fn translate_named(value: source::Named) -> values::Named {
    values::Named {
        resource: translate_resource(value.resource),
        role: translate_resource_role(value.role),
        hold: translate_hold(value.hold),
    }
}

#[must_use]
pub fn translate_purpose(value: source::Purpose) -> values::Purpose {
    match value {
        source::Purpose::Call { attempt, completion, position } => {
            values::Purpose::Call { attempt, completion, position }
        }
        source::Purpose::Procedure { purpose } => values::Purpose::Procedure { purpose },
        source::Purpose::Projection { purpose } => values::Purpose::Projection { purpose },
    }
}

#[must_use]
pub fn translate_key(value: source::Key) -> values::Key {
    values::Key {
        deployment: value.deployment,
        task: value.task,
        origin: translate_purpose(value.origin),
        purpose: value.purpose,
    }
}

#[must_use]
pub fn translate_backend(value: source::Backend) -> values::Backend {
    match value {
        source::Backend::OperationIds => values::Backend::OperationIds,
        source::Backend::NoOperationIds => values::Backend::NoOperationIds,
    }
}

#[must_use]
pub fn translate_effect(value: source::Effect) -> values::Effect {
    match value {
        source::Effect::Restart { service, operation } => {
            values::Effect::Restart { service: translate_service(service), operation }
        }
        source::Effect::Scale { service, from, to } => {
            values::Effect::Scale { service: translate_service(service), from, to }
        }
        source::Effect::Rollback { service, from, to } => {
            values::Effect::Rollback { service: translate_service(service), from, to }
        }
        source::Effect::CreateEnvironment { environment, until, price } => {
            values::Effect::CreateEnvironment { environment: translate_environment(environment), until, price }
        }
        source::Effect::TearDown { environment, created_by } => values::Effect::TearDown {
            environment: translate_environment(environment),
            created_by: translate_key(created_by),
        },
    }
}

#[must_use]
pub fn translate_form(value: source::Form) -> values::Form {
    match value {
        source::Form::Creation => values::Form::Creation,
        source::Form::Transition => values::Form::Transition,
        source::Form::Set => values::Form::Set,
    }
}

#[must_use]
pub fn translate_recovery(value: source::Recovery) -> values::Recovery {
    match value {
        source::Recovery::Keyed => values::Recovery::Keyed,
        source::Recovery::Conditional => values::Recovery::Conditional,
        source::Recovery::Unrecoverable => values::Recovery::Unrecoverable,
    }
}

#[must_use]
pub fn translate_description(value: source::Description) -> values::Description {
    values::Description {
        kind: value.kind,
        resources: value.resources.into_vec().into_iter().map(translate_resource).collect::<Vec<_>>().into(),
        condition: value.condition,
        form: translate_form(value.form),
        recovery: translate_recovery(value.recovery),
        price: value.price,
        state: value.state,
    }
}

#[must_use]
pub fn translate_phase(value: source::Phase) -> values::Phase {
    match value {
        source::Phase::Kept => values::Phase::Kept,
        source::Phase::Sent => values::Phase::Sent,
        source::Phase::Uncertain => values::Phase::Uncertain,
        source::Phase::Held => values::Phase::Held,
    }
}

#[must_use]
pub fn translate_entry(value: source::Entry) -> values::Entry {
    values::Entry {
        project: value.project,
        number: value.number,
        key: translate_key(value.key),
        effect: translate_effect(value.effect),
        attempt: value.attempt,
        deadline: value.deadline,
        phase: translate_phase(value.phase),
    }
}

#[must_use]
pub fn translate_service_fact(value: source::ServiceFact) -> values::ServiceFact {
    values::ServiceFact {
        version: value.version,
        replicas: value.replicas,
        healthy: value.healthy,
        revision: value.revision,
        observed: value.observed,
    }
}

#[must_use]
pub fn translate_environment_fact(value: source::EnvironmentFact) -> values::EnvironmentFact {
    values::EnvironmentFact {
        created_by: translate_key(value.created_by),
        ready_at: value.ready_at,
        until: value.until,
        price: value.price,
    }
}

#[must_use]
pub fn translate_procedure(value: source::Procedure) -> values::Procedure {
    match value {
        source::Procedure::Remediate { service, operation, deadline } => {
            values::Procedure::Remediate { service: translate_service(service), operation, deadline }
        }
        source::Procedure::Scale { service, replicas, deadline } => {
            values::Procedure::Scale { service: translate_service(service), replicas, deadline }
        }
        source::Procedure::Provision { environment, until, price, deadline } => {
            values::Procedure::Provision { environment: translate_environment(environment), until, price, deadline }
        }
        source::Procedure::TearDown { environment, created_by, deadline } => values::Procedure::TearDown {
            environment: translate_environment(environment),
            created_by: translate_key(created_by),
            deadline,
        },
    }
}

#[must_use]
pub fn translate_procedure_phase(value: source::ProcedurePhase) -> values::ProcedurePhase {
    match value {
        source::ProcedurePhase::Active => values::ProcedurePhase::Active,
        source::ProcedurePhase::ReadingFacts => values::ProcedurePhase::ReadingFacts,
        source::ProcedurePhase::FactsReady => values::ProcedurePhase::FactsReady,
        source::ProcedurePhase::EffectOutstanding => values::ProcedurePhase::EffectOutstanding,
        source::ProcedurePhase::WaitingCondition => values::ProcedurePhase::WaitingCondition,
        source::ProcedurePhase::Holding => values::ProcedurePhase::Holding,
        source::ProcedurePhase::Done => values::ProcedurePhase::Done,
        source::ProcedurePhase::Held => values::ProcedurePhase::Held,
    }
}

#[must_use]
pub fn translate_procedure_state(value: source::ProcedureState) -> values::ProcedureState {
    values::ProcedureState {
        task: value.task,
        procedure: translate_procedure(value.procedure),
        phase: translate_procedure_phase(value.phase),
    }
}

#[must_use]
pub fn translate_drift_kind(value: source::DriftKind) -> values::DriftKind {
    match value {
        source::DriftKind::Changed => values::DriftKind::Changed,
        source::DriftKind::Deleted => values::DriftKind::Deleted,
    }
}

#[must_use]
pub fn translate_drift_account(value: source::DriftAccount) -> values::DriftAccount {
    values::DriftAccount {
        resource: translate_resource(value.resource),
        kind: translate_drift_kind(value.kind),
        at: value.at,
        by: value.by,
    }
}

#[must_use]
pub fn translate_environment_end(value: source::EnvironmentEnd) -> values::EnvironmentEnd {
    values::EnvironmentEnd { revision: value.revision, until: value.until }
}

#[must_use]
pub fn translate_client_pool(value: source::ClientPool) -> values::ClientPool {
    values::ClientPool { quota: value.quota, used: value.used }
}

#[must_use]
pub fn translate_resource_fact(value: source::ResourceFact) -> values::ResourceFact {
    match value {
        source::ResourceFact::Service(value) => values::ResourceFact::Service(value.map(translate_service_fact)),
        source::ResourceFact::Environment { fact, end } => values::ResourceFact::Environment {
            fact: fact.map(translate_environment_fact),
            end: end.map(translate_environment_end),
        },
        source::ResourceFact::Pool(value) => values::ResourceFact::Pool(value.map(translate_client_pool)),
        source::ResourceFact::Quota(value) => values::ResourceFact::Quota(value.map(translate_client_pool)),
    }
}

#[must_use]
pub fn translate_client_resource(value: source::ClientResource) -> values::ClientResource {
    values::ClientResource { named: translate_named(value.named), fact: translate_resource_fact(value.fact) }
}

#[must_use]
pub fn translate_client_environment(value: source::ClientEnvironment) -> values::ClientEnvironment {
    values::ClientEnvironment {
        environment: translate_environment(value.environment),
        fact: translate_environment_fact(value.fact),
        end: value.end.map(translate_environment_end),
        pool: value.pool.map(translate_client_pool),
    }
}

fn translate_resources(source: skein_lib::List<source::ClientResource>) -> skein_lib::List<values::ClientResource> {
    let mut values = skein_lib::List::with_capacity(source.capacity());
    for value in source.into_boxed() {
        values.push(translate_client_resource(value)).expect("source list capacity");
    }
    values
}
#[must_use]
pub fn translate_value(value: source::ClientDocument) -> values::Value {
    match value {
        source::ClientDocument::Environment { environment, fact, end, pool, end_revision } => {
            values::Value::Environment {
                end_revision,
                environment: translate_environment(environment),
                fact: translate_environment_fact(fact),
                end: end.map(translate_environment_end),
                pool: pool.map(translate_client_pool),
            }
        }
        source::ClientDocument::Panel { task, state, resources, effects } => values::Value::Panel {
            task,
            state: translate_procedure_state(state),
            resources: translate_resources(resources),
            effects: {
                let source = effects;
                let mut values = skein_lib::List::with_capacity(source.capacity());
                for value in &source {
                    values.push(translate_entry(value.clone())).expect("source list capacity");
                }
                values
            },
        },
        source::ClientDocument::Result { task, state, resources } => values::Value::Result {
            task,
            state: translate_procedure_state(state),
            resources: translate_resources(resources),
        },
        source::ClientDocument::Effect { proposal, task, effect, description } => values::Value::Effect {
            proposal,
            task,
            effect: translate_effect(effect),
            description: translate_description(description),
        },
        source::ClientDocument::Outbox { entry, description } => {
            values::Value::Outbox { entry: translate_entry(entry), description: translate_description(description) }
        }
        source::ClientDocument::Hold { task, resources, drift, effects } => values::Value::Hold {
            task,
            resources: translate_resources(resources),
            drift: {
                let source = drift;
                let mut values = skein_lib::List::with_capacity(source.capacity());
                for value in &source {
                    values.push(translate_drift_account(value.clone())).expect("source list capacity");
                }
                values
            },
            effects: {
                let source = effects;
                let mut values = skein_lib::List::with_capacity(source.capacity());
                for value in &source {
                    values.push(translate_entry(value.clone())).expect("source list capacity");
                }
                values
            },
        },
        source::ClientDocument::Project { project, resources, environments } => values::Value::Project {
            project,
            resources: translate_resources(resources),
            environments: {
                let source = environments;
                let mut values = skein_lib::List::with_capacity(source.capacity());
                for value in &source {
                    values.push(translate_client_environment(value.clone())).expect("source list capacity");
                }
                values
            },
        },
        source::ClientDocument::Deployment { backend, effects, made, proposal_limit, staged_limit } => {
            values::Value::Deployment {
                backend: translate_backend(backend),
                effects: {
                    let source = effects;
                    let mut values = skein_lib::List::with_capacity(source.capacity());
                    for value in &source {
                        values.push(translate_entry(value.clone())).expect("source list capacity");
                    }
                    values
                },
                made,
                proposal_limit,
                staged_limit,
            }
        }
    }
}

/// The family wire bytes determine the protocol digest.
/// # Errors
/// The request must fit the family wire bounds.
pub fn ask(
    value: jig_ops_client_domain::infrastructure::Ask,
) -> Result<(u32, source::ApplicationRequest, [u8; 32]), &'static str> {
    use jig_ops_client_domain::infrastructure::Ask;
    use sha2::{Digest, Sha256};
    let bytes = jig_ops_client_domain::wire::ask(&value).map_err(|_| "request exceeds family wire bounds")?;
    let (project, value) = match value {
        Ask::MoveEnd { project, environment, revision, until, object: _ } => (
            project,
            source::ApplicationRequest {
                environment: source::Environment { pool: environment.pool, name: environment.name },
                revision: revision.value,
                until,
            },
        ),
    };
    let digest = Sha256::new().chain_update(&bytes);
    Ok((project, value, digest.finalize().into()))
}
#[must_use]
pub fn answer(value: source::ApplicationOutcome) -> jig_ops_client_domain::infrastructure::Answer {
    use jig_client_domain::contract::Revision;
    use jig_ops_client_domain::infrastructure::{Answer, documents::Refusal};
    match value {
        source::ApplicationOutcome::MovedEnd { environment, now } => Answer::MovedEnd {
            environment: translate_environment(environment),
            revision: Revision { value: now.revision },
            until: now.until,
        },
        source::ApplicationOutcome::Moved { environment, now } => Answer::Moved {
            environment: translate_environment(environment),
            revision: Revision { value: now.revision },
            until: now.until,
        },
        source::ApplicationOutcome::Role { held } => Answer::Refused { why: Refusal::Role { held } },
        source::ApplicationOutcome::Unknown => Answer::Gone,
        source::ApplicationOutcome::Past => Answer::Refused { why: Refusal::Past },
        source::ApplicationOutcome::Exhausted => Answer::Refused { why: Refusal::Exhausted },
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use jig_client_domain::contract::Revision;
    use jig_ops_client_domain::infrastructure::{Answer, Ask, documents::Refusal};
    use skein_lib::Token;
    #[test]
    fn move_end_wire_keeps_scope_raw_names_whole_width_values_and_distinct_terminals() {
        let environment = values::Environment { pool: b"Q.pool".as_slice().into(), name: b"preview".as_slice().into() };
        let request = Ask::MoveEnd {
            object: Token::new(17),
            project: u32::MAX,
            environment: environment.clone(),
            revision: Revision { value: u64::MAX },
            until: u64::MAX,
        };
        let (project, value, digest) = ask(request.clone()).expect("values ask");
        assert_eq!(project, u32::MAX);
        assert_eq!(
            value,
            source::ApplicationRequest {
                environment: source::Environment { pool: environment.pool.clone(), name: environment.name.clone() },
                revision: u64::MAX,
                until: u64::MAX
            }
        );
        let (_, _, replay_digest) = ask(Ask::MoveEnd {
            object: Token::new(29),
            project,
            environment: environment.clone(),
            revision: Revision { value: u64::MAX },
            until: u64::MAX,
        })
        .expect("replayed semantic request");
        assert_ne!(digest, replay_digest, "the opaque payload includes the supplied view handle");
        for changed in [
            Ask::MoveEnd {
                object: Token::new(17),
                project: project - 1,
                environment: environment.clone(),
                revision: Revision { value: u64::MAX },
                until: u64::MAX,
            },
            Ask::MoveEnd {
                object: Token::new(17),
                project,
                environment: values::Environment { pool: b"P.pool".as_slice().into(), name: environment.name.clone() },
                revision: Revision { value: u64::MAX },
                until: u64::MAX,
            },
            Ask::MoveEnd {
                object: Token::new(17),
                project,
                environment: environment.clone(),
                revision: Revision { value: 0 },
                until: u64::MAX,
            },
            Ask::MoveEnd {
                object: Token::new(17),
                project,
                environment: environment.clone(),
                revision: Revision { value: u64::MAX },
                until: 0,
            },
        ] {
            assert_ne!(digest, ask(changed).expect("changed values ask").2);
        }
        let now = source::EnvironmentEnd { revision: u64::MAX, until: u64::MAX };
        assert_eq!(
            answer(source::ApplicationOutcome::MovedEnd { environment: value.environment.clone(), now }),
            Answer::MovedEnd {
                environment: environment.clone(),
                revision: Revision { value: u64::MAX },
                until: u64::MAX
            }
        );
        assert_eq!(
            answer(source::ApplicationOutcome::Moved { environment: value.environment, now }),
            Answer::Moved { environment, revision: Revision { value: u64::MAX }, until: u64::MAX }
        );
        assert_eq!(
            answer(source::ApplicationOutcome::Role { held: u32::MAX }),
            Answer::Refused { why: Refusal::Role { held: u32::MAX } }
        );
        assert_eq!(answer(source::ApplicationOutcome::Unknown), Answer::Gone);
        assert_eq!(answer(source::ApplicationOutcome::Past), Answer::Refused { why: Refusal::Past });
        assert_eq!(answer(source::ApplicationOutcome::Exhausted), Answer::Refused { why: Refusal::Exhausted });
    }
}
