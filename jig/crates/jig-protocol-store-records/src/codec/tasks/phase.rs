//! Task lifecycle values and bounded creation templates (protocol/store/records.md, 1).
use super::value;
use crate::tasks_v13 as wire;
use alloc::boxed::Box;
use jig_domain_tasks as domain;
use skein_lib::{List, Wall};

/// Translate `Holding` to its store value.
pub fn holding(source: &domain::Holding, limits: &wire::Limits) -> Result<wire::Holding, wire::Problem> {
    Ok(match source {
        domain::Holding::Write { resource, kind } => {
            wire::Holding::Write(wire::WriteHolding { resource: value::name(resource, limits)?, kind: *kind })
        }
        domain::Holding::Slot { pool, kind } => {
            wire::Holding::Slot(wire::SlotHolding { pool: value::name(pool, limits)?, kind: *kind })
        }
    })
}
/// Read `Holding` from its store value.
pub fn read_holding(source: wire::Holding, limits: &wire::Limits) -> Result<domain::Holding, wire::Problem> {
    Ok(match source {
        wire::Holding::Write(value) => {
            let wire::WriteHolding { resource, kind } = value;
            domain::Holding::Write { resource: value::read_name(resource, limits)?, kind }
        }
        wire::Holding::Slot(value) => {
            let wire::SlotHolding { pool, kind } = value;
            domain::Holding::Slot { pool: value::read_name(pool, limits)?, kind }
        }
    })
}

/// Translate `WakeRule` to its store value.
pub fn wake_rule(source: &domain::WakeRule, _limits: &wire::Limits) -> Result<wire::WakeRule, wire::Problem> {
    Ok(match source {
        domain::WakeRule::Never => wire::WakeRule::Never,
        domain::WakeRule::Immediate => wire::WakeRule::Immediate,
        domain::WakeRule::Batch { count, age } => wire::WakeRule::Batch(wire::WakeBatch { count: *count, age: *age }),
        domain::WakeRule::Paced { span } => wire::WakeRule::Paced(wire::WakePace { span: *span }),
    })
}
/// Read `WakeRule` from its store value.
pub fn read_wake_rule(source: wire::WakeRule) -> Result<domain::WakeRule, wire::Problem> {
    Ok(match source {
        wire::WakeRule::Never => domain::WakeRule::Never,
        wire::WakeRule::Immediate => domain::WakeRule::Immediate,
        wire::WakeRule::Batch(value) => {
            let wire::WakeBatch { count, age } = value;
            domain::WakeRule::Batch { count, age }
        }
        wire::WakeRule::Paced(value) => domain::WakeRule::Paced { span: value.span },
    })
}

/// Translate `ResultsWake` to its store value.
pub fn results_wake(source: &domain::ResultsWake) -> Result<wire::ResultsWake, wire::Problem> {
    Ok(match source {
        domain::ResultsWake::Never => wire::ResultsWake::Never,
        domain::ResultsWake::Each => wire::ResultsWake::Each,
        domain::ResultsWake::LastOrFailure => wire::ResultsWake::LastOrFailure,
    })
}
/// Read `ResultsWake` from its store value.
pub fn read_results_wake(source: wire::ResultsWake) -> Result<domain::ResultsWake, wire::Problem> {
    Ok(match source {
        wire::ResultsWake::Never => domain::ResultsWake::Never,
        wire::ResultsWake::Each => domain::ResultsWake::Each,
        wire::ResultsWake::LastOrFailure => domain::ResultsWake::LastOrFailure,
    })
}

/// Translate `WakePolicy` to its store record.
pub fn wake(source: &domain::WakePolicy, limits: &wire::Limits) -> Result<wire::WakePolicy, wire::Problem> {
    let domain::WakePolicy { words, notices, news, results, questions, answers, timers } = source;
    Ok(wire::WakePolicy {
        words: wake_rule(words, limits)?,
        notices: wake_rule(notices, limits)?,
        news: wake_rule(news, limits)?,
        results: results_wake(results)?,
        questions: *questions,
        answers: *answers,
        timers: *timers,
    })
}
/// Read `WakePolicy` from its store record.
pub fn read_wake(source: wire::WakePolicy) -> Result<domain::WakePolicy, wire::Problem> {
    let wire::WakePolicy { words, notices, news, results, questions, answers, timers } = source;
    Ok(domain::WakePolicy {
        words: read_wake_rule(words)?,
        notices: read_wake_rule(notices)?,
        news: read_wake_rule(news)?,
        results: read_results_wake(results)?,
        questions,
        answers,
        timers,
    })
}

