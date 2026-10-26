//! The agent capability at the worker root (jig's `domain/hosts.md`, sections
//! 4, 5.1 and 6.8): the hub's requests to smith's parent face, smith's notices
//! to the hub, and smith's payloads by hand-out. A start goes out to be
//! decoded, and comes back as smith's, which the root completes with where
//! the repositories sit; a turn, a call's name and a run's last word go out
//! to be encoded, and what they are for waits in order; smith's facts go out
//! to be decoded as jig's activity. An agent that has gone waits behind its
//! hand-outs. The hub hears an agent has started only once smith admits its
//! run.

use alloc::boxed::Box;
use jig_domain_host as host;
use skein_lib::{Env, Queue, Time, Token};
use smith_host_domain as smith;

use crate::boundary::{Encodable, Encoded, Invalid, Request};
use crate::hand_out::{Finish, Pending};
use crate::{Domain, Limits, route, smith_delivery, translate, workspace};

/// An agent smith started for a hosted run, by the run's token.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub(crate) struct Bound {
    pub(crate) agent: Token,
    /// smith admitted its run, and the hub was told it started.
    pub(crate) admitted: bool,
}

/// A call of a run, from smith's `Called` until the hub's reply.
#[derive(Debug)]
pub(crate) struct Call {
    agent: Token,
    deadline: Time,
    /// What the run asks, until its name comes back encoded.
    ask: Option<Asked>,
    /// The name the hub knows it by, once encoded.
    name: Option<Box<[u8]>>,
    /// The run withdrew it before its name came back.
    withdrawn: bool,
}

/// What a call asks, once its delivery's fields have passed.
#[derive(Debug)]
enum Asked {
    Relay { tool: Box<[u8]>, writes: bool, input: Box<[u8]> },
    Deliver { title: Box<[u8]>, body: Box<[u8]> },
}

/// One of the hub's requests to the agent capability.
pub(crate) fn to_agent(domain: &mut Domain, env: &Env<Limits>, request: host::ToAgent, out: &mut Queue<Request>) {
    let event = match request {
        host::ToAgent::Start { owner, start } => {
            let run = domain.host.hosting(owner).expect("a hosted start").run;
            return out.push(Request::Decode { agent: owner, run, start });
        }
        host::ToAgent::Message { agent, name, sender, words, finish_requested } => {
            smith::parent::Event::Message { agent, name, label: sender, text: words, finish_requested }
        }
        host::ToAgent::Acknowledge { agent, turn } => smith::parent::Event::Acknowledge { agent, turn },
        host::ToAgent::Cancel { agent } => smith::parent::Event::Stop { agent },
        // The hub hands its agent only the LLM accounts' grants, and keeps
        // the application credentials, the git identities, to itself (jig's
        // domain/hosts.md, 6.2; jig 7ca3c388): the table needs no reading.
        host::ToAgent::Grant { agent, grant } => smith::parent::Event::Grant {
            agent,
            grant: translate::grant(grant).expect("the hub forwards only account grants"),
        },
        host::ToAgent::Answer { agent, call: name, reply } => {
            let mut found = None;
            for (key, call) in &domain.calls {
                if call.agent == agent && call.name.as_deref() == Some(name.as_ref()) {
                    found = Some(*key);
                    break;
                }
            }
            let key = found.expect("the hub answers a call it was given once");
            domain.calls.remove(&key);
            let reply = smith_delivery::reply(domain, env.limits.reply_bytes, reply);
            smith::parent::Event::Answer { agent, call: key.1, reply }
        }
    };
    route::agent_step(domain, env, smith::Input::Parent(event));
}

