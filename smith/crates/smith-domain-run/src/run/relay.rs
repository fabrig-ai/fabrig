//! One host relay from admission through retries to its terminal.
//! State retains the immutable call name, input, deadline and uncertainty; it knows
//! no host policy. Contract: smith/docs/design/domain/run.md, section 5.2.

use alloc::boxed::Box;
use core::mem;

use skein_lib::{Deadlines, Duration, Env, Id, Queue, Slab, Time, Token};

use super::{Alarm, Asking, Conversation, Run, begin_call, may_finish, retire_call};
use crate::call::{self, Call, Calls, Withdrawal, Work};
use crate::{
    AskRefusal, CallName, Domain, HostEffect, HostInput, HostProblem, HostReply, Limits, RelayName, Request, Returned,
};

#[derive(Debug)]
pub(crate) struct Relay {
    pub(crate) name: CallName,
    pub(crate) tool: Box<[u8]>,
    pub(crate) effect: HostEffect,
    pub(crate) input: HostInput,
    pub(crate) timeout: Duration,
    pub(crate) caller_deadline: Time,
    pub(crate) stopped: Option<Withdrawal>,
    pub(crate) unknown: bool,
    state: State,
}

#[derive(Debug)]
enum State {
    Sending { attempt: u32, deadline: Time },
    Withdrawing { attempt: u32 },
    Backoff { attempt: u32, at: Time },
    Closed,
}

#[expect(
    clippy::too_many_arguments,
    reason = "host admission checks the owning conversation and immutable call fields"
)]
pub(super) fn host_call(
    run: &Run,
    run_id: Id<Run>,
    conversations: &mut Slab<Conversation>,
    calls: &mut Calls,
    alarms: &mut Deadlines<Alarm>,
    conversation: Id<Conversation>,
    made: Asking,
    tool: Box<[u8]>,
    effect: HostEffect,
    input: HostInput,
    env: &Env<Limits>,
    out: &mut Queue<Request>,
) {
    let Asking { name, call, deadline } = made;
    if conversations.get(conversation).expect("caller lives").asker.is_some() {
        out.push(Request::Return { spent: 0, call, result: Returned::HostRejected(HostProblem::Undeclared) });
        return;
    }
    let mut timeout = None;
    for declaration in &run.charter.grants.host_tools {
        if declaration.name == tool {
            if declaration.effect != effect {
                out.push(Request::Return { spent: 0, call, result: Returned::HostRejected(HostProblem::Effect) });
                return;
            }
            timeout = Some(declaration.timeout.min(env.limits.host_timeout_max));
        }
    }
    let Some(timeout) = timeout else {
        out.push(Request::Return { spent: 0, call, result: Returned::HostRejected(HostProblem::Undeclared) });
        return;
    };
    if input.bytes().len() > usize::try_from(env.limits.host_input_bytes).expect("byte cap fits") {
        out.push(Request::Return { spent: 0, call, result: Returned::HostRejected(HostProblem::TooLarge) });
        return;
    }
    if name.activation != run.activation || name.completion == 0 {
        out.push(Request::Return { spent: 0, call, result: Returned::Refused { refusal: AskRefusal::Name } });
        return;
    }
    if env.now >= deadline || env.now >= run.deadline {
        out.push(Request::Return { spent: 0, call, result: Returned::TimedOut });
        return;
    }
    if calls.is_full() {
        out.push(Request::Return { spent: 0, call, result: Returned::Busy });
        return;
    }
    let relay = Relay {
        name,
        tool,
        effect,
        input,
        timeout,
        caller_deadline: deadline.min(run.deadline),
        stopped: None,
        unknown: false,
        state: State::Closed,
    };
    let work = Work::Host(relay);
    let id = begin_call(calls, alarms, Call { run: run_id, conversation, owner: call, work }, deadline);
    match &mut calls.get_mut(id).expect("inserted above").work {
        Work::Host(relay) => host_send(relay, id, run.host_name, 1, env.now, out),
        Work::Child(_) | Work::Landing(_) => unreachable!("inserted as host relay"),
    }
    let caller = conversations.get_mut(conversation).expect("caller lives");
    caller.calls = caller.calls.checked_add(1).expect("bounded calls");
    host_follow(calls, alarms, id);
}

fn host_send(relay: &mut Relay, id: Id<Call>, host_run: Token, attempt: u32, now: Time, out: &mut Queue<Request>) {
    let deadline = now.saturating_add(relay.timeout).min(relay.caller_deadline);
    assert!(now < deadline, "admission and recovery refuse expired relays");
    out.push(Request::HostCall {
        host_run,
        relay: RelayName { owner: id.token(), attempt },
        name: relay.name,
        tool: relay.tool.clone(),
        effect: relay.effect,
        input: relay.input.clone(),
        deadline,
    });
    relay.state = State::Sending { attempt, deadline };
}

fn host_follow(calls: &Calls, alarms: &mut Deadlines<Alarm>, id: Id<Call>) {
    let relay = match &calls.get(id).expect("host call lives").work {
        Work::Host(relay) => relay,
        Work::Child(_) | Work::Landing(_) => unreachable!("host handler owns its call"),
    };
    let at = match relay.state {
        State::Sending { deadline, .. } => Some(deadline),
        State::Backoff { at, .. } => Some(at),
        State::Withdrawing { .. } | State::Closed => None,
    };
    if let Some(at) = at {
        alarms.arm(Alarm::Host { call: id }, at).expect("two alarm slots per call");
    } else {
        alarms.cancel(Alarm::Host { call: id });
    }
}

