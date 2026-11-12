//! Decision objects and retained rows (`client/domain/objects.md`, sections 4–8;
//! Closed at once (jig/docs/design/domain/people/acts.md, 2).
//! `client/domain/pages.md`, 5.2). Store readers supply retained rows. Routing
//! trails and before/after history require those store rows; no trail is cached.

pub use crate::below::documents::objects::*;

mod party;

use crate::{
    Core, Limits,
    documents::{Chip, Ending, Missing, Parameter, Part, PartKey, PartKind, PriorPhase, Proposer, TaskLink, Text},
};
use alloc::boxed::Box;
use jig_domain_tasks as tasks;
use skein_lib::{List, Wall};

/// The ended delegate's identity and outcome, made from its retained task row.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct EndedDelegate {
    pub chip: Chip,
    pub ended: Ending,
    pub at: Option<Wall>,
}

fn bound(limits: &Limits) -> usize {
    skein_lib::length::usize_of(limits.documents.lines)
}
fn numbers(limits: &Limits, values: &[u64]) -> List<u64> {
    let mut result = List::with_capacity(limits.documents.lines);
    for value in values.iter().take(bound(limits)) {
        result.push(*value).expect("bounded numbers");
    }
    result
}
pub(super) fn spec(limits: &Limits, value: &tasks::Spec) -> SpecLine {
    let mut parameters = List::with_capacity(limits.documents.lines);
    for value in value.parameters.iter().take(bound(limits)) {
        parameters
            .push(match value {
                tasks::Parameter::Number { name, value } => Parameter::Number { name: *name, value: *value },
                tasks::Parameter::Bytes { name, value } => {
                    Parameter::Bytes { name: *name, value: super::small::text(value, limits.documents.text_bytes) }
                }
                tasks::Parameter::Resource { name, connector, resource } => Parameter::Resource {
                    name: *name,
                    part: Part { connector: *connector, kind: PartKind::Resource, key: PartKey::Number(*resource) },
                },
            })
            .expect("bounded spec parameters");
    }
    SpecLine {
        asks: value.asks,
        words: super::small::text(&value.words, limits.documents.text_bytes),
        parameters,
        inputs: numbers(limits, &value.inputs),
        cut: value.parameters.len() > bound(limits) || value.inputs.len() > bound(limits),
    }
}
pub(super) fn proposed(limits: &Limits, value: &tasks::New) -> Result<ProposedTask, Missing> {
    let mut resources = List::with_capacity(limits.documents.lines);
    for value in value.holdings.iter().take(bound(limits)) {
        let (name, kind) = match value {
            tasks::Holding::Write { resource, .. } => (resource, PartKind::Resource),
            tasks::Holding::Slot { pool, .. } => (pool, PartKind::Pool),
        };
        resources
            .push(super::task_tree::part_path(limits, name.connector, kind, &name.path)?)
            .expect("bounded holdings");
    }
    Ok(ProposedTask {
        deadline: value.deadline,
        number: value.number,
        executor: value.executor,
        spec: spec(limits, &value.spec),
        contract: super::task_tree::contract(limits, &value.contract)?,
        authority: super::act_authority(limits, &value.authority)?,
        budget: value.numbers.budget,
        funder: value.funder,
        dependencies: numbers(limits, &value.dependencies),
        resources,
        wake: value.wake,
        priority: value.tracked,
        cut: value.dependencies.len() > bound(limits) || value.holdings.len() > bound(limits),
    })
}
fn amendment(
    core: &Core,
    limits: &Limits,
    row: &tasks::TaskRecord,
    change: Option<&tasks::Amendment>,
) -> Result<AmendmentState, Missing> {
    let (description, authority, wake, dependencies) = match change {
        None => (&row.spec, &row.authority, row.wake, &row.dependencies),
        Some(change) => (
            change.spec.as_ref().unwrap_or(&row.spec),
            change.authority.as_ref().unwrap_or(&row.authority),
            change.wake.unwrap_or(row.wake),
            change.dependencies.as_ref().unwrap_or(&row.dependencies),
        ),
    };
    Ok(AmendmentState {
        addressed: match change {
            Some(change) => match change.addressed {
                Some(change) => Some(change.address),
                None => match row.executor {
                    tasks::Executor::Person(address) => Some(address),
                    tasks::Executor::Agent { .. } | tasks::Executor::Procedure { .. } | tasks::Executor::Job { .. } => {
                        None
                    }
                },
            },
            None => match row.executor {
                tasks::Executor::Person(address) => Some(address),
                tasks::Executor::Agent { .. } | tasks::Executor::Procedure { .. } | tasks::Executor::Job { .. } => None,
            },
        },
        taken_by: match change {
            Some(change) if change.addressed.is_some() => None,
            Some(_) | None => row.taken_by,
        },
        instructions: match row.executor {
            tasks::Executor::Agent { charter } => {
                let value = match change {
                    Some(change) => change.instructions.as_ref().or(row.instructions.as_ref()),
                    None => row.instructions.as_ref(),
                };
                let value = match value {
                    Some(value) => Some(value),
                    None => match core.charter_at(charter, row.settings.task) {
                        Some(charter) => Some(&charter.policy.instructions),
                        None => None,
                    },
                };
                match value {
                    Some(value) => Some(super::small::text(value, limits.documents.text_bytes)),
                    None => None,
                }
            }
            tasks::Executor::Procedure { .. } | tasks::Executor::Person(_) | tasks::Executor::Job { .. } => None,
        },
        spec: spec(limits, description),
        authority: super::act_authority(limits, authority)?,
        wake,
        dependencies: numbers(limits, dependencies),
        cut: dependencies.len() > bound(limits),
    })
}
fn action(
    core: &Core,
    limits: &Limits,
    party: u64,
    number: u64,
    value: &tasks::ProposalAction,
) -> Result<ProposalAction, Missing> {
    Ok(match value {
        tasks::ProposalAction::Settings { against, items, .. } => ProposalAction::Settings {
            against: *against,
            items: super::settings_review::settings_heads(items),
            review: None,
            made: None,
        },
        tasks::ProposalAction::Batch(batch) => {
            if batch.len() > bound(limits) {
                return Err(Missing::TooLarge);
            }
            let mut result = List::with_capacity(limits.documents.lines);
            for value in batch {
                result.push(proposed(limits, value)?).expect("whole bounded batch");
            }
            ProposalAction::Batch(result)
        }
        tasks::ProposalAction::Effect { connector, authority, attempt, completion, position, purpose } => {
            ProposalAction::Effect {
                part: Part { connector: *connector, kind: PartKind::Effect, key: PartKey::Number(number) },
                authority: super::act_authority(limits, authority)?,
                attempt: *attempt,
                completion: *completion,
                position: *position,
                purpose: *purpose,
            }
        }
        tasks::ProposalAction::ProcedureEffect { connector, authority, step, purpose } => {
            ProposalAction::ProcedureEffect {
                part: Part { connector: *connector, kind: PartKind::Effect, key: PartKey::Number(number) },
                authority: super::act_authority(limits, authority)?,
                step: *step,
                purpose: *purpose,
            }
        }
        tasks::ProposalAction::Amend { task, amendment, .. } => {
            party::amendment(core, limits, party, *task, amendment)?
        }
        tasks::ProposalAction::Widen { task, authority } => ProposalAction::Widen {
            task: super::task_tree::link(core, limits, party, *task)?,
            before: super::act_authority(limits, &super::small::visible(core, party, *task)?.authority)?,
            after: super::act_authority(limits, authority)?,
        },
        tasks::ProposalAction::ReleasePlaced { task, .. } | tasks::ProposalAction::Release { task } => {
            ProposalAction::Release {
                task: super::task_tree::link(core, limits, party, *task)?,
                hold: super::task_tree::hold_for(core, limits, party, super::small::visible(core, party, *task)?)?,
            }
        }
    })
}
pub(super) fn left(value: tasks::Numbers) -> u64 {
    value
        .budget
        .checked_sub(value.spent)
        .expect("spend fits budget")
        .checked_sub(value.spent_below)
        .expect("settled spend fits")
        .checked_sub(value.reserved)
        .expect("reserved fits")
}
fn funding(core: &Core, source: tasks::Funder, needed: u64) -> FundingOffer {
    let balance = match source {
        tasks::Funder::Task(task) => match core.tasks.task(task) {
            Some(row) => Some(row.numbers),
            None => None,
        },
        tasks::Funder::Pool { .. } | tasks::Funder::Period { .. } | tasks::Funder::Recurring { .. } => {
            match core.tasks.funding(source) {
                Some(row) => Some(row.numbers),
                None => None,
            }
        }
    };
    let left = match balance {
        Some(value) => Some(left(value)),
        None => None,
    };
    FundingOffer {
        source,
        needed,
        left,
        after: match left {
            Some(left) => left.checked_sub(needed),
            None => None,
        },
    }
}
fn holder_source(core: &Core, party: u64, project: u32, holder: tasks::ProposalHolder) -> tasks::Funder {
    match holder {
        tasks::ProposalHolder::Task(number) => tasks::Funder::Task(number),
        tasks::ProposalHolder::Person(person) => tasks::Funder::Pool { project, person, period: core.period(project) },
        tasks::ProposalHolder::Policy { .. } => {
            tasks::Funder::Pool { project, person: party, period: core.period(project) }
        }
    }
}
fn reviewed_party_action(
    core: &Core,
    limits: &Limits,
    party: u64,
    value: &tasks::PersonProposal,
) -> Result<(ProposalAction, tasks::ProposalKind), Missing> {
    let (mut action, kind) = party::action(core, limits, party, &value.action)?;
    match &mut action {
        ProposalAction::Settings { review, .. } => {
            *review = super::settings_review::review(core, limits, Proposer::Party(value.proposer), value.number);
        }
        ProposalAction::Batch(_)
        | ProposalAction::Effect { .. }
        | ProposalAction::ProcedureEffect { .. }
        | ProposalAction::Amend { .. }
        | ProposalAction::Widen { .. }
        | ProposalAction::Release { .. } => {}
    }
    Ok((action, kind))
}
fn party_proposal_visible(core: &Core, party: u64, proposer: u64, project: u32) -> bool {
    (project == jig_domain_people::DEPLOYMENT || core.open_project(project))
        && (party == proposer || core.people.role(party, project).is_some())
}