/// Translate `RecurringOverlap` to its store value.
pub fn overlap(source: &domain::RecurringOverlap) -> Result<wire::RecurringOverlap, wire::Problem> {
    Ok(match source {
        domain::RecurringOverlap::Skip => wire::RecurringOverlap::Skip,
        domain::RecurringOverlap::Wait => wire::RecurringOverlap::Wait,
    })
}
/// Read `RecurringOverlap` from its store value.
pub fn read_overlap(source: wire::RecurringOverlap) -> Result<domain::RecurringOverlap, wire::Problem> {
    Ok(match source {
        wire::RecurringOverlap::Skip => domain::RecurringOverlap::Skip,
        wire::RecurringOverlap::Wait => domain::RecurringOverlap::Wait,
    })
}

/// Translate `Hold` to its store value.
pub fn hold(source: &domain::Hold, _limits: &wire::Limits) -> Result<wire::Hold, wire::Problem> {
    Ok(match source {
        domain::Hold::Exceeded(value) => wire::Hold::Exceeded(wire::Exceeded {
            bound: match value.bound {
                domain::Bound::Reasoning => wire::Bound::Reasoning,
                domain::Bound::Metadata => wire::Bound::Metadata,
                domain::Bound::Items => wire::Bound::Items,
                domain::Bound::Answer => wire::Bound::Answer,
                domain::Bound::Tools => wire::Bound::Tools,
                domain::Bound::HistoryItems => wire::Bound::HistoryItems,
                domain::Bound::History => wire::Bound::History,
                domain::Bound::Window => wire::Bound::Window,
                domain::Bound::Output => wire::Bound::Output,
            },
            value: value.value,
        }),
        domain::Hold::Failures(class) => wire::Hold::Failures(wire::FailedClass { class: value::class(*class) }),
        domain::Hold::Stalled => wire::Hold::Stalled,
        domain::Hold::Dependency(task) => wire::Hold::Dependency(wire::Number { value: *task }),
        domain::Hold::Stopped => wire::Hold::Stopped,
        domain::Hold::Placed { hold } => wire::Hold::Placed(wire::Number { value: *hold }),
        domain::Hold::Drift => wire::Hold::Drift,
        domain::Hold::Effects => wire::Hold::Effects,
        domain::Hold::EffectFailed => wire::Hold::EffectFailed,
        domain::Hold::Uncertain { entry } => wire::Hold::Uncertain(wire::Effect { entry: *entry }),
        domain::Hold::Budget => wire::Hold::Budget,
        domain::Hold::Deadline => wire::Hold::Deadline,
        domain::Hold::HoldsWaited => wire::Hold::HoldsWaited,
        domain::Hold::ResourceUnavailable => wire::Hold::ResourceUnavailable,
        domain::Hold::ResourceAccess => wire::Hold::ResourceAccess,
        domain::Hold::PoolLost { pool } => wire::Hold::PoolLost(wire::Number { value: *pool }),
        domain::Hold::Procedure => wire::Hold::Procedure,
        domain::Hold::Circle => wire::Hold::Circle,
    })
}
/// Read `Hold` from its store value.
pub fn read_hold(source: wire::Hold) -> Result<domain::Hold, wire::Problem> {
    Ok(match source {
        wire::Hold::Exceeded(value) => domain::Hold::Exceeded(domain::Exceeded {
            bound: match value.bound {
                wire::Bound::Reasoning => domain::Bound::Reasoning,
                wire::Bound::Metadata => domain::Bound::Metadata,
                wire::Bound::Items => domain::Bound::Items,
                wire::Bound::Answer => domain::Bound::Answer,
                wire::Bound::Tools => domain::Bound::Tools,
                wire::Bound::HistoryItems => domain::Bound::HistoryItems,
                wire::Bound::History => domain::Bound::History,
                wire::Bound::Window => domain::Bound::Window,
                wire::Bound::Output => domain::Bound::Output,
            },
            value: value.value,
        }),
        wire::Hold::Failures(value) => {
            let wire::FailedClass { class } = value;
            domain::Hold::Failures(value::read_class(class))
        }
        wire::Hold::Stalled => domain::Hold::Stalled,
        wire::Hold::Dependency(value) => {
            let wire::Number { value } = value;
            domain::Hold::Dependency(value)
        }
        wire::Hold::Stopped => domain::Hold::Stopped,
        wire::Hold::Placed(value) => {
            let wire::Number { value: hold } = value;
            domain::Hold::Placed { hold }
        }
        wire::Hold::Drift => domain::Hold::Drift,
        wire::Hold::Effects => domain::Hold::Effects,
        wire::Hold::EffectFailed => domain::Hold::EffectFailed,
        wire::Hold::Uncertain(value) => {
            let wire::Effect { entry } = value;
            domain::Hold::Uncertain { entry }
        }
        wire::Hold::Budget => domain::Hold::Budget,
        wire::Hold::Deadline => domain::Hold::Deadline,
        wire::Hold::HoldsWaited => domain::Hold::HoldsWaited,
        wire::Hold::ResourceUnavailable => domain::Hold::ResourceUnavailable,
        wire::Hold::ResourceAccess => domain::Hold::ResourceAccess,
        wire::Hold::PoolLost(value) => {
            let wire::Number { value } = value;
            domain::Hold::PoolLost { pool: value }
        }
        wire::Hold::Procedure => domain::Hold::Procedure,
        wire::Hold::Circle => domain::Hold::Circle,
    })
}

