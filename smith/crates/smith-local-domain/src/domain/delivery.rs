//! Delivery survey, durable intent, git effects and answer (domain/host.md, section 8;
//! protocol/host.md, section 5.5). The operation state owns marker evidence or
//! uncertainty until its one terminal. Delivering keeps the live call, loaded intent
//! or store operation and its continuation, never an owner without its call.
//! Entrances route one terminal and follow advances transient states; git effects
//! remain unknown until inspected evidence settles them.
use super::Domain;
use super::reconcile::{self, Reconcile};
use super::{observe, resume_loading};
use crate::agent::command;
use crate::delivery::commit_message;
use crate::{
    DeliveryIntent, DeliveryRecord, DeliveryState, Fact, Failure, GitOp, GitResult, IntentDirectory, Limits, Notice,
    Request,
};
use alloc::boxed::Box;
use skein_lib::{Env, List, Queue, Time, Token};
use smith_domain::run;
use smith_host_domain::{self as host, parent};

/// The one delivery this host holds, live or loaded, and its one store operation.
#[derive(Debug)]
pub(super) enum Delivering {
    /// No delivery in the host's hands.
    Idle,
    /// A live call, awaiting one git or filesystem terminal.
    InPlace(InPlace),
    /// A loaded intent, awaiting one git terminal.
    Reconciling(Reconcile),
    /// One record awaiting its store terminal.
    Saving { record: DeliveryRecord, after: AfterSave },
}

/// What a record's store terminal continues to.
#[derive(Debug)]
pub(super) enum AfterSave {
    /// The intent is durable: the call's git effects run.
    Execute(InPlace),
    /// The answer is durable: it returns to this call.
    Return { owner: Token },
    /// The reconciled answer is durable: loading goes on.
    ResumeLoading,
}

impl Delivering {
    /// Whether a record awaits its store terminal.
    pub(super) fn saving(&self) -> bool {
        match self {
            Self::Saving { .. } => true,
            Self::Idle | Self::InPlace(_) | Self::Reconciling(_) => false,
        }
    }
}

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
            state: State::Advance,
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
    let state = core::mem::replace(&mut domain.delivering, Delivering::Idle);
    domain.delivering = match state {
        Delivering::Idle => begin(domain, env, name, owner, change, deadline, out),
        Delivering::InPlace(_) | Delivering::Reconciling(_) | Delivering::Saving { .. } => {
            unreachable!("one delivery in flight")
        }
    };
    follow(domain, env, out);
}

fn begin(
    domain: &mut Domain,
    env: &Env<Limits>,
    name: run::CallName,
    owner: Token,
    change: run::outcome::Change,
    deadline: Time,
    out: &mut Queue<Request>,
) -> Delivering {
    if let Some(delivery) = crate::delivery::cached(&domain.records, name) {
        deliver_to_agent(domain, env, owner, delivery);
        return Delivering::Idle;
    }
    if domain.records.get(&name).is_some() {
        return deliver_unknown(domain, env, owner, 0);
    }
    let rules = match &domain.config.contract {
        crate::Contract::Change(spec) => &spec.fields,
        crate::Contract::Report(_) => match &domain.config.deliver {
            Some(spec) => &spec.fields,
            None => unreachable!("a Report delivery has a separate grant"),
        },
    };
    let Some(message) = commit_message(&change, &domain.config.title_field, rules) else {
        return save_answer(
            name,
            run::Delivery::Failed(run::DeliveryFailure::new(0, run::DeliveryReason::TooLarge)),
            Box::new([]),
            AfterSave::Return { owner },
            out,
        );
    };
    let Some(message) = crate::delivery::named_message(message, name) else {
        return save_answer(
            name,
            run::Delivery::Failed(run::DeliveryFailure::new(0, run::DeliveryReason::TooLarge)),
            Box::new([]),
            AfterSave::Return { owner },
            out,
        );
    };
    let in_place = InPlace::new(name, owner, deadline, message);
    Delivering::InPlace(in_place)
}

fn advance_delivery(
    domain: &mut Domain,
    env: &Env<Limits>,
    mut in_place: InPlace,
    out: &mut Queue<Request>,
) -> Delivering {
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
                    return save_timeout(in_place, position, out);
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
                out.push(request);
                return Delivering::InPlace(in_place);
            }
            save_intent(domain, in_place, out)
        }
        Stage::Execute => {
            for index in 0..in_place.directories.len() {
                let entry = in_place.directories.get(index).expect("surveyed directory").clone();
                if entry.directory < in_place.next || !entry.changed {
                    continue;
                }
                if env.now >= in_place.deadline {
                    return save_timeout(in_place, entry.directory, out);
                }
                in_place.next = entry.directory;
                if entry.head.is_some() {
                    return commit_request(in_place, out);
                }
                in_place.receipt(entry.directory, Box::from(&b"files kept"[..]));
                in_place.next = entry.directory.checked_add(1).expect("admitted directory position");
            }
            let name = in_place.name;
            let owner = in_place.owner;
            let answer = in_place.result();
            save_answer(name, answer, in_place.receipts.into_boxed(), AfterSave::Return { owner }, out)
        }
    }
}

