//! Borrowed erasure-row ownership for bounded core reads (domain/engine.md, 11).
use crate::{CallPart, CallRecord, RunProof, settings};
use alloc::boxed::Box;
use core::mem::size_of_val;
use jig_domain_authority as authority;

fn bytes(value: usize) -> Option<u64> {
    u64::try_from(value).ok()
}

pub(super) fn proof(value: &RunProof) -> Option<u64> {
    let mut total = bytes(size_of_val(&*value.credentials))?
        .checked_add(bytes(size_of_val(&*value.accounts))?)?
        .checked_add(bytes(size_of_val(&*value.outstanding))?)?;
    if let Some(terminal) = &value.terminal {
        total = total.checked_add(jig_domain_tasks::terminal_bytes(&terminal.end)?)?;
    }
    Some(total)
}

pub(super) fn call(value: &CallRecord) -> Option<u64> {
    let mut total = match &value.part {
        CallPart::EffectDenied { findings, .. }
        | CallPart::ToolDenied { findings, .. }
        | CallPart::DelegationDenied { findings, .. } => bytes(size_of_val(&**findings))?,
        CallPart::Delegated(rows) => bytes(size_of_val(&**rows))?,
        CallPart::NoteRecalled { entries, .. } => recalled(entries)?,
        CallPart::SettingsRefused(why) => refusal(why)?,
        CallPart::EffectRefused(problem)
        | CallPart::EscalationRefused(problem)
        | CallPart::ProposalRefused(problem)
        | CallPart::ControlRefused(problem)
        | CallPart::MessageRefused(problem)
        | CallPart::SubscriptionRefused(problem)
        | CallPart::DelegationRefused(problem) => match &problem.blocked_by {
            Some(tasks) => bytes(size_of_val(&**tasks))?,
            None => 0,
        },
        CallPart::HoldOwner { .. }
        | CallPart::Again { .. }
        | CallPart::PassedUp { .. }
        | CallPart::Revised { .. }
        | CallPart::Moved { .. }
        | CallPart::Effect { .. }
        | CallPart::Connector { .. }
        | CallPart::EscalationDecided { .. }
        | CallPart::Proposed { .. }
        | CallPart::ProposalDecided { .. }
        | CallPart::Controlled
        | CallPart::ControlDenied { .. }
        | CallPart::Sent { .. }
        | CallPart::Introduced
        | CallPart::Subscribed { .. }
        | CallPart::Unsubscribed
        | CallPart::NoteWritten { .. }
        | CallPart::NoteRefused(_)
        | CallPart::Unavailable => 0,
    };
    if let Some(settled) = &value.settled {
        total = total.checked_add(settled.owned_bytes()?)?;
    }
    Some(total)
}

fn recalled(entries: &[jig_domain_notes::Entry]) -> Option<u64> {
    let mut total = bytes(size_of_val(entries))?;
    for entry in entries {
        total = total
            .checked_add(bytes(entry.description.len())?)?
            .checked_add(bytes(entry.body.len())?)?
            .checked_add(skein_lib::List::<u64>::price(entry.references.capacity())?)?;
        match &entry.scope {
            jig_domain_notes::Scope::Resources { pattern, .. } => {
                total = total.checked_add(bytes(size_of_val(&*pattern.segments))?)?;
                for segment in &pattern.segments {
                    total = total.checked_add(bytes(segment.len())?)?;
                }
                total = total.checked_add(match &pattern.last {
                    jig_domain_notes::Last::Exact(last) | jig_domain_notes::Last::Open(last) => bytes(last.len())?,
                })?;
            }
            jig_domain_notes::Scope::Deployment
            | jig_domain_notes::Scope::Project { .. }
            | jig_domain_notes::Scope::Goal { .. } => {}
        }
    }
    Some(total)
}

