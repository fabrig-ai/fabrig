//! Total translations of the worker records (protocol/channel/README.md, section 3, and protocol/channel/owners.md, section 1.4).
//! Jobs use their own records (jig/docs/design/protocol/channel/records.md, 4).
use crate::generated::worker_wire as mapped;
use alloc::boxed::Box;
use jig_domain_host as host;
use jig_domain_link::{Form, Hello, JobKind, Login, LoginMethod, Scheme, SourceName, valid_program_endpoints};
use jig_protocol_worker_wire::{self as wire, Record};
use skein_codec::convert::{token_to_domain, token_to_record, wall_to_domain};
use skein_lib::{Duration, List, Reader, Token};

pub(super) fn job_kind_to_domain(kind: u32) -> JobKind {
    JobKind(kind)
}

fn job_kind_to_record(kind: JobKind) -> u32 {
    kind.0
}

pub(super) fn source_name_to_domain(name: Box<[u8]>) -> Result<SourceName, skein_codec::Reason> {
    SourceName::new(&name).ok_or(skein_codec::Reason::Malformed)
}

fn source_name_to_record(name: &SourceName) -> Box<[u8]> {
    Box::from(name.as_bytes())
}

pub(super) fn login_method_to_domain(method: Box<[u8]>) -> Result<LoginMethod, skein_codec::Reason> {
    LoginMethod::new(&method).ok_or(skein_codec::Reason::Malformed)
}

pub(crate) fn wait_ms_to_record(wait: Duration) -> u64 {
    wait.as_millis()
}

pub(crate) fn hello(record: wire::Hello) -> Option<Hello> {
    if !valid_program_endpoints(record.bounds.program_endpoints.as_slice()) {
        return None;
    }
    let mut hosting = List::with_capacity(record.hosting.len());
    for run in record.hosting.into_boxed() {
        hosting.push(mapped::hosting_to_domain(run)).expect("one slot per decoded run");
    }
    let mut jobs = List::with_capacity(record.jobs.len());
    for job in record.jobs.into_boxed() {
        jobs.push(mapped::job_bound_to_domain(job)).expect("one slot per decoded job kind");
    }
    Some(Hello {
        fresh: record.fresh,
        reads_usage: false,
        jobs: jobs.into_boxed(),
        slots: record.slots,
        stop_bound: record.stop_bound,
        workstreams: record.workstreams.into_boxed(),
        hosting: hosting.into_boxed(),
        bounds: mapped::bounds_to_domain(record.bounds),
    })
}

