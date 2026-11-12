//! Task pages, live trees and goal lines (`client/domain/pages.md`, 5 and 6.2).
//! Closed at once (jig/docs/design/domain/people/acts.md, 2).
//! Copies bounded rows from their owners. Trees fold every unshown live row into
//! phase counts; held counts cover the whole subtree. Goal progress uses retained
//! goal projections, including ended rows. Nothing here mutates a task or policy.

pub use crate::below::documents::task_tree::*;

use crate::{
    Core, Limits,
    documents::{Cursor, Missing, Proposer, Window},
};
use jig_domain_authority as authority;
use jig_domain_tasks as tasks;
use skein_lib::{List, Wall};

pub(super) fn pattern(limits: &Limits, value: &authority::Pattern) -> Result<PatternEnd, Missing> {
    let bytes = match &value.last {
        authority::Last::Exact(bytes) | authority::Last::Open(bytes) => bytes,
    };
    if bytes.len() > skein_lib::length::usize_of(limits.documents.text_bytes) {
        return Err(Missing::TooLarge);
    }
    Ok(match &value.last {
        authority::Last::Exact(bytes) => PatternEnd::Exact(skein_lib::bytes::copy_of(bytes)),
        authority::Last::Open(bytes) => PatternEnd::Open(skein_lib::bytes::copy_of(bytes)),
    })
}

pub(super) fn authority(limits: &Limits, value: &authority::Authority) -> Result<AuthorityLine, Missing> {
    let mut grants = List::with_capacity(limits.documents.grants);
    for value in &value.grants {
        grants
            .push(GrantLine {
                connector: value.connector,
                kind: value.kind,
                segments: path(limits, &value.pattern.segments)?,
                last: pattern(limits, &value.pattern)?,
            })
            .expect("bounded grants");
    }
    let mut executors = List::with_capacity(limits.documents.executors);
    for value in &value.delegation.kinds {
        executors
            .push(match value {
                authority::Executor::Charter(number) => tasks::AuthorityExecutor::Charter(*number),
                authority::Executor::Procedure(number) => tasks::AuthorityExecutor::Procedure(*number),
                authority::Executor::Role(number) => tasks::AuthorityExecutor::Role(*number),
                authority::Executor::Job(number) => tasks::AuthorityExecutor::Job(*number),
            })
            .expect("bounded executors");
    }
    let mut note_resources = List::with_capacity(limits.documents.grants);
    for value in &value.note_resources {
        note_resources
            .push(NoteScope {
                connector: value.connector,
                segments: path(limits, &value.pattern.segments)?,
                last: pattern(limits, &value.pattern)?,
            })
            .expect("bounded note scopes");
    }
    Ok(AuthorityLine {
        models: tasks::Models::from_parts(value.models.bits(), value.models.is_all()),
        tools: tasks::Tools(value.tools.0),
        grants,
        executors,
        tasks: value.delegation.tasks,
        depth: value.delegation.depth,
        ceiling: tasks::Budget { spend: value.budget.spend, deadline: value.budget.deadline },
        notes: tasks::Scopes(value.notes.0),
        note_resources,
    })
}

pub(super) fn contract(limits: &Limits, value: &tasks::Contract) -> Result<ContractLine, Missing> {
    Ok(match value {
        tasks::Contract::Approval { state, part } => {
            if state.len() > skein_lib::length::usize_of(limits.documents.text_bytes) {
                return Err(Missing::TooLarge);
            }
            match &part.key {
                PartKey::Number(_) => {}
                PartKey::Path(segments) => {
                    if segments.capacity() > limits.documents.segments {
                        return Err(Missing::TooLarge);
                    }
                    for segment in segments {
                        if segment.len() > skein_lib::length::usize_of(limits.documents.text_bytes) {
                            return Err(Missing::TooLarge);
                        }
                    }
                }
            }
            ContractLine::Approval { state: state.clone(), part: part.clone() }
        }
        tasks::Contract::Job { bytes } => ContractLine::Job { bytes: *bytes },
        tasks::Contract::Report { words } => ContractLine::Report { words: *words },
        tasks::Contract::Change { connector, kind, words } => {
            ContractLine::Change { connector: *connector, kind: *kind, words: *words }
        }
        tasks::Contract::Verdict { choices } => {
            let mut result = List::with_capacity(limits.documents.lines);
            for choice in choices.iter().take(bound(limits)) {
                if choice.fields.len() > bound(limits) {
                    return Err(Missing::TooLarge);
                }
                let mut names = 0_usize;
                for field in &choice.fields {
                    names = names.checked_add(field.name.len()).ok_or(Missing::TooLarge)?;
                }
                if names > skein_lib::length::usize_of(limits.documents.text_bytes) {
                    return Err(Missing::TooLarge);
                }
                result.push(choice.clone()).expect("bounded form choices");
            }
            ContractLine::Verdict { cut: size(result.len()) < choices.len(), choices: result }
        }
    })
}

