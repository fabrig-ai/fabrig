//! Total translations of the worker records (protocol/channel/README.md, sections 3 and 5.4).
//! Jobs use their own records (jig/docs/design/protocol/channel/records.md, 4).
use alloc::boxed::Box;
use jig_domain_host as host;
use jig_domain_link::{
    Bounds, CountKind, EndpointBound, Form, Hello, Hosting, JobBound, JobKind, Login, LoginMethod, LoginState,
    Operation, Phase, Scheme, SignOutResult, SourceFailure, SourceName, valid_program_endpoints,
};
use jig_protocol_worker_wire as wire;
use skein_lib::{List, Reader, Token};
pub(crate) fn hello(record: wire::Hello) -> Option<Hello> {
    let fields = record.into_parts();
    let bounds = fields.bounds.into_parts();
    if !valid_program_endpoints(bounds.program_endpoints.as_slice()) {
        return None;
    }
    let mut endpoints = List::with_capacity(bounds.endpoints.len());
    for model in bounds.endpoints.into_boxed() {
        let model = model.into_parts();
        endpoints
            .push(EndpointBound { endpoint: model.endpoint, window: model.window, output: model.output })
            .expect("one slot per decoded model");
    }
    let mut hosting = List::with_capacity(fields.hosting.len());
    for run in fields.hosting.into_boxed() {
        let run = run.into_parts();
        hosting
            .push(Hosting { run: Token::new(run.task), attempt: Token::new(run.attempt), phase: phase(run.phase) })
            .expect("one slot per decoded run");
    }
    let mut jobs = List::with_capacity(fields.jobs.len());
    for job in fields.jobs.into_boxed() {
        let job = job.into_parts();
        jobs.push(JobBound { kind: JobKind(job.kind), spec: job.spec, result: job.result, deadline: job.deadline })
            .expect("one slot per decoded job kind");
    }
    Some(Hello {
        jobs: jobs.into_boxed(),
        slots: fields.slots,
        stop_bound: fields.stop_bound,
        workstreams: fields.workstreams.into_boxed(),
        hosting: hosting.into_boxed(),
        bounds: Bounds {
            program_endpoints: bounds.program_endpoints.into_boxed(),
            endpoints: endpoints.into_boxed(),
            turns: bounds.turns,
            spend: bounds.spend,
            time: bounds.time,
            waiting: bounds.waiting,
            checks: bounds.checks,
            message_bytes: bounds.message_bytes,
            inbox: bounds.inbox,
            unacknowledged: bounds.unacknowledged,
            reply_bytes: bounds.reply_bytes,
        },
    })
}
fn phase(value: wire::Phase) -> Phase {
    match value {
        wire::Phase::Preparing => Phase::Preparing,
        wire::Phase::Starting => Phase::Starting,
        wire::Phase::Active => Phase::Active,
        wire::Phase::Waiting => Phase::Waiting,
        wire::Phase::Ending => Phase::Ending,
        wire::Phase::Answered => Phase::Answered,
    }
}

