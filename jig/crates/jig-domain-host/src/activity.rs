//! Content-free best-effort activity (domain/hosts.md, section 6.5).
use crate::{AgentOperation, CountKind};
use crate::{Hub, LinkUp, Phase, Request};
use alloc::boxed::Box;
use skein_lib::{Duration, Queue, Token};

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum AgentActivity {
    Phase(Phase),
    Long { operation: AgentOperation, span: Duration },
    LongDone,
    Count { kind: CountKind, elapsed: Duration, count: u64 },
}

pub(crate) fn reported(domain: &mut Hub, owner: Token, activity: AgentActivity) {
    if let Some(hosting) = live(domain, owner) {
        domain.facts.activity(LinkUp::Activity { run: hosting.run, attempt: hosting.attempt, activity });
    }
}

pub(crate) fn live(domain: &Hub, owner: Token) -> Option<crate::Hosting> {
    let hosting = domain.hosting(owner)?;
    match hosting.phase {
        Phase::Active | Phase::Waiting => Some(hosting),
        Phase::Preparing | Phase::Starting | Phase::Ending | Phase::Answered => None,
    }
}

pub(crate) fn unauthorized(domain: &Hub, owner: Token, account: u32, generation: u64, out: &mut Queue<Request>) {
    if let Some(hosting) = live(domain, owner) {
        out.push(Request::Up(LinkUp::Unauthorized { run: hosting.run, attempt: hosting.attempt, account, generation }));
    }
}

pub(crate) fn limited(domain: &Hub, owner: Token, account: u32, reset: Option<Duration>, out: &mut Queue<Request>) {
    if let Some(hosting) = live(domain, owner) {
        out.push(Request::Up(LinkUp::Limited { run: hosting.run, attempt: hosting.attempt, account, reset }));
    }
}

pub(crate) fn account_refused(
    domain: &Hub,
    owner: Token,
    account: u32,
    code: Box<[u8]>,
    reason: Box<[u8]>,
    out: &mut Queue<Request>,
) {
    if let Some(hosting) = live(domain, owner) {
        out.push(Request::Up(LinkUp::AccountRefused {
            run: hosting.run,
            attempt: hosting.attempt,
            account,
            code,
            reason,
        }));
    }
}
