//! Owned payloads and link metadata (jig/docs/design/domain/hosts/hub.md, 4–8).
use crate::Link;
use alloc::boxed::Box;
use jig_domain_core_vocabulary as core;
use jig_domain_fleet as fleet;
use jig_domain_host as host;
use jig_domain_link::Phase;
use jig_domain_tasks as tasks;
use skein_lib::{List, Token};
#[derive(Hash, Debug)]
pub enum Payload {
    Busy,
    Turn {
        task: u64,
        attempt: u64,
        turn: u32,
        cumulative: u64,
        read: Option<u64>,
        transcript: Box<[u8]>,
        opens_window: bool,
        refused: Box<[u64]>,
    },
    Answer {
        task: u64,
        attempt: u64,
        cumulative: u64,
        end: Ending,
        read: Option<u64>,
        refused: Box<[u64]>,
    },
    SettledCall(core::SettledCall),
    InboxWord(tasks::Word),
}
/// A scripted terminal is already read; a channel terminal is opaque.
#[derive(Hash, Debug)]
pub enum Ending {
    Scripted(tasks::End),
    Link(host::Ending),
    Job(host::JobEnding),
}

/// Retain a worker's bytes and translate only their boundary metadata.
#[must_use]
pub fn up(domain: &mut Link, channel: Token, up: host::LinkUp) -> Option<core::Event> {
    let event = match up {
        host::LinkUp::Ask { run, attempt, number, bytes } => fleet::Event::Ask { channel, run, attempt, number, bytes },
        host::LinkUp::Draining => fleet::Event::Draining { channel },
        host::LinkUp::JobAnswer { run, attempt, answer } => {
            let kind = match &answer {
                host::JobEnding::Finished { .. } => fleet::Answer::Ended,
                host::JobEnding::Failed { .. } => fleet::Answer::Failed { turned: None },
                host::JobEnding::Refused(host::JobRefusal::Busy) => fleet::Answer::Busy,
                host::JobEnding::Refused(host::JobRefusal::Untranslated(_)) => fleet::Answer::Untranslated,
                host::JobEnding::Refused(host::JobRefusal::Invalid(_)) => fleet::Answer::Invalid,
            };
            let payload = domain.retain(Payload::Answer {
                task: run.raw(),
                attempt: attempt.raw(),
                cumulative: 0,
                read: None,
                refused: Box::new([]),
                end: Ending::Job(answer),
            });
            fleet::Event::Answer { channel, run, attempt, answer: kind, payload }
        }
        host::LinkUp::Piece { run, attempt, piece } => return live_piece(domain, run, attempt, piece),
        host::LinkUp::ListRead { read, outcome } => fleet::Event::ListRead { channel, read, outcome },
        host::LinkUp::Logins { logins } => fleet::Event::Logins { channel, logins },
        host::LinkUp::SignedOut { name, result } => fleet::Event::SignedOut { channel, name, result },
        host::LinkUp::Turn { run, attempt, turn } => {
            let number = turn.turn;
            let body = domain.retain(Payload::Turn {
                task: run.raw(),
                attempt: attempt.raw(),
                turn: number,
                cumulative: turn.spent,
                read: read_name(turn.read),
                transcript: turn.body,
                opens_window: turn.opens_window,
                refused: refusals(turn.refused),
            });
            fleet::Event::Turn { channel, run, attempt, turn: number, body }
        }
        host::LinkUp::Answer { run, attempt, answer } => {
            let kind = match answer.ending {
                host::Ending::Ended { .. } => fleet::Answer::Ended,
                host::Ending::Parked { .. } => fleet::Answer::Parked,
                host::Ending::Failed { failure, .. } => fleet::Answer::Failed { turned: turned(failure) },
                host::Ending::Refused(host::Refusal::Busy) => fleet::Answer::Busy,
                host::Ending::Refused(host::Refusal::Invalid(_)) => fleet::Answer::Invalid,
                host::Ending::Refused(host::Refusal::Version(_)) => fleet::Answer::Version,
                host::Ending::Refused(host::Refusal::Untranslated(_)) => fleet::Answer::Untranslated,
            };
            let payload = domain.retain(Payload::Answer {
                task: run.raw(),
                attempt: attempt.raw(),
                cumulative: answer.spent,
                read: read_name(answer.read),
                refused: refusals(answer.refused),
                end: Ending::Link(answer.ending),
            });
            fleet::Event::Answer { channel, run, attempt, answer: kind, payload }
        }
        host::LinkUp::Call { run, attempt, contract, call, tool, writes, input, deadline } => {
            domain
                .host_calls
                .insert(call.clone(), crate::HostCall { channel, run, attempt })
                .expect("uncapped relay routes");
            fleet::Event::Relay {
                channel,
                run,
                attempt,
                call: fleet::Call { name: call, contract, tool, writes, input, deadline },
            }
        }
        host::LinkUp::Unauthorized { run, attempt, account, generation } => {
            fleet::Event::Unauthorized { channel, run, attempt, account, generation }
        }
        host::LinkUp::Limited { run, attempt, account, reset } => {
            fleet::Event::Limited { channel, run, attempt, account, reset }
        }
        host::LinkUp::AccountRefused { run, attempt, account, code, reason } => {
            fleet::Event::AccountRefused { channel, run, attempt, account, code, reason }
        }
        host::LinkUp::Activity { run, attempt, activity } => {
            fleet::Event::Observe { channel, run, attempt, report: report(activity) }
        }
    };
    Some(core::Event::Fleet(event))
}
/// The account a failed run turned on, as the fleet carries it.
const fn turned(failure: host::Failure) -> Option<fleet::Turned> {
    match failure {
        host::Failure::Run(host::RunFailure::Limited { account, reset }) => {
            Some(fleet::Turned::Limited { account, reset })
        }
        host::Failure::Run(host::RunFailure::Refused { account }) => Some(fleet::Turned::Refused { account }),
        host::Failure::Run(
            host::RunFailure::Exceeded(_)
            | host::RunFailure::Transcript
            | host::RunFailure::Model
            | host::RunFailure::Budget
            | host::RunFailure::Policy
            | host::RunFailure::Cancelled
            | host::RunFailure::Stale,
        )
        | host::Failure::Unprepared(_)
        | host::Failure::Agent(_)
        | host::Failure::Cancelled(_) => None,
    }
}
fn read_name(name: Option<Token>) -> Option<u64> {
    Some(name?.raw())
}
fn refusals(values: Box<[host::Refused]>) -> Box<[u64]> {
    let mut names = List::with_capacity(u32::try_from(values.len()).expect("bounded refusals"));
    for value in values {
        names.push(value.name.raw()).expect("one slot per refusal");
    }
    names.into_boxed()
}