/// Translate `Active` to its store value.
pub fn active(source: &domain::Active, _limits: &wire::Limits) -> Result<wire::Active, wire::Problem> {
    Ok(match source {
        domain::Active::Idle => wire::Active::Idle,
        domain::Active::Due => wire::Active::Due,
        domain::Active::Preparing => wire::Active::Preparing,
        domain::Active::Claimed { attempt } => wire::Active::Claimed(wire::Number { value: *attempt }),
        domain::Active::Running { attempt } => wire::Active::Running(wire::Number { value: *attempt }),
        domain::Active::BackingOff { until } => wire::Active::BackingOff(wire::Number { value: until.as_nanos() }),
    })
}
/// Read `Active` from its store value.
pub fn read_active(source: wire::Active) -> Result<domain::Active, wire::Problem> {
    Ok(match source {
        wire::Active::Idle => domain::Active::Idle,
        wire::Active::Due => domain::Active::Due,
        wire::Active::Preparing => domain::Active::Preparing,
        wire::Active::Claimed(value) => {
            let wire::Number { value } = value;
            domain::Active::Claimed { attempt: value }
        }
        wire::Active::Running(value) => {
            let wire::Number { value } = value;
            domain::Active::Running { attempt: value }
        }
        wire::Active::BackingOff(value) => {
            let wire::Number { value } = value;
            domain::Active::BackingOff { until: Wall::from_nanos(value) }
        }
    })
}

/// Translate `Stage` to its store value.
pub fn stage(source: &domain::Stage, _limits: &wire::Limits) -> Result<wire::Stage, wire::Problem> {
    Ok(match source {
        domain::Stage::Run { attempt } => wire::Stage::Run(wire::Number { value: *attempt }),
        domain::Stage::Delegates => wire::Stage::Delegates,
        domain::Stage::Effects => wire::Stage::Effects,
        domain::Stage::Releases => wire::Stage::Releases,
        domain::Stage::Settled => wire::Stage::Settled,
    })
}
/// Read `Stage` from its store value.
pub fn read_stage(source: wire::Stage) -> Result<domain::Stage, wire::Problem> {
    Ok(match source {
        wire::Stage::Run(value) => {
            let wire::Number { value } = value;
            domain::Stage::Run { attempt: value }
        }
        wire::Stage::Delegates => domain::Stage::Delegates,
        wire::Stage::Effects => domain::Stage::Effects,
        wire::Stage::Releases => domain::Stage::Releases,
        wire::Stage::Settled => domain::Stage::Settled,
    })
}

