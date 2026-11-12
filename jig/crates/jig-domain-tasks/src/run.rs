//! Fenced run preparation, claims, activation terminals and holds
//! (domain/tasks/lifecycle.md, sections 2 and 6–8). Root supplies run
//! events; tasks checks lifecycle/result shape and emits bounded state changes.
//! Priced inputs use combined accounting admission; exact replay is root-owned.
pub(crate) mod ends;

use crate::domain::{Domain, entrance, fact, publish, record, refused, task_mut};
use crate::{
    Accepted, Active, Class, Closing, Contract, End, Ending, Fact, Hold, InvalidResult, Limits, Phase, Refusal,
    Request, SavedResource, Stage, TaskResult, Tries, Was,
};
use alloc::boxed::Box;
use skein_lib::{Env, Queue, ReplyTo};

pub(crate) fn saved_within(saved: Option<&[SavedResource]>, limits: &Limits) -> bool {
    let Some(resources) = saved else { return true };
    if resources.len() > skein_lib::length::usize_of(limits.saved_resources) {
        return false;
    }
    let mut previous = None;
    for resource in resources {
        if resource.path.is_empty() || resource.path.len() > skein_lib::length::usize_of(limits.authority_segments) {
            return false;
        }
        let mut bytes = 0usize;
        for segment in &resource.path {
            let Some(sum) = bytes.checked_add(segment.len()) else { return false };
            bytes = sum;
        }
        if bytes > skein_lib::length::usize_of(limits.authority_bytes) {
            return false;
        }
        if let Some(before) = previous
            && before >= resource
        {
            return false;
        }
        previous = Some(resource);
    }
    true
}

pub(crate) fn prepare(domain: &mut Domain, env: &Env<Limits>, to: ReplyTo, number: u64, out: &mut Queue<Request>) {
    let to = match entrance(domain, to, number) {
        Ok(to) => to,
        Err((to, why)) => return refused(to, Some(number), why, out),
    };
    let kept = ends::kept(domain, number);
    let moves = domain.task(number).expect("entrance names task").flags.conversation && domain.may_move(number);
    let newest = domain.newest();
    let task = task_mut(domain, number).expect("entrance names task");
    match task.record.executor {
        crate::Executor::Agent { .. } | crate::Executor::Job { .. } => {}
        crate::Executor::Procedure { .. } | crate::Executor::Person(_) => {
            return refused(to, Some(number), Refusal::Executor, out);
        }
    }
    if task.record.phase != Phase::Active(Active::Due) || kept.blocks_claims() {
        return refused(to, Some(number), Refusal::State, out);
    }
    if moves && task.record.settings.task != newest {
        crate::session::end(&mut task.record, env.wall, crate::SessionEnd::Rules, out);
        task.record.settings.task = newest;
    }
    task.record.settings.run = Some(task.record.settings.task);
    task.record.phase = Phase::Active(Active::Preparing);
    publish(domain, env, number, out);
    out.push(Request::Done { reply_to: to });
}

pub(crate) struct Claiming {
    pub(crate) session_end: Option<crate::SessionEnd>,
    pub(crate) reasons: crate::Reasons,
    pub(crate) offered: crate::Reasons,
    pub(crate) number: u64,
    pub(crate) attempt: u64,
    pub(crate) resumed: bool,
    pub(crate) budget: u64,
    pub(crate) account: Option<u32>,
    pub(crate) writes: Box<[crate::Name]>,
}

pub(crate) fn claim(domain: &mut Domain, env: &Env<Limits>, to: ReplyTo, claiming: Claiming, out: &mut Queue<Request>) {
    let Claiming { number, attempt, budget, account, ref writes, .. } = claiming;
    let to = match entrance(domain, to, number) {
        Ok(to) => to,
        Err((to, why)) => return refused(to, Some(number), why, out),
    };
    let task = record(domain, number).expect("entrance names task");
    if attempt <= task.attempt {
        return refused(to, Some(number), Refusal::Attempt, out);
    }
    if task.phase != Phase::Active(Active::Preparing) || ends::kept(domain, number).blocks_claims() {
        return refused(to, Some(number), Refusal::State, out);
    }
    let fits = match task.executor {
        crate::Executor::Job { .. } => {
            if account.is_some() {
                return refused(to, Some(number), Refusal::Executor, out);
            }
            budget == 0
        }
        crate::Executor::Agent { .. } => {
            match crate::funders::available(crate::closer::claim_numbers(domain, number).expect("live claim")) {
                Some(left) => budget != 0 && budget <= left,
                None => false,
            }
        }
        crate::Executor::Procedure { .. } | crate::Executor::Person(_) => {
            return refused(to, Some(number), Refusal::Executor, out);
        }
    };
    if !fits {
        return refused(to, Some(number), Refusal::Funding, out);
    }
    match crate::writers::claim(domain, env, number, attempt, writes, out) {
        Ok(()) => {}
        Err(Some(resource)) => return out.push(Request::WriterWaiting { reply_to: to, task: Some(number), resource }),
        Err(None) => return refused(to, Some(number), Refusal::Holds, out),
    }
    crate::closer::yield_for_run(domain, env, number, out);
    finish_claim(domain, env, to, claiming, out);
}

