//! Delivery survey, durable intent, git effects and answer (domain/host.md, section 8;
//! protocol/hosts.md, section 5.5). The operation state owns marker evidence or
//! uncertainty until its one terminal. Receipts have one in-flight owner.
use super::Domain;
use super::{observe, resume_loading};
use crate::agent::command;
use crate::delivery::commit_message;
use crate::{DeliveryIntent, DeliveryRecord, DeliveryState, Fact, GitOp, GitResult, IntentDirectory, Limits, Request};
use alloc::boxed::Box;
use skein_lib::{Env, List, Queue, Time, Token};
use smith_domain::run;
use smith_host_domain::{self as host, parent};

/// One awaited git or filesystem terminal, or a transition followed immediately.
#[derive(Debug)]
enum State {
    Plain,
    Status,
    Markers { head: Box<[u8]> },
    Commit,
    Inspect { reason: run::DeliveryReason, diagnostic: Box<run::Diagnostic> },
    InspectPush,
    Push,
    Saving,
    Advance,
    Answer(Box<run::Delivery>),
    Unknown,
    Done,
}

/// Survey reads every writable directory before a commit; execution uses that snapshot.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Stage {
    Survey,
    Execute,
}

/// One child delivery call and the directories already made durable.
#[derive(Debug)]
pub(super) struct InPlace {
    name: run::CallName,
    owner: Token,
    deadline: Time,
    next: u32,
    receipts: List<run::Receipt>,
    message: Box<[u8]>,
    state: State,
    stage: Stage,
    directories: List<IntentDirectory>,
}

impl InPlace {
    fn new(name: run::CallName, owner: Token, deadline: Time, message: Box<[u8]>) -> Self {
        Self {
            name,
            owner,
            deadline,
            next: 0,
            receipts: List::with_capacity(run::MAX_DIRECTORIES),
            message,
            state: State::Status,
            stage: Stage::Survey,
            directories: List::with_capacity(run::MAX_DIRECTORIES),
        }
    }

    fn receipt(&mut self, directory: u32, text: Box<[u8]>) {
        let receipt = run::Receipt::new(directory, text).expect("bounded host receipt");
        self.receipts.push(receipt).expect("admitted directory count fits receipts");
    }

    fn directory(&mut self, directory: u32, changed: bool, head: Option<Box<[u8]>>) {
        self.directories
            .push(IntentDirectory { directory, changed, head, push: None })
            .expect("admitted directory count");
    }

    fn result(&self) -> run::Delivery {
        let receipts = self.receipts.clone().into_boxed();
        if receipts.is_empty() {
            run::Delivery::Nothing
        } else {
            run::Delivery::Delivered(run::Delivered::new(receipts).expect("ordered unique writable directories"))
        }
    }
}

pub(super) fn begin_delivery(
    domain: &mut Domain,
    env: &Env<Limits>,
    name: run::CallName,
    owner: Token,
    change: run::outcome::Change,
    deadline: Time,
    out: &mut Queue<Request>,
) {
    assert!(domain.delivery.is_none() && domain.saving.is_none(), "one delivery in flight");
    if let Some(delivery) = crate::delivery::cached(&domain.records, name) {
        deliver_to_agent(domain, env, owner, delivery);
        return;
    }
    if domain.records.get(&name).is_some() {
        deliver_unknown(domain, env, owner, 0);
        return;
    }
    let rules = match &domain.config.contract {
        crate::Contract::Change(spec) => &spec.fields,
        crate::Contract::Report(_) => match &domain.config.deliver {
            Some(spec) => &spec.fields,
            None => unreachable!("a Report delivery has a separate grant"),
        },
    };
    let Some(message) = commit_message(&change, &domain.config.title_field, rules) else {
        save_delivery(
            domain,
            name,
            owner,
            run::Delivery::Failed(run::DeliveryFailure::new(0, run::DeliveryReason::TooLarge)),
            Box::new([]),
            out,
        );
        return;
    };
    let Some(message) = crate::delivery::named_message(message, name) else {
        save_delivery(
            domain,
            name,
            owner,
            run::Delivery::Failed(run::DeliveryFailure::new(0, run::DeliveryReason::TooLarge)),
            Box::new([]),
            out,
        );
        return;
    };
    let in_place = InPlace::new(name, owner, deadline, message);
    advance_delivery(domain, env, in_place, out);
}

