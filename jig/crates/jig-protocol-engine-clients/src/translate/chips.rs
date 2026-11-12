//! Whole owner facts encoded without clipping (protocol/client/wire.md, 1).
use super::owner;
use super::{pieces, problem, w};
use alloc::boxed::Box;
use jig_domain_core::documents as d;
use jig_domain_tasks as t;
use skein_lib::{List, bytes};
pub fn class(value: t::Class) -> Result<w::HoldFailure, w::Problem> {
    Ok(match value {
        t::Class::Transient => w::HoldFailure::Transient,
        t::Class::Permanent => w::HoldFailure::Permanent,
        t::Class::Run => w::HoldFailure::Run,
        t::Class::Agent => w::HoldFailure::Agent,
        t::Class::Lost => w::HoldFailure::Lost,
        t::Class::Invalid => w::HoldFailure::Invalid,
        t::Class::Infrastructure => w::HoldFailure::Infrastructure,
    })
}
pub fn retry(value: &t::RetryNotice, limits: &w::Limits) -> Result<w::RetryNotice, w::Problem> {
    Ok(match value {
        t::RetryNotice::Failed { class: value, count, allowance, reason } => {
            w::RetryNotice::OwnerFailed(w::RetryNoticeOwnerFailed::new(
                limits,
                w::RetryNoticeOwnerFailedParts {
                    class: class(*value)?,
                    count: *count,
                    allowance: *allowance,
                    reason: reason.clone(),
                },
            )?)
        }
        t::RetryNotice::Refused { count, reason } => w::RetryNotice::Refused(w::RetryNoticeRefused::new(
            limits,
            w::RetryNoticeRefusedParts { count: *count, reason: reason.clone() },
        )?),
    })
}
pub fn hold(value: t::Hold, limits: &w::Limits) -> Result<w::HoldCategory, w::Problem> {
    Ok(match value {
        t::Hold::Exceeded(value) => w::HoldCategory::Exceeded(w::Exceeded::new(
            limits,
            w::ExceededParts {
                bound: match value.bound {
                    t::Bound::Reasoning => w::Bound::Reasoning,
                    t::Bound::Metadata => w::Bound::Metadata,
                    t::Bound::Items => w::Bound::Items,
                    t::Bound::Answer => w::Bound::Answer,
                    t::Bound::Tools => w::Bound::Tools,
                    t::Bound::HistoryItems => w::Bound::HistoryItems,
                    t::Bound::History => w::Bound::History,
                    t::Bound::Window => w::Bound::Window,
                    t::Bound::Output => w::Bound::Output,
                },
                value: value.value,
            },
        )?),
        t::Hold::Failures(value) => w::HoldCategory::Failures(w::HoldCategoryFailures::new(
            limits,
            w::HoldCategoryFailuresParts { class: class(value)? },
        )?),
        t::Hold::Dependency(task) => w::HoldCategory::Dependency(w::HoldCategoryDependency::new(
            limits,
            w::HoldCategoryDependencyParts { task },
        )?),
        t::Hold::Placed { hold } => {
            w::HoldCategory::Placed(w::HoldCategoryPlaced::new(limits, w::HoldCategoryPlacedParts { hold })?)
        }
        t::Hold::Uncertain { entry } => {
            w::HoldCategory::Uncertain(w::HoldCategoryUncertain::new(limits, w::HoldCategoryUncertainParts { entry })?)
        }
        t::Hold::PoolLost { pool } => {
            w::HoldCategory::PoolLost(w::HoldCategoryPoolLost::new(limits, w::HoldCategoryPoolLostParts { pool })?)
        }
        t::Hold::Stalled => w::HoldCategory::Stalled,
        t::Hold::Stopped => w::HoldCategory::Stopped,
        t::Hold::Drift => w::HoldCategory::Drift,
        t::Hold::Effects => w::HoldCategory::Effects,
        t::Hold::EffectFailed => w::HoldCategory::EffectFailed,
        t::Hold::Budget => w::HoldCategory::Budget,
        t::Hold::Deadline => w::HoldCategory::Deadline,
        t::Hold::HoldsWaited => w::HoldCategory::HoldsWaited,
        t::Hold::ResourceUnavailable => w::HoldCategory::ResourceUnavailable,
        t::Hold::ResourceAccess => w::HoldCategory::ResourceAccess,
        t::Hold::Procedure => w::HoldCategory::Procedure,
        t::Hold::Circle => w::HoldCategory::Circle,
    })
}
pub fn active(value: t::Active, limits: &w::Limits) -> Result<w::RunStage, w::Problem> {
    Ok(match value {
        t::Active::Idle => w::RunStage::Idle,
        t::Active::Due => w::RunStage::Due,
        t::Active::Preparing => w::RunStage::Preparing,
        t::Active::Claimed { attempt } => {
            w::RunStage::Claimed(w::RunStageClaimed::new(limits, w::RunStageClaimedParts { attempt })?)
        }
        t::Active::Running { attempt } => {
            w::RunStage::Running(w::RunStageRunning::new(limits, w::RunStageRunningParts { attempt })?)
        }
        t::Active::BackingOff { until } => w::RunStage::BackingOff(w::RunStageBackingOff::new(
            limits,
            w::RunStageBackingOffParts { until: until.as_nanos() },
        )?),
    })
}
pub fn stage(value: t::Stage, limits: &w::Limits) -> Result<w::ClosingStage, w::Problem> {
    Ok(match value {
        t::Stage::Run { attempt } => {
            w::ClosingStage::Run(w::ClosingStageRun::new(limits, w::ClosingStageRunParts { attempt })?)
        }
        t::Stage::Delegates => w::ClosingStage::Delegates,
        t::Stage::Effects => w::ClosingStage::Effects,
        t::Stage::Releases => w::ClosingStage::Releases,
        t::Stage::Settled => w::ClosingStage::Settled,
    })
}
pub fn address(value: t::PersonAddress, limits: &w::Limits) -> Result<w::PersonAddress, w::Problem> {
    Ok(match value {
        t::PersonAddress::Person(number) => {
            w::PersonAddress::Party(w::PersonAddressParty::new(limits, w::PersonAddressPartyParts { value: number })?)
        }
        t::PersonAddress::Role(number) => {
            w::PersonAddress::Role(w::PersonAddressRole::new(limits, w::PersonAddressRoleParts { value: number })?)
        }
    })
}
pub fn phase(value: &d::PhaseLine, limits: &w::Limits) -> Result<w::ChipPhase, w::Problem> {
    Ok(match value {
        d::PhaseLine::Waiting { dependencies, more, holds } => w::ChipPhase::Waiting(w::ChipPhaseWaiting::new(
            limits,
            w::ChipPhaseWaitingParts { dependencies: dependencies.clone(), more: *more, holds: *holds },
        )?),
        d::PhaseLine::Active(value) => w::ChipPhase::Active(w::ChipPhaseActive::new(
            limits,
            w::ChipPhaseActiveParts { value: active(*value, limits)? },
        )?),
        d::PhaseLine::Person { address: value, taken_by, since } => w::ChipPhase::Person(w::ChipPhasePerson::new(
            limits,
            w::ChipPhasePersonParts {
                address: address(*value, limits)?,
                taken_by: *taken_by,
                since: match *since {
                    Some(item) => Some(item.as_nanos()),
                    None => None,
                },
            },
        )?),
        d::PhaseLine::Closing(value) => w::ChipPhase::Closing(w::ChipPhaseClosing::new(
            limits,
            w::ChipPhaseClosingParts { value: stage(*value, limits)? },
        )?),
        d::PhaseLine::Held { reason, revision } => w::ChipPhase::Held(w::ChipPhaseHeld::new(
            limits,
            w::ChipPhaseHeldParts { reason: hold(*reason, limits)?, revision: *revision },
        )?),
        d::PhaseLine::Ended => w::ChipPhase::Ended,
    })
}
pub fn prior(value: d::PriorPhase, limits: &w::Limits) -> Result<w::PriorPhase, w::Problem> {
    Ok(match value {
        d::PriorPhase::Waiting => w::PriorPhase::Waiting,
        d::PriorPhase::Active(value) => w::PriorPhase::Active(w::PriorPhaseActive::new(
            limits,
            w::PriorPhaseActiveParts { value: active(value, limits)? },
        )?),
        d::PriorPhase::Closing(value) => w::PriorPhase::Closing(w::PriorPhaseClosing::new(
            limits,
            w::PriorPhaseClosingParts { value: stage(value, limits)? },
        )?),
    })
}
pub fn holder(value: t::EscalationHolder, limits: &w::Limits) -> Result<w::EscalationHolder, w::Problem> {
    Ok(match value {
        t::EscalationHolder::Task(number) => w::EscalationHolder::Task(w::EscalationHolderTask::new(
            limits,
            w::EscalationHolderTaskParts { value: number },
        )?),
        t::EscalationHolder::Person(number) => w::EscalationHolder::Person(w::EscalationHolderPerson::new(
            limits,
            w::EscalationHolderPersonParts { value: number },
        )?),
        t::EscalationHolder::Role { project, role } => w::EscalationHolder::Role(w::EscalationHolderRole::new(
            limits,
            w::EscalationHolderRoleParts { project, role },
        )?),
    })
}
pub fn escalation(value: &d::EscalationState, limits: &w::Limits) -> Result<w::HoldEscalation, w::Problem> {
    Ok(match value {
        d::EscalationState::Routing => w::HoldEscalation::Routing,
        d::EscalationState::Waiting { holder: value, entry, since } => {
            w::HoldEscalation::Waiting(w::HoldEscalationWaiting::new(
                limits,
                w::HoldEscalationWaitingParts {
                    holder: holder(*value, limits)?,
                    entry: *entry,
                    since: since.as_nanos(),
                },
            )?)
        }
        d::EscalationState::Rejected { by, reason } => w::HoldEscalation::Rejected(w::HoldEscalationRejected::new(
            limits,
            w::HoldEscalationRejectedParts { by: *by, reason: owner::text(reason, limits)? },
        )?),
    })
}
pub fn tries(value: t::Tries, limits: &w::Limits) -> Result<w::FailureTries, w::Problem> {
    w::FailureTries::new(
        limits,
        w::FailureTriesParts {
            transient: value.transient,
            permanent: value.permanent,
            run: value.run,
            agent: value.agent,
            lost: value.lost,
            invalid: value.invalid,
            infrastructure: value.infrastructure },
    )
}
pub fn held(value: &d::HoldLine, limits: &w::Limits) -> Result<w::ChipHold, w::Problem> {
    w::ChipHold::new(
        limits,
        w::ChipHoldParts {
            words: match &value.words {
                Some(text) => Some(owner::text(text, limits)?),
                None => None,
            },
            placed: match &value.placed {
                Some(value) => Some(placed(value, limits)?),
                None => None,
            },
            circle: circle(value.circle.as_ref(), limits)?,
            account: match &value.account {
                Some(item) => Some(owner::part(item, limits)?),
                None => None,
            },
            source: match &value.source {
                Some(item) => Some(owner::part(item, limits)?),
                None => None,
            },
            reason: hold(value.reason, limits)?,
            revision: value.revision,
            before: prior(value.before, limits)?,
            escalation: match &value.escalation {
                Some(item) => Some(escalation(item, limits)?),
                None => None,
            },
            tries: tries(value.tries, limits)?,
            hold_wait_since: match value.hold_wait_since {
                Some(item) => Some(item.as_nanos()),
                None => None,
            },
        },
    )
}
fn placed(value: &d::PlacedLine, limits: &w::Limits) -> Result<w::PlacedLine, w::Problem> {
    w::PlacedLine::new(
        limits,
        w::PlacedLineParts {
            number: value.number,
            owner: super::refusals::hold_owner(value.owner, limits)?,
            mode: match value.mode {
                t::Mode::Stop => w::HoldMode::Stop,
                t::Mode::Drain => w::HoldMode::Drain,
            },
            scope: match value.scope {
                t::Scope::Subtree { task } => {
                    w::HoldScope::Subtree(w::HoldScopeSubtree::new(limits, w::HoldScopeSubtreeParts { task })?)
                }
                t::Scope::Project => w::HoldScope::Project,
            },
            reason: value.reason.clone(),
            placed_at: value.placed_at.as_nanos(),
            revision: value.revision,
            lifted: value.lifted,
            may_lift: value.may_lift,
        },
    )
}
pub fn writer(value: t::Writer, limits: &w::Limits) -> Result<w::ResourceWriter, w::Problem> {
    Ok(match value {
        t::Writer::Run { task, attempt } => {
            w::ResourceWriter::Run(w::ResourceWriterRun::new(limits, w::ResourceWriterRunParts { task, attempt })?)
        }
        t::Writer::Effect { entry } => {
            w::ResourceWriter::Effect(w::ResourceWriterEffect::new(limits, w::ResourceWriterEffectParts { entry })?)
        }
    })
}
pub fn resource(value: &d::ResourceLine, limits: &w::Limits) -> Result<w::ResourceLine, w::Problem> {
    w::ResourceLine::new(
        limits,
        w::ResourceLineParts {
            part: owner::part(&value.part, limits)?,
            kind: value.kind,
            taken: value.taken,
            holder: value.holder,
            writer: match value.writer {
                Some(item) => Some(writer(item, limits)?),
                None => None,
            },
            writer_lost: value.writer_lost,
        },
    )
}
pub fn resources_pieces(rows: &List<w::ResourceLine>, bound: u32, path: w::Path) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = skein_lib::Writer::new(skein_lib::length::usize_of(row.measure()));
        match row.encode(&mut out) {
            Ok(()) => (),
            Err(_) => return Err(problem(skein_codec::Reason::Bound)),
        }
        encoded.push(out.finish()).expect("supplied piece count");
    }
    pieces(&encoded, bound, path)
}
pub fn work(value: &d::ChipWork, limits: &w::Limits) -> Result<w::ChipWork, w::Problem> {
    Ok(match value {
        d::ChipWork::Preparing => w::ChipWork::Preparing,
        d::ChipWork::Starting => w::ChipWork::Starting,
        d::ChipWork::Active => w::ChipWork::Active,
        d::ChipWork::Writing => w::ChipWork::Writing,
        d::ChipWork::Waiting => w::ChipWork::Waiting,
        d::ChipWork::Ending => w::ChipWork::Ending,
        d::ChipWork::Tool { call, tool } => w::ChipWork::Tool(w::ChipWorkTool::new(
            limits,
            w::ChipWorkToolParts { call: bytes::copy_of(call), tool: bytes::copy_of(tool) },
        )?),
    })
}
pub fn observation(value: &d::ChipObservation, limits: &w::Limits) -> Result<w::ChipObservation, w::Problem> {
    w::ChipObservation::new(
        limits,
        w::ChipObservationParts {
            channel: value.channel.raw(),
            work: work(&value.work, limits)?,
            since: value.since.as_nanos(),
            reported_at: value.reported_at.as_nanos(),
            started: match value.started {
                Some(item) => Some(item.as_nanos()),
                None => None,
            },
        },
    )
}
pub fn person_notice(value: t::PersonStallNotice, limits: &w::Limits) -> Result<w::PersonStallNotice, w::Problem> {
    w::PersonStallNotice::new(
        limits,
        w::PersonStallNoticeParts { message: value.message, at: value.at.as_nanos(), activity: value.activity },
    )
}
pub fn person_wait(value: t::PersonWait, limits: &w::Limits) -> Result<w::PersonWait, w::Problem> {
    w::PersonWait::new(
        limits,
        w::PersonWaitParts {
            since: value.since.as_nanos(),
            stall: value.stall,
            deadline: value.deadline.as_nanos(),
            notice: match value.notice {
                Some(item) => Some(person_notice(item, limits)?),
                None => None,
            },
        },
    )
}
fn compact(value: &d::PhaseLine) -> w::Phase {
    match value {
        d::PhaseLine::Waiting { .. } => w::Phase::Waiting,
        d::PhaseLine::Active(value) => match value {
            t::Active::Idle => w::Phase::Idle,
            t::Active::Due => w::Phase::Due,
            t::Active::Preparing => w::Phase::Preparing,
            t::Active::Claimed { .. } => w::Phase::Claimed,
            t::Active::Running { .. } => w::Phase::Running,
            t::Active::BackingOff { .. } => w::Phase::BackingOff,
        },
        d::PhaseLine::Person { .. } => w::Phase::Active,
        d::PhaseLine::Closing(_) => w::Phase::Closing,
        d::PhaseLine::Held { .. } => w::Phase::Held,
        d::PhaseLine::Ended => w::Phase::Ended,
    }
}
fn chip_executor(value: &d::Chip, limits: &w::Limits) -> Result<w::ChipExecutor, w::Problem> {
    Ok(match value.details.executor {
        t::Executor::Job { kind } => {
            if value.executor != d::ExecutorKind::Job {
                return Err(problem(skein_codec::Reason::Malformed));
            }
            w::ChipExecutor::Job(w::ChipExecutorJob::new(limits, w::ChipExecutorJobParts { kind })?)
        }
        t::Executor::Agent { charter } => {
            if value.executor != d::ExecutorKind::Agent {
                return Err(problem(skein_codec::Reason::Malformed));
            }
            w::ChipExecutor::Agent(w::ChipExecutorAgent::new(limits, w::ChipExecutorAgentParts { charter })?)
        }
        t::Executor::Procedure { connector, code } => {
            if value.executor != d::ExecutorKind::Procedure {
                return Err(problem(skein_codec::Reason::Malformed));
            }
            w::ChipExecutor::Procedure(w::ChipExecutorProcedure::new(
                limits,
                w::ChipExecutorProcedureParts { connector, kind: code },
            )?)
        }
        t::Executor::Person(address) => {
            if value.executor != d::ExecutorKind::Person {
                return Err(problem(skein_codec::Reason::Malformed));
            }
            match address {
                t::PersonAddress::Person(party) => {
                    w::ChipExecutor::Party(w::ExecutorParty::new(limits, w::ExecutorPartyParts { party })?)
                }
                t::PersonAddress::Role(role) => w::ChipExecutor::Role(w::ExecutorRole::new(
                    limits,
                    w::ExecutorRoleParts { project: value.details.project, role },
                )?),
            }
        }
    })
}
pub fn chip(value: &d::Chip, limits: &w::Limits) -> Result<w::Chip, w::Problem> {
    let mut facts = List::with_capacity(5);
    facts
        .push(w::Fact::Chip(w::ChipRow::new(
            limits,
            w::ChipRowParts {
                parent: None,
                value: w::ChipFields::new(
                    limits,
                    w::ChipFieldsParts {
                        task: value.number,
                        title: owner::text(&value.title, limits)?,
                        executor: chip_executor(value, limits)?,
                        phase: compact(&value.phase),
                        details: 1,
                    },
                )?,
            },
        )?))
        .expect("chip root");
    facts
        .push(w::Fact::ChipPhase(w::ChipPhaseRow::new(
            limits,
            w::ChipPhaseRowParts {
                parent: Some(0),
                value: w::ChipPhaseOwnerFields::new(
                    limits,
                    w::ChipPhaseOwnerFieldsParts { phase: phase(&value.phase, limits)?, details: 2 },
                )?,
            },
        )?))
        .expect("chip phase");
    let ending = match &value.details.ending {
        Some(_) => Some(3),
        None => None,
    };
    facts
        .push(w::Fact::ChipDetails(Box::new(w::ChipDetailsRow::new(
            limits,
            w::ChipDetailsRowParts { parent: Some(1), value: details_fields(&value.details, ending, limits)? },
        )?)))
        .expect("chip details");
    if let Some(ending) = &value.details.ending {
        append_ending(ending, Some(2), &mut facts, limits)?;
    }
    w::Chip::new(
        limits,
        w::ChipParts { rows: fact_pieces(&facts, limits.chip_rows.min(w::CEILINGS.chip_rows), w::Path::ChipRows)? },
    )
}
fn details_fields(
    value: &d::ChipDetails,
    ending: Option<u32>,
    limits: &w::Limits,
) -> Result<w::ChipDetailsFields, w::Problem> {
    let mut resources = List::with_capacity(value.resources.len());
    for item in &value.resources {
        resources.push(resource(item, limits)?).expect("supplied resource rows");
    }
    w::ChipDetailsFields::new(
        limits,
        w::ChipDetailsFieldsParts {
            dependencies: dependencies(&value.dependencies, limits)?,
            project: value.project,
            executor: owner::executor(value.executor, limits)?,
            phase_since: value.phase_since.as_nanos(),
            run_since: match value.run_since {
                Some(item) => Some(item.as_nanos()),
                None => None,
            },
            attempt: value.attempt,
            retry: match &value.retry {
                Some(item) => Some(retry(item, limits)?),
                None => None,
            },
            wake: owner::wake(value.wake, limits)?,
            person_wait: match value.person_wait {
                Some(item) => Some(person_wait(item, limits)?),
                None => None,
            },
            hold: match &value.hold {
                Some(item) => Some(held(item, limits)?),
                None => None,
            },
            budget: owner::budget(&value.budget, limits)?,
            activity: value.activity,
            resources: resources_pieces(
                &resources,
                limits.chip_details_fields_resources.min(w::CEILINGS.chip_details_fields_resources),
                w::Path::ChipDetailsFieldsResources,
            )?,
            resources_cut: value.resources_cut,
            resources_observed: value.resources_observed,
            procedure: match &value.procedure {
                Some(item) => Some(owner::part(item, limits)?),
                None => None,
            },
            live_delegates: value.live_delegates,
            closing_connectors: value.closing_connectors.clone(),
            ending,
            ended_at: match value.ended_at {
                Some(item) => Some(item.as_nanos()),
                None => None,
            },
            observation: match &value.observation {
                Some(item) => Some(observation(item, limits)?),
                None => None,
            },
        },
    )
}