/// Translate `Ending` to its store value.
pub fn ending(source: &domain::Ending, limits: &wire::Limits) -> Result<wire::Ending, wire::Problem> {
    Ok(match source {
        domain::Ending::Done(result) => wire::Ending::Done(wire::Done { result: self::result(result, limits)? }),
        domain::Ending::Failed { reason } => wire::Ending::Failed(wire::FailureResult { reason: reason.clone() }),
        domain::Ending::Cancelled { by, at, reason, result } => wire::Ending::Cancelled(wire::Cancelled {
            by: value::party(by, limits)?,
            at: at.as_nanos(),
            reason: reason.clone(),
            result: optional_result(result.as_ref(), limits)?,
        }),
    })
}
/// Read `Ending` from its store value.
pub fn read_ending(source: wire::Ending, limits: &wire::Limits) -> Result<domain::Ending, wire::Problem> {
    Ok(match source {
        wire::Ending::Done(value) => {
            let wire::Done { result } = value;
            domain::Ending::Done(read_result(result, limits)?)
        }
        wire::Ending::Failed(value) => {
            let wire::FailureResult { reason } = value;
            domain::Ending::Failed { reason }
        }
        wire::Ending::Cancelled(value) => {
            let wire::Cancelled { by, at, reason, result } = value;
            domain::Ending::Cancelled {
                by: value::read_party(by),
                at: Wall::from_nanos(at),
                reason,
                result: read_optional_result(result, limits)?,
            }
        }
    })
}

/// Translate `Closing` to its store record.
pub fn closing(source: &domain::Closing, limits: &wire::Limits) -> Result<wire::Closing, wire::Problem> {
    let domain::Closing { stage, ending } = source;
    Ok(wire::Closing { stage: self::stage(stage, limits)?, ending: self::ending(ending, limits)? })
}
/// Read `Closing` from its store record.
pub fn read_closing(source: wire::Closing, limits: &wire::Limits) -> Result<domain::Closing, wire::Problem> {
    let wire::Closing { stage, ending } = source;
    Ok(domain::Closing { stage: read_stage(stage)?, ending: read_ending(ending, limits)? })
}

/// Translate `Was` to its store value.
pub fn was(source: &domain::Was, limits: &wire::Limits) -> Result<wire::Was, wire::Problem> {
    Ok(match source {
        domain::Was::Waiting => wire::Was::Waiting,
        domain::Was::Active(value) => wire::Was::Active(wire::Activation { value: active(value, limits)? }),
        domain::Was::Closing(value) => wire::Was::Closing(closing(value, limits)?),
    })
}
/// Read `Was` from its store value.
pub fn read_was(source: wire::Was, limits: &wire::Limits) -> Result<domain::Was, wire::Problem> {
    Ok(match source {
        wire::Was::Waiting => domain::Was::Waiting,
        wire::Was::Active(value) => {
            let wire::Activation { value } = value;
            domain::Was::Active(read_active(value)?)
        }
        wire::Was::Closing(value) => domain::Was::Closing(read_closing(value, limits)?),
    })
}

/// Translate `HoldSource` to its store value.
pub fn hold_source(source: &domain::HoldSource, limits: &wire::Limits) -> Result<wire::HoldSource, wire::Problem> {
    Ok(match source {
        domain::HoldSource::Preparation { words } => {
            wire::HoldSource::Preparation(wire::HoldPreparation { words: words.clone() })
        }
        domain::HoldSource::Account { connector, entry, words } => {
            wire::HoldSource::Account(wire::HoldAccount { connector: *connector, entry: *entry, words: words.clone() })
        }
        domain::HoldSource::Circle(circle) => wire::HoldSource::Circle(wire::Circle {
            tasks: references(&circle.tasks, limits.circle_tasks, wire::paths::CIRCLE_TASKS, limits)?,
            cut: circle.cut,
        }),
        domain::HoldSource::Resource(value) => wire::HoldSource::Resource(value::name(value, limits)?),
        domain::HoldSource::Effect { connector, entry } => {
            wire::HoldSource::Effect(wire::HoldEffect { connector: *connector, entry: *entry })
        }
    })
}
/// Read `HoldSource` from its store value.
pub fn read_hold_source(source: wire::HoldSource, limits: &wire::Limits) -> Result<domain::HoldSource, wire::Problem> {
    Ok(match source {
        wire::HoldSource::Preparation(value) => {
            let wire::HoldPreparation { words } = value;
            domain::HoldSource::Preparation { words }
        }
        wire::HoldSource::Account(value) => {
            let wire::HoldAccount { connector, entry, words } = value;
            domain::HoldSource::Account { connector, entry, words }
        }
        wire::HoldSource::Circle(value) => {
            let wire::Circle { tasks, cut } = value;
            domain::HoldSource::Circle(domain::Circle { tasks: read_references(tasks), cut })
        }
        wire::HoldSource::Resource(value) => domain::HoldSource::Resource(value::read_name(value, limits)?),
        wire::HoldSource::Effect(value) => {
            let wire::HoldEffect { connector, entry } = value;
            domain::HoldSource::Effect { connector, entry }
        }
    })
}

