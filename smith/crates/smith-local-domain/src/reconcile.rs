//! Recovery of saved delivery intents (protocol/hosts.md, section 5.5).
//! Only inspected commit evidence can establish a landed effect; remote uncertainty
//! retains the intent rather than claiming a successful delivery.
use super::{Domain, Phase};
use crate::{DeliveryIntent, DeliveryRecord, DeliveryState, GitOp, GitResult, IntentDirectory, Limits, Request};
use alloc::boxed::Box;
use skein_lib::{Env, List, Queue, Token};
use smith_domain as agent;

#[derive(Debug)]
enum State {
    Inspect { entry: IntentDirectory },
    Push { directory: u32 },
    InspectPush,
    Advance,
    Interrupted { directory: u32 },
    Answer(Box<agent::run::Delivery>),
    Unknown,
    Done,
}

#[derive(Debug)]
pub(super) struct Reconcile {
    record: DeliveryRecord,
    next: u32,
    receipts: List<agent::run::Receipt>,
    state: State,
}

impl Reconcile {
    pub(super) fn new(record: DeliveryRecord) -> Self {
        Self { record, next: 0, receipts: List::with_capacity(agent::run::MAX_DIRECTORIES), state: State::Done }
    }

    fn intent(&self) -> &DeliveryIntent {
        match &self.record.state {
            DeliveryState::Intent(intent) => intent,
            DeliveryState::Answer(_) => unreachable!("only an intent is reconciled"),
        }
    }

    fn receipt(&mut self, directory: u32, text: Box<[u8]>) {
        self.receipts
            .push(agent::run::Receipt::new(directory, text).expect("bounded receipt"))
            .expect("admitted directory count");
    }
}

pub(super) fn reconcile_next(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let mut reconcile = domain.reconciling.take().expect("loaded intent is being reconciled");
    let start = usize::try_from(reconcile.next).expect("bounded directory position");
    for index in start..reconcile.intent().directories.len() {
        let entry = reconcile.intent().directories.get(index).expect("position in saved intent").clone();
        reconcile.next = u32::try_from(index).expect("admitted directory count");
        if !entry.changed {
            continue;
        }
        match &entry.head {
            Some(_) => {
                let name = reconcile.record.name;
                let directory = entry.directory;
                reconcile.state = State::Inspect { entry };
                domain.reconciling = Some(reconcile);
                out.push(Request::Git {
                    owner: Token::new(0),
                    directory,
                    op: GitOp::Inspect { name },
                    deadline: env.now.saturating_add(domain.config.budget.time),
                });
                return;
            }
            None => reconcile.receipt(entry.directory, Box::from(&b"files kept"[..])),
        }
    }
    reconcile_answer(domain, reconcile, None, out);
}

pub(super) fn reconcile_git(
    domain: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    result: GitResult,
    out: &mut Queue<Request>,
) {
    assert_eq!(owner, Token::new(0), "recovery inspection has its own owner");
    let mut reconcile = domain.reconciling.take().expect("inspection answers a loaded intent");
    let state = core::mem::replace(&mut reconcile.state, State::Done);
    reconcile.state = match state {
        State::Inspect { entry } => inspected(domain, env, &mut reconcile, entry, result, out),
        State::Push { directory } => pushed(domain, env, &mut reconcile, directory, result, out),
        State::InspectPush => State::Unknown,
        State::Advance | State::Interrupted { .. } | State::Answer(_) | State::Unknown | State::Done => {
            unreachable!("recovery terminal answers an issued git request")
        }
    };
    follow(domain, env, reconcile, out);
}

fn pushed(
    domain: &Domain,
    env: &Env<Limits>,
    reconcile: &mut Reconcile,
    directory: u32,
    result: GitResult,
    out: &mut Queue<Request>,
) -> State {
    match result {
        GitResult::Pushed => {
            reconcile.next = reconcile.next.checked_add(1).expect("admitted position");
            State::Advance
        }
        GitResult::Stale => State::Answer(Box::new(agent::run::Delivery::Stale)),
        GitResult::Failed { reason, diagnostic } | GitResult::NoEffect { reason, diagnostic } => {
            State::Answer(Box::new(agent::run::Delivery::Failed(agent::run::DeliveryFailure {
                directory,
                reason,
                diagnostic: *diagnostic,
            })))
        }
        GitResult::Uncertain { .. } => {
            out.push(Request::Git {
                owner: Token::new(0),
                directory,
                op: GitOp::Inspect { name: reconcile.record.name },
                deadline: env.now.saturating_add(domain.config.budget.time),
            });
            State::InspectPush
        }
        GitResult::Head { .. }
        | GitResult::Status { .. }
        | GitResult::Inspected { .. }
        | GitResult::Markers { .. }
        | GitResult::Committed { .. } => unreachable!("recovery awaits one push terminal"),
    }
}