fn finish_claim(domain: &mut Domain, env: &Env<Limits>, to: ReplyTo, claiming: Claiming, out: &mut Queue<Request>) {
    let Claiming { number, attempt, resumed, budget, account, reasons, offered, session_end, writes: _ } = claiming;
    let task = task_mut(domain, number).expect("entrance names task");
    task.record.numbers.reserved = task.record.numbers.reserved.checked_add(budget).expect("available budget");
    task.record.run_reserved = budget;
    if let Some(account) = account {
        task.record.kept_account = Some(account);
    }
    if task.record.flags.ended_by == crate::EndedBy::Standing {
        if !task.record.activation_open
            && crate::talk_state(task.record.talk, task.record.session, env.wall) == crate::TalkState::Lapsed
        {
            crate::session::leave(&mut task.record, env.wall, true, out);
        }
        if let Some(end) = session_end {
            crate::session::end(&mut task.record, env.wall, end, out);
        }
        if task.record.session.is_none() {
            task.record.session = Some(crate::Session { first: attempt, reasons, last: env.wall });
            task.record.tries = Tries::NONE;
        }
    }
    task.record.activation_open = true;
    task.record.reasons = reasons;
    task.record.offered = offered;
    task.record.attempt = attempt;
    task.record.run_since = None;
    task.record.retry_notice = None;
    task.record.run_spent = 0;
    task.record.invalid_result = None;
    task.record.turn = 0;
    task.record.refused = Box::new([]);
    task.record.phase = Phase::Active(Active::Claimed { attempt });
    let why = task.record.due_reason;
    crate::activity::append(&mut task.record, env.wall, crate::Line::RunBegan { attempt, why, resumed }, out);
    publish(domain, env, number, out);
    fact(domain, Fact::Claimed { task: number, attempt });
    out.push(Request::Done { reply_to: to });
}

pub(crate) fn started(domain: &mut Domain, env: &Env<Limits>, number: u64, attempt: u64, out: &mut Queue<Request>) {
    if !domain.ready() {
        return;
    }
    if let Some(task) = task_mut(domain, number)
        && task.record.phase == Phase::Active(Active::Claimed { attempt })
    {
        task.record.phase = Phase::Active(Active::Running { attempt });
        task.record.run_since = Some(env.wall);
        publish(domain, env, number, out);
    }
}

/// The claimed attempt whose answer this phase has not heard.
#[must_use]
pub fn run_attempt(phase: &Phase) -> Option<u64> {
    match phase {
        Phase::Active(Active::Claimed { attempt } | Active::Running { attempt })
        | Phase::Closing(Closing { stage: Stage::Run { attempt }, .. })
        | Phase::Held {
            was:
                Was::Active(Active::Claimed { attempt } | Active::Running { attempt })
                | Was::Closing(Closing { stage: Stage::Run { attempt }, .. }),
            ..
        } => Some(*attempt),
        Phase::Waiting
        | Phase::Active(Active::Idle | Active::Due | Active::Preparing | Active::BackingOff { .. })
        | Phase::Closing(Closing {
            stage: Stage::Delegates | Stage::Effects | Stage::Releases | Stage::Settled, ..
        })
        | Phase::Held {
            was:
                Was::Waiting
                | Was::Active(Active::Idle | Active::Due | Active::Preparing | Active::BackingOff { .. })
                | Was::Closing(Closing {
                    stage: Stage::Delegates | Stage::Effects | Stage::Releases | Stage::Settled, ..
                }),
            ..
        }
        | Phase::Ended(_) => None,
    }
}

pub(crate) fn valid_result(contract: &Contract, result: &TaskResult, limits: &Limits) -> bool {
    invalid_result(contract, result, limits).is_none()
}

pub(crate) fn invalid_result(contract: &Contract, result: &TaskResult, limits: &Limits) -> Option<InvalidResult> {
    match result {
        TaskResult::Job { bytes } => match contract {
            Contract::Job { bytes: bound } => {
                if bytes.len() <= skein_lib::length::usize_of(*bound) {
                    None
                } else {
                    Some(InvalidResult::Words)
                }
            }
            Contract::Approval { .. }
            | Contract::Report { .. }
            | Contract::Verdict { .. }
            | Contract::Change { .. } => Some(InvalidResult::Form),
        },
        TaskResult::NoChange { words } => invalid_no_change(contract, words, limits),
        TaskResult::Approved { .. } => invalid_approval(contract, result, limits),
        TaskResult::Failure { reason } => match contract {
            Contract::Job { .. }
            | Contract::Approval { .. }
            | Contract::Report { .. }
            | Contract::Verdict { .. }
            | Contract::Change { .. } => {
                if reason.len() <= skein_lib::length::usize_of(limits.result_bytes) {
                    None
                } else {
                    Some(InvalidResult::Words)
                }
            }
        },
        TaskResult::Report { words } => match contract {
            Contract::Report { words: limit } => {
                if words.len() <= skein_lib::length::usize_of(*limit) {
                    None
                } else {
                    Some(InvalidResult::Words)
                }
            }
            Contract::Job { .. } | Contract::Approval { .. } | Contract::Verdict { .. } | Contract::Change { .. } => {
                Some(InvalidResult::Form)
            }
        },
        TaskResult::Verdict { code, words, fields } => match contract {
            Contract::Verdict { choices } => {
                for choice in choices {
                    if choice.code == *code {
                        return if words.len() > skein_lib::length::usize_of(choice.words)
                            || result_bytes(result) > skein_lib::length::usize_of(limits.result_bytes)
                        {
                            Some(InvalidResult::Words)
                        } else if !crate::verdict::fields_valid(&choice.fields, fields) {
                            Some(InvalidResult::Form)
                        } else {
                            None
                        };
                    }
                }
                Some(InvalidResult::Verdict)
            }
            Contract::Job { .. } | Contract::Approval { .. } | Contract::Report { .. } | Contract::Change { .. } => {
                Some(InvalidResult::Form)
            }
        },
        TaskResult::Change { connector, kind, title, words, .. } => {
            invalid_change(contract, *connector, *kind, &title.words, words, limits)
        }
    }
}

