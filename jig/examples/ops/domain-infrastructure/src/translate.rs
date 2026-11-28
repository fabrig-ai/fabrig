//! Translation between the contract and this connector's own types.
use alloc::boxed::Box;
use jig_domain_contract::{self as jig, effects, ops};
use skein_lib::List;

pub(crate) fn infrastructure_key(key: effects::EffectKey) -> crate::Key {
    let origin = match key.origin {
        effects::EffectPurpose::Call { attempt, completion, position } => {
            crate::Purpose::Call { attempt, completion, position }
        }
        effects::EffectPurpose::Procedure { purpose } => crate::Purpose::Procedure { purpose },
        effects::EffectPurpose::Projection { purpose } => crate::Purpose::Projection { purpose },
    };
    crate::Key { deployment: key.deployment, task: key.task, origin, purpose: key.purpose }
}

pub(crate) fn state(value: u64) -> [u8; 32] {
    let mut state = [0; 32];
    for (slot, byte) in state.get_mut(24..).expect("eight-byte state").iter_mut().zip(value.to_be_bytes()) {
        *slot = byte;
    }
    state
}

pub(crate) fn infrastructure_description(number: u16, value: crate::Description) -> effects::EffectDescription {
    let mut names = List::with_capacity(u32::try_from(value.resources.len()).expect("bounded resources"));
    for resource in value.resources {
        names.push(effects::Name { segments: resource.segments() }).expect("one resource name");
    }
    let mut additional = List::with_capacity(names.len().saturating_sub(1));
    for name in names.as_slice().get(1..).unwrap_or_default() {
        additional
            .push(effects::EffectResource { name: name.clone(), access: effects::EffectAccess::Owned })
            .expect("additional names");
    }
    effects::EffectDescription {
        connector: number,
        purpose: value.state,
        effect: effects::Effect {
            connector: number,
            kind: value.kind,
            name: names.as_slice().first().expect("described resource").clone(),
            state: state(value.state),
            price: value.price,
            access: effects::EffectAccess::Owned,
            additional: additional.into_boxed(),
            guards: Box::new([]),
        },
    }
}

pub(crate) const fn infrastructure_outcome(value: crate::Outcome) -> effects::OutboxOutcome {
    match value {
        crate::Outcome::Made => effects::OutboxOutcome::Made,
        crate::Outcome::Failed => effects::OutboxOutcome::Failed,
        crate::Outcome::Skipped => effects::OutboxOutcome::NotMade,
        crate::Outcome::Uncertain => effects::OutboxOutcome::Uncertain,
    }
}

pub(crate) const fn resource_role(role: crate::ResourceRole) -> jig::resources::ResourceRole {
    match role {
        crate::ResourceRole::Owned => jig::resources::ResourceRole::Owned,
        crate::ResourceRole::Participant => jig::resources::ResourceRole::Participant,
        crate::ResourceRole::Context => jig::resources::ResourceRole::Context,
        crate::ResourceRole::Unavailable => jig::resources::ResourceRole::Unavailable,
    }
}

pub(crate) fn infrastructure_added(values: Box<[ops::Added]>) -> Box<[crate::Added]> {
    let mut added = List::with_capacity(u32::try_from(values.len()).expect("bounded dependencies"));
    for value in values {
        let by = match value.by {
            ops::Party::Person(person) => crate::Party::Person(person),
            ops::Party::Task(task) => crate::Party::Task(task),
            ops::Party::Deployment { project } => crate::Party::Deployment { project },
        };
        added.push(crate::Added { task: value.task, at: value.at, by }).expect("one row per dependency");
    }
    added.into_boxed()
}

pub(crate) fn infrastructure_spec(spec: &ops::Spec) -> crate::Spec {
    let mut parameters = List::with_capacity(u32::try_from(spec.parameters.len()).expect("spec parameter bound"));
    for parameter in &spec.parameters {
        parameters
            .push(match parameter {
                ops::Parameter::Bytes { name, value } => crate::Parameter::Bytes { name: *name, value: value.clone() },
                ops::Parameter::Number { name, value } => crate::Parameter::Number { name: *name, value: *value },
                ops::Parameter::Resource { .. } => crate::Parameter::Resource,
            })
            .expect("one parameter per carrier");
    }
    crate::Spec { parameters: parameters.into_boxed() }
}

pub(crate) const fn infrastructure_executor(executor: ops::Executor) -> crate::Executor {
    match executor {
        ops::Executor::Procedure { connector, code } => crate::Executor::Procedure { connector, code },
        ops::Executor::Agent { .. } | ops::Executor::Person(_) | ops::Executor::Job { .. } => crate::Executor::Other,
    }
}

pub(crate) fn holdings(number: u16, values: Option<Box<[crate::Holding]>>) -> Option<Box<[jig::resources::Holding]>> {
    let values = values?;
    let mut holdings = List::with_capacity(u32::try_from(values.len()).expect("holding count"));
    for value in values {
        holdings
            .push(match value {
                crate::Holding::Write { resource, kind } => jig::resources::Holding::Write {
                    resource: jig::resources::Name { connector: number, path: resource.segments() },
                    kind,
                },
                crate::Holding::Slot { pool, kind } => jig::resources::Holding::Slot {
                    pool: jig::resources::Name { connector: number, path: crate::Resource::Pool(pool).segments() },
                    kind,
                },
            })
            .expect("one holding per value");
    }
    Some(holdings.into_boxed())
}

pub(crate) const fn admission(answer: effects::Answer) -> crate::Admission {
    match answer {
        effects::Answer::Allow => crate::Admission::Allow,
        effects::Answer::Wait => crate::Admission::Wait,
        effects::Answer::Propose => crate::Admission::Propose,
        effects::Answer::Refuse => crate::Admission::Refuse,
    }
}
