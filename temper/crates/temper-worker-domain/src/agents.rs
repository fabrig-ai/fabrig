//! The agent capability at the worker root (jig's `domain/hosts/README.md`, sections 4 and 5.1, and `domain/hosts/hub.md`, section 8
//! ): the hub's requests to smith's parent face, smith's notices
//! to the hub, and smith's payloads by hand-out. A start goes out to be
//! decoded, and comes back as smith's, which the root completes with where
//! the repositories sit; a turn, a call's name and a run's last word go out
//! to be encoded, and what they are for waits in order; smith's facts go out
//! to be decoded as jig's activity. An agent that has gone waits behind its
//! hand-outs. The hub hears an agent has started only once smith admits its
//! run. A refused start waits for cleanup's bounded operator evidence
//! (`temper/docs/design/domain/worker/README.md`, 4.3).

use alloc::boxed::Box;
use jig_domain_host as host;
use jig_domain_link::Phase;
use skein_lib::{Env, List, Queue, Time, Token};
use smith_host_domain as smith;

use crate::boundary::{Encodable, Encoded, Request};
use crate::hand_out::{Finish, Pending, StartClass, StartRefusal};
use crate::{Domain, Limits, route, smith_delivery, translate, workspace};

/// An agent smith started for a hosted run, by the run's token.
#[derive(Debug)]
pub(crate) struct Bound {
    pub(crate) agent: Token,
    /// smith admitted its run, and the hub was told it started.
    pub(crate) admitted: bool,
    /// The charter's model-indexed call names (domain/worker/README.md, 4.3).
    models: Box<[crate::hand_out::ModelName]>,
    /// Bounded operator evidence retained until process cleanup.
    detail: Box<[u8]>,
    /// A refused start waits for its process to drain before answering.
    refused: Option<StartRefusal>,
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
    Save,
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
        host::ToAgent::Drain { agent } => smith::parent::Event::Drain { agent },
        host::ToAgent::Cancel { agent } => {
            let mut cut = false;
            for (owner, bound) in &domain.agents {
                if bound.agent == agent {
                    cut = crate::stopping::cut(domain, *owner);
                    break;
                }
            }
            if cut { smith::parent::Event::Drop { agent } } else { smith::parent::Event::Stop { agent } }
        }
        // The hub hands its agent only the LLM accounts' grants, and keeps
        // the application credentials, the git identities, to itself (jig's
        // domain/hosts/hub.md, 2): the table needs no reading.
        host::ToAgent::Grant { agent, grant } => smith::parent::Event::Grant {
            agent,
            grant: translate::grant(grant).expect("the hub forwards only account grants"),
        },
        host::ToAgent::Answer { agent, call: name, answer: reply } => {
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
            let models = match domain.starting_models.remove(&client) {
                Some(models) => models,
                None => Box::new([]),
            };
            let old = domain
                .agents
                .insert(client, Bound { agent, admitted: false, models, detail: Box::new([]), refused: None })
                .expect("smith bounds its agents");
            assert!(old.is_none(), "one agent per hosted run");
            if crate::stopping::cut(domain, client) {
                route::agent_step(domain, env, smith::Input::Parent(smith::parent::Event::Drop { agent }));
            }
            return;
        }
        smith::parent::Request::Admitted { client } => {
            let bound = domain.agents.get_mut(&client).expect("started before admitted");
            bound.admitted = true;
            host::Event::Agent(host::FromAgent::Started { owner: client, agent: bound.agent })
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
            host::Event::Agent(host::FromAgent::Withdrawn { owner: client, call: name.clone() })
        }
        smith::parent::Request::Answered { client, answer, detail } => {
            return answered(domain, env, client, answer, &detail, out);
        }
        smith::parent::Request::Told { client, body } => {
            domain.facts_out = domain.facts_out.saturating_add(1);
            return out.push(Request::Fact { owner: client, body });
        }
        smith::parent::Request::Waiting { client, read: _ } => {
            host::Event::Agent(host::FromAgent::Yielded { owner: client })
        }
        smith::parent::Request::Long { client, operation, span } => host::Event::Agent(host::FromAgent::Activity {
            owner: client,
            activity: host::AgentActivity::Long { operation: translate::operation(operation), span },
        }),
        smith::parent::Request::LongDone { client } => {
            host::Event::Agent(host::FromAgent::Activity { owner: client, activity: host::AgentActivity::LongDone })
        }
        smith::parent::Request::Unauthorized { client, account, generation } => {
            host::Event::Agent(host::FromAgent::Unauthorized { owner: client, account, generation })
        }
        smith::parent::Request::Limited { client, account, reset } => {
            host::Event::Agent(host::FromAgent::Limited { owner: client, account, reset })
        }
        smith::parent::Request::Refused { client, account, code, reason } => {
            host::Event::Agent(host::FromAgent::AccountRefused { owner: client, account, code, reason })
        }
        smith::parent::Request::Faulted { client, fault } => {
            host::Event::Agent(host::FromAgent::Faulted { owner: client, fault: translate::fault(fault) })
        }
        smith::parent::Request::MessageRefused { client, name, reason } => {
            host::Event::Agent(host::FromAgent::Refused { owner: client, name, reason: translate::refusal(reason) })
        }
        smith::parent::Request::Unread { client, names } => {
            if admitted(domain, client) {
                for name in names {
                    let refused = host::Event::Agent(host::FromAgent::Refused {
                        owner: client,
                        name,
                        reason: host::MessageRefusal::Ending,
                    });
                    route::host_step(domain, env, refused);
                }
            }
            return;
        }
        smith::parent::Request::Gone { client, end, detail } => return ended(domain, env, client, end, detail),
    };
    route::host_step(domain, env, event);
}

