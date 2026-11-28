//! Translation between the contract and this connector's own types.
use alloc::boxed::Box;
use jig_domain_contract::{self as jig, effects, ops};
use skein_lib::{List, Wall};

pub(crate) fn observability_key(key: effects::EffectKey) -> crate::Key {
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

pub(crate) fn observability_description(
    number: u16,
    value: crate::Effect,
    vocabulary: &crate::Vocabulary,
) -> effects::EffectDescription {
    effects::EffectDescription {
        connector: number,
        purpose: value.until,
        effect: effects::Effect {
            connector: number,
            kind: vocabulary.silence_effect,
            name: effects::Name { segments: crate::Resource::AlertRule(value.rule).segments() },
            state: state(value.until),
            price: None,
            access: effects::EffectAccess::Owned,
            additional: Box::new([]),
            guards: Box::new([]),
        },
    }
}

pub(crate) const fn observability_outcome(value: crate::Outcome) -> effects::OutboxOutcome {
    match value {
        crate::Outcome::Made => effects::OutboxOutcome::Made,
        crate::Outcome::Failed => effects::OutboxOutcome::Failed,
        crate::Outcome::Uncertain => effects::OutboxOutcome::Uncertain,
    }
}

pub(crate) const fn verdict(value: crate::Verdict, wall: Wall) -> (effects::Verdict, Wall) {
    match value {
        crate::Verdict::Met { observed } => {
            (effects::Verdict::Met, Wall::from_nanos(observed.saturating_mul(1_000_000_000)))
        }
        crate::Verdict::Wait => (effects::Verdict::Wait, wall),
        crate::Verdict::Refuse => (effects::Verdict::Refuse, wall),
    }
}

pub(crate) fn service(names: &[effects::Name]) -> Option<crate::Service> {
    let segments = &names.first()?.segments;
    if segments.len() == 4 && segments.first()?.as_ref() == b"env" && segments.get(2)?.as_ref() == b"service" {
        return Some(crate::Service::new(segments.get(1)?.clone(), segments.get(3)?.clone()));
    }
    if segments.len() == 3 && segments.first()?.as_ref() == b"services" {
        Some(crate::Service::new(segments.get(1)?.clone(), segments.get(2)?.clone()))
    } else {
        None
    }
}

pub(crate) const fn requirement(number: u16, policy: &crate::Requirements) -> Option<crate::Requirement> {
    if number == policy.healthy_replica_requirement {
        Some(crate::Requirement::HealthyReplicaElsewhere)
    } else if number == policy.low_load_requirement {
        Some(crate::Requirement::LoadBelow {
            percent: policy.scale_down_load_percent,
            for_seconds: policy.scale_down_load_seconds,
        })
    } else {
        None
    }
}

pub(crate) const fn news_class(value: crate::Class) -> jig::topics::NewsClass {
    match value {
        crate::Class::Wake => jig::topics::NewsClass::Wakes,
        crate::Class::Keep => jig::topics::NewsClass::Kept,
    }
}

pub(crate) fn observability_spec(spec: &ops::Spec) -> crate::Spec {
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

pub(crate) fn watch_inputs(inbox: Box<[ops::Word]>) -> Box<[crate::WatchInput]> {
    let mut inputs = List::with_capacity(u32::try_from(inbox.len()).expect("offered inbox bound"));
    for word in inbox {
        let input = match word.kind {
            ops::MessageKind::News { .. } => Some(crate::WatchInput::News),
            ops::MessageKind::Result(result) => {
                let child = match word.from {
                    ops::Party::Task(child) => Some(child),
                    ops::Party::Person(_) | ops::Party::Deployment { .. } => None,
                };
                let result = match result {
                    ops::ResultKind::Verdict { code, fields } => {
                        let mut values = List::with_capacity(u32::try_from(fields.len()).expect("verdict field bound"));
                        for field in fields {
                            values
                                .push(crate::ResultField { name: field.name, value: field.value })
                                .expect("one result field");
                        }
                        crate::ResultKind::Verdict { code, fields: values.into_boxed() }
                    }
                    ops::ResultKind::Approved { .. }
                    | ops::ResultKind::Job
                    | ops::ResultKind::NoChange
                    | ops::ResultKind::Report
                    | ops::ResultKind::Change { .. }
                    | ops::ResultKind::Failed
                    | ops::ResultKind::Cancelled => crate::ResultKind::Ended,
                };
                Some(crate::WatchInput::Result { child, result })
            }
            ops::MessageKind::Escalation { .. }
            | ops::MessageKind::Proposal { .. }
            | ops::MessageKind::PassedQuestion { .. }
            | ops::MessageKind::PassedAnswer { .. }
            | ops::MessageKind::SettingsDecision { .. }
            | ops::MessageKind::EffectNotMade { .. }
            | ops::MessageKind::Words
            | ops::MessageKind::Met { .. }
            | ops::MessageKind::FinishRequested { .. }
            | ops::MessageKind::Amendment { .. }
            | ops::MessageKind::Question
            | ops::MessageKind::Answer { .. }
            | ops::MessageKind::Notice { .. }
            | ops::MessageKind::ProposalDecision { .. }
            | ops::MessageKind::QuestionAnswered { .. }
            | ops::MessageKind::QuestionOwed
            | ops::MessageKind::QuestionEnded { .. }
            | ops::MessageKind::NothingAwaited
            | ops::MessageKind::Timer { .. }
            | ops::MessageKind::PersonStalled { .. } => None,
        };
        if let Some(input) = input {
            inputs.push(input).expect("one input per inbox word");
        }
    }
    inputs.into_boxed()
}
