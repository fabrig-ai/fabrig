//! Wire projections of owner chip facts (client/domain/testing.md, 4).
use super::super::{bytes, c, facts, list, pieces, w};
use super::{
    wire_budget, wire_closing_stage, wire_document_text, wire_executor, wire_hold_category, wire_part,
    wire_placed_line, wire_requester, wire_retry_notice, wire_run_stage, wire_task_status, wire_wake_policy,
};
pub(in crate::codec) fn wire_resource_line(value: &c::ResourceLine) -> w::ResourceLine {
    w::ResourceLine {
        part: wire_part(&value.part),
        kind: value.kind,
        taken: value.taken,
        holder: value.holder,
        writer: value.writer.as_ref().map(wire_resource_writer),
        writer_lost: value.writer_lost,
    }
}

pub(in crate::codec) fn wire_resource_writer(value: &c::ResourceWriter) -> w::ResourceWriter {
    match value {
        c::ResourceWriter::Run { task, attempt } => {
            w::ResourceWriter::Run(w::ResourceWriterRun { task: *(task), attempt: *(attempt) })
        }
        c::ResourceWriter::Effect { entry } => w::ResourceWriter::Effect(w::ResourceWriterEffect { entry: *(entry) }),
    }
}

pub(in crate::codec) fn wire_chip_hold(value: &c::ChipHold) -> w::ChipHold {
    w::ChipHold {
        account: value.account.as_ref().map(wire_part),
        source: value.source.as_ref().map(wire_part),
        reason: wire_hold_category(&value.reason),
        revision: value.revision,
        before: wire_prior_phase(&value.before),
        escalation: value.escalation.as_ref().map(wire_hold_escalation),
        tries: wire_failure_tries(&value.tries),
        hold_wait_since: value.hold_wait_since.as_ref().map(|item| (item).as_nanos()),
        circle: value.circle.as_ref().map(wire_circle),
        placed: value.placed.as_ref().map(wire_placed_line),
        words: value.words.as_ref().map(wire_document_text),
    }
}

pub(in crate::codec) fn wire_prior_phase(value: &c::PriorPhase) -> w::PriorPhase {
    match value {
        c::PriorPhase::Waiting => w::PriorPhase::Waiting,
        c::PriorPhase::Active(value) => w::PriorPhase::Active(w::PriorPhaseActive { value: wire_run_stage(value) }),
        c::PriorPhase::Closing(value) => {
            w::PriorPhase::Closing(w::PriorPhaseClosing { value: wire_closing_stage(value) })
        }
    }
}

pub(in crate::codec) fn wire_hold_escalation(value: &c::HoldEscalation) -> w::HoldEscalation {
    match value {
        c::HoldEscalation::Routing => w::HoldEscalation::Routing,
        c::HoldEscalation::Waiting { holder, entry, since } => w::HoldEscalation::Waiting(w::HoldEscalationWaiting {
            holder: wire_escalation_holder(holder),
            entry: *(entry),
            since: (since).as_nanos(),
        }),
        c::HoldEscalation::Rejected { by, reason } => {
            w::HoldEscalation::Rejected(w::HoldEscalationRejected { by: *(by), reason: wire_document_text(reason) })
        }
    }
}

pub(in crate::codec) fn wire_escalation_holder(value: &c::EscalationHolder) -> w::EscalationHolder {
    match value {
        c::EscalationHolder::Task(value) => w::EscalationHolder::Task(w::EscalationHolderTask { value: *(value) }),
        c::EscalationHolder::Person(value) => {
            w::EscalationHolder::Person(w::EscalationHolderPerson { value: *(value) })
        }
        c::EscalationHolder::Role { project, role } => {
            w::EscalationHolder::Role(w::EscalationHolderRole { project: *(project), role: *(role) })
        }
    }
}

pub(in crate::codec) fn wire_failure_tries(value: &c::FailureTries) -> w::FailureTries {
    w::FailureTries {
        transient: value.transient,
        permanent: value.permanent,
        run: value.run,
        agent: value.agent,
        lost: value.lost,
        invalid: value.invalid,
        infrastructure: value.infrastructure,
    }
}

pub(in crate::codec) fn wire_circle(value: &c::Circle) -> w::Circle {
    w::Circle { tasks: list(value.tasks.iter().copied()), cut: value.cut }
}

pub(in crate::codec) fn wire_dependency(value: &c::Dependency) -> w::Dependency {
    w::Dependency {
        number: value.number,
        pending: value.pending,
        added: value.added.as_ref().map(wire_dependency_added),
        keeps: value.keeps,
        status: value.status.as_ref().map(wire_task_status),
    }
}

