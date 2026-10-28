//! The protocol layer's routing as the worlds play it (jig's
//! `domain/hosts.md`, 6.8): smith's payloads decoded and encoded by hand-out
//! with smith's codecs, keeping no state, as jig's worker worlds answer them
//! (jig's `testing/jig-test-service`, `worker_route.rs`). And the typed values
//! a scripted agent says, with the charter and transcript an engine gives.

use skein_lib::{Duration, List, Reader, Token, Writer};
use smith_domain_session::{llm, record};
use smith_protocol_channel::{Endpoint, Endpoints};
use temper_worker_domain::{Encodable, Encoded, Invalid, agent, host};

/// smith's agent's one endpoint: what its charter's LLM and its turns name.
#[must_use]
pub fn endpoints() -> Endpoints {
    let mut entries = List::with_capacity(1);
    entries.push(Endpoint { name: Box::new([]), number: 0, dialect: crate::DIALECT }).expect("one endpoint");
    Endpoints::new(entries)
}

/// A finite charter as the engine writes it, with a budget, a report
/// contract and a model.
#[must_use]
pub fn charter() -> Box<[u8]> {
    use smith_charter::{
        Budget, BudgetParts, CEILINGS, Charter, CharterParts, Contract, ContractParts, Families, FamiliesParts, Llm,
        LlmParts, Mode, Prices, PricesParts, Reserve, ReserveParts, TextRule, TextRuleParts, Tools, ToolsParts,
    };
    let reserve = Reserve::new(&CEILINGS, ReserveParts { turns: 0, time: Duration::ZERO }).expect("reserve");
    let budget = Budget::new(&CEILINGS, BudgetParts { turns: 1, spend: 2, time: Duration::from_secs(60), reserve })
        .expect("budget");
    let report = TextRule::new(&CEILINGS, TextRuleParts { max: 128, fields: List::with_capacity(0) }).expect("report");
    let contract = Contract::new(
        &CEILINGS,
        ContractParts { report: Some(report), verdicts: List::with_capacity(0), change: None, failure: None },
    )
    .expect("contract");
    let prices =
        Prices::new(&CEILINGS, PricesParts { input: 1, cached: 1, cache_write: None, output: 1, unit: 1_000_000 })
            .expect("prices");
    let main = Llm::new(
        &CEILINGS,
        LlmParts {
            endpoint: Box::new([]),
            account: None,
            model: Box::from(&b"fake"[..]),
            effort: None,
            window: 1_048_576,
            output: 512,
            prices,
        },
    )
    .expect("model");
    let families = Families::new(&CEILINGS, FamiliesParts { skein_bools: [false; 4] }).expect("families");
    let tools =
        Tools::new(&CEILINGS, ToolsParts { families, wait: false, deliver: None, host: List::with_capacity(0) })
            .expect("tools");
    let parts = CharterParts {
        instructions: Box::new([]),
        brief: List::with_capacity(0),
        tools,
        contract,
        conventions: None,
        budget,
        main,
        models: List::with_capacity(0),
        waiting: Duration::ZERO,
        resume: false,
        mode: Mode::Finite,
    };
    let record = Charter::new(&CEILINGS, parts).expect("charter");
    let mut writer = Writer::new(usize::try_from(record.measure()).expect("charter size"));
    record.encode(&mut writer).expect("measured charter");
    writer.finish()
}

/// A turn of `length` bytes of text, as smith's session settles it: the
/// `first` of a conversation opens its window.
#[must_use]
pub fn session_turn(sequence: u32, spent: u64, length: usize, first: bool) -> record::Turn {
    let kind = if first {
        record::Kind::First(Box::new(record::Opener {
            affinity: llm::Affinity { key: [5; 16], thread: 0 },
            account: Some(0),
            backend: record::Backend::Direct,
        }))
    } else {
        record::Kind::Ordinary
    };
    let text = vec![u8::try_from(sequence % 200).expect("a byte") + b' '; length].into_boxed_slice();
    record::Turn {
        model: Box::from(*b"model"),
        effort: None,
        version: record::VERSION,
        kind,
        endpoint: Some(llm::Endpoint(0)),
        dialect: crate::DIALECT,
        sequence,
        usage: llm::Usage::ZERO,
        jobs: 0,
        spent,
        messages: Box::new([llm::Message {
            role: llm::Role::Assistant,
            content: Box::new([llm::Block::Text { text, replay: None }]),
        }]),
    }
}