fn save_intent(domain: &Domain, mut in_place: InPlace, out: &mut Queue<Request>) -> Delivering {
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
    save(record, AfterSave::Execute(in_place), out)
}

pub(super) fn plain_status(
    domain: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    changed: bool,
    out: &mut Queue<Request>,
) {
    let state = core::mem::replace(&mut domain.delivering, Delivering::Idle);
    domain.delivering = match state {
        Delivering::InPlace(call) => Delivering::InPlace(plain_terminal(call, owner, changed)),
        Delivering::Idle | Delivering::Reconciling(_) | Delivering::Saving { .. } => {
            unreachable!("plain status answers one delivery operation")
        }
    };
    follow(domain, env, out);
}

fn plain_terminal(mut call: InPlace, owner: Token, changed: bool) -> InPlace {
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
        | State::Advance
        | State::Answer(_)
        | State::Unknown
        | State::Done => unreachable!("plain terminal answers one plain status request"),
    };
    call
}

pub(super) fn git_result(
    domain: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    result: GitResult,
    out: &mut Queue<Request>,
) {
    let state = core::mem::replace(&mut domain.delivering, Delivering::Idle);
    domain.delivering = match state {
        Delivering::InPlace(call) => Delivering::InPlace(git_terminal(domain, env, call, owner, result, out)),
        Delivering::Reconciling(reconcile) => {
            Delivering::Reconciling(reconcile::reconcile_git(domain, env, reconcile, owner, result, out))
        }
        Delivering::Idle | Delivering::Saving { .. } => unreachable!("git terminal answers one delivery operation"),
    };
    follow(domain, env, out);
}

fn git_terminal(
    domain: &mut Domain,
    env: &Env<Limits>,
    mut call: InPlace,
    owner: Token,
    result: GitResult,
    out: &mut Queue<Request>,
) -> InPlace {
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
        State::Plain | State::Advance | State::Answer(_) | State::Unknown | State::Done => {
            unreachable!("git terminal answers an issued git request")
        }
    };
    call
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
            domain.chat.live_mut().own_heads.insert(call.next, head).expect("one head per writable repository");
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

/// What the state implies, in one place: a transient inner state moves on.
fn follow(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let state = core::mem::replace(&mut domain.delivering, Delivering::Idle);
    domain.delivering = match state {
        Delivering::InPlace(call) => follow_in_place(domain, env, call, out),
        Delivering::Reconciling(reconcile) => reconcile::follow(domain, env, reconcile, out),
        state @ (Delivering::Idle | Delivering::Saving { .. }) => state,
    };
}

fn follow_in_place(domain: &mut Domain, env: &Env<Limits>, mut call: InPlace, out: &mut Queue<Request>) -> Delivering {
    let state = core::mem::replace(&mut call.state, State::Done);
    match state {
        State::Advance => advance_delivery(domain, env, call, out),
        State::Answer(answer) => {
            save_answer(call.name, *answer, call.receipts.into_boxed(), AfterSave::Return { owner: call.owner }, out)
        }
        State::Unknown => deliver_unknown(domain, env, call.owner, call.next),
        state @ (State::Plain
        | State::Status
        | State::Markers { .. }
        | State::Commit
        | State::Inspect { .. }
        | State::InspectPush
        | State::Push) => {
            call.state = state;
            Delivering::InPlace(call)
        }
        State::Done => unreachable!("each delivery transition yields its next state"),
    }
}

fn commit_request(mut in_place: InPlace, out: &mut Queue<Request>) -> Delivering {
    in_place.state = State::Commit;
    out.push(Request::Git {
        owner: in_place.owner,
        directory: in_place.next,
        op: GitOp::Commit { message: in_place.message.clone() },
        deadline: in_place.deadline,
    });
    Delivering::InPlace(in_place)
}

fn stale_head(domain: &Domain, directory: u32, head: &[u8]) -> bool {
    let start = domain.chat.live().heads.get(&directory).expect("writable repository head known before run");
    if start.as_ref() == head {
        return false;
    }
    match domain.chat.live().own_heads.get(&directory) {
        Some(own) => own.as_ref() != head,
        None => true,
    }
}

fn deliver_unknown(domain: &mut Domain, env: &Env<Limits>, owner: Token, directory: u32) -> Delivering {
    deliver_to_agent(
        domain,
        env,
        owner,
        run::Delivery::Failed(run::DeliveryFailure::new(directory, run::DeliveryReason::Unknown)),
    );
    Delivering::Idle
}