fn advance_delivery(domain: &mut Domain, env: &Env<Limits>, mut in_place: InPlace, out: &mut Queue<Request>) {
    let workspace = domain.config.workspace.as_ref().expect("admitted delivery has a workspace");
    match in_place.stage {
        Stage::Survey => {
            let start = usize::try_from(in_place.next).expect("bounded directory position");
            for (index, directory) in workspace.directories.iter().enumerate().skip(start) {
                if !directory.writable {
                    continue;
                }
                let position = u32::try_from(index).expect("admitted directory count fits u32");
                if env.now >= in_place.deadline {
                    save_timeout(domain, in_place, position, out);
                    return;
                }
                in_place.next = position;
                let request = if directory.git {
                    in_place.state = State::Status;
                    Request::Git {
                        owner: in_place.owner,
                        directory: position,
                        op: GitOp::Status,
                        deadline: in_place.deadline,
                    }
                } else {
                    in_place.state = State::Plain;
                    Request::PlainStatus { owner: in_place.owner, directory: position, deadline: in_place.deadline }
                };
                domain.delivery = Some(in_place);
                out.push(request);
                return;
            }
            save_intent(domain, in_place, out);
        }
        Stage::Execute => {
            for index in 0..in_place.directories.len() {
                let entry = in_place.directories.get(index).expect("surveyed directory").clone();
                if entry.directory < in_place.next || !entry.changed {
                    continue;
                }
                if env.now >= in_place.deadline {
                    save_timeout(domain, in_place, entry.directory, out);
                    return;
                }
                in_place.next = entry.directory;
                if entry.head.is_some() {
                    commit_request(domain, in_place, out);
                    return;
                }
                in_place.receipt(entry.directory, Box::from(&b"files kept"[..]));
                in_place.next = entry.directory.checked_add(1).expect("admitted directory position");
            }
            let name = in_place.name;
            let owner = in_place.owner;
            let answer = in_place.result();
            save_delivery(domain, name, owner, answer, in_place.receipts.into_boxed(), out);
        }
    }
}

fn save_intent(domain: &mut Domain, mut in_place: InPlace, out: &mut Queue<Request>) {
    let mut entries = List::with_capacity(run::MAX_DIRECTORIES);
    for index in 0..in_place.directories.len() {
        let entry = in_place.directories.get_mut(index).expect("surveyed directory");
        if entry.head.is_some() {
            entry.push = match &domain.config.push {
                Some(targets) => {
                    match targets.get(usize::try_from(entry.directory).expect("bounded directory position")) {
                        Some(target) => target.clone(),
                        None => None,
                    }
                }
                None => None,
            };
        }
        entries.push(entry.clone()).expect("admitted directory count");
    }
    let record = DeliveryRecord {
        name: in_place.name,
        state: DeliveryState::Intent(DeliveryIntent { directories: entries.into_boxed() }),
        landed: Box::new([]),
    };
    in_place.state = State::Saving;
    domain.saving = Some(record.clone());
    domain.delivery_owner = Some(in_place.owner);
    domain.delivery = Some(in_place);
    out.push(Request::SaveDelivery { record: Box::new(record) });
}

pub(super) fn plain_status(
    domain: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    changed: bool,
    out: &mut Queue<Request>,
) {
    let mut call = domain.delivery.take().expect("plain status answers one delivery operation");
    assert_eq!(call.owner, owner, "plain status matches the active request");
    let state = core::mem::replace(&mut call.state, State::Done);
    call.state = match state {
        State::Plain => {
            call.directory(call.next, changed, None);
            call.next = call.next.checked_add(1).expect("admitted directory position");
            State::Advance
        }
        State::Status
        | State::Markers { .. }
        | State::Commit
        | State::Inspect { .. }
        | State::InspectPush
        | State::Push
        | State::Saving
        | State::Advance
        | State::Answer(_)
        | State::Unknown
        | State::Done => unreachable!("plain terminal answers one plain status request"),
    };
    follow(domain, env, call, out);
}