/// One of smith's notices: to the hub, or out by hand-out.
pub(crate) fn up(domain: &mut Domain, env: &Env<Limits>, request: smith::parent::Request, out: &mut Queue<Request>) {
    let event = match request {
        smith::parent::Request::Started { client, agent } => {
            let old = domain.agents.insert(client, Bound { agent, admitted: false }).expect("smith bounds its agents");
            assert!(old.is_none(), "one agent per hosted run");
            return;
        }
        smith::parent::Request::Admitted { client } => {
            let bound = domain.agents.get_mut(&client).expect("started before admitted");
            bound.admitted = true;
            host::Event::Started { owner: client, agent: bound.agent }
        }
        smith::parent::Request::Turn { client, turn } => {
            domain.pending.push(Pending::Turn { owner: client });
            return out.push(Request::Encode { owner: client, value: Encodable::Turn(turn) });
        }
        smith::parent::Request::Called { client, logical_run: _, call, name, deadline, ask } => {
            return called(domain, env, client, call, name, deadline, ask, out);
        }
        smith::parent::Request::Withdrawn { client, call } => {
            let Some(entry) = domain.calls.get_mut(&(client, call)) else { return };
            entry.withdrawn = true;
            let Some(name) = &entry.name else { return };
            host::Event::Withdrawn { owner: client, call: name.clone() }
        }
        smith::parent::Request::Answered { client, answer, detail: _ } => {
            let smith::Answer { read, turns, spent, result } = answer;
            domain.pending.push(Pending::Result { owner: client, turns, spent, read, finish: Finish::of(&result) });
            return out.push(Request::Encode { owner: client, value: Encodable::Result(result) });
        }
        smith::parent::Request::Told { client, body } => {
            domain.facts_out = domain.facts_out.saturating_add(1);
            return out.push(Request::Fact { owner: client, body });
        }
        smith::parent::Request::Waiting { client, read: _ } => host::Event::Yielded { owner: client },
        smith::parent::Request::Long { client, operation, span } => host::Event::Activity {
            owner: client,
            activity: host::Activity::Long { operation: translate::operation(operation), span },
        },
        smith::parent::Request::LongDone { client } => {
            host::Event::Activity { owner: client, activity: host::Activity::LongDone }
        }
        smith::parent::Request::Unauthorized { client, account, generation } => {
            host::Event::Unauthorized { owner: client, account, generation }
        }
        smith::parent::Request::Limited { client, account, reset } => {
            host::Event::Limited { owner: client, account, reset }
        }
        smith::parent::Request::Refused { client, account, code, reason } => {
            host::Event::AccountRefused { owner: client, account, code, reason }
        }
        smith::parent::Request::Faulted { client, fault } => {
            // Before its run is admitted, the agent's end answers for it.
            if !admitted(domain, client) {
                return;
            }
            host::Event::Faulted { owner: client, fault: translate::fault(fault) }
        }
        smith::parent::Request::MessageRefused { client, name, reason } => {
            host::Event::Refused { owner: client, name, reason: translate::refusal(reason) }
        }
        smith::parent::Request::Unread { client, names } => {
            if admitted(domain, client) {
                for name in names {
                    let refused = host::Event::Refused { owner: client, name, reason: host::MessageRefusal::Ending };
                    route::host_step(domain, env, refused);
                }
            }
            return;
        }
        smith::parent::Request::Gone { client, end, detail } => return ended(domain, env, client, end, detail),
    };
    route::host_step(domain, env, event);
}

/// Whether the hub was told the agent of the run `owner` started.
fn admitted(domain: &Domain, owner: Token) -> bool {
    match domain.agents.get(&owner) {
        Some(bound) => bound.admitted,
        None => false,
    }
}

/// A call of the run `client`: its name goes out to be encoded. A delivery
/// whose fields are not temper's commit message is refused at once.
#[expect(clippy::too_many_arguments, reason = "one of smith's calls enters with its fields")]
fn called(
    domain: &mut Domain,
    env: &Env<Limits>,
    client: Token,
    call: Token,
    name: smith::CallName,
    deadline: Time,
    ask: smith::Ask,
    out: &mut Queue<Request>,
) {
    let agent = domain.agents.get(&client).expect("an admitted run calls").agent;
    let asked = match ask {
        smith::Ask::Host { tool, effect, body } => {
            let writes = match effect {
                smith::Effect::Read => false,
                smith::Effect::Write => true,
            };
            Asked::Relay { tool, writes, input: body }
        }
        smith::Ask::Deliver { fields } => match smith_delivery::fields(fields, env.limits.checkout.message_bytes) {
            Some(smith_delivery::Fields { title, body }) => Asked::Deliver { title, body },
            None => {
                let refusal =
                    smith::DeliveryRefusal::new(None, Box::from(&b"delivery needs bounded title and body"[..]))
                        .expect("bounded refusal");
                let reply = smith::Reply::Delivery(smith::Delivery::Refused(refusal));
                let answer = smith::parent::Event::Answer { agent, call, reply };
                return route::agent_step(domain, env, smith::Input::Parent(answer));
            }
        },
    };
    let entry = Call { agent, deadline, ask: Some(asked), name: None, withdrawn: false };
    let old = domain.calls.insert((client, call), entry).expect("smith bounds its calls");
    assert!(old.is_none(), "one callback per call");
    domain.pending.push(Pending::Call { owner: client, call });
    out.push(Request::Encode { owner: client, value: Encodable::CallName(name) });
}