pub fn ending(value: &d::Ending, limits: &w::Limits) -> Result<w::ChipEnding, w::Problem> {
    let mut facts = List::with_capacity(2);
    append_ending(value, None, &mut facts, limits)?;
    w::ChipEnding::new(
        limits,
        w::ChipEndingParts {
            rows: fact_pieces(
                &facts,
                limits.chip_ending_rows.min(w::CEILINGS.chip_ending_rows),
                w::Path::ChipEndingRows,
            )?,
        },
    )
}
pub(super) fn append_ending(
    value: &d::Ending,
    parent: Option<u32>,
    facts: &mut List<w::Fact>,
    limits: &w::Limits,
) -> Result<(), w::Problem> {
    let index = facts.len();
    let child = index.checked_add(1).ok_or(problem(skein_codec::Reason::Bound))?;
    let fields = match value {
        d::Ending::Done(_) => w::ChipEndingFields::Done(w::ChipEndingDoneFields::new(
            limits,
            w::ChipEndingDoneFieldsParts { value: child },
        )?),
        d::Ending::Failed(value) => w::ChipEndingFields::Failed(w::ChipEndingFailedFields::new(
            limits,
            w::ChipEndingFailedFieldsParts { value: owner::text(value, limits)? },
        )?),
        d::Ending::Cancelled { reason, result, by, at } => {
            w::ChipEndingFields::Cancelled(w::ChipEndingCancelledFields::new(
                limits,
                w::ChipEndingCancelledFieldsParts {
                    reason: owner::text(reason, limits)?,
                    result: match result {
                        Some(_) => Some(child),
                        None => None,
                    },
                    by: owner::requester(*by, limits)?,
                    at: at.as_nanos(),
                },
            )?)
        }
    };
    facts
        .push(w::Fact::ChipEnding(w::ChipEndingRow::new(limits, w::ChipEndingRowParts { parent, value: fields })?))
        .expect("bounded ending facts");
    let result = match value {
        d::Ending::Done(value) => Some(value),
        d::Ending::Failed(_) => None,
        d::Ending::Cancelled { result, .. } => result.as_ref(),
    };
    if let Some(value) = result {
        facts
            .push(w::Fact::ResultValue(w::ResultValueRow::new(
                limits,
                w::ResultValueRowParts { parent: Some(index), value: result_fields(value, limits)? },
            )?))
            .expect("bounded result facts");
    }
    Ok(())
}
pub(super) fn result_fields(value: &d::ResultValue, limits: &w::Limits) -> Result<w::ResultValueFields, w::Problem> {
    Ok(match value {
        d::ResultValue::Approved { state, words } => w::ResultValueFields::Approved(w::ResultValueApprovedFields::new(
            limits,
            w::ResultValueApprovedFieldsParts {
                state: bytes::copy_of(state),
                words: match words {
                    Some(item) => Some(owner::text(item, limits)?),
                    None => None,
                },
            },
        )?),
        d::ResultValue::Report(value) => w::ResultValueFields::Report(w::ResultValueReportFields::new(
            limits,
            w::ResultValueReportFieldsParts { value: owner::text(value, limits)? },
        )?),
        d::ResultValue::Failure(value) => w::ResultValueFields::Failure(w::ResultValueFailureFields::new(
            limits,
            w::ResultValueFailureFieldsParts { value: owner::text(value, limits)? },
        )?),
        d::ResultValue::Change { kind, part, words } => w::ResultValueFields::Change(w::ResultValueChangeFields::new(
            limits,
            w::ResultValueChangeFieldsParts {
                kind: *kind,
                part: owner::part(part, limits)?,
                words: owner::text(words, limits)?,
            },
        )?),
        d::ResultValue::Verdict { code, fields, words } => {
            let mut copied = List::with_capacity(super::count(fields.len())?);
            for item in fields {
                copied
                    .push(w::ResultField::new(
                        limits,
                        w::ResultFieldParts { name: bytes::copy_of(&item.name), value: bytes::copy_of(&item.value) },
                    )?)
                    .expect("supplied result fields");
            }
            w::ResultValueFields::OwnerVerdict(w::ResultValueOwnerVerdictFields::new(
                limits,
                w::ResultValueOwnerVerdictFieldsParts {
                    verdict: *code,
                    words: owner::text(words, limits)?,
                    fields: copied,
                },
            )?)
        }
    })
}

