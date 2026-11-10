//! Local source checks and sign-outs, independent of run slots (domain/hosts/README.md, 5.1).
//! Commands are serialized, including across list replacement and reconnect.
use crate::{Limits, LinkUp, Login, LoginState, Request, SignOutResult, SourceFailure, SourceName};
use alloc::boxed::Box;
use jig_domain_link::{Grant, ListFailure, ListOutcome};
use skein_lib::{Env, List, Queue, Time, Token};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Operation {
    Check,
    SignOut,
}
#[derive(PartialEq, Eq, Debug)]
pub enum Answer {
    Checked { state: LoginState, email: Option<Box<[u8]>> },
    SignedOut(SignOutResult),
    Read(ListOutcome),
}
#[derive(Debug)]
struct Row {
    name: SourceName,
    state: Option<LoginState>,
    email: Option<Box<[u8]>>,
    check: bool,
    sign_out: bool,
    explicit: bool,
    next: Option<Time>,
}
#[derive(Debug)]
struct Command {
    owner: Token,
    name: SourceName,
    operation: Operation,
    current: bool,
    explicit: bool,
}
#[derive(Debug)]
struct Reading {
    read: u64,
    endpoint: u32,
    grant: Grant,
}
#[derive(Debug)]
enum Busy {
    Source(Command),
    Read { reading: Reading, current: bool },
}
#[derive(Debug)]
pub struct Domain {
    rows: List<Row>,
    busy: Option<Busy>,
    reading: Option<Reading>,
    sequence: u64,
    first: bool,
    connected: bool,
    closed: bool,
}
impl Domain {
    #[must_use]
    pub fn new(capacity: u32) -> Self {
        Self {
            rows: List::with_capacity(capacity),
            busy: None,
            reading: None,
            sequence: 1,
            first: false,
            connected: false,
            closed: false,
        }
    }
    #[must_use]
    pub fn next_deadline(&self) -> Option<Time> {
        if self.closed || !self.connected || self.busy.is_some() {
            return None;
        }
        let mut next: Option<Time> = None;
        for row in &self.rows {
            if let Some(at) = row.next {
                next = Some(match next {
                    Some(old) => old.min(at),
                    None => at,
                });
            }
        }
        next
    }
    pub(crate) fn holds_grant(&self, account: u32, generation: u64) -> bool {
        if let Some(reading) = &self.reading
            && reading.grant.account == account
            && reading.grant.generation == generation
        {
            return true;
        }
        match &self.busy {
            Some(Busy::Read { reading, .. }) => {
                reading.grant.account == account && reading.grant.generation == generation
            }
            Some(Busy::Source(_)) | None => false,
        }
    }
    #[must_use]
    pub fn settled(&self) -> bool {
        self.busy.is_none()
    }
}
/// Whether these limits are representable: every edge bound they size
/// is finite and every capacity they derive fits (programming-model.md, 6.3).
#[must_use]
pub fn valid(limits: &Limits) -> bool {
    limits.source_interval != skein_lib::Duration::ZERO && limits.source_timeout != skein_lib::Duration::ZERO
}

