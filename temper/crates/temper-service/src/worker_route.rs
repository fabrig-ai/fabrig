//! Smith payloads are translated in the worker layer (jig's
//! protocol/channel.md, 6.1 and 6.4).
use alloc::boxed::Box;
use jig_domain_host as hub;
use skein_lib::{List, Reader, Token, Writer};
use smith_host_domain as smith;
use temper_worker_domain::{Encodable, Encoded, Invalid};

#[must_use]
pub fn encode(value: Encodable, agent: &crate::settings::Agent) -> Encoded {
    let limits = &agent.limits;
    let endpoints = &agent.endpoints;
    match value {
        Encodable::Turn(turn) => {
            match smith_protocol_channel::encode_turn(turn.body.value(), &limits.transcript, endpoints) {
                Ok(body) => Encoded::Turn(hub::Turn {
                    opens_window: turn.opens_window,
                    refused: Box::default(),
                    turn: turn.number,
                    spent: turn.spent,
                    read: turn.read,
                    body,
                }),
                Err(_) => Encoded::Undecodable,
            }
        }
        Encodable::CallName(name) => {
            let record = smith_channel::CallName::new(
                &limits.bodies,
                smith_channel::CallNameParts {
                    activation: name.activation,
                    completion: name.completion,
                    position: name.position,
                },
            );
            let Ok(record) = record else { return Encoded::Undecodable };
            let Ok(length) = usize::try_from(record.measure()) else { return Encoded::Undecodable };
            let mut writer = Writer::new(length);
            if record.encode(&mut writer).is_err() {
                return Encoded::Undecodable;
            }
            Encoded::CallName(writer.finish())
        }
        Encodable::Result(result) => match result {
            smith::RunResult::Accepted { outcome } => {
                match smith_protocol_channel::encode_result(outcome.value(), &limits.charter) {
                    Ok(body) => Encoded::Result(body),
                    Err(_) => Encoded::Undecodable,
                }
            }
            smith::RunResult::Refused { refusal } => Encoded::Result(Box::from(refused(refusal))),
            // The root retains these typed dispositions beside the hand-out.
            smith::RunResult::Parked | smith::RunResult::Failed { .. } => Encoded::Result(Box::default()),
        },
    }
}

fn refused(refusal: smith::Refusal) -> &'static [u8] {
    match refusal {
        smith::Refusal::Busy => b"agent busy",
        smith::Refusal::Invalid(invalid) => match invalid {
            smith::RunInvalid::HostPart { .. } => b"agent host part bound",
            smith::RunInvalid::Room { .. } => b"agent model room bound",
            smith::RunInvalid::Messages => b"agent messages invalid",
            smith::RunInvalid::CharterVersion => b"agent charter version invalid",
            smith::RunInvalid::MalformedCharter => b"agent charter malformed",
            smith::RunInvalid::Activation => b"agent activation invalid",
            smith::RunInvalid::Window => b"agent window invalid",
            smith::RunInvalid::Conventions => b"agent conventions invalid",
            smith::RunInvalid::TooLarge => b"agent charter too large",
            smith::RunInvalid::Workspace => b"agent workspace invalid",
            smith::RunInvalid::Grants => b"agent grants invalid",
            smith::RunInvalid::Outcome => b"agent outcome invalid",
            smith::RunInvalid::Budget(budget) => match budget {
                smith::BudgetRefusal::Ceiling(ceiling) => match ceiling {
                    smith::Ceiling::Turns(_) => b"agent turns ceiling",
                    smith::Ceiling::Spend(_) => b"agent spend ceiling",
                    smith::Ceiling::Time(_) => b"agent time ceiling",
                },
                smith::BudgetRefusal::Empty(part) => match part {
                    smith::Part::Turns => b"agent turns empty",
                    smith::Part::Spend => b"agent spend empty",
                    smith::Part::Time => b"agent time empty",
                },
                smith::BudgetRefusal::Reserve(part) => match part {
                    smith::Part::Turns => b"agent turns reserve",
                    smith::Part::Spend => b"agent spend reserve",
                    smith::Part::Time => b"agent time reserve",
                },
                smith::BudgetRefusal::Checks { .. } => b"agent checks reserve",
                smith::BudgetRefusal::Completion { .. } => b"agent completion reserve",
            },
            smith::RunInvalid::Price(rate) => match rate {
                smith::Rate::Input => b"agent input price invalid",
                smith::Rate::Cached => b"agent cached price invalid",
                smith::Rate::CacheWrite => b"agent cache write price invalid",
                smith::Rate::Output => b"agent output price invalid",
            },
            smith::RunInvalid::Effort => b"agent effort invalid",
            smith::RunInvalid::Llm => b"agent llm invalid",
            smith::RunInvalid::Conversation => b"agent conversation invalid",
            smith::RunInvalid::Endpoint => b"agent endpoint invalid",
        },
    }
}

