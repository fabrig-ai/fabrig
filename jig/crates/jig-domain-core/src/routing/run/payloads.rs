//! Turn and answer admission fences (domain/engine/executors.md, 2 and 4).
use crate::{Core, Event, Request, routing::Work};
use alloc::boxed::Box;
use jig_domain_tasks as tasks;
use skein_lib::{Queue, ReplyTo, Token};

pub(crate) struct Answer {
    pub payload: Token,
    pub cumulative: u64,
    pub end: tasks::End,
    pub saved: Option<Box<[tasks::SavedResource]>>,
    pub invalid_saved: bool,
    pub unusable_history: bool,
    pub read: Option<u64>,
    pub refused: Box<[u64]>,
}

pub(crate) fn route_answer_payload(
    core: &mut Core,
    limits: &tasks::Limits,
    work: &mut Queue<Work>,
    run: Token,
    attempt: Token,
    answer: Answer,
) -> Queue<Request> {
    let Answer { payload, cumulative, end, saved, invalid_saved, unusable_history, read, refused } = answer;
    let refused = refused.as_ref();
    if !core.current_proof(run.raw(), attempt.raw()) {
        let mut out = Queue::uncapped();
        out.push(Request::Now(Box::new(crate::Now::Link(crate::LinkNow::DropPayload { payload }))));
        return out;
    }
    let (cumulative, end, saved) = match core.committed_terminal(run.raw(), attempt.raw()) {
        Some(terminal) => (terminal.cumulative, terminal.end, None),
        None => {
            let proof = core.proofs.get(&run.raw()).expect("current proof");
            let last_read = match proof.turn {
                Some(turn) => turn.read,
                None => None,
            };
            let mut invalid = read > last_read;
            for number in refused {
                invalid |= *number == 0 || Some(*number) > proof.offered;
            }
            if invalid_saved || invalid || past_batch_bytes(limits, &end) {
                (cumulative, tasks::End::Failed(tasks::Class::Invalid), None)
            } else {
                if !refused.is_empty() {
                    work.push(Work::Event(Event::Tasks(tasks::Event::Refusals {
                        task: run.raw(),
                        attempt: attempt.raw(),
                        refused: Box::from(refused),
                        offered: proof.offered,
                    })));
                }
                if unusable_history {
                    // Only this window's history closes it (jig/docs/design/domain/engine/executors.md, 2).
                    core.close_window(run.raw());
                }
                (cumulative, end, saved)
            }
        }
    };
    let proof = core.proofs.get(&run.raw()).expect("current proof");
    let account_wait = proof.account_terminal
        || match proof.account_failure {
            Some(source) => match core.accounts.source(source) {
                Some(fact) => !fact.usable,
                None => true,
            },
            None => false,
        };
    let proof = core.proofs.get_mut(&run.raw()).expect("terminal proof pre-reserved");
    let end = match end {
        tasks::End::Failed(tasks::Class::Transient | tasks::Class::Infrastructure | tasks::Class::Permanent | tasks::Class::Run)
        | tasks::End::FailedWithReason {
            class: tasks::Class::Transient | tasks::Class::Infrastructure | tasks::Class::Permanent | tasks::Class::Run,
            ..
        } if account_wait && !unusable_history => tasks::End::Recheck,
        tasks::End::Finished { .. }
        | tasks::End::FinishedWithFollowups { .. }
        | tasks::End::Exceeded(_)
        | tasks::End::Parked
        | tasks::End::Failed(_)
        | tasks::End::FailedWithReason { .. }
        | tasks::End::Refused
        | tasks::End::Recheck => end,
    };
    proof.terminal =
        Some(crate::TerminalRecord { task: run.raw(), attempt: attempt.raw(), cumulative, end: end.clone() });
    work.push(Work::Event(Event::Tasks(tasks::Event::Activation {
        reply_to: ReplyTo::new(payload),
        task: run.raw(),
        attempt: attempt.raw(),
        cause: tasks::Cause::Priced { cumulative },
        end,
        saved,
    })));
    Queue::uncapped()
}

fn past_batch_bytes(limits: &tasks::Limits, end: &tasks::End) -> bool {
    if let tasks::End::FailedWithReason { reason, .. } = end
        && (reason.len() > skein_lib::usize_of(tasks::REASON_BYTES) || !skein_lib::bytes::is_utf8(reason))
    {
        return true;
    }
    let batch = match end {
        tasks::End::FinishedWithFollowups { followups, .. } => match followups {
            tasks::ResultFollowups::Delegates(batch) => Some(batch),
            tasks::ResultFollowups::Proposal(proposal) => match &proposal.action {
                tasks::ProposalAction::Batch(batch) => Some(batch),
                tasks::ProposalAction::Settings { .. }
                | tasks::ProposalAction::Effect { .. }
                | tasks::ProposalAction::ProcedureEffect { .. }
                | tasks::ProposalAction::Amend { .. }
                | tasks::ProposalAction::Widen { .. }
                | tasks::ProposalAction::Release { .. }
                | tasks::ProposalAction::ReleasePlaced { .. } => None,
            },
        },
        tasks::End::Finished { .. }
        | tasks::End::Exceeded(_)
        | tasks::End::Parked
        | tasks::End::Failed(_)
        | tasks::End::FailedWithReason { .. }
        | tasks::End::Refused
        | tasks::End::Recheck => None,
    };
    match batch {
        Some(batch) => match tasks::batch_bytes(batch) {
            Some(bytes) => bytes > u64::from(limits.batch_bytes),
            None => true,
        },
        None => false,
    }
}

pub(crate) fn route_turn_payload(
    core: &mut Core,
    work: &mut Queue<Work>,
    run: Token,
    attempt: Token,
    turn: crate::TurnProof,
    body: Token,
    refused: Box<[u64]>,
) -> Queue<Request> {
    let crate::TurnProof { turn, cumulative, read } = turn;
    assert!(core.current_proof(run.raw(), attempt.raw()), "current turn has reserved proof");
    let proof = core.proofs.get(&run.raw()).expect("current proof");
    let offered = proof.offered;
    let last_read = match proof.turn {
        Some(turn) => turn.read,
        None => None,
    };
    if read < last_read || read > last_read && !proof.outstanding.contains(&read.expect("new read is present")) {
        let mut out = Queue::uncapped();
        out.push(Request::Now(Box::new(crate::Now::Link(crate::LinkNow::RefusedPayload {
            request: body,
            problem: tasks::Problem {
                task: Some(run.raw()),
                why: tasks::Refusal::Read,
                blocked_by: None,
                capacity: None,
            },
        }))));
        return out;
    }
    // The parent retains the original fence with the accepted-turn payload.
    let read = if read == last_read { None } else { read };
    work.push(Work::Event(Event::Tasks(tasks::Event::Turn {
        reply_to: ReplyTo::new(body),
        task: run.raw(),
        attempt: attempt.raw(),
        turn,
        read,
        refused,
        offered,
        outstanding: proof.outstanding.clone(),
        cumulative,
    })));
    Queue::uncapped()
}