pub(super) fn git_result(
    domain: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    result: GitResult,
    out: &mut Queue<Request>,
) {
    let mut call = domain.delivery.take().expect("git terminal answers one delivery operation");
    assert_eq!(call.owner, owner, "git terminal matches active delivery");
    let state = core::mem::replace(&mut call.state, State::Done);
    call.state = match state {
        State::Status => status(domain, env, &mut call, result, out),
        State::Markers { head } => markers(&mut call, head, result),
        State::Commit => committed(domain, env, &mut call, result, out),
        State::Inspect { reason, diagnostic } => inspected(domain, env, &mut call, reason, diagnostic, result, out),
        State::Push => pushed(domain, env, &mut call, result, out),
        State::InspectPush => match result {
            GitResult::Inspected { .. } | GitResult::Failed { .. } | GitResult::Uncertain { .. } => State::Unknown,
            GitResult::Head { .. }
            | GitResult::Status { .. }
            | GitResult::Markers { .. }
            | GitResult::Committed { .. }
            | GitResult::Pushed
            | GitResult::Stale
            | GitResult::NoEffect { .. } => unreachable!("uncertain push awaits inspection"),
        },
        State::Plain | State::Saving | State::Advance | State::Answer(_) | State::Unknown | State::Done => {
            unreachable!("git terminal answers an issued git request")
        }
    };
    follow(domain, env, call, out);
}

fn status(
    domain: &Domain,
    env: &Env<Limits>,
    call: &mut InPlace,
    result: GitResult,
    out: &mut Queue<Request>,
) -> State {
    match result {
        GitResult::Status { changed, merging, head } => {
            if stale_head(domain, call.next, &head) {
                return State::Answer(Box::new(run::Delivery::Stale));
            }
            if head.is_empty() || head.len() > run::Receipt::CAPACITY {
                return failure(call.next, run::DeliveryReason::TooLarge);
            }
            match merging {
                Some(paths) => {
                    let within_count =
                        paths.len() <= usize::try_from(env.limits.smith.run.conflicts).expect("u32 fits usize");
                    let mut within_bytes = true;
                    for path in &paths {
                        if path.len()
                            > usize::try_from(env.limits.smith.run.conflict_path_bytes).expect("u32 fits usize")
                        {
                            within_bytes = false;
                        }
                    }
                    if !within_count || !within_bytes {
                        return failure(call.next, run::DeliveryReason::TooLarge);
                    }
                    out.push(Request::Git {
                        owner: call.owner,
                        directory: call.next,
                        op: GitOp::Markers { paths },
                        deadline: call.deadline,
                    });
                    State::Markers { head }
                }
                None => {
                    call.directory(call.next, changed, Some(head));
                    call.next = call.next.checked_add(1).expect("admitted directory position");
                    State::Advance
                }
            }
        }
        GitResult::Failed { reason, diagnostic } | GitResult::NoEffect { reason, diagnostic } => {
            failed(call.next, reason, &diagnostic)
        }
        GitResult::Head { .. }
        | GitResult::Markers { .. }
        | GitResult::Committed { .. }
        | GitResult::Pushed
        | GitResult::Stale
        | GitResult::Inspected { .. }
        | GitResult::Uncertain { .. } => unreachable!("status awaits a status terminal"),
    }
}

fn markers(call: &mut InPlace, head: Box<[u8]>, result: GitResult) -> State {
    match result {
        GitResult::Markers { first: None } => {
            call.directory(call.next, true, Some(head));
            call.next = call.next.checked_add(1).expect("admitted directory position");
            State::Advance
        }
        GitResult::Markers { first: Some(path) } => {
            let Some(marker) = run::Marker::new(call.next, path) else {
                return failure(call.next, run::DeliveryReason::TooLarge);
            };
            let refusal = run::DeliveryRefusal::new(Some(marker), Box::from(&b"conflict markers remain"[..]))
                .expect("bounded host explanation");
            State::Answer(Box::new(run::Delivery::Refused(refusal)))
        }
        GitResult::Failed { reason, diagnostic } | GitResult::NoEffect { reason, diagnostic } => {
            failed(call.next, reason, &diagnostic)
        }
        GitResult::Head { .. }
        | GitResult::Status { .. }
        | GitResult::Committed { .. }
        | GitResult::Pushed
        | GitResult::Stale
        | GitResult::Inspected { .. }
        | GitResult::Uncertain { .. } => unreachable!("markers await a marker terminal"),
    }
}

