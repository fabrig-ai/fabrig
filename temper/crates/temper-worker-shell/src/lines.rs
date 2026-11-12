//! The worker's own lines (temper/docs/design/protocol/services.md, 7).
use temper_shell::directories::Directories;
use temper_worker_service::worker_alone as service;

/// A worker alone has no listening diagnostic; its first drain carries its worker notes.
pub fn worker_listening_line(service: &mut service::Service, binary: &str) -> Option<(String, bool)> {
    worker_note_line(service, binary).map(|line| (line, false))
}
pub fn worker_note_line(service: &mut service::Service, binary: &str) -> Option<String> {
    service.notes().pop().map(|note| note_line(binary, note))
}
#[must_use]
pub fn worker_service_ending(
    service: &service::Service,
    binary: &str,
    failed: bool,
    directories: &Directories,
    store: &[u8],
) -> (String, u8) {
    ending_with_binary(binary, service.end(), failed, directories, store)
}
#[must_use]
pub fn note_line(binary: &str, note: service::Note) -> String {
    match note {
        service::Note::StopBoundPassed { abandoned } => {
            format!("{binary} worker: stop bound passed; abandoned {abandoned} turns and answers\n")
        }
        service::Note::WorkerJob { run, attempt, kind, verb, ending, why } => {
            let verb = verb.escape_ascii();
            let prefix = format!("{binary} worker: run {run} attempt {attempt} job {kind} {verb} ");
            let (class, _) = worker_job_ending(&ending);
            run_detail_line(format!("{prefix}{class}: "), &why)
        }
        service::Note::Linked { worker, address } => {
            format!("linked {} {address}\n", String::from_utf8_lossy(&worker))
        }
        service::Note::Lost { worker, address, why } => {
            format!("lost {} {address} {}\n", String::from_utf8_lossy(&worker), worker_loss(why))
        }
        service::Note::Refused { worker, address, reason } => {
            let reason = refusal(reason);
            format!("refused {} {address} {reason}\n", String::from_utf8_lossy(&worker))
        }
    }
}
#[must_use]
pub fn ending_with_binary(
    binary: &str,
    end: Option<&service::End>,
    failed: bool,
    _directories: &Directories,
    _store: &[u8],
) -> (String, u8) {
    let lines = match end {
        Some(service::End::Stopped(service::Stop::Start(found))) => {
            use temper_service_core::startup::problems::{Problems, SharedWhat};
            temper_service_shell::problems(
                binary,
                &Problems::<SharedWhat>::jig(found.clone(), &jig_protocol_config::Path::root()),
            )
        }
        None | Some(service::End::Drained | service::End::Aborted) => String::new(),
    };
    (lines, u8::from(failed || end != Some(&service::End::Drained)))
}
fn worker_job_ending(ending: &temper_worker_service::WorkerJobEnding) -> (String, &[u8]) {
    use jig_domain_host::{JobInvalid, JobRefusal};
    use temper_worker_service::WorkerJobEnding;
    match ending {
        WorkerJobEnding::Failed { failure, detail } => (format!("failed {failure:?}"), detail),
        WorkerJobEnding::Refused(JobRefusal::Invalid(JobInvalid::Start { why })) => {
            (String::from("refused Invalid(Start)"), why)
        }
        WorkerJobEnding::Refused(
            reason @ (JobRefusal::Busy
            | JobRefusal::Invalid(
                JobInvalid::Kind | JobInvalid::Spec | JobInvalid::Deadline | JobInvalid::Grants,
            )),
        ) => (format!("refused {reason:?}"), b""),
    }
}

fn run_detail_line(mut line: String, detail: &[u8]) -> String {
    use std::fmt::Write as _;
    let from = detail.len().saturating_sub(4096);
    for byte in detail.get(from..).expect("bounded diagnostic tail") {
        for escaped in core::ascii::escape_default(*byte) {
            line.write_char(char::from(escaped)).expect("format escaped diagnostic");
        }
    }
    line.push('\n');
    line
}
fn loss_refusal(code: u16) -> String {
    refusal(match code {
        2 => temper_worker_service::Refusal::Terms,
        3 => temper_worker_service::Refusal::Malformed,
        code => temper_worker_service::Refusal::Peer(code),
    })
}
fn worker_loss(why: temper_worker_service::WorkerLoss) -> String {
    use temper_worker_service::{WorkerClosed as Closed, WorkerLoss as Loss};
    match why {
        Loss::Closed(Closed::Stream) => "stream".into(),
        Loss::Closed(Closed::Truncated) => "truncated".into(),
        Loss::Closed(Closed::Framing) => "framing".into(),
        Loss::Closed(Closed::Owner) => "owner".into(),
        Loss::Closed(Closed::RefusedHere(reason)) => format!("refused {}", loss_refusal(reason)),
        Loss::Closed(Closed::RefusedPeer(reason)) => format!("refused-by-peer {}", loss_refusal(reason)),
        Loss::Silence => "silence".into(),
        Loss::Stall => "stall".into(),
        Loss::Handshake => "handshake".into(),
        Loss::Full => "full".into(),
        Loss::Busy => "busy".into(),
        Loss::Fault => "fault".into(),
        Loss::Unreachable => "unreachable".into(),
    }
}

fn refusal(reason: temper_worker_service::Refusal) -> String {
    use temper_worker_service::WorkerRefusalReason as Reason;
    match reason {
        temper_worker_service::Refusal::Terms => "terms".into(),
        temper_worker_service::Refusal::Malformed => "malformed".into(),
        temper_worker_service::Refusal::Peer(code) => match Reason::from_code(code) {
            Some(Reason::Unauthorized) => "unauthorized".into(),
            Some(Reason::Application) => "application".into(),
            Some(Reason::Busy) => "busy".into(),
            Some(Reason::Replaced) => "replaced".into(),
            Some(Reason::NotAdmitted) => "not-admitted".into(),
            Some(Reason::Stopping) => "stopping".into(),
            None => format!("code-{code}"),
        },
    }
}

#[cfg(test)]
#[path = "worker_lines_tests.rs"]
mod tests;
