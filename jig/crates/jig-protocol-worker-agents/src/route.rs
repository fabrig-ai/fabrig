//! Smith payloads are translated in the worker layer (jig/docs/design/protocol/channel/README.md, 5.1 and 5.4).
use alloc::boxed::Box;
use jig_domain_host as hub;
use jig_domain_link::{CountKind, Form, Grant, Scheme};
use skein_lib::{List, Reader, Token, Writer};
use smith_host_domain as smith;
use smith_host_protocol::Limits;
use smith_protocol_channel::Endpoints;

/// What a settled delivery's evidence is (jig/docs/design/protocol/channel/README.md, 5.2).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Evidence {
    /// The bytes are a receipt's or a refusal's words, as they stand.
    Text,
    /// The bytes are smith's `DeliveryReply`, which must decode and agree with the outcome.
    Reply,
}

/// An agent process's event, in the worker root's terms.
#[derive(Debug)]
pub enum Processed {
    Live { agent: Token, piece: hub::Piece },
    Process(Box<smith::process::Event>),
}

#[must_use]
pub fn encode_turn(turn: smith::Turn, limits: &Limits, endpoints: &Endpoints) -> Option<hub::Turn> {
    let body = smith_protocol_channel::encode_turn(turn.body.value(), &limits.transcript, endpoints).ok()?;
    Some(hub::Turn {
        opens_window: turn.opens_window,
        refused: Box::default(),
        turn: turn.number,
        spent: turn.spent,
        read: turn.read,
        body,
    })
}

#[must_use]
pub fn encode_call_name(name: smith::CallName, limits: &Limits) -> Option<Box<[u8]>> {
    let record = smith_channel::CallName::new(
        &limits.bodies,
        smith_channel::CallNameParts {
            activation: name.activation,
            completion: name.completion,
            position: name.position,
        },
    )
    .ok()?;
    let mut writer = Writer::new(skein_lib::length::usize_of(record.measure()));
    record.encode(&mut writer).ok()?;
    Some(writer.finish())
}

#[must_use]
pub fn encode_result(result: smith::RunResult, limits: &Limits) -> Option<Box<[u8]>> {
    match result {
        smith::RunResult::Accepted { outcome } => {
            smith_protocol_channel::encode_result(outcome.value(), &limits.charter).ok()
        }
        smith::RunResult::Refused { refusal } => Some(Box::from(refused(refusal))),
        // The root retains these typed dispositions beside the hand-out.
        smith::RunResult::Parked | smith::RunResult::Failed { .. } => Some(Box::default()),
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
            smith::RunInvalid::Llm => b"agent llm invalid",
            smith::RunInvalid::Conversation => b"agent conversation invalid",
            smith::RunInvalid::Endpoint => b"agent endpoint invalid",
            smith::RunInvalid::Effort => b"agent effort invalid",
        },
    }
}

#[must_use]
pub fn activity(body: &[u8], limits: &Limits) -> Option<hub::AgentActivity> {
    let mut reader = Reader::new(body);
    let Ok(fact) = smith_channel::Fact::decode(&limits.bodies, &mut reader) else { return None };
    if !reader.is_empty() {
        return None;
    }
    let kind = match fact.kind() {
        smith_channel::FactKind::Admitted => CountKind::Admitted,
        smith_channel::FactKind::Ended => CountKind::Ended,
        smith_channel::FactKind::LlmStarted => CountKind::LlmStarted,
        smith_channel::FactKind::LlmRetried => CountKind::LlmRetried,
        smith_channel::FactKind::LlmFinished => CountKind::LlmFinished,
        smith_channel::FactKind::ToolStarted => CountKind::ToolStarted,
        smith_channel::FactKind::ToolFinished => CountKind::ToolFinished,
        smith_channel::FactKind::CheckStarted => CountKind::CheckStarted,
        smith_channel::FactKind::CheckFinished => CountKind::CheckFinished,
        smith_channel::FactKind::TextArrived => CountKind::TextArrived,
    };
    Some(hub::AgentActivity::Count { kind, elapsed: fact.elapsed(), count: fact.count() })
}

