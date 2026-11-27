//! One bounded engine question for each waiting checkout hold
//! (temper/docs/design/domain/worker/checkouts-push.md, 1).
use crate::{Domain, Limits, Request, checkout, route, wire, workspace};
use alloc::boxed::Box;
use jig_domain_host as host;
use skein_lib::{Duration, Env, Map, Queue, Time, Token};

#[derive(Debug)]
pub(crate) struct Admits {
    pending: Map<Token, Pending>,
    cookie: u64,
}

#[derive(Debug)]
struct Pending {
    hold: Token,
    run: Token,
    attempt: Token,
    cookie: u64,
    deadline: Time,
    phase: Phase,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
enum Phase {
    Encoding,
    Numbering,
    Asked { number: u64 },
    Decoding,
}

impl Admits {
    pub(crate) fn new(slots: u32) -> Self {
        Self { pending: Map::with_capacity(slots), cookie: 0 }
    }

    pub(crate) fn next(&self) -> Option<Time> {
        let mut next = None;
        for (_, pending) in &self.pending {
            next = Some(match next {
                None => pending.deadline,
                Some(before) => pending.deadline.min(before),
            });
        }
        next
    }
}

#[expect(clippy::too_many_arguments, reason = "the checkout names the hold and its exact proposed write")]
pub(crate) fn start(
    domain: &mut Domain,
    env: &Env<Limits>,
    client: Token,
    hold: Token,
    repository: u32,
    branch: Box<[u8]>,
    expected: Option<checkout::git::Commit>,
    commit: checkout::git::Commit,
    out: &mut Queue<Request>,
) {
    if let Some((owner, tag)) = workspace::report_repository(domain, client, repository)
        && let Some(hosting) = domain.host.hosting(owner)
    {
        let state = &mut domain.push_admits;
        state.cookie = state.cookie.checked_add(1).expect("push admission lifetime");
        let cookie = state.cookie;
        let deadline = env.now.saturating_add(Duration::from_millis(u64::from(env.limits.checkout.reds_read_ms)));
        let pending =
            Pending { hold, run: hosting.run, attempt: hosting.attempt, cookie, deadline, phase: Phase::Encoding };
        assert!(state.pending.insert(client, pending).expect("one hold per slot").is_none(), "one admission per hold");
        let expected = match expected {
            Some(tip) => Some(tip.raw()),
            None => None,
        };
        let ask = wire::Ask::Admit { tag, branch, expected, commit: commit.raw() };
        out.push(Request::AdmitAsk { client, cookie, ask });
    } else {
        reply(domain, env, hold, Err(b"the attempt is not hosted".as_slice().into()));
    }
}

pub(crate) fn withdraw(domain: &mut Domain, client: Token) {
    domain.push_admits.pending.remove(&client);
}

pub(crate) fn encoded(domain: &mut Domain, env: &Env<Limits>, client: Token, cookie: u64, bytes: Box<[u8]>) {
    let Some(pending) = domain.push_admits.pending.get_mut(&client) else { return };
    if pending.cookie != cookie || pending.phase != Phase::Encoding || pending.deadline < env.now {
        return;
    }
    pending.phase = Phase::Numbering;
    let (run, attempt) = (pending.run, pending.attempt);
    route::host_step(domain, env, host::Event::Ask { run, attempt, bytes });
}

fn waiting(state: &Admits, run: Token, attempt: Token, number: Option<u64>) -> Option<Token> {
    for (client, pending) in &state.pending {
        if pending.run == run && pending.attempt == attempt {
            if pending.phase == Phase::Numbering {
                return Some(*client);
            }
            if let Some(number) = number
                && pending.phase == (Phase::Asked { number })
            {
                return Some(*client);
            }
        }
    }
    None
}

pub(crate) fn numbered(domain: &mut Domain, run: Token, attempt: Token, number: u64) -> bool {
    let Some(client) = waiting(&domain.push_admits, run, attempt, None) else { return false };
    domain.push_admits.pending.get_mut(&client).expect("just found").phase = Phase::Asked { number };
    true
}

/// The hub numbers forwarded asks. A refusal during numbering is synchronous
/// and may have no number. Once numbered, only that number can settle it.
pub(crate) fn asked(
    domain: &mut Domain,
    env: &Env<Limits>,
    run: Token,
    attempt: Token,
    number: u64,
    answer: host::AskAnswer,
    out: &mut Queue<Request>,
) -> Result<(), host::AskAnswer> {
    let Some(client) = waiting(&domain.push_admits, run, attempt, Some(number)) else { return Err(answer) };
    let pending = domain.push_admits.pending.get_mut(&client).expect("just found");
    if pending.deadline >= env.now {
        match answer {
            host::AskAnswer::Refused => deny(domain, env, client, b"the engine refused the ask"),
            host::AskAnswer::Bytes(bytes) => {
                pending.phase = Phase::Decoding;
                out.push(Request::AdmitAnswer { client, cookie: pending.cookie, bytes });
            }
        }
    }
    Ok(())
}

pub(crate) fn decoded(
    domain: &mut Domain,
    env: &Env<Limits>,
    client: Token,
    cookie: u64,
    answer: Option<wire::Answer>,
) {
    let Some(pending) = domain.push_admits.pending.get(&client) else { return };
    if pending.cookie != cookie || pending.phase != Phase::Decoding || pending.deadline < env.now {
        return;
    }
    let pending = domain.push_admits.pending.remove(&client).expect("just found");
    let answer = match answer {
        Some(wire::Answer::Admitted) => Ok(()),
        Some(wire::Answer::Unadmitted { words }) => Err(words),
        Some(wire::Answer::Kept | wire::Answer::Landed | wire::Answer::Unlanded | wire::Answer::Reds { .. }) => {
            Err(b"the engine answered a different ask".as_slice().into())
        }
        None => Err(b"the engine's answer did not decode".as_slice().into()),
    };
    reply(domain, env, pending.hold, answer);
}

pub(crate) fn is_due(domain: &Domain, now: Time) -> bool {
    due(&domain.push_admits, now).is_some()
}

pub(crate) fn fire(domain: &mut Domain, env: &Env<Limits>) {
    while let Some(client) = due(&domain.push_admits, env.now) {
        deny(domain, env, client, b"the engine did not answer in time");
    }
}

fn due(state: &Admits, now: Time) -> Option<Token> {
    for (client, pending) in &state.pending {
        if pending.deadline <= now {
            return Some(*client);
        }
    }
    None
}

fn deny(domain: &mut Domain, env: &Env<Limits>, client: Token, why: &[u8]) {
    let pending = domain.push_admits.pending.remove(&client).expect("waiting admission");
    reply(domain, env, pending.hold, Err(why.into()));
}

fn reply(domain: &mut Domain, env: &Env<Limits>, hold: Token, answer: Result<(), Box<[u8]>>) {
    route::checkout_step(domain, env, checkout::Event::PushAdmission { hold, answer });
}
