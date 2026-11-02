//! Application vocabulary in small total functions. Copy the core/host pairs;
//! replace resource paths, effect kinds and tool declarations (domain/root.md, 6).
use alloc::boxed::Box;
use jig_domain_authority as authority;
use jig_domain_core as core;
use jig_domain_tasks as tasks;
use jig_ops_domain_infrastructure as infrastructure;
use jig_ops_domain_observability as observability;
use skein_lib::{List, Wall};

pub(crate) fn infrastructure_key(key: core::EffectKey) -> infrastructure::Key {
    let origin = match key.origin {
        core::EffectPurpose::Call { attempt, completion, position } => {
            infrastructure::Purpose::Call { attempt, completion, position }
        }
        core::EffectPurpose::Procedure { purpose } => infrastructure::Purpose::Procedure { purpose },
        core::EffectPurpose::Projection { purpose } => infrastructure::Purpose::Projection { purpose },
    };
    infrastructure::Key { deployment: key.deployment, task: key.task, origin, purpose: key.purpose }
}
pub(crate) fn observability_key(key: core::EffectKey) -> observability::Key {
    let origin = match key.origin {
        core::EffectPurpose::Call { attempt, completion, position } => {
            observability::Purpose::Call { attempt, completion, position }
        }
        core::EffectPurpose::Procedure { purpose } => observability::Purpose::Procedure { purpose },
        core::EffectPurpose::Projection { purpose } => observability::Purpose::Projection { purpose },
    };
    observability::Key { deployment: key.deployment, task: key.task, origin, purpose: key.purpose }
}
pub(crate) fn state(value: u64) -> [u8; 32] {
    let mut state = [0; 32];
    for (slot, byte) in state.get_mut(24..).expect("eight-byte state").iter_mut().zip(value.to_be_bytes()) {
        *slot = byte;
    }
    state
}
pub(crate) fn infrastructure_description(
    number: u16,
    value: infrastructure::Description,
    waits_for_dependencies: bool,
) -> core::effects::connector::EffectDescription {
    let mut names = List::with_capacity(u32::try_from(value.resources.len()).expect("bounded resources"));
    for resource in value.resources {
        names.push(authority::Name { segments: resource.segments() }).expect("one resource name");
    }
    let mut additional = List::with_capacity(names.len().saturating_sub(1));
    for name in names.as_slice().get(1..).unwrap_or_default() {
        additional
            .push(authority::EffectResource { name: name.clone(), access: authority::EffectAccess::Owned })
            .expect("additional names");
    }
    let form = match value.form {
        infrastructure::Form::Creation => core::effects::connector::EffectForm::Creation,
        infrastructure::Form::Transition => core::effects::connector::EffectForm::Transition,
        infrastructure::Form::Set => core::effects::connector::EffectForm::Set,
    };
    let recovery = match value.recovery {
        infrastructure::Recovery::Keyed => core::effects::connector::Recovery::Keyed,
        infrastructure::Recovery::Conditional => core::effects::connector::Recovery::Conditional,

        infrastructure::Recovery::Unrecoverable => core::effects::connector::Recovery::Unrecoverable,
    };
    core::effects::connector::EffectDescription {
        waits_for_dependencies,
        connector: number,
        purpose: value.state,
        form,
        recovery,
        effect: authority::Effect {
            connector: number,
            kind: value.kind,
            name: names.as_slice().first().expect("described resource").clone(),
            state: state(value.state),
            price: value.price,
            access: authority::EffectAccess::Owned,
            additional: additional.into_boxed(),
            guards: Box::new([]),
        },
    }
}
pub(crate) fn observability_description(
    number: u16,
    value: observability::Effect,
    waits_for_dependencies: bool,
    vocabulary: &observability::Vocabulary,
) -> core::effects::connector::EffectDescription {
    core::effects::connector::EffectDescription {
        waits_for_dependencies,
        connector: number,
        purpose: value.until,
        form: core::effects::connector::EffectForm::Set,
        recovery: core::effects::connector::Recovery::Keyed,
        effect: authority::Effect {
            connector: number,
            kind: vocabulary.silence_effect,
            name: authority::Name { segments: observability::Resource::AlertRule(value.rule).segments() },
            state: state(value.until),
            price: None,
            access: authority::EffectAccess::Owned,
            additional: Box::new([]),
            guards: Box::new([]),
        },
    }
}
pub(crate) const fn infrastructure_outcome(value: infrastructure::Outcome) -> core::effects::connector::OutboxOutcome {
    match value {
        infrastructure::Outcome::Made => core::effects::connector::OutboxOutcome::Made,
        infrastructure::Outcome::Failed => core::effects::connector::OutboxOutcome::Failed,
        infrastructure::Outcome::Skipped => core::effects::connector::OutboxOutcome::NotMade,
        infrastructure::Outcome::Uncertain => core::effects::connector::OutboxOutcome::Uncertain,
    }
}
pub(crate) const fn observability_outcome(value: observability::Outcome) -> core::effects::connector::OutboxOutcome {
    match value {
        observability::Outcome::Made => core::effects::connector::OutboxOutcome::Made,
        observability::Outcome::Failed => core::effects::connector::OutboxOutcome::Failed,
        observability::Outcome::Uncertain => core::effects::connector::OutboxOutcome::Uncertain,
    }
}
pub(crate) const fn verdict(value: observability::Verdict, wall: Wall) -> (authority::Verdict, Wall) {
    match value {
        observability::Verdict::Met { observed } => {
            (authority::Verdict::Met, Wall::from_nanos(observed.saturating_mul(1_000_000_000)))
        }
        observability::Verdict::Wait => (authority::Verdict::Wait, wall),
        observability::Verdict::Refuse => (authority::Verdict::Refuse, wall),
    }
}
pub(crate) fn service(names: &[authority::Name]) -> Option<observability::Service> {
    let segments = &names.first()?.segments;
    if segments.len() == 4 && segments.first()?.as_ref() == b"env" && segments.get(2)?.as_ref() == b"service" {
        return Some(observability::Service::new(segments.get(1)?.clone(), segments.get(3)?.clone()));
    }
    if segments.len() == 3 && segments.first()?.as_ref() == b"services" {
        Some(observability::Service::new(segments.get(1)?.clone(), segments.get(2)?.clone()))
    } else {
        None
    }
}