pub(crate) fn replace(domain: &mut Domain, env: &Env<Limits>, names: Box<[SourceName]>, out: &mut Queue<Request>) {
    if names.len() > skein_lib::length::usize_of(env.limits.program_sources) || !unique(&names) {
        out.push(Request::SourceFault);
        return;
    }
    if domain.closed {
        return;
    }
    let mut old = core::mem::replace(&mut domain.rows, List::with_capacity(env.limits.program_sources));
    for name in names {
        let mut row = Row { name, state: None, email: None, check: true, sign_out: false, explicit: false, next: None };
        for index in 0..old.len() {
            let previous = old.get_mut(index).expect("previous source row");
            if previous.name == name {
                row.state = previous.state.take();
                row.email = previous.email.take();
                row.sign_out = previous.sign_out;
                row.explicit = previous.explicit;
                previous.sign_out = false;
            }
        }
        domain.rows.push(row).expect("validated source list");
    }
    for row in &old {
        if row.sign_out && domain.connected {
            out.push(Request::Up(LinkUp::SignedOut {
                name: row.name,
                result: SignOutResult::Failed { reason: SourceFailure::Unknown, detail: Box::new([]) },
            }));
        }
    }
    domain.connected = true;
    domain.first = true;
    advance(domain, env, out);
}
#[must_use]
pub fn unique(names: &[SourceName]) -> bool {
    for (index, name) in names.iter().enumerate() {
        for other in names.iter().skip(index.saturating_add(1)) {
            if name == other {
                return false;
            }
        }
    }
    true
}
pub(crate) fn check(domain: &mut Domain, env: &Env<Limits>, name: SourceName, out: &mut Queue<Request>) {
    if domain.closed {
        return;
    }
    let mut found = false;
    for index in 0..domain.rows.len() {
        let row = domain.rows.get_mut(index).expect("source row");
        if row.name == name {
            row.check = true;
            row.explicit = true;
            found = true;
        }
    }
    if !found {
        out.push(Request::SourceFault);
        return;
    }
    advance(domain, env, out);
}
pub(crate) fn sign_out(domain: &mut Domain, env: &Env<Limits>, name: SourceName, out: &mut Queue<Request>) {
    if domain.closed {
        out.push(Request::Up(LinkUp::SignedOut {
            name,
            result: SignOutResult::Failed { reason: SourceFailure::Closed, detail: Box::new([]) },
        }));
        return;
    }
    match &domain.busy {
        Some(Busy::Source(busy)) if busy.current && busy.name == name && busy.operation == Operation::SignOut => {
            out.push(Request::SourceFault);
            return;
        }
        Some(Busy::Source(_) | Busy::Read { .. }) | None => {}
    }
    let mut found = false;
    for index in 0..domain.rows.len() {
        let row = domain.rows.get_mut(index).expect("source row");
        if row.name == name {
            if row.sign_out {
                out.push(Request::SourceFault);
                return;
            }
            row.sign_out = true;
            found = true;
        }
    }
    if !found {
        let reason = SourceFailure::Unknown;
        out.push(Request::Up(LinkUp::SignedOut {
            name,
            result: SignOutResult::Failed { reason, detail: Box::new([]) },
        }));
        return;
    }
    // The link contract allows one outstanding sign-out per source/channel.
    advance(domain, env, out);
}
pub(crate) fn answered(domain: &mut Domain, env: &Env<Limits>, owner: Token, answer: Answer, out: &mut Queue<Request>) {
    match &domain.busy {
        Some(Busy::Source(busy)) if busy.owner == owner => {}
        Some(Busy::Source(_) | Busy::Read { .. }) | None => return,
    }
    let busy = match domain.busy.take().expect("matched command") {
        Busy::Source(busy) => busy,
        Busy::Read { .. } => unreachable!("matched source command"),
    };
    if busy.current {
        match answer {
            Answer::Read(_) => unreachable!("source terminal is not a list"),
            Answer::Checked { state, email } => {
                let state = match state {
                    LoginState::Failed { reason, detail } => {
                        LoginState::Failed { reason, detail: tail(detail, env.limits.detail_bytes) }
                    }
                    state @ (LoginState::Usable | LoginState::NotSignedIn | LoginState::SignedInOtherwise(_)) => state,
                };
                assert!(busy.operation == Operation::Check, "typed check terminal");
                let mut changed = false;
                for index in 0..domain.rows.len() {
                    let row = domain.rows.get_mut(index).expect("source row");
                    if row.name == busy.name {
                        changed = row.state.as_ref() != Some(&state) || row.email != email;
                        row.email = email;
                        row.state = Some(state);
                        row.next = Some(env.now.saturating_add(env.limits.source_interval));
                        break;
                    }
                }
                if !domain.first && domain.connected && !domain.closed && (changed || busy.explicit) {
                    publish(domain, out);
                }
            }
            Answer::SignedOut(result) => {
                let result = match result {
                    SignOutResult::SignedOut => SignOutResult::SignedOut,
                    SignOutResult::Failed { reason, detail } => {
                        SignOutResult::Failed { reason, detail: tail(detail, env.limits.detail_bytes) }
                    }
                };
                assert!(busy.operation == Operation::SignOut, "typed sign-out terminal");
                if domain.connected {
                    out.push(Request::Up(LinkUp::SignedOut { name: busy.name, result }));
                }
                for index in 0..domain.rows.len() {
                    let row = domain.rows.get_mut(index).expect("source row");
                    if row.name == busy.name {
                        row.check = !domain.closed;
                    }
                }
            }
        }
    }
    advance(domain, env, out);
}
pub(crate) fn fire(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    if !domain.connected || domain.closed {
        return;
    }
    for index in 0..domain.rows.len() {
        let row = domain.rows.get_mut(index).expect("source row");
        if let Some(at) = row.next
            && at <= env.now
        {
            row.check = true;
            row.next = None;
        }
    }
    advance(domain, env, out);
}
pub(crate) fn lost(domain: &mut Domain, out: &mut Queue<Request>) {
    domain.connected = false;
    domain.first = false;
    domain.rows.clear();
    domain.reading = None;
    match &mut domain.busy {
        Some(Busy::Source(busy)) => busy.current = false,
        Some(Busy::Read { reading, current }) => {
            if *current {
                out.push(Request::CancelRead { read: reading.read });
            }
            *current = false;
        }
        None => {}
    }
}
pub(crate) fn close(domain: &mut Domain, out: &mut Queue<Request>) {
    domain.closed = true;
    if let Some(reading) = domain.reading.take() {
        failed_read(reading.read, out);
    }
    match &mut domain.busy {
        Some(Busy::Read { reading, current }) => {
            if *current {
                failed_read(reading.read, out);
                out.push(Request::CancelRead { read: reading.read });
            }
            *current = false;
        }
        Some(Busy::Source(_)) | None => {}
    }
    for index in 0..domain.rows.len() {
        let row = domain.rows.get_mut(index).expect("source row");
        row.check = false;
        row.next = None;
        if row.sign_out {
            row.sign_out = false;
            if domain.connected {
                out.push(Request::Up(LinkUp::SignedOut {
                    name: row.name,
                    result: SignOutResult::Failed { reason: SourceFailure::Closed, detail: Box::new([]) },
                }));
            }
        }
    }
}
fn publish(domain: &Domain, out: &mut Queue<Request>) {
    let mut rows = List::with_capacity(domain.rows.len());
    for row in &domain.rows {
        rows.push(Login {
            name: row.name,
            state: row.state.clone().expect("whole inventory checked"),
            email: row.email.clone(),
        })
        .expect("source capacity");
    }
    out.push(Request::Up(LinkUp::Logins { logins: rows.into_boxed() }));
}
fn advance(domain: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    if domain.busy.is_some() || domain.closed {
        return;
    }
    if domain.first {
        let mut complete = true;
        for row in &domain.rows {
            if row.state.is_none() || row.check {
                complete = false;
            }
        }
        if complete {
            if domain.connected {
                publish(domain, out);
            }
            domain.first = false;
        }
    }
    if !domain.connected {
        return;
    }
    let mut selected = None;
    // Sign-out wins over a coalesced check; list order breaks ties.
    for index in 0..domain.rows.len() {
        let row = domain.rows.get_mut(index).expect("source row");
        if row.sign_out {
            row.sign_out = false;
            selected = Some((row.name, Operation::SignOut, false));
            break;
        }
    }
    if selected.is_none()
        && let Some(reading) = domain.reading.take()
    {
        out.push(Request::ReadList { read: reading.read, endpoint: reading.endpoint, grant: reading.grant });
        domain.busy = Some(Busy::Read { reading, current: true });
        return;
    }
    if selected.is_none() {
        for index in 0..domain.rows.len() {
            let row = domain.rows.get_mut(index).expect("source row");
            if row.check {
                row.check = false;
                row.next = None;
                selected = Some((row.name, Operation::Check, row.explicit));
                row.explicit = false;
                break;
            }
        }
    }
    if let Some((name, operation, explicit)) = selected {
        let owner = Token::new(domain.sequence);
        domain.sequence = domain.sequence.checked_add(1).expect("source command names never reused");
        domain.busy = Some(Busy::Source(Command { owner, name, operation, current: true, explicit }));
        out.push(Request::Source {
            owner,
            name,
            operation,
            deadline: env.now.saturating_add(env.limits.source_timeout),
        });
    }
}