/// Translate `Phase` to its store value.
pub fn phase(source: &domain::Phase, limits: &wire::Limits) -> Result<wire::Phase, wire::Problem> {
    Ok(match source {
        domain::Phase::Waiting => wire::Phase::Waiting,
        domain::Phase::Active(value) => wire::Phase::Active(wire::Activation { value: active(value, limits)? }),
        domain::Phase::Closing(value) => wire::Phase::Closing(closing(value, limits)?),
        domain::Phase::Held { was, why, source } => wire::Phase::Held(wire::Held {
            was: self::was(was, limits)?,
            why: hold(why, limits)?,
            source: optional_source(source.as_deref(), limits)?,
        }),
        domain::Phase::Ended(value) => wire::Phase::Ended(wire::Final { ending: ending(value, limits)? }),
    })
}
/// Read `Phase` from its store value.
pub fn read_phase(source: wire::Phase, limits: &wire::Limits) -> Result<domain::Phase, wire::Problem> {
    Ok(match source {
        wire::Phase::Waiting => domain::Phase::Waiting,
        wire::Phase::Active(value) => {
            let wire::Activation { value } = value;
            domain::Phase::Active(read_active(value)?)
        }
        wire::Phase::Closing(value) => domain::Phase::Closing(read_closing(value, limits)?),
        wire::Phase::Held(value) => {
            let wire::Held { was, why, source } = value;
            domain::Phase::Held {
                was: read_was(was, limits)?,
                why: read_hold(why)?,
                source: read_optional_source(source, limits)?,
            }
        }
        wire::Phase::Ended(value) => {
            let wire::Final { ending } = value;
            domain::Phase::Ended(read_ending(ending, limits)?)
        }
    })
}

/// Translate `Grace` to its store record.
pub fn grace(source: &domain::Grace, limits: &wire::Limits) -> Result<wire::Grace, wire::Problem> {
    let domain::Grace { by, until, reason, message } = source;
    Ok(wire::Grace {
        by: value::party(by, limits)?,
        until: until.as_nanos(),
        reason: reason.clone(),
        message: *message,
    })
}
/// Read `Grace` from its store record.
pub fn read_grace(source: wire::Grace) -> Result<domain::Grace, wire::Problem> {
    let wire::Grace { by, until, reason, message } = source;
    Ok(domain::Grace { by: value::read_party(by), until: Wall::from_nanos(until), reason, message })
}

/// Translate `Revisions` to its store record.
pub fn revisions(source: &domain::Revisions, _limits: &wire::Limits) -> Result<wire::Revisions, wire::Problem> {
    let domain::Revisions { hold, spec, person } = source;
    Ok(wire::Revisions { hold: *hold, spec: *spec, person: *person })
}
/// Read `Revisions` from its store record.
pub fn read_revisions(source: wire::Revisions) -> Result<domain::Revisions, wire::Problem> {
    let wire::Revisions { hold, spec, person } = source;
    Ok(domain::Revisions { hold, spec, person })
}

/// Translate `RetryNotice` to its store value.
pub fn retry_notice(source: &domain::RetryNotice, _limits: &wire::Limits) -> Result<wire::RetryNotice, wire::Problem> {
    Ok(match source {
        domain::RetryNotice::Failed { class, count, allowance, reason } => {
            wire::RetryNotice::Failed(wire::FailedRetry {
                class: value::class(*class),
                count: *count,
                allowance: *allowance,
                reason: reason.clone(),
            })
        }
        domain::RetryNotice::Refused { count, reason } => {
            wire::RetryNotice::Refused(wire::RefusedRetry { count: *count, reason: reason.clone() })
        }
    })
}
/// Read `RetryNotice` from its store value.
pub fn read_retry_notice(source: wire::RetryNotice) -> Result<domain::RetryNotice, wire::Problem> {
    Ok(match source {
        wire::RetryNotice::Failed(value) => {
            let wire::FailedRetry { class, count, allowance, reason } = value;
            domain::RetryNotice::Failed { class: value::read_class(class), count, allowance, reason }
        }
        wire::RetryNotice::Refused(value) => {
            let wire::RefusedRetry { count, reason } = value;
            domain::RetryNotice::Refused { count, reason }
        }
    })
}