/// Ops names its two requirements in its connector vocabulary.
pub(crate) const fn requirement(number: u16, policy: &crate::Policy) -> Option<observability::Requirement> {
    if number == policy.healthy_replica_requirement {
        Some(observability::Requirement::HealthyReplicaElsewhere)
    } else if number == policy.low_load_requirement {
        Some(observability::Requirement::LoadBelow {
            percent: policy.scale_down_load_percent,
            for_seconds: policy.scale_down_load_seconds,
        })
    } else {
        None
    }
}

pub(crate) const fn news_class(value: observability::Class) -> tasks::NewsClass {
    match value {
        observability::Class::Wake => tasks::NewsClass::Wakes,
        observability::Class::Keep => tasks::NewsClass::Kept,
    }
}
pub(crate) fn alert_words(alert: &observability::Alert) -> Box<[u8]> {
    let bytes = b"alert "
        .len()
        .checked_add(skein_lib::Decimal::of(alert.number).as_bytes().len())
        .expect("alert number")
        .checked_add(b" severity ".len())
        .expect("severity heading")
        .checked_add(skein_lib::Decimal::of(u64::from(alert.severity)).as_bytes().len())
        .expect("severity");
    let mut writer = skein_lib::Writer::new(bytes);
    writer.put(b"alert ").expect("alert heading");
    writer.put(skein_lib::Decimal::of(alert.number).as_bytes()).expect("alert identity");
    writer.put(b" severity ").expect("severity heading");
    writer.put(skein_lib::Decimal::of(u64::from(alert.severity)).as_bytes()).expect("severity");
    writer.finish()
}
pub(crate) fn health_words(_service: &observability::Service, healthy: bool) -> Box<[u8]> {
    if healthy { Box::from(&b"healthy"[..]) } else { Box::from(&b"unhealthy"[..]) }
}
pub(crate) fn triage(
    delegate: &[u8],
    alerts: &[u64],
    limits: &jig_protocol_tools::Limits,
) -> Option<Box<[core::Delegate]>> {
    let mut batch = jig_protocol_tools::batch(delegate, limits).ok()?;
    if batch.len() != 1 {
        return None;
    }
    for member in &mut batch {
        let mut room = member.spec.words.len().checked_add(b"\nAlerts:".len())?;
        for alert in alerts {
            room = room.checked_add(1)?.checked_add(skein_lib::Decimal::of(*alert).as_bytes().len())?;
        }
        let mut words = skein_lib::Writer::new(room);
        words.put(&member.spec.words).ok()?;
        words.put(b"\nAlerts:").ok()?;
        for alert in alerts {
            words.put(b" ").ok()?;
            words.put(skein_lib::Decimal::of(*alert).as_bytes()).ok()?;
        }
        member.spec.words = words.finish();
    }
    Some(batch)
}