fn refusal(why: &jig_domain_people::Refusal) -> Option<u64> {
    Some(match why {
        jig_domain_people::Refusal::Full { tasks, .. } | jig_domain_people::Refusal::Circle { tasks, .. } => {
            bytes(size_of_val(&**tasks))?
        }
        jig_domain_people::Refusal::Kept { pending } => bytes(size_of_val(&**pending))?,
        jig_domain_people::Refusal::SignInRedirect { .. }
        | jig_domain_people::Refusal::Stale { .. }
        | jig_domain_people::Refusal::NameTaken
        | jig_domain_people::Refusal::AccountMove { .. }
        | jig_domain_people::Refusal::Dependency { .. }
        | jig_domain_people::Refusal::Account { .. }
        | jig_domain_people::Refusal::NotWellFormed { .. }
        | jig_domain_people::Refusal::Role { .. }
        | jig_domain_people::Refusal::Authority { .. }
        | jig_domain_people::Refusal::Limit { .. }
        | jig_domain_people::Refusal::Moved { .. }
        | jig_domain_people::Refusal::Ended { .. }
        | jig_domain_people::Refusal::Taken { .. }
        | jig_domain_people::Refusal::NotWaiting
        | jig_domain_people::Refusal::NoFurther
        | jig_domain_people::Refusal::Unknown
        | jig_domain_people::Refusal::NotOffered
        | jig_domain_people::Refusal::Provider
        | jig_domain_people::Refusal::KeyConflict
        | jig_domain_people::Refusal::NoEscalationHolder
        | jig_domain_people::Refusal::Llm { .. } => 0,
    })
}

pub(super) fn proposal(value: &settings::Proposal) -> Option<u64> {
    let mut total = bytes(size_of_val(&*value.items))?;
    for item in &value.items {
        match &item.edit {
            settings::Edit::Set(value) => total = total.checked_add(setting(value)?)?,
            settings::Edit::Removed | settings::Edit::Reset => {}
        }
    }
    Some(total)
}

fn setting(value: &settings::Value) -> Option<u64> {
    match value {
        settings::Value::Charter(value) => {
            bytes(size_of_val(&**value))?.checked_add(crate::memory::policy_owned_bytes(&value.policy)?)
        }
        settings::Value::Template(value) => {
            let contract = match &value.contract {
                jig_domain_tasks::Contract::Approval { state, part } => {
                    let mut total = bytes(state.len())?;
                    match &part.key {
                        jig_domain_tasks::PartKey::Path(path) => {
                            total = total.checked_add(skein_lib::List::<Box<[u8]>>::price(path.capacity())?)?;
                            for segment in path {
                                total = total.checked_add(bytes(segment.len())?)?;
                            }
                        }
                        jig_domain_tasks::PartKey::Number(_) => {}
                    }
                    total
                }
                jig_domain_tasks::Contract::Verdict { choices } => {
                    let mut total = bytes(size_of_val(&**choices))?;
                    for choice in choices {
                        total = total.checked_add(bytes(size_of_val(&*choice.fields))?)?;
                        for field in &choice.fields {
                            total = total.checked_add(bytes(field.name.len())?)?;
                        }
                    }
                    total
                }
                jig_domain_tasks::Contract::Report { .. } | jig_domain_tasks::Contract::Change { .. } => 0,
            };
            bytes(size_of_val(&**value))?.checked_add(bytes(value.name.len())?)?.checked_add(contract)?.checked_add(
                match &value.llm {
                    Some(llm) => bytes(size_of_val(&*llm.subagents))?,
                    None => 0,
                },
            )
        }
        settings::Value::Policy(value) => {
            let mut total = bytes(size_of_val(&**value))?
                .checked_add(bytes(size_of_val(&*value.permissions))?)?
                .checked_add(bytes(size_of_val(&*value.templates))?)?
                .checked_add(bytes(size_of_val(&*value.policy.roles))?)?
                .checked_add(authority(&value.policy.ceiling)?)?
                .checked_add(grants(&value.policy.projections)?)?
                .checked_add(requirements(&value.policy.requirements)?)?;
            for role in &value.policy.roles {
                total = total.checked_add(authority(&role.authority)?)?;
            }
            Some(total)
        }
        settings::Value::Rules(value) => bytes(size_of_val(&**value))?
            .checked_add(authority(&value.ceiling)?)?
            .checked_add(bytes(size_of_val(value.implies.entries()))?)?
            .checked_add(requirements(&value.requirements)?),
    }
}