pub(super) fn numbers(source: &domain::Numbers, _limits: &wire::Limits) -> wire::Numbers {
    let domain::Numbers { budget, spent, spent_below, reserved } = source;
    wire::Numbers { budget: *budget, spent: *spent, spent_below: *spent_below, reserved: *reserved }
}
pub(super) fn read_numbers(source: wire::Numbers) -> domain::Numbers {
    let wire::Numbers { budget, spent, spent_below, reserved } = source;
    domain::Numbers { budget, spent, spent_below, reserved }
}
pub(super) fn references(
    source: &[u64],
    cap: u32,
    path: wire::Path,
    _limits: &wire::Limits,
) -> Result<List<wire::Number>, wire::Problem> {
    let mut members = List::with_capacity(cap);
    for number in source {
        let number = wire::Number { value: *number };
        if members.push(number).is_err() {
            return Err(problem(path));
        }
    }
    Ok(members)
}
pub(super) fn read_references(source: List<wire::Number>) -> Box<[u64]> {
    let mut members = List::with_capacity(source.len());
    for number in source.into_boxed() {
        members.push(number.value).expect("same number of references");
    }
    members.into_boxed()
}
pub(super) fn member(
    source: &domain::New,
    limits: &wire::Limits,
    moved: Option<&[u64; 64]>,
) -> Result<wire::Member, wire::Problem> {
    let domain::New {
        label,
        flags,
        number,
        project,
        executor,
        llm,
        spec,
        contract,
        authority,
        deadline,
        numbers: allotment,
        funder,
        dependencies,
        holdings,
        wake: policy,
        recurring: _,
        tracked,
    } = source;
    let mut resources = List::with_capacity(limits.member_holdings);
    for resource in holdings {
        let resource = holding(resource, limits)?;
        if resources.push(resource).is_err() {
            return Err(problem(wire::paths::MEMBER_HOLDINGS));
        }
    }
    Ok(wire::Member {
        label: *label,
        flags: self::flags(*flags, limits),
        number: *number,
        project: *project,
        executor: value::executor(*executor, limits)?,
        llm: value::llm(llm.as_ref(), limits)?,
        spec: value::spec(spec, limits)?,
        contract: value::contract(contract, limits)?,
        authority: value::authority(authority, limits, moved)?,
        deadline: match deadline {
            domain::Deadline::Named => wire::Deadline::Named,
            domain::Deadline::Span(value) => wire::Deadline::Span(wire::DeadlineSpan { value: *value }),
        },
        numbers: numbers(allotment, limits),
        funder: value::funder(*funder, limits),
        dependencies: references(dependencies, limits.member_dependencies, wire::paths::MEMBER_DEPENDENCIES, limits)?,
        holdings: resources,
        wake: wake(policy, limits)?,
        tracked: *tracked,
    })
}
pub(super) fn read_member(source: wire::Member, limits: &wire::Limits) -> Result<domain::New, wire::Problem> {
    let wire::Member {
        label,
        flags,
        number,
        project,
        executor,
        llm,
        spec,
        contract,
        authority,
        deadline,
        numbers,
        funder,
        dependencies,
        holdings,
        wake: policy,
        tracked,
    } = source;
    let mut resources = List::with_capacity(holdings.len());
    for resource in holdings.into_boxed() {
        resources.push(read_holding(resource, limits)?).expect("same number of holdings");
    }
    Ok(domain::New {
        deadline: match deadline {
            wire::Deadline::Named => domain::Deadline::Named,
            wire::Deadline::Span(value) => domain::Deadline::Span(value.value),
        },
        label,
        flags: read_flags(flags),
        number,
        project,
        executor: value::read_executor(executor),
        llm: value::read_llm(llm),
        spec: value::read_spec(spec, limits)?,
        contract: value::read_contract(contract, limits)?,
        authority: value::read_authority(authority, limits)?,
        numbers: read_numbers(numbers),
        funder: value::read_funder(funder),
        dependencies: read_references(dependencies),
        holdings: resources.into_boxed(),
        wake: read_wake(policy)?,
        recurring: None,
        tracked,
    })
}
/// Translate one creation, including its optional one-level template.
pub fn new(
    source: &domain::New,
    limits: &wire::Limits,
    moved: Option<&[u64; 64]>,
) -> Result<wire::Creation, wire::Problem> {
    let recurring = match &source.recurring {
        Some(value) => Some(template(value, limits, moved)?),
        None => None,
    };
    Ok(wire::Creation { member: member(source, limits, moved)?, recurring })
}
/// Read one creation, including its optional one-level template.
pub fn read_new(source: wire::Creation, limits: &wire::Limits) -> Result<domain::New, wire::Problem> {
    let wire::Creation { member, recurring } = source;
    let mut source = read_member(member, limits)?;
    source.recurring = match recurring {
        Some(value) => Some(Box::new(read_template(value, limits)?)),
        None => None,
    };
    Ok(source)
}
/// Translate a recurring batch, refusing a recurring member.
pub fn template(
    source: &domain::RecurringTemplate,
    limits: &wire::Limits,
    moved: Option<&[u64; 64]>,
) -> Result<wire::RecurringTemplate, wire::Problem> {
    let domain::RecurringTemplate { key, batch, overlap: rule } = source;
    let members = super::pieces::members(
        batch,
        limits,
        limits.recurring_template_batch,
        wire::paths::RECURRING_TEMPLATE_BATCH,
        moved,
    )?;
    Ok(wire::RecurringTemplate { key: *key, batch: members, overlap: overlap(rule)? })
}
/// Read a recurring batch whose schema members cannot recur.
pub fn read_template(
    source: wire::RecurringTemplate,
    limits: &wire::Limits,
) -> Result<domain::RecurringTemplate, wire::Problem> {
    let wire::RecurringTemplate { key, batch, overlap } = source;
    let members = super::pieces::read_members(batch, limits, wire::paths::RECURRING_TEMPLATE_BATCH)?;
    Ok(domain::RecurringTemplate { key, batch: members, overlap: read_overlap(overlap)? })
}
/// Translate a result within its combined byte bound.
pub fn result(source: &domain::TaskResult, limits: &wire::Limits) -> Result<wire::TaskResult, wire::Problem> {
    check_result(source, limits)?;
    Ok(match source {
        domain::TaskResult::Job { bytes } => wire::TaskResult::Job(wire::JobResult { bytes: bytes.clone() }),
        domain::TaskResult::Approved { state, words } => {
            wire::TaskResult::Approved(wire::Approved { state: state.clone(), words: words.clone() })
        }
        domain::TaskResult::NoChange { words } => {
            wire::TaskResult::NoChange(wire::ReportResult { words: words.clone() })
        }
        domain::TaskResult::Report { words } => wire::TaskResult::Report(wire::ReportResult { words: words.clone() }),
        domain::TaskResult::Verdict { code, fields, words } => {
            let members = super::pieces::fields(
                fields,
                limits,
                limits.verdict_result_fields,
                wire::paths::VERDICT_RESULT_FIELDS,
            )?;
            wire::TaskResult::Verdict(wire::VerdictResult { code: *code, fields: members, words: words.clone() })
        }
        domain::TaskResult::Change { connector, kind, resource, title, words } => {
            wire::TaskResult::Change(wire::ChangeResult {
                connector: *connector,
                kind: *kind,
                resource: *resource,
                title: title.words.clone(),
                words: words.clone(),
            })
        }
        domain::TaskResult::Failure { reason } => {
            wire::TaskResult::Failure(wire::FailureResult { reason: reason.clone() })
        }
    })
}
/// Read a result within its combined byte bound.
pub fn read_result(source: wire::TaskResult, limits: &wire::Limits) -> Result<domain::TaskResult, wire::Problem> {
    let source = match source {
        wire::TaskResult::Job(value) => domain::TaskResult::Job { bytes: value.bytes },
        wire::TaskResult::Approved(value) => {
            let wire::Approved { state, words } = value;
            domain::TaskResult::Approved { state, words }
        }
        wire::TaskResult::NoChange(value) => domain::TaskResult::NoChange { words: value.words },
        wire::TaskResult::Report(value) => domain::TaskResult::Report { words: value.words },
        wire::TaskResult::Verdict(value) => {
            let wire::VerdictResult { code, fields, words } = value;
            let members = super::pieces::read_fields(fields, limits, wire::paths::VERDICT_RESULT_FIELDS)?;
            domain::TaskResult::Verdict { code, fields: members, words }
        }
        wire::TaskResult::Change(value) => {
            let wire::ChangeResult { connector, kind, resource, title, words } = value;
            domain::TaskResult::Change { connector, kind, resource, title: domain::ChangeTitle::new(title), words }
        }
        wire::TaskResult::Failure(value) => domain::TaskResult::Failure { reason: value.reason },
    };
    check_result(&source, limits)?;
    Ok(source)
}
fn check_result(source: &domain::TaskResult, limits: &wire::Limits) -> Result<(), wire::Problem> {
    let bytes = match source {
        domain::TaskResult::Job { bytes } => bytes.len(),
        domain::TaskResult::Approved { state, words } => state.len().saturating_add(match words {
            Some(value) => value.len(),
            None => 0,
        }),
        domain::TaskResult::Verdict { words, fields, .. } => {
            let mut bytes = words.len();
            for field in fields {
                bytes = bytes.saturating_add(field.name.len()).saturating_add(field.value.len());
            }
            bytes
        }
        domain::TaskResult::NoChange { words } | domain::TaskResult::Report { words } => words.len(),
        domain::TaskResult::Change { title, words, .. } => title.words.len().max(words.len()),
        domain::TaskResult::Failure { reason } => reason.len(),
    };
    if bytes > skein_lib::length::usize_of(limits.result_field_value) {
        return Err(problem(wire::paths::TASK_RESULT_TAG));
    }
    Ok(())
}
fn optional_result(
    source: Option<&domain::TaskResult>,
    limits: &wire::Limits,
) -> Result<Option<wire::TaskResult>, wire::Problem> {
    match source {
        Some(value) => Ok(Some(result(value, limits)?)),
        None => Ok(None),
    }
}
fn read_optional_result(
    source: Option<wire::TaskResult>,
    limits: &wire::Limits,
) -> Result<Option<domain::TaskResult>, wire::Problem> {
    match source {
        Some(value) => Ok(Some(read_result(value, limits)?)),
        None => Ok(None),
    }
}
pub(super) fn optional_source(
    source: Option<&domain::HoldSource>,
    limits: &wire::Limits,
) -> Result<Option<wire::HoldSource>, wire::Problem> {
    match source {
        Some(value) => Ok(Some(hold_source(value, limits)?)),
        None => Ok(None),
    }
}
pub(super) fn read_optional_source(
    source: Option<wire::HoldSource>,
    limits: &wire::Limits,
) -> Result<Option<Box<domain::HoldSource>>, wire::Problem> {
    match source {
        Some(value) => Ok(Some(Box::new(read_hold_source(value, limits)?))),
        None => Ok(None),
    }
}
fn problem(path: wire::Path) -> wire::Problem {
    wire::Problem { path, reason: skein_codec::Reason::Bound }
}