fn tail(detail: Box<[u8]>, bound: u32) -> Box<[u8]> {
    let bound = skein_lib::length::usize_of(bound);
    if detail.len() <= bound {
        return detail;
    }
    Box::from(detail.get(detail.len().saturating_sub(bound)..).expect("bounded tail"))
}

pub(crate) fn read(
    domain: &mut Domain,
    env: &Env<Limits>,
    read: u64,
    endpoint: u32,
    grant: Grant,
    out: &mut Queue<Request>,
) {
    if domain.closed || !domain.connected || domain.reading.is_some() {
        failed_read(read, out);
        return;
    }
    domain.reading = Some(Reading { read, endpoint, grant });
    advance(domain, env, out);
}
pub(crate) fn list_read(
    domain: &mut Domain,
    env: &Env<Limits>,
    read: u64,
    outcome: ListOutcome,
    out: &mut Queue<Request>,
) {
    let current = match &domain.busy {
        Some(Busy::Read { reading, current }) if reading.read == read => *current,
        Some(Busy::Read { .. } | Busy::Source(_)) | None => return,
    };
    domain.busy = None;
    if current && domain.connected {
        out.push(Request::Up(LinkUp::ListRead { read, outcome }));
    }
    advance(domain, env, out);
}
fn failed_read(read: u64, out: &mut Queue<Request>) {
    out.push(Request::Up(LinkUp::ListRead { read, outcome: ListOutcome::Failed(ListFailure::Agent) }));
}