#[derive(Debug)]
pub(crate) enum Up {
    Source(host::LinkUp),
    Ask(wire::Ask),
    ListRead(wire::ListRead),
    Turn(wire::Turn),
    Call(wire::Call),
    Answer(wire::Answer),
    JobAnswer(wire::JobAnswer),
    Fact(wire::Fact),
    Unauthorized(wire::Unauthorized),
    Limited(wire::Limited),
    AccountRefused(wire::AccountRefused),
}
pub(crate) fn decode(kind: u16, body: &[u8], limits: &wire::v14::Limits) -> Option<Up> {
    let mut found = None;
    for rule in wire::KINDS {
        if rule.kind == kind {
            found = Some(rule.body);
            break;
        }
    }
    let kind = found?;
    let reader = &mut Reader::new(body);
    match kind {
        wire::Kind::Ask => Some(Up::Ask(wire::Ask::decode(limits, reader).ok()?)),
        wire::Kind::Draining => {
            wire::Draining::decode(limits, reader).ok()?;
            Some(Up::Source(host::LinkUp::Draining))
        }
        wire::Kind::ListRead => Some(Up::ListRead(wire::ListRead::decode(limits, reader).ok()?)),
        wire::Kind::Logins => {
            let record = wire::Logins::decode(limits, reader).ok()?.into_parts();
            let mut rows = List::<Login>::with_capacity(record.logins.len());
            for row in record.logins.into_boxed() {
                let row = row.into_parts();
                let name = SourceName::new(&row.name)?;
                for old in &rows {
                    if old.name == name {
                        return None;
                    }
                }
                rows.push(Login { name, state: login_state(row.state)?, email: row.email }).ok()?;
            }
            Some(Up::Source(host::LinkUp::Logins { logins: rows.into_boxed() }))
        }
        wire::Kind::SignedOut => {
            let record = wire::SignedOut::decode(limits, reader).ok()?.into_parts();
            let name = SourceName::new(&record.name)?;
            let result = match record.result {
                wire::SignOutResult::SignedOut => SignOutResult::SignedOut,
                wire::SignOutResult::Failed(failure) => {
                    let failure = failure.into_parts();
                    SignOutResult::Failed { reason: source_failure(failure.reason), detail: failure.detail }
                }
            };
            Some(Up::Source(host::LinkUp::SignedOut { name, result }))
        }
        wire::Kind::Turn => Some(Up::Turn(wire::Turn::decode(limits, reader).ok()?)),
        wire::Kind::Call => Some(Up::Call(wire::Call::decode(limits, reader).ok()?)),
        wire::Kind::JobAnswer => Some(Up::JobAnswer(wire::JobAnswer::decode(limits, reader).ok()?)),
        wire::Kind::Answer => Some(Up::Answer(wire::Answer::decode(limits, reader).ok()?)),
        wire::Kind::Fact => Some(Up::Fact(wire::Fact::decode(limits, reader).ok()?)),
        wire::Kind::Unauthorized => Some(Up::Unauthorized(wire::Unauthorized::decode(limits, reader).ok()?)),
        wire::Kind::Limited => Some(Up::Limited(wire::Limited::decode(limits, reader).ok()?)),
        wire::Kind::AccountRefused => Some(Up::AccountRefused(wire::AccountRefused::decode(limits, reader).ok()?)),
        wire::Kind::Asked
        | wire::Kind::ReadList
        | wire::Kind::Job
        | wire::Kind::Sources
        | wire::Kind::Check
        | wire::Kind::SignOut
        | wire::Kind::Hello
        | wire::Kind::Assignment
        | wire::Kind::Message
        | wire::Kind::Relayed
        | wire::Kind::TurnKept
        | wire::Kind::TurnBusy
        | wire::Kind::Acknowledge
        | wire::Kind::Wires
        | wire::Kind::Cancel
        | wire::Kind::Grant => None,
    }
}
pub(crate) fn up(record: Up, parts: &mut Parts) -> host::LinkUp {
    match record {
        Up::Ask(record) => {
            let record = record.into_parts();
            host::LinkUp::Ask {
                run: Token::new(record.run),
                attempt: Token::new(record.attempt),
                number: record.number,
                bytes: record.bytes,
            }
        }
        Up::Source(up) => up,
        Up::ListRead(record) => {
            let record = record.into_parts();
            host::LinkUp::ListRead { read: record.read, outcome: list_outcome(record.outcome) }
        }
        Up::Turn(record) => turn(record),
        Up::Call(record) => call(record),
        Up::Answer(record) => answer(record, parts),
        Up::JobAnswer(record) => job_answer(record, parts),
        Up::Fact(record) => fact(record),
        Up::Unauthorized(record) => {
            let fields = record.into_parts();
            host::LinkUp::Unauthorized {
                run: Token::new(fields.task),
                attempt: Token::new(fields.attempt),
                account: fields.account,
                generation: fields.generation,
            }
        }
        Up::Limited(record) => {
            let fields = record.into_parts();
            host::LinkUp::Limited {
                run: Token::new(fields.task),
                attempt: Token::new(fields.attempt),
                account: fields.account,
                reset: fields.reset,
            }
        }
        Up::AccountRefused(record) => {
            let fields = record.into_parts();
            host::LinkUp::AccountRefused {
                run: Token::new(fields.task),
                attempt: Token::new(fields.attempt),
                account: fields.account,
                code: fields.code,
                reason: fields.reason,
            }
        }
    }
}
fn turn(record: wire::Turn) -> host::LinkUp {
    let fields = record.into_parts();
    let refused = refusals(fields.refused);
    host::LinkUp::Turn {
        run: Token::new(fields.task),
        attempt: Token::new(fields.attempt),
        turn: host::Turn {
            opens_window: fields.opens_window,
            refused,
            turn: fields.number,
            spent: fields.spent,
            read: optional_token(fields.read),
            body: fields.body,
        },
    }
}
fn refusals(records: List<wire::Refused>) -> Box<[host::Refused]> {
    let mut refused = List::with_capacity(records.len());
    for record in records.into_boxed() {
        let fields = record.into_parts();
        let reason = match fields.reason {
            wire::MessageRefusal::TooLarge => host::MessageRefusal::TooLarge,
            wire::MessageRefusal::Full => host::MessageRefusal::Full,
            wire::MessageRefusal::NameInUse => host::MessageRefusal::NameInUse,
            wire::MessageRefusal::Ending => host::MessageRefusal::Ending,
        };
        refused.push(host::Refused { name: Token::new(fields.name), reason }).expect("one slot per refusal");
    }
    refused.into_boxed()
}
fn optional_token(value: Option<u64>) -> Option<Token> {
    Some(Token::new(value?))
}
fn fact(record: wire::Fact) -> host::LinkUp {
    let fields = record.into_parts();
    let activity = match fields.value {
        wire::FactValue::Piece(piece) => {
            let piece = piece.into_parts();
            let kind = match piece.kind {
                wire::LiveKind::Text => host::LiveKind::Text,
                wire::LiveKind::Reasoning => host::LiveKind::Reasoning,
                wire::LiveKind::Input(call) => {
                    let call = call.into_parts();
                    host::LiveKind::Input { call: call.id, tool: call.tool }
                }
            };
            return host::LinkUp::Piece {
                run: Token::new(fields.task),
                attempt: Token::new(fields.attempt),
                piece: host::Piece {
                    turn: piece.turn,
                    retry: piece.retry,
                    kind,
                    offset: piece.offset,
                    bytes: piece.bytes,
                },
            };
        }
        wire::FactValue::Phase(value) => host::AgentActivity::Phase(phase(value.into_parts().value)),
        wire::FactValue::Long(value) => {
            let fields = value.into_parts();
            let operation = match fields.operation {
                wire::Operation::Checks => Operation::Checks,
                wire::Operation::Completion => Operation::Completion,
                wire::Operation::Command => Operation::Command,
            };
            host::AgentActivity::Long { operation, span: fields.span }
        }
        wire::FactValue::LongDone => host::AgentActivity::LongDone,
        wire::FactValue::Count(value) => {
            let fields = value.into_parts();
            host::AgentActivity::Count { kind: count_kind(fields.kind), elapsed: fields.elapsed, count: fields.value }
        }
    };
    host::LinkUp::Activity { run: Token::new(fields.task), attempt: Token::new(fields.attempt), activity }
}
fn count_kind(kind: wire::CountKind) -> CountKind {
    match kind {
        wire::CountKind::Admitted => CountKind::Admitted,
        wire::CountKind::Ended => CountKind::Ended,
        wire::CountKind::LlmStarted => CountKind::LlmStarted,
        wire::CountKind::LlmRetried => CountKind::LlmRetried,
        wire::CountKind::LlmFinished => CountKind::LlmFinished,
        wire::CountKind::ToolStarted => CountKind::ToolStarted,
        wire::CountKind::ToolFinished => CountKind::ToolFinished,
        wire::CountKind::CheckStarted => CountKind::CheckStarted,
        wire::CountKind::CheckFinished => CountKind::CheckFinished,
        wire::CountKind::TextArrived => CountKind::TextArrived,
    }
}