fn proposal_reason(reason: &[u8], action: &ProposalAction, limits: &Limits) -> Text {
    let bytes = match action {
        ProposalAction::Settings { .. } => limits.people.words,
        ProposalAction::Batch(_)
        | ProposalAction::Effect { .. }
        | ProposalAction::ProcedureEffect { .. }
        | ProposalAction::Amend { .. }
        | ProposalAction::Widen { .. }
        | ProposalAction::Release { .. } => limits.documents.text_bytes,
    };
    super::small::text(reason, bytes)
}

pub(super) fn proposal(
    core: &Core,
    limits: &Limits,
    party: u64,
    proposer: Proposer,
    number: u64,
    decision: Wall,
) -> Result<Proposal, Missing> {
    proposal_for(core, limits, Some(party), proposer, number, decision)
}

pub(crate) fn run_proposal(
    core: &Core,
    limits: &Limits,
    proposer: Proposer,
    number: u64,
    decision: Wall,
) -> Result<Proposal, Missing> {
    proposal_for(core, limits, None, proposer, number, decision)
}

#[expect(clippy::too_many_lines, reason = "one live builder keeps party and run proposal documents identical")]
fn proposal_for(
    core: &Core,
    limits: &Limits,
    party: Option<u64>,
    proposer: Proposer,
    number: u64,
    decision: Wall,
) -> Result<Proposal, Missing> {
    let (project, reason, mut made, as_holder, waiting) = match proposer {
        Proposer::Task(task) => {
            let row = core.tasks.task(task).ok_or(Missing::Ended)?;
            let value = row.proposal.as_ref().ok_or(Missing::Ended)?;
            if party.is_none() && value.number != number {
                return Err(Missing::Ended);
            }
            let made = reviewed_task_action(core, limits, party, task, value)?;
            if value.number != number {
                return Err(Missing::Ended);
            }
            let waiting = match value.state {
                tasks::ProposalState::Pending { holder, since } => ProposalWaiting { holder, since },
                tasks::ProposalState::Accepted { .. }
                | tasks::ProposalState::Rejected { .. }
                | tasks::ProposalState::Withdrawn => return Err(Missing::Ended),
            };
            (value.project, proposal_reason(&value.reason, &made, limits), made, value.as_holder, waiting)
        }
        Proposer::Party(person) => {
            let value = core.tasks.person_proposal(person, number).ok_or(Missing::Ended)?;
            if match party {
                Some(party) => !party_proposal_visible(core, party, person, value.project),
                None => !value.action.is_settings(),
            } {
                return Err(Missing::NotAllowed);
            }
            let since = match value.state {
                tasks::PersonProposalState::Pending { since } => since,
                tasks::PersonProposalState::SettingsAccepted { .. }
                | tasks::PersonProposalState::Accepted { .. }
                | tasks::PersonProposalState::Rejected { .. }
                | tasks::PersonProposalState::Withdrawn { .. } => {
                    return Err(Missing::Ended);
                }
            };
            let (action, kind) = reviewed_party_action(core, limits, party.unwrap_or(0), &value)?;
            (
                value.project,
                proposal_reason(&value.reason, &action, limits),
                action,
                kind != tasks::ProposalKind::Settings,
                ProposalWaiting { holder: tasks::ProposalHolder::Policy { project: value.project, kind }, since },
            )
        }
    };
    let mut offers = List::with_capacity(limits.documents.lines);
    let source = holder_source(core, party.unwrap_or(0), project, waiting.holder);
    let widening = match &mut made {
        ProposalAction::Batch(batch) => {
            let mut needed = 0_u64;
            for index in 0..batch.len() {
                let member = batch.get_mut(index).expect("bounded batch index");
                needed = needed.checked_add(member.budget).ok_or(Missing::TooLarge)?;
                member.funder = source;
            }
            offers.push(funding(core, source, needed)).expect("one acceptance source");
            None
        }
        ProposalAction::Settings { .. }
        | ProposalAction::Effect { .. }
        | ProposalAction::ProcedureEffect { .. }
        | ProposalAction::Release { .. } => {
            // Price belongs to the connector's effect description; authority's
            // budget is a ceiling, never a quoted effect price.
            None
        }
        ProposalAction::Amend { task, after, .. } => Some((task, after.authority.ceiling.spend)),
        ProposalAction::Widen { task, after, .. } => Some((task, after.ceiling.spend)),
    };
    if let Some((task, ceiling)) = widening {
        let task = match task {
            TaskLink::Live(chip) | TaskLink::Retained { chip, .. } => chip.number,
            TaskLink::Stored { number, .. } => *number,
        };
        let row = core.tasks.task(task).ok_or(Missing::Ended)?;
        let needed = if source == row.funder {
            ceiling.saturating_sub(row.numbers.budget)
        } else {
            ceiling.saturating_sub(row.numbers.spent).saturating_sub(row.numbers.spent_below)
        };
        offers.push(funding(core, source, needed)).expect("one widening source");
    }
    let projects = match u32::try_from(number) {
        Ok(number) => match core.settings_proposals.get(&(proposer, number)) {
            Some(proposal) => proposal.projects(),
            None => List::with_capacity(0),
        },
        Err(_) => List::with_capacity(0),
    };
    Ok(Proposal {
        projects,
        acts: match party {
            Some(party) => {
                super::offers::subject(core, limits, party, &super::Subject::Proposal { proposer, number }, decision)?
            }
            None => List::with_capacity(0),
        },
        proposer,
        number,
        project,
        reason,
        action: made,
        as_holder,
        waiting,
        funding: offers,
    })
}
pub(super) fn escalation(
    core: &Core,
    limits: &Limits,
    party: u64,
    number: u64,
    decision: Wall,
) -> Result<Escalation, Missing> {
    let task = super::task_tree::task(core, limits, party, number, decision)?;
    if task.hold.is_none() {
        return Err(Missing::Ended);
    }
    Ok(Escalation {
        acts: super::offers::subject(core, limits, party, &super::Subject::Escalation { task: number }, decision)?,
        task: Box::new(task),
    })
}
pub(super) fn question(
    core: &Core,
    limits: &Limits,
    party: u64,
    number: u64,
    message: u64,
    decision: Wall,
) -> Result<Question, Missing> {
    let row = super::small::visible(core, party, number)?;
    if !crate::inbox::questions_live(&row.phase) {
        return Err(Missing::Ended);
    }
    for line in &row.question_lines {
        if line.message != message || line.answer.is_some() {
            continue;
        }
        let fact = core.question_activity.get(&(number, line.line)).ok_or(Missing::Ended)?;
        let tasks::Line::Words { by, kind: tasks::MessageKind::Question, words, .. } = &fact.line else {
            return Err(Missing::Unknown);
        };
        return Ok(Question {
            acts: super::offers::subject(
                core,
                limits,
                party,
                &super::Subject::Question { task: number, message },
                decision,
            )?,
            task: super::small::chip(core, limits, party, number)?,
            message,
            from: *by,
            at: fact.at,
            words: super::small::text(words, limits.documents.text_bytes),
        });
    }
    Err(Missing::Ended)
}