#[derive(Debug)]
pub(crate) enum Up {
    Source(host::LinkUp),
    Ask(wire::Ask),
    Guides(wire::v22::Guides),
    ListRead(wire::ListRead),
    Part(wire::v22::Part),
    Turn(wire::Turn),
    Call(wire::Call),
    Answer(wire::Answer),
    JobAnswer(wire::JobAnswer),
    Fact(wire::Fact),
    Unauthorized(wire::Unauthorized),
    Limited(wire::Limited),
    AccountRefused(wire::AccountRefused),
}
pub(crate) fn decode(kind: u16, body: &[u8], limits: &wire::v22::Limits) -> Option<Up> {
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
        wire::Kind::Paused => {
            let record = wire::Paused::decode(limits, reader).ok()?;
            let state = mapped::pause_state_to_domain(record.state);
            Some(Up::Source(host::LinkUp::Paused { state }))
        }
        wire::Kind::Draining => {
            wire::Draining::decode(limits, reader).ok()?;
            Some(Up::Source(host::LinkUp::Draining))
        }
        wire::Kind::Usage => {
            let row = wire::Usage::decode(limits, reader).ok()?;
            let window = usage_window(row.label, row.length, row.used, row.resets, row.state)?;
            Some(Up::Source(host::LinkUp::Usage {
                run: token_to_domain(row.task),
                attempt: token_to_domain(row.attempt),
                account: row.account,
                window,
            }))
        }
        wire::Kind::UsageRead => {
            let row = wire::UsageRead::decode(limits, reader).ok()?;
            Some(Up::Source(host::LinkUp::UsageRead {
                read: row.read,
                outcome: mapped::usage_outcome_to_domain(row.outcome).ok()?,
            }))
        }
        wire::Kind::ListRead => Some(Up::ListRead(wire::ListRead::decode(limits, reader).ok()?)),
        wire::Kind::Logins => {
            let record = wire::Logins::decode(limits, reader).ok()?;
            let mut rows = List::<Login>::with_capacity(record.logins.len());
            for row in record.logins.into_boxed() {
                let login = mapped::login_to_domain(row).ok()?;
                for old in &rows {
                    if old.name == login.name {
                        return None;
                    }
                }
                rows.push(login).ok()?;
            }
            Some(Up::Source(host::LinkUp::Logins { logins: rows.into_boxed() }))
        }
        wire::Kind::SignedOut => {
            let record = wire::SignedOut::decode(limits, reader).ok()?;
            let name = source_name_to_domain(record.name).ok()?;
            let result = mapped::sign_out_result_to_domain(record.result);
            Some(Up::Source(host::LinkUp::SignedOut { name, result }))
        }
        wire::Kind::Guides => Some(Up::Guides(wire::v22::Guides::decode(&wire::sizes::guides(limits), reader).ok()?)),
        wire::Kind::Part => Some(Up::Part(wire::v22::Part::decode(limits, reader).ok()?)),
        wire::Kind::Turn => Some(Up::Turn(wire::Turn::decode(limits, reader).ok()?)),
        wire::Kind::Call => Some(Up::Call(wire::Call::decode(limits, reader).ok()?)),
        wire::Kind::JobAnswer => Some(Up::JobAnswer(wire::JobAnswer::decode(limits, reader).ok()?)),
        wire::Kind::Answer => Some(Up::Answer(wire::Answer::decode(limits, reader).ok()?)),
        wire::Kind::Fact => Some(Up::Fact(wire::Fact::decode(limits, reader).ok()?)),
        wire::Kind::Unauthorized => Some(Up::Unauthorized(wire::Unauthorized::decode(limits, reader).ok()?)),
        wire::Kind::Limited => Some(Up::Limited(wire::Limited::decode(limits, reader).ok()?)),
        wire::Kind::AccountRefused => Some(Up::AccountRefused(wire::AccountRefused::decode(limits, reader).ok()?)),
        wire::Kind::Pause
        | wire::Kind::Restarting
        | wire::Kind::Asked
        | wire::Kind::ReadUsage
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
        | wire::Kind::PartKept
        | wire::Kind::PartBusy
        | wire::Kind::TurnBusy
        | wire::Kind::Acknowledge
        | wire::Kind::Cancel
        | wire::Kind::Figures
        | wire::Kind::Grant => None,
    }
}
pub(crate) fn up(record: Up, parts: &mut Parts) -> host::LinkUp {
    match record {
        Up::Guides(record) => host::LinkUp::Guides {
            run: token_to_domain(record.task),
            attempt: token_to_domain(record.attempt),
            bytes: record.bytes.bytes,
        },
        Up::Ask(record) => host::LinkUp::Ask {
            run: token_to_domain(record.run),
            attempt: token_to_domain(record.attempt),
            number: record.number,
            bytes: record.bytes,
        },
        Up::Source(up) => up,
        Up::ListRead(record) => {
            host::LinkUp::ListRead { read: record.read, outcome: mapped::list_outcome_to_domain(record.outcome) }
        }
        Up::Part(record) => host::LinkUp::Part {
            run: token_to_domain(record.task),
            attempt: token_to_domain(record.attempt),
            part: jig_domain_link::Part {
                number: record.number,
                turn: record.turn,
                thread: record.thread,
                call: record.call,
                kind: mapped::part_kind_to_domain(record.kind),
                body: record.body,
            },
        },
        Up::Turn(record) => turn(record),
        Up::Call(record) => call(record),
        Up::Answer(record) => answer(record, parts),
        Up::JobAnswer(record) => job_answer(record, parts),
        Up::Fact(record) => fact(record),
        Up::Unauthorized(record) => {
            let fields = record;
            host::LinkUp::Unauthorized {
                run: token_to_domain(fields.task),
                attempt: token_to_domain(fields.attempt),
                account: fields.account,
                generation: fields.generation,
            }
        }
        Up::Limited(record) => {
            let fields = record;
            host::LinkUp::Limited {
                run: token_to_domain(fields.task),
                attempt: token_to_domain(fields.attempt),
                account: fields.account,
                reset: fields.reset,
            }
        }
        Up::AccountRefused(record) => {
            let fields = record;
            host::LinkUp::AccountRefused {
                run: token_to_domain(fields.task),
                attempt: token_to_domain(fields.attempt),
                account: fields.account,
                code: fields.code,
                reason: fields.reason,
            }
        }
    }
}
fn turn(record: wire::Turn) -> host::LinkUp {
    let fields = record;
    let refused = refusals(fields.refused);
    host::LinkUp::Turn {
        run: token_to_domain(fields.task),
        attempt: token_to_domain(fields.attempt),
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
        refused.push(mapped::refused_to_domain(record)).expect("one slot per refusal");
    }
    refused.into_boxed()
}
fn optional_token(value: Option<u64>) -> Option<Token> {
    Some(token_to_domain(value?))
}
fn fact(record: wire::Fact) -> host::LinkUp {
    let fields = record;
    let activity = match fields.value {
        wire::FactValue::Piece(piece) => {
            return host::LinkUp::Piece {
                run: token_to_domain(fields.task),
                attempt: token_to_domain(fields.attempt),
                piece: mapped::piece_to_domain(piece),
            };
        }
        wire::FactValue::Phase(value) => host::AgentActivity::Phase(mapped::phase_to_domain(value.value)),
        wire::FactValue::Long(value) => {
            host::AgentActivity::Long { operation: mapped::operation_to_domain(value.operation), span: value.span }
        }
        wire::FactValue::LongDone => host::AgentActivity::LongDone,
        wire::FactValue::Count(value) => {
            let fields = value;
            host::AgentActivity::Count {
                kind: mapped::count_kind_to_domain(fields.kind),
                elapsed: fields.elapsed,
                count: fields.value,
            }
        }
    };
    host::LinkUp::Activity { run: token_to_domain(fields.task), attempt: token_to_domain(fields.attempt), activity }
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
    let fields = record;
    let ending = ending(fields.ending, parts);
    host::LinkUp::Answer {
        run: token_to_domain(fields.task),
        attempt: token_to_domain(fields.attempt),
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
    let fields = record;
    let answer = match fields.ending {
        wire::JobEnding::Refused(record) => {
            let refusal = match record.value {
                wire::JobRefusal::Busy => host::JobRefusal::Busy,
                wire::JobRefusal::Invalid(record) => {
                    host::JobRefusal::Invalid(mapped::job_invalid_to_domain(record.value))
                }
            };
            host::JobEnding::Refused(refusal)
        }
        wire::JobEnding::Finished(record) => {
            let fields = record;
            host::JobEnding::Finished { result: fields.result, work: work(fields.work, parts) }
        }
        wire::JobEnding::Failed(record) => {
            let fields = record;
            host::JobEnding::Failed {
                failure: job_failure(fields.failure, parts),
                detail: fields.detail,
                work: work(fields.work, parts),
            }
        }
    };
    host::LinkUp::JobAnswer { run: token_to_domain(fields.task), attempt: token_to_domain(fields.attempt), answer }
}
fn job_failure(value: wire::JobFailure, parts: &mut Parts) -> host::JobFailure {
    match value {
        wire::JobFailure::Unprepared(record) => host::JobFailure::Unprepared(match record.value {
            wire::Preparation::Transient => host::Preparation::Transient,
            wire::Preparation::Permanent(record) => {
                host::Preparation::Permanent { resource: parts.optional(record.resource) }
            }
        }),
        wire::JobFailure::Failed => host::JobFailure::Failed,
        wire::JobFailure::Deadline => host::JobFailure::Deadline,
        wire::JobFailure::Result => host::JobFailure::Result,
        wire::JobFailure::Unstarted => host::JobFailure::Unstarted,
        wire::JobFailure::Undelivered(record) => {
            host::JobFailure::Undelivered(mapped::delivery_outcome_to_domain(record.value))
        }
        wire::JobFailure::Cancelled(record) => {
            host::JobFailure::Cancelled(mapped::cancel_reason_to_domain(record.value))
        }
    }
}
#[expect(clippy::manual_map, reason = "step code keeps the optional model translation explicit")]
fn ending(value: wire::Ending, parts: &mut Parts) -> host::Ending {
    match value {
        wire::Ending::Refused(record) => {
            let refusal = match record.value {
                wire::Refusal::Busy => host::Refusal::Busy,
                wire::Refusal::Version(record) => host::Refusal::Version(host::Mismatch {
                    wire: record.wire,
                    version: record.version,
                    worker_version: record.highest,
                }),
                wire::Refusal::Invalid(record) => host::Refusal::Invalid(mapped::invalid_to_domain(record.value)),
            };
            host::Ending::Refused(refusal)
        }
        wire::Ending::Ended(record) => {
            let fields = record;
            host::Ending::Ended { outcome: fields.outcome, work: work(fields.work, parts) }
        }
        wire::Ending::Parked(record) => host::Ending::Parked { work: work(record.work, parts) },
        wire::Ending::Failed(record) => {
            let fields = record;
            let failure = failure(fields.failure, parts);
            let model = match fields.model {
                Some(model) => Some(mapped::model_failure_to_domain(model)),
                None => None,
            };
            host::Ending::Failed { failure, detail: fields.detail, work: work(fields.work, parts), model }
        }
    }
}

fn work(value: wire::Work, parts: &mut Parts) -> host::Work {
    let fields = value;
    let left = parts.optional(fields.left);
    let saved = parts.optional(fields.saved);
    host::Work { left, saved }
}
fn failure(value: wire::Failure, parts: &mut Parts) -> host::Failure {
    match value {
        wire::Failure::Unprepared(record) => {
            let preparation = match record.value {
                wire::Preparation::Transient => host::Preparation::Transient,
                wire::Preparation::Permanent(record) => {
                    host::Preparation::Permanent { resource: parts.optional(record.resource) }
                }
            };
            host::Failure::Unprepared(preparation)
        }
        wire::Failure::Run(record) => host::Failure::Run(mapped::run_failure_to_domain(record.value)),
        wire::Failure::Agent(record) => host::Failure::Agent(mapped::agent_failure_to_domain(record.value)),
        wire::Failure::Cancelled(record) => host::Failure::Cancelled(mapped::cancel_reason_to_domain(record.value)),
    }
}

pub(crate) fn call(record: wire::Call) -> host::LinkUp {
    let fields = record;
    host::LinkUp::Call {
        run: token_to_domain(fields.task),
        attempt: token_to_domain(fields.attempt),
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
    Pause(wire::Pause),
    Restarting(wire::Restarting),
    Asked(wire::Asked),
    ReadUsage(wire::ReadUsage),
    ReadList(wire::ReadList),
    Sources(wire::Sources),
    Check(wire::Check),
    SignOut(wire::SignOut),
    Assign(wire::Assignment),
    AssignJob(wire::Job),
    Message(wire::Inbound),
    Relayed(wire::Relayed),
    PartKept(wire::PartKept),
    PartBusy(wire::PartBusy),
    TurnKept(wire::TurnKept),
    TurnBusy(wire::TurnBusy),
    Acknowledge(wire::Acknowledge),
    Cancel(wire::Cancel),
    Grant(wire::Granted),
}
impl Down {
    pub fn kind(&self) -> wire::Kind {
        match self {
            Self::Pause(_) => wire::Kind::Pause,
            Self::Restarting(_) => wire::Kind::Restarting,
            Self::Asked(_) => wire::Kind::Asked,
            Self::ReadUsage(_) => wire::Kind::ReadUsage,
            Self::ReadList(_) => wire::Kind::ReadList,
            Self::Sources(_) => wire::Kind::Sources,
            Self::Check(_) => wire::Kind::Check,
            Self::SignOut(_) => wire::Kind::SignOut,
            Self::Assign(_) => wire::Kind::Assignment,
            Self::AssignJob(_) => wire::Kind::Job,
            Self::Message(_) => wire::Kind::Message,
            Self::Relayed(_) => wire::Kind::Relayed,
            Self::PartKept(_) => wire::Kind::PartKept,
            Self::PartBusy(_) => wire::Kind::PartBusy,
            Self::TurnKept(_) => wire::Kind::TurnKept,
            Self::TurnBusy(_) => wire::Kind::TurnBusy,
            Self::Acknowledge(_) => wire::Kind::Acknowledge,
            Self::Cancel(_) => wire::Kind::Cancel,
            Self::Grant(_) => wire::Kind::Grant,
        }
    }
    pub fn frame(self, limits: &wire::v22::Limits) -> Result<skein_channel::Frame, crate::Unencoded> {
        let (kind, length) = match &self {
            Self::Pause(record) => (wire::Kind::Pause.number(), record.measure(limits)),
            Self::Restarting(record) => (wire::Kind::Restarting.number(), record.measure(limits)),
            Self::Asked(record) => (wire::Kind::Asked.number(), record.measure(limits)),
            Self::ReadUsage(record) => (wire::Kind::ReadUsage.number(), record.measure(limits)),
            Self::ReadList(record) => (wire::Kind::ReadList.number(), record.measure(limits)),
            Self::Sources(record) => (0x0188, record.measure(limits)),
            Self::Check(record) => (0x0189, record.measure(limits)),
            Self::SignOut(record) => (0x018a, record.measure(limits)),
            Self::Assign(record) => (0x0180, record.measure(limits)),
            Self::AssignJob(record) => (wire::Kind::Job.number(), record.measure(limits)),
            Self::Message(record) => (0x0181, record.measure(limits)),
            Self::Relayed(record) => (0x0182, record.measure(limits)),
            Self::PartKept(record) => (wire::Kind::PartKept.number(), record.measure(limits)),
            Self::PartBusy(record) => (wire::Kind::PartBusy.number(), record.measure(limits)),
            Self::TurnKept(record) => (0x0183, record.measure(limits)),
            Self::TurnBusy(record) => (0x0184, record.measure(limits)),
            Self::Acknowledge(record) => (0x0185, record.measure(limits)),
            Self::Cancel(record) => (0x0186, record.measure(limits)),
            Self::Grant(record) => (0x0187, record.measure(limits)),
        };
        let length = match length {
            Ok(length) => length,
            Err(why) => return Err(problem(why)),
        };
        let mut writer = skein_channel::frame_writer(kind, length).expect("checked record fits a frame");
        let mut body = skein_lib::Writer::new(skein_lib::length::usize_of(length));
        let encoded = match self {
            Self::Pause(record) => record.write(limits, &mut body),
            Self::Restarting(record) => record.write(limits, &mut body),
            Self::Asked(record) => record.write(limits, &mut body),
            Self::ReadUsage(record) => record.write(limits, &mut body),
            Self::ReadList(record) => record.write(limits, &mut body),
            Self::Sources(record) => record.write(limits, &mut body),
            Self::Check(record) => record.write(limits, &mut body),
            Self::SignOut(record) => record.write(limits, &mut body),
            Self::Assign(record) => record.write(limits, &mut body),
            Self::AssignJob(record) => record.write(limits, &mut body),
            Self::Message(record) => record.write(limits, &mut body),
            Self::Relayed(record) => record.write(limits, &mut body),
            Self::PartKept(record) => record.write(limits, &mut body),
            Self::PartBusy(record) => record.write(limits, &mut body),
            Self::TurnKept(record) => record.write(limits, &mut body),
            Self::TurnBusy(record) => record.write(limits, &mut body),
            Self::Acknowledge(record) => record.write(limits, &mut body),
            Self::Cancel(record) => record.write(limits, &mut body),
            Self::Grant(record) => record.write(limits, &mut body),
        };
        encoded.expect("measured record");
        writer.put(&body.finish()).expect("measured body fits frame");
        Ok(writer.finish().expect("complete record"))
    }
}
pub(crate) fn down(
    down: host::LinkDown,
    parts: &[wire::Part],
    values: Box<[Option<crate::GrantValue>]>,
    limits: &wire::v22::Limits,
) -> Result<Down, crate::Unencoded> {
    match down {
        host::LinkDown::Pause { mode } => {
            let mode = mapped::pause_mode_to_record(&mode);
            Ok(Down::Pause(wire::Pause { mode }))
        }
        host::LinkDown::Asked { run, attempt, number, answer } => {
            let answer = mapped::ask_answer_to_record(&answer);
            Ok(Down::Asked(wire::Asked {
                run: token_to_record(&run),
                attempt: token_to_record(&attempt),
                number,
                answer,
            }))
        }
        host::LinkDown::ReadUsage { read, source } => {
            Ok(Down::ReadUsage(wire::ReadUsage { read, source: source_name_to_record(&source) }))
        }
        host::LinkDown::ReadList { read, endpoint, grant } => {
            match grant.form {
                Form::Credential(_) => {}
                Form::Application | Form::Program(_) => return Err(crate::Unencoded::field("grant.value")),
            }
            let grant = grant_record(
                grant,
                values.into_iter().next().flatten().ok_or(crate::Unencoded::field("grant.value"))?,
            )?;
            Ok(Down::ReadList(wire::ReadList { read, endpoint, grant }))
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
                wire::v22::CEILINGS.sources_sources,
            )?;
            let mut names = List::with_capacity(u32::try_from(sources.len()).expect("measured sources count fits"));
            for name in sources {
                names.push(wire::ProgramSource { name: source_name_to_record(&name) }).expect("measured sources count");
            }
            Ok(Down::Sources(wire::Sources { sources: names }))
        }
        host::LinkDown::Check { name } => Ok(Down::Check(wire::Check { name: source_name_to_record(&name) })),
        host::LinkDown::SignOut { name } => Ok(Down::SignOut(wire::SignOut { name: source_name_to_record(&name) })),
        host::LinkDown::Assign { assignment } => {
            Ok(Down::Assign(assignment_record(assignment, parts, values, limits)?))
        }
        host::LinkDown::Message { run, attempt, name, sender, words, finish_requested } => {
            Ok(Down::Message(wire::Inbound {
                task: token_to_record(&run),
                attempt: token_to_record(&attempt),
                name: token_to_record(&name),
                sender,
                words,
                finish_requested,
            }))
        }
        host::LinkDown::Relayed { run, attempt, call, answer } => {
            let answer = mapped::relay_answer_to_record(&answer);
            Ok(Down::Relayed(wire::Relayed {
                task: token_to_record(&run),
                attempt: token_to_record(&attempt),
                name: call,
                answer,
            }))
        }
        host::LinkDown::PartKept { run, attempt, part } => Ok(Down::PartKept(wire::PartKept {
            task: token_to_record(&run),
            attempt: token_to_record(&attempt),
            number: part,
        })),
        host::LinkDown::TurnKept { run, attempt, turn } => Ok(Down::TurnKept(wire::TurnKept {
            task: token_to_record(&run),
            attempt: token_to_record(&attempt),
            number: turn,
        })),
        host::LinkDown::Acknowledge { run, attempt } => {
            Ok(Down::Acknowledge(wire::Acknowledge { task: token_to_record(&run), attempt: token_to_record(&attempt) }))
        }
        host::LinkDown::Cancel { run, attempt } => {
            Ok(Down::Cancel(wire::Cancel { task: token_to_record(&run), attempt: token_to_record(&attempt) }))
        }
        host::LinkDown::Grant { run, attempt, grant } => {
            let value = values.into_iter().next().flatten().ok_or(crate::Unencoded::field("grant.value"))?;
            let grant = grant_record(grant, value)?;
            Ok(Down::Grant(wire::Granted { task: token_to_record(&run), attempt: token_to_record(&attempt), grant }))
        }
    }
}
fn job_record(
    job: host::JobAssignment,
    parts: &[wire::Part],
    values: Box<[Option<crate::GrantValue>]>,
    limits: &wire::v22::Limits,
) -> Result<wire::Job, crate::Unencoded> {
    let workspace = workspace_record(job.workspace, parts)?;
    let grants = grants_record(job.grants, values, "Job.grants", limits.job_grants, wire::v22::CEILINGS.job_grants)?;
    Ok(wire::Job {
        task: token_to_record(&job.run),
        attempt: token_to_record(&job.attempt),
        kind: job_kind_to_record(job.kind),
        spec: job.spec,
        deadline: job.deadline,
        workspace,
        grants,
    })
}
fn assignment_record(
    assignment: host::Assignment,
    parts: &[wire::Part],
    values: Box<[Option<crate::GrantValue>]>,
    limits: &wire::v22::Limits,
) -> Result<wire::Assignment, crate::Unencoded> {
    let run = assignment.assignment;
    let workspace = workspace_record(run.workspace, parts)?;
    measured(
        "Assignment.messages",
        skein_lib::length::len_u64(assignment.messages.len()),
        limits.assignment_messages,
        wire::v22::CEILINGS.assignment_messages,
    )?;
    let mut messages =
        List::with_capacity(u32::try_from(assignment.messages.len()).expect("measured messages count fits"));
    for message in assignment.messages {
        messages.push(mapped::message_to_record(&message)).expect("measured messages count");
    }
    measured(
        "Assignment.answered",
        skein_lib::length::len_u64(assignment.answered.len()),
        limits.assignment_answered,
        wire::v22::CEILINGS.assignment_answered,
    )?;
    let mut answered =
        List::with_capacity(u32::try_from(assignment.answered.len()).expect("measured answered count fits"));
    for call in assignment.answered {
        answered.push(mapped::answered_call_to_record(&call)).expect("measured answered count");
    }
    let grants = grants_record(
        run.grants,
        values,
        "Assignment.grants",
        limits.assignment_grants,
        wire::v22::CEILINGS.assignment_grants,
    )?;
    measured(
        "Assignment.charter",
        skein_lib::length::len_u64(run.charter.len()),
        limits.assignment_charter,
        wire::v22::CEILINGS.assignment_charter,
    )?;
    let transcript = transcript(&assignment.turns, limits.assignment_transcript)?;
    let mut kept_parts = List::with_capacity(u32::try_from(assignment.parts.len()).expect("bounded kept parts"));
    for part in assignment.parts {
        kept_parts.push(mapped::kept_part_to_record(&part)).expect("one slot per kept part");
    }
    let kept_parts = match wire::parts::encode(kept_parts.as_slice(), limits) {
        Ok(parts) => parts,
        Err(error) => return Err(problem(error)),
    };
    Ok(wire::Assignment {
        parts: kept_parts,
        task: token_to_record(&run.run),
        attempt: token_to_record(&run.attempt),
        identity: run.identity,
        workspace,
        save: run.save,
        live: mapped::live_ask_to_record(&run.live),
        charter: run.charter,
        transcript,
        answered,
        messages,
        grants,
    })
}
pub(crate) fn problem(problem: wire::v22::Problem) -> crate::Unencoded {
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
            records.push(grant_record(grant, value)?).expect("measured grants count");
        }
    }
    Ok(records)
}
fn workspace_record(
    workspace: Option<host::Workspace>,
    parts: &[wire::Part],
) -> Result<Option<wire::Workspace>, crate::Unencoded> {
    match workspace {
        Some(workspace) => {
            Ok(Some(wire::Workspace { workstream: workspace.workstream, items: part(workspace.items, parts)? }))
        }
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
    measured("Assignment.transcript", length, bound, wire::v22::CEILINGS.assignment_transcript)?;
    let mut writer = skein_lib::Writer::new(usize::try_from(length).expect("bounded framed transcript fits usize"));
    for turn in turns {
        skein_codec::piece::write(&mut writer, turn).expect("measured framed transcript fits");
    }
    Ok(writer.finish())
}
/// A grant's form and its value's arm agree, or the grant is not sent.
fn grant_record(grant: jig_domain_link::Grant, value: crate::GrantValue) -> Result<wire::Grant, crate::Unencoded> {
    let value = match grant.form {
        Form::Program(form) => match value {
            crate::GrantValue::Program { name } => {
                if name != form {
                    return Err(crate::Unencoded::field("grant.value"));
                }
                wire::GrantValue::Program(wire::ProgramValue { name: source_name_to_record(&name) })
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
                let scheme = mapped::scheme_to_record(&form);
                wire::GrantValue::Credential(wire::CredentialValue { scheme, bytes: token })
            }
            crate::GrantValue::Application { .. } | crate::GrantValue::Program { .. } => {
                return Err(crate::Unencoded::field("grant.value"));
            }
        },
        Form::Application => match value {
            crate::GrantValue::Application { secret } => {
                wire::GrantValue::Application(wire::ApplicationValue { secret })
            }
            crate::GrantValue::Credential { .. } | crate::GrantValue::Program { .. } => {
                return Err(crate::Unencoded::field("grant.value"));
            }
        },
    };
    Ok(wire::Grant { number: grant.account, generation: grant.generation, valid: grant.valid, value })
}
const fn grant_scheme(scheme: Scheme) -> crate::Scheme {
    match scheme {
        Scheme::Bearer => crate::Scheme::Bearer,
        Scheme::Key => crate::Scheme::Key,
    }
}

pub(crate) fn turn_busy(run: Token, attempt: Token, turn: u32) -> Down {
    Down::TurnBusy(wire::TurnBusy { task: token_to_record(&run), attempt: token_to_record(&attempt), number: turn })
}

fn usage_window(
    label: Box<[u8]>,
    length: Option<Duration>,
    used: Option<u16>,
    resets: Option<u64>,
    state: u16,
) -> Option<jig_domain_link::UsageWindow> {
    if label.is_empty() || used > Some(10_000) {
        return None;
    }
    for byte in &label {
        if !byte.is_ascii() || byte.is_ascii_control() {
            return None;
        }
    }
    let resets = reset_wall(resets);
    Some(jig_domain_link::UsageWindow { label, window: jig_domain_link::FactWindow { length, used, resets, state } })
}
pub(super) fn usage_windows_to_domain(
    record: wire::UsageWindows,
) -> Result<Box<[jig_domain_link::UsageWindow]>, wire::v22::Problem> {
    let path = wire::v22::paths::USAGE_WINDOWS_WINDOWS;
    let mut rows = List::with_capacity(jig_domain_link::USAGE_WINDOWS);
    for row in record.windows.into_boxed() {
        let window = usage_window(row.label, row.length, row.used, row.resets, row.state)
            .ok_or(path.problem(skein_codec::Reason::Malformed))?;
        rows.push(window).or(Err(path.problem(skein_codec::Reason::Bound)))?;
    }
    Ok(rows.into_boxed())
}

fn reset_wall(reset: Option<u64>) -> Option<skein_lib::Wall> {
    Some(wall_to_domain(reset?))
}