/// The turn `number` smith's agent says, of a conversation resumed after the
/// engine's transcript.
#[must_use]
pub fn turn(number: u32, spent: u64, length: usize) -> agent::Turn {
    let body = session_turn(number.saturating_add(1), spent, length, false);
    agent::Turn {
        opens_window: false,
        number,
        spent,
        read: None,
        body: agent::TurnValue::new(body, u64::MAX).expect("an owned turn"),
    }
}

/// The bytes of the session's turn, as the engine keeps it in a transcript.
#[must_use]
pub fn transcript_turn(sequence: u32, length: usize, first: bool) -> Box<[u8]> {
    let turn = session_turn(sequence, 0, length, first);
    smith_protocol_channel::encode_turn(&turn, &smith_transcript::CEILINGS, &endpoints()).expect("an encodable turn")
}

/// The hub's start, decoded into smith's for the run `run`, as the routing
/// does: the charter and transcript with the agent's endpoints, the calls
/// answered after the transcript by their names, the start's inbox, its
/// LLM accounts' grants, and the live content asked of it (none when nothing
/// is). The root adds the workspace's directories.
///
/// # Errors
///
/// What of the start did not decode.
pub fn decode(run: Token, source: host::Start, limits: &agent::Limits) -> Result<Box<agent::Start>, Invalid> {
    let endpoints = endpoints();
    let charter = smith_protocol_channel::decode_charter(&source.charter, &smith_charter::CEILINGS, &endpoints)
        .map_err(|_| Invalid::Charter)?;
    let charter = agent::Charter::new(charter, limits.charter_bytes).ok_or(Invalid::Charter)?;
    let piece = smith_protocol_channel::transcript_piece(&source.turns).ok_or(Invalid::Transcript)?;
    let found = smith_transcript::CEILINGS.found_hits;
    let decoded = smith_protocol_channel::decode_transcript(&piece, 64, found, &smith_transcript::CEILINGS, &endpoints)
        .map_err(|_| Invalid::Transcript)?;
    let transcript = match decoded {
        Some(value) => Some(agent::Transcript::new(value, limits.transcript_bytes).ok_or(Invalid::Transcript)?),
        None => None,
    };
    let mut answered = Vec::with_capacity(source.answered.len());
    for call in source.answered {
        let reply = match call.answer {
            host::SettledAnswer::Host { error, body } => agent::SavedReply::Host { error, body },
            host::SettledAnswer::Delivery { outcome, evidence } => {
                agent::SavedReply::Delivery(Box::new(crate::evidence::decode(outcome, &evidence)?))
            }
        };
        answered.push(agent::AnsweredCall { name: call_name(&call.name)?, tool: call.tool, reply });
    }
    let mut messages = Vec::with_capacity(source.messages.len());
    for message in source.messages {
        messages.push(agent::Message {
            name: message.name,
            label: message.sender,
            text: message.words,
            finish_requested: message.finish_requested,
        });
    }
    let mut grants = Vec::with_capacity(source.grants.len());
    for grant in source.grants {
        if let Some(grant) = smith_grant(grant) {
            grants.push(grant);
        }
    }
    Ok(Box::new(agent::Start {
        identity: Some(smith_domain_run::Identity::new(source.identity)),
        logical_run: run,
        activation: source.activation,
        messages: messages.into_boxed_slice(),
        workspace: source.workspace,
        charter,
        transcript,
        answered: answered.into_boxed_slice(),
        directories: Box::new([]),
        grants: grants.into_boxed_slice(),
        live: if source.live == host::LiveAsk::NONE {
            None
        } else {
            Some(agent::channel::LiveAsk {
                text: source.live.text,
                reasoning: source.live.reasoning,
                input: source.live.input,
            })
        },
    }))
}

fn call_name(bytes: &[u8]) -> Result<agent::CallName, Invalid> {
    let mut reader = Reader::new(bytes);
    match smith_channel::CallName::decode(&smith_channel::CEILINGS, &mut reader) {
        Ok(name) if reader.is_empty() => Ok(agent::CallName {
            activation: name.activation(),
            completion: name.completion(),
            position: name.position(),
        }),
        Ok(_) | Err(_) => Err(Invalid::CallName),
    }
}