fn invalid_no_change(contract: &Contract, words: &[u8], limits: &Limits) -> Option<InvalidResult> {
    match contract {
        Contract::Change { words: bound, .. } => {
            if words.len() <= skein_lib::length::usize_of((*bound).min(limits.result_bytes)) {
                None
            } else {
                Some(InvalidResult::Words)
            }
        }
        Contract::Job { .. } | Contract::Approval { .. } | Contract::Report { .. } | Contract::Verdict { .. } => {
            Some(InvalidResult::Form)
        }
    }
}

fn invalid_approval(contract: &Contract, result: &TaskResult, limits: &Limits) -> Option<InvalidResult> {
    match contract {
        Contract::Approval { state: expected, .. } => {
            if let TaskResult::Approved { state, .. } = result
                && state != expected
            {
                Some(InvalidResult::Form)
            } else if result_bytes(result) > skein_lib::length::usize_of(limits.result_bytes) {
                Some(InvalidResult::Words)
            } else {
                None
            }
        }
        Contract::Job { .. } | Contract::Report { .. } | Contract::Verdict { .. } | Contract::Change { .. } => {
            Some(InvalidResult::Form)
        }
    }
}

fn invalid_change(
    contract: &Contract,
    connector: u16,
    kind: u16,
    title: &[u8],
    words: &[u8],
    limits: &Limits,
) -> Option<InvalidResult> {
    match contract {
        Contract::Change { connector: expected_connector, kind: expected_kind, words: limit } => {
            if connector != *expected_connector || kind != *expected_kind {
                Some(InvalidResult::Change)
            } else if title.len().max(words.len()) > skein_lib::length::usize_of(*limit)
                || title.len().max(words.len()) > skein_lib::length::usize_of(limits.result_bytes)
            {
                Some(InvalidResult::Words)
            } else {
                None
            }
        }
        Contract::Job { .. } | Contract::Approval { .. } | Contract::Report { .. } | Contract::Verdict { .. } => {
            Some(InvalidResult::Form)
        }
    }
}

/// Check the admitted verdict's follow-up allowance as well as its result words.
pub(crate) fn invalid_followups(
    contract: &Contract,
    result: &TaskResult,
    followups: &crate::ResultFollowups,
    limits: &Limits,
) -> Option<InvalidResult> {
    if let Some(reason) = invalid_result(contract, result, limits) {
        return Some(reason);
    }
    let batch = match followups {
        crate::ResultFollowups::Delegates(batch) => batch,
        crate::ResultFollowups::Proposal(proposal) => match &proposal.action {
            crate::ProposalAction::Batch(batch) => batch,
            crate::ProposalAction::Settings { .. }
            | crate::ProposalAction::Effect { .. }
            | crate::ProposalAction::ProcedureEffect { .. }
            | crate::ProposalAction::Amend { .. }
            | crate::ProposalAction::Widen { .. }
            | crate::ProposalAction::Release { .. }
            | crate::ProposalAction::ReleasePlaced { .. } => {
                return Some(InvalidResult::Form);
            }
        },
    };
    if !crate::batch::within(limits, batch) {
        return Some(InvalidResult::Followups);
    }
    let count = batch.len();
    match (contract, result) {
        (Contract::Verdict { choices }, TaskResult::Verdict { code, .. }) => {
            for choice in choices {
                if choice.code == *code {
                    return if count > 0 && count <= skein_lib::length::usize_of(choice.followups) {
                        None
                    } else {
                        Some(InvalidResult::Followups)
                    };
                }
            }
            Some(InvalidResult::Verdict)
        }
        (
            Contract::Job { .. }
            | Contract::Approval { .. }
            | Contract::Report { .. }
            | Contract::Change { .. }
            | Contract::Verdict { .. },
            _,
        ) => Some(InvalidResult::Form),
    }
}

pub(crate) fn result_bytes(result: &TaskResult) -> usize {
    match result {
        TaskResult::Job { bytes } => bytes.len(),
        TaskResult::Report { words } | TaskResult::NoChange { words } => words.len(),
        TaskResult::Change { title, words, .. } => title.words.len().max(words.len()),
        TaskResult::Verdict { words, fields, .. } => match crate::verdict::fields_bytes(fields) {
            Some(bytes) => bytes.saturating_add(words.len()),
            None => usize::MAX,
        },
        TaskResult::Approved { state, words } => state.len().saturating_add(match words {
            Some(words) => words.len(),
            None => 0,
        }),
        TaskResult::Failure { reason } => reason.len(),
    }
}