fn host_return(domain: &mut Domain, id: Id<Call>, result: Returned, out: &mut Queue<Request>) {
    let call = domain.calls.get(id).expect("host call lives");
    domain.facts.about(call.run.token());
    let owner = call.owner;
    out.push(Request::Return { spent: 0, call: owner, result });
    retire_call(&mut domain.calls, &mut domain.alarms, &mut domain.conversations, id);
}

pub(super) fn host_stop(domain: &mut Domain, id: Id<Call>, why: Withdrawal, out: &mut Queue<Request>) {
    let call = domain.calls.get_mut(id).expect("caller lives until relay settles");
    let relay = match &mut call.work {
        Work::Host(relay) => relay,
        Work::Child(_) | Work::Landing(_) => unreachable!("host stop dispatched by work kind"),
    };
    relay.stopped = relay.stopped.or(Some(why));
    let stage = mem::replace(&mut relay.state, State::Closed);
    let result = match stage {
        State::Sending { attempt, .. } => {
            out.push(Request::WithdrawHost { relay: RelayName { owner: id.token(), attempt } });
            relay.state = State::Withdrawing { attempt };
            None
        }
        State::Withdrawing { attempt } => {
            relay.state = State::Withdrawing { attempt };
            None
        }
        State::Backoff { .. } => Some(if relay.unknown { Returned::HostUnknown } else { call::stopped(why) }),
        State::Closed => unreachable!("only a live call stops"),
    };
    if let Some(result) = result {
        host_return(domain, id, result, out);
    } else {
        host_follow(&domain.calls, &mut domain.alarms, id);
    }
}

pub(crate) fn host_alarm(domain: &mut Domain, env: &Env<Limits>, id: Id<Call>, out: &mut Queue<Request>) {
    let call = domain.calls.get_mut(id).expect("alarm belongs to live host call");
    let run = domain.runs.get(call.run).expect("run waits for calls");
    let active = may_finish(&run.state);
    let relay = match &mut call.work {
        Work::Host(relay) => relay,
        Work::Child(_) | Work::Landing(_) => unreachable!("host alarm belongs to host work"),
    };
    let stage = mem::replace(&mut relay.state, State::Closed);
    let result = match stage {
        State::Sending { attempt, .. } => {
            out.push(Request::WithdrawHost { relay: RelayName { owner: id.token(), attempt } });
            relay.state = State::Withdrawing { attempt };
            None
        }
        State::Backoff { attempt, .. } => {
            if active && relay.stopped.is_none() && env.now < relay.caller_deadline {
                host_send(relay, id, run.host_name, attempt, env.now, out);
                None
            } else {
                Some(if relay.unknown { Returned::HostUnknown } else { Returned::Busy })
            }
        }
        State::Withdrawing { .. } | State::Closed => unreachable!("non-timed host stage"),
    };
    if let Some(result) = result {
        host_return(domain, id, result, out);
    } else {
        host_follow(&domain.calls, &mut domain.alarms, id);
    }
}

pub(crate) fn host_returned(
    domain: &mut Domain,
    env: &Env<Limits>,
    name: RelayName,
    reply: HostReply,
    out: &mut Queue<Request>,
) {
    let id = Id::<Call>::from_token(name.owner);
    let Some(call) = domain.calls.get(id) else {
        return;
    };
    if domain.calls.find(call.conversation, call.owner) != Some(id) {
        return;
    }
    let relay = match &call.work {
        Work::Host(relay) => relay,
        Work::Child(_) | Work::Landing(_) => return,
    };
    let expected = match relay.state {
        State::Sending { attempt, .. } | State::Withdrawing { attempt } => Some(attempt),
        State::Backoff { .. } | State::Closed => None,
    };
    if expected != Some(name.attempt) {
        return;
    }
    let call = domain.calls.get_mut(id).expect("named call is live");
    let run = domain.runs.get(call.run).expect("run waits for relay terminal");
    let active = may_finish(&run.state);
    let relay = match &mut call.work {
        Work::Host(relay) => relay,
        Work::Child(_) | Work::Landing(_) => unreachable!("verified host work"),
    };
    relay.state = State::Closed;
    let result = match reply {
        HostReply::Answered(answer) => {
            Some(if answer.text().len() <= usize::try_from(env.limits.host_reply_bytes).expect("cap fits") {
                Returned::HostAnswered(answer)
            } else {
                Returned::HostTooLarge {
                    bytes: u32::try_from(answer.text().len()).expect("constructor-bounded host answer"),
                    max: env.limits.host_reply_bytes,
                }
            })
        }
        HostReply::Busy => {
            let exhausted = if relay.unknown { Returned::HostUnknown } else { Returned::Busy };
            host_recover(relay, name.attempt, active, env, exhausted)
        }
        HostReply::Withdrawn | HostReply::Unanswered(_) => {
            relay.unknown = true;
            host_recover(relay, name.attempt, active, env, Returned::HostUnknown)
        }
        HostReply::TooLarge => Some(Returned::HostReportedTooLarge),
    };
    if let Some(result) = result {
        host_return(domain, id, result, out);
    } else {
        host_follow(&domain.calls, &mut domain.alarms, id);
    }
}

fn host_recover(
    relay: &mut Relay,
    attempt: u32,
    active: bool,
    env: &Env<Limits>,
    exhausted: Returned,
) -> Option<Returned> {
    let at = env.now.saturating_add(env.limits.host_backoff);
    if !active || relay.stopped.is_some() || at >= relay.caller_deadline {
        return Some(exhausted);
    }
    let Some(next) = attempt.checked_add(1) else {
        return Some(exhausted);
    };
    relay.state = State::Backoff { attempt: next, at };
    None
}
