//! Wire projections of owner chip facts (client/domain/testing.md, 4).
use super::super::{bytes, c, facts, list, pieces, w};
use super::{
    wire_closing_stage, wire_document_text, wire_hold_category, wire_owner_budget, wire_owner_executor,
    wire_owner_part, wire_owner_requester, wire_owner_task_status, wire_owner_wake_policy, wire_placed_line,
    wire_retry_notice, wire_run_stage,
};
pub(in crate::codec) fn wire_owner_resource_line(
    value: &c::OwnerResourceLine,
    limits: &w::Limits,
) -> Result<w::OwnerResourceLine, w::Problem> {
    w::OwnerResourceLine::new(
        limits,
        w::OwnerResourceLineParts {
            part: wire_owner_part(&value.part, limits)?,
            kind: value.kind,
            taken: value.taken,
            holder: value.holder.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            writer: value.writer.as_ref().map(|item| wire_owner_resource_writer(item, limits)).transpose()?,
            writer_lost: value.writer_lost,
        },
    )
}

pub(in crate::codec) fn wire_owner_resource_writer(
    value: &c::OwnerResourceWriter,
    limits: &w::Limits,
) -> Result<w::OwnerResourceWriter, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerResourceWriter::Run { task, attempt } => w::OwnerResourceWriter::Run(w::OwnerResourceWriterRun::new(
            limits,
            w::OwnerResourceWriterRunParts { task: *(task), attempt: *(attempt) },
        )?),
        c::OwnerResourceWriter::Effect { entry } => w::OwnerResourceWriter::Effect(w::OwnerResourceWriterEffect::new(
            limits,
            w::OwnerResourceWriterEffectParts { entry: *(entry) },
        )?),
    })
}

pub(in crate::codec) fn wire_owner_chip_hold(
    value: &c::OwnerChipHold,
    limits: &w::Limits,
) -> Result<w::OwnerChipHold, w::Problem> {
    w::OwnerChipHold::new(
        limits,
        w::OwnerChipHoldParts {
            account: value.account.as_ref().map(|item| wire_owner_part(item, limits)).transpose()?,
            source: value.source.as_ref().map(|item| wire_owner_part(item, limits)).transpose()?,
            reason: wire_hold_category(&value.reason, limits)?,
            revision: value.revision,
            before: wire_owner_prior_phase(&value.before, limits)?,
            escalation: value.escalation.as_ref().map(|item| wire_owner_hold_escalation(item, limits)).transpose()?,
            tries: wire_owner_failure_tries(&value.tries, limits)?,
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

pub(in crate::codec) fn wire_owner_prior_phase(
    value: &c::OwnerPriorPhase,
    limits: &w::Limits,
) -> Result<w::OwnerPriorPhase, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerPriorPhase::Waiting => w::OwnerPriorPhase::Waiting,
        c::OwnerPriorPhase::Active(value) => w::OwnerPriorPhase::Active(w::OwnerPriorPhaseActive::new(
            limits,
            w::OwnerPriorPhaseActiveParts { value: wire_run_stage(value, limits)? },
        )?),
        c::OwnerPriorPhase::Closing(value) => w::OwnerPriorPhase::Closing(w::OwnerPriorPhaseClosing::new(
            limits,
            w::OwnerPriorPhaseClosingParts { value: wire_closing_stage(value, limits)? },
        )?),
    })
}

pub(in crate::codec) fn wire_owner_hold_escalation(
    value: &c::OwnerHoldEscalation,
    limits: &w::Limits,
) -> Result<w::OwnerHoldEscalation, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerHoldEscalation::Routing => w::OwnerHoldEscalation::Routing,
        c::OwnerHoldEscalation::Waiting { holder, entry, since } => {
            w::OwnerHoldEscalation::Waiting(w::OwnerHoldEscalationWaiting::new(
                limits,
                w::OwnerHoldEscalationWaitingParts {
                    holder: wire_owner_escalation_holder(holder, limits)?,
                    entry: *(entry),
                    since: (since).as_nanos(),
                },
            )?)
        }
        c::OwnerHoldEscalation::Rejected { by, reason } => {
            w::OwnerHoldEscalation::Rejected(w::OwnerHoldEscalationRejected::new(
                limits,
                w::OwnerHoldEscalationRejectedParts { by: *(by), reason: wire_document_text(reason, limits)? },
            )?)
        }
    })
}