fn ending(result: TaskResult) -> Ending {
    match result {
        TaskResult::Failure { reason } => Ending::Failed { reason },
        TaskResult::Approved { .. }
        | TaskResult::Report { .. }
        | TaskResult::Verdict { .. }
        | TaskResult::Job { .. }
        | TaskResult::NoChange { .. }
        | TaskResult::Change { .. } => Ending::Done(result),
    }
}

/// Job terminals cannot park or ask the executor for another decision (jig/docs/design/domain/tasks/lifecycle.md, 3.1).
pub(crate) fn refuses_job_end(executor: crate::Executor, end: &End) -> bool {
    if !crate::batch::matches_job(executor) {
        return false;
    }
    match end {
        End::Parked | End::FinishedWithFollowups { .. } | End::Exceeded(_) | End::Recheck => true,
        End::Finished { .. } | End::Failed(_) | End::FailedWithReason { .. } | End::Refused => false,
    }
}

pub(crate) struct Ended {
    pub(crate) number: u64,
    pub(crate) attempt: u64,
    pub(crate) end: End,
    pub(crate) saved: Option<Box<[SavedResource]>>,
}

#[expect(clippy::too_many_lines, reason = "one terminal normalizes the result and settles its current attempt")]
pub(crate) fn activation(domain: &mut Domain, env: &Env<Limits>, to: ReplyTo, ended: Ended, out: &mut Queue<Request>) {
    let Ended { number, attempt, end, saved } = ended;
    let (end, reason) = match end {
        End::FailedWithReason { class, reason } => {
            let reason = preparation_reason(&reason);
            let reason = if reason.is_empty() { None } else { Some(reason) };
            (End::Failed(class), reason)
        }
        end @ (End::Exceeded(_)
        | End::Finished { .. }
        | End::FinishedWithFollowups { .. }
        | End::Parked
        | End::Failed(_)
        | End::Refused
        | End::Recheck) => (end, None),
    };
    if !domain.ready() {
        return refused(to, Some(number), Refusal::NotReady, out);
    }
    let Some(old) = record(domain, number) else {
        return refused(to, Some(number), Refusal::Unknown, out);
    };
    if old.last_answer == Some(attempt) {
        return out.push(Request::Acknowledged {
            reply_to: to,
            task: number,
            attempt,
            accepted: Accepted::Already,
            parked: false,
        });
    }
    if run_attempt(&old.phase) != Some(attempt) {
        return refused(to, Some(number), Refusal::Attempt, out);
    }
    if refuses_job_end(old.executor, &end) {
        return refused(to, Some(number), Refusal::Executor, out);
    }
    if !saved_within(saved.as_deref(), &env.limits) {
        return refused(to, Some(number), Refusal::Contract, out);
    }
    let lost_writer = match &end {
        End::Failed(Class::Lost) => true,
        End::Failed(_)
        | End::FailedWithReason { .. }
        | End::Exceeded(_)
        | End::Parked
        | End::Refused
        | End::Recheck
        | End::Finished { .. }
        | End::FinishedWithFollowups { .. } => false,
    };
    let held = match &old.phase {
        Phase::Held { why, source, .. } => Some((*why, source.clone())),
        Phase::Waiting | Phase::Active(_) | Phase::Closing(_) | Phase::Ended(_) => None,
    };
    let closing = match &old.phase {
        Phase::Closing(closing) | Phase::Held { was: Was::Closing(closing), .. } => Some(closing.clone()),
        Phase::Waiting
        | Phase::Active(_)
        | Phase::Held { was: Was::Waiting | Was::Active(_), .. }
        | Phase::Ended(_) => None,
    };
    let narrowed = old.narrowing;
    let previous_tries = old.tries;
    let kept = ends::kept(domain, number);
    let discarded = !narrowed && kept.discards(&end);
    let stood = if discarded && old.flags.ended_by == crate::EndedBy::Standing { ends::stood(&end) } else { None };
    let end = if narrowed || discarded { End::Parked } else { end };
    let result_followups = match &end {
        End::FinishedWithFollowups { .. } => true,
        End::Finished { .. }
        | End::Exceeded(_)
        | End::Parked
        | End::Failed(_)
        | End::FailedWithReason { .. }
        | End::Refused
        | End::Recheck => false,
    };
    let invalid = match &end {
        End::Finished { result, .. } => invalid_result(&old.contract, result, &env.limits),
        End::FinishedWithFollowups { result, followups } => {
            invalid_followups(&old.contract, result, followups, &env.limits)
        }
        End::Exceeded(_)
        | End::Parked
        | End::Failed(_)
        | End::FailedWithReason { .. }
        | End::Refused
        | End::Recheck => None,
    };
    let parked = invalid.is_none() && kept.run_parks(&end);
    let end = match end {
        End::Finished { result, cancel_delegates } => {
            if invalid.is_none() {
                End::Finished { result, cancel_delegates }
            } else {
                End::Failed(Class::Invalid)
            }
        }
        End::FinishedWithFollowups { result, .. } => {
            if invalid.is_none() {
                End::Finished { result, cancel_delegates: false }
            } else {
                End::Failed(Class::Invalid)
            }
        }
        End::Exceeded(_)
        | End::Parked
        | End::Failed(_)
        | End::FailedWithReason { .. }
        | End::Refused
        | End::Recheck => end,
    };
    let end = if parked { End::Parked } else { end };
    let refuses_live = match &end {
        End::Finished { cancel_delegates, .. } => !cancel_delegates && !result_followups,
        End::FinishedWithFollowups { .. } => unreachable!("follow-ups normalized"),
        End::Exceeded(_)
        | End::Parked
        | End::Failed(_)
        | End::FailedWithReason { .. }
        | End::Refused
        | End::Recheck => false,
    };
    if closing.is_none() && refuses_live && !old.delegates.is_empty() {
        return out
            .push(Request::Refused { reply_to: to, problem: crate::Problem::live_delegates(number, &old.delegates) });
    }
    let end_kind = if (narrowed && !old.llm_changed) || closing.is_some() {
        crate::EndKind::Cancelled
    } else {
        match &end {
            End::Parked => crate::EndKind::Parked,
            End::Finished { .. } => crate::EndKind::Finished,
            End::Exceeded(_) => crate::EndKind::Failed { class: Class::Run, invalid_result: None },
            End::FailedWithReason { class, .. } | End::Failed(class) => {
                crate::EndKind::Failed { class: *class, invalid_result: invalid }
            }
            End::Refused | End::Recheck => crate::EndKind::Refused,
            End::FinishedWithFollowups { .. } => unreachable!("follow-ups normalized"),
        }
    };
    let terminal_parks = end == End::Parked;
    let activation_ended = match &end {
        End::Parked => !narrowed && !kept.blocks_claims() && !kept.owes,
        End::Finished { .. } => true,
        End::Exceeded(_) | End::Failed(_) | End::FailedWithReason { .. } | End::Refused | End::Recheck => false,
        End::FinishedWithFollowups { .. } => unreachable!("follow-ups normalized"),
    };
    let next = if let Some(mut closing) = closing {
        closing.stage = Stage::Delegates;
        match &mut closing.ending {
            Ending::Cancelled { result: partial, .. } => match end {
                End::Finished { result, .. } => *partial = Some(result),
                End::FinishedWithFollowups { .. } => unreachable!("follow-ups normalized"),
                End::Exceeded(_)
                | End::Parked
                | End::Failed(_)
                | End::FailedWithReason { .. }
                | End::Refused
                | End::Recheck => {}
            },
            Ending::Done(_) | Ending::Failed { .. } => {}
        }
        Phase::Closing(closing)
    } else {
        match end {
            End::Finished { result, cancel_delegates } => {
                if cancel_delegates {
                    crate::closing::cancel_delegates(domain, env, number, b"", out);
                }
                Phase::Closing(Closing { stage: Stage::Delegates, ending: ending(result) })
            }
            End::FinishedWithFollowups { .. } => unreachable!("follow-ups normalized"),
            End::Parked => {
                let task = task_mut(domain, number).expect("terminal names live task");
                if !kept.owes && task.record.flags.ended_by != crate::EndedBy::Standing {
                    task.record.tries = Tries::NONE;
                }
                task.record.refusals = 0;
                if narrowed && !kept.blocks_claims() { Phase::Active(Active::Due) } else { Phase::Active(Active::Idle) }
            }
            End::Exceeded(value) => exceeded(domain, number, value),
            End::FailedWithReason { class, .. } | End::Failed(class) => failure(domain, env, number, class, reason),
            End::Refused => pause(domain, env, number, None),
            End::Recheck => {
                if kept.blocks_claims() {
                    Phase::Active(Active::Idle)
                } else {
                    Phase::Active(Active::Due)
                }
            }
        }
    };
    let next = if parked && kept.owes {
        let task = task_mut(domain, number).expect("owing task live");
        task.record.tries.add(Class::Run);
        let times = task.record.tries.of(Class::Run);
        if times > env.limits.retries.run.retries {
            Phase::Closing(Closing { stage: Stage::Delegates, ending: Ending::Failed { reason: Box::new([]) } })
        } else {
            next
        }
    } else {
        next
    };
    let task = task_mut(domain, number).expect("terminal names live task");
    task.record.invalid_result = invalid;
    if activation_ended {
        task.record.activation_open = false;
        task.record.reasons = crate::Reasons::EMPTY;
    }
    task.record.numbers.reserved =
        task.record.numbers.reserved.checked_sub(task.record.run_reserved).expect("run held");
    task.record.run_reserved = 0;
    task.record.last_answer = Some(attempt);
    task.record.narrowing = false;
    task.record.llm_changed = false;
    let spent = task.record.run_spent;
    crate::activity::append(&mut task.record, env.wall, crate::Line::RunEnded { attempt, end: end_kind, spent }, out);
    if let Some(words) = stood {
        crate::activity::append(&mut task.record, env.wall, crate::Line::Stood { attempt, words }, out);
    }
    if parked && !kept.owes {
        let pending = task.record.waiting_on.clone();
        crate::activity::append(
            &mut task.record,
            env.wall,
            crate::Line::Kept { attempt, pending, question: kept.question },
            out,
        );
    }
    if terminal_parks && activation_ended && crate::session::work_closed(&task.record) {
        crate::session::end(&mut task.record, env.wall, crate::SessionEnd::Closed, out);
    }
    if let Some(tags) = saved {
        task.record.saved = tags;
    }
    task.record.phase = match held {
        Some((why, source)) => Phase::Held { was: was(next), why, source },
        None => next,
    };
    crate::writers::answered(domain, env, number, attempt, lost_writer, out);
    if (discarded || parked || !kept.finishes_alone() && end_kind == crate::EndKind::Parked)
        && let Some(grace) = record(domain, number).expect("answered task live").grace.clone()
    {
        crate::closing::cancel_tree(domain, env, number, grace.by, &grace.reason, out);
    }
    crate::standing::after_run(domain, env, number, out);
    // The numbered reminder must precede any preparation caused by unread words.
    if !parked || !kept.owes {
        crate::wake::after_run(domain, env, number, out);
    }
    let nothing_awaited = end_kind == crate::EndKind::Parked
        && !narrowed
        && nothing_awaited(domain, record(domain, number).expect("answered task live"));
    let question = if end_kind == crate::EndKind::Parked && !narrowed {
        question_left_open(domain, record(domain, number).expect("answered task live"))
    } else {
        None
    };
    if question.is_some() {
        task_mut(domain, number).expect("idle question task").record.tries = previous_tries;
    }
    let closer = crate::closer::named(record(domain, number).expect("answered task live"));
    if nothing_awaited {
        let task = task_mut(domain, number).expect("idle task live");
        task.record.tries = previous_tries;
        if !closer {
            task.record.tries.add(Class::Run);
        }
        if task.record.tries.of(Class::Run) > env.limits.retries.run.retries {
            task.record.phase =
                Phase::Closing(Closing { stage: Stage::Delegates, ending: Ending::Failed { reason: Box::new([]) } });
        }
    }
    crate::closer::take_available(domain, env, number, out);
    publish(domain, env, number, out);
    if let Some(question) = question {
        out.push(Request::QuestionParked { task: number, question });
    }
    if nothing_awaited && record(domain, number).expect("idle task live").phase == Phase::Active(Active::Idle) {
        if closer {
            crate::closer::ask(domain, number, out);
        } else {
            out.push(Request::NothingAwaitedDue { task: number });
        }
    }
    if parked && kept.owes && record(domain, number).expect("owing task live").phase == Phase::Active(Active::Idle) {
        out.push(Request::QuestionOwedDue { task: number });
    }
    if (narrowed || end_kind == crate::EndKind::Refused)
        && record(domain, number).expect("amended task live").phase == Phase::Active(Active::Due)
    {
        crate::domain::activate(domain, number, out);
    }
    out.push(Request::Acknowledged {
        reply_to: to,
        task: number,
        attempt,
        accepted: Accepted::New,
        parked: terminal_parks,
    });
}

