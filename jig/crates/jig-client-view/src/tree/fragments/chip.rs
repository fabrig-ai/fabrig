//! Source-faithful standalone chip metadata (client/domain/objects.md, 3;
//! view.md, 7 and 10). Borrowed normal Words supply every label; opaque
//! channel identity is retained by the contract and never turned into a number.
use crate::JigBindings;
use crate::words::Words;
use crate::words::chip_words::ChipPhrase as P;
use skein_ui::phrases::Hole;
use skein_ui::{Builder, Role, Run};

use crate::{Borrowed, Config};
use jig_client::contract::{
    ChipDetails, ChipEnding, ChipHold, ChipObservation, ChipWork, ClosingStage, DocumentText, EscalationHolder,
    HoldCategory, HoldEscalation, HoldFailure, PersonWait, PriorPhase, Requester, ResourceLine, ResourceWriter,
    ResultValue, RetryNotice, RunStage,
};
use skein_lib::Wall;

/// Whole bounded metadata, including one terminal result with brief follow-up chips.
#[must_use]
pub fn max_nodes(limits: &Config, client: &jig_client::DomainLimits) -> Option<u32> {
    120_u32
        .checked_add(crate::tree::fragments::proposed_task::part_nodes(limits, client)?)?
        .checked_add(client.segments.checked_add(20)?.checked_mul(client.rows)?)?
        .checked_add(client.segments.checked_add(6)?.checked_mul(2)?)?
        .checked_add(client.rows.checked_mul(6)?)?
        .checked_add(crate::result_nodes(limits, client)?)
}
fn text(b: &mut Builder, words: &Words, phrase: P, holes: &[Hole]) {
    let saying =
        words.book().say(words.language(), phrase.id(), holes).expect("checked chip words and exact typed holes");
    b.text(Run::Plain, &saying.text);
}
fn number(b: &mut Builder, words: &Words, phrase: P, value: u64) {
    text(b, words, phrase, &[Hole::Number(value)]);
}
fn wall(b: &mut Builder, words: &Words, phrase: P, at: Wall) {
    text(b, words, phrase, &[Hole::Wall(words.adjusted_wall(at))]);
}
fn document(b: &mut Builder, value: &DocumentText, words: &Words) {
    crate::words::literal_segment::render(b, value.bytes());
    if value.cut {
        text(b, words, P::Cut, &[]);
    }
}
pub(crate) fn resource(b: &mut Builder, value: &ResourceLine, words: &Words, bindings: &mut JigBindings) {
    b.open(Role::Region);
    let name = words
        .book()
        .say(words.language(), P::Kind.id(), &[Hole::Number(u64::from(value.kind))])
        .expect("checked resource kind and exact supplied number");
    b.name(&name.text);
    crate::tree::fragments::proposed_task::part(b, &value.part, words, bindings);
    number(b, words, P::Kind, u64::from(value.kind));
    text(b, words, if value.taken { P::Taken } else { P::NotTaken }, &[]);
    if let Some(holder) = value.holder {
        number(b, words, P::Holder, holder);
    }
    match value.writer {
        Some(ResourceWriter::Run { task, attempt }) => {
            number(b, words, P::WriterTask, task);
            number(b, words, P::WriterAttempt, attempt);
        }
        Some(ResourceWriter::Effect { entry }) => number(b, words, P::WriterEffect, entry),
        None => {}
    }
    text(b, words, if value.writer_lost { P::WriterLost } else { P::WriterKept }, &[]);
    b.close();
}
fn person_wait(b: &mut Builder, value: &PersonWait, words: &Words) {
    wall(b, words, P::WaitSince, value.since);
    text(b, words, P::Stall, &[Hole::Span(value.stall)]);
    wall(b, words, P::Deadline, value.deadline);
    if let Some(notice) = &value.notice {
        number(b, words, P::NoticeMessage, notice.message);
        wall(b, words, P::NoticeAt, notice.at);
        number(b, words, P::NoticeActivity, notice.activity);
    }
}
pub(crate) fn hold_failure(b: &mut Builder, value: HoldFailure, words: &Words) {
    let phrase = match value {
        HoldFailure::Transient => P::Transient,
        HoldFailure::Permanent => P::Permanent,
        HoldFailure::Run => P::RunFailure,
        HoldFailure::Agent => P::AgentFailure,
        HoldFailure::Lost => P::Lost,
        HoldFailure::Invalid => P::InvalidFailure,
        HoldFailure::Infrastructure => P::InfrastructureFailure,
    };
    text(b, words, phrase, &[]);
}
pub(crate) fn reason(b: &mut Builder, value: HoldCategory, words: &Words) {
    match value {
        HoldCategory::Exceeded(_) => words.hold_category(b, value),
        HoldCategory::Failures { class } => {
            text(b, words, P::Failures, &[]);
            hold_failure(b, class, words);
        }
        HoldCategory::Stalled => text(b, words, P::Stalled, &[]),
        HoldCategory::Dependency { task } => number(b, words, P::Dependency, task),
        HoldCategory::Stopped => text(b, words, P::Stopped, &[]),
        HoldCategory::Placed { .. } => text(b, words, P::Placed, &[]),
        HoldCategory::Drift => text(b, words, P::Drift, &[]),
        HoldCategory::Effects => text(b, words, P::Effects, &[]),
        HoldCategory::EffectFailed => text(b, words, P::EffectFailed, &[]),
        HoldCategory::Uncertain { entry } => number(b, words, P::Uncertain, entry),
        HoldCategory::Budget => text(b, words, P::Budget, &[]),
        HoldCategory::Deadline => text(b, words, P::HoldDeadline, &[]),
        HoldCategory::HoldsWaited => text(b, words, P::HoldsWaited, &[]),
        HoldCategory::ResourceUnavailable => text(b, words, P::ResourceUnavailable, &[]),
        HoldCategory::ResourceAccess => text(b, words, P::ResourceAccess, &[]),
        HoldCategory::PoolLost { pool } => number(b, words, P::PoolLost, pool),
        HoldCategory::Circle => words.text(b, crate::words::phrases::jig::JigPhrase::CircleHold, &[]),
        HoldCategory::Procedure => text(b, words, P::HoldProcedure, &[]),
    }
}
pub(crate) fn prior(b: &mut Builder, value: PriorPhase, words: &Words) {
    match value {
        PriorPhase::Waiting => text(b, words, P::PriorWaiting, &[]),
        PriorPhase::Active(stage) => match stage {
            RunStage::Idle => text(b, words, P::PriorIdle, &[]),
            RunStage::Due => text(b, words, P::PriorDue, &[]),
            RunStage::Preparing => text(b, words, P::PriorPreparing, &[]),
            RunStage::Claimed { attempt } => number(b, words, P::PriorClaimed, attempt),
            RunStage::Running { attempt } => number(b, words, P::PriorRunning, attempt),
            RunStage::BackingOff { until } => wall(b, words, P::PriorBackingOff, until),
        },
        PriorPhase::Closing(stage) => match stage {
            ClosingStage::Run { attempt } => number(b, words, P::PriorClosingRun, attempt),
            ClosingStage::Delegates => text(b, words, P::PriorDelegates, &[]),
            ClosingStage::Effects => text(b, words, P::PriorEffects, &[]),
            ClosingStage::Releases => text(b, words, P::PriorReleases, &[]),
            ClosingStage::Settled => text(b, words, P::PriorSettled, &[]),
        },
    }
}
fn escalation(b: &mut Builder, value: &HoldEscalation, words: &Words) {
    match value {
        HoldEscalation::Routing => text(b, words, P::Routing, &[]),
        HoldEscalation::Waiting { holder, entry, since } => {
            match holder {
                EscalationHolder::Task(task) => number(b, words, P::EscalationTask, *task),
                EscalationHolder::Person(person) => number(b, words, P::EscalationPerson, *person),
                EscalationHolder::Role { project, role } => {
                    number(b, words, P::EscalationProject, u64::from(*project));
                    number(b, words, P::EscalationRole, u64::from(*role));
                }
            }
            number(b, words, P::Entry, *entry);
            wall(b, words, P::EscalationSince, *since);
        }
        HoldEscalation::Rejected { by, reason } => {
            number(b, words, P::RejectedBy, *by);
            document(b, reason, words);
        }
    }
}
pub(crate) fn hold(b: &mut Builder, value: &ChipHold, words: &Words, bindings: &mut JigBindings) {
    if let Some(account) = &value.account {
        crate::tree::fragments::proposed_task::part(b, account, words, bindings);
    }
    if let Some(source) = &value.source {
        crate::tree::fragments::proposed_task::part(b, source, words, bindings);
    }
    reason(b, value.reason, words);
    if let Some(account) = &value.words {
        document(b, account, words);
    }
    if let Some(circle) = &value.circle {
        for task in &circle.tasks {
            words.text(b, crate::words::phrases::jig::JigPhrase::CircleWaitsFor, &[Hole::Number(*task)]);
        }
        words.text(b, crate::words::phrases::jig::JigPhrase::CircleLifts, &[]);
        if circle.cut {
            words.text(b, crate::words::phrases::jig::JigPhrase::TaskListsCut, &[]);
        }
    }
    number(b, words, P::Revision, value.revision);
    prior(b, value.before, words);
    if let Some(value) = &value.escalation {
        escalation(b, value, words);
    }
    for pair in [
        (P::TransientTries, value.tries.transient),
        (P::PermanentTries, value.tries.permanent),
        (P::RunTries, value.tries.run),
        (P::AgentTries, value.tries.agent),
        (P::LostTries, value.tries.lost),
        (P::InvalidTries, value.tries.invalid),
        (P::InfrastructureTries, value.tries.infrastructure),
    ] {
        number(b, words, pair.0, u64::from(pair.1));
    }
    if let Some(at) = value.hold_wait_since {
        wall(b, words, P::HoldWaitSince, at);
    }
}
/// Maximum leaves for the supplied work report: three tool leaves and three
/// independent clock leaves. No other metadata or inferred work is included.
#[must_use]
pub const fn observation_nodes() -> u32 {
    6
}