#[derive(Debug)]
pub(crate) struct Parts {
    next: u64,
    values: List<wire::Part>,
}
impl Parts {
    pub fn new(next: u64) -> Self {
        Self { next, values: List::with_capacity(3) }
    }
    pub fn optional(&mut self, bytes: Option<Box<[u8]>>) -> Option<Token> {
        let bytes = bytes?;
        let token = Token::new(self.next);
        self.next = self.next.checked_add(1).expect("part tokens never wrap in a process life");
        self.values.push(wire::Part { token, bytes }).expect("at most resource, left and saved work");
        Some(token)
    }
    #[cfg(test)]
    pub fn is_empty(&self) -> bool {
        self.values.is_empty()
    }
    pub fn next(&self) -> u64 {
        self.next
    }
    pub fn finish(self) -> Box<[wire::Part]> {
        self.values.into_boxed()
    }
}
pub(crate) fn answer(record: wire::Answer, parts: &mut Parts) -> host::LinkUp {
    let fields = record.into_parts();
    let ending = ending(fields.ending, parts);
    host::LinkUp::Answer {
        run: Token::new(fields.task),
        attempt: Token::new(fields.attempt),
        answer: host::RunAnswer {
            read: optional_token(fields.read),
            refused: refusals(fields.refused),
            turns: fields.turns,
            spent: fields.spent,
            ending,
        },
    }
}
fn job_answer(record: wire::JobAnswer, parts: &mut Parts) -> host::LinkUp {
    let fields = record.into_parts();
    let answer = match fields.ending {
        wire::JobEnding::Refused(record) => {
            let refusal = match record.into_parts().value {
                wire::JobRefusal::Busy => host::JobRefusal::Busy,
                wire::JobRefusal::Invalid(record) => host::JobRefusal::Invalid(match record.into_parts().value {
                    wire::JobInvalid::Kind => host::JobInvalid::Kind,
                    wire::JobInvalid::Spec => host::JobInvalid::Spec,
                    wire::JobInvalid::Deadline => host::JobInvalid::Deadline,
                    wire::JobInvalid::Grants => host::JobInvalid::Grants,
                    wire::JobInvalid::Start(record) => host::JobInvalid::Start { why: record.into_parts().why },
                }),
            };
            host::JobEnding::Refused(refusal)
        }
        wire::JobEnding::Finished(record) => {
            let fields = record.into_parts();
            host::JobEnding::Finished { result: fields.result, work: work(fields.work, parts) }
        }
        wire::JobEnding::Failed(record) => {
            let fields = record.into_parts();
            host::JobEnding::Failed {
                failure: job_failure(fields.failure, parts),
                detail: fields.detail,
                work: work(fields.work, parts),
            }
        }
    };
    host::LinkUp::JobAnswer { run: Token::new(fields.task), attempt: Token::new(fields.attempt), answer }
}
fn job_failure(value: wire::JobFailure, parts: &mut Parts) -> host::JobFailure {
    match value {
        wire::JobFailure::Unprepared(record) => host::JobFailure::Unprepared(match record.into_parts().value {
            wire::Preparation::Transient => host::Preparation::Transient,
            wire::Preparation::Permanent(record) => {
                host::Preparation::Permanent { resource: parts.optional(record.into_parts().resource) }
            }
        }),
        wire::JobFailure::Failed => host::JobFailure::Failed,
        wire::JobFailure::Deadline => host::JobFailure::Deadline,
        wire::JobFailure::Result => host::JobFailure::Result,
        wire::JobFailure::Unstarted => host::JobFailure::Unstarted,
        wire::JobFailure::Undelivered(record) => host::JobFailure::Undelivered(match record.into_parts().value {
            wire::DeliveryOutcome::Delivered => host::DeliveryOutcome::Delivered,
            wire::DeliveryOutcome::Nothing => host::DeliveryOutcome::Nothing,
            wire::DeliveryOutcome::Stale => host::DeliveryOutcome::Stale,
            wire::DeliveryOutcome::Refused => host::DeliveryOutcome::Refused,
            wire::DeliveryOutcome::Failed => host::DeliveryOutcome::Failed,
        }),
        wire::JobFailure::Cancelled(record) => host::JobFailure::Cancelled(match record.into_parts().value {
            wire::CancelReason::Engine => host::Reason::Engine,
            wire::CancelReason::Contact => host::Reason::Contact,
            wire::CancelReason::Shutdown => host::Reason::Shutdown,
        }),
    }
}
fn ending(value: wire::Ending, parts: &mut Parts) -> host::Ending {
    match value {
        wire::Ending::Refused(record) => {
            let refusal = match record.into_parts().value {
                wire::Refusal::Busy => host::Refusal::Busy,
                wire::Refusal::Version(record) => {
                    let value = record.into_parts();
                    host::Refusal::Version(host::Mismatch {
                        wire: value.wire,
                        version: value.version,
                        lowest: value.lowest,
                        highest: value.highest,
                    })
                }
                wire::Refusal::Invalid(record) => {
                    let invalid = match record.into_parts().value {
                        wire::Invalid::Charter => host::Invalid::Charter,
                        wire::Invalid::Transcript => host::Invalid::Transcript,
                        wire::Invalid::Grants => host::Invalid::Grants,
                        wire::Invalid::Messages => host::Invalid::Messages,
                        wire::Invalid::Start(record) => {
                            let record = record.into_parts();
                            host::Invalid::Start {
                                why: record.why,
                                cause: match record.cause {
                                    wire::StartCause::Other => host::StartCause::Other,
                                    wire::StartCause::Grants => host::StartCause::Grants,
                                    wire::StartCause::Transcript => host::StartCause::Transcript,
                                },
                            }
                        }
                    };
                    host::Refusal::Invalid(invalid)
                }
            };
            host::Ending::Refused(refusal)
        }
        wire::Ending::Ended(record) => {
            let fields = record.into_parts();
            host::Ending::Ended { outcome: fields.outcome, work: work(fields.work, parts) }
        }
        wire::Ending::Parked(record) => host::Ending::Parked { work: work(record.into_parts().work, parts) },
        wire::Ending::Failed(record) => {
            let fields = record.into_parts();
            let failure = failure(fields.failure, parts);
            let model = model_failure(fields.model);
            host::Ending::Failed { failure, detail: fields.detail, work: work(fields.work, parts), model }
        }
    }
}
fn model_failure(value: Option<wire::ModelFailure>) -> Option<host::ModelFailure> {
    let fields = value?.into_parts();
    let kind = match fields.kind {
        wire::ModelFailureKind::Unavailable => host::ModelFailureKind::Unavailable,
        wire::ModelFailureKind::Overloaded => host::ModelFailureKind::Overloaded,
        wire::ModelFailureKind::RateLimited => host::ModelFailureKind::RateLimited,
        wire::ModelFailureKind::Exhausted => host::ModelFailureKind::Exhausted,
        wire::ModelFailureKind::TimedOut => host::ModelFailureKind::TimedOut,
        wire::ModelFailureKind::ContextTooLong => host::ModelFailureKind::ContextTooLong,
        wire::ModelFailureKind::Invalid => host::ModelFailureKind::Invalid,
        wire::ModelFailureKind::Refused => host::ModelFailureKind::Refused,
        wire::ModelFailureKind::Unsupported => host::ModelFailureKind::Unsupported,
        wire::ModelFailureKind::Unauthorized => host::ModelFailureKind::Unauthorized,
        wire::ModelFailureKind::Protocol => host::ModelFailureKind::Protocol,
        wire::ModelFailureKind::Cancelled => host::ModelFailureKind::Cancelled,
        wire::ModelFailureKind::Limit => host::ModelFailureKind::Limit,
    };
    let evidence = match fields.evidence {
        wire::ModelEvidence::Unsent => host::ModelEvidence::Unsent,
        wire::ModelEvidence::Unknown => host::ModelEvidence::Unknown,
        wire::ModelEvidence::Response => host::ModelEvidence::Response,
    };
    Some(host::ModelFailure { endpoint: fields.endpoint, model: fields.model, kind, evidence, status: fields.status })
}