/// The hub's start, decoded: smith starts the agent in the workspace the
/// checkout prepared, each repository's directory added; a run with no
/// repository has none. A start whose run is no longer starting ends at
/// once, and one that did not decode is refused as invalid.
pub(crate) fn decoded(domain: &mut Domain, env: &Env<Limits>, owner: Token, start: Result<Box<smith::Start>, Invalid>) {
    let Some(hosting) = domain.host.hosting(owner) else { return };
    if hosting.phase != host::Phase::Starting {
        return route::host_step(domain, env, host::Event::Gone { owner, detail: Box::new([]) });
    }
    let event = match start {
        Ok(mut start) => {
            // Earlier answers cross the same agent channel as live replies
            // (`domain/agent.md`, 4.3), after the protocol decodes the start.
            for call in &mut start.answered {
                if let smith::SavedReply::Host { body, .. } = &call.reply
                    && body.len() > usize::try_from(env.limits.reply_bytes).expect("u32 fits usize")
                {
                    call.reply = smith::SavedReply::TooLarge;
                }
            }
            match start.workspace {
                Some(workspace) => {
                    let (directory, directories) = workspace::directories(domain, workspace);
                    start.workspace = Some(directory);
                    start.directories = directories;
                }
                None => start.directories = Box::new([]),
            }
            let spawn = smith::parent::Event::Spawn { client: owner, start: *start };
            return route::agent_step(domain, env, smith::Input::Parent(spawn));
        }
        Err(invalid) => {
            let why: &[u8] = match invalid {
                Invalid::Charter => b"charter",
                Invalid::Transcript => b"transcript",
                Invalid::CallName => b"call name",
                Invalid::Evidence => b"delivery evidence",
            };
            host::FromAgent::StartRefused { owner, cause: host::StartCause::Other, why: Box::from(why) }
        }
    };
    route::host_step(domain, env, host::Event::from_agent(event));
}

/// A payload handed out came back: what it was for goes on. An agent that
/// went meanwhile is gone once the last of its hand-outs is back.
pub(crate) fn encoded(domain: &mut Domain, env: &Env<Limits>, owner: Token, encoded: Encoded) {
    let pending = domain.pending.pop().expect("each hand-out comes back once");
    assert!(pending.owner() == owner, "hand-outs come back in order");
    match pending {
        Pending::Turn { .. } => {
            let event = match encoded {
                Encoded::Turn(turn) => host::Event::Turn { owner, turn },
                Encoded::Undecodable => host::Event::Faulted { owner, fault: host::AgentFailure::Rules },
                Encoded::CallName(_) | Encoded::Result(_) => unreachable!("a hand-out comes back as it went"),
            };
            route::host_step(domain, env, event);
        }
        Pending::Call { call, .. } => named(domain, env, owner, call, encoded),
        Pending::Result { turns, spent, read, finish, .. } => {
            let event = match encoded {
                Encoded::Result(body) => match finish {
                    Finish::Ended => host::FromAgent::Finished {
                        owner,
                        turns,
                        spent,
                        read,
                        finish: host::Finish::Ended { outcome: body },
                    },
                    Finish::Parked => {
                        host::FromAgent::Finished { owner, turns, spent, read, finish: host::Finish::Parked }
                    }
                    Finish::Failed(failure) => host::FromAgent::Finished {
                        owner,
                        turns,
                        spent,
                        read,
                        finish: host::Finish::Failed { failure },
                    },
                    Finish::Refused(cause) => host::FromAgent::StartRefused { owner, cause, why: body },
                },
                // A refusal comes before smith admits the run, a failure after.
                Encoded::Undecodable => match finish {
                    Finish::Refused(cause) => {
                        host::FromAgent::StartRefused { owner, cause, why: Box::from(&b"refusal"[..]) }
                    }
                    Finish::Ended | Finish::Parked | Finish::Failed(_) => {
                        host::FromAgent::Faulted { owner, fault: host::AgentFailure::Rules }
                    }
                },
                Encoded::Turn(_) | Encoded::CallName(_) => unreachable!("a hand-out comes back as it went"),
            };
            route::host_step(domain, env, host::Event::from_agent(event));
        }
    }
    if !hand_out_pending(domain, owner)
        && let Some(detail) = domain.gone.remove(&owner)
    {
        gone(domain, env, owner, detail);
    }
}