pub(super) fn hold_for(
    core: &Core,
    limits: &Limits,
    party: u64,
    row: &tasks::TaskRecord,
) -> Result<Option<HoldLine>, Missing> {
    let mut line = hold(limits, row)?;
    if let Some(line) = &mut line {
        line.placed = placed(core, limits, party, row.number)?;
    }
    Ok(line)
}

pub(super) fn placed(core: &Core, limits: &Limits, party: u64, task: u64) -> Result<Option<PlacedLine>, Missing> {
    let Some(hold) = core.tasks.placed_on(task) else { return Ok(None) };
    if hold.reason.len() > skein_lib::length::usize_of(limits.documents.text_bytes) {
        return Err(Missing::TooLarge);
    }
    Ok(Some(PlacedLine {
        number: hold.number,
        owner: crate::admission::roles::hold_owner(hold.owner),
        mode: hold.mode,
        scope: hold.scope,
        reason: hold.reason.clone(),
        placed_at: Wall::from_nanos(hold.placed_at.checked_mul(1_000_000).ok_or(Missing::TooLarge)?),
        revision: hold.revision,
        lifted: hold.lifted,
        may_lift: core.party_hold_admit(party, hold).is_ok(),
    }))
}

pub(super) fn link(core: &Core, limits: &Limits, party: u64, number: u64) -> Result<TaskLink, Missing> {
    match core.tasks.task(number) {
        Some(_) => Ok(TaskLink::Live(super::small::chip(core, limits, party, number)?)),
        None => Ok(TaskLink::Stored {
            number,
            status: match core.tasks.stub(number) {
                Some(stub) => Some(stub.phase),
                None => None,
            },
        }),
    }
}

fn links(core: &Core, limits: &Limits, party: u64, numbers: &[u64]) -> Result<List<TaskLink>, Missing> {
    let mut result = List::with_capacity(limits.documents.lines);
    for number in numbers.iter().take(bound(limits)) {
        result.push(link(core, limits, party, *number)?).expect("bounded task links");
    }
    Ok(result)
}

pub(super) fn descendant(core: &Core, number: u64, root: u64, depth: u32) -> bool {
    let mut current = number;
    for _ in 0..=depth {
        if current == root {
            return true;
        }
        let Some(row) = core.tasks.task(current) else { return false };
        match row.requester {
            tasks::Party::Task(parent) => current = parent,
            tasks::Party::Person(_) | tasks::Party::Deployment { .. } => return false,
        }
    }
    false
}

fn proposal(row: &tasks::TaskRecord) -> Option<ProposalRef> {
    let value = row.proposal.as_ref()?;
    match value.state {
        tasks::ProposalState::Pending { .. } => {
            Some(ProposalRef { proposer: Proposer::Task(value.proposer), number: value.number })
        }
        tasks::ProposalState::Accepted { .. }
        | tasks::ProposalState::Rejected { .. }
        | tasks::ProposalState::Withdrawn => None,
    }
}

/// Only the pool's person may read a projectless Task (jig/docs/design/domain/authority.md, 6.1).
pub(crate) fn person_funded(row: &tasks::TaskRecord, party: u64) -> bool {
    row.project == jig_domain_people::DEPLOYMENT
        && match row.funder {
            tasks::Funder::Pool { person, .. } => person == party,
            tasks::Funder::Period { .. } | tasks::Funder::Recurring { .. } | tasks::Funder::Task(_) => false,
        }
}

pub(super) fn task(core: &Core, limits: &Limits, party: u64, number: u64, decision: Wall) -> Result<Task, Missing> {
    let row = core.tasks.task(number).ok_or(Missing::Unknown)?;
    let row = if person_funded(row, party) {
        match row.phase {
            tasks::Phase::Ended(_) => return Err(Missing::Ended),
            tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Held { .. } => {
                row
            }
        }
    } else {
        super::small::visible(core, party, number)?
    };
    task_from_row(core, limits, party, row, decision)
}

/// Construct a visible task from its retained owner row, without restoring it.
pub fn task_from_row(
    core: &Core,
    limits: &Limits,
    party: u64,
    row: &tasks::TaskRecord,
    decision: Wall,
) -> Result<Task, Missing> {
    task_with_lineage(core, limits, party, row, decision, true)
}