fn work(value: wire::Work, parts: &mut Parts) -> host::Work {
    let fields = value.into_parts();
    let left = parts.optional(fields.left);
    let saved = parts.optional(fields.saved);
    host::Work { left, saved }
}
fn failure(value: wire::Failure, parts: &mut Parts) -> host::Failure {
    match value {
        wire::Failure::Unprepared(record) => {
            let preparation = match record.into_parts().value {
                wire::Preparation::Transient => host::Preparation::Transient,
                wire::Preparation::Permanent(record) => {
                    host::Preparation::Permanent { resource: parts.optional(record.into_parts().resource) }
                }
            };
            host::Failure::Unprepared(preparation)
        }
        wire::Failure::Run(record) => host::Failure::Run(run_failure(record.into_parts().value)),
        wire::Failure::Agent(record) => host::Failure::Agent(agent_failure(record.into_parts().value)),
        wire::Failure::Cancelled(record) => {
            let reason = match record.into_parts().value {
                wire::CancelReason::Engine => host::Reason::Engine,
                wire::CancelReason::Contact => host::Reason::Contact,
                wire::CancelReason::Shutdown => host::Reason::Shutdown,
            };
            host::Failure::Cancelled(reason)
        }
    }
}
fn run_failure(value: wire::RunFailure) -> host::RunFailure {
    match value {
        wire::RunFailure::Exceeded(value) => host::RunFailure::Exceeded(host::Exceeded {
            bound: match value.bound() {
                wire::Bound::Reasoning => host::Bound::Reasoning,
                wire::Bound::Metadata => host::Bound::Metadata,
                wire::Bound::Items => host::Bound::Items,
                wire::Bound::Answer => host::Bound::Answer,
                wire::Bound::Tools => host::Bound::Tools,
                wire::Bound::HistoryItems => host::Bound::HistoryItems,
                wire::Bound::History => host::Bound::History,
                wire::Bound::Window => host::Bound::Window,
                wire::Bound::Output => host::Bound::Output,
            },
            value: value.value(),
        }),
        wire::RunFailure::Model => host::RunFailure::Model,
        wire::RunFailure::Transcript => host::RunFailure::Transcript,
        wire::RunFailure::Budget => host::RunFailure::Budget,
        wire::RunFailure::Policy => host::RunFailure::Policy,
        wire::RunFailure::Cancelled => host::RunFailure::Cancelled,
        wire::RunFailure::Stale => host::RunFailure::Stale,
        wire::RunFailure::Limited(record) => {
            let fields = record.into_parts();
            host::RunFailure::Limited { account: fields.account, reset: fields.reset }
        }
        wire::RunFailure::Refused(record) => host::RunFailure::Refused { account: record.into_parts().account },
    }
}
fn agent_failure(value: wire::AgentFailure) -> host::AgentFailure {
    match value {
        wire::AgentFailure::Unstarted => host::AgentFailure::Unstarted,
        wire::AgentFailure::Exited => host::AgentFailure::Exited,
        wire::AgentFailure::Rules => host::AgentFailure::Rules,
        wire::AgentFailure::NoProgress => host::AgentFailure::NoProgress,
        wire::AgentFailure::WallTime => host::AgentFailure::WallTime,
    }
}

pub(crate) fn call(record: wire::Call) -> host::LinkUp {
    let fields = record.into_parts();
    host::LinkUp::Call {
        run: Token::new(fields.task),
        attempt: Token::new(fields.attempt),
        contract: fields.contract,
        call: fields.name,
        tool: fields.tool,
        writes: fields.writes,
        input: fields.input,
        deadline: fields.deadline,
    }
}

