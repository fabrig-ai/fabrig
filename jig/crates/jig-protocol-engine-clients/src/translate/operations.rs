//! General operation targets and template overrides, translated into people records.
use super::{Untranslatable, asks};
use crate::generated::client as mapped;
use alloc::boxed::Box;
use jig_client_protocol_wire as w;
use jig_client_protocol_wire::Record as _;
use jig_domain_people as p;
use skein_lib::List;

pub(super) fn answer(project: u32, on: w::AnswerOn) -> p::Ask {
    match on {
        w::AnswerOn::Question(value) => {
            let v = value;
            p::Ask::AnswerQuestion { project, task: v.task, question: v.message, words: v.words }
        }
        w::AnswerOn::Person(value) => {
            let v = value;
            p::Ask::AnswerPerson {
                project,
                task: v.task,
                revision: v.revision.value,
                result: asks::person_answer(v.answer),
            }
        }
    }
}
pub(super) fn decide(project: u32, on: w::DecideOn) -> Result<p::Ask, Untranslatable> {
    Ok(match on {
        w::DecideOn::Proposal(value) => {
            let v = value;
            p::Ask::DecideProposal {
                project,
                proposer: asks::proposer(&v.proposal.proposer),
                proposal: v.proposal.number,
                decision: asks::proposal_decision(v.decision)?,
                running: match v.running {
                    Some(value) => Some(mapped::running_to_domain(value)),
                    None => None,
                },
            }
        }
        w::DecideOn::Escalation(value) => {
            let v = value;
            p::Ask::DecideEscalation {
                project,
                task: v.task,
                revision: v.revision.value,
                decision: asks::escalation_decision(v.decision)?,
            }
        }
    })
}
pub(super) fn templates(value: w::PolicyEditTemplates) -> p::PolicyChange {
    p::PolicyChange::Templates { overrides: overrides(value.overrides) }
}
pub(super) fn overrides(values: List<w::TemplateOverride>) -> Box<[p::TemplateOverride]> {
    let mut overrides = List::with_capacity(values.len());
    for value in values.into_boxed() {
        overrides.push(mapped::template_override_to_domain(value)).expect("same count");
    }
    overrides.into_boxed()
}
pub(super) fn executor(value: w::TemplateExecutor) -> p::TemplateExecutor {
    mapped::template_executor_to_domain(value)
}
pub(super) fn budget(value: w::TemplateBudget) -> p::TemplateBudget {
    mapped::template_budget_to_domain(value)
}
pub(super) fn deadline(value: w::TemplateDeadline) -> p::TemplateDeadline {
    mapped::template_deadline_to_domain(value)
}

pub(super) fn tracking(value: w::Tracking) -> p::Tracking {
    mapped::tracking_to_domain(value)
}

/// Decode the pieces before anything reaches the core.
pub(super) fn compound(
    value: w::Compound,
    models: &[crate::Model],
    limits: &w::Limits,
    names: &mut List<Box<[u8]>>,
) -> Result<p::Ask, Untranslatable> {
    let path = w::paths::COMPOUND_OPERATIONS;
    let mut reader = skein_lib::Reader::new(value.operations.as_ref());
    let mut decoded = List::with_capacity(asks::count(value.operations.as_ref(), path)?);
    for _ in 0..decoded.capacity() {
        let bytes = skein_codec::piece::counted(&mut reader);
        let Ok(operation) = w::Operation::decode(limits, &mut skein_lib::Reader::new(&bytes)) else {
            return Err(Untranslatable { path });
        };
        decoded.push(operation).expect("pieces bounded by bytes");
    }
    let mut operations = List::with_capacity(decoded.len());
    for operation in decoded.into_boxed() {
        operations.push(operation_to_people(operation, value.project, models, limits, names)?).expect("same capacity");
    }
    Ok(p::Ask::Compound { project: value.project, operations })
}
fn operation_to_people(
    value: w::Operation,
    project: u32,
    models: &[crate::Model],
    limits: &w::Limits,
    names: &mut List<Box<[u8]>>,
) -> Result<p::Operation, Untranslatable> {
    if operation_project(&value) != project {
        return Err(Untranslatable { path: w::paths::COMPOUND_OPERATIONS });
    }
    Ok(match value {
        w::Operation::Message(v) => p::Operation::Message { task: v.task, words: v.words },
        w::Operation::Amend(v) => p::Operation::Amend {
            task: v.task,
            revision: match v.revision {
                Some(v) => v.value,
                None => 0,
            },
            amendment: Box::new(asks::amendment(v.change, v.reason, models, limits, names)?),
            release: None,
        },
        w::Operation::Cancel(v) => p::Operation::Cancel { task: v.task, reason: v.reason, grace: v.grace },
        w::Operation::Release(v) => p::Operation::Release { task: v.task, revision: v.revision.value, again: v.again },
        w::Operation::Stop(v) => p::Operation::Stop { task: v.task, reason: v.reason },
        w::Operation::MoveTask(v) => p::Operation::Move { task: v.task, reason: v.reason },
    })
}
fn operation_project(value: &w::Operation) -> u32 {
    match value {
        w::Operation::Message(v) => v.project,
        w::Operation::Amend(v) => v.project,
        w::Operation::Cancel(v) => v.project,
        w::Operation::Release(v) => v.project,
        w::Operation::Stop(v) => v.project,
        w::Operation::MoveTask(v) => v.project,
    }
}