pub(super) fn person(
    core: &Core,
    limits: &Limits,
    party: u64,
    number: u64,
    decision: Wall,
) -> Result<PersonTask, Missing> {
    let row = super::small::visible(core, party, number)?;
    person_from_row(core, limits, party, row, decision)
}

/// Construct a person form from its retained task row.
pub fn person_from_row(
    core: &Core,
    limits: &Limits,
    party: u64,
    row: &tasks::TaskRecord,
    decision: Wall,
) -> Result<PersonTask, Missing> {
    if !core.open_project(row.project) || core.people.role(party, row.project).is_none() {
        return Err(Missing::NotAllowed);
    }
    let address = match row.executor {
        tasks::Executor::Person(address) => address,
        tasks::Executor::Agent { .. } | tasks::Executor::Procedure { .. } | tasks::Executor::Job { .. } => {
            return Err(Missing::Unknown);
        }
    };
    let mut results = List::with_capacity(limits.documents.lines);
    for task in row.dependencies.iter().chain(row.spec.inputs.iter()).take(bound(limits)) {
        results
            .push(ShownResult {
                task: super::task_tree::link(core, limits, party, *task)?,
                result: match core.tasks.task(*task) {
                    Some(row) => match result_from_row(core, limits, party, row, decision) {
                        Ok(result) => Some(result),
                        Err(Missing::Ended | Missing::Unknown) => None,
                        Err(Missing::NotAllowed) => return Err(Missing::NotAllowed),
                        Err(Missing::TooLarge) => return Err(Missing::TooLarge),
                    },
                    None => None,
                },
            })
            .expect("bounded dependency results");
    }
    Ok(PersonTask {
        acts: super::offers::person(core, limits, party, row, decision)?,
        task: super::small::chip_with_core(core, limits, party, row)?,
        revision: row.revisions.person,
        requester: row.requester,
        spec: spec(limits, &row.spec),
        address,
        taken_by: row.taken_by,
        since: match row.person_wait {
            Some(wait) => Some(wait.since),
            None => None,
        },
        stall: match row.person_wait {
            Some(wait) => Some(wait.stall),
            None => None,
        },
        deadline: match row.person_wait {
            Some(wait) => Some(wait.deadline),
            None => None,
        },
        contract: super::task_tree::contract(limits, &row.contract)?,
        results,
        cut: row.dependencies.len().checked_add(row.spec.inputs.len()).ok_or(Missing::TooLarge)? > bound(limits),
    })
}
/// Make a result from the row loaded by the store reader, or a live closing row.
pub fn result_from_row(
    core: &Core,
    limits: &Limits,
    party: u64,
    row: &tasks::TaskRecord,
    decision: Wall,
) -> Result<ResultDoc, Missing> {
    if !core.open_project(row.project) || core.people.role(party, row.project).is_none() {
        return Err(Missing::NotAllowed);
    }
    let value = match &row.phase {
        tasks::Phase::Ended(ending) => ending,
        tasks::Phase::Closing(closing) | tasks::Phase::Held { was: tasks::Was::Closing(closing), .. } => {
            &closing.ending
        }
        tasks::Phase::Waiting
        | tasks::Phase::Active(_)
        | tasks::Phase::Held { was: tasks::Was::Waiting | tasks::Was::Active(_), .. } => return Err(Missing::Ended),
    };
    let mut delegates = List::with_capacity(limits.documents.lines);
    for task in row.delegates.iter().take(bound(limits)) {
        delegates.push(super::task_tree::link(core, limits, party, *task)?).expect("bounded delegates");
    }
    let mut followups = List::with_capacity(limits.documents.lines);
    for task in row.result_followups.iter().take(bound(limits)) {
        followups.push(super::task_tree::link(core, limits, party, *task)?).expect("bounded followups");
    }
    Ok(ResultDoc {
        acts: super::offers::subject(core, limits, party, &super::Subject::Result { task: row.number }, decision)?,
        task: super::small::chip_with_core(core, limits, party, row)?,
        contract: super::task_tree::contract(limits, &row.contract)?,
        ending: super::task_tree::ending(limits, value)?,
        at: row.ended_at,
        requester: row.requester,
        executor: row.executor,
        chosen_by: match row.executor {
            tasks::Executor::Person(_) => match row.person_answer {
                Some(answer) => Some(answer.by),
                None => None,
            },
            tasks::Executor::Agent { .. } | tasks::Executor::Procedure { .. } | tasks::Executor::Job { .. } => None,
        },
        delegates,
        followups: Some(followups),
        proposal: match &row.proposal {
            Some(value) => Some(super::ProposalRef { proposer: Proposer::Task(value.proposer), number: value.number }),
            None => None,
        },
        cut: row.delegates.len() > bound(limits) || row.result_followups.len() > bound(limits),
    })
}
pub(super) fn result(
    core: &Core,
    limits: &Limits,
    party: u64,
    number: u64,
    decision: Wall,
) -> Result<ResultDoc, Missing> {
    match core.tasks.task(number) {
        Some(row) => result_from_row(core, limits, party, row, decision),
        None => {
            if core.tasks.stub(number).is_some() {
                Err(Missing::Ended)
            } else {
                Err(Missing::Unknown)
            }
        }
    }
}
/// Copy a retained semantic history row without inventing absent before/after values.
pub fn history_from_row(limits: &Limits, row: &tasks::History) -> Result<HistoryLine, Missing> {
    Ok(HistoryLine {
        task: row.task,
        revision: row.revision,
        by: row.by,
        change: row.change,
        reason: super::small::text(&row.reason, limits.documents.text_bytes),
        at: Some(row.at),
        amendment: match &row.amendment {
            Some(change) => {
                Some((Box::new(settings(limits, &change.before)?), Box::new(settings(limits, &change.after)?)))
            }
            None => None,
        },
        proposal: match &row.proposal {
            Some(value) => Some(ProposalHistory {
                reason: super::small::text(
                    &value.reason,
                    if let tasks::ProposalAction::Settings { .. } = value.action {
                        limits.people.words
                    } else {
                        limits.documents.text_bytes
                    },
                ),
                action: history_action(limits, value)?,
                number: value.number,
                proposer: value.proposer,
                as_holder: value.as_holder,
                state: match &value.state {
                    tasks::ProposalState::Pending { holder, since } => {
                        ProposalHistoryState::Pending(ProposalWaiting { holder: *holder, since: *since })
                    }
                    tasks::ProposalState::Accepted { by } => ProposalHistoryState::Accepted { by: *by },
                    tasks::ProposalState::Rejected { by, reason } => ProposalHistoryState::Rejected {
                        by: *by,
                        reason: super::small::text(reason, limits.documents.text_bytes),
                    },
                    tasks::ProposalState::Withdrawn => ProposalHistoryState::Withdrawn,
                },
            }),
            None => None,
        },
    })
}
/// Copy an ended delegate after the store reader has checked project visibility.
pub fn ended_delegate_from_row(limits: &Limits, row: &tasks::TaskRecord) -> Result<EndedDelegate, Missing> {
    match &row.phase {
        tasks::Phase::Ended(value) => Ok(EndedDelegate {
            chip: super::small::chip_from_row(limits, row)?,
            ended: super::task_tree::ending(limits, value)?,
            at: row.ended_at,
        }),
        tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Held { .. } => {
            Err(Missing::Unknown)
        }
    }
}

