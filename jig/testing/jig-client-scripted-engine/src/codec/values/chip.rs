//! Wire projections of owner chip facts (client/domain/testing.md, 4).
use super::super::{bytes, c, facts, list, pieces, w};
use super::{
    wire_budget, wire_closing_stage, wire_document_text, wire_executor, wire_hold_category, wire_part,
    wire_placed_line, wire_requester, wire_retry_notice, wire_run_stage, wire_task_status, wire_wake_policy,
};
pub(in crate::codec) fn wire_resource_line(
    value: &c::ResourceLine,
    limits: &w::Limits,
) -> Result<w::ResourceLine, w::Problem> {
    w::ResourceLine::new(
        limits,
        w::ResourceLineParts {
            part: wire_part(&value.part, limits)?,
            kind: value.kind,
            taken: value.taken,
            holder: value.holder.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            writer: value.writer.as_ref().map(|item| wire_resource_writer(item, limits)).transpose()?,
            writer_lost: value.writer_lost,
        },
    )
}

pub(in crate::codec) fn wire_resource_writer(
    value: &c::ResourceWriter,
    limits: &w::Limits,
) -> Result<w::ResourceWriter, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::ResourceWriter::Run { task, attempt } => w::ResourceWriter::Run(w::ResourceWriterRun::new(
            limits,
            w::ResourceWriterRunParts { task: *(task), attempt: *(attempt) },
        )?),
        c::ResourceWriter::Effect { entry } => w::ResourceWriter::Effect(w::ResourceWriterEffect::new(
            limits,
            w::ResourceWriterEffectParts { entry: *(entry) },
        )?),
    })
}

pub(in crate::codec) fn wire_chip_hold(value: &c::ChipHold, limits: &w::Limits) -> Result<w::ChipHold, w::Problem> {
    w::ChipHold::new(
        limits,
        w::ChipHoldParts {
            account: value.account.as_ref().map(|item| wire_part(item, limits)).transpose()?,
            source: value.source.as_ref().map(|item| wire_part(item, limits)).transpose()?,
            reason: wire_hold_category(&value.reason, limits)?,
            revision: value.revision,
            before: wire_prior_phase(&value.before, limits)?,
            escalation: value.escalation.as_ref().map(|item| wire_hold_escalation(item, limits)).transpose()?,
            tries: wire_failure_tries(&value.tries, limits)?,
            hold_wait_since: value
                .hold_wait_since
                .as_ref()
                .map(|item| Ok::<_, w::Problem>((item).as_nanos()))
                .transpose()?,
            circle: value.circle.as_ref().map(|item| wire_circle(item, limits)).transpose()?,
            placed: value.placed.as_ref().map(|item| wire_placed_line(item, limits)).transpose()?,
            words: value.words.as_ref().map(|item| wire_document_text(item, limits)).transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_prior_phase(
    value: &c::PriorPhase,
    limits: &w::Limits,
) -> Result<w::PriorPhase, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::PriorPhase::Waiting => w::PriorPhase::Waiting,
        c::PriorPhase::Active(value) => w::PriorPhase::Active(w::PriorPhaseActive::new(
            limits,
            w::PriorPhaseActiveParts { value: wire_run_stage(value, limits)? },
        )?),
        c::PriorPhase::Closing(value) => w::PriorPhase::Closing(w::PriorPhaseClosing::new(
            limits,
            w::PriorPhaseClosingParts { value: wire_closing_stage(value, limits)? },
        )?),
    })
}

pub(in crate::codec) fn wire_hold_escalation(
    value: &c::HoldEscalation,
    limits: &w::Limits,
) -> Result<w::HoldEscalation, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::HoldEscalation::Routing => w::HoldEscalation::Routing,
        c::HoldEscalation::Waiting { holder, entry, since } => {
            w::HoldEscalation::Waiting(w::HoldEscalationWaiting::new(
                limits,
                w::HoldEscalationWaitingParts {
                    holder: wire_escalation_holder(holder, limits)?,
                    entry: *(entry),
                    since: (since).as_nanos(),
                },
            )?)
        }
        c::HoldEscalation::Rejected { by, reason } => w::HoldEscalation::Rejected(w::HoldEscalationRejected::new(
            limits,
            w::HoldEscalationRejectedParts { by: *(by), reason: wire_document_text(reason, limits)? },
        )?),
    })
}