fn inspected(
    domain: &Domain,
    env: &Env<Limits>,
    reconcile: &mut Reconcile,
    entry: IntentDirectory,
    result: GitResult,
    out: &mut Queue<Request>,
) -> State {
    let before = entry.head.expect("inspection applies to a git directory");
    match result {
        GitResult::Inspected { head, named } => {
            if named
                && !head.is_empty()
                && head.len() <= agent::run::Receipt::CAPACITY.checked_sub(7).expect("receipt prefix fits")
            {
                let text = crate::commit_receipt(&head).expect("bounded named head fits a receipt");
                reconcile.receipt(entry.directory, text);
                match entry.push {
                    Some(target) => {
                        out.push(Request::Git {
                            owner: Token::new(0),
                            directory: entry.directory,
                            op: GitOp::Push { remote: target.remote, branch: target.branch },
                            deadline: env.now.saturating_add(domain.config.budget.time),
                        });
                        State::Push { directory: entry.directory }
                    }
                    None => {
                        reconcile.next = reconcile.next.checked_add(1).expect("admitted position");
                        State::Advance
                    }
                }
            } else if head == before && !named {
                State::Interrupted { directory: entry.directory }
            } else {
                State::Unknown
            }
        }
        GitResult::Failed { .. } | GitResult::Uncertain { .. } => State::Unknown,
        GitResult::Head { .. }
        | GitResult::Status { .. }
        | GitResult::Markers { .. }
        | GitResult::Committed { .. }
        | GitResult::Pushed
        | GitResult::Stale
        | GitResult::NoEffect { .. } => unreachable!("recovery awaits one inspection terminal"),
    }
}

fn follow(domain: &mut Domain, env: &Env<Limits>, mut reconcile: Reconcile, out: &mut Queue<Request>) {
    let state = core::mem::replace(&mut reconcile.state, State::Done);
    match state {
        State::Advance => {
            domain.reconciling = Some(reconcile);
            reconcile_next(domain, env, out);
        }
        State::Interrupted { directory } => reconcile_answer(domain, reconcile, Some(directory), out),
        State::Answer(answer) => reconcile_terminal(domain, reconcile, *answer, out),
        State::Unknown => recovery_unknown(domain, out),
        state @ (State::Inspect { .. } | State::Push { .. } | State::InspectPush) => {
            reconcile.state = state;
            domain.reconciling = Some(reconcile);
        }
        State::Done => unreachable!("each recovery transition yields its next state"),
    }
}

fn recovery_unknown(domain: &mut Domain, out: &mut Queue<Request>) {
    domain.stop_failed = true;
    crate::person::show_dropped(&mut domain.lines, out);
    domain.chat.phase = Phase::Done;
    out.push(Request::Exit { status: crate::end::exit_status(domain.end_cause) });
}

fn reconcile_answer(domain: &mut Domain, reconcile: Reconcile, interrupted: Option<u32>, out: &mut Queue<Request>) {
    let mut landed = List::with_capacity(agent::run::MAX_DIRECTORIES);
    for receipt in &reconcile.receipts {
        landed.push(receipt.clone()).expect("admitted directories");
    }
    let answer = match interrupted {
        None => {
            let receipts = reconcile.receipts.into_boxed();
            if receipts.is_empty() {
                agent::run::Delivery::Nothing
            } else {
                agent::run::Delivery::Delivered(
                    agent::run::Delivered::new(receipts).expect("ordered unique directories"),
                )
            }
        }
        Some(directory) => {
            let mut text = List::with_capacity(512);
            crate::delivery::append_bounded(&mut text, b"delivery interrupted; committed: ");
            if reconcile.receipts.is_empty() {
                crate::delivery::append_bounded(&mut text, b"none");
            } else {
                for receipt in &reconcile.receipts {
                    crate::delivery::append_bounded(&mut text, b"directory ");
                    crate::delivery::decimal(&mut text, u64::from(receipt.directory()));
                    crate::delivery::append_bounded(&mut text, b": ");
                    crate::delivery::append_bounded(&mut text, receipt.text());
                    crate::delivery::append_bounded(&mut text, b"; ");
                }
            }
            agent::run::Delivery::Failed(agent::run::DeliveryFailure {
                directory,
                reason: agent::run::DeliveryReason::Broken,
                diagnostic: agent::run::Diagnostic::new(&text.into_boxed(), 0),
            })
        }
    };
    reconcile_terminal_with_receipts(domain, reconcile.record.name, answer, landed.into_boxed(), out);
}

fn reconcile_terminal(
    domain: &mut Domain,
    reconcile: Reconcile,
    answer: agent::run::Delivery,
    out: &mut Queue<Request>,
) {
    reconcile_terminal_with_receipts(domain, reconcile.record.name, answer, reconcile.receipts.into_boxed(), out);
}

fn reconcile_terminal_with_receipts(
    domain: &mut Domain,
    name: agent::run::CallName,
    answer: agent::run::Delivery,
    landed: Box<[agent::run::Receipt]>,
    out: &mut Queue<Request>,
) {
    let record = DeliveryRecord { name, state: DeliveryState::Answer(answer), landed };
    domain.saving = Some(record.clone());
    out.push(Request::SaveDelivery { record: Box::new(record) });
}