pub fn decode(
    run: Token,
    source: hub::Start,
    limits: &Limits,
    host_limits: &smith::Limits,
    endpoints: &Endpoints,
    evidence: Evidence,
) -> Result<Box<smith::Start>, hub::Undecoded> {
    version(&source.charter, smith_charter::wire::FAMILY, smith_charter::wire::VERSION)?;
    let Ok(charter) = smith_protocol_channel::decode_charter(&source.charter, &limits.charter, endpoints) else {
        return Err(hub::Undecoded::Charter);
    };
    let charter = smith::Charter::new(charter, host_limits.charter_bytes).ok_or(hub::Undecoded::Charter)?;
    for turn in &source.turns {
        version(turn, smith_transcript::wire::FAMILY, smith_transcript::wire::VERSION)?;
    }
    let piece = smith_protocol_channel::transcript_piece(&source.turns).ok_or(hub::Undecoded::Transcript)?;
    let transcript = match smith_protocol_channel::decode_transcript(
        &piece,
        u32::try_from(host_limits.transcript_bytes.checked_div(4).ok_or(hub::Undecoded::Transcript)?)
            .or(Err(hub::Undecoded::Transcript))?,
        limits.transcript.found_hits,
        &limits.transcript,
        endpoints,
    ) {
        Ok(Some(value)) => {
            Some(smith::Transcript::new(value, host_limits.transcript_bytes).ok_or(hub::Undecoded::Transcript)?)
        }
        Ok(None) => None,
        Err(_) => return Err(hub::Undecoded::Transcript),
    };
    let mut answered = List::with_capacity(u32::try_from(source.answered.len()).or(Err(hub::Undecoded::CallName))?);
    for call in source.answered {
        let mut reader = Reader::new(&call.name);
        let name = match smith_channel::CallName::decode(&limits.bodies, &mut reader) {
            Ok(value) if reader.is_empty() => smith::CallName {
                activation: value.activation(),
                completion: value.completion(),
                position: value.position(),
            },
            Ok(_) | Err(_) => return Err(hub::Undecoded::CallName),
        };
        let reply = match call.answer {
            hub::SettledAnswer::Host { error, body } => smith::SavedReply::Host { error, body },
            hub::SettledAnswer::Delivery { outcome, evidence: bytes } => {
                let delivery = match evidence {
                    Evidence::Reply => self::evidence(outcome, &bytes)?,
                    Evidence::Text => delivery(outcome, bytes)?,
                };
                smith::SavedReply::Delivery(Box::new(delivery))
            }
        };
        answered.push(smith::AnsweredCall { name, tool: call.tool, reply }).expect("answer count");
    }
    let mut messages = List::with_capacity(u32::try_from(source.messages.len()).or(Err(hub::Undecoded::Charter))?);
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
    let mut grants = List::with_capacity(u32::try_from(source.grants.len()).or(Err(hub::Undecoded::Charter))?);
    for grant in source.grants {
        if let Some(grant) = self::grant(grant) {
            grants.push(grant).expect("grant count");
        }
    }
    Ok(Box::new(smith::Start {
        identity: Some(smith_domain_run::Identity::new(source.identity)),
        live: if source.live == hub::LiveAsk::NONE {
            None
        } else {
            Some(smith::channel::LiveAsk {
                text: source.live.text,
                reasoning: source.live.reasoning,
                input: source.live.input,
            })
        },
        logical_run: run,
        activation: source.activation,
        workspace: source.workspace,
        charter,
        transcript,
        answered: answered.into_boxed(),
        messages: messages.into_boxed(),
        directories: Box::default(),
        grants: grants.into_boxed(),
    }))
}

fn version(bytes: &[u8], wire: &str, current: u16) -> Result<(), hub::Undecoded> {
    let mut reader = Reader::new(bytes);
    if let Some(version) = reader.u16()
        && version != current
    {
        return Err(hub::Undecoded::Version(hub::Mismatch {
            wire: Box::from(wire.as_bytes()),
            version,
            lowest: current,
            highest: current,
        }));
    }
    Ok(())
}

fn delivery(outcome: hub::DeliveryOutcome, evidence: Box<[u8]>) -> Result<smith::Delivery, hub::Undecoded> {
    Ok(match outcome {
        hub::DeliveryOutcome::Delivered => {
            let receipt = smith::Receipt::new(0, evidence).ok_or(hub::Undecoded::CallName)?;
            smith::Delivery::Delivered(smith::Delivered::new(Box::new([receipt])).ok_or(hub::Undecoded::CallName)?)
        }
        hub::DeliveryOutcome::Nothing => smith::Delivery::Nothing,
        hub::DeliveryOutcome::Stale => smith::Delivery::Stale,
        hub::DeliveryOutcome::Refused => {
            smith::Delivery::Refused(smith::DeliveryRefusal::new(None, evidence).ok_or(hub::Undecoded::CallName)?)
        }
        hub::DeliveryOutcome::Failed => smith::Delivery::Failed(smith::DeliveryFailure {
            directory: 0,
            reason: smith::DeliveryReason::Unknown,
            diagnostic: smith::Diagnostic::new(&evidence, 0),
        }),
    })
}

