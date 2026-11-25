//! General operation targets and project template overrides on the wire.
use super::asks;
use crate::generated::client;
use alloc::boxed::Box;
use jig_client_domain::contract as c;
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
use skein_lib::{List, bytes};

pub(super) fn answer(value: &c::AnswerOn) -> w::AnswerOn {
    match value {
        c::AnswerOn::Question { task, message, words } => w::AnswerOn::Question(w::AnswerQuestion {
            task: *task,
            message: *message,
            words: bytes::copy_of(words.bytes()),
        }),
        c::AnswerOn::Person { task, revision, answer } => w::AnswerOn::Person(w::AnswerPerson {
            task: *task,
            revision: asks::wire_revision(*revision),
            answer: asks::wire_person_answer(answer),
        }),
    }
}
pub(super) fn decide(value: &c::DecideOn) -> w::DecideOn {
    client::decide_on_to_record(value)
}
pub(super) fn templates(values: &List<c::TemplateOverride>) -> w::PolicyEdit {
    let mut overrides = List::with_capacity(values.len());
    for v in values {
        overrides.push(client::template_override_to_record(v)).expect("same capacity");
    }
    w::PolicyEdit::Templates(w::PolicyEditTemplates { overrides })
}

pub(super) fn restore_templates(values: &List<w::TemplateOverride>) -> c::PolicyEdit {
    c::PolicyEdit::Templates { overrides: restore_template_overrides(values) }
}
pub(super) fn restore_template_overrides(values: &List<w::TemplateOverride>) -> List<c::TemplateOverride> {
    let mut overrides = List::with_capacity(values.len());
    for v in values {
        overrides.push(client::template_override_to_domain(v.clone())).expect("same capacity");
    }
    overrides
}

pub(super) fn tracking(value: c::Tracking) -> w::Tracking {
    client::tracking_to_record(&value)
}
pub(super) fn restore_tracking(value: &w::Tracking) -> c::Tracking {
    client::tracking_to_domain(value.clone())
}

pub(super) fn create(project: u32, tasks: &List<c::NewTask>, limits: &w::Limits) -> Result<w::Ask, w::Problem> {
    Ok(w::Ask::Create(w::Create { project, tasks: batch(tasks, limits.create_tasks, limits)? }))
}
pub(super) fn delegate(
    project: u32,
    task: u64,
    tasks: &List<c::NewTask>,
    limits: &w::Limits,
) -> Result<w::Ask, w::Problem> {
    Ok(w::Ask::Delegate(w::Delegate { project, task, tasks: batch(tasks, limits.delegate_tasks, limits)? }))
}
fn batch(tasks: &List<c::NewTask>, bound: u32, limits: &w::Limits) -> Result<Box<[u8]>, w::Problem> {
    let mut records = List::with_capacity(tasks.len());
    let mut size = 0_u32;
    for task in tasks {
        let record = new_task(task, limits)?;
        size = size
            .checked_add(record.measure(limits)?)
            .ok_or(super::too_large())?
            .checked_add(4)
            .ok_or(super::too_large())?;
        if size > bound {
            return Err(super::too_large());
        }
        records.push(record).expect("same capacity");
    }
    let mut out = skein_lib::Writer::new(skein_lib::length::usize_of(size));
    for record in records.into_boxed() {
        out.put(&record.measure(limits)?.to_be_bytes()).expect("measured piece");
        record.write(limits, &mut out).expect("measured task");
    }
    Ok(out.finish())
}
fn new_task(v: &c::NewTask, limits: &w::Limits) -> Result<w::NewTask, w::Problem> {
    let mut dependencies = List::with_capacity(v.dependencies.len());
    for item in &v.dependencies {
        dependencies.push(client::task_ref_to_record(item)).expect("same capacity");
    }
    let mut named = List::with_capacity(v.named.len());
    for grant in &v.named {
        named.push(asks::wire_owner_grant(grant)).expect("same capacity");
    }
    let executor = client::executor_to_record(&v.executor);
    Ok(w::NewTask {
        symbols: super::documents::me::wire_symbols(&v.symbols),
        executor,
        spec: asks::wire_specification(&v.spec),
        contract: client::new_contract_to_record(&v.contract),
        authority: asks::wire_owner_authority(&v.authority, limits)?,
        deadline: client::new_deadline_to_record(&v.deadline),
        budget: client::new_budget_to_record(&v.budget),
        wake: match &v.wake {
            Some(wake) => Some(asks::wire_owner_wake_policy(wake)),
            None => None,
        },
        dependencies,
        named: asks::wire_owner_grant_piece(&named, limits)?,
        tracked: v.tracked,
        label: v.label,
        flags: client::task_flags_to_record(&v.flags),
        llm: super::llm::wire_choice(v.llm.as_ref()),
    })
}
pub(super) fn restore_create(value: &w::Create, limits: &crate::Limits) -> Result<c::Ask, w::Problem> {
    Ok(c::Ask::Create { project: value.project, tasks: restore_batch(&value.tasks, limits)? })
}
fn restore_batch(bytes: &[u8], limits: &crate::Limits) -> Result<List<c::NewTask>, w::Problem> {
    let mut reader = skein_lib::Reader::new(bytes);
    let mut tasks = List::with_capacity(limits.client().rows);
    while let Some(bytes) = super::piece(&mut reader)? {
        if tasks.len() == tasks.capacity() {
            return Err(super::too_large());
        }
        let v = w::NewTask::decode(limits.receiving(), &mut skein_lib::Reader::new(&bytes))?;
        tasks.push(restore_task(&v, limits)?).expect("checked capacity");
    }
    Ok(tasks)
}
fn restore_task(v: &w::NewTask, limits: &crate::Limits) -> Result<c::NewTask, w::Problem> {
    let mut dependencies = List::with_capacity(v.dependencies.len());
    for item in &v.dependencies {
        dependencies.push(client::task_ref_to_domain(item.clone())).expect("same capacity");
    }
    match &v.contract {
        w::NewContract::Verdict(verdict) => {
            for choice in &verdict.choices {
                super::documents::check_verdict_choice(choice, limits)?;
            }
        }
        w::NewContract::Report(_) | w::NewContract::Change(_) => {}
    }
    Ok(c::NewTask {
        symbols: super::documents::me::symbols(&v.symbols),
        deadline: client::new_deadline_to_domain(v.deadline.clone()),
        executor: super::facts::executor(&v.executor),
        spec: super::pages::specification_fields(&v.spec, limits)?,
        contract: client::new_contract_to_domain(v.contract.clone()),
        authority: super::documents::authority_fields(&v.authority, limits)?,
        budget: client::new_budget_to_domain(v.budget.clone()),
        wake: restore_wake(v.wake.as_ref()),
        dependencies,
        named: super::documents::grant_piece(&v.named, limits, limits.client().authority)?,
        tracked: v.tracked,
        label: v.label,
        flags: client::task_flags_to_domain(v.flags.clone()),
        llm: super::llm::choice(v.llm.as_ref(), limits)?,
    })
}