pub(in crate::codec) fn wire_owner_escalation_holder(
    value: &c::OwnerEscalationHolder,
    limits: &w::Limits,
) -> Result<w::OwnerEscalationHolder, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerEscalationHolder::Task(value) => w::OwnerEscalationHolder::Task(w::OwnerEscalationHolderTask::new(
            limits,
            w::OwnerEscalationHolderTaskParts { value: *(value) },
        )?),
        c::OwnerEscalationHolder::Person(value) => w::OwnerEscalationHolder::Person(
            w::OwnerEscalationHolderPerson::new(limits, w::OwnerEscalationHolderPersonParts { value: *(value) })?,
        ),
        c::OwnerEscalationHolder::Role { project, role } => {
            w::OwnerEscalationHolder::Role(w::OwnerEscalationHolderRole::new(
                limits,
                w::OwnerEscalationHolderRoleParts { project: *(project), role: *(role) },
            )?)
        }
    })
}

pub(in crate::codec) fn wire_owner_failure_tries(
    value: &c::OwnerFailureTries,
    limits: &w::Limits,
) -> Result<w::OwnerFailureTries, w::Problem> {
    w::OwnerFailureTries::new(
        limits,
        w::OwnerFailureTriesParts {
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
            status: value.status.as_ref().map(|item| wire_owner_task_status(item, limits)).transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_dependency_added(
    value: &c::DependencyAdded,
    limits: &w::Limits,
) -> Result<w::DependencyAdded, w::Problem> {
    w::DependencyAdded::new(
        limits,
        w::DependencyAddedParts { at: value.at.as_nanos(), by: wire_owner_requester(&value.by, limits)? },
    )
}

pub(in crate::codec) fn wire_owner_person_wait(
    value: &c::OwnerPersonWait,
    limits: &w::Limits,
) -> Result<w::OwnerPersonWait, w::Problem> {
    w::OwnerPersonWait::new(
        limits,
        w::OwnerPersonWaitParts {
            since: value.since.as_nanos(),
            stall: value.stall,
            deadline: value.deadline.as_nanos(),
            notice: value.notice.as_ref().map(|item| wire_owner_person_stall_notice(item, limits)).transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_owner_person_stall_notice(
    value: &c::OwnerPersonStallNotice,
    limits: &w::Limits,
) -> Result<w::OwnerPersonStallNotice, w::Problem> {
    w::OwnerPersonStallNotice::new(
        limits,
        w::OwnerPersonStallNoticeParts { message: value.message, at: value.at.as_nanos(), activity: value.activity },
    )
}

pub(in crate::codec) fn wire_owner_chip_observation(
    value: &c::OwnerChipObservation,
    limits: &w::Limits,
) -> Result<w::OwnerChipObservation, w::Problem> {
    w::OwnerChipObservation::new(
        limits,
        w::OwnerChipObservationParts {
            channel: value.channel.raw(),
            work: wire_owner_chip_work(&value.work, limits)?,
            since: value.since.as_nanos(),
            reported_at: value.reported_at.as_nanos(),
            started: value.started.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
        },
    )
}

pub(in crate::codec) fn wire_owner_chip_work(
    value: &c::OwnerChipWork,
    limits: &w::Limits,
) -> Result<w::OwnerChipWork, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerChipWork::Preparing => w::OwnerChipWork::Preparing,
        c::OwnerChipWork::Starting => w::OwnerChipWork::Starting,
        c::OwnerChipWork::Active => w::OwnerChipWork::Active,
        c::OwnerChipWork::Writing => w::OwnerChipWork::Writing,
        c::OwnerChipWork::Tool { call, tool } => w::OwnerChipWork::Tool(w::OwnerChipWorkTool::new(
            limits,
            w::OwnerChipWorkToolParts { call: bytes::copy_of(call), tool: bytes::copy_of(tool) },
        )?),
        c::OwnerChipWork::Waiting => w::OwnerChipWork::Waiting,
        c::OwnerChipWork::Ending => w::OwnerChipWork::Ending,
    })
}

pub(in crate::codec) fn wire_owner_chip_details_fields(
    value: &c::OwnerChipDetails,
    limits: &w::Limits,
    facts: &mut facts::Facts,
    parent: u32,
) -> Result<w::OwnerChipDetailsFields, w::Problem> {
    w::OwnerChipDetailsFields::new(
        limits,
        w::OwnerChipDetailsFieldsParts {
            project: value.project,
            executor: wire_owner_executor(&value.executor, limits)?,
            phase_since: value.phase_since.as_nanos(),
            run_since: value.run_since.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            attempt: value.attempt.as_ref().map(|item| Ok::<_, w::Problem>(*(item))).transpose()?,
            retry: value.retry.as_ref().map(|item| wire_retry_notice(item, limits)).transpose()?,
            wake: wire_owner_wake_policy(&value.wake, limits)?,
            person_wait: value.person_wait.as_ref().map(|item| wire_owner_person_wait(item, limits)).transpose()?,
            hold: value.hold.as_ref().map(|item| wire_owner_chip_hold(item, limits)).transpose()?,
            budget: wire_owner_budget(&value.budget, limits)?,
            activity: value.activity,
            resources: pieces(
                value.resources.iter().map(|item| wire_owner_resource_line(item, limits)),
                w::OwnerResourceLine::measure,
                w::OwnerResourceLine::encode,
            )?,
            resources_cut: value.resources_cut,
            resources_observed: value.resources_observed,
            procedure: value.procedure.as_ref().map(|item| wire_owner_part(item, limits)).transpose()?,
            live_delegates: value.live_delegates,
            closing_connectors: value
                .closing_connectors
                .as_ref()
                .map(|item| list((item).iter().map(|item| Ok::<_, w::Problem>(*(item)))))
                .transpose()?,
            ending: value
                .ending
                .as_ref()
                .map(|item| facts.owner_chip_ending(item, Some(parent), limits))
                .transpose()?,
            ended_at: value.ended_at.as_ref().map(|item| Ok::<_, w::Problem>((item).as_nanos())).transpose()?,
            observation: value
                .observation
                .as_ref()
                .map(|item| wire_owner_chip_observation(item, limits))
                .transpose()?,
            dependencies: list(value.dependencies.iter().map(|item| wire_dependency(item, limits)))?,
        },
    )
}

pub(in crate::codec) fn wire_owner_chip_ending_fields(
    value: &c::OwnerChipEnding,
    limits: &w::Limits,
    facts: &mut facts::Facts,
    parent: u32,
) -> Result<w::OwnerChipEndingFields, w::Problem> {
    Ok::<_, w::Problem>(match value {
        c::OwnerChipEnding::Done(value) => w::OwnerChipEndingFields::Done(w::OwnerChipEndingDoneFields::new(
            limits,
            w::OwnerChipEndingDoneFieldsParts { value: facts.result_value(value, Some(parent), limits)? },
        )?),
        c::OwnerChipEnding::Failed(value) => w::OwnerChipEndingFields::Failed(w::OwnerChipEndingFailedFields::new(
            limits,
            w::OwnerChipEndingFailedFieldsParts { value: wire_document_text(value, limits)? },
        )?),
        c::OwnerChipEnding::Cancelled { reason, result, by, at } => {
            w::OwnerChipEndingFields::Cancelled(w::OwnerChipEndingCancelledFields::new(
                limits,
                w::OwnerChipEndingCancelledFieldsParts {
                    reason: wire_document_text(reason, limits)?,
                    result: (result).as_ref().map(|item| facts.result_value(item, Some(parent), limits)).transpose()?,
                    by: wire_owner_requester(by, limits)?,
                    at: (at).as_nanos(),
                },
            )?)
        }
    })
}
