//! The hub's records across the channel (protocol/channel/records.md and protocol/channel/owners.md, 1).
//! Jobs use their own records (jig/docs/design/protocol/channel/records.md, 4).
//! Pieces cross whole; only the root judges their kinds and offsets (6.4).
use crate::generated::worker_wire as mapped;
use crate::{GrantValue, Request, Scheme};
use alloc::boxed::Box;
use jig_domain_host as host;
use jig_domain_link::{
    Form, Grant, Hello, JobKind, Login, LoginMethod, SignOutResult, SourceName, UsageOutcome, UsageWindow,
    valid_program_endpoints,
};
use jig_protocol_worker_wire::{self as wire, Part, Record};
use skein_channel::{Frame, frame_writer};
use skein_codec::convert::{token_to_domain, token_to_record, wall_to_record};
use skein_lib::{Duration, List, Queue, Reader, Time, Token, bytes};

#[expect(clippy::trivially_copy_pass_by_ref, reason = "the generated caller borrows the domain field")]
pub(super) fn job_kind_to_record(kind: &JobKind) -> u32 {
    kind.0
}

fn job_kind_to_domain(kind: u32) -> JobKind {
    JobKind(kind)
}

pub(super) fn source_name_to_record(name: &SourceName) -> Box<[u8]> {
    Box::from(name.as_bytes())
}

fn source_name_to_domain(name: Box<[u8]>) -> Result<SourceName, skein_codec::Reason> {
    SourceName::new(&name).ok_or(skein_codec::Reason::Malformed)
}

pub(super) fn login_method_to_record(method: &LoginMethod) -> Box<[u8]> {
    Box::from(method.as_bytes())
}

pub(crate) fn wait_ms_to_domain(wait: u64) -> Duration {
    Duration::from_millis(wait)
}

fn count_fits(len: usize, limit: u32) -> bool {
    len <= skein_lib::usize_of(limit)
}

pub(crate) fn hello(hello: Hello, limits: &wire::v22::Limits) -> Option<Frame> {
    if !valid_program_endpoints(&hello.bounds.program_endpoints)
        || !count_fits(hello.bounds.program_endpoints.len(), limits.bounds_program_endpoints)
        || !count_fits(hello.bounds.endpoints.len(), limits.bounds_endpoints)
        || !count_fits(hello.workstreams.len(), limits.hello_workstreams)
        || !count_fits(hello.hosting.len(), limits.hello_hosting)
        || !count_fits(hello.jobs.len(), limits.hello_jobs)
    {
        return None;
    }
    let bounds = mapped::bounds_to_record(&hello.bounds).ok()?;
    let mut hosting = List::with_capacity(limits.hello_hosting);
    for run in hello.hosting {
        hosting.push(mapped::hosting_to_record(&run)).ok()?;
    }
    let mut jobs = List::with_capacity(limits.hello_jobs);
    for job in hello.jobs {
        jobs.push(mapped::job_bound_to_record(&job)).ok()?;
    }
    let record = wire::Hello {
        slots: hello.slots,
        stop_bound: hello.stop_bound,
        bounds,
        workstreams: List::from_boxed(hello.workstreams),
        hosting,
        jobs,
        fresh: hello.fresh,
    };
    frame(0x0100, record.encode(limits).ok()?)
}