/// Show only the supplied observation, preserving all seven work kinds and
/// each independent clock. Channel identity remains an opaque contract fact.
pub fn observation(b: &mut Builder, value: &ChipObservation, words: &Words) {
    match &value.work {
        ChipWork::Preparing => text(b, words, P::Preparing, &[]),
        ChipWork::Starting => text(b, words, P::Starting, &[]),
        ChipWork::Active => text(b, words, P::Active, &[]),
        ChipWork::Writing => text(b, words, P::Writing, &[]),
        ChipWork::Tool { call, tool } => {
            text(b, words, P::Tool, &[]);
            crate::words::literal_segment::render(b, call);
            crate::words::literal_segment::render(b, tool);
        }
        ChipWork::Waiting => text(b, words, P::Waiting, &[]),
        ChipWork::Ending => text(b, words, P::Ending, &[]),
    }
    wall(b, words, P::ObservedSince, value.since);
    wall(b, words, P::ReportedAt, value.reported_at);
    match value.started {
        Some(at) => wall(b, words, P::Started, at),
        None => text(b, words, P::StartUnknown, &[]),
    }
}
fn result_value(
    b: &mut Builder,
    value: &ResultValue,
    client: &Borrowed<'_>,
    words: &Words,
    bindings: &mut JigBindings,
) {
    match value {
        ResultValue::Approved { state, words: explanation } => {
            words.text(b, crate::words::phrases::jig::JigPhrase::ApprovalExpected, &[]);
            crate::words::literal_segment::render(b, state);
            if let Some(explanation) = explanation {
                document(b, explanation, words);
            }
        }
        ResultValue::Report(_)
        | ResultValue::Words(_)
        | ResultValue::Failure(_)
        | ResultValue::Verdict { .. }
        | ResultValue::OwnerVerdict { .. }
        | ResultValue::Connector { .. }
        | ResultValue::Change { .. }
        | ResultValue::Choice { .. } => crate::tree::cards::result::value_nodes(b, client, value, words, bindings),
    }
}
/// The departure caption and its supplied reason, author and time.
/// The complete result payload is rendered by the document's content below this state.
pub(crate) fn ending_summary(b: &mut Builder, value: &ChipEnding, words: &Words) {
    match value {
        ChipEnding::Done(_) => text(b, words, P::Done, &[]),
        ChipEnding::Failed(reason) => {
            text(b, words, P::Failed, &[]);
            document(b, reason, words);
        }
        ChipEnding::Cancelled { reason, by, at, .. } => {
            text(b, words, P::Cancelled, &[]);
            document(b, reason, words);
            match by {
                Requester::Task(task) => number(b, words, P::RequesterTask, *task),
                Requester::Person(person) => number(b, words, P::RequesterPerson, *person),
                Requester::Deployment { project } => number(b, words, P::RequesterDeployment, u64::from(*project)),
            }
            wall(b, words, P::CancelledAt, *at);
        }
    }
}
pub(crate) fn ending(
    b: &mut Builder,
    value: &ChipEnding,
    client: &Borrowed<'_>,
    words: &Words,
    bindings: &mut JigBindings,
) {
    ending_summary(b, value, words);
    match value {
        ChipEnding::Done(result) => result_value(b, result, client, words, bindings),
        ChipEnding::Failed(_) => {}
        ChipEnding::Cancelled { result, .. } => {
            if let Some(result) = result {
                result_value(b, result, client, words, bindings);
            }
        }
    }
}
/// Render metadata without changing domain state, routing or issuing any read.
/// A borrowed `Ui` is used only by the existing supplied `ResultValue` fragment.
pub fn render(b: &mut Builder, value: &ChipDetails, client: &Borrowed<'_>, words: &Words, bindings: &mut JigBindings) {
    b.open(Role::Region);
    let name = words
        .book()
        .say(words.language(), P::Project.id(), &[Hole::Number(u64::from(value.project))])
        .expect("checked chip project and exact supplied number");
    b.name(&name.text);
    number(b, words, P::Project, u64::from(value.project));
    crate::tree::fragments::proposed_task::executor(b, value.executor, words);
    wall(b, words, P::PhaseSince, value.phase_since);
    if let Some(at) = value.run_since {
        wall(b, words, P::Placement, at);
    }
    if let Some(attempt) = value.attempt {
        number(b, words, P::Attempt, attempt);
    }
    match &value.retry {
        Some(RetryNotice::Failed { class, count, allowance, reason }) => {
            crate::tree::cards::chip::failure(b, *class, words);
            words.text(
                b,
                crate::words::phrases::jig::JigPhrase::RunRetryFailed,
                &[Hole::Number(u64::from(*count)), Hole::Number(u64::from(*allowance))],
            );
            retry_reason(b, reason.as_ref(), words);
        }
        Some(RetryNotice::OwnerFailed { class, count, allowance, reason }) => {
            hold_failure(b, *class, words);
            words.text(
                b,
                crate::words::phrases::jig::JigPhrase::RunRetryFailed,
                &[Hole::Number(u64::from(*count)), Hole::Number(u64::from(*allowance))],
            );
            retry_reason(b, reason.as_ref(), words);
        }
        Some(RetryNotice::Refused { count, reason }) => {
            words.text(b, crate::words::phrases::jig::JigPhrase::RunRetryRefused, &[Hole::Number(u64::from(*count))]);
            retry_reason(b, reason.as_ref(), words);
        }
        None => {}
    }
    crate::tree::fragments::wake::policy(b, &value.wake, words.book(), words.language());
    if let Some(wait) = &value.person_wait {
        person_wait(b, wait, words);
    }
    if let Some(value) = &value.hold {
        hold(b, value, words, bindings);
    }
    crate::tree::fragments::budget::render(b, &value.budget, words);
    number(b, words, P::Activity, value.activity);
    text(b, words, if value.resources_observed { P::ResourcesObserved } else { P::ResourcesUnobserved }, &[]);
    for value in &value.resources {
        resource(b, value, words, bindings);
    }
    if value.resources_cut {
        text(b, words, P::ResourcesCut, &[]);
    }
    if let Some(part) = &value.procedure {
        text(b, words, P::Procedure, &[]);
        crate::tree::fragments::proposed_task::part(b, part, words, bindings);
    }
    number(b, words, P::LiveDelegates, u64::from(value.live_delegates));
    match &value.closing_connectors {
        None => text(b, words, P::ConnectorsUnknown, &[]),
        Some(connectors) => {
            if connectors.is_empty() {
                text(b, words, P::ConnectorsEmpty, &[]);
            }
            for connector in connectors {
                number(b, words, P::Connector, u64::from(*connector));
            }
        }
    }
    if let Some(value) = &value.ending {
        ending(b, value, client, words, bindings);
    }
    if let Some(at) = value.ended_at {
        wall(b, words, P::EndedAt, at);
    }
    match &value.observation {
        Some(value) => observation(b, value, words),
        None => text(b, words, P::Unobserved, &[]),
    }
    b.close();
}

/// A supplied retry reason follows the retry count in the selected language.
pub(crate) fn retry_reason(b: &mut Builder, reason: Option<&DocumentText>, words: &Words) {
    if let Some(reason) = reason {
        words.text(b, crate::words::phrases::jig::JigPhrase::RunRetryReason, &[crate::words::name(reason.bytes())]);
        if reason.cut {
            words.text(b, crate::words::phrases::jig::JigPhrase::DocumentTextCut, &[]);
        }
    }
}
