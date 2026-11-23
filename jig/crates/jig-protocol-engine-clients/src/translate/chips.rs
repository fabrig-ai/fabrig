//! Whole owner facts encoded without clipping (protocol/client/wire.md, 1).
use super::owner;
use super::{pieces, problem, w};
use alloc::boxed::Box;
use jig_client_protocol_wire::Record as _;
use jig_domain_core_vocabulary::documents as d;
use jig_domain_tasks as t;
use skein_lib::{List, bytes};
#[must_use]
pub fn class(value: t::Class) -> w::HoldFailure {
    match value {
        t::Class::Transient => w::HoldFailure::Transient,
        t::Class::Permanent => w::HoldFailure::Permanent,
        t::Class::Run => w::HoldFailure::Run,
        t::Class::Agent => w::HoldFailure::Agent,
        t::Class::Lost => w::HoldFailure::Lost,
        t::Class::Invalid => w::HoldFailure::Invalid,
        t::Class::Infrastructure => w::HoldFailure::Infrastructure,
    }
}
#[must_use]
pub fn retry(value: &t::RetryNotice) -> w::RetryNotice {
    match value {
        t::RetryNotice::Failed { class: value, count, allowance, reason } => {
            w::RetryNotice::OwnerFailed(w::RetryNoticeOwnerFailed {
                class: class(*value),
                count: *count,
                allowance: *allowance,
                reason: reason.clone(),
            })
        }
        t::RetryNotice::Refused { count, reason } => {
            w::RetryNotice::Refused(w::RetryNoticeRefused { count: *count, reason: reason.clone() })
        }
    }
}
#[must_use]
pub fn hold(value: t::Hold) -> w::HoldCategory {
    match value {
        t::Hold::Exceeded(value) => w::HoldCategory::Exceeded(w::Exceeded {
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
        }),
        t::Hold::Failures(value) => w::HoldCategory::Failures(w::HoldCategoryFailures { class: class(value) }),
        t::Hold::Dependency(task) => w::HoldCategory::Dependency(w::HoldCategoryDependency { task }),
        t::Hold::Placed { hold } => w::HoldCategory::Placed(w::HoldCategoryPlaced { hold }),
        t::Hold::Uncertain { entry } => w::HoldCategory::Uncertain(w::HoldCategoryUncertain { entry }),
        t::Hold::PoolLost { pool } => w::HoldCategory::PoolLost(w::HoldCategoryPoolLost { pool }),
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
    }
}
#[must_use]
pub fn active(value: t::Active) -> w::RunStage {
    match value {
        t::Active::Idle => w::RunStage::Idle,
        t::Active::Due => w::RunStage::Due,
        t::Active::Preparing => w::RunStage::Preparing,
        t::Active::Claimed { attempt } => w::RunStage::Claimed(w::RunStageClaimed { attempt }),
        t::Active::Running { attempt } => w::RunStage::Running(w::RunStageRunning { attempt }),
        t::Active::BackingOff { until } => w::RunStage::BackingOff(w::RunStageBackingOff { until: until.as_nanos() }),
    }
}
#[must_use]
pub fn stage(value: t::Stage) -> w::ClosingStage {
    match value {
        t::Stage::Run { attempt } => w::ClosingStage::Run(w::ClosingStageRun { attempt }),
        t::Stage::Delegates => w::ClosingStage::Delegates,
        t::Stage::Effects => w::ClosingStage::Effects,
        t::Stage::Releases => w::ClosingStage::Releases,
        t::Stage::Settled => w::ClosingStage::Settled,
    }
}
#[must_use]
pub fn address(value: t::PersonAddress) -> w::PersonAddress {
    match value {
        t::PersonAddress::Person(number) => w::PersonAddress::Party(w::PersonAddressParty { value: number }),
        t::PersonAddress::Role(number) => w::PersonAddress::Role(w::PersonAddressRole { value: number }),
    }
}
#[must_use]
pub fn phase(value: &d::PhaseLine) -> w::ChipPhase {
    match value {
        d::PhaseLine::Waiting { dependencies, more, holds } => w::ChipPhase::Waiting(w::ChipPhaseWaiting {
            dependencies: dependencies.clone(),
            more: *more,
            holds: *holds,
        }),
        d::PhaseLine::Active(value) => w::ChipPhase::Active(w::ChipPhaseActive { value: active(*value) }),
        d::PhaseLine::Person { address: value, taken_by, since } => w::ChipPhase::Person(w::ChipPhasePerson {
            address: address(*value),
            taken_by: *taken_by,
            since: match *since {
                Some(item) => Some(item.as_nanos()),
                None => None,
            },
        }),
        d::PhaseLine::Closing(value) => w::ChipPhase::Closing(w::ChipPhaseClosing { value: stage(*value) }),
        d::PhaseLine::Held { reason, revision } => {
            w::ChipPhase::Held(w::ChipPhaseHeld { reason: hold(*reason), revision: *revision })
        }
        d::PhaseLine::Ended => w::ChipPhase::Ended,
    }
}
#[must_use]
pub fn prior(value: d::PriorPhase) -> w::PriorPhase {
    match value {
        d::PriorPhase::Waiting => w::PriorPhase::Waiting,
        d::PriorPhase::Active(value) => w::PriorPhase::Active(w::PriorPhaseActive { value: active(value) }),
        d::PriorPhase::Closing(value) => w::PriorPhase::Closing(w::PriorPhaseClosing { value: stage(value) }),
    }
}
#[must_use]
pub fn holder(value: t::EscalationHolder) -> w::EscalationHolder {
    match value {
        t::EscalationHolder::Task(number) => w::EscalationHolder::Task(w::EscalationHolderTask { value: number }),
        t::EscalationHolder::Person(number) => w::EscalationHolder::Person(w::EscalationHolderPerson { value: number }),
        t::EscalationHolder::Role { project, role } => {
            w::EscalationHolder::Role(w::EscalationHolderRole { project, role })
        }
    }
}
#[expect(clippy::match_same_arms, reason = "the interim wire projects each decider kind to its number")]
#[must_use]
pub fn escalation(value: &d::EscalationState) -> w::HoldEscalation {
    match value {
        d::EscalationState::Routing => w::HoldEscalation::Routing,
        d::EscalationState::Waiting { holder: value, entry, since } => {
            w::HoldEscalation::Waiting(w::HoldEscalationWaiting {
                holder: holder(*value),
                entry: *entry,
                since: since.as_nanos(),
            })
        }
        d::EscalationState::Rejected { by, reason } => w::HoldEscalation::Rejected(w::HoldEscalationRejected {
            by: match by {
                t::EscalationDecider::Task(number) => *number,
                t::EscalationDecider::Person(number) => *number,
            },
            reason: owner::text(reason),
        }),
    }
}
#[must_use]
pub fn tries(value: t::Tries) -> w::FailureTries {
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
#[must_use]
pub fn held(value: &d::HoldPhaseLine) -> w::ChipHold {
    w::ChipHold {
        words: match &value.words {
            Some(text) => Some(owner::text(text)),
            None => None,
        },
        placed: match &value.placed {
            Some(value) => Some(placed(value)),
            None => None,
        },
        circle: circle(value.circle.as_ref()),
        account: match &value.account {
            Some(item) => Some(owner::part(item)),
            None => None,
        },
        source: match &value.source {
            Some(item) => Some(owner::part(item)),
            None => None,
        },
        reason: hold(value.reason),
        revision: value.revision,
        before: prior(value.before),
        escalation: match &value.escalation {
            Some(item) => Some(escalation(item)),
            None => None,
        },
        tries: tries(value.tries),
        hold_wait_since: match value.hold_wait_since {
            Some(item) => Some(item.as_nanos()),
            None => None,
        },
    }
}
fn placed(value: &d::PlacedLine) -> w::PlacedLine {
    w::PlacedLine {
        number: value.number,
        owner: super::refusals::hold_owner(value.owner),
        mode: match value.mode {
            t::Mode::Stop => w::HoldMode::Stop,
            t::Mode::Drain => w::HoldMode::Drain,
        },
        scope: match value.scope {
            t::Scope::Subtree { task } => w::HoldScope::Subtree(w::HoldScopeSubtree { task }),
            t::Scope::Project => w::HoldScope::Project,
        },
        reason: value.reason.clone(),
        placed_at: value.placed_at.as_nanos(),
        revision: value.revision,
        lifted: value.lifted,
        may_lift: value.may_lift,
    }
}
#[must_use]
pub fn writer(value: t::Writer) -> w::ResourceWriter {
    match value {
        t::Writer::Run { task, attempt } => w::ResourceWriter::Run(w::ResourceWriterRun { task, attempt }),
        t::Writer::Effect { entry } => w::ResourceWriter::Effect(w::ResourceWriterEffect { entry }),
    }
}
#[must_use]
pub fn resource(value: &d::ResourceLine) -> w::ResourceLine {
    w::ResourceLine {
        part: owner::part(&value.part),
        kind: value.kind,
        taken: value.taken,
        holder: value.holder,
        writer: match value.writer {
            Some(item) => Some(writer(item)),
            None => None,
        },
        writer_lost: value.writer_lost,
    }
}
pub fn resources_pieces(
    rows: &List<w::ResourceLine>,
    bound: u32,
    path: w::Path,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = skein_lib::Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        match row.write(limits, &mut out) {
            Ok(()) => (),
            Err(_) => return Err(problem(skein_codec::Reason::Bound)),
        }
        encoded.push(out.finish()).expect("supplied piece count");
    }
    pieces(&encoded, bound, path)
}
#[must_use]
pub fn work(value: &d::ChipWork) -> w::ChipWork {
    match value {
        d::ChipWork::Preparing => w::ChipWork::Preparing,
        d::ChipWork::Starting => w::ChipWork::Starting,
        d::ChipWork::Active => w::ChipWork::Active,
        d::ChipWork::Writing => w::ChipWork::Writing,
        d::ChipWork::Waiting => w::ChipWork::Waiting,
        d::ChipWork::Ending => w::ChipWork::Ending,
        d::ChipWork::Tool { call, tool } => {
            w::ChipWork::Tool(w::ChipWorkTool { call: bytes::copy_of(call), tool: bytes::copy_of(tool) })
        }
    }
}
#[must_use]
pub fn observation(value: &d::ChipObservation) -> w::ChipObservation {
    w::ChipObservation {
        channel: value.channel.raw(),
        work: work(&value.work),
        since: value.since.as_nanos(),
        reported_at: value.reported_at.as_nanos(),
        started: match value.started {
            Some(item) => Some(item.as_nanos()),
            None => None,
        },
    }
}
#[must_use]
pub fn person_notice(value: t::PersonStallNotice) -> w::PersonStallNotice {
    w::PersonStallNotice { message: value.message, at: value.at.as_nanos(), activity: value.activity }
}
#[must_use]
pub fn person_wait(value: t::PersonWait) -> w::PersonWait {
    w::PersonWait {
        since: value.since.as_nanos(),
        stall: value.stall,
        deadline: value.deadline.as_nanos(),
        notice: match value.notice {
            Some(item) => Some(person_notice(item)),
            None => None,
        },
    }
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
fn chip_executor(value: &d::Chip) -> Result<w::ChipExecutor, w::Problem> {
    Ok(match value.details.executor {
        t::Executor::Job { kind } => {
            if value.executor != d::ExecutorKind::Job {
                return Err(problem(skein_codec::Reason::Malformed));
            }
            w::ChipExecutor::Job(w::ChipExecutorJob { kind })
        }
        t::Executor::Agent { charter } => {
            if value.executor != d::ExecutorKind::Agent {
                return Err(problem(skein_codec::Reason::Malformed));
            }
            w::ChipExecutor::Agent(w::ChipExecutorAgent { charter })
        }
        t::Executor::Procedure { connector, code } => {
            if value.executor != d::ExecutorKind::Procedure {
                return Err(problem(skein_codec::Reason::Malformed));
            }
            w::ChipExecutor::Procedure(w::ChipExecutorProcedure { connector, kind: code })
        }
        t::Executor::Person(address) => {
            if value.executor != d::ExecutorKind::Person {
                return Err(problem(skein_codec::Reason::Malformed));
            }
            match address {
                t::PersonAddress::Person(party) => w::ChipExecutor::Party(w::ExecutorParty { party }),
                t::PersonAddress::Role(role) => {
                    w::ChipExecutor::Role(w::ExecutorRole { project: value.details.project, role })
                }
            }
        }
    })
}
pub fn chip(value: &d::Chip, limits: &w::Limits) -> Result<w::Chip, w::Problem> {
    let mut facts = List::with_capacity(5);
    facts
        .push(w::Fact::Chip(w::ChipRow {
            parent: None,
            value: w::ChipFields {
                task: value.number,
                title: owner::text(&value.title),
                executor: chip_executor(value)?,
                phase: compact(&value.phase),
                details: 1,
            },
        }))
        .expect("chip root");
    facts
        .push(w::Fact::ChipPhase(w::ChipPhaseRow {
            parent: Some(0),
            value: w::ChipPhaseOwnerFields { phase: phase(&value.phase), details: 2 },
        }))
        .expect("chip phase");
    let ending = match &value.details.ending {
        Some(_) => Some(3),
        None => None,
    };
    facts
        .push(w::Fact::ChipDetails(Box::new(w::ChipDetailsRow {
            parent: Some(1),
            value: details_fields(&value.details, ending, limits)?,
        })))
        .expect("chip details");
    if let Some(ending) = &value.details.ending {
        append_ending(ending, Some(2), &mut facts)?;
    }
    Ok(w::Chip { rows: fact_pieces(&facts, limits.chip_rows.min(w::CEILINGS.chip_rows), w::paths::CHIP_ROWS, limits)? })
}
fn details_fields(
    value: &d::ChipDetails,
    ending: Option<u32>,
    limits: &w::Limits,
) -> Result<w::ChipDetailsFields, w::Problem> {
    let mut resources = List::with_capacity(value.resources.len());
    for item in &value.resources {
        resources.push(resource(item)).expect("supplied resource rows");
    }
    Ok(w::ChipDetailsFields {
        dependencies: dependencies(&value.dependencies),
        project: value.project,
        executor: owner::executor(value.executor),
        phase_since: value.phase_since.as_nanos(),
        run_since: match value.run_since {
            Some(item) => Some(item.as_nanos()),
            None => None,
        },
        attempt: value.attempt,
        retry: match &value.retry {
            Some(item) => Some(retry(item)),
            None => None,
        },
        wake: owner::wake(value.wake),
        person_wait: match value.person_wait {
            Some(item) => Some(person_wait(item)),
            None => None,
        },
        hold: match &value.hold {
            Some(item) => Some(held(item)),
            None => None,
        },
        budget: owner::budget(&value.budget),
        activity: value.activity,
        resources: resources_pieces(
            &resources,
            limits.chip_details_fields_resources.min(w::CEILINGS.chip_details_fields_resources),
            w::paths::CHIP_DETAILS_FIELDS_RESOURCES,
            limits,
        )?,
        resources_cut: value.resources_cut,
        resources_observed: value.resources_observed,
        procedure: match &value.procedure {
            Some(item) => Some(owner::part(item)),
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
            Some(item) => Some(observation(item)),
            None => None,
        },
    })
}

pub fn ending(value: &d::Ending, limits: &w::Limits) -> Result<w::ChipEnding, w::Problem> {
    let mut facts = List::with_capacity(2);
    append_ending(value, None, &mut facts)?;
    Ok(w::ChipEnding {
        rows: fact_pieces(
            &facts,
            limits.chip_ending_rows.min(w::CEILINGS.chip_ending_rows),
            w::paths::CHIP_ENDING_ROWS,
            limits,
        )?,
    })
}
pub(super) fn append_ending(
    value: &d::Ending,
    parent: Option<u32>,
    facts: &mut List<w::Fact>,
) -> Result<(), w::Problem> {
    let index = facts.len();
    let child = index.checked_add(1).ok_or(problem(skein_codec::Reason::Bound))?;
    let fields = match value {
        d::Ending::Done(_) => w::ChipEndingFields::Done(w::ChipEndingDoneFields { value: child }),
        d::Ending::Failed(value) => {
            w::ChipEndingFields::Failed(w::ChipEndingFailedFields { value: owner::text(value) })
        }
        d::Ending::Cancelled { reason, result, by, at } => {
            w::ChipEndingFields::Cancelled(w::ChipEndingCancelledFields {
                reason: owner::text(reason),
                result: match result {
                    Some(_) => Some(child),
                    None => None,
                },
                by: owner::requester(*by),
                at: at.as_nanos(),
            })
        }
    };
    facts.push(w::Fact::ChipEnding(w::ChipEndingRow { parent, value: fields })).expect("bounded ending facts");
    let result = match value {
        d::Ending::Done(value) => Some(value),
        d::Ending::Failed(_) => None,
        d::Ending::Cancelled { result, .. } => result.as_ref(),
    };
    if let Some(value) = result {
        facts
            .push(w::Fact::ResultValue(w::ResultValueRow { parent: Some(index), value: result_fields(value)? }))
            .expect("bounded result facts");
    }
    Ok(())
}
pub(super) fn result_fields(value: &d::ResultValue) -> Result<w::ResultValueFields, w::Problem> {
    Ok(match value {
        d::ResultValue::Approved { state, words } => w::ResultValueFields::Approved(w::ResultValueApprovedFields {
            state: bytes::copy_of(state),
            words: match words {
                Some(item) => Some(owner::text(item)),
                None => None,
            },
        }),
        d::ResultValue::Report(value) => {
            w::ResultValueFields::Report(w::ResultValueReportFields { value: owner::text(value) })
        }
        d::ResultValue::Failure(value) => {
            w::ResultValueFields::Failure(w::ResultValueFailureFields { value: owner::text(value) })
        }
        d::ResultValue::Change { kind, part, words } => w::ResultValueFields::Change(w::ResultValueChangeFields {
            kind: *kind,
            part: owner::part(part),
            words: owner::text(words),
        }),
        d::ResultValue::Verdict { code, fields, words } => {
            let mut copied = List::with_capacity(super::count(fields.len())?);
            for item in fields {
                copied
                    .push(w::ResultField { name: bytes::copy_of(&item.name), value: bytes::copy_of(&item.value) })
                    .expect("supplied result fields");
            }
            w::ResultValueFields::OwnerVerdict(w::ResultValueOwnerVerdictFields {
                verdict: *code,
                words: owner::text(words),
                fields: copied,
            })
        }
    })
}

pub fn fact_pieces(
    rows: &List<w::Fact>,
    bound: u32,
    path: w::Path,
    limits: &w::Limits,
) -> Result<Box<[u8]>, w::Problem> {
    let mut encoded = List::with_capacity(rows.len());
    for row in rows {
        let mut out = skein_lib::Writer::new(skein_lib::length::usize_of(row.measure(limits)?));
        match row.write(limits, &mut out) {
            Ok(()) => (),
            Err(_) => return Err(problem(skein_codec::Reason::Bound)),
        }
        encoded.push(out.finish()).expect("supplied piece count");
    }
    pieces(&encoded, bound, path)
}

fn circle(value: Option<&t::Circle>) -> Option<w::Circle> {
    let value = value?;
    let mut tasks = List::with_capacity(u32::try_from(value.tasks.len()).expect("bounded circle"));
    for task in &value.tasks {
        tasks.push(*task).expect("same circle");
    }
    Some(w::Circle { tasks, cut: value.cut })
}
fn status(value: t::Status) -> w::TaskStatus {
    match value {
        t::Status::Done => w::TaskStatus::Done,
        t::Status::Failed => w::TaskStatus::Failed,
        t::Status::Cancelled => w::TaskStatus::Cancelled,
    }
}
pub(crate) fn dependencies(values: &List<d::Dependency>) -> List<w::Dependency> {
    let mut rows = List::with_capacity(values.len());
    for value in values {
        let added = match &value.added {
            Some(added) => Some(w::DependencyAdded { at: added.at.as_nanos(), by: owner::requester(added.by) }),
            None => None,
        };
        rows.push(w::Dependency {
            number: value.number,
            pending: value.pending,
            added,
            keeps: value.keeps,
            status: match value.status {
                Some(value) => Some(status(value)),
                None => None,
            },
        })
        .expect("same dependency count");
    }
    rows
}