/// These records are private to the translation; the wire crate exposes individual codecs.
#[derive(Debug)]
pub(crate) enum Down {
    Asked(wire::Asked),
    ReadList(wire::ReadList),
    Sources(wire::Sources),
    Check(wire::Check),
    SignOut(wire::SignOut),
    Assign(wire::Assignment),
    AssignJob(wire::Job),
    Message(wire::Inbound),
    Relayed(wire::Relayed),
    TurnKept(wire::TurnKept),
    TurnBusy(wire::TurnBusy),
    Acknowledge(wire::Acknowledge),
    Cancel(wire::Cancel),
    Grant(wire::Granted),
}
impl Down {
    pub fn kind(&self) -> wire::Kind {
        match self {
            Self::Asked(_) => wire::Kind::Asked,
            Self::ReadList(_) => wire::Kind::ReadList,
            Self::Sources(_) => wire::Kind::Sources,
            Self::Check(_) => wire::Kind::Check,
            Self::SignOut(_) => wire::Kind::SignOut,
            Self::Assign(_) => wire::Kind::Assignment,
            Self::AssignJob(_) => wire::Kind::Job,
            Self::Message(_) => wire::Kind::Message,
            Self::Relayed(_) => wire::Kind::Relayed,
            Self::TurnKept(_) => wire::Kind::TurnKept,
            Self::TurnBusy(_) => wire::Kind::TurnBusy,
            Self::Acknowledge(_) => wire::Kind::Acknowledge,
            Self::Cancel(_) => wire::Kind::Cancel,
            Self::Grant(_) => wire::Kind::Grant,
        }
    }
    pub fn frame(self) -> skein_channel::Frame {
        let (kind, length) = match &self {
            Self::Asked(record) => (wire::Kind::Asked.number(), record.measure()),
            Self::ReadList(record) => (wire::Kind::ReadList.number(), record.measure()),
            Self::Sources(record) => (0x0188, record.measure()),
            Self::Check(record) => (0x0189, record.measure()),
            Self::SignOut(record) => (0x018a, record.measure()),
            Self::Assign(record) => (0x0180, record.measure()),
            Self::AssignJob(record) => (wire::Kind::Job.number(), record.measure()),
            Self::Message(record) => (0x0181, record.measure()),
            Self::Relayed(record) => (0x0182, record.measure()),
            Self::TurnKept(record) => (0x0183, record.measure()),
            Self::TurnBusy(record) => (0x0184, record.measure()),
            Self::Acknowledge(record) => (0x0185, record.measure()),
            Self::Cancel(record) => (0x0186, record.measure()),
            Self::Grant(record) => (0x0187, record.measure()),
        };
        let mut writer = skein_channel::frame_writer(kind, length).expect("checked record fits a frame");
        let mut body = skein_lib::Writer::new(skein_lib::length::usize_of(length));
        let encoded = match self {
            Self::Asked(record) => record.encode(&mut body),
            Self::ReadList(record) => record.encode(&mut body),
            Self::Sources(record) => record.encode(&mut body),
            Self::Check(record) => record.encode(&mut body),
            Self::SignOut(record) => record.encode(&mut body),
            Self::Assign(record) => record.encode(&mut body),
            Self::AssignJob(record) => record.encode(&mut body),
            Self::Message(record) => record.encode(&mut body),
            Self::Relayed(record) => record.encode(&mut body),
            Self::TurnKept(record) => record.encode(&mut body),
            Self::TurnBusy(record) => record.encode(&mut body),
            Self::Acknowledge(record) => record.encode(&mut body),
            Self::Cancel(record) => record.encode(&mut body),
            Self::Grant(record) => record.encode(&mut body),
        };
        encoded.expect("measured record");
        writer.put(&body.finish()).expect("measured body fits frame");
        writer.finish().expect("complete record")
    }
}
#[expect(
    clippy::too_many_lines,
    reason = "one arm per downward record keeps each constructor's refusal beside its fields"
)]
pub(crate) fn down(
    down: host::LinkDown,
    parts: &[wire::Part],
    values: Box<[Option<crate::GrantValue>]>,
    limits: &wire::v14::Limits,
) -> Result<Down, crate::Unencoded> {
    match down {
        host::LinkDown::Asked { run, attempt, number, answer } => {
            let answer = match answer {
                host::AskAnswer::Bytes(bytes) => {
                    wire::AskAnswer::Bytes(match wire::AskBytes::new(limits, wire::AskBytesParts { bytes }) {
                        Ok(record) => record,
                        Err(why) => return Err(problem(why)),
                    })
                }
                host::AskAnswer::Refused => wire::AskAnswer::Refused,
            };
            Ok(Down::Asked(
                match wire::Asked::new(
                    limits,
                    wire::AskedParts { run: run.raw(), attempt: attempt.raw(), number, answer },
                ) {
                    Ok(record) => record,
                    Err(why) => return Err(problem(why)),
                },
            ))
        }
        host::LinkDown::ReadList { read, endpoint, grant } => {
            match grant.form {
                Form::Credential(_) => {}
                Form::Application | Form::Program(_) => return Err(crate::Unencoded::field("grant.value")),
            }
            let grant = grant_record(
                grant,
                values.into_iter().next().flatten().ok_or(crate::Unencoded::field("grant.value"))?,
                limits,
            )?;
            Ok(Down::ReadList(match wire::ReadList::new(limits, wire::ReadListParts { read, endpoint, grant }) {
                Ok(record) => record,
                Err(why) => return Err(problem(why)),
            }))
        }
        host::LinkDown::AssignJob { job } => Ok(Down::AssignJob(job_record(job, parts, values, limits)?)),

        host::LinkDown::Sources { sources } => {
            if !host::sources::unique(&sources) {
                return Err(crate::Unencoded::field("sources.names"));
            }
            measured(
                "Sources.sources",
                skein_lib::length::len_u64(sources.len()),
                limits.sources_sources,
                wire::v14::CEILINGS.sources_sources,
            )?;
            let mut names = List::with_capacity(u32::try_from(sources.len()).expect("measured sources count fits"));
            for name in sources {
                names
                    .push(
                        match wire::ProgramSource::new(
                            limits,
                            wire::ProgramSourceParts { name: Box::from(name.as_bytes()) },
                        ) {
                            Ok(record) => record,
                            Err(why) => return Err(problem(why)),
                        },
                    )
                    .expect("measured sources count");
            }
            Ok(Down::Sources(match wire::Sources::new(limits, wire::SourcesParts { sources: names }) {
                Ok(record) => record,
                Err(why) => return Err(problem(why)),
            }))
        }
        host::LinkDown::Check { name } => {
            Ok(Down::Check(match wire::Check::new(limits, wire::CheckParts { name: Box::from(name.as_bytes()) }) {
                Ok(record) => record,
                Err(why) => return Err(problem(why)),
            }))
        }
        host::LinkDown::SignOut { name } => Ok(Down::SignOut(
            match wire::SignOut::new(limits, wire::SignOutParts { name: Box::from(name.as_bytes()) }) {
                Ok(record) => record,
                Err(why) => return Err(problem(why)),
            },
        )),
        host::LinkDown::Assign { assignment } => {
            Ok(Down::Assign(assignment_record(assignment, parts, values, limits)?))
        }
        host::LinkDown::Message { run, attempt, name, sender, words, finish_requested } => Ok(Down::Message(
            match wire::Inbound::new(
                limits,
                wire::InboundParts {
                    task: run.raw(),
                    attempt: attempt.raw(),
                    name: name.raw(),
                    sender,
                    words,
                    finish_requested,
                },
            ) {
                Ok(record) => record,
                Err(why) => return Err(problem(why)),
            },
        )),
        host::LinkDown::Relayed { run, attempt, call, answer } => {
            let answer = match answer {
                host::Relayed::Busy => wire::RelayAnswer::Busy,
                host::Relayed::Answer { error, text } => {
                    wire::RelayAnswer::Text(match wire::RelayText::new(limits, wire::RelayTextParts { error, text }) {
                        Ok(record) => record,
                        Err(why) => return Err(problem(why)),
                    })
                }
            };
            Ok(Down::Relayed(
                match wire::Relayed::new(
                    limits,
                    wire::RelayedParts { task: run.raw(), attempt: attempt.raw(), name: call, answer },
                ) {
                    Ok(record) => record,
                    Err(why) => return Err(problem(why)),
                },
            ))
        }
        host::LinkDown::TurnKept { run, attempt, turn } => Ok(Down::TurnKept(
            match wire::TurnKept::new(
                limits,
                wire::TurnKeptParts { task: run.raw(), attempt: attempt.raw(), number: turn },
            ) {
                Ok(record) => record,
                Err(why) => return Err(problem(why)),
            },
        )),
        host::LinkDown::Acknowledge { run, attempt } => Ok(Down::Acknowledge(
            match wire::Acknowledge::new(limits, wire::AcknowledgeParts { task: run.raw(), attempt: attempt.raw() }) {
                Ok(record) => record,
                Err(why) => return Err(problem(why)),
            },
        )),
        host::LinkDown::Cancel { run, attempt } => Ok(Down::Cancel(
            match wire::Cancel::new(limits, wire::CancelParts { task: run.raw(), attempt: attempt.raw() }) {
                Ok(record) => record,
                Err(why) => return Err(problem(why)),
            },
        )),
        host::LinkDown::Grant { run, attempt, grant } => {
            let value = values.into_iter().next().flatten().ok_or(crate::Unencoded::field("grant.value"))?;
            let grant = grant_record(grant, value, limits)?;
            Ok(Down::Grant(
                match wire::Granted::new(limits, wire::GrantedParts { task: run.raw(), attempt: attempt.raw(), grant })
                {
                    Ok(record) => record,
                    Err(why) => return Err(problem(why)),
                },
            ))
        }
    }
}
fn job_record(
    job: host::JobAssignment,
    parts: &[wire::Part],
    values: Box<[Option<crate::GrantValue>]>,
    limits: &wire::v14::Limits,
) -> Result<wire::Job, crate::Unencoded> {
    let workspace = workspace_record(job.workspace, parts, limits)?;
    let grants =
        grants_record(job.grants, values, "Job.grants", limits.job_grants, wire::v14::CEILINGS.job_grants, limits)?;
    match wire::Job::new(
        limits,
        wire::JobParts {
            task: job.run.raw(),
            attempt: job.attempt.raw(),
            kind: job.kind.0,
            spec: job.spec,
            deadline: job.deadline,
            workspace,
            grants,
        },
    ) {
        Ok(record) => Ok(record),
        Err(why) => Err(problem(why)),
    }
}
fn assignment_record(
    assignment: host::Assignment,
    parts: &[wire::Part],
    values: Box<[Option<crate::GrantValue>]>,
    limits: &wire::v14::Limits,
) -> Result<wire::Assignment, crate::Unencoded> {
    let run = assignment.assignment;
    let workspace = workspace_record(run.workspace, parts, limits)?;
    measured(
        "Assignment.messages",
        skein_lib::length::len_u64(assignment.messages.len()),
        limits.assignment_messages,
        wire::v14::CEILINGS.assignment_messages,
    )?;
    let mut messages =
        List::with_capacity(u32::try_from(assignment.messages.len()).expect("measured messages count fits"));
    for message in assignment.messages {
        messages
            .push(
                match wire::Message::new(
                    limits,
                    wire::MessageParts {
                        name: message.name.raw(),
                        sender: message.sender,
                        words: message.words,
                        finish_requested: message.finish_requested,
                    },
                ) {
                    Ok(record) => record,
                    Err(why) => return Err(problem(why)),
                },
            )
            .expect("measured messages count");
    }
    measured(
        "Assignment.answered",
        skein_lib::length::len_u64(assignment.answered.len()),
        limits.assignment_answered,
        wire::v14::CEILINGS.assignment_answered,
    )?;
    let mut answered =
        List::with_capacity(u32::try_from(assignment.answered.len()).expect("measured answered count fits"));
    for call in assignment.answered {
        let answer = settled(call.answer, limits)?;
        answered
            .push(
                match wire::AnsweredCall::new(
                    limits,
                    wire::AnsweredCallParts { name: call.name, tool: call.tool, answer },
                ) {
                    Ok(record) => record,
                    Err(why) => return Err(problem(why)),
                },
            )
            .expect("measured answered count");
    }
    let grants = grants_record(
        run.grants,
        values,
        "Assignment.grants",
        limits.assignment_grants,
        wire::v14::CEILINGS.assignment_grants,
        limits,
    )?;
    measured(
        "Assignment.charter",
        skein_lib::length::len_u64(run.charter.len()),
        limits.assignment_charter,
        wire::v14::CEILINGS.assignment_charter,
    )?;
    let transcript = transcript(&assignment.turns, limits.assignment_transcript)?;
    match wire::Assignment::new(
        limits,
        wire::AssignmentParts {
            task: run.run.raw(),
            attempt: run.attempt.raw(),
            identity: run.identity,
            workspace,
            save: run.save,
            live: match wire::LiveAsk::new(
                limits,
                wire::LiveAskParts { text: run.live.text, reasoning: run.live.reasoning, input: run.live.input },
            ) {
                Ok(record) => record,
                Err(why) => return Err(problem(why)),
            },
            charter: run.charter,
            transcript,
            answered,
            messages,
            grants,
        },
    ) {
        Ok(record) => Ok(record),
        Err(why) => Err(problem(why)),
    }
}
fn problem(problem: wire::v14::Problem) -> crate::Unencoded {
    crate::Unencoded {
        field: problem.path.name(),
        reason: Some(problem.reason),
        figure: None,
        bound: None,
        ceiling: None,
    }
}
fn measured(field: &'static str, figure: u64, bound: u32, ceiling: u32) -> Result<(), crate::Unencoded> {
    if figure > u64::from(bound.min(ceiling)) {
        return Err(crate::Unencoded {
            field,
            reason: None,
            figure: Some(figure),
            bound: Some(u64::from(bound)),
            ceiling: Some(u64::from(ceiling)),
        });
    }
    Ok(())
}
fn grants_record(
    grants: Box<[jig_domain_link::Grant]>,
    values: Box<[Option<crate::GrantValue>]>,
    field: &'static str,
    bound: u32,
    ceiling: u32,
    limits: &wire::v14::Limits,
) -> Result<List<wire::Grant>, crate::Unencoded> {
    let mut count = 0_u64;
    for value in values.iter().take(grants.len()) {
        if value.is_some() {
            count = count.saturating_add(1);
        }
    }
    measured(field, count, bound, ceiling)?;
    let mut records = List::with_capacity(u32::try_from(count).expect("measured grants count fits"));
    for (grant, value) in grants.into_iter().zip(values) {
        if let Some(value) = value {
            records.push(grant_record(grant, value, limits)?).expect("measured grants count");
        }
    }
    Ok(records)
}
fn workspace_record(
    workspace: Option<host::Workspace>,
    parts: &[wire::Part],
    limits: &wire::v14::Limits,
) -> Result<Option<wire::Workspace>, crate::Unencoded> {
    match workspace {
        Some(workspace) => match wire::Workspace::new(
            limits,
            wire::WorkspaceParts { workstream: workspace.workstream, items: part(workspace.items, parts)? },
        ) {
            Ok(record) => Ok(Some(record)),
            Err(why) => Err(problem(why)),
        },
        None => Ok(None),
    }
}
fn part(token: Token, parts: &[wire::Part]) -> Result<Box<[u8]>, crate::Unencoded> {
    for part in parts {
        if part.token == token {
            return Ok(part.bytes.clone());
        }
    }
    Err(crate::Unencoded::field("workspace.items"))
}
fn transcript(turns: &[Box<[u8]>], bound: u32) -> Result<Box<[u8]>, crate::Unencoded> {
    let mut length = 0_u64;
    for turn in turns {
        length = length
            .checked_add(skein_lib::length::len_u64(turn.len()))
            .ok_or(crate::Unencoded::field("Assignment.transcript"))?;
        length = length.checked_add(4).ok_or(crate::Unencoded::field("Assignment.transcript"))?;
    }
    measured("Assignment.transcript", length, bound, wire::v14::CEILINGS.assignment_transcript)?;
    let mut writer = skein_lib::Writer::new(usize::try_from(length).expect("bounded framed transcript fits usize"));
    for turn in turns {
        skein_codec::piece::write(&mut writer, turn).expect("measured framed transcript fits");
    }
    Ok(writer.finish())
}
/// A grant's form and its value's arm agree, or the grant is not sent.
fn grant_record(
    grant: jig_domain_link::Grant,
    value: crate::GrantValue,
    limits: &wire::v14::Limits,
) -> Result<wire::Grant, crate::Unencoded> {
    let value = match grant.form {
        Form::Program(form) => match value {
            crate::GrantValue::Program { name } => {
                if name != form {
                    return Err(crate::Unencoded::field("grant.value"));
                }
                wire::GrantValue::Program(
                    match wire::ProgramValue::new(limits, wire::ProgramValueParts { name: name.as_bytes().into() }) {
                        Ok(record) => record,
                        Err(why) => return Err(problem(why)),
                    },
                )
            }
            crate::GrantValue::Credential { .. } | crate::GrantValue::Application { .. } => {
                return Err(crate::Unencoded::field("grant.value"));
            }
        },
        Form::Credential(form) => match value {
            crate::GrantValue::Credential { scheme, token } => {
                if scheme != grant_scheme(form) {
                    return Err(crate::Unencoded::field("grant.value"));
                }
                let scheme = match form {
                    Scheme::Bearer => wire::Scheme::Bearer,
                    Scheme::Key => wire::Scheme::Key,
                };
                wire::GrantValue::Credential(
                    match wire::CredentialValue::new(limits, wire::CredentialValueParts { scheme, bytes: token }) {
                        Ok(record) => record,
                        Err(why) => return Err(problem(why)),
                    },
                )
            }
            crate::GrantValue::Application { .. } | crate::GrantValue::Program { .. } => {
                return Err(crate::Unencoded::field("grant.value"));
            }
        },
        Form::Application => match value {
            crate::GrantValue::Application { secret } => wire::GrantValue::Application(
                match wire::ApplicationValue::new(limits, wire::ApplicationValueParts { secret }) {
                    Ok(record) => record,
                    Err(why) => return Err(problem(why)),
                },
            ),
            crate::GrantValue::Credential { .. } | crate::GrantValue::Program { .. } => {
                return Err(crate::Unencoded::field("grant.value"));
            }
        },
    };
    match wire::Grant::new(
        limits,
        wire::GrantParts { number: grant.account, generation: grant.generation, valid: grant.valid, value },
    ) {
        Ok(record) => Ok(record),
        Err(why) => Err(problem(why)),
    }
}
const fn grant_scheme(scheme: Scheme) -> crate::Scheme {
    match scheme {
        Scheme::Bearer => crate::Scheme::Bearer,
        Scheme::Key => crate::Scheme::Key,
    }
}
fn settled(answer: host::SettledAnswer, limits: &wire::v14::Limits) -> Result<wire::SettledAnswer, crate::Unencoded> {
    match answer {
        host::SettledAnswer::Host { error, body } => {
            Ok(wire::SettledAnswer::Host(match wire::HostAnswer::new(limits, wire::HostAnswerParts { error, body }) {
                Ok(record) => record,
                Err(why) => return Err(problem(why)),
            }))
        }
        host::SettledAnswer::Delivery { outcome, evidence } => {
            let outcome = match outcome {
                host::DeliveryOutcome::Delivered => wire::DeliveryOutcome::Delivered,
                host::DeliveryOutcome::Nothing => wire::DeliveryOutcome::Nothing,
                host::DeliveryOutcome::Stale => wire::DeliveryOutcome::Stale,
                host::DeliveryOutcome::Refused => wire::DeliveryOutcome::Refused,
                host::DeliveryOutcome::Failed => wire::DeliveryOutcome::Failed,
            };
            Ok(wire::SettledAnswer::Delivery(
                match wire::DeliveryAnswer::new(limits, wire::DeliveryAnswerParts { outcome, evidence }) {
                    Ok(record) => record,
                    Err(why) => return Err(problem(why)),
                },
            ))
        }
    }
}

