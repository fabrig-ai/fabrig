//! Field-for-field document values (client/domain/contract/documents.md and client/domain/contract/README.md, 4–5).
use super::{
    Box, Limits, List, Wall, authority_document_fields, c, chip_fields, chip_piece, live_delegate_piece, offered_piece,
    piece_problem, specification_document_fields, spend_line_fields, subscriptions_fields,
    task_contract_document_fields, task_link_piece, task_resource_piece, task_result_fields, text, too_large, w,
};
pub(crate) fn impact_counts_fields(value: &w::ImpactCounts) -> c::ImpactCounts {
    c::ImpactCounts {
        ended: value.ended(),
        waiting: value.waiting(),
        running: value.running(),
        held: value.held(),
        closing: value.closing(),
        done: value.done(),
        failed: value.failed(),
        cancelled: value.cancelled(),
    }
}

pub(crate) fn impact_fields(value: &w::Impact) -> c::Impact {
    c::Impact { tasks: impact_counts_fields(value.tasks()), live_runs: value.live_runs(), proposals: value.proposals() }
}

pub(crate) fn application_part_piece(bytes: &[u8]) -> Result<List<c::ApplicationPart>, w::Problem> {
    if !bytes.is_empty() {
        return Err(piece_problem(skein_codec::Reason::Malformed));
    }
    Ok(List::with_capacity(0))
}

pub(crate) fn closing_fields(value: &w::Closing, limits: &Limits) -> Result<c::Closing, w::Problem> {
    match crate::translate::facts::decode(value.rows(), limits)? {
        crate::translate::facts::Node::Closing(value) => Ok(*value),
        crate::translate::facts::Node::Chip(_)
        | crate::translate::facts::Node::ChipPhaseFacts(_)
        | crate::translate::facts::Node::Ending(_)
        | crate::translate::facts::Node::HeldRuntime(_)
        | crate::translate::facts::Node::HoldReason(_)
        | crate::translate::facts::Node::ChipDetails(_)
        | crate::translate::facts::Node::ChipEnding(_)
        | crate::translate::facts::Node::ResultValue(_) => Err(piece_problem(skein_codec::Reason::Malformed)),
    }
}

pub(crate) fn held_runtime_fields(value: &w::HeldRuntime, limits: &Limits) -> Result<c::HeldRuntime, w::Problem> {
    match crate::translate::facts::decode(value.rows(), limits)? {
        crate::translate::facts::Node::HeldRuntime(value) => Ok(*value),
        crate::translate::facts::Node::Chip(_)
        | crate::translate::facts::Node::ChipPhaseFacts(_)
        | crate::translate::facts::Node::Closing(_)
        | crate::translate::facts::Node::Ending(_)
        | crate::translate::facts::Node::HoldReason(_)
        | crate::translate::facts::Node::ChipDetails(_)
        | crate::translate::facts::Node::ChipEnding(_)
        | crate::translate::facts::Node::ResultValue(_) => Err(piece_problem(skein_codec::Reason::Malformed)),
    }
}

pub(crate) fn runtime_fields(value: &w::Runtime, limits: &Limits) -> Result<c::Runtime, w::Problem> {
    Ok(match value {
        w::Runtime::Active(row) => c::Runtime::Active(crate::translate::facts::active_run(row.value(), limits)?),
        w::Runtime::Idle => c::Runtime::Idle,
        w::Runtime::Starting => c::Runtime::Starting,
        w::Runtime::Running(row) => {
            let opt0 = match row.tool() {
                Some(item) => Some(text(item, limits)?),
                None => None,
            };
            c::Runtime::Running { attempt: row.attempt(), since: Wall::from_nanos(row.since()), tool: opt0 }
        }
        w::Runtime::WaitingForWords(row) => {
            c::Runtime::WaitingForWords { attempt: row.attempt(), since: Wall::from_nanos(row.since()) }
        }
        w::Runtime::BackingOff(row) => c::Runtime::BackingOff {
            at: Wall::from_nanos(row.at()),
            attempt: row.attempt(),
            tries: row.tries(),
            class: crate::translate::facts::failure_class(row.class()),
            reason: text(row.reason(), limits)?,
        },
        w::Runtime::Person(row) => {
            let opt0 = match row.since() {
                Some(item) => Some(Wall::from_nanos(*item)),
                None => None,
            };
            c::Runtime::Person { addressed: crate::translate::facts::recipient(row.addressed()), since: opt0 }
        }
        w::Runtime::Closing(row) => c::Runtime::Closing(closing_fields(row.value(), limits)?),
        w::Runtime::Held(row) => {
            let opt0 = match row.was() {
                Some(item) => Some(held_runtime_fields(item, limits)?),
                None => None,
            };
            c::Runtime::Held { was: opt0 }
        }
        w::Runtime::Ended => c::Runtime::Ended,
    })
}

pub(crate) fn writer_slot_piece(bytes: &[u8]) -> Result<List<c::WriterSlot>, w::Problem> {
    if !bytes.is_empty() {
        return Err(piece_problem(skein_codec::Reason::Malformed));
    }
    Ok(List::with_capacity(0))
}