/// Read kinds and resources belong to the application's connector vocabulary.
pub(crate) fn read_description(
    number: u16,
    read: &observability::Read,
    vocabulary: &observability::Vocabulary,
) -> authority::Effect {
    let (kind, resource) = match read {
        observability::Read::Logs { service, .. } => {
            (vocabulary.logs_read, observability::Resource::Logs(service.clone()))
        }
        observability::Read::Series { service, .. } => {
            (vocabulary.series_read, observability::Resource::Metrics(service.clone()))
        }
        observability::Read::Fired { service, .. } => {
            (vocabulary.fired_read, observability::Resource::Service(service.clone()))
        }
    };
    authority::Effect {
        connector: number,
        kind,
        name: authority::Name { segments: resource.segments() },
        state: [0; 32],
        price: None,
        access: authority::EffectAccess::Context,
        additional: Box::new([]),
        guards: Box::new([]),
    }
}

/// The connector owns adoption roles; the root carries them into core admission.
pub(crate) const fn resource_role(role: infrastructure::ResourceRole) -> core::effects::connector::ResourceRole {
    match role {
        infrastructure::ResourceRole::Owned => core::effects::connector::ResourceRole::Owned,
        infrastructure::ResourceRole::Participant => core::effects::connector::ResourceRole::Participant,
        infrastructure::ResourceRole::Context => core::effects::connector::ResourceRole::Context,
        infrastructure::ResourceRole::Unavailable => core::effects::connector::ResourceRole::Unavailable,
    }
}

/// The owner templates always request one delegate, matching the outcome's one child.
pub(crate) fn lead(delegate: &[u8], limits: &jig_protocol_tools::Limits) -> Option<Box<[core::Delegate]>> {
    let batch = jig_protocol_tools::batch(delegate, limits).ok()?;
    if batch.len() != 1 {
        return None;
    }
    Some(batch)
}

pub(crate) fn infrastructure_added(values: Box<[tasks::Added]>) -> Box<[infrastructure::Added]> {
    let mut added = List::with_capacity(u32::try_from(values.len()).expect("bounded dependencies"));
    for value in values {
        let by = match value.by {
            tasks::Party::Person(person) => infrastructure::Party::Person(person),
            tasks::Party::Task(task) => infrastructure::Party::Task(task),
            tasks::Party::Deployment { project } => infrastructure::Party::Deployment { project },
        };
        added.push(infrastructure::Added { task: value.task, at: value.at, by }).expect("one row per dependency");
    }
    added.into_boxed()
}