pub(crate) fn turn_busy(run: Token, attempt: Token, turn: u32, limits: &wire::v14::Limits) -> Down {
    Down::TurnBusy(
        wire::TurnBusy::new(limits, wire::TurnBusyParts { task: run.raw(), attempt: attempt.raw(), number: turn })
            .expect("fixed turn busy fields fit"),
    )
}

fn source_failure(reason: wire::SourceFailure) -> SourceFailure {
    match reason {
        wire::SourceFailure::Missing => SourceFailure::Missing,
        wire::SourceFailure::Deadline => SourceFailure::Deadline,
        wire::SourceFailure::Unreadable => SourceFailure::Unreadable,
        wire::SourceFailure::Exited => SourceFailure::Exited,
        wire::SourceFailure::Unconfigured => SourceFailure::Unconfigured,
        wire::SourceFailure::Unknown => SourceFailure::Unknown,
        wire::SourceFailure::Closed => SourceFailure::Closed,
    }
}
fn login_state(state: wire::LoginState) -> Option<LoginState> {
    Some(match state {
        wire::LoginState::Usable => LoginState::Usable,
        wire::LoginState::NotSignedIn => LoginState::NotSignedIn,
        wire::LoginState::SignedInOtherwise(method) => {
            LoginState::SignedInOtherwise(LoginMethod::new(method.method())?)
        }
        wire::LoginState::Failed(failure) => {
            let failure = failure.into_parts();
            LoginState::Failed { reason: source_failure(failure.reason), detail: failure.detail }
        }
    })
}