/// Retain operator evidence beside the result until its process has drained.
fn answered(
    domain: &mut Domain,
    env: &Env<Limits>,
    client: Token,
    answer: smith::Answer,
    detail: &[u8],
    out: &mut Queue<Request>,
) {
    let smith::Answer { read, turns, spent, result } = answer;
    let bound = domain.agents.get_mut(&client).expect("a started agent answers");
    bound.detail = translate::failure_detail(&result, detail, env.limits.host.detail_bytes);
    let finish = Finish::of(&result, &bound.models);
    domain.pending.push(Pending::Result { owner: client, turns, spent, read, finish });
    out.push(Request::Encode { owner: client, value: Encodable::Result(result) });
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
        smith::Ask::Save => Asked::Save,
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
pub(crate) fn decoded(
    domain: &mut Domain,
    env: &Env<Limits>,
    owner: Token,
    start: Result<Box<smith::Start>, host::Undecoded>,
    endpoints: Box<[Box<[u8]>]>,
) {
    let Some(hosting) = domain.host.hosting(owner) else { return };
    if hosting.phase != Phase::Starting {
        return route::host_step(
            domain,
            env,
            host::Event::Agent(host::FromAgent::Gone { owner, detail: Box::new([]) }),
        );
    }
    let event = match start {
        Ok(mut start) => {
            route::host_step(
                domain,
                env,
                host::Event::Agent(host::FromAgent::Decoded { owner, contract: start.charter.value().grants.contract }),
            );
            let models = crate::hand_out::model_names(&start, endpoints);
            let old = domain.starting_models.insert(owner, models).expect("one name list per hosted start");
            assert!(old.is_none(), "each start is decoded once");
            // Earlier answers cross the same agent channel as live replies
            // (`domain/agent/tools.md`), after the protocol decodes the start.
            for call in &mut start.answered {
                if let smith::SavedReply::Host { body, .. } = &call.reply
                    && body.len() > skein_lib::usize_of(env.limits.reply_bytes)
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
        Err(host::Undecoded::Version(mismatch)) => host::FromAgent::StartVersion { owner, mismatch },
        Err(invalid) => {
            let cause = match &invalid {
                host::Undecoded::Version(_) => unreachable!("version handled above"),
                host::Undecoded::Transcript => host::StartCause::Transcript,
                host::Undecoded::Charter | host::Undecoded::CallName | host::Undecoded::Evidence => {
                    host::StartCause::Other
                }
            };
            let why: &[u8] = match invalid {
                host::Undecoded::Version(_) => unreachable!("version handled above"),
                host::Undecoded::Charter => b"charter",
                host::Undecoded::Transcript => b"transcript",
                host::Undecoded::CallName => b"call name",
                host::Undecoded::Evidence => b"delivery evidence",
            };
            host::FromAgent::StartRefused { owner, cause, why: Box::from(why) }
        }
    };
    route::host_step(domain, env, host::Event::Agent(event));
}

/// A payload handed out came back: what it was for goes on. An agent that
/// went meanwhile is gone once the last of its hand-outs is back.
pub(crate) fn encoded(domain: &mut Domain, env: &Env<Limits>, owner: Token, encoded: Encoded) {
    let pending = domain.pending.pop().expect("each hand-out comes back once");
    assert!(pending.owner() == owner, "hand-outs come back in order");
    match pending {
        Pending::Turn { .. } => {
            let event = match encoded {
                Encoded::Turn(turn) => host::Event::Agent(host::FromAgent::Turn { owner, turn }),
                Encoded::Undecodable => {
                    host::Event::Agent(host::FromAgent::Faulted { owner, fault: host::AgentFailure::Rules })
                }
                Encoded::CallName(_) | Encoded::Result(_) => unreachable!("a hand-out comes back as it went"),
            };
            route::host_step(domain, env, event);
        }
        Pending::Call { call, .. } => named(domain, env, owner, call, encoded),
        Pending::Result { .. } => result_encoded(domain, env, pending, encoded),
    }
    if !hand_out_pending(domain, owner)
        && let Some(detail) = domain.gone.remove(&owner)
    {
        gone(domain, env, owner, detail);
    }
}

/// A result finishes an admitted run; a refusal waits for cleanup evidence.
fn result_encoded(domain: &mut Domain, env: &Env<Limits>, pending: Pending, encoded: Encoded) {
    let (owner, turns, spent, read, finish) = match pending {
        Pending::Result { owner, turns, spent, read, finish } => (owner, turns, spent, read, finish),
        Pending::Turn { .. } | Pending::Call { .. } => unreachable!("a result hand-out"),
    };
    let body = match encoded {
        Encoded::Result(body) => Some(body),
        Encoded::Undecodable => None,
        Encoded::Turn(_) | Encoded::CallName(_) => unreachable!("a hand-out comes back as it went"),
    };
    let event = match finish {
        Finish::Refused(class) => {
            let why = match body {
                Some(body) => body,
                None => Box::from(&b"refusal"[..]),
            };
            let bound = domain.agents.get_mut(&owner).expect("a refused agent waits for its hand-outs");
            bound.refused = Some(StartRefusal { class, why });
            return;
        }
        Finish::Ended | Finish::Parked | Finish::Failed { .. } => match body {
            Some(body) => {
                let finish = match finish {
                    Finish::Ended => host::Finish::Ended { outcome: body },
                    Finish::Parked => host::Finish::Parked,
                    Finish::Failed { failure, model } => host::Finish::Failed { failure, model },
                    Finish::Refused(_) => unreachable!("a refusal waits for cleanup"),
                };
                host::FromAgent::Finished { owner, turns, spent, read, finish }
            }
            None => host::FromAgent::Faulted { owner, fault: host::AgentFailure::Rules },
        },
    };
    route::host_step(domain, env, host::Event::Agent(event));
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
            return route::host_step(
                domain,
                env,
                host::Event::Agent(host::FromAgent::Faulted { owner, fault: host::AgentFailure::Rules }),
            );
        }
        Encoded::Turn(_) | Encoded::Result(_) => unreachable!("a hand-out comes back as it went"),
    };
    let entry = domain.calls.get_mut(&(owner, call)).expect("a call waits for its name");
    entry.name = Some(name.clone());
    let deadline = entry.deadline.saturating_since(env.now);
    let withdrawn = entry.withdrawn;
    let ask = match entry.ask.take().expect("a call goes to the hub once") {
        Asked::Relay { tool, writes, input } => host::Ask::Relay { tool, writes, input, deadline },
        Asked::Save => host::Ask::Save,
        Asked::Deliver { title, body } => host::Ask::Deliver { title, body },
    };
    if withdrawn {
        route::host_step(domain, env, host::Event::Agent(host::FromAgent::Called { owner, call: name.clone(), ask }));
        route::host_step(domain, env, host::Event::Agent(host::FromAgent::Withdrawn { owner, call: name }));
    } else {
        route::host_step(domain, env, host::Event::Agent(host::FromAgent::Called { owner, call: name, ask }));
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
/// refused before any process ran keeps its refusal class; any other end waits
/// behind the run's hand-outs.
fn ended(domain: &mut Domain, env: &Env<Limits>, owner: Token, end: smith::End, detail: Box<[u8]>) {
    drop(domain.starting_models.remove(&owner));
    let started = domain.agents.contains_key(&owner);
    let unstarted = match end {
        smith::End::Busy | smith::End::Invalid(_) | smith::End::Unspawned => true,
        smith::End::Exited { .. } | smith::End::Settled => false,
    };
    assert!(started || unstarted, "an agent that ran started first");
    if let Some(why) = translate::unstarted(end) {
        domain.agents.remove(&owner);
        let class = match end {
            smith::End::Busy => StartClass::Busy,
            smith::End::Invalid(smith::Invalid::Transcript) => StartClass::Invalid(host::StartCause::Transcript),
            smith::End::Invalid(_) | smith::End::Unspawned | smith::End::Exited { .. } | smith::End::Settled => {
                StartClass::Invalid(host::StartCause::Other)
            }
        };
        let refused = class.event(owner, Box::from(why));
        route::host_step(domain, env, host::Event::Agent(refused));
    } else if hand_out_pending(domain, owner) {
        let old = domain.gone.insert(owner, detail).expect("room for every agent's end");
        assert!(old.is_none(), "an agent goes once");
    } else {
        gone(domain, env, owner, detail);
    }
}

/// The hub hears the agent has gone, unless its run has answered already.
fn gone(domain: &mut Domain, env: &Env<Limits>, owner: Token, detail: Box<[u8]>) {
    let bound = domain.agents.remove(&owner);
    let (detail, refused) = match bound {
        Some(bound) => (diagnostic(&bound.detail, &detail, env.limits.host.detail_bytes), bound.refused),
        None => (detail, None),
    };
    if let Some(hosting) = domain.host.hosting(owner)
        && hosting.phase != Phase::Answered
    {
        let event = match refused {
            Some(StartRefusal { class, why }) => {
                let why = diagnostic(&why, &detail, env.limits.host.detail_bytes);
                class.event(owner, why)
            }
            None => host::FromAgent::Gone { owner, detail },
        };
        route::host_step(domain, env, host::Event::Agent(event));
    }
}

/// Keep the reason's prefix and the cleanup's tail, reserving room for both.
fn diagnostic(reason: &[u8], cleanup: &[u8], bound: u32) -> Box<[u8]> {
    let mut text = List::with_capacity(bound);
    let room = skein_lib::usize_of(bound);
    let keep = if cleanup.is_empty() {
        room
    } else {
        let reserved = cleanup.len().min(room.saturating_sub(1).checked_div(2).expect("nonzero divisor"));
        room.saturating_sub(reserved).saturating_sub(1)
    };
    for byte in reason.get(..reason.len().min(keep)).expect("bounded prefix") {
        text.push(*byte).expect("reason room");
    }
    if !text.is_empty() && !cleanup.is_empty() && text.room() > 0 {
        text.push(b'\n').expect("separator room");
    }
    let from = cleanup.len().saturating_sub(skein_lib::usize_of(text.room()));
    for byte in cleanup.get(from..).expect("bounded cleanup tail") {
        text.push(*byte).expect("cleanup room");
    }
    text.into_boxed()
}