pub(in crate::codec) fn wire_dependency_added(value: &c::DependencyAdded) -> w::DependencyAdded {
    w::DependencyAdded { at: value.at.as_nanos(), by: wire_requester(&value.by) }
}

pub(in crate::codec) fn wire_person_wait(value: &c::PersonWait) -> w::PersonWait {
    w::PersonWait {
        since: value.since.as_nanos(),
        stall: value.stall,
        deadline: value.deadline.as_nanos(),
        notice: value.notice.as_ref().map(wire_person_stall_notice),
    }
}

pub(in crate::codec) fn wire_person_stall_notice(value: &c::PersonStallNotice) -> w::PersonStallNotice {
    w::PersonStallNotice { message: value.message, at: value.at.as_nanos(), activity: value.activity }
}

pub(in crate::codec) fn wire_chip_observation(value: &c::ChipObservation) -> w::ChipObservation {
    w::ChipObservation {
        channel: value.channel.raw(),
        work: wire_chip_work(&value.work),
        since: value.since.as_nanos(),
        reported_at: value.reported_at.as_nanos(),
        started: value.started.as_ref().map(|item| (item).as_nanos()),
    }
}

pub(in crate::codec) fn wire_chip_work(value: &c::ChipWork) -> w::ChipWork {
    match value {
        c::ChipWork::Preparing => w::ChipWork::Preparing,
        c::ChipWork::Starting => w::ChipWork::Starting,
        c::ChipWork::Active => w::ChipWork::Active,
        c::ChipWork::Writing => w::ChipWork::Writing,
        c::ChipWork::Tool { call, tool } => {
            w::ChipWork::Tool(w::ChipWorkTool { call: bytes::copy_of(call), tool: bytes::copy_of(tool) })
        }
        c::ChipWork::Waiting => w::ChipWork::Waiting,
        c::ChipWork::Ending => w::ChipWork::Ending,
    }
}

pub(in crate::codec) fn wire_chip_details_fields(
    value: &c::ChipDetails,
    limits: &w::Limits,
    facts: &mut facts::Facts,
    parent: u32,
) -> Result<w::ChipDetailsFields, w::Problem> {
    Ok(w::ChipDetailsFields {
        project: value.project,
        executor: wire_executor(&value.executor),
        phase_since: value.phase_since.as_nanos(),
        run_since: value.run_since.as_ref().map(|item| (item).as_nanos()),
        attempt: value.attempt,
        retry: value.retry.as_ref().map(wire_retry_notice),
        wake: wire_wake_policy(&value.wake),
        person_wait: value.person_wait.as_ref().map(wire_person_wait),
        hold: value.hold.as_ref().map(wire_chip_hold),
        budget: wire_budget(&value.budget),
        activity: value.activity,
        resources: pieces(value.resources.iter().map(wire_resource_line), limits)?,
        resources_cut: value.resources_cut,
        resources_observed: value.resources_observed,
        procedure: value.procedure.as_ref().map(wire_part),
        live_delegates: value.live_delegates,
        closing_connectors: value.closing_connectors.as_ref().map(|item| list((item).iter().copied())),
        ending: value.ending.as_ref().map(|item| facts.chip_ending(item, Some(parent), limits)).transpose()?,
        ended_at: value.ended_at.as_ref().map(|item| (item).as_nanos()),
        observation: value.observation.as_ref().map(wire_chip_observation),
        dependencies: list(value.dependencies.iter().map(wire_dependency)),
    })
}

pub(in crate::codec) fn wire_chip_ending_fields(
    value: &c::ChipEnding,
    limits: &w::Limits,
    facts: &mut facts::Facts,
    parent: u32,
) -> Result<w::ChipEndingFields, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::ChipEnding::Done(value) => w::ChipEndingFields::Done(w::ChipEndingDoneFields {
            value: facts.result_value(value, Some(parent), limits)?,
        }),
        c::ChipEnding::Failed(value) => {
            w::ChipEndingFields::Failed(w::ChipEndingFailedFields { value: wire_document_text(value) })
        }
        c::ChipEnding::Cancelled { reason, result, by, at } => {
            w::ChipEndingFields::Cancelled(w::ChipEndingCancelledFields {
                reason: wire_document_text(reason),
                result: (result).as_ref().map(|item| facts.result_value(item, Some(parent), limits)).transpose()?,
                by: wire_requester(by),
                at: (at).as_nanos(),
            })
        }
    })
}