/// Unoffered acts and project mismatches are typed refusals before the core sees them.
/// Job admission: jig/docs/design/client/domain/contract/documents.md, 7.
pub(crate) fn refused(record: &w::Ask, limits: &w::Limits) -> Result<Option<w::KeyedAnswer>, w::Problem> {
    let tasks = match record {
        w::Ask::Create(value) => Some(value.tasks.as_ref()),
        w::Ask::Delegate(value) => Some(value.tasks.as_ref()),
        w::Ask::Message(_)
        | w::Ask::Answer(_)
        | w::Ask::Take(_)
        | w::Ask::HandBack(_)
        | w::Ask::Decide(_)
        | w::Ask::Release(_)
        | w::Ask::Amend(_)
        | w::Ask::Stop(_)
        | w::Ask::Cancel(_)
        | w::Ask::MoveTask(_)
        | w::Ask::Seen(_)
        | w::Ask::EditNote(_)
        | w::Ask::DeleteNote(_)
        | w::Ask::ChangePolicy(_)
        | w::Ask::SetRoles(_)
        | w::Ask::MakeService(_)
        | w::Ask::ReissueService(_)
        | w::Ask::SetPool(_)
        | w::Ask::Compound(_)
        | w::Ask::Propose(_)
        | w::Ask::Withdraw(_)
        | w::Ask::SignIn(_)
        | w::Ask::SignOut(_)
        | w::Ask::AccountChange(_)
        | w::Ask::Rename(_)
        | w::Ask::MakeProject(_)
        | w::Ask::ChangeSettings(_)
        | w::Ask::PauseWorker(_)
        | w::Ask::MoveToNewest(_)
        | w::Ask::Lift(_)
        | w::Ask::RemoveProject(_)
        | w::Ask::Restore(_)
        | w::Ask::SetVisibility(_)
        | w::Ask::ChangeModel(_)
        | w::Ask::ReadLists(_)
        | w::Ask::Talk(_)
        | w::Ask::Leave(_)
        | w::Ask::Conclude(_) => None,
    };
    if let Some(bytes) = tasks {
        let mut reader = skein_lib::Reader::new(bytes);
        while let Some(bytes) = wire_piece(&mut reader)? {
            let task = w::NewTask::decode(limits, &mut skein_lib::Reader::new(&bytes))?;
            match &task.executor {
                w::Executor::Job(_) => {
                    return encoded_refusal(w::Refusal::NotOffered, None, limits);
                }
                w::Executor::Agent(_) | w::Executor::Procedure(_) | w::Executor::Person(_) => {}
            }
        }
        return Ok(None);
    }
    let w::Ask::Compound(value) = record else {
        return Ok(None);
    };
    let mut reader = skein_lib::Reader::new(value.operations.as_ref());
    let mut at = 0_u32;
    while let Some(bytes) = wire_piece(&mut reader)? {
        let operation = w::Operation::decode(limits, &mut skein_lib::Reader::new(&bytes))?;
        let project = operation_project(&operation);
        if project != value.project {
            let refusal = w::Refusal::Malformed(w::RefusalMalformed { at, why: w::Malformed::ProjectMismatch });
            return encoded_refusal(refusal, Some(at), limits);
        }
        at = at.checked_add(1).ok_or(super::problem(skein_codec::Reason::Bound))?;
    }
    Ok(None)
}

fn encoded_refusal(
    refusal: w::Refusal,
    at: Option<u32>,
    limits: &w::Limits,
) -> Result<Option<w::KeyedAnswer>, w::Problem> {
    let answer = w::Final::Refused(w::Refused { refusal, at });
    let mut out = skein_lib::Writer::new(skein_lib::length::usize_of(answer.measure(limits)?));
    answer.write(limits, &mut out).expect("measured answer");
    Ok(Some(super::answers::owned(w::Owner::Jig, jig_client_protocol_wire::VERSION, out.finish())))
}

fn wire_piece(reader: &mut skein_lib::Reader<'_>) -> Result<Option<Box<[u8]>>, w::Problem> {
    match skein_codec::piece::split(reader) {
        Ok(value) => Ok(value),
        Err(_) => Err(super::problem(skein_codec::Reason::Malformed)),
    }
}