/// A call's name came back: the call goes to the hub, and its withdrawal
/// after it if the run withdrew it meanwhile; the hub answers one whose name
/// is longer than it takes as unavailable. One that did not encode is
/// answered unavailable, and the agent faulted.
fn named(domain: &mut Domain, env: &Env<Limits>, owner: Token, call: Token, encoded: Encoded) {
    let name = match encoded {
        Encoded::CallName(name) => name,
        Encoded::Undecodable => {
            let entry = domain.calls.remove(&(owner, call)).expect("a call waits for its name");
            let answer = smith::parent::Event::Answer { agent: entry.agent, call, reply: smith::Reply::Unavailable };
            route::agent_step(domain, env, smith::Input::Parent(answer));
            return route::host_step(domain, env, host::Event::Faulted { owner, fault: host::AgentFailure::Rules });
        }
        Encoded::Turn(_) | Encoded::Result(_) => unreachable!("a hand-out comes back as it went"),
    };
    let entry = domain.calls.get_mut(&(owner, call)).expect("a call waits for its name");
    entry.name = Some(name.clone());
    let deadline = entry.deadline.saturating_since(env.now);
    let withdrawn = entry.withdrawn;
    let ask = match entry.ask.take().expect("a call goes to the hub once") {
        Asked::Relay { tool, writes, input } => host::Ask::Relay { tool, writes, input, deadline },
        Asked::Deliver { title, body } => host::Ask::Deliver { title, body },
    };
    if withdrawn {
        route::host_step(domain, env, host::Event::Called { owner, call: name.clone(), ask });
        route::host_step(domain, env, host::Event::Withdrawn { owner, call: name });
    } else {
        route::host_step(domain, env, host::Event::Called { owner, call: name, ask });
    }
}

/// Whether a hand-out of the run `owner` is still out.
fn hand_out_pending(domain: &Domain, owner: Token) -> bool {
    for pending in &domain.pending {
        if pending.owner() == owner {
            return true;
        }
    }
    false
}

/// The agent of the run `owner` has gone, as smith ended it: a start smith
/// refused before any process ran is refused as invalid; any other end waits
/// behind the run's hand-outs.
fn ended(domain: &mut Domain, env: &Env<Limits>, owner: Token, end: smith::End, detail: Box<[u8]>) {
    let started = domain.agents.remove(&owner).is_some();
    let unstarted = match end {
        smith::End::Busy | smith::End::Invalid(_) | smith::End::Unspawned => true,
        smith::End::Exited { .. } | smith::End::Settled => false,
    };
    assert!(started || unstarted, "an agent that ran started first");
    if let Some(why) = translate::unstarted(end) {
        let refused = host::FromAgent::StartRefused { owner, cause: host::StartCause::Other, why: Box::from(why) };
        route::host_step(domain, env, host::Event::from_agent(refused));
    } else if hand_out_pending(domain, owner) {
        let old = domain.gone.insert(owner, detail).expect("room for every agent's end");
        assert!(old.is_none(), "an agent goes once");
    } else {
        gone(domain, env, owner, detail);
    }
}

/// The hub hears the agent has gone, unless its run has answered already.
fn gone(domain: &mut Domain, env: &Env<Limits>, owner: Token, detail: Box<[u8]>) {
    if let Some(hosting) = domain.host.hosting(owner)
        && hosting.phase != host::Phase::Answered
    {
        route::host_step(domain, env, host::Event::Gone { owner, detail });
    }
}