/// Translate immutable task flags to their durable form.
pub(super) fn flags(source: domain::Flags, _limits: &wire::Limits) -> wire::Flags {
    let retention = match source.retention {
        domain::Retention::Kept => wire::Retention::Kept,
        domain::Retention::ForgottenOnceRead => wire::Retention::ForgottenOnceRead,
    };
    wire::Flags {
        ended_by: match source.ended_by {
            domain::EndedBy::Result => wire::EndedBy::Result,
            domain::EndedBy::Requester => wire::EndedBy::Requester,
            domain::EndedBy::Standing => wire::EndedBy::Standing,
            domain::EndedBy::Closer(closer) => {
                wire::EndedBy::Closer(wire::Closer { connector: closer.connector, code: closer.code })
            }
        },
        conversation: source.conversation,
        listed: source.listed,
        retention,
    }
}
/// Read immutable task flags.
pub(super) fn read_flags(source: wire::Flags) -> domain::Flags {
    let retention = match source.retention {
        wire::Retention::Kept => domain::Retention::Kept,
        wire::Retention::ForgottenOnceRead => domain::Retention::ForgottenOnceRead,
    };
    domain::Flags {
        ended_by: match source.ended_by {
            wire::EndedBy::Result => domain::EndedBy::Result,
            wire::EndedBy::Requester => domain::EndedBy::Requester,
            wire::EndedBy::Standing => domain::EndedBy::Standing,
            wire::EndedBy::Closer(closer) => {
                domain::EndedBy::Closer(domain::Closer { connector: closer.connector, code: closer.code })
            }
        },
        conversation: source.conversation,
        listed: source.listed,
        retention,
    }
}