pub(crate) fn up(up: host::LinkUp, parts: &[Part], limits: &wire::v22::Limits) -> Option<Frame> {
    match up {
        host::LinkUp::Ask { run, attempt, number, bytes } => {
            let record = wire::Ask { run: token_to_record(&run), attempt: token_to_record(&attempt), number, bytes };
            frame(wire::Kind::Ask.number(), record.encode(limits).ok()?)
        }
        host::LinkUp::Paused { state } => {
            let state = mapped::pause_state_to_record(&state);
            let record = wire::Paused { state };
            frame(wire::Kind::Paused.number(), record.encode(limits).ok()?)
        }
        host::LinkUp::Draining => {
            let record = wire::Draining {};
            frame(wire::Kind::Draining.number(), record.encode(limits).ok()?)
        }
        host::LinkUp::Usage { run, attempt, account, window } => {
            let resets = reset_nanos(window.window.resets);
            let record = wire::Usage {
                task: token_to_record(&run),
                attempt: token_to_record(&attempt),
                account,
                label: window.label,
                length: window.window.length,
                used: window.window.used,
                resets,
                state: window.window.state,
            };
            frame(wire::Kind::Usage.number(), record.encode(limits).ok()?)
        }
        host::LinkUp::UsageRead { read, outcome } => usage_read_frame(read, outcome, limits),
        host::LinkUp::ListRead { read, outcome } => list_read_frame(read, outcome, limits),
        host::LinkUp::JobAnswer { run, attempt, answer } => job_answer_frame(run, attempt, answer, parts, limits),

        host::LinkUp::Piece { run, attempt, piece } => {
            let value = wire::FactValue::Piece(mapped::piece_to_record(&piece));
            fact_frame(run, attempt, value, limits)
        }
        host::LinkUp::Logins { logins } => logins_frame(logins, limits),
        host::LinkUp::SignedOut { name, result } => signed_out_frame(name, result, limits),
        host::LinkUp::Activity { run, attempt, activity: value } => fact_frame(run, attempt, activity(value), limits),
        host::LinkUp::Unauthorized { run, attempt, account, generation } => {
            let record = wire::Unauthorized {
                task: token_to_record(&run),
                attempt: token_to_record(&attempt),
                account,
                generation,
            };
            frame(0x0105, record.encode(limits).ok()?)
        }
        host::LinkUp::Limited { run, attempt, account, reset } => {
            let record =
                wire::Limited { task: token_to_record(&run), attempt: token_to_record(&attempt), account, reset };
            frame(0x0106, record.encode(limits).ok()?)
        }
        host::LinkUp::AccountRefused { run, attempt, account, code, reason } => {
            let record = wire::AccountRefused {
                task: token_to_record(&run),
                attempt: token_to_record(&attempt),
                account,
                code,
                reason,
            };
            frame(0x0107, record.encode(limits).ok()?)
        }
        host::LinkUp::Part { run, attempt, part } => part_frame(run, attempt, part, limits),
        host::LinkUp::Turn { run, attempt, turn } => {
            let record = wire::Turn {
                task: token_to_record(&run),
                attempt: token_to_record(&attempt),
                number: turn.turn,
                spent: turn.spent,
                read: name(turn.read),
                opens_window: turn.opens_window,
                refused: refused(turn.refused, limits)?,
                body: turn.body,
            };
            frame(0x0101, record.encode(limits).ok()?)
        }
        host::LinkUp::Call { run, attempt, contract, call, tool, writes, input, deadline } => {
            let record = wire::Call {
                task: token_to_record(&run),
                attempt: token_to_record(&attempt),
                name: call,
                tool,
                writes,
                input,
                deadline,
                contract,
            };
            frame(0x0102, record.encode(limits).ok()?)
        }
        host::LinkUp::Answer { run, attempt, answer } => {
            let record = wire::Answer {
                task: token_to_record(&run),
                attempt: token_to_record(&attempt),
                turns: answer.turns,
                spent: answer.spent,
                read: name(answer.read),
                refused: refused(answer.refused, limits)?,
                ending: ending(answer.ending, parts)?,
            };
            frame(0x0103, record.encode(limits).ok()?)
        }
    }
}
fn job_answer_frame(
    run: Token,
    attempt: Token,
    answer: host::JobEnding,
    parts: &[Part],
    limits: &wire::v22::Limits,
) -> Option<Frame> {
    let record = wire::JobAnswer {
        task: token_to_record(&run),
        attempt: token_to_record(&attempt),
        ending: job_ending(answer, parts)?,
    };
    frame(wire::Kind::JobAnswer.number(), record.encode(limits).ok()?)
}
fn fact_frame(run: Token, attempt: Token, value: wire::FactValue, limits: &wire::v22::Limits) -> Option<Frame> {
    let record = wire::Fact { task: token_to_record(&run), attempt: token_to_record(&attempt), value };
    frame(wire::Kind::Fact.number(), record.encode(limits).ok()?)
}
fn frame(kind: u16, body: Box<[u8]>) -> Option<Frame> {
    let mut frame = frame_writer(kind, u32::try_from(body.len()).ok()?).ok()?;
    frame.put(&body).ok()?;
    frame.finish().ok()
}
fn part(token: Option<Token>, parts: &[Part]) -> Result<Option<Box<[u8]>>, ()> {
    let Some(token) = token else { return Ok(None) };
    for part in parts {
        if part.token == token {
            return Ok(Some(bytes::copy_of(&part.bytes)));
        }
    }
    Err(())
}
fn name(token: Option<Token>) -> Option<u64> {
    Some(token_to_record(&token?))
}
fn work(work: host::Work, parts: &[Part]) -> Option<wire::Work> {
    Some(wire::Work { left: part(work.left, parts).ok()?, saved: part(work.saved, parts).ok()? })
}
fn refused(items: Box<[host::Refused]>, limits: &wire::v22::Limits) -> Option<List<wire::Refused>> {
    if !count_fits(items.len(), limits.turn_refused) {
        return None;
    }
    let mut list = List::with_capacity(limits.turn_refused);
    for item in items {
        list.push(mapped::refused_to_record(&item)).ok()?;
    }
    Some(list)
}
#[expect(clippy::manual_map, reason = "step code keeps the optional model translation explicit")]
fn ending(ending: host::Ending, parts: &[Part]) -> Option<wire::Ending> {
    Some(match ending {
        host::Ending::Refused(refusal) => {
            let value = match refusal {
                host::Refusal::Untranslated(_) => return None,
                host::Refusal::Busy => wire::Refusal::Busy,
                host::Refusal::Version(value) => wire::Refusal::Version(mapped::mismatch_to_record(&value)),
                host::Refusal::Invalid(invalid) => {
                    let value = mapped::invalid_to_record(&invalid);
                    let value = wire::InvalidValue { value };
                    wire::Refusal::Invalid(value)
                }
            };
            let value = wire::RefusalValue { value };
            wire::Ending::Refused(value)
        }
        host::Ending::Ended { outcome, work: kept } => {
            let work = work(kept, parts)?;
            wire::Ending::Ended(wire::Ended { outcome, work })
        }
        host::Ending::Parked { work: kept } => {
            let work = work(kept, parts)?;
            wire::Ending::Parked(wire::Parked { work })
        }
        host::Ending::Failed { failure: failed, detail, work: kept, model } => {
            let work = work(kept, parts)?;
            let failure = mapped::failure_to_record(&failed, parts).ok()?;
            let model = match model {
                Some(model) => Some(mapped::model_failure_to_record(&model)),
                None => None,
            };
            wire::Ending::Failed(wire::Failed { failure, detail, work, model })
        }
    })
}