pub(crate) fn ending(ending: &host::Ending) -> tasks::End {
    match ending {
        host::Ending::Refused(host::Refusal::Busy) => tasks::End::Refused,
        host::Ending::Refused(host::Refusal::Version(mismatch) | host::Refusal::Untranslated(mismatch)) => {
            tasks::End::RefusedWithReason { reason: mismatch.reason() }
        }
        host::Ending::Refused(host::Refusal::Invalid(
            host::Invalid::Transcript | host::Invalid::Start { cause: host::StartCause::Transcript, .. },
        )) => tasks::End::Failed(tasks::Class::Transient),
        host::Ending::Refused(host::Refusal::Invalid(_)) => tasks::End::Failed(tasks::Class::Permanent),
        host::Ending::Parked { .. } => tasks::End::Parked,
        host::Ending::Failed { failure: host::Failure::Run(host::RunFailure::Model), model: Some(model), .. } => {
            tasks::End::FailedWithReason {
                class: if model.outage() { tasks::Class::Infrastructure } else { tasks::Class::Run },
                reason: model.words(),
            }
        }
        host::Ending::Failed { failure, detail, .. } => match failure {
            host::Failure::Run(host::RunFailure::Exceeded(value)) => tasks::End::Exceeded(tasks::Exceeded {
                bound: match value.bound {
                    host::Bound::Reasoning => tasks::Bound::Reasoning,
                    host::Bound::Metadata => tasks::Bound::Metadata,
                    host::Bound::Items => tasks::Bound::Items,
                    host::Bound::Answer => tasks::Bound::Answer,
                    host::Bound::Tools => tasks::Bound::Tools,
                    host::Bound::HistoryItems => tasks::Bound::HistoryItems,
                    host::Bound::History => tasks::Bound::History,
                    host::Bound::Window => tasks::Bound::Window,
                    host::Bound::Output => tasks::Bound::Output,
                },
                value: value.value,
            }),
            host::Failure::Unprepared(host::Preparation::Transient) => {
                if detail.is_empty() {
                    tasks::End::Failed(tasks::Class::Infrastructure)
                } else {
                    tasks::End::FailedWithReason { class: tasks::Class::Infrastructure, reason: job_reason(detail) }
                }
            }
            host::Failure::Run(host::RunFailure::Transcript) => tasks::End::Failed(tasks::Class::Transient),
            host::Failure::Unprepared(host::Preparation::Permanent { .. }) => {
                tasks::End::Failed(tasks::Class::Permanent)
            }
            host::Failure::Run(
                host::RunFailure::Model
                | host::RunFailure::Budget
                | host::RunFailure::Policy
                | host::RunFailure::Cancelled
                | host::RunFailure::Stale
                | host::RunFailure::Limited { .. }
                | host::RunFailure::Refused { .. },
            ) => tasks::End::Failed(tasks::Class::Run),
            host::Failure::Cancelled(host::Reason::Contact | host::Reason::Shutdown) => tasks::End::Recheck,
            host::Failure::Agent(_) | host::Failure::Cancelled(host::Reason::Engine) => {
                tasks::End::Failed(tasks::Class::Agent)
            }
        },
        host::Ending::Ended { .. } => unreachable!("ended payloads leave for the routing"),
    }
}

