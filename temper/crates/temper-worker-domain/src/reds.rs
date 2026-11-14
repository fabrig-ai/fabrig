//! Fresh reds and serialized reports for each check (domain/forge/reds.md, 5.3–5.6).
use crate::{Domain, Limits, Request, route, wire, workspace};
use alloc::boxed::Box;
use jig_domain_host as host;
use skein_lib::{Duration, Env, List, Map, Queue, Time, Token};
use temper_worker_domain_checkout::git;

#[derive(Debug)]
pub(crate) struct State {
    active: Option<Check>,
    reports: Option<Reports>,
    cookie: u64,
    numbering: Option<Ask>,
    outstanding: Map<u64, Ask>,
}
#[derive(Debug)]
struct Check {
    generation: u64,
    deadline: Option<Time>,
    cookie: u64,
}
#[derive(Debug)]
struct Reports {
    generation: u64,
    reading: bool,
    queued: Map<u64, wire::CheckResults>,
    inflight: Option<u64>,
    encoding: bool,
}
#[derive(Clone, Copy, Debug)]
struct Ask {
    cookie: u64,
    kind: Kind,
}
#[derive(Clone, Copy, Debug)]
enum Kind {
    Reds,
    Results,
}
impl State {
    pub fn new() -> Self {
        Self { active: None, reports: None, cookie: 0, numbering: None, outstanding: Map::with_capacity(2) }
    }
    fn next_cookie(&mut self) -> u64 {
        self.cookie = self.cookie.checked_add(1).expect("ask lifetime");
        self.cookie
    }
}
pub(crate) fn start(domain: &mut Domain, env: &Env<Limits>, owner: Token, out: &mut Queue<Request>) {
    let Some(bound) = domain.agents.get_mut(&owner) else { return };
    bound.check_number = bound.check_number.checked_add(1).expect("check generation lifetime");
    let cookie = bound.checks.next_cookie();
    bound.checks.active = Some(Check {
        generation: bound.check_number,
        deadline: Some(env.now.saturating_add(Duration::from_millis(u64::from(env.limits.checkout.reds_read_ms)))),
        cookie,
    });
    bound.checks.reports = None;
    workspace::reds(domain, env, owner, git::Reds::Clear);
    out.push(Request::RedsAsk { owner, check: cookie, ask: wire::Ask::Reds });
    resume(domain, env, owner);
}
pub(crate) fn end(domain: &mut Domain, env: &Env<Limits>, owner: Token) {
    let Some(bound) = domain.agents.get_mut(&owner) else { return };
    if let Some(check) = bound.checks.active.take() {
        bound.checks.reports = Some(Reports {
            generation: check.generation,
            reading: true,
            queued: Map::with_capacity(env.limits.checkout.repositories),
            inflight: None,
            encoding: false,
        });
        if !workspace::reds(domain, env, owner, git::Reds::Read { check: check.generation }) {
            domain.agents.get_mut(&owner).expect("live agent").checks.reports = None;
        }
    }
}
fn current(domain: &mut Domain, owner: Token, cookie: u64, now: Time) -> Option<&mut Check> {
    let check = domain.agents.get_mut(&owner)?.checks.active.as_mut()?;
    if check.cookie == cookie && check.deadline? >= now { Some(check) } else { None }
}
pub(crate) fn encoded(
    domain: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    cookie: u64,
    bytes: Box<[u8]>,
    out: &mut Queue<Request>,
) {
    let kind = if current(domain, owner, cookie, env.now).is_some() {
        Kind::Reds
    } else {
        let Some(bound) = domain.agents.get_mut(&owner) else { return };
        let Some(reports) = &mut bound.checks.reports else { return };
        if reports.inflight != Some(cookie) || !reports.encoding {
            return;
        }
        reports.encoding = false;
        Kind::Results
    };
    let Some(hosting) = domain.host.hosting(owner) else { return };
    domain.agents.get_mut(&owner).expect("live agent").checks.numbering = Some(Ask { cookie, kind });
    route::host_step(domain, env, host::Event::Ask { run: hosting.run, attempt: hosting.attempt, bytes });
    // The hub's numbered ask must be handed off before a resumed delivery can answer.
    route::hand_off(domain, env, out);
    resume(domain, env, owner);
}
fn owner(domain: &Domain, run: Token, attempt: Token) -> Option<Token> {
    for (owner, _) in &domain.agents {
        if let Some(hosting) = domain.host.hosting(*owner)
            && hosting.run == run
            && hosting.attempt == attempt
        {
            return Some(*owner);
        }
    }
    None
}
pub(crate) fn numbered(domain: &mut Domain, run: Token, attempt: Token, number: u64) {
    let Some(owner) = owner(domain, run, attempt) else { return };
    let state = &mut domain.agents.get_mut(&owner).expect("live agent").checks;
    if let Some(ask) = state.numbering.take() {
        let old = state.outstanding.insert(number, ask).expect("hub bounds outstanding asks");
        assert!(old.is_none(), "hub numbers each ask once");
    }
}
pub(crate) fn asked(
    domain: &mut Domain,
    env: &Env<Limits>,
    run: Token,
    attempt: Token,
    number: u64,
    answer: host::AskAnswer,
    out: &mut Queue<Request>,
) {
    let Some(owner) = owner(domain, run, attempt) else { return };
    let state = &mut domain.agents.get_mut(&owner).expect("live agent").checks;
    let ask = if number == 0 {
        state.numbering.take()
    } else {
        match state.outstanding.remove(&number) {
            Some(ask) => Some(ask),
            None => state.numbering.take(),
        }
    };
    let Some(ask) = ask else { return };
    match ask.kind {
        Kind::Reds => {
            let Some(check) = current(domain, owner, ask.cookie, env.now) else { return };
            match answer {
                host::AskAnswer::Refused => check.deadline = None,
                host::AskAnswer::Bytes(bytes) => out.push(Request::RedsAnswer { owner, check: ask.cookie, bytes }),
            }
        }
        Kind::Results => {
            let Some(reports) = &mut domain.agents.get_mut(&owner).expect("live agent").checks.reports else { return };
            if reports.inflight != Some(ask.cookie) {
                return;
            }
            match answer {
                host::AskAnswer::Refused => {
                    reports.inflight = None;
                    send(domain, env, owner, out);
                }
                host::AskAnswer::Bytes(bytes) => out.push(Request::RedsAnswer { owner, check: ask.cookie, bytes }),
            }
        }
    }
}
pub(crate) fn decoded(
    domain: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    cookie: u64,
    answer: Option<wire::Answer>,
    out: &mut Queue<Request>,
) {
    if let Some(check) = current(domain, owner, cookie, env.now) {
        check.deadline = None;
        let Some(wire::Answer::Reds { reds }) = answer else { return };
        let mut kept = List::with_capacity(u32::try_from(reds.len()).expect("decoded red count"));
        for red in reds {
            kept.push(git::Red { key: red.key, head: red.head }).expect("decoded red count");
        }
        workspace::reds(domain, env, owner, git::Reds::Write { reds: kept.into_boxed() });
    } else {
        let Some(bound) = domain.agents.get_mut(&owner) else { return };
        let Some(reports) = &mut bound.checks.reports else { return };
        if reports.inflight == Some(cookie) {
            reports.inflight = None;
            send(domain, env, owner, out);
        }
    }
}
pub(crate) fn read(
    domain: &mut Domain,
    client: Token,
    generation: u64,
    repository: u32,
    bytes: Option<Box<[u8]>>,
    out: &mut Queue<Request>,
) {
    let Some((owner, tag)) = workspace::report_repository(domain, client, repository) else { return };
    let Some(bound) = domain.agents.get_mut(&owner) else { return };
    let Some(reports) = &mut bound.checks.reports else { return };
    if reports.generation != generation || !reports.reading {
        return;
    }
    let parsed = match bytes {
        Some(bytes) => crate::results::parse(tag, generation, &bytes),
        None => Err("results file absent or unreadable"),
    };
    match parsed {
        Ok(results) => {
            let old = reports.queued.insert(tag, results).expect("one report per repository");
            assert!(old.is_none(), "repository read once");
        }
        Err(reason) => out.push(Request::ResultsIgnored { owner, tag, reason: Box::from(reason.as_bytes()) }),
    }
}
pub(crate) fn read_end(
    domain: &mut Domain,
    env: &Env<Limits>,
    client: Token,
    generation: u64,
    out: &mut Queue<Request>,
) {
    let Some(owner) = workspace::report_owner(domain, client) else { return };
    let Some(bound) = domain.agents.get_mut(&owner) else { return };
    let Some(reports) = &mut bound.checks.reports else { return };
    if reports.generation != generation {
        return;
    }
    reports.reading = false;
    send(domain, env, owner, out);
}
fn send(domain: &mut Domain, env: &Env<Limits>, owner: Token, out: &mut Queue<Request>) {
    let Some(bound) = domain.agents.get_mut(&owner) else { return };
    let state = &mut bound.checks;
    if let Some(reports) = &mut state.reports
        && !reports.reading
        && reports.inflight.is_none()
        && let Some((tag, _)) = reports.queued.first()
    {
        let tag = *tag;
        let results = reports.queued.remove(&tag).expect("first report");
        state.cookie = state.cookie.checked_add(1).expect("ask lifetime");
        reports.inflight = Some(state.cookie);
        reports.encoding = true;
        out.push(Request::RedsAsk { owner, check: state.cookie, ask: wire::Ask::Results(results) });
    }
    resume(domain, env, owner);
}
pub(crate) fn busy(domain: &Domain, owner: Token) -> bool {
    if let Some(bound) = domain.agents.get(&owner)
        && let Some(reports) = &bound.checks.reports
    {
        return reports.reading || reports.encoding || !reports.queued.is_empty();
    }
    false
}
fn resume(domain: &mut Domain, env: &Env<Limits>, owner: Token) {
    if !busy(domain, owner) {
        workspace::reports_sent(domain, env, owner);
        if let Some(detail) = domain.gone.remove(&owner) {
            crate::agents::gone(domain, env, owner, detail);
        }
    }
}
pub(crate) fn next(domain: &Domain) -> Option<Time> {
    let mut next = None;
    for (_, bound) in &domain.agents {
        if let Some(check) = &bound.checks.active
            && let Some(at) = check.deadline
        {
            next = Some(match next {
                Some(old) => at.min(old),
                None => at,
            });
        }
    }
    next
}
pub(crate) fn due(domain: &Domain, now: Time) -> bool {
    match next(domain) {
        Some(at) => at <= now,
        None => false,
    }
}
pub(crate) fn fire(domain: &mut Domain, now: Time) {
    let mut expired = List::with_capacity(domain.agents.len());
    for (owner, bound) in &domain.agents {
        if let Some(check) = &bound.checks.active
            && let Some(at) = check.deadline
            && at <= now
        {
            expired.push(*owner).expect("agent count");
        }
    }
    for owner in &expired {
        domain.agents.get_mut(owner).expect("live agent").checks.active.as_mut().expect("current check").deadline =
            None;
    }
}