pub(crate) fn was(phase: Phase) -> Was {
    match phase {
        Phase::Waiting => Was::Waiting,
        Phase::Active(active) => Was::Active(active),
        Phase::Closing(closing) => Was::Closing(closing),
        Phase::Held { was, .. } => was,
        Phase::Ended(_) => unreachable!("ended task cannot be held"),
    }
}

fn wall_after(env: &Env<Limits>, duration: skein_lib::Duration) -> skein_lib::Wall {
    skein_lib::Wall::from_nanos(env.wall.as_nanos().saturating_add(duration.as_nanos()))
}

fn failure(domain: &mut Domain, env: &Env<Limits>, number: u64, class: Class, reason: Option<Box<[u8]>>) -> Phase {
    let task = task_mut(domain, number).expect("failure names live task");
    task.record.tries.add(class);
    task.record.refusals = 0;
    let times = task.record.tries.of(class);
    let retry = env.limits.retries.of(class);
    task.record.retry_notice =
        Some(crate::RetryNotice::Failed { class, count: times, allowance: retry.retries, reason: reason.clone() });
    fact(domain, Fact::Failed { task: number, class });
    if times > retry.retries {
        fact(domain, Fact::Held { task: number, why: Hold::Failures(class) });
        Phase::Held {
            was: Was::Active(Active::Due),
            why: Hold::Failures(class),
            source: match reason {
                Some(words) => Some(Box::new(crate::HoldSource::Preparation { words })),
                None => None,
            },
        }
    } else {
        let duration = crate::failures::backoff(times, retry, &mut domain.rng);
        Phase::Active(Active::BackingOff { until: wall_after(env, duration) })
    }
}