fn job_ending(ending: host::JobEnding, parts: &[Part]) -> Option<wire::JobEnding> {
    Some(match ending {
        host::JobEnding::Refused(refusal) => {
            let value = match refusal {
                host::JobRefusal::Untranslated(_) => return None,
                host::JobRefusal::Busy => wire::JobRefusal::Busy,
                host::JobRefusal::Invalid(invalid) => {
                    let value = mapped::job_invalid_to_record(&invalid);
                    wire::JobRefusal::Invalid(wire::JobInvalidValue { value })
                }
            };
            wire::JobEnding::Refused(wire::JobRefusalValue { value })
        }
        host::JobEnding::Finished { result, work: kept } => {
            wire::JobEnding::Finished(wire::JobFinished { result, work: work(kept, parts)? })
        }
        host::JobEnding::Failed { failure, detail, work: kept } => wire::JobEnding::Failed(wire::JobFailed {
            failure: mapped::job_failure_to_record(&failure, parts).ok()?,
            detail,
            work: work(kept, parts)?,
        }),
    })
}
pub(super) fn preparation_to_record(
    value: &host::Preparation,
    parts: &[Part],
) -> Result<wire::Preparation, wire::v22::Problem> {
    match value {
        host::Preparation::Transient => Ok(wire::Preparation::Transient),
        host::Preparation::Permanent { resource } => {
            let resource = part(*resource, parts)
                .or(Err(wire::v22::paths::RESOURCE_RESOURCE.problem(skein_codec::Reason::Malformed)))?;
            Ok(wire::Preparation::Permanent(wire::Resource { resource }))
        }
    }
}
fn activity(activity: host::AgentActivity) -> wire::FactValue {
    match activity {
        host::AgentActivity::Phase(value) => {
            let value = mapped::phase_to_record(&value);
            wire::FactValue::Phase(wire::PhaseValue { value })
        }
        host::AgentActivity::Long { operation: op, span } => {
            let operation = mapped::operation_to_record(&op);
            wire::FactValue::Long(wire::Long { operation, span })
        }
        host::AgentActivity::LongDone => wire::FactValue::LongDone,
        host::AgentActivity::Count { kind, elapsed, count } => {
            let kind = mapped::count_kind_to_record(&kind);
            wire::FactValue::Count(wire::Count { kind, elapsed, value: count })
        }
    }
}