#[must_use]
pub fn activity(body: &[u8], agent: &crate::settings::Agent) -> Option<hub::Activity> {
    let limits = &agent.limits;
    let mut reader = Reader::new(body);
    let Ok(fact) = smith_channel::Fact::decode(&limits.bodies, &mut reader) else { return None };
    if !reader.is_empty() {
        return None;
    }
    let kind = match fact.kind() {
        smith_channel::FactKind::Admitted => hub::CountKind::Admitted,
        smith_channel::FactKind::Ended => hub::CountKind::Ended,
        smith_channel::FactKind::LlmStarted => hub::CountKind::LlmStarted,
        smith_channel::FactKind::LlmRetried => hub::CountKind::LlmRetried,
        smith_channel::FactKind::LlmFinished => hub::CountKind::LlmFinished,
        smith_channel::FactKind::ToolStarted => hub::CountKind::ToolStarted,
        smith_channel::FactKind::ToolFinished => hub::CountKind::ToolFinished,
        smith_channel::FactKind::CheckStarted => hub::CountKind::CheckStarted,
        smith_channel::FactKind::CheckFinished => hub::CountKind::CheckFinished,
        smith_channel::FactKind::TextArrived => hub::CountKind::TextArrived,
    };
    Some(hub::Activity::Count { kind, elapsed: fact.elapsed(), count: fact.count() })
}

/// Decode the hub start, including its live ask (domain/worker.md, 4.1 and 7).
pub fn decode(run: Token, source: hub::Start, agent: &crate::settings::Agent) -> Result<Box<smith::Start>, Invalid> {
    let Ok(charter) = smith_protocol_channel::decode_charter(&source.charter, &agent.limits.charter, &agent.endpoints)
    else {
        return Err(Invalid::Charter);
    };
    let charter = smith::Charter::new(charter, agent.host_limits.charter_bytes).ok_or(Invalid::Charter)?;
    let piece = smith_protocol_channel::transcript_piece(&source.turns).ok_or(Invalid::Transcript)?;
    let transcript = match smith_protocol_channel::decode_transcript(
        &piece,
        u32::try_from(agent.host_limits.transcript_bytes.checked_div(4).ok_or(Invalid::Transcript)?)
            .or(Err(Invalid::Transcript))?,
        agent.limits.transcript.found_hits,
        &agent.limits.transcript,
        &agent.endpoints,
    ) {
        Ok(Some(value)) => {
            Some(smith::Transcript::new(value, agent.host_limits.transcript_bytes).ok_or(Invalid::Transcript)?)
        }
        Ok(None) => None,
        Err(_) => return Err(Invalid::Transcript),
    };
    let mut answered = List::with_capacity(u32::try_from(source.answered.len()).or(Err(Invalid::CallName))?);
    for call in source.answered {
        let mut reader = Reader::new(&call.name);
        let name = match smith_channel::CallName::decode(&agent.limits.bodies, &mut reader) {
            Ok(value) if reader.is_empty() => smith::CallName {
                activation: value.activation(),
                completion: value.completion(),
                position: value.position(),
            },
            Ok(_) | Err(_) => return Err(Invalid::CallName),
        };
        let reply = match call.answer {
            hub::SettledAnswer::Host { error, body } => smith::SavedReply::Host { error, body },
            hub::SettledAnswer::Delivery { outcome, evidence } => {
                smith::SavedReply::Delivery(Box::new(self::evidence(outcome, &evidence)?))
            }
        };
        answered.push(smith::AnsweredCall { name, tool: call.tool, reply }).expect("answer count");
    }
    let mut messages = List::with_capacity(u32::try_from(source.messages.len()).or(Err(Invalid::Charter))?);
    for message in source.messages {
        messages
            .push(smith::Message {
                name: message.name,
                label: message.sender,
                text: message.words,
                finish_requested: message.finish_requested,
            })
            .expect("inbox count");
    }
    let mut grants = List::with_capacity(u32::try_from(source.grants.len()).or(Err(Invalid::Charter))?);
    for grant in source.grants {
        if let Some(grant) = smith_grant(grant) {
            grants.push(grant).expect("grant count");
        }
    }
    Ok(Box::new(smith::Start {
        identity: Some(smith_domain_run::Identity::new(source.identity)),
        logical_run: run,
        activation: source.activation,
        workspace: source.workspace,
        charter,
        transcript,
        answered: answered.into_boxed(),
        messages: messages.into_boxed(),
        directories: Box::default(),
        grants: grants.into_boxed(),
        live: if source.live == hub::LiveAsk::NONE {
            None
        } else {
            Some(smith::channel::LiveAsk {
                text: source.live.text,
                reasoning: source.live.reasoning,
                input: source.live.input,
            })
        },
    }))
}