fn pause(domain: &mut Domain, env: &Env<Limits>, number: u64, reason: Option<Box<[u8]>>) -> Phase {
    let task = task_mut(domain, number).expect("pause names live task");
    task.record.refusals = task.record.refusals.saturating_add(1);
    let times = task.record.refusals;
    task.record.retry_notice = Some(crate::RetryNotice::Refused { count: times, reason });
    let duration = crate::failures::backoff(times, env.limits.retries.transient, &mut domain.rng);
    Phase::Active(Active::BackingOff { until: wall_after(env, duration) })
}

pub(crate) fn preparation_waiting(domain: &mut Domain, env: &Env<Limits>, number: u64, out: &mut Queue<Request>) {
    if domain.ready()
        && let Some(task) = record(domain, number)
        && task.phase == Phase::Active(Active::Preparing)
        && !ends::kept(domain, number).blocks_claims()
    {
        task_mut(domain, number).expect("preparing task").record.phase = Phase::Active(Active::Due);
        publish(domain, env, number, out);
        crate::domain::activate(domain, number, out);
    }
}

fn preparation_reason(reason: &[u8]) -> Box<[u8]> {
    Box::from(skein_lib::bytes::utf8_prefix(reason, skein_lib::length::usize_of(crate::REASON_BYTES)))
}