#[must_use]
pub fn process(event: smith_host_protocol::ProcessEvent) -> Option<Processed> {
    use smith_host_protocol::{ProcessEvent, Received};
    let event = match event {
        ProcessEvent::Spawned { agent, process } => smith::process::Event::Spawned { owner: agent, process },
        ProcessEvent::Unspawned { agent, detail } => smith::process::Event::Unspawned { owner: agent, detail },
        ProcessEvent::Exited { agent, exit } => smith::process::Event::Exited { owner: agent, exit: exited(exit) },
        ProcessEvent::Reaped { agent, detail } => smith::process::Event::Reaped { owner: agent, detail },
        ProcessEvent::Channel { agent, event } => match *event {
            Received::Opened { .. } | Received::WriteFailed => return None,
            Received::Live { turn, retry, kind, offset, bytes, call } => {
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
                return Some(Processed::Live { agent, piece: hub::Piece { turn, retry, kind, offset, bytes } });
            }
            Received::Hangup { .. } => smith::process::Event::Hangup { owner: agent },
            Received::Sent { .. } => smith::process::Event::Sent { owner: agent },
            Received::Unsent { .. } => smith::process::Event::Unsent { owner: agent },
            Received::Answer { answer, .. } => {
                smith::process::Event::Received { owner: agent, message: smith::Up::Answer { answer } }
            }
            Received::Message { message } => smith::process::Event::Received { owner: agent, message },
        },
    };
    Some(Processed::Process(Box::new(event)))
}
fn grant(value: Grant) -> Option<smith::Grant> {
    let form = match value.form {
        Form::Credential(Scheme::Bearer) => smith::Form::Credential(smith::Scheme::Bearer),
        Form::Credential(Scheme::Key) => smith::Form::Credential(smith::Scheme::Key),
        Form::Program(name) => smith::Form::Program(smith::SourceName::new(name.as_bytes())?),
        Form::Application => return None,
    };
    Some(smith::Grant { account: value.account, generation: value.generation, valid: value.valid, form })
}

/// Decode a resumed delivery's evidence and check its committed outcome.
pub fn evidence(outcome: hub::DeliveryOutcome, bytes: &[u8]) -> Result<smith::Delivery, hub::Undecoded> {
    let value = decoded_delivery(bytes).ok_or(hub::Undecoded::Evidence)?;
    let recorded = match &value {
        smith::Delivery::Delivered(_) => hub::DeliveryOutcome::Delivered,
        smith::Delivery::Nothing => hub::DeliveryOutcome::Nothing,
        smith::Delivery::Stale => hub::DeliveryOutcome::Stale,
        smith::Delivery::Refused(_) => hub::DeliveryOutcome::Refused,
        smith::Delivery::Failed(_) => hub::DeliveryOutcome::Failed,
    };
    if recorded == outcome { Ok(value) } else { Err(hub::Undecoded::Evidence) }
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

/// Translate a settled agent list answer without exposing its credential (protocol/channel/records.md, 1).
#[must_use]
pub fn list_outcome(outcome: smith::ListOutcome) -> jig_domain_link::ListOutcome {
    use jig_domain_link::{ListFailure, ListOutcome, ListedModel};
    match outcome {
        smith::ListOutcome::Listed(rows) => {
            let mut listed = List::with_capacity(u32::try_from(rows.len()).expect("bounded agent list"));
            for row in rows {
                listed
                    .push(ListedModel {
                        name: row.name,
                        shown: row.shown,
                        window: row.window,
                        output: row.output,
                        efforts: row.efforts,
                    })
                    .expect("one slot per list row");
            }
            ListOutcome::Listed(listed.into_boxed())
        }
        smith::ListOutcome::Failed(failure) => ListOutcome::Failed(match failure {
            smith::ListFailure::Refused { status } => ListFailure::Refused { status },
            smith::ListFailure::Unreachable => ListFailure::Unreachable,
            smith::ListFailure::Unreadable => ListFailure::Unreadable,
            smith::ListFailure::Deadline => ListFailure::Deadline,
            smith::ListFailure::Agent => ListFailure::Agent,
        }),
    }
}