#[must_use]
pub fn process(event: smith_host_protocol::ProcessEvent) -> Option<temper_worker_domain::Event> {
    use smith_host_protocol::{OpenEvent, ProcessEvent};
    let event = match event {
        ProcessEvent::Spawned { agent, process } => smith::process::Event::Spawned { owner: agent, process },
        ProcessEvent::Unspawned { agent, detail } => smith::process::Event::Unspawned { owner: agent, detail },
        ProcessEvent::Exited { agent, exit } => smith::process::Event::Exited { owner: agent, exit: exited(exit) },
        ProcessEvent::Reaped { agent, detail } => smith::process::Event::Reaped { owner: agent, detail },
        ProcessEvent::Channel { agent, event } => match *event {
            OpenEvent::Opened { .. } | OpenEvent::WriteFailed => return None,
            OpenEvent::Live { turn, retry, kind, offset, bytes, call } => {
                let kind = match kind {
                    smith_channel::LiveKind::Text => {
                        if call.is_some() {
                            return None;
                        }
                        hub::LiveKind::Text
                    }
                    smith_channel::LiveKind::Reasoning => {
                        if call.is_some() {
                            return None;
                        }
                        hub::LiveKind::Reasoning
                    }
                    smith_channel::LiveKind::Input => {
                        let call = call?.into_parts();
                        hub::LiveKind::Input { call: call.id, tool: call.tool }
                    }
                };
                return Some(temper_worker_domain::Event::Live {
                    agent,
                    piece: hub::Piece { turn, retry, kind, offset, bytes },
                });
            }
            OpenEvent::Hangup { .. } => smith::process::Event::Hangup { owner: agent },
            OpenEvent::Sent { .. } => smith::process::Event::Sent { owner: agent },
            OpenEvent::Unsent { .. } => smith::process::Event::Unsent { owner: agent },
            event @ (OpenEvent::Answer { .. }
            | OpenEvent::Admitted
            | OpenEvent::MessageRefused { .. }
            | OpenEvent::Waiting { .. }
            | OpenEvent::Long { .. }
            | OpenEvent::LongDone
            | OpenEvent::Turn { .. }
            | OpenEvent::Fact { .. }
            | OpenEvent::Unauthorized { .. }
            | OpenEvent::Limited { .. }
            | OpenEvent::Refused { .. }
            | OpenEvent::Call { .. }
            | OpenEvent::Withdraw { .. }) => smith::process::Event::Received { owner: agent, message: channel(event)? },
        },
    };
    Some(temper_worker_domain::Event::Process(Box::new(event)))
}

/// Decode a resumed delivery's evidence and check its committed outcome.
pub fn evidence(outcome: hub::DeliveryOutcome, bytes: &[u8]) -> Result<smith::Delivery, Invalid> {
    let value = decoded_delivery(bytes).ok_or(Invalid::Evidence)?;
    let recorded = match &value {
        smith::Delivery::Delivered(_) => hub::DeliveryOutcome::Delivered,
        smith::Delivery::Nothing => hub::DeliveryOutcome::Nothing,
        smith::Delivery::Stale => hub::DeliveryOutcome::Stale,
        smith::Delivery::Refused(_) => hub::DeliveryOutcome::Refused,
        smith::Delivery::Failed(_) => hub::DeliveryOutcome::Failed,
    };
    if recorded == outcome { Ok(value) } else { Err(Invalid::Evidence) }
}