fn save_timeout(call: InPlace, directory: u32, out: &mut Queue<Request>) -> Delivering {
    save_answer(
        call.name,
        run::Delivery::Failed(run::DeliveryFailure::new(directory, run::DeliveryReason::TimedOut)),
        call.receipts.into_boxed(),
        AfterSave::Return { owner: call.owner },
        out,
    )
}

pub(super) fn save_answer(
    name: run::CallName,
    delivery: run::Delivery,
    landed: Box<[run::Receipt]>,
    after: AfterSave,
    out: &mut Queue<Request>,
) -> Delivering {
    let record = DeliveryRecord { name, state: DeliveryState::Answer(delivery), landed };
    save(record, after, out)
}

/// The one place a record goes to the store.
pub(super) fn save(record: DeliveryRecord, after: AfterSave, out: &mut Queue<Request>) -> Delivering {
    out.push(Request::SaveDelivery { record: Box::new(record.clone()) });
    Delivering::Saving { record, after }
}

pub(super) fn delivery_saved(domain: &mut Domain, env: &Env<Limits>, name: run::CallName, out: &mut Queue<Request>) {
    let state = core::mem::replace(&mut domain.delivering, Delivering::Idle);
    match state {
        Delivering::Saving { record, after } => {
            assert_eq!(record.name, name, "delivery store terminal names the decision");
            domain.records.insert(record.name, record.clone()).expect("one turn's calls fit the bound");
            match after {
                AfterSave::Execute(mut call) => {
                    call.state = State::Advance;
                    call.stage = Stage::Execute;
                    call.next = 0;
                    domain.delivering = Delivering::InPlace(call);
                }
                AfterSave::Return { owner } => {
                    let delivery = saved_answer(record);
                    observe(domain, Fact::DeliveryReturned { name });
                    deliver_to_agent(domain, env, owner, delivery);
                }
                AfterSave::ResumeLoading => resume_loading(domain, env, out),
            }
        }
        Delivering::Idle => assert!(domain.ending.store_failed(), "DeliverySaved answers one SaveDelivery"),
        Delivering::InPlace(_) | Delivering::Reconciling(_) => unreachable!("DeliverySaved answers one SaveDelivery"),
    }
    follow(domain, env, out);
}

pub(super) fn reconcile_intent(
    domain: &mut Domain,
    env: &Env<Limits>,
    record: DeliveryRecord,
    out: &mut Queue<Request>,
) {
    let state = core::mem::replace(&mut domain.delivering, Delivering::Idle);
    domain.delivering = match state {
        Delivering::Idle => Delivering::Reconciling(Reconcile::new(record)),
        Delivering::InPlace(_) | Delivering::Reconciling(_) | Delivering::Saving { .. } => {
            unreachable!("one delivery in flight")
        }
    };
    follow(domain, env, out);
}

pub(super) fn save_failed(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let state = core::mem::replace(&mut domain.delivering, Delivering::Idle);
    domain.delivering = match state {
        Delivering::Saving { record, after } => failed_save(domain, env, record, after, out),
        state @ (Delivering::Idle | Delivering::InPlace(_) | Delivering::Reconciling(_)) => state,
    };
    follow(domain, env, out);
}

fn failed_save(
    domain: &mut Domain,
    env: &Env<Limits>,
    record: DeliveryRecord,
    after: AfterSave,
    out: &mut Queue<Request>,
) -> Delivering {
    match &record.state {
        DeliveryState::Answer(run::Delivery::Delivered(_)) => {
            out.push(Request::Notice { notice: Notice::Failure(Failure::DeliveryUnsaved) });
        }
        DeliveryState::Intent(_)
        | DeliveryState::Answer(
            run::Delivery::Nothing | run::Delivery::Refused(_) | run::Delivery::Failed(_) | run::Delivery::Stale,
        ) => {}
    }
    match after {
        AfterSave::Execute(call) => {
            deliver_to_agent(
                domain,
                env,
                call.owner,
                run::Delivery::Failed(run::DeliveryFailure::new(0, run::DeliveryReason::Broken)),
            );
        }
        AfterSave::Return { owner } => {
            deliver_to_agent(domain, env, owner, saved_answer(record));
        }
        AfterSave::ResumeLoading => {}
    }
    Delivering::Idle
}

fn saved_answer(record: DeliveryRecord) -> run::Delivery {
    match record.state {
        DeliveryState::Answer(delivery) => delivery,
        DeliveryState::Intent(_) => unreachable!("saved answer continues a delivery return"),
    }
}

pub(super) fn deliver_to_agent(domain: &mut Domain, env: &Env<Limits>, owner: Token, delivery: run::Delivery) {
    let reply = host::Reply::Delivery(delivery);
    command(
        &mut domain.agent,
        env.now,
        env.wall,
        parent::Event::Answer {
            agent: domain.child.run().expect("delivery belongs to the started child"),
            call: owner,
            reply,
        },
    );
}