/// Batch members remain data until people admission resolves their authority.
pub(super) fn create(
    value: w::Create,
    models: &[crate::Model],
    limits: &w::Limits,
    names: &mut List<Box<[u8]>>,
) -> Result<p::Ask, Untranslatable> {
    Ok(p::Ask::Create {
        project: value.project,
        batch: batch(value.tasks.as_ref(), w::paths::CREATE_TASKS, models, limits, names)?,
    })
}
pub(super) fn delegate(
    value: w::Delegate,
    models: &[crate::Model],
    limits: &w::Limits,
    names: &mut List<Box<[u8]>>,
) -> Result<p::Ask, Untranslatable> {
    Ok(p::Ask::Delegate {
        project: value.project,
        task: value.task,
        batch: batch(value.tasks.as_ref(), w::paths::DELEGATE_TASKS, models, limits, names)?,
    })
}
fn batch(
    bytes: &[u8],
    path: w::Path,
    models: &[crate::Model],
    limits: &w::Limits,
    names: &mut List<Box<[u8]>>,
) -> Result<Box<[p::Member]>, Untranslatable> {
    let mut tasks = List::with_capacity(asks::count(bytes, path)?);
    let mut reader = skein_lib::Reader::new(bytes);
    for _ in 0..tasks.capacity() {
        let bytes = skein_codec::piece::counted(&mut reader);
        let task = match w::NewTask::decode(limits, &mut skein_lib::Reader::new(&bytes)) {
            Ok(task) => task,
            Err(problem) => return Err(Untranslatable { path: problem.path }),
        };
        tasks.push(member(task, models, limits, names)?).expect("counted pieces");
    }
    Ok(tasks.into_boxed())
}
fn member(
    value: w::NewTask,
    models: &[crate::Model],
    limits: &w::Limits,
    names: &mut List<Box<[u8]>>,
) -> Result<p::Member, Untranslatable> {
    let v = value;
    let spec = asks::spec(v.spec);
    let mut dependencies = List::with_capacity(v.dependencies.len());
    for dependency in v.dependencies.into_boxed() {
        dependencies.push(mapped::task_ref_to_domain(dependency)).expect("same capacity");
    }
    if v.symbols.len() > limits.new_task_symbols {
        return Err(Untranslatable { path: w::paths::NEW_TASK_SYMBOLS });
    }
    let mut symbols = List::with_capacity(v.symbols.len());
    for symbol in v.symbols.into_boxed() {
        symbols.push(mapped::symbol_to_domain(symbol)).expect("same capacity");
    }
    Ok(p::Member {
        symbols: symbols.into_boxed(),
        deadline: mapped::new_deadline_to_domain(v.deadline),
        executor: match v.executor {
            w::Executor::Job(_) => return Err(Untranslatable { path: w::paths::NEW_TASK_EXECUTOR }),
            w::Executor::Agent(v) => p::TaskExecutor::Agent { charter: v.charter },
            w::Executor::Procedure(v) => p::TaskExecutor::Procedure { connector: v.connector, code: v.code },
            w::Executor::Person(v) => p::TaskExecutor::Person(mapped::person_address_to_domain(v.value)),
        },
        words: spec.words,
        parameters: spec.parameters,
        inputs: spec.inputs,
        contract: mapped::new_contract_to_domain(v.contract),
        authority: asks::authority(v.authority, limits, names)?,
        budget: mapped::new_budget_to_domain(v.budget),
        wake: match v.wake {
            Some(wake) => Some(asks::wake(wake)),
            None => None,
        },
        dependencies: dependencies.into_boxed(),
        named: asks::grants(&v.named, limits.new_task_named, w::paths::NEW_TASK_NAMED, limits)?,
        tracked: v.tracked,
        label: v.label,
        flags: mapped::task_flags_to_domain(v.flags),
        llm: super::llm::people(v.llm, models, w::paths::NEW_TASK_LLM)?,
    })
}

pub(super) fn propose(
    value: w::Propose,
    models: &[crate::Model],
    limits: &w::Limits,
    names: &mut List<Box<[u8]>>,
) -> Result<p::Ask, Untranslatable> {
    let digest = match &value.action {
        w::ProposeAction::Settings(_) => {
            let record = w::Ask::Propose(value.clone());
            let mut bytes = skein_lib::Writer::new(skein_lib::length::usize_of(
                record.measure(limits).expect("bounded client record"),
            ));
            record.write(limits, &mut bytes).expect("measured ask");
            crate::sign_in::digest(&bytes.finish()).expose().try_into().expect("32-byte digest")
        }
        w::ProposeAction::Batch(_) | w::ProposeAction::Amendment(_) => [0; 32],
    };
    let v = value;
    let action = match v.action {
        w::ProposeAction::Settings(value) => p::Action::Settings { against: value.against, digest },
        w::ProposeAction::Batch(value) => {
            p::Action::Batch(batch(value.tasks.as_ref(), w::paths::PROPOSE_BATCH_TASKS, models, limits, names)?)
        }
        w::ProposeAction::Amendment(value) => p::Action::Amend {
            task: value.task,
            revision: value.revision.value,
            amendment: Box::new(asks::amendment(value.change, v.reason.clone(), models, limits, names)?),
        },
    };
    Ok(p::Ask::Propose { project: v.project, action, reason: v.reason })
}