/// Unusable resume history keeps its own reason (jig/docs/design/domain/hosts/hub.md, 3).
pub(crate) fn unusable_history(ending: &host::Ending) -> bool {
    match ending {
        host::Ending::Refused(host::Refusal::Invalid(
            host::Invalid::Transcript | host::Invalid::Start { cause: host::StartCause::Transcript, .. },
        ))
        | host::Ending::Failed { failure: host::Failure::Run(host::RunFailure::Transcript), .. } => true,
        host::Ending::Refused(
            host::Refusal::Busy
            | host::Refusal::Version(_)
            | host::Refusal::Untranslated(_)
            | host::Refusal::Invalid(
                host::Invalid::Charter
                | host::Invalid::Grants
                | host::Invalid::Messages
                | host::Invalid::Start { cause: host::StartCause::Other | host::StartCause::Grants, .. },
            ),
        )
        | host::Ending::Failed {
            failure:
                host::Failure::Unprepared(_)
                | host::Failure::Agent(_)
                | host::Failure::Cancelled(_)
                | host::Failure::Run(
                    host::RunFailure::Exceeded(_)
                    | host::RunFailure::Model
                    | host::RunFailure::Budget
                    | host::RunFailure::Policy
                    | host::RunFailure::Cancelled
                    | host::RunFailure::Stale
                    | host::RunFailure::Limited { .. }
                    | host::RunFailure::Refused { .. },
                ),
            ..
        }
        | host::Ending::Parked { .. }
        | host::Ending::Ended { .. } => false,
    }
}

fn report(activity: host::AgentActivity) -> fleet::Report {
    match activity {
        host::AgentActivity::Phase(phase) => fleet::Report::Activity(match phase {
            Phase::Preparing => fleet::Activity::Preparing,
            Phase::Starting => fleet::Activity::Starting,
            Phase::Active => fleet::Activity::Active,
            Phase::Waiting => fleet::Activity::Waiting,
            Phase::Ending | Phase::Answered => fleet::Activity::Ending,
        }),
        host::AgentActivity::Long { operation, span } => {
            fleet::Report::Activity(fleet::Activity::Long { operation, span })
        }
        host::AgentActivity::LongDone => fleet::Report::Activity(fleet::Activity::Active),
        host::AgentActivity::Count { kind, elapsed, count } => fleet::Report::Count { kind, elapsed, count },
    }
}