#[expect(clippy::too_many_lines, reason = "one exhaustive match translates the worker's down records")]
pub(crate) fn down(
    kind: u16,
    body: &[u8],
    limits: &wire::v22::Limits,
    now: Time,
    parts: &mut Queue<Part>,
    next: &mut u64,
    out: &mut Queue<Request>,
) -> Option<host::LinkDown> {
    let mut reader = Reader::new(body);
    let mut known = None;
    for rule in wire::KINDS {
        if rule.kind == kind {
            known = Some(rule.body);
        }
    }
    let down = match known? {
        wire::Kind::Pause => {
            let record = wire::Pause::decode(limits, &mut reader).ok()?;
            let mode = mapped::pause_mode_to_domain(record.mode);
            host::LinkDown::Pause { mode }
        }
        wire::Kind::Asked => {
            let record = wire::Asked::decode(limits, &mut reader).ok()?;
            let answer = mapped::ask_answer_to_domain(record.answer);
            host::LinkDown::Asked {
                run: token_to_domain(record.run),
                attempt: token_to_domain(record.attempt),
                number: record.number,
                answer,
            }
        }
        wire::Kind::ReadUsage => {
            let record = wire::ReadUsage::decode(limits, &mut reader).ok()?;
            host::LinkDown::ReadUsage { read: record.read, source: source_name_to_domain(record.source).ok()? }
        }
        wire::Kind::ReadList => {
            let record = wire::ReadList::decode(limits, &mut reader).ok()?;
            match &record.grant.value {
                wire::GrantValue::Credential(_) => {}
                wire::GrantValue::Application(_) | wire::GrantValue::Program(_) => return None,
            }
            host::LinkDown::ReadList {
                read: record.read,
                endpoint: record.endpoint,
                grant: grant(record.grant, now, out)?,
            }
        }
        wire::Kind::Sources => {
            let record = wire::Sources::decode(limits, &mut reader).ok()?;
            let mut names = List::with_capacity(record.sources.len());
            for source in record.sources.into_boxed() {
                let name = source_name_to_domain(source.name).ok()?;
                names.push(name).ok()?;
            }
            if !host::sources::unique(names.as_slice()) {
                return None;
            }
            host::LinkDown::Sources { sources: names.into_boxed() }
        }
        wire::Kind::Check => host::LinkDown::Check {
            name: source_name_to_domain(wire::Check::decode(limits, &mut reader).ok()?.name).ok()?,
        },
        wire::Kind::SignOut => host::LinkDown::SignOut {
            name: source_name_to_domain(wire::SignOut::decode(limits, &mut reader).ok()?.name).ok()?,
        },
        wire::Kind::Job => {
            let record = wire::Job::decode(limits, &mut reader).ok()?;
            host::LinkDown::AssignJob { job: job_assignment(record, now, parts, next, out)? }
        }
        wire::Kind::Assignment => {
            let record = wire::Assignment::decode(limits, &mut reader).ok()?;
            host::LinkDown::Assign { assignment: assignment(record, now, parts, next, out, limits)? }
        }
        wire::Kind::Message => {
            let record = wire::Inbound::decode(limits, &mut reader).ok()?;
            host::LinkDown::Message {
                run: token_to_domain(record.task),
                attempt: token_to_domain(record.attempt),
                name: token_to_domain(record.name),
                sender: record.sender,
                words: record.words,
                finish_requested: record.finish_requested,
            }
        }
        wire::Kind::PartKept => {
            let record = wire::PartKept::decode(limits, &mut reader).ok()?;
            host::LinkDown::PartKept {
                run: token_to_domain(record.task),
                attempt: token_to_domain(record.attempt),
                part: record.number,
            }
        }
        wire::Kind::TurnKept => {
            let record = wire::TurnKept::decode(limits, &mut reader).ok()?;
            host::LinkDown::TurnKept {
                run: token_to_domain(record.task),
                attempt: token_to_domain(record.attempt),
                turn: record.number,
            }
        }
        wire::Kind::Acknowledge => {
            let record = wire::Acknowledge::decode(limits, &mut reader).ok()?;
            host::LinkDown::Acknowledge { run: token_to_domain(record.task), attempt: token_to_domain(record.attempt) }
        }
        wire::Kind::Cancel => {
            let record = wire::Cancel::decode(limits, &mut reader).ok()?;
            host::LinkDown::Cancel { run: token_to_domain(record.task), attempt: token_to_domain(record.attempt) }
        }
        wire::Kind::Grant => {
            let record = wire::Granted::decode(limits, &mut reader).ok()?;
            let grant = grant(record.grant, now, out)?;
            host::LinkDown::Grant { run: token_to_domain(record.task), attempt: token_to_domain(record.attempt), grant }
        }
        wire::Kind::Relayed => {
            let record = wire::Relayed::decode(limits, &mut reader).ok()?;
            let answer = mapped::relay_answer_to_domain(record.answer);
            host::LinkDown::Relayed {
                run: token_to_domain(record.task),
                attempt: token_to_domain(record.attempt),
                call: record.name,
                answer,
            }
        }
        // TurnBusy is decoded below the hub by the component; up kinds are invalid here.
        wire::Kind::Part
        | wire::Kind::PartBusy
        | wire::Kind::Paused
        | wire::Kind::Draining
        | wire::Kind::Usage
        | wire::Kind::UsageRead
        | wire::Kind::ListRead
        | wire::Kind::JobAnswer
        | wire::Kind::Logins
        | wire::Kind::SignedOut
        | wire::Kind::TurnBusy
        | wire::Kind::Hello
        | wire::Kind::Turn
        | wire::Kind::Call
        | wire::Kind::Answer
        | wire::Kind::Fact
        | wire::Kind::Unauthorized
        | wire::Kind::Limited
        | wire::Kind::AccountRefused
        | wire::Kind::Ask
        | wire::Kind::Figures
        | wire::Kind::Restarting
        | wire::Kind::Wires => return None,
    };
    Some(down)
}
/// A value's arm and scheme name the form in which it is lent.
fn grant_form(value: &wire::GrantValue) -> Option<Form> {
    Some(match value {
        wire::GrantValue::Program(value) => Form::Program(SourceName::new(&value.name)?),
        wire::GrantValue::Credential(value) => Form::Credential(mapped::scheme_to_domain(value.scheme.clone())),
        wire::GrantValue::Application(_) => Form::Application,
    })
}
fn grant(grant: wire::Grant, now: Time, out: &mut Queue<Request>) -> Option<Grant> {
    let form = grant_form(&grant.value)?;
    let value = match grant.value {
        wire::GrantValue::Program(_) => {
            return Some(Grant { form, account: grant.number, generation: grant.generation, valid: grant.valid });
        }
        wire::GrantValue::Credential(value) => {
            let scheme = match mapped::scheme_to_domain(value.scheme) {
                jig_domain_link::Scheme::Bearer => Scheme::Bearer,
                jig_domain_link::Scheme::Key => Scheme::Key,
            };
            GrantValue::Credential { scheme, token: value.bytes }
        }
        wire::GrantValue::Application(value) => GrantValue::Application { secret: value.secret },
    };
    let expires = now.saturating_add(grant.valid);
    out.push(Request::Value { account: grant.number, generation: grant.generation, expires, value });
    Some(Grant { form, account: grant.number, generation: grant.generation, valid: grant.valid })
}
fn transcript(bytes: &[u8]) -> Box<[Box<[u8]>]> {
    let mut reader = Reader::new(bytes);
    let mut turns = List::with_capacity(u32::try_from(bytes.len().saturating_div(4)).expect("bounded transcript"));
    for _ in 0..bytes.len().saturating_div(4).saturating_add(1) {
        match skein_codec::piece::split(&mut reader) {
            Ok(Some(turn)) => turns.push(turn).expect("four-byte prefix per turn"),
            Ok(None) => return turns.into_boxed(),
            Err(_) => return Box::new([Box::new([])]),
        }
    }
    turns.into_boxed()
}