fn decoded_delivery(bytes: &[u8]) -> Option<smith::Delivery> {
    let mut reader = Reader::new(bytes);
    let record = smith_channel::DeliveryReply::decode(&smith_channel::CEILINGS, &mut reader).ok()?;
    if !reader.is_empty() {
        return None;
    }
    Some(match record.into_parts().value {
        smith_channel::Delivery::Delivered(delivered) => {
            let rows = delivered.into_parts().receipts;
            let mut receipts = List::with_capacity(rows.len());
            for row in rows.into_boxed() {
                let parts = row.into_parts();
                receipts.push(smith::Receipt::new(parts.directory, parts.text)?).ok()?;
            }
            smith::Delivery::Delivered(smith::Delivered::new(receipts.into_boxed())?)
        }
        smith_channel::Delivery::Nothing => smith::Delivery::Nothing,
        smith_channel::Delivery::Stale => smith::Delivery::Stale,
        smith_channel::Delivery::Refused(refusal) => {
            let parts = refusal.into_parts();
            let marker = match parts.marker {
                Some(marker) => {
                    let fields = marker.into_parts();
                    Some(smith::Marker::new(fields.directory, fields.path)?)
                }
                None => None,
            };
            smith::Delivery::Refused(smith::DeliveryRefusal::new(marker, parts.explanation)?)
        }
        smith_channel::Delivery::Failed(failure) => {
            let parts = failure.into_parts();
            smith::Delivery::Failed(smith::DeliveryFailure {
                directory: parts.directory,
                reason: delivery_reason(parts.reason),
                diagnostic: smith::Diagnostic::new(&parts.diagnostic, parts.dropped),
            })
        }
    })
}

fn delivery_reason(reason: smith_channel::DeliveryReason) -> smith::DeliveryReason {
    match reason {
        smith_channel::DeliveryReason::Unreachable => smith::DeliveryReason::Unreachable,
        smith_channel::DeliveryReason::RefusedByTarget => smith::DeliveryReason::RefusedByTarget,
        smith_channel::DeliveryReason::TimedOut => smith::DeliveryReason::TimedOut,
        smith_channel::DeliveryReason::Broken => smith::DeliveryReason::Broken,
        smith_channel::DeliveryReason::TooLarge => smith::DeliveryReason::TooLarge,
        smith_channel::DeliveryReason::Missing => smith::DeliveryReason::Missing,
        smith_channel::DeliveryReason::Busy => smith::DeliveryReason::Busy,
        smith_channel::DeliveryReason::Unavailable => smith::DeliveryReason::Unavailable,
        smith_channel::DeliveryReason::Cancelled => smith::DeliveryReason::Cancelled,
        smith_channel::DeliveryReason::Unknown => smith::DeliveryReason::Unknown,
    }
}
fn exited(exit: Option<skein_io::kernel::Exit>) -> Option<smith::Exit> {
    match exit {
        Some(skein_io::kernel::Exit::Code(code)) => Some(smith::Exit::Code(code)),
        Some(skein_io::kernel::Exit::Signal(signal)) => Some(smith::Exit::Signal(signal)),
        None => None,
    }
}
fn channel(event: smith_host_protocol::OpenEvent) -> Option<smith::Up> {
    use smith_host_protocol::OpenEvent;
    Some(match event {
        OpenEvent::MessageRefused { name, reason } => smith::Up::MessageRefused { name, reason },
        OpenEvent::Answer { answer, .. } => smith::Up::Answer { answer },
        OpenEvent::Admitted => smith::Up::Admitted,
        OpenEvent::Waiting { read } => smith::Up::Waiting { read },
        OpenEvent::Long { operation, span } => smith::Up::Long { operation, span },
        OpenEvent::LongDone => smith::Up::LongDone,
        OpenEvent::Turn { turn } => smith::Up::Turn { turn },
        OpenEvent::Fact { body } => smith::Up::Fact { body },
        OpenEvent::Unauthorized { account, generation } => smith::Up::Unauthorized { account, generation },
        OpenEvent::Limited { account, reset } => smith::Up::Limited { account, reset },
        OpenEvent::Refused { account, code, reason } => smith::Up::Refused { account, code, reason },
        OpenEvent::Call { call, name, deadline, ask } => smith::Up::Call { call, name, deadline, ask },
        OpenEvent::Withdraw { call } => smith::Up::Withdraw { call },
        OpenEvent::Live { .. }
        | OpenEvent::Opened { .. }
        | OpenEvent::WriteFailed
        | OpenEvent::Hangup { .. }
        | OpenEvent::Sent { .. }
        | OpenEvent::Unsent { .. } => return None,
    })
}