/// Convert an immutable retained action, without borrowing its ended target.
pub fn retained_action(limits: &Limits, number: u64, value: &tasks::ProposalAction) -> Result<RetainedAction, Missing> {
    Ok(match value {
        tasks::ProposalAction::Settings { against, items, projects } => RetainedAction::Settings {
            projects: {
                let mut kept = List::with_capacity(crate::settings::PROPOSAL_PROJECTS);
                for project in projects {
                    kept.push(*project).or(Err(Missing::TooLarge))?;
                }
                kept
            },
            against: *against,
            items: super::settings_review::settings_heads(items),
            made: None,
        },
        tasks::ProposalAction::Batch(batch) => {
            if batch.len() > bound(limits) {
                return Err(Missing::TooLarge);
            }
            let mut result = List::with_capacity(limits.documents.lines);
            for value in batch {
                result.push(proposed(limits, value)?).expect("bounded retained batch");
            }
            RetainedAction::Batch(result)
        }
        tasks::ProposalAction::Effect { connector, authority, attempt, completion, position, purpose } => {
            RetainedAction::Effect {
                part: Part { connector: *connector, kind: PartKind::Effect, key: PartKey::Number(number) },
                authority: super::act_authority(limits, authority)?,
                attempt: *attempt,
                completion: *completion,
                position: *position,
                purpose: *purpose,
            }
        }
        tasks::ProposalAction::ProcedureEffect { connector, authority, step, purpose } => {
            RetainedAction::ProcedureEffect {
                part: Part { connector: *connector, kind: PartKind::Effect, key: PartKey::Number(number) },
                authority: super::act_authority(limits, authority)?,
                step: *step,
                purpose: *purpose,
            }
        }
        tasks::ProposalAction::Amend { task, amendment: change, .. } => RetainedAction::Amend {
            task: *task,
            change: Box::new(AmendmentChange {
                addressed: change.addressed,
                instructions: match &change.instructions {
                    Some(value) => Some(super::small::text(value, limits.documents.text_bytes)),
                    None => None,
                },
                release: change.release,
                spec: match &change.spec {
                    Some(value) => Some(spec(limits, value)),
                    None => None,
                },
                authority: match &change.authority {
                    Some(value) => Some(super::act_authority(limits, value)?),
                    None => None,
                },
                wake: change.wake,
                dependencies: match &change.dependencies {
                    Some(value) => Some(numbers(limits, value)),
                    None => None,
                },
                reason: super::small::text(&change.reason, limits.documents.text_bytes),
                cut: match &change.dependencies {
                    Some(value) => value.len() > bound(limits),
                    None => false,
                },
            }),
        },
        tasks::ProposalAction::Widen { task, authority } => {
            RetainedAction::Widen { task: *task, authority: super::act_authority(limits, authority)? }
        }
        tasks::ProposalAction::ReleasePlaced { task, .. } | tasks::ProposalAction::Release { task } => {
            RetainedAction::Release { task: *task }
        }
    })
}
/// Convert a retained lifecycle milestone without inventing missing actor/time.
pub fn milestone_from_row(limits: &Limits, value: &tasks::Milestone) -> Result<MilestoneLine, Missing> {
    Ok(MilestoneLine {
        task: value.task,
        position: value.position,
        at: Some(value.at),
        phase: match &value.phase {
            tasks::Phase::Waiting => MilestonePhase::Waiting,
            tasks::Phase::Active(active) => MilestonePhase::Active(*active),
            tasks::Phase::Closing(closing) => MilestonePhase::Closing {
                stage: closing.stage,
                ending: super::task_tree::ending(limits, &closing.ending)?,
            },
            tasks::Phase::Held { was, why, source } => MilestonePhase::Held {
                why: *why,
                source: super::task_tree::hold_source(limits, source.as_deref())?,
                ending: match was {
                    tasks::Was::Closing(closing) => Some(super::task_tree::ending(limits, &closing.ending)?),
                    tasks::Was::Waiting | tasks::Was::Active(_) => None,
                },
                before: match was {
                    tasks::Was::Waiting => PriorPhase::Waiting,
                    tasks::Was::Active(active) => PriorPhase::Active(*active),
                    tasks::Was::Closing(closing) => PriorPhase::Closing(closing.stage),
                },
            },
            tasks::Phase::Ended(ending) => MilestonePhase::Ended(super::task_tree::ending(limits, ending)?),
        },
    })
}