fn job_assignment(
    record: wire::Job,
    now: Time,
    parts: &mut Queue<Part>,
    next: &mut u64,
    out: &mut Queue<Request>,
) -> Option<host::JobAssignment> {
    for granted in &record.grants {
        grant_form(&granted.value)?;
    }
    let workspace = match record.workspace {
        Some(workspace) => {
            let items = Token::new(*next);
            *next = next.checked_add(1).expect("part tokens never wrap");
            parts.push(Part { token: items, bytes: workspace.items });
            Some(host::Workspace { workstream: workspace.workstream, items })
        }
        None => None,
    };
    let mut grants = List::with_capacity(record.grants.len());
    for granted in record.grants.into_boxed() {
        grants.push(grant(granted, now, out)?).expect("one slot per grant");
    }
    Some(host::JobAssignment {
        run: token_to_domain(record.task),
        attempt: token_to_domain(record.attempt),
        kind: job_kind_to_domain(record.kind),
        spec: record.spec,
        deadline: record.deadline,
        workspace,
        grants: grants.into_boxed(),
    })
}

fn assignment(
    record: wire::Assignment,
    now: Time,
    parts: &mut Queue<Part>,
    next: &mut u64,
    out: &mut Queue<Request>,
    limits: &wire::v22::Limits,
) -> Option<host::Assignment> {
    let kept = wire::parts::decode(&record.parts, limits).ok()?;
    let mut kept_parts = List::with_capacity(kept.len());
    for part in kept.into_boxed() {
        kept_parts.push(mapped::kept_part_to_domain(part)).expect("one slot per kept part");
    }
    // Decode every form before emitting values or parts. A second bounded pass
    // avoids staging credential copies or another output queue.
    for granted in &record.grants {
        grant_form(&granted.value)?;
    }

    let workspace = match record.workspace {
        Some(workspace) => {
            let items = Token::new(*next);
            *next = next.checked_add(1).expect("part tokens never wrap");
            parts.push(Part { token: items, bytes: workspace.items });
            Some(host::Workspace { workstream: workspace.workstream, items })
        }
        None => None,
    };
    let mut messages = List::with_capacity(record.messages.len());
    for message in record.messages.into_boxed() {
        messages.push(mapped::message_to_domain(message)).expect("one slot per message");
    }
    let mut answered = List::with_capacity(record.answered.len());
    for call in record.answered.into_boxed() {
        answered.push(mapped::answered_call_to_domain(call)).expect("one slot per call");
    }
    let turns = transcript(&record.transcript);
    let mut grants = List::with_capacity(record.grants.len());
    for granted in record.grants.into_boxed() {
        let granted = grant(granted, now, out)?;
        grants.push(granted).expect("one slot per grant");
    }
    Some(host::Assignment {
        assignment: host::RunAssignment {
            identity: record.identity,
            run: token_to_domain(record.task),
            attempt: token_to_domain(record.attempt),
            workspace,
            save: record.save,
            live: mapped::live_ask_to_domain(record.live),
            charter: record.charter,
            grants: grants.into_boxed(),
        },
        messages: messages.into_boxed(),
        answered: answered.into_boxed(),
        turns,
        parts: kept_parts.into_boxed(),
        tell_parts: false,
    })
}

