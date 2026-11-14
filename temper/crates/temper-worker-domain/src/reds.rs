//! A fresh, bounded read for each check (domain/forge/reds.md, 5.3).
use crate::{Domain, Limits, Request, route, wire, workspace};
use alloc::boxed::Box;
use jig_domain_host as host;
use skein_lib::{Duration, Env, List, Queue, Time, Token};
use temper_worker_domain_checkout::git;
#[derive(Debug)]
pub(crate) struct Check {
    generation: u64,
    deadline: Option<Time>,
    number: Option<u64>,
}
pub(crate) fn start(domain: &mut Domain, env: &Env<Limits>, owner: Token, out: &mut Queue<Request>) {
    let Some(bound) = domain.agents.get_mut(&owner) else { return };
    bound.check_number = bound.check_number.checked_add(1).expect("check generation lifetime");
    let generation = bound.check_number;
    bound.check = Some(Check {
        generation,
        deadline: Some(env.now.saturating_add(Duration::from_millis(u64::from(env.limits.checkout.reds_read_ms)))),
        number: None,
    });
    workspace::reds(domain, env, owner, git::Reds::Clear);
    out.push(Request::RedsAsk { owner, check: generation, ask: wire::Ask::Reds });
}
pub(crate) fn end(domain: &mut Domain, env: &Env<Limits>, owner: Token) {
    let Some(bound) = domain.agents.get_mut(&owner) else { return };
    if bound.check.take().is_some() {
        workspace::reds(domain, env, owner, git::Reds::Clear);
    }
}
fn current(domain: &mut Domain, owner: Token, generation: u64, now: Time) -> Option<&mut Check> {
    let check = domain.agents.get_mut(&owner)?.check.as_mut()?;
    if check.generation == generation && check.deadline? >= now { Some(check) } else { None }
}
pub(crate) fn encoded(domain: &mut Domain, env: &Env<Limits>, owner: Token, generation: u64, bytes: Box<[u8]>) {
    if current(domain, owner, generation, env.now).is_none() {
        return;
    }
    let Some(hosting) = domain.host.hosting(owner) else { return };
    route::host_step(domain, env, host::Event::Ask { run: hosting.run, attempt: hosting.attempt, bytes });
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
    if let Some(check) = &mut domain.agents.get_mut(&owner).expect("live agent").check
        && check.deadline.is_some()
        && check.number.is_none()
    {
        check.number = Some(number);
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
    let Some(check) = &mut domain.agents.get_mut(&owner).expect("live agent").check else { return };
    if check.number != Some(number) && number != 0 {
        return;
    }
    let Some(deadline) = check.deadline else { return };
    if env.now > deadline {
        check.deadline = None;
        return;
    }
    match answer {
        host::AskAnswer::Refused => check.deadline = None,
        host::AskAnswer::Bytes(bytes) => out.push(Request::RedsAnswer { owner, check: check.generation, bytes }),
    }
}
pub(crate) fn decoded(
    domain: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    generation: u64,
    answer: Option<wire::Answer>,
) {
    let Some(check) = current(domain, owner, generation, env.now) else { return };
    check.deadline = None;
    let Some(wire::Answer::Reds { reds }) = answer else { return };
    let mut kept = List::with_capacity(u32::try_from(reds.len()).expect("decoded red count"));
    for red in reds {
        kept.push(git::Red { key: red.key, head: red.head }).expect("decoded red count");
    }
    workspace::reds(domain, env, owner, git::Reds::Write { reds: kept.into_boxed() });
}
pub(crate) fn next(domain: &Domain) -> Option<Time> {
    let mut next = None;
    for (_, bound) in &domain.agents {
        if let Some(check) = &bound.check
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
        if let Some(check) = &bound.check
            && let Some(at) = check.deadline
            && at <= now
        {
            expired.push(*owner).expect("agent count");
        }
    }
    for owner in &expired {
        domain.agents.get_mut(owner).expect("live agent").check.as_mut().expect("current check").deadline = None;
    }
}