pub(in crate::codec) fn wire_escalation_holder(
    value: &c::EscalationHolder,
    limits: &w::Limits,
) -> Result<w::EscalationHolder, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::EscalationHolder::Task(value) => w::EscalationHolder::Task(w::EscalationHolderTask::new(
            limits,
            w::EscalationHolderTaskParts { value: *(value) },
        )?),
        c::EscalationHolder::Person(value) => w::EscalationHolder::Person(w::EscalationHolderPerson::new(
            limits,
            w::EscalationHolderPersonParts { value: *(value) },
        )?),
        c::EscalationHolder::Role { project, role } => w::EscalationHolder::Role(w::EscalationHolderRole::new(
            limits,
            w::EscalationHolderRoleParts { project: *(project), role: *(role) },
        )?),
    })
}

pub(in crate::codec) fn wire_failure_tries(
    value: &c::FailureTries,
    limits: &w::Limits,
) -> Result<w::FailureTries, w::Problem> {
    w::FailureTries::new(
        limits,
        w::FailureTriesParts {
            transient: value.transient,
            permanent: value.permanent,
            run: value.run,
            agent: value.agent,
            lost: value.lost,
            invalid: value.invalid,
        },
    )
}

pub(in crate::codec) fn wire_circle(value: &c::Circle, limits: &w::Limits) -> Result<w::Circle, w::Problem> {
    w::Circle::new(
        limits,
        w::CircleParts { tasks: list(value.tasks.iter().map(|item| Ok::<_, w::Problem>(*(item))))?, cut: value.cut },
    )
}