pub(crate) fn task_context_fields(value: &w::TaskContext, limits: &Limits) -> Result<c::TaskContext, w::Problem> {
    let opt1 = match value.closing() {
        Some(item) => Some(closing_fields(item, limits)?),
        None => None,
    };
    if value.spend().len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list2 = List::with_capacity(value.spend().len());
    for item in value.spend() {
        list2.push(spend_line_fields(item)).expect("reserved supplied list");
    }
    Ok(c::TaskContext {
        parts: application_part_piece(value.parts())?,
        closing: opt1,
        spend: list2,
        runtime: runtime_fields(value.runtime(), limits)?,
        impact: impact_fields(value.impact()),
        writer_slots: writer_slot_piece(value.writer_slots())?,
    })
}

#[expect(clippy::too_many_lines, reason = "one bounded task document translation preserves its optional facts")]
pub(crate) fn task_fields(value: &w::Task, limits: &Limits) -> Result<c::Task, w::Problem> {
    let opt0 = match value.goal() {
        Some(item) => Some(*item),
        None => None,
    };
    let opt1 = match value.priority() {
        Some(item) => Some(*item),
        None => None,
    };
    if value.lineage().len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list2 = List::with_capacity(value.lineage().len());
    for item in value.lineage() {
        list2.push(*item).expect("reserved supplied list");
    }
    let opt3 = match value.person_revision() {
        Some(item) => Some(crate::translate::answers::revision(item)),
        None => None,
    };
    let opt4 = match value.taken_by() {
        Some(item) => Some(*item),
        None => None,
    };
    let opt5 = match value.instructions() {
        Some(item) => Some(crate::translate::facts::document_text(item, limits)?),
        None => None,
    };
    let opt6 = match value.hold() {
        Some(item) => Some(Box::new(crate::translate::facts::chip_hold(item, limits)?)),
        None => None,
    };
    if value.proposals().len() > limits.client().rows {
        return Err(too_large());
    }
    let mut list7 = List::with_capacity(value.proposals().len());
    for item in value.proposals() {
        list7.push(crate::translate::answers::proposal_key(item)).expect("reserved supplied list");
    }
    let opt8 = match value.result() {
        Some(item) => Some(task_result_fields(item, limits)?),
        None => None,
    };
    let opt9 = match value.context() {
        Some(item) => Some(Box::new(task_context_fields(item, limits)?)),
        None => None,
    };
    Ok(c::Task {
        talk: match value.talk() {
            Some(talk) => Some(c::Talk {
                party: talk.party(),
                opened: Wall::from_nanos(talk.opened()),
                last: Wall::from_nanos(talk.last()),
            }),
            None => None,
        },
        session: match value.session() {
            Some(session) => Some(c::Session { first: session.first(), last: Wall::from_nanos(session.last()) }),
            None => None,
        },
        settings: match value.settings() {
            Some(settings) => Some(c::TaskSettings {
                task: settings.task(),
                run: match settings.run() {
                    Some(run) => Some(*run),
                    None => None,
                },
                newest: settings.newest(),
            }),
            None => None,
        },
        dependency_facts: crate::translate::facts::dependencies(value.dependency_facts()),
        label: value.label(),
        flags: crate::translate::queries::flags(value.flags()),
        last_activity: Wall::from_nanos(value.last_activity()),
        made: Wall::from_nanos(value.made()),
        waits_on: crate::translate::queries::waits(value.waits_on()),
        chip: chip_fields(value.chip(), limits)?,
        goal: opt0,
        priority: opt1,
        lineage_chips: chip_piece(value.lineage_chips(), limits, limits.client().rows)?,
        inputs: task_link_piece(value.inputs(), limits, limits.client().rows)?,
        project: value.project(),
        requester: crate::translate::facts::requester(value.requester()),
        lineage: list2,
        spec: specification_document_fields(value.spec(), limits)?,
        spec_revision: crate::translate::answers::revision(value.spec_revision()),
        person_revision: opt3,
        taken_by: opt4,
        instructions: opt5,
        wake: crate::translate::facts::wake_policy(value.wake()),
        subscriptions: subscriptions_fields(value.subscriptions(), limits)?,
        result_contract: task_contract_document_fields(value.result_contract(), limits)?,
        dependencies: task_link_piece(value.dependencies(), limits, limits.client().rows)?,
        references: task_link_piece(value.references(), limits, limits.client().rows)?,
        cut: value.cut(),
        resources: task_resource_piece(value.resources(), limits, limits.client().rows)?,
        budget: crate::translate::facts::budget(value.budget()),
        authority: authority_document_fields(value.authority(), limits)?,
        hold: opt6,
        proposals: list7,
        activity: value.activity(),
        changed: Wall::from_nanos(value.changed()),
        acts: offered_piece(value.acts(), limits, c::MAX_OFFERED_ACTS)?,
        result: opt8,
        delegates: live_delegate_piece(value.delegates(), limits, limits.client().rows)?,
        context: opt9,
        llm: crate::translate::llm::choice(value.llm().as_ref(), limits)?,
        kept_account: *value.kept_account(),
        run_wait: crate::translate::task_wait::waiting(value.run_wait().as_ref()),
    })
}