pub fn fact_pieces(rows: &List<w::Fact>, bound: u32, path: w::Path) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = skein_lib::Writer::new(skein_lib::length::usize_of(row.measure()));
        match row.encode(&mut out) {
            Ok(()) => (),
            Err(_) => return Err(problem(skein_codec::Reason::Bound)),
        }
        encoded.push(out.finish()).expect("supplied piece count");
    }
    pieces(&encoded, bound, path)
}

fn circle(value: Option<&t::Circle>, limits: &w::Limits) -> Result<Option<w::Circle>, w::Problem> {
    let Some(value) = value else { return Ok(None) };
    let mut tasks = List::with_capacity(u32::try_from(value.tasks.len()).expect("bounded circle"));
    for task in &value.tasks {
        tasks.push(*task).expect("same circle");
    }
    Ok(Some(w::Circle::new(limits, w::CircleParts { tasks, cut: value.cut })?))
}
fn status(value: t::Status) -> w::TaskStatus {
    match value {
        t::Status::Done => w::TaskStatus::Done,
        t::Status::Failed => w::TaskStatus::Failed,
        t::Status::Cancelled => w::TaskStatus::Cancelled,
    }
}
pub(crate) fn dependencies(
    values: &List<d::Dependency>,
    limits: &w::Limits,
) -> Result<List<w::Dependency>, w::Problem> {
    let mut rows = List::with_capacity(values.len());
    for value in values {
        let added = match &value.added {
            Some(added) => Some(w::DependencyAdded::new(
                limits,
                w::DependencyAddedParts { at: added.at.as_nanos(), by: owner::requester(added.by, limits)? },
            )?),
            None => None,
        };
        rows.push(w::Dependency::new(
            limits,
            w::DependencyParts {
                number: value.number,
                pending: value.pending,
                added,
                keeps: value.keeps,
                status: match value.status {
                    Some(value) => Some(status(value)),
                    None => None,
                },
            },
        )?)
        .expect("same dependency count");
    }
    Ok(rows)
}