fn list_outcome(value: wire::ListOutcome) -> jig_domain_link::ListOutcome {
    use jig_domain_link::{ListFailure, ListOutcome, ListedModel};
    match value {
        wire::ListOutcome::Listed(record) => {
            let rows = record.into_parts().rows;
            let mut listed = List::with_capacity(rows.len());
            for row in rows.into_boxed() {
                let row = row.into_parts();
                listed
                    .push(ListedModel {
                        name: row.name,
                        shown: row.shown,
                        window: row.window,
                        output: row.output,
                        efforts: row.efforts.into_boxed(),
                    })
                    .expect("one slot per decoded row");
            }
            ListOutcome::Listed(listed.into_boxed())
        }
        wire::ListOutcome::Failed(record) => ListOutcome::Failed(match record.into_parts().value {
            wire::ListFailed::Refused(record) => ListFailure::Refused { status: record.status() },
            wire::ListFailed::Unreachable => ListFailure::Unreachable,
            wire::ListFailed::Unreadable => ListFailure::Unreadable,
            wire::ListFailed::Deadline => ListFailure::Deadline,
            wire::ListFailed::Agent => ListFailure::Agent,
        }),
    }
}

/// Byte bridges surround this component's current-value/domain translation (bridges.md, 5).
pub(crate) fn send(
    machine: &mut wire::translation::Translation,
    frame: skein_channel::Frame,
    version: u16,
    agreed: &List<wire::Agreed>,
    limits: &wire::v14::Limits,
) -> Result<skein_channel::Frame, wire::translation::Untranslated> {
    machine.send(frame, version, agreed.as_slice(), limits)
}
pub(crate) fn receive(
    machine: &mut wire::translation::Translation,
    kind: u16,
    body: &[u8],
    version: u16,
    agreed: &List<wire::Agreed>,
    limits: &wire::v14::Limits,
) -> Result<skein_channel::Frame, wire::translation::Untranslated> {
    machine.receive(kind, body, version, agreed.as_slice(), limits)
}