/// One of smith's payloads, encoded for the hub: a turn as the engine keeps
/// it, a call's name as smith's channel writes it, an accepted result as its
/// charter's codec does; a refusal's words; nothing for a parked or failed
/// run, which the root keeps typed.
#[must_use]
pub fn encode(value: Encodable) -> Encoded {
    match value {
        Encodable::Turn(turn) => {
            let encoded =
                smith_protocol_channel::encode_turn(turn.body.value(), &smith_transcript::CEILINGS, &endpoints());
            match encoded {
                Ok(body) => Encoded::Turn(host::Turn {
                    opens_window: turn.opens_window,
                    refused: Box::new([]),
                    turn: turn.number,
                    spent: turn.spent,
                    read: turn.read,
                    body,
                }),
                Err(_) => Encoded::Undecodable,
            }
        }
        Encodable::CallName(name) => {
            let parts = smith_channel::CallNameParts {
                activation: name.activation,
                completion: name.completion,
                position: name.position,
            };
            let record = smith_channel::CallName::new(&smith_channel::CEILINGS, parts).expect("a call name");
            let mut writer = Writer::new(usize::try_from(record.measure()).expect("a call name's size"));
            record.encode(&mut writer).expect("a measured call name");
            Encoded::CallName(writer.finish())
        }
        Encodable::Result(agent::RunResult::Accepted { outcome }) => {
            match smith_protocol_channel::encode_result(outcome.value(), &smith_charter::CEILINGS) {
                Ok(body) => Encoded::Result(body),
                Err(_) => Encoded::Undecodable,
            }
        }
        Encodable::Result(agent::RunResult::Refused { .. }) => Encoded::Result(Box::from(&b"agent refused"[..])),
        Encodable::Result(agent::RunResult::Parked | agent::RunResult::Failed { .. }) => Encoded::Result(Box::new([])),
    }
}

/// One of smith's channel facts as jig's activity, if it decodes.
#[must_use]
pub fn activity(body: &[u8]) -> Option<host::Activity> {
    let mut reader = Reader::new(body);
    let fact = smith_channel::Fact::decode(&smith_channel::CEILINGS, &mut reader).ok()?;
    if !reader.is_empty() {
        return None;
    }
    let kind = match fact.kind() {
        smith_channel::FactKind::Admitted => host::CountKind::Admitted,
        smith_channel::FactKind::Ended => host::CountKind::Ended,
        smith_channel::FactKind::LlmStarted => host::CountKind::LlmStarted,
        smith_channel::FactKind::LlmRetried => host::CountKind::LlmRetried,
        smith_channel::FactKind::LlmFinished => host::CountKind::LlmFinished,
        smith_channel::FactKind::ToolStarted => host::CountKind::ToolStarted,
        smith_channel::FactKind::ToolFinished => host::CountKind::ToolFinished,
        smith_channel::FactKind::CheckStarted => host::CountKind::CheckStarted,
        smith_channel::FactKind::CheckFinished => host::CountKind::CheckFinished,
        smith_channel::FactKind::TextArrived => host::CountKind::TextArrived,
    };
    Some(host::Activity::Count { kind, elapsed: fact.elapsed(), count: fact.count() })
}

/// A delivery the agent asks for: its commit's title and body.
#[must_use]
pub fn delivery(title: &[u8], body: &[u8]) -> agent::Ask {
    let fields = smith_domain_run::outcome::Change {
        fields: Box::new([
            smith_domain_run::outcome::Field { name: Box::from(&b"title"[..]), value: Box::from(title) },
            smith_domain_run::outcome::Field { name: Box::from(&b"body"[..]), value: Box::from(body) },
        ]),
    };
    agent::Ask::Deliver { fields: agent::Fields::new(fields, u64::MAX).expect("bounded fields") }
}

/// A run's accepted report, `text`.
#[must_use]
pub fn report(text: &[u8]) -> agent::RunResult {
    let report = smith_domain_run::outcome::Report { text: Box::from(text), fields: Box::new([]) };
    let declared = smith_domain_run::outcome::Declared::Report(report);
    agent::RunResult::Accepted { outcome: agent::Declared::new(declared, u64::MAX).expect("a bounded result") }
}

fn smith_grant(value: host::Grant) -> Option<agent::Grant> {
    let form = match value.form {
        host::Form::Credential(host::Scheme::Bearer) => agent::Form::Credential(agent::Scheme::Bearer),
        host::Form::Credential(host::Scheme::Key) => agent::Form::Credential(agent::Scheme::Key),
        host::Form::Program(name) => {
            agent::Form::Program(agent::SourceName::new(name.as_bytes()).expect("same source name grammar"))
        }
        host::Form::Application => return None,
    };
    Some(agent::Grant { account: value.account, generation: value.generation, valid: value.valid, form })
}