/// A load supplies its joined ancestry afterwards, including ended parents.
/// Missing live ancestry must not mark that complete document cut in advance.
pub(crate) fn task_for_load(
    core: &Core,
    limits: &Limits,
    party: u64,
    row: &tasks::TaskRecord,
    decision: Wall,
) -> Result<Task, Missing> {
    task_with_lineage(core, limits, party, row, decision, false)
}

#[expect(clippy::too_many_lines, reason = "the task page gathers one bounded overview without changing child state")]
fn task_with_lineage(
    core: &Core,
    limits: &Limits,
    party: u64,
    row: &tasks::TaskRecord,
    decision: Wall,
    live_lineage: bool,
) -> Result<Task, Missing> {
    if !(core.open_project(row.project) && core.people.role(party, row.project).is_some() || person_funded(row, party))
    {
        return Err(Missing::NotAllowed);
    }
    let number = row.number;
    let mut lineage = List::with_capacity(limits.documents.lines);
    let mut ancestor = row.requester;
    let mut cut = false;
    let goal = core.goal_of(row);
    let lineage_depth = if live_lineage { limits.tasks.depth } else { 0 };
    for _ in 0..lineage_depth {
        match ancestor {
            tasks::Party::Task(parent) => {
                let parent_row = match super::small::visible(core, party, parent) {
                    Ok(row) => row,
                    Err(Missing::Unknown | Missing::Ended) => {
                        cut = true;
                        break;
                    }
                    Err(why @ (Missing::NotAllowed | Missing::TooLarge)) => return Err(why),
                };
                if lineage.room() > 0 {
                    lineage.push(super::small::chip(core, limits, party, parent)?).expect("lineage room checked");
                } else {
                    cut = true;
                }
                ancestor = parent_row.requester;
            }
            tasks::Party::Person(_) | tasks::Party::Deployment { .. } => break,
        }
    }
    let mut ordered = List::with_capacity(limits.documents.lines);
    for index in (0..lineage.len()).rev() {
        ordered.push(lineage.get(index).expect("lineage index").clone()).expect("same bounded lineage");
    }
    let mut parameters = List::with_capacity(limits.documents.lines);
    for value in row.spec.parameters.iter().take(bound(limits)) {
        let parameter = match value {
            tasks::Parameter::Number { name, value } => Parameter::Number { name: *name, value: *value },
            tasks::Parameter::Bytes { name, value } => {
                Parameter::Bytes { name: *name, value: super::small::text(value, limits.documents.text_bytes) }
            }
            tasks::Parameter::Resource { name, connector, resource } => Parameter::Resource {
                name: *name,
                part: Part { connector: *connector, kind: PartKind::Resource, key: PartKey::Number(*resource) },
            },
        };
        parameters.push(parameter).expect("bounded parameters");
    }
    let mut resources = List::with_capacity(limits.documents.lines);
    for value in row.holdings.iter().take(bound(limits)) {
        let (name, kind, part_kind) = match value {
            tasks::Holding::Write { resource, kind } => (resource, *kind, PartKind::Resource),
            tasks::Holding::Slot { pool, kind } => (pool, *kind, PartKind::Pool),
        };
        let slot = core.tasks.writer(name);
        resources
            .push(ResourceLine {
                part: part_path(limits, name.connector, part_kind, &name.path)?,
                kind,
                taken: row.holds_taken,
                holder: core.tasks.holder(name),
                writer: match slot {
                    Some(slot) => Some(slot.writer),
                    None => None,
                },
                writer_lost: match slot {
                    Some(slot) => slot.lost,
                    None => false,
                },
            })
            .expect("bounded resource lines");
    }
    let mut proposals = List::with_capacity(limits.documents.lines);
    for candidate in core.tasks.view_tasks().as_ref() {
        if descendant(core, candidate.number, number, limits.tasks.depth)
            && let Some(reference) = proposal(core.tasks.task(candidate.number).expect("live indexed task"))
        {
            if proposals.room() == 0 {
                cut = true;
            } else {
                proposals.push(reference).expect("proposal room checked");
            }
        }
    }
    let mut subscriptions = List::with_capacity(limits.documents.lines);
    for value in row.subscriptions.iter().take(bound(limits)) {
        subscriptions.push(*value).expect("bounded subscriptions");
    }
    cut |= size(parameters.len()) < row.spec.parameters.len()
        || size(resources.len()) < row.holdings.len()
        || size(subscriptions.len()) < row.subscriptions.len()
        || row.spec.inputs.len() > bound(limits)
        || row.dependencies.len() > bound(limits)
        || row.references.len() > bound(limits);
    let result = match &row.phase {
        tasks::Phase::Closing(closing) => Some(ending(limits, &closing.ending)?),
        tasks::Phase::Held { was, .. } => match was {
            tasks::Was::Closing(closing) => Some(ending(limits, &closing.ending)?),
            tasks::Was::Waiting | tasks::Was::Active(_) => None,
        },
        tasks::Phase::Ended(value) => Some(ending(limits, value)?),
        tasks::Phase::Waiting | tasks::Phase::Active(_) => None,
    };
    let acts = super::offers::task(core, limits, party, row, decision)?;
    let authority = authority(limits, &crate::translate::tasks_authority(&row.authority))?;
    Ok(Task {
        talk: row.talk,
        session: row.session,
        settings: match row.executor {
            tasks::Executor::Agent { .. } => Some(TaskSettings {
                task: row.settings.task,
                run: match row.settings.run {
                    Some(run) if run != row.settings.task => Some(run),
                    Some(_) | None => None,
                },
                newest: core.current.newest(),
            }),
            tasks::Executor::Procedure { .. } | tasks::Executor::Person(_) | tasks::Executor::Job { .. } => None,
        },
        waits_on: crate::runs::waits::from_row(core, limits, row),
        label: row.label,
        flags: row.flags,
        last_activity: row.last_activity,
        made: row.created_at,
        acts,
        chip: super::small::chip_with_core(core, limits, party, row)?,
        project: row.project,
        goal,
        priority: row.tracked,
        executor: row.executor,
        requester: row.requester,
        lineage: ordered,
        delegates: super::live_delegates::from_row(core, limits, party, row)?,
        changed: row.changed,
        spec_revision: row.revisions.spec,
        person_revision: match row.executor {
            tasks::Executor::Person(_) => Some(row.revisions.person),
            tasks::Executor::Agent { .. } | tasks::Executor::Procedure { .. } | tasks::Executor::Job { .. } => None,
        },
        taken_by: row.taken_by,
        activity: row.activity,
        spec: super::objects::spec(limits, &row.spec),
        words: super::small::text(&row.spec.words, limits.documents.text_bytes),
        parameters,
        instructions: match row.executor {
            tasks::Executor::Agent { charter } => {
                let value = match &row.instructions {
                    Some(instructions) => Some(instructions),
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
        llm: core.document_llm(row),
        kept_account: row.kept_account,
        run_wait: super::task_wait::waiting(core, row),
        resources,
        inputs: links(core, limits, party, &row.spec.inputs)?,
        dependencies: links(core, limits, party, &row.dependencies)?,
        dependency_facts: dependency_facts(core, limits, row),
        references: links(core, limits, party, &row.references)?,
        subscriptions,
        wake: row.wake,
        contract: contract(limits, &row.contract)?,
        result,
        budget: budget(row),
        authority,
        hold: hold_for(core, limits, party, row)?,
        proposals,
        cut,
    })
}

fn folding_parent(core: &Core, rows: &List<TreeRow>, root: u64, number: u64, depth: u32) -> u64 {
    let mut current = number;
    for _ in 0..=depth {
        let row = core.tasks.task(current).expect("live descendant has live ancestors");
        let parent = match row.requester {
            tasks::Party::Task(parent) => parent,
            tasks::Party::Person(_) | tasks::Party::Deployment { .. } => {
                unreachable!("descendant has a requester task")
            }
        };
        if parent == root {
            return root;
        }
        for shown in rows {
            if shown.chip.number == parent {
                return parent;
            }
        }
        current = parent;
    }
    unreachable!("descendant reaches its root within the task depth")
}

fn fold_phase(folded: &mut List<FoldedCount>, parent: u64, root: u64, phase: &tasks::Phase) {
    let mut root_index = None;
    for index in 0..folded.len() {
        let fold = folded.get_mut(index).expect("fold index");
        if fold.parent == parent {
            fold.phases.add(phase);
            fold.held_below = fold.phases.held;
            return;
        }
        if fold.parent == root {
            root_index = Some(index);
        }
    }
    if folded.room() > 0 {
        let mut phases = PhaseCounts::ZERO;
        phases.add(phase);
        folded.push(FoldedCount { parent, held_below: phases.held, phases }).expect("fold room checked");
        return;
    }
    // At most rows + 1 parents can fold. Keep overflow in the root's count.
    let index = match root_index {
        Some(index) => index,
        None => folded.len().checked_sub(1).expect("at least one folding parent fits"),
    };
    let fold = folded.get_mut(index).expect("overflow fold index");
    fold.parent = root;
    fold.phases.add(phase);
    fold.held_below = fold.phases.held;
}

pub(super) fn tree(core: &Core, limits: &Limits, party: u64, number: u64, window: &Window) -> Result<Tree, Missing> {
    let root = super::small::visible(core, party, number)?;
    let mut rows = List::with_capacity(window.rows);
    let mut folded = List::with_capacity(window.rows.max(1));
    let mut held_below = 0_u32;
    let mut last = None;
    let after = match window.after {
        Some(Cursor::Number { number }) => number,
        Some(Cursor::Inbox { .. }) => return Err(Missing::Unknown),
        None => 0,
    };
    let mut more = false;
    for candidate in core.tasks.view_tasks().as_ref() {
        if candidate.number == number || !descendant(core, candidate.number, number, limits.tasks.depth) {
            continue;
        }
        let row = super::small::visible(core, party, candidate.number)?;
        let held = match row.phase {
            tasks::Phase::Held { .. } => true,
            tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Ended(_) => {
                false
            }
        };
        if held {
            held_below = held_below.checked_add(1).expect("bounded held count");
        }
        if rows.room() == 0 || candidate.number <= after {
            let parent = folding_parent(core, &rows, number, row.number, limits.tasks.depth);
            fold_phase(&mut folded, parent, number, &row.phase);
            more |= candidate.number > after;
            continue;
        }
        let parent = match row.requester {
            tasks::Party::Task(parent) => parent,
            tasks::Party::Person(_) | tasks::Party::Deployment { .. } => {
                unreachable!("descendant has a requester task")
            }
        };
        let mut proposals = List::with_capacity(1);
        if let Some(value) = proposal(row) {
            proposals.push(value).expect("one proposal per task");
        }
        rows.push(TreeRow {
            chip: super::small::chip(core, limits, party, row.number)?,
            parent,
            depth: row.depth.checked_sub(root.depth).expect("subtree depth"),
            proposals,
            spent: row.numbers.spent.checked_add(row.numbers.spent_below).expect("bounded spent"),
            dependencies: links(core, limits, party, &row.dependencies)?,
            cut: row.dependencies.len() > bound(limits),
        })
        .expect("tree window room checked");
        last = Some(Cursor::Number { number: row.number });
    }
    Ok(Tree {
        root: super::small::chip(core, limits, party, number)?,
        rows,
        folded,
        held_below,
        next: if more { last } else { None },
    })
}

pub(super) fn dependency_facts(core: &Core, limits: &Limits, row: &tasks::TaskRecord) -> List<Dependency> {
    let mut facts = dependency_rows(limits, row);
    for index in 0..facts.len() {
        let fact = facts.get_mut(index).expect("bounded dependency index");
        fact.status = match core.tasks.task(fact.number) {
            Some(task) => match &task.phase {
                tasks::Phase::Ended(ending) => Some(match ending {
                    tasks::Ending::Done(_) => tasks::Status::Done,
                    tasks::Ending::Failed { .. } => tasks::Status::Failed,
                    tasks::Ending::Cancelled { .. } => tasks::Status::Cancelled,
                }),
                tasks::Phase::Waiting
                | tasks::Phase::Active(_)
                | tasks::Phase::Held { .. }
                | tasks::Phase::Closing(_) => None,
            },
            None => match core.tasks.stub(fact.number) {
                Some(stub) => Some(stub.phase),
                None => None,
            },
        };
    }
    facts
}
/// A retained ended root has no live subtree (client/domain/contract/README.md, 5.2).
pub(crate) fn ended_tree(core: &Core, limits: &Limits, party: u64, row: &tasks::TaskRecord) -> Result<Tree, Missing> {
    if !(core.open_project(row.project) && core.people.role(party, row.project).is_some() || person_funded(row, party))
    {
        return Err(Missing::NotAllowed);
    }
    match row.phase {
        tasks::Phase::Ended(_) => {}
        tasks::Phase::Waiting | tasks::Phase::Active(_) | tasks::Phase::Closing(_) | tasks::Phase::Held { .. } => {
            return Err(Missing::Unknown);
        }
    }
    Ok(Tree {
        root: super::small::chip_with_core(core, limits, party, row)?,
        rows: List::with_capacity(0),
        folded: List::with_capacity(0),
        held_below: 0,
        next: None,
    })
}