pub(in crate::codec) fn wire_dependency(
    value: &c::Dependency,
    limits: &w::Limits,
) -> Result<w::Dependency, w::Problem> {
    w::Dependency::new(
        limits,
        w::DependencyParts {
            number: value.number,
            pending: value.pending,
            added: value.added.as_ref().map(|item| wire_dependency_added(item, limits)).transpose()?,
            keeps: value.keeps,
            status: value.status.as_ref().map(|item| wire_task_status(item, limits)).transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_dependency_added(
    value: &c::DependencyAdded,
    limits: &w::Limits,
) -> Result<w::DependencyAdded, w::Problem> {
    w::DependencyAdded::new(
        limits,
        w::DependencyAddedParts { at: value.at.as_nanos(), by: wire_requester(&value.by, limits)? },
    )
}

pub(in crate::codec) fn wire_person_wait(
    value: &c::PersonWait,
    limits: &w::Limits,
) -> Result<w::PersonWait, w::Problem> {
    w::PersonWait::new(
        limits,
        w::PersonWaitParts {
            since: value.since.as_nanos(),
            stall: value.stall,
            deadline: value.deadline.as_nanos(),
            notice: value.notice.as_ref().map(|item| wire_person_stall_notice(item, limits)).transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_person_stall_notice(
    value: &c::PersonStallNotice,
    limits: &w::Limits,
) -> Result<w::PersonStallNotice, w::Problem> {
    w::PersonStallNotice::new(
        limits,
        w::PersonStallNoticeParts { message: value.message, at: value.at.as_nanos(), activity: value.activity },
    )
}

pub(in crate::codec) fn wire_chip_observation(
    value: &c::ChipObservation,
    limits: &w::Limits,
) -> Result<w::ChipObservation, w::Problem> {
    w::ChipObservation::new(
        limits,
        w::ChipObservationParts {
            channel: value.channel.raw(),
            work: wire_chip_work(&value.work, limits)?,
            since: value.since.as_nanos(),
            reported_at: value.reported_at.as_nanos(),
            started: value.started.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_chip_work(value: &c::ChipWork, limits: &w::Limits) -> Result<w::ChipWork, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::ChipWork::Preparing => w::ChipWork::Preparing,
        c::ChipWork::Starting => w::ChipWork::Starting,
        c::ChipWork::Active => w::ChipWork::Active,
        c::ChipWork::Writing => w::ChipWork::Writing,
        c::ChipWork::Tool { call, tool } => w::ChipWork::Tool(w::ChipWorkTool::new(
            limits,
            w::ChipWorkToolParts { call: bytes::copy_of(call), tool: bytes::copy_of(tool) },
        )?),
        c::ChipWork::Waiting => w::ChipWork::Waiting,
        c::ChipWork::Ending => w::ChipWork::Ending,
    })
}

pub(in crate::codec) fn wire_chip_details_fields(
    value: &c::ChipDetails,
    limits: &w::Limits,
    facts: &mut facts::Facts,
    parent: u32,
) -> Result<w::ChipDetailsFields, w::Problem> {
    w::ChipDetailsFields::new(
        limits,
        w::ChipDetailsFieldsParts {
            project: value.project,
            executor: wire_executor(&value.executor, limits)?,
            phase_since: value.phase_since.as_nanos(),
            run_since: value.run_since.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            attempt: value.attempt.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            retry: value.retry.as_ref().map(|item| wire_retry_notice(item, limits)).transpose()?,
            wake: wire_wake_policy(&value.wake, limits)?,
            person_wait: value.person_wait.as_ref().map(|item| wire_person_wait(item, limits)).transpose()?,
            hold: value.hold.as_ref().map(|item| wire_chip_hold(item, limits)).transpose()?,
            budget: wire_budget(&value.budget, limits)?,
            activity: value.activity,
            resources: pieces(
                value.resources.iter().map(|item| wire_resource_line(item, limits)),
                w::ResourceLine::measure,
                w::ResourceLine::encode,
            )?,
            resources_cut: value.resources_cut,
            resources_observed: value.resources_observed,
            procedure: value.procedure.as_ref().map(|item| wire_part(item, limits)).transpose()?,
            live_delegates: value.live_delegates,
            closing_connectors: value
                .closing_connectors
                .as_ref()
                .map(|item| list((item).iter().map(|item| Ok::<_, w::Problem>(*(item)))))
                .transpose()?,
            ending: value.ending.as_ref().map(|item| facts.chip_ending(item, Some(parent), limits)).transpose()?,
            ended_at: value.ended_at.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            observation: value.observation.as_ref().map(|item| wire_chip_observation(item, limits)).transpose()?,
            dependencies: list(value.dependencies.iter().map(|item| wire_dependency(item, limits)))?,
        },
    )
}

pub(in crate::codec) fn wire_chip_ending_fields(
    value: &c::ChipEnding,
    limits: &w::Limits,
    facts: &mut facts::Facts,
    parent: u32,
) -> Result<w::ChipEndingFields, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::ChipEnding::Done(value) => w::ChipEndingFields::Done(w::ChipEndingDoneFields::new(
            limits,
            w::ChipEndingDoneFieldsParts { value: facts.result_value(value, Some(parent), limits)? },
        )?),
        c::ChipEnding::Failed(value) => w::ChipEndingFields::Failed(w::ChipEndingFailedFields::new(
            limits,
            w::ChipEndingFailedFieldsParts { value: wire_document_text(value, limits)? },
        )?),
        c::ChipEnding::Cancelled { reason, result, by, at } => {
            w::ChipEndingFields::Cancelled(w::ChipEndingCancelledFields::new(
                limits,
                w::ChipEndingCancelledFieldsParts {
                    reason: wire_document_text(reason, limits)?,
                    result: (result).as_ref().map(|item| facts.result_value(item, Some(parent), limits)).transpose()?,
                    by: wire_requester(by, limits)?,
                    at: (at).as_nanos(),
                },
            )?)
        }
    })
}