fn settings(limits: &Limits, row: &tasks::Settings) -> Result<AmendmentState, Missing> {
    Ok(AmendmentState {
        addressed: row.addressed,
        taken_by: row.taken_by,
        instructions: match &row.instructions {
            Some(value) => Some(super::small::text(value, limits.documents.text_bytes)),
            None => None,
        },
        spec: spec(limits, &row.spec),
        authority: super::act_authority(limits, &row.authority)?,
        wake: row.wake,
        dependencies: numbers(limits, &row.dependencies),
        cut: row.dependencies.len() > bound(limits),
    })
}

fn history_action(limits: &Limits, value: &tasks::Proposal) -> Result<RetainedAction, Missing> {
    let mut action = retained_action(limits, value.number, &value.action)?;
    if let RetainedAction::Settings { against, made, .. } = &mut action
        && let tasks::ProposalState::Accepted { .. } = value.state
    {
        // A settings decision always makes the immediate successor of its checked basis.
        *made = against.checked_add(1);
    }
    Ok(action)
}

fn reviewed_task_action(
    core: &Core,
    limits: &Limits,
    party: Option<u64>,
    task: u64,
    value: &tasks::Proposal,
) -> Result<ProposalAction, Missing> {
    match value.action {
        tasks::ProposalAction::Settings { .. } => {
            if let Some(party) = party
                && core.people.role(party, jig_domain_people::DEPLOYMENT) != Some(jig_domain_people::Role::Owner)
            {
                return Err(Missing::NotAllowed);
            }
        }
        tasks::ProposalAction::Batch(_)
        | tasks::ProposalAction::Effect { .. }
        | tasks::ProposalAction::ProcedureEffect { .. }
        | tasks::ProposalAction::Amend { .. }
        | tasks::ProposalAction::Widen { .. }
        | tasks::ProposalAction::Release { .. }
        | tasks::ProposalAction::ReleasePlaced { .. } => {
            super::small::visible(core, party.ok_or(Missing::NotAllowed)?, task)?;
        }
    }
    let mut made = action(core, limits, party.unwrap_or(0), value.number, &value.action)?;
    if let ProposalAction::Settings { review, .. } = &mut made {
        *review = super::settings_review::review(core, limits, Proposer::Task(task), value.number);
    }
    Ok(made)
}