pub(crate) fn infrastructure_spec(spec: &tasks::Spec) -> infrastructure::Spec {
    let mut parameters = List::with_capacity(u32::try_from(spec.parameters.len()).expect("spec parameter bound"));
    for parameter in &spec.parameters {
        parameters
            .push(match parameter {
                tasks::Parameter::Bytes { name, value } => {
                    infrastructure::Parameter::Bytes { name: *name, value: value.clone() }
                }
                tasks::Parameter::Number { name, value } => {
                    infrastructure::Parameter::Number { name: *name, value: *value }
                }
                tasks::Parameter::Resource { .. } => infrastructure::Parameter::Resource,
            })
            .expect("one parameter per carrier");
    }
    infrastructure::Spec { parameters: parameters.into_boxed() }
}
pub(crate) fn observability_spec(spec: &tasks::Spec) -> observability::Spec {
    let mut parameters = List::with_capacity(u32::try_from(spec.parameters.len()).expect("spec parameter bound"));
    for parameter in &spec.parameters {
        parameters
            .push(match parameter {
                tasks::Parameter::Bytes { name, value } => {
                    observability::Parameter::Bytes { name: *name, value: value.clone() }
                }
                tasks::Parameter::Number { name, value } => {
                    observability::Parameter::Number { name: *name, value: *value }
                }
                tasks::Parameter::Resource { .. } => observability::Parameter::Resource,
            })
            .expect("one parameter per carrier");
    }
    observability::Spec { parameters: parameters.into_boxed() }
}
pub(crate) const fn infrastructure_executor(executor: tasks::Executor) -> infrastructure::Executor {
    match executor {
        tasks::Executor::Procedure { connector, code } => infrastructure::Executor::Procedure { connector, code },
        tasks::Executor::Agent { .. } | tasks::Executor::Person(_) => infrastructure::Executor::Other,
    }
}
pub(crate) fn holdings(number: u16, values: Option<Box<[infrastructure::Holding]>>) -> Option<Box<[tasks::Holding]>> {
    let values = values?;
    let mut holdings = List::with_capacity(u32::try_from(values.len()).expect("holding count"));
    for value in values {
        holdings
            .push(match value {
                infrastructure::Holding::Write { resource, kind } => tasks::Holding::Write {
                    resource: tasks::Name { connector: number, path: resource.segments() },
                    kind,
                },
                infrastructure::Holding::Slot { pool, kind } => tasks::Holding::Slot {
                    pool: tasks::Name { connector: number, path: infrastructure::Resource::Pool(pool).segments() },
                    kind,
                },
            })
            .expect("one holding per value");
    }
    Some(holdings.into_boxed())
}
pub(crate) fn hold_kinds(number: u16, vocabulary: &infrastructure::Vocabulary) -> Box<[tasks::Kind]> {
    let values = infrastructure::hold_kinds(vocabulary);
    let mut kinds = List::with_capacity(u32::try_from(values.len()).expect("hold kinds"));
    for value in values {
        let hold = match value.hold {
            infrastructure::Hold::None => tasks::HoldKind::Shared,
            infrastructure::Hold::ExclusiveWait => tasks::HoldKind::Exclusive { taken: tasks::Taken::Waits },
            infrastructure::Hold::PooledWait => tasks::HoldKind::Pooled { taken: tasks::Taken::Waits },
        };
        kinds.push(tasks::Kind { connector: number, kind: value.kind, hold }).expect("one kind per value");
    }
    kinds.into_boxed()
}
pub(crate) const fn admission(answer: authority::Answer) -> infrastructure::Admission {
    match answer {
        authority::Answer::Allow => infrastructure::Admission::Allow,
        authority::Answer::Wait => infrastructure::Admission::Wait,
        authority::Answer::Propose => infrastructure::Admission::Propose,
        authority::Answer::Refuse => infrastructure::Admission::Refuse,
    }
}
pub(crate) fn watch_inputs(inbox: Box<[tasks::Word]>) -> Box<[observability::WatchInput]> {
    let mut inputs = List::with_capacity(u32::try_from(inbox.len()).expect("offered inbox bound"));
    for word in inbox {
        let input = match word.kind.procedure_input() {
            Some(tasks::ProcedureInput::News) => Some(observability::WatchInput::News),
            Some(tasks::ProcedureInput::Result(result)) => {
                let child = match word.from {
                    tasks::Party::Task(child) => Some(child),
                    tasks::Party::Person(_) | tasks::Party::Deployment { .. } => None,
                };
                let result = match result {
                    tasks::ResultKind::Verdict { code, fields } => {
                        let mut values = List::with_capacity(u32::try_from(fields.len()).expect("verdict field bound"));
                        for field in fields {
                            values
                                .push(observability::ResultField { name: field.name, value: field.value })
                                .expect("one result field");
                        }
                        observability::ResultKind::Verdict { code, fields: values.into_boxed() }
                    }
                    tasks::ResultKind::Approved { .. }
                    | tasks::ResultKind::NoChange
                    | tasks::ResultKind::Report
                    | tasks::ResultKind::Change { .. }
                    | tasks::ResultKind::Failed
                    | tasks::ResultKind::Cancelled => observability::ResultKind::Ended,
                };
                Some(observability::WatchInput::Result { child, result })
            }
            None => None,
        };
        if let Some(input) = input {
            inputs.push(input).expect("one input per inbox word");
        }
    }
    inputs.into_boxed()
}