pub(super) fn propose(
    project: u32,
    action: &c::ProposeAction,
    reason: &c::Text,
    limits: &w::Limits,
) -> Result<w::Ask, w::Problem> {
    let action = match action {
        c::ProposeAction::Settings(value) => {
            w::ProposeAction::Settings(super::settings_proposals::wire_propose_settings(value, limits)?)
        }
        c::ProposeAction::Batch { tasks } => {
            w::ProposeAction::Batch(w::ProposeBatch { tasks: batch(tasks, limits.propose_batch_tasks, limits)? })
        }
        c::ProposeAction::Amendment { task, revision, change } => {
            w::ProposeAction::Amendment(Box::new(w::ProposeAmendment {
                task: *task,
                revision: asks::wire_revision(*revision),
                change: asks::wire_amendment(change, limits)?,
            }))
        }
    };
    Ok(w::Ask::Propose(w::Propose { project, action, reason: bytes::copy_of(reason.bytes()) }))
}
pub(super) fn restore_propose(value: &w::Propose, limits: &crate::Limits) -> Result<c::Ask, w::Problem> {
    let action = match &value.action {
        w::ProposeAction::Settings(value) => {
            c::ProposeAction::Settings(super::settings_proposals::restore_propose_settings(value, limits)?)
        }
        w::ProposeAction::Batch(value) => c::ProposeAction::Batch { tasks: restore_batch(&value.tasks, limits)? },
        w::ProposeAction::Amendment(value) => c::ProposeAction::Amendment {
            task: value.task,
            revision: super::answers::revision(&value.revision),
            change: Box::new(super::pages::amendment_fields(&value.change, limits)?),
        },
    };
    let reason = super::store::text(&value.reason, limits)?;
    Ok(c::Ask::Propose { project: value.project, action, reason })
}

fn restore_wake(value: Option<&w::WakePolicy>) -> Option<c::WakePolicy> {
    let wake = value?;
    Some(super::facts::wake_policy(wake))
}

pub(super) fn restore_delegate(value: &w::Delegate, limits: &crate::Limits) -> Result<c::Ask, w::Problem> {
    Ok(c::Ask::Delegate { project: value.project, task: value.task, tasks: restore_batch(&value.tasks, limits)? })
}