/// Workspace items named by the assignment, decoded before entering the root.
pub fn items(
    assignment: &hub::Assignment,
    parts: &[jig_protocol_worker_wire::Part],
    limits: &temper_protocol_worker_wire::v1::Limits,
) -> Result<crate::translate::Assigned, temper_worker_domain::wire::Undecodable> {
    let Some(workspace) = &assignment.assignment.workspace else { return Ok(crate::translate::workspace(None)) };
    let bytes = part(workspace.items, parts).ok_or(temper_worker_domain::wire::Undecodable)?;
    let mut reader = Reader::new(bytes);
    let items = temper_protocol_worker_wire::v1::Items::decode(limits, &mut reader)
        .or(Err(temper_worker_domain::wire::Undecodable))?;
    if !reader.is_empty() {
        return Err(temper_worker_domain::wire::Undecodable);
    }
    Ok(crate::translate::workspace(Some(&items)))
}
fn part(token: Token, parts: &[jig_protocol_worker_wire::Part]) -> Option<&[u8]> {
    for part in parts {
        if part.token == token {
            return Some(&part.bytes);
        }
    }
    None
}

/// An application's granted value, decoded and checked once before being kept.
#[must_use]
pub fn identity(
    value: &skein_lib::Secret,
    limits: &temper_protocol_worker_wire::v1::Limits,
) -> Option<crate::identities::Value> {
    let mut reader = Reader::new(value.expose());
    let identity = temper_protocol_worker_wire::v1::Identity::decode(limits, &mut reader).ok()?;
    if !reader.is_empty() {
        return None;
    }
    crate::translate::value(&identity)
}

/// Parts only under tokens an answer names; refusals have no application parts.
#[must_use]
pub fn answer_parts(
    answer: &hub::Answer,
    work: &temper_worker_domain::wire::Work,
    preparation: Option<&temper_worker_domain::wire::Preparation>,
) -> Box<[jig_protocol_worker_wire::Part]> {
    use jig_protocol_worker_wire::Part;
    use temper_protocol_worker_wire as payload;
    let (names, resource) = match &answer.ending {
        hub::Ending::Refused(_) => return Box::new([]),
        hub::Ending::Ended { work, .. } | hub::Ending::Parked { work } => (work, None),
        hub::Ending::Failed { failure, work, .. } => {
            let resource = match failure {
                hub::Failure::Unprepared(hub::Preparation::Permanent { resource }) => *resource,
                hub::Failure::Unprepared(hub::Preparation::Transient)
                | hub::Failure::Run(_)
                | hub::Failure::Agent(_)
                | hub::Failure::Cancelled(_) => None,
            };
            (work, resource)
        }
    };
    let mut parts = List::with_capacity(3);
    let (left, saved) = crate::translate::work(work);
    if let Some(token) = names.left {
        parts.push(Part { token, bytes: payload::left(&left) }).expect("three parts");
    }
    if let Some(token) = names.saved
        && let Some(saved) = saved
    {
        parts.push(Part { token, bytes: payload::saved(&saved) }).expect("three parts");
    }
    if let Some(token) = resource
        && let Some(preparation) = preparation
        && let Some(preparation) = crate::translate::preparation(preparation)
    {
        parts.push(Part { token, bytes: payload::preparation(&preparation) }).expect("three parts");
    }
    parts.into_boxed()
}

fn smith_grant(value: hub::Grant) -> Option<smith::Grant> {
    let form = match value.form {
        hub::Form::Credential(hub::Scheme::Bearer) => smith::Form::Credential(smith::Scheme::Bearer),
        hub::Form::Credential(hub::Scheme::Key) => smith::Form::Credential(smith::Scheme::Key),
        hub::Form::Program(name) => {
            smith::Form::Program(smith::SourceName::new(name.as_bytes()).expect("same source name grammar"))
        }
        hub::Form::Application => return None,
    };
    Some(smith::Grant { account: value.account, generation: value.generation, valid: value.valid, form })
}