pub(crate) fn preparation_paused(
    domain: &mut Domain,
    env: &Env<Limits>,
    number: u64,
    reason: &[u8],
    out: &mut Queue<Request>,
) {
    if domain.ready()
        && let Some(task) = record(domain, number)
        && task.phase == Phase::Active(Active::Preparing)
    {
        let phase = pause(domain, env, number, Some(preparation_reason(reason)));
        task_mut(domain, number).expect("prepared task exists").record.phase = phase;
        publish(domain, env, number, out);
    }
}

pub(crate) fn preparation_counted(
    domain: &mut Domain,
    env: &Env<Limits>,
    number: u64,
    class: Class,
    reason: &[u8],
    out: &mut Queue<Request>,
) {
    if domain.ready()
        && let Some(task) = record(domain, number)
        && task.phase == Phase::Active(Active::Preparing)
    {
        let phase = failure(domain, env, number, class, Some(preparation_reason(reason)));
        task_mut(domain, number).expect("prepared task exists").record.phase = phase;
        publish(domain, env, number, out);
    }
}

pub(crate) fn hold(domain: &mut Domain, env: &Env<Limits>, number: u64, why: Hold, out: &mut Queue<Request>) {
    hold_from(domain, env, number, why, None, out);
}

pub(crate) fn hold_from(
    domain: &mut Domain,
    env: &Env<Limits>,
    number: u64,
    why: Hold,
    source: Option<Box<crate::HoldSource>>,
    out: &mut Queue<Request>,
) {
    if crate::placed::is_placed(why) {
        return;
    }
    if let Some(plan) = check_stop(domain, &crate::compound::Staged::new(0), env, number, why, source) {
        apply_stop(domain, env, plan, out);
    }
}

pub(crate) struct StopPlan {
    pub(crate) number: u64,
    pub(crate) why: Hold,
    pub(crate) attempt: Option<u64>,
    pub(crate) next: Phase,
}

pub(crate) fn check_stop(
    domain: &Domain,
    staged: &crate::compound::Staged,
    env: &Env<Limits>,
    number: u64,
    why: Hold,
    source: Option<Box<crate::HoldSource>>,
) -> Option<StopPlan> {
    if !crate::hold_origin::valid(&env.limits, why, source.as_deref()) {
        return None;
    }
    if !why.valid() {
        return None;
    }
    if !domain.ready() {
        return None;
    }
    let task = staged.record(domain, number)?;
    match task.phase {
        Phase::Held { why: current, source: ref previous, .. }
            if current == why && (source.is_none() || previous == &source) =>
        {
            return None;
        }
        Phase::Ended(_) => return None,
        Phase::Waiting | Phase::Active(_) | Phase::Closing(_) | Phase::Held { .. } => {}
    }
    let next = match task.phase.clone() {
        Phase::Held { was, source: previous, why: before } => {
            Phase::Held { was, why, source: if before == why && source.is_none() { previous } else { source } }
        }
        Phase::Waiting | Phase::Active(_) | Phase::Closing(_) | Phase::Ended(_) => {
            Phase::Held { was: was(task.phase.clone()), why, source }
        }
    };
    Some(StopPlan { number, why, attempt: run_attempt(&task.phase), next })
}

pub(crate) fn apply_stop(domain: &mut Domain, env: &Env<Limits>, plan: StopPlan, out: &mut Queue<Request>) {
    let StopPlan { number, why, attempt, next } = plan;
    if let Some(attempt) = attempt {
        out.push(Request::Stop { task: number, attempt });
    }
    task_mut(domain, number).expect("hold names live task").record.phase = next;
    publish(domain, env, number, out);
    fact(domain, Fact::Held { task: number, why });
}

fn exceeded(domain: &mut Domain, number: u64, exceeded: crate::Exceeded) -> Phase {
    let task = task_mut(domain, number).expect("exceeded names live task");
    task.record.refusals = 0;
    task.record.retry_notice = None;
    let why = Hold::Exceeded(exceeded);
    fact(domain, Fact::Held { task: number, why });
    Phase::Held { was: Was::Active(Active::Due), why, source: None }
}