fn authority(value: &authority::Authority) -> Option<u64> {
    let mut total = grants(&value.grants)?
        .checked_add(bytes(size_of_val(&*value.note_resources))?)?
        .checked_add(bytes(size_of_val(&*value.delegation.kinds))?)?;
    for scope in &value.note_resources {
        total = total.checked_add(pattern(&scope.pattern)?)?;
    }
    Some(total)
}

fn grants(values: &[authority::Grant]) -> Option<u64> {
    let mut total = bytes(size_of_val(values))?;
    for value in values {
        total = total.checked_add(pattern(&value.pattern)?)?;
    }
    Some(total)
}

fn requirements(values: &[authority::Requirement]) -> Option<u64> {
    let mut total = bytes(size_of_val(values))?;
    for value in values {
        total = total.checked_add(pattern(&value.pattern)?)?;
    }
    Some(total)
}

fn pattern(value: &authority::Pattern) -> Option<u64> {
    let mut total = bytes(size_of_val(&*value.segments))?;
    for segment in &value.segments {
        total = total.checked_add(bytes(segment.len())?)?;
    }
    total.checked_add(match &value.last {
        authority::Last::Exact(last) | authority::Last::Open(last) => bytes(last.len())?,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CallKey, SettledAnswer, SettledCall};

    #[test]
    fn erasure_call_bytes_include_refusal_blockers_and_settled_payloads() {
        let call = CallRecord {
            key: CallKey { task: 1, attempt: 1, completion: 1, position: 1 },
            part: CallPart::ControlRefused(jig_domain_tasks::Problem {
                task: Some(1),
                why: jig_domain_tasks::Refusal::Unknown,
                blocked_by: Some(Box::new([2, 3])),
                capacity: None,
            }),
            settled: Some(SettledCall {
                serial: 1,
                tool: Box::new(*b"tool"),
                answer: SettledAnswer::Host { error: false, body: Box::new(*b"answer") },
            }),
        };
        assert_eq!(super::call(&call), Some(16 + 4 + 6));
    }

    #[test]
    fn erasure_settings_bytes_include_delegation_executors() {
        let (core, _) = crate::tests::core();
        let mut value = core.authority.policy(1).unwrap().ceiling.clone();
        value.delegation.kinds = Box::new([]);
        let empty = authority(&value).unwrap();
        value.delegation.kinds = Box::new([authority::Executor::Charter(1), authority::Executor::Procedure(2)]);
        assert_eq!(authority(&value), empty.checked_add(bytes(size_of_val(&*value.delegation.kinds)).unwrap()));
    }

    #[test]
    fn erasure_settings_bytes_include_each_configured_value() {
        let (core, _) = crate::tests::core();
        let charter = core.current.defaults.get(&settings::Setting::Charter(1)).unwrap().value.clone();
        let rules = core.current.defaults.get(&settings::Setting::Rules).unwrap().value.clone();
        let policy = settings::Value::Policy(Box::new(crate::ConfiguredPolicy {
            policy: core.authority.policy(1).unwrap().clone(),
            permissions: Box::new([]),
            templates: Box::new([]),
        }));
        let template = settings::Value::Template(Box::new(crate::Template {
            number: 1,
            name: Box::new(*b"template"),
            executor: jig_domain_people::TemplateExecutor::Chosen,
            contract: jig_domain_tasks::Contract::Verdict {
                choices: Box::new([jig_domain_tasks::Verdict {
                    code: 1,
                    words: 32,
                    followups: 0,
                    fields: Box::new([jig_domain_tasks::FieldRule { name: Box::new(*b"field"), max: 32 }]),
                }]),
            },
            shallower: 0,
            budget: jig_domain_people::TemplateBudget::Given,
            deadline: jig_domain_people::TemplateDeadline::None,
            tracked: None,
            flags: jig_domain_tasks::Flags::PLAIN,
            label: 0,
            grace: None,
            llm: None,
        }));
        for (setting, value) in [
            (settings::Setting::Charter(1), charter),
            (settings::Setting::Rules, rules),
            (settings::Setting::Policy(1), policy),
            (settings::Setting::Template(1), template),
        ] {
            let value = settings::Proposal {
                against: 1,
                items: Box::new([settings::Item { setting, edit: settings::Edit::Set(value) }]),
            };
            assert!(proposal(&value).expect("bounded settings ownership") > bytes(size_of_val(&*value.items)).unwrap());
        }
    }
}
