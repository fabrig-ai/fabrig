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
    pub untooled: bool,
    pub read: Option<u64>,
    pub refused: Box<[u64]>,
}

/// An account terminal whose source is usable again takes a refusal pause (jig/docs/design/domain/tasks/lifecycle.md, 6).
pub(crate) fn route_answer_payload(
    core: &mut Core,
    limits: &tasks::Limits,
    work: &mut Queue<Work>,
    run: Token,
    attempt: Token,
    answer: Answer,
) -> Queue<Request> {
    let Answer { payload, cumulative, end, saved, invalid_saved, unusable_history, untooled, read, refused } = answer;
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
                let end = if untooled {
                    match untooled_place(core, run, attempt) {
                        Some((reason, fault)) => {
                            work.push(Work::Event(Event::Fleet(fault)));
                            tasks::End::RefusedWithReason { reason }
                        }
                        None => tasks::End::Failed(tasks::Class::Run),
                    }
                } else {
                    end
                };
                (cumulative, end, saved)
            }
        }
    };
    let proof = core.proofs.get(&run.raw()).expect("current proof");
    let source = match proof.account_failure {
        Some(source) => core.accounts.source(source),
        None => None,
    };
    let account_wait = proof.account_terminal
        || match source {
            Some(fact) => !fact.usable,
            None => proof.account_failure.is_some(),
        };
    let end = match end {
        tasks::End::Failed(
            tasks::Class::Transient | tasks::Class::Infrastructure | tasks::Class::Permanent | tasks::Class::Run,
        )
        | tasks::End::FailedWithReason {
            class: tasks::Class::Transient | tasks::Class::Infrastructure | tasks::Class::Permanent | tasks::Class::Run,
            ..
        } if account_wait && !unusable_history => match source {
            Some(fact) if proof.account_terminal && fact.usable => {
                tasks::End::RefusedWithReason { reason: account_terminal_reason(fact.account) }
            }
            Some(_) | None => tasks::End::Recheck,
        },
        tasks::End::Finished { .. }
        | tasks::End::FinishedWithFollowups { .. }
        | tasks::End::Exceeded(_)
        | tasks::End::Parked
        | tasks::End::Failed(_)
        | tasks::End::FailedWithReason { .. }
        | tasks::End::Refused
        | tasks::End::RefusedWithReason { .. }
        | tasks::End::Suspended(_)
        | tasks::End::Recheck => end,
    };
    let proof = core.proofs.get_mut(&run.raw()).expect("terminal proof pre-reserved");
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

/// Fixed words and a u32 account number fit within `REASON_BYTES` (domain/tasks/lifecycle.md, 6).
fn account_terminal_reason(account: u32) -> Box<[u8]> {
    let number = skein_lib::Decimal::of(u64::from(account));
    let parts = [b"account ".as_slice(), number.as_bytes(), b"'s source ended the run and is usable again"];
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("bounded account terminal reason");
    }
    let mut reason = skein_lib::Writer::new(length);
    for part in parts {
        reason.put(part).expect("measured account terminal reason");
    }
    reason.finish()
}

/// The main model's program binding and the answering claim's current channel, including adoption.
fn untooled_place(core: &Core, run: Token, attempt: Token) -> Option<(Box<[u8]>, jig_domain_fleet::Event)> {
    use jig_domain_fleet as fleet;
    let proof = core.proofs.get(&run.raw())?;
    let policy = core.chosen_policy(run.raw());
    let program = policy.model.program.as_ref()?;
    let mut source = None;
    for binding in &proof.accounts {
        if binding.provider == policy.model.provider
            && core.accounts.source(binding.source)?.kind == jig_domain_accounts::Kind::Program
        {
            let account = core.accounts.account(binding.account)?;
            source = Some(
                jig_domain_link::SourceName::new(jig_domain_accounts::programs::name(
                    &core.accounts,
                    binding.source,
                    account.holder(),
                )?)
                .expect("issued program source name"),
            );
        }
    }
    let source = source?;
    let mut channel = proof.placement;
    for claim in &core.fleet.attempt_views() {
        if claim.run == run && claim.attempt == attempt {
            channel = match claim.placement {
                fleet::Placement::Connected { channel, .. } => Some(channel),
                fleet::Placement::Adrift { channel, .. } => channel,
                fleet::Placement::None => None,
            };
        }
    }
    let channel = channel?;
    let mut worker = None;
    for host in &core.fleet.host_views() {
        if host.channel == channel {
            worker = host.worker;
        }
    }
    let worker = worker?;
    let number = skein_lib::Decimal::of(u64::from(worker).checked_add(1).expect("configured worker number"));
    let fallback = [b"worker ".as_slice(), number.as_bytes()];
    let mut unnamed = skein_lib::Writer::new(7_usize.checked_add(number.as_bytes().len()).expect("worker name length"));
    for part in fallback {
        unnamed.put(part).expect("worker number fits");
    }
    let unnamed = unnamed.finish();
    let worker = core.fleet.worker_name(worker).unwrap_or(&unnamed);
    let parts =
        [program.as_ref(), b" started without smith's tools: account ", source.as_bytes(), b" on worker ", worker];
    let mut length = 0_usize;
    for part in parts {
        length = length.checked_add(part.len()).expect("bounded place names");
    }
    let mut text = skein_lib::Writer::new(length);
    for part in parts {
        text.put(part).expect("measured place words");
    }
    let text = text.finish();
    Some((place_reason(&text), fleet::Event::Untooled { run, channel, source }))
}

/// Diagnostic heads keep the same character boundary and marker as invalid starts (hosts/hub.md, 3).
fn place_reason(words: &[u8]) -> Box<[u8]> {
    let mut text = skein_lib::Writer::new(words.len());
    let utf8 = skein_lib::bytes::is_utf8(words);
    for byte in words {
        text.put(&[if utf8 || byte.is_ascii() { *byte } else { b'?' }]).expect("one byte per word byte");
    }
    let text = text.finish();
    let bound = skein_lib::usize_of(tasks::REASON_BYTES);
    if text.len() <= bound {
        return text;
    }
    let marker = b" [cut]";
    let head = skein_lib::bytes::utf8_prefix(&text, bound.saturating_sub(marker.len()));
    let mut reason = skein_lib::Writer::new(head.len().checked_add(marker.len()).expect("bounded reason"));
    reason.put(head).expect("head leaves room for cut marker");
    reason.put(marker).expect("marker fits");
    reason.finish()
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
        | tasks::End::RefusedWithReason { .. }
        | tasks::End::Suspended(_)
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