fn committed(
    domain: &mut Domain,
    env: &Env<Limits>,
    call: &mut InPlace,
    result: GitResult,
    out: &mut Queue<Request>,
) -> State {
    match result {
        GitResult::Committed { receipt, head } => {
            if receipt.is_empty() || receipt.len() > run::Receipt::CAPACITY {
                return failure(call.next, run::DeliveryReason::TooLarge);
            }
            assert!(!head.is_empty() && head.len() <= run::Receipt::CAPACITY, "commit head is bounded");
            domain.own_heads.insert(call.next, head).expect("one head per writable repository");
            call.receipt(call.next, receipt);
            let mut target = None;
            for entry in &call.directories {
                if entry.directory == call.next {
                    target.clone_from(&entry.push);
                }
            }
            match target {
                Some(target) => {
                    out.push(Request::Git {
                        owner: call.owner,
                        directory: call.next,
                        op: GitOp::Push { remote: target.remote, branch: target.branch },
                        deadline: call.deadline,
                    });
                    State::Push
                }
                None => {
                    call.next = call.next.checked_add(1).expect("admitted directory position");
                    State::Advance
                }
            }
        }
        GitResult::Failed { reason, diagnostic } | GitResult::NoEffect { reason, diagnostic } => {
            failed(call.next, reason, &diagnostic)
        }
        GitResult::Uncertain { reason, diagnostic } => {
            inspect_request(domain, env, call, out);
            State::Inspect { reason, diagnostic }
        }
        GitResult::Head { .. }
        | GitResult::Status { .. }
        | GitResult::Markers { .. }
        | GitResult::Pushed
        | GitResult::Stale
        | GitResult::Inspected { .. } => unreachable!("commit awaits a commit terminal"),
    }
}

fn inspected(
    domain: &mut Domain,
    env: &Env<Limits>,
    call: &mut InPlace,
    reason: run::DeliveryReason,
    diagnostic: Box<run::Diagnostic>,
    result: GitResult,
    out: &mut Queue<Request>,
) -> State {
    match result {
        GitResult::Inspected { head, named: true } => {
            let Some(receipt) = crate::commit_receipt(&head) else {
                return failure(call.next, run::DeliveryReason::TooLarge);
            };
            committed(domain, env, call, GitResult::Committed { receipt, head }, out)
        }
        GitResult::Inspected { head, named: false } => {
            let mut unchanged = false;
            for entry in &call.directories {
                if entry.directory == call.next && entry.head.as_deref() == Some(head.as_ref()) {
                    unchanged = true;
                }
            }
            if unchanged { failed(call.next, reason, &diagnostic) } else { State::Unknown }
        }
        GitResult::Failed { .. } | GitResult::Uncertain { .. } => State::Unknown,
        GitResult::Head { .. }
        | GitResult::Status { .. }
        | GitResult::Markers { .. }
        | GitResult::Committed { .. }
        | GitResult::Pushed
        | GitResult::Stale
        | GitResult::NoEffect { .. } => unreachable!("uncertain write awaits inspection"),
    }
}

fn pushed(
    domain: &Domain,
    env: &Env<Limits>,
    call: &mut InPlace,
    result: GitResult,
    out: &mut Queue<Request>,
) -> State {
    match result {
        GitResult::Uncertain { .. } => {
            inspect_request(domain, env, call, out);
            State::InspectPush
        }
        GitResult::Pushed => {
            call.next = call.next.checked_add(1).expect("admitted directory position");
            State::Advance
        }
        GitResult::Stale => State::Answer(Box::new(run::Delivery::Stale)),
        GitResult::Failed { reason, diagnostic } | GitResult::NoEffect { reason, diagnostic } => {
            failed(call.next, reason, &diagnostic)
        }
        GitResult::Head { .. }
        | GitResult::Status { .. }
        | GitResult::Markers { .. }
        | GitResult::Committed { .. }
        | GitResult::Inspected { .. } => unreachable!("push awaits a push terminal"),
    }
}

fn inspect_request(domain: &Domain, env: &Env<Limits>, call: &InPlace, out: &mut Queue<Request>) {
    out.push(Request::Git {
        owner: call.owner,
        directory: call.next,
        op: GitOp::Inspect { name: call.name },
        deadline: env.now.saturating_add(domain.config.budget.time),
    });
}

fn failure(directory: u32, reason: run::DeliveryReason) -> State {
    failed(directory, reason, &run::Diagnostic::empty())
}

fn failed(directory: u32, reason: run::DeliveryReason, diagnostic: &run::Diagnostic) -> State {
    State::Answer(Box::new(run::Delivery::Failed(run::DeliveryFailure { directory, reason, diagnostic: *diagnostic })))
}