fn logins_frame(logins: Box<[Login]>, limits: &wire::v22::Limits) -> Option<Frame> {
    if !count_fits(logins.len(), limits.logins_logins) {
        return None;
    }
    let mut rows = List::<wire::Login>::with_capacity(limits.logins_logins);
    for login in logins {
        for row in &rows {
            if row.name.as_ref() == login.name.as_bytes() {
                return None;
            }
        }
        rows.push(mapped::login_to_record(&login)).ok()?;
    }
    let record = wire::Logins { logins: rows };
    frame(wire::Kind::Logins.number(), record.encode(limits).ok()?)
}
fn signed_out_frame(name: SourceName, result: SignOutResult, limits: &wire::v22::Limits) -> Option<Frame> {
    let record =
        wire::SignedOut { name: source_name_to_record(&name), result: mapped::sign_out_result_to_record(&result) };
    frame(wire::Kind::SignedOut.number(), record.encode(limits).ok()?)
}

fn list_read_frame(read: u64, outcome: jig_domain_link::ListOutcome, limits: &wire::v22::Limits) -> Option<Frame> {
    match &outcome {
        jig_domain_link::ListOutcome::Listed(rows) => {
            if !count_fits(rows.len(), limits.listed_rows_rows) {
                return None;
            }
            for row in rows {
                if !count_fits(row.efforts.len(), limits.list_row_efforts) {
                    return None;
                }
            }
        }
        jig_domain_link::ListOutcome::Failed(_) => {}
    }
    let outcome = mapped::list_outcome_to_record(&outcome).ok()?;
    let record = wire::ListRead { read, outcome };
    frame(wire::Kind::ListRead.number(), record.encode(limits).ok()?)
}