/// An idle fresh-brief task polls only when nothing can bring its answer
/// (domain/tasks/lifecycle.md, 2). The received positions and the asker's credits
/// are the same predicate used by reasons and briefs.
fn question_left_open(domain: &Domain, row: &crate::TaskRecord) -> Option<crate::OpenQuestion> {
    if row.phase != Phase::Active(Active::Idle)
        || !row.wake.questions
        || row.flags.conversation
        || !ends::for_record(row).finishes_alone()
    {
        return None;
    }
    let charter = match row.executor {
        crate::Executor::Agent { charter } => charter,
        crate::Executor::Procedure { .. } | crate::Executor::Person(_) | crate::Executor::Job { .. } => return None,
    };
    let mut latest = 0;
    let mut resumes = false;
    for version in domain.charters() {
        if version.number == charter && version.revision <= row.settings.task && version.revision >= latest {
            latest = version.revision;
            resumes = version.resumes;
        }
    }
    if resumes {
        return None;
    }
    if let Some(proposal) = &row.proposal {
        match proposal.state {
            crate::ProposalState::Pending { .. } => return None,
            crate::ProposalState::Accepted { .. }
            | crate::ProposalState::Rejected { .. }
            | crate::ProposalState::Withdrawn => {}
        }
    }
    for delegate in &row.delegates {
        if let Some(child) = record(domain, *delegate) {
            match child.executor {
                crate::Executor::Person(_) => return None,
                crate::Executor::Agent { .. } | crate::Executor::Procedure { .. } | crate::Executor::Job { .. } => {}
            }
        }
    }
    for subscription in &row.subscriptions {
        match subscription.kind {
            crate::SubscriptionKind::Timer { .. } => return None,
            crate::SubscriptionKind::Task { target, .. } => {
                if record(domain, target).is_some() {
                    return None;
                }
            }
            crate::SubscriptionKind::Topic { .. } => {}
        }
    }
    domain.open_questions(row.number).get(0).copied()
}

pub(crate) fn question_parked(
    domain: &mut Domain,
    env: &Env<Limits>,
    number: u64,
    question: u64,
    reason: &[u8],
    out: &mut Queue<Request>,
) {
    let Some(row) = record(domain, number) else { return };
    let Some(open) = question_left_open(domain, row) else { return };
    if open.number != question {
        return;
    }
    if row.tries.of(Class::Run) >= env.limits.retries.run.retries {
        let phase = failure(domain, env, number, Class::Run, Some(preparation_reason(reason)));
        task_mut(domain, number).expect("idle question task").record.phase = phase;
        publish(domain, env, number, out);
    } else {
        let task = task_mut(domain, number).expect("idle question task");
        task.record.tries.add(Class::Run);
        task.record.phase = Phase::Active(Active::Due);
        publish(domain, env, number, out);
        crate::domain::activate(domain, number, out);
    }
}

/// Fixed durable notice text (domain/tasks/lifecycle.md, 2).
pub(crate) const NOTHING_AWAITED_WORDS: &[u8] =
    b"Nothing more will come: your delegates have ended and no question waits. Finish with your result.";

pub(crate) fn nothing_awaited(domain: &Domain, row: &crate::TaskRecord) -> bool {
    if row.phase != Phase::Active(Active::Idle)
        || !ends::for_record(row).finishes_alone()
        || !row.questions.is_empty()
        || !row.delegates.is_empty()
    {
        return false;
    }
    if !domain.open_questions(row.number).is_empty() {
        return false;
    }
    if let Some(proposal) = &row.proposal {
        match proposal.state {
            crate::ProposalState::Pending { .. } => return false,
            crate::ProposalState::Accepted { .. }
            | crate::ProposalState::Rejected { .. }
            | crate::ProposalState::Withdrawn => {}
        }
    }
    for subscription in &row.subscriptions {
        match subscription.kind {
            crate::SubscriptionKind::Task { target, .. } => {
                if record(domain, target).is_some() {
                    return false;
                }
            }
            crate::SubscriptionKind::Timer { .. } => return false,
            crate::SubscriptionKind::Topic { .. } => {}
        }
    }
    true
}

pub(crate) fn nothing_awaited_notice(
    domain: &mut Domain,
    env: &Env<Limits>,
    number: u64,
    message: u64,
    out: &mut Queue<Request>,
) {
    let Some(row) = record(domain, number) else { return };
    if !nothing_awaited(domain, row) || message == 0 || message <= row.last_message {
        return;
    }
    let word = crate::Word {
        number: message,
        from: crate::Party::Task(number),
        kind: crate::MessageKind::NothingAwaited,
        words: Box::from(NOTHING_AWAITED_WORDS),
        at: env.wall,
        hits: 1,
        eligible: true,
    };
    let prepared = crate::inbox::prepare(domain, &env.limits, number, word, crate::inbox::Admission::Merged)
        .expect("nothing-awaited notice shares the met slot");
    let task = task_mut(domain, number).expect("idle notice task live");
    task.record.last_message = message;
    task.record.inbox = prepared.inbox;
    crate::activity::message(&mut task.record, &prepared.activity_word, env.limits.inbox_messages, out);
    publish(domain, env, number, out);
    crate::wake::after_message(domain, env, number, prepared.previous, prepared.word, out);
}