fn live_piece(domain: &mut Link, run: Token, attempt: Token, piece: host::Piece) -> Option<core::Event> {
    if let Ok(offset) = u32::try_from(piece.offset) {
        match piece.kind {
            host::LiveKind::Text => {
                return Some(core::Event::RunText {
                    run,
                    attempt,
                    turn: piece.turn,
                    retry: piece.retry,
                    offset,
                    text: piece.bytes,
                    call: None,
                });
            }
            host::LiveKind::Input { call, tool } => {
                return Some(core::Event::RunInput {
                    run,
                    attempt,
                    turn: piece.turn,
                    retry: piece.retry,
                    offset,
                    id: call,
                    tool,
                    input: piece.bytes,
                });
            }
            host::LiveKind::Reasoning => {}
        }
    }
    domain.pieces_dropped = domain.pieces_dropped.saturating_add(1);
    None
}

/// A job carries its bytes directly, with no result read (jig/docs/design/domain/engine/executors.md, 5).
pub(crate) fn job_end(ending: &host::JobEnding) -> tasks::End {
    let end = match ending {
        host::JobEnding::Finished { result, .. } => {
            tasks::End::Finished { result: tasks::TaskResult::Job { bytes: result.clone() }, cancel_delegates: false }
        }
        host::JobEnding::Refused(host::JobRefusal::Busy) => tasks::End::Refused,
        host::JobEnding::Refused(host::JobRefusal::Untranslated(mismatch)) => {
            tasks::End::RefusedWithReason { reason: mismatch.reason() }
        }
        host::JobEnding::Refused(host::JobRefusal::Invalid(_)) => tasks::End::Failed(tasks::Class::Permanent),
        host::JobEnding::Failed { failure, .. } => match failure {
            host::JobFailure::Unprepared(host::Preparation::Transient) => {
                tasks::End::Failed(tasks::Class::Infrastructure)
            }
            host::JobFailure::Undelivered(host::DeliveryOutcome::Failed) => tasks::End::Failed(tasks::Class::Transient),
            host::JobFailure::Unprepared(host::Preparation::Permanent { .. })
            | host::JobFailure::Undelivered(host::DeliveryOutcome::Stale | host::DeliveryOutcome::Refused) => {
                tasks::End::Failed(tasks::Class::Permanent)
            }
            host::JobFailure::Failed | host::JobFailure::Deadline => tasks::End::Failed(tasks::Class::Run),
            host::JobFailure::Unstarted => tasks::End::Failed(tasks::Class::Agent),
            host::JobFailure::Result => tasks::End::Failed(tasks::Class::Invalid),
            host::JobFailure::Cancelled(host::Reason::Shutdown) => tasks::End::Recheck,
            host::JobFailure::Cancelled(host::Reason::Contact) => tasks::End::Failed(tasks::Class::Lost),
            host::JobFailure::Cancelled(host::Reason::Engine) => tasks::End::Refused,
            host::JobFailure::Undelivered(host::DeliveryOutcome::Delivered | host::DeliveryOutcome::Nothing) => {
                tasks::End::Failed(tasks::Class::Invalid)
            }
        },
    };
    if let tasks::End::Failed(class) = &end
        && let host::JobEnding::Refused(host::JobRefusal::Invalid(host::JobInvalid::Start { why })) = ending
        && !why.is_empty()
    {
        return tasks::End::FailedWithReason { class: *class, reason: job_reason(why) };
    }
    if let tasks::End::Failed(class) = &end
        && let host::JobEnding::Failed { detail, .. } = ending
        && !detail.is_empty()
    {
        return tasks::End::FailedWithReason { class: *class, reason: job_reason(detail) };
    }
    end
}

// Raw worker diagnostics may begin inside a UTF-8 character after tail truncation.
// Keep a bounded text cause even for binary stderr.
fn job_reason(detail: &[u8]) -> Box<[u8]> {
    let bound = skein_lib::usize_of(tasks::REASON_BYTES);
    let start = detail.len().saturating_sub(bound);
    let tail = detail.get(start..).expect("bounded failure tail");
    if skein_lib::bytes::is_utf8(tail) {
        return Box::from(tail);
    }
    let mut text = skein_lib::Writer::new(tail.len());
    for byte in tail {
        let byte = if byte.is_ascii() { *byte } else { b'?' };
        text.put(&[byte]).expect("one text byte per diagnostic byte");
    }
    text.finish()
}