/// Byte bridges surround this component's current-value/domain translation (bridges.md, 5).
pub(crate) fn send(
    machine: &mut wire::translation::Translation,
    frame: Frame,
    version: u16,
    agreed: &[wire::Agreed],
    limits: &wire::v22::Limits,
) -> Result<Frame, wire::translation::Untranslated> {
    machine.send(frame, version, agreed, limits)
}
pub(crate) fn receive(
    machine: &mut wire::translation::Translation,
    kind: u16,
    body: &[u8],
    version: u16,
    agreed: &[wire::Agreed],
    limits: &wire::v22::Limits,
) -> Result<Frame, wire::translation::Untranslated> {
    machine.receive(kind, body, version, agreed, limits)
}

fn usage_read_frame(read: u64, outcome: UsageOutcome, limits: &wire::v22::Limits) -> Option<Frame> {
    match &outcome {
        UsageOutcome::Read(windows) => {
            if !count_fits(windows.len(), limits.usage_windows_windows) {
                return None;
            }
        }
        UsageOutcome::NotSignedIn | UsageOutcome::Spent | UsageOutcome::Failed { .. } => {}
    }
    let outcome = mapped::usage_outcome_to_record(&outcome).ok()?;
    let record = wire::UsageRead { read, outcome };
    frame(wire::Kind::UsageRead.number(), record.encode(limits).ok()?)
}

pub(super) fn usage_window_to_record(row: &UsageWindow) -> wire::UsageWindow {
    wire::UsageWindow {
        label: row.label.clone(),
        length: row.window.length,
        used: row.window.used,
        resets: reset_nanos(row.window.resets),
        state: row.window.state,
    }
}

fn reset_nanos(reset: Option<skein_lib::Wall>) -> Option<u64> {
    Some(wall_to_record(&reset?))
}

fn part_frame(run: Token, attempt: Token, part: jig_domain_link::Part, limits: &wire::v22::Limits) -> Option<Frame> {
    let record = wire::v22::Part {
        task: token_to_record(&run),
        attempt: token_to_record(&attempt),
        number: part.number,
        turn: part.turn,
        thread: part.thread,
        call: part.call,
        kind: mapped::part_kind_to_record(&part.kind),
        body: part.body,
    };
    frame(wire::Kind::Part.number(), record.encode(limits).ok()?)
}