fn follow(domain: &mut Domain, env: &Env<Limits>, mut call: InPlace, out: &mut Queue<Request>) {
    let state = core::mem::replace(&mut call.state, State::Done);
    match state {
        State::Advance => advance_delivery(domain, env, call, out),
        State::Answer(answer) => save_delivery(domain, call.name, call.owner, *answer, call.receipts.into_boxed(), out),
        State::Unknown => deliver_unknown(domain, env, call.owner, call.next),
        state @ (State::Plain
        | State::Status
        | State::Markers { .. }
        | State::Commit
        | State::Inspect { .. }
        | State::InspectPush
        | State::Push
        | State::Saving) => {
            call.state = state;
            domain.delivery = Some(call);
        }
        State::Done => unreachable!("each delivery transition yields its next state"),
    }
}

fn commit_request(domain: &mut Domain, mut in_place: InPlace, out: &mut Queue<Request>) {
    in_place.state = State::Commit;
    out.push(Request::Git {
        owner: in_place.owner,
        directory: in_place.next,
        op: GitOp::Commit { message: in_place.message.clone() },
        deadline: in_place.deadline,
    });
    domain.delivery = Some(in_place);
}

fn stale_head(domain: &Domain, directory: u32, head: &[u8]) -> bool {
    let start = domain.heads.get(&directory).expect("writable repository head known before run");
    if start.as_ref() == head {
        return false;
    }
    match domain.own_heads.get(&directory) {
        Some(own) => own.as_ref() != head,
        None => true,
    }
}

fn deliver_unknown(domain: &mut Domain, env: &Env<Limits>, owner: Token, directory: u32) {
    domain.delivery = None;
    deliver_to_agent(
        domain,
        env,
        owner,
        run::Delivery::Failed(run::DeliveryFailure::new(directory, run::DeliveryReason::Unknown)),
    );
}

fn save_timeout(domain: &mut Domain, call: InPlace, directory: u32, out: &mut Queue<Request>) {
    save_delivery(
        domain,
        call.name,
        call.owner,
        run::Delivery::Failed(run::DeliveryFailure::new(directory, run::DeliveryReason::TimedOut)),
        call.receipts.into_boxed(),
        out,
    );
}

fn save_delivery(
    domain: &mut Domain,
    name: run::CallName,
    owner: Token,
    delivery: run::Delivery,
    landed: Box<[run::Receipt]>,
    out: &mut Queue<Request>,
) {
    let record = DeliveryRecord { name, state: DeliveryState::Answer(delivery), landed };
    assert!(domain.saving.is_none(), "one delivery store operation in flight");
    domain.saving = Some(record.clone());
    domain.delivery_owner = Some(owner);
    domain.delivery = None;
    out.push(Request::SaveDelivery { record: Box::new(record) });
}

pub(super) fn delivery_saved(domain: &mut Domain, env: &Env<Limits>, name: run::CallName, out: &mut Queue<Request>) {
    let record = domain.saving.take().expect("DeliverySaved answers one SaveDelivery");
    assert_eq!(record.name, name, "delivery store terminal names the decision");
    domain.records.insert(record.name, record.clone()).expect("one turn's calls fit the bound");
    match record.state {
        DeliveryState::Intent(_) => {
            let mut in_place = domain.delivery.take().expect("saved intent retains delivery");
            in_place.stage = Stage::Execute;
            in_place.next = 0;
            advance_delivery(domain, env, in_place, out);
        }
        DeliveryState::Answer(delivery) => {
            if let Some(owner) = domain.delivery_owner.take() {
                observe(domain, Fact::DeliveryReturned { name });
                deliver_to_agent(domain, env, owner, delivery);
            } else {
                resume_loading(domain, env, out);
            }
        }
    }
}

pub(super) fn deliver_to_agent(domain: &mut Domain, env: &Env<Limits>, owner: Token, delivery: run::Delivery) {
    let reply = match crate::agent::delivery_to_host(delivery) {
        Ok(delivery) => host::Reply::Delivery(delivery),
        Err(_) => host::Reply::Unavailable,
    };
    command(
        &mut domain.agent,
        env.now,
        env.wall,
        parent::Event::Answer { agent: domain.run.expect("delivery belongs to the started child"), call: owner, reply },
    );
}
