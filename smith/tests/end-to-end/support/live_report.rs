//! Content-free outside evidence (smith/docs/design/testing.md, section 2.3).
//! Keeps no state and prints no bytes from a provider, a prompt or a file.
//! `exec_report`, `file_report` and `stderr_lines` accept settled observations;
//! only typed facts, prescribed paths and fixed diagnostic forms leave them.

use smith_events::{Event, Record};
use std::fmt::Write as _;

/// Which story, account and invocation a report is about.
#[derive(Clone, Copy)]
pub struct Leg<'a> {
    pub story: &'a str,
    pub account: &'a str,
    /// One-based, in the story's invocation order.
    pub invocation: usize,
    pub step: Step,
}

/// The product entry point whose settled evidence the caller reports.
#[derive(Clone, Copy)]
pub enum Step {
    /// Configuration and credential inspection before execution.
    Check,
    /// The run and its typed event stream.
    Exec,
}

/// A settled process's content-free evidence, without its standard output.
pub struct Settled<'a> {
    pub exit: smith_bench::Exit,
    pub forced: smith_bench::Forced,
    /// A harness fault in fixed words.
    pub error: Option<&'a str>,
    pub live_pids: usize,
    pub stderr: &'a [u8],
}

impl<'a> Settled<'a> {
    /// Borrow only the evidence admitted by the live reporting contract.
    #[must_use]
    pub fn of(result: &'a smith_bench::process::ProcessResult) -> Self {
        Self {
            exit: result.exit,
            forced: result.forced,
            error: result.error.as_deref(),
            live_pids: result.tree.live_pids.len(),
            stderr: &result.stderr,
        }
    }
}

const LIMIT: usize = 4096;
const TRUNCATED: &str = "\n[truncated]\n";

fn append(report: &mut String, part: &str) -> bool {
    let ceiling = LIMIT.checked_sub(TRUNCATED.len()).expect("marker fits");
    let room = ceiling.saturating_sub(report.len());
    if part.len() <= room {
        report.push_str(part);
        return true;
    }
    if report.len() <= ceiling {
        let end = part.char_indices().map(|(index, _)| index).take_while(|index| *index <= room).last().unwrap_or(0);
        report.push_str(part.get(..end).expect("character boundary"));
        report.push_str(TRUNCATED);
    }
    false
}

/// One report of at most 4096 bytes, containing only admitted outside facts.
#[must_use]
pub fn exec_report(
    leg: &Leg<'_>,
    failed: &str,
    settled: &Settled<'_>,
    records: &[&Record],
    end: Option<&smith_bench::End>,
) -> String {
    let mut report = String::new();
    let step = match leg.step {
        Step::Check => "Check",
        Step::Exec => "Exec",
    };
    append(
        &mut report,
        &format!(
            "story={} account={} invocation={} step={step}\nfailed={failed}\nexit={:?} forced={:?} live_pids={}\nerror={}\n",
            leg.story,
            leg.account,
            leg.invocation,
            settled.exit,
            settled.forced,
            settled.live_pids,
            settled.error.unwrap_or("none")
        ),
    );
    append(&mut report, &format!("{}\n", end_report(end)));
    let (bytes, lines, kept) = stderr_lines(settled.stderr, leg.account);
    append(
        &mut report,
        &format!(
            "stderr: bytes={bytes} lines={lines} kept={} omitted={}\n",
            kept.len(),
            lines.saturating_sub(kept.len())
        ),
    );
    // Put authoritative answers before potentially numerous tool records.
    if !append_records(&mut report, records, true) {
        return report;
    }
    if !append_records(&mut report, records, false) {
        return report;
    }
    for line in kept {
        if !append(&mut report, &format!("{line}\n")) {
            break;
        }
    }
    report
}

fn append_records(report: &mut String, records: &[&Record], answers: bool) -> bool {
    let mut session = 0_usize;
    for record in records {
        if matches!(record.event, Event::SessionStarted(_)) {
            session = session.saturating_add(1);
        }
        if matches!(record.event, Event::RunCompleted(_)) != answers {
            continue;
        }
        let line = record_report(record);
        if !line.is_empty() && !append(report, &format!("session={} {line}", session.max(1))) {
            return false;
        }
    }
    true
}

fn end_report(end: Option<&smith_bench::End>) -> String {
    use smith_bench::End;
    match end {
        None => "end=none".into(),
        Some(End::Completed) => "end=Completed".into(),
        Some(End::Failed { reason, detail: _ }) => format!("end=Failed reason={reason:?}"),
        Some(End::Budget { which }) => format!("end=Budget which={which:?}"),
        Some(End::Refused { setup: _ }) => "end=Refused".into(),
        Some(End::Timeout) => "end=Timeout".into(),
        Some(End::HarnessError { what }) => {
            // The classifier interpolates Unknown payloads into these faults.
            let safe = ["unknown answer failure class", "unknown run status", "unknown session mode"]
                .into_iter()
                .find(|prefix| what.starts_with(prefix));
            safe.map_or_else(
                || format!("end=HarnessError what={what}"),
                |prefix| format!("end=HarnessError what={prefix} unknown"),
            )
        }
    }
}

fn record_report(record: &Record) -> String {
    match &record.event {
        Event::SessionStarted(_) => "session.started\n".into(),
        Event::RunStarted(start) => format!(
            "run.started resumed={} messages={} backend={}\n",
            start.resumed,
            start.messages,
            backend(&start.main.backend)
        ),
        Event::RunCompleted(completed) => {
            let mut line = format!("run.completed status={} turns={}", status(&completed.status), completed.turns);
            if let Some(failure) = &completed.failure {
                write!(line, " failure.class={}", answer_class(&failure.class)).expect("String write");
                for (name, value) in [
                    ("attempts", failure.attempts),
                    ("used", failure.used),
                    ("bound", failure.bound),
                    ("turn", failure.turn),
                ] {
                    if let Some(value) = value {
                        write!(line, " {name}={value}").expect("String write");
                    }
                }
                if let Some(state) = &failure.state {
                    write!(line, " state={}", account_state(state)).expect("String write");
                }
                if let Some(exit) = &failure.exit {
                    write!(line, " exit={exit:?}").expect("String write");
                }
                if let Some(ended) = &failure.ended {
                    write!(line, " ended={ended:?}").expect("String write");
                }
                if let Some(which) = &failure.which {
                    write!(line, " which={}", which.escape_ascii()).expect("String write");
                }
                if let Some(reason) = &failure.reason {
                    write!(line, " reason={}", reason.escape_ascii()).expect("String write");
                }
                if let Some(completion) = &failure.completion {
                    write!(
                        line,
                        " completion.class={} completion.evidence={}",
                        failure_class(&completion.class),
                        evidence(&completion.evidence)
                    )
                    .expect("String write");
                    if let Some(phase) = &completion.phase {
                        write!(line, " completion.phase={}", failure_phase(phase)).expect("String write");
                    }
                    if let Some(status) = completion.status {
                        write!(line, " completion.status={status}").expect("String write");
                    }
                }
            }
            line.push('\n');
            line
        }
        Event::ConversationClosed(closed) => format!("conversation.closed end={}\n", conversation_end(&closed.end)),
        Event::Notice(notice) => format!("notice kind={}\n", notice_kind(&notice.kind)),
        Event::ToolCompleted(tool) => format!(
            "tool.completed tool={} verdict={} delivery={}\n",
            tool_name(&tool.tool),
            verdict(&tool.verdict),
            tool.delivery.as_ref().map_or("none", delivered)
        ),
        Event::CheckCompleted(check) => {
            format!("check.completed exit={} passed={}\n", check_exit(&check.exit), check.passed)
        }
        Event::SessionEnded(ended) => format!("session.ended exit={} answered={}\n", ended.exit, ended.answered),
        Event::Budget(_)
        | Event::ConversationOpened(_)
        | Event::ResponseStarted(_)
        | Event::ResponseCompleted(_)
        | Event::TextDelta(_)
        | Event::ToolStarted(_)
        | Event::JobEnded(_)
        | Event::CheckStarted(_)
        | Event::MessageReceived(_)
        | Event::MessageRead(_)
        | Event::MessageUnread(_) => String::new(),
    }
}

fn tool_name(name: &[u8]) -> &'static str {
    match name {
        b"finish" => "finish",
        b"deliver" => "deliver",
        b"read" => "read",
        b"list" => "list",
        b"search" => "search",
        b"write" => "write",
        b"edit" => "edit",
        b"remove" => "remove",
        b"shell" => "shell",
        b"job" => "job",
        b"Read" => "Read",
        b"Glob" => "Glob",
        b"Grep" => "Grep",
        b"Write" => "Write",
        b"Edit" => "Edit",
        b"Bash" => "Bash",
        b"permission_prompt" => "permission_prompt",
        _ => "unknown",
    }
}

fn check_exit(exit: &smith_events::CheckExit) -> String {
    use smith_events::CheckExit;
    match exit {
        CheckExit::Code { code } => format!("Code({code})"),
        CheckExit::Signal { signal } => format!("Signal({signal})"),
        CheckExit::TimedOut => "TimedOut".into(),
        CheckExit::Unstarted => "Unstarted".into(),
        CheckExit::Cancelled => "Cancelled".into(),
        CheckExit::Unknown(_) => "unknown".into(),
    }
}

/// Describe a prescribed file's mismatch without printing either file's bytes.
#[must_use]
pub fn file_report(path: &[u8], expected: &[u8], found: Option<&[u8]>) -> String {
    let relation = match found {
        None => "missing".into(),
        Some([]) => "empty".into(),
        Some(found) if expected.strip_suffix(b"\n") == Some(found) => "expected without its final newline".into(),
        Some(found) if found.starts_with(expected) => {
            format!("expected followed by {} more bytes", found.len().saturating_sub(expected.len()))
        }
        Some(found) if expected.starts_with(found) => {
            format!("a prefix of expected ({} of {} bytes)", found.len(), expected.len())
        }
        Some(found) => {
            let byte =
                expected.iter().zip(found).position(|(left, right)| left != right).expect("different non-prefix files");
            format!("differs at byte {byte} (expected {} bytes, found {})", expected.len(), found.len())
        }
    };
    format!("file \"{}\": {relation}", path.escape_ascii())
}

/// Count every byte and line, retaining only complete content-free forms.
#[must_use]
pub fn stderr_lines(stderr: &[u8], account: &str) -> (usize, usize, Vec<String>) {
    let mut count = 0_usize;
    let mut kept = Vec::new();
    for bytes in stderr.split_inclusive(|byte| *byte == b'\n') {
        count = count.saturating_add(1);
        let bytes = bytes.strip_suffix(b"\n").unwrap_or(bytes);
        let bytes = bytes.strip_suffix(b"\r").unwrap_or(bytes);
        if let Ok(line) = std::str::from_utf8(bytes)
            && let Some(line) = allowed_line(line, account)
        {
            kept.push(line.into());
        }
    }
    (stderr.len(), count, kept)
}

fn allowed_line<'a>(line: &'a str, account: &str) -> Option<&'a str> {
    let complete = matches!(
        line,
        "smith: local host started"
            | "smith: agent started"
            | "smith: commands run in process groups and are not contained"
            | "smith: chat chat"
    ) || trust_notice(line)
        || budget_line(line)
        || credential_line(line, account);
    if complete { Some(line) } else { agent_failure_line(line) }
}

fn agent_failure_line(line: &str) -> Option<&str> {
    const PREFIX: &str = "smith: run failed: the agent ";
    let rest = line.strip_prefix(PREFIX)?;
    let form = rest.split_once(": ").map_or(rest, |(form, _)| form);
    let (words, end) = form.split_once(" (")?;
    if !matches!(
        words,
        "could not start"
            | "ended without answering"
            | "broke the channel rules"
            | "exceeded its receiving bound"
            | "made no progress"
            | "reached its wall time bound"
    ) {
        return None;
    }
    let end = end.strip_suffix("; terminated").or_else(|| end.strip_suffix("; killed")).unwrap_or(end);
    let end = end.strip_suffix(')')?;
    let valid = matches!(end, "exit not observed" | "not spawned" | "inline run settled")
        || end
            .strip_prefix("exit ")
            .is_some_and(|value| value.parse::<u8>().is_ok_and(|number| number.to_string() == value))
        || end
            .strip_prefix("signal ")
            .is_some_and(|value| value.parse::<u32>().is_ok_and(|number| number.to_string() == value));
    if valid { line.get(..PREFIX.len().checked_add(form.len()).expect("substring length fits")) } else { None }
}

fn credential_line(line: &str, account: &str) -> bool {
    use smith_local_shell::credentials::Credential;
    [
        Credential::NotSignedIn,
        Credential::SignInAgain,
        Credential::ProgramNotSignedIn { program: smith_agent_shell::config::ProgramName::ClaudeCode },
    ]
    .into_iter()
    .any(|credential| {
        ["smith: ", "smith: warning: "]
            .into_iter()
            .any(|prefix| line == format!("{prefix}{}", credential.line(account)))
    })
}

fn trust_notice(line: &str) -> bool {
    line.strip_prefix("smith: trust roots: ").is_some_and(|counts| {
        let fields: Vec<_> = counts.split(", ").collect();
        fields.len() == 4
            && fields
                .iter()
                .zip([" taken", " skipped", " malformed blocks", " other blocks"])
                .all(|(field, suffix)| field.strip_suffix(suffix).is_some_and(|count| count.parse::<u32>().is_ok()))
    })
}

fn budget_line(line: &str) -> bool {
    line.strip_prefix("smith: run failed: out of ").is_some_and(|rest| {
        let Some((which, rest)) = rest.split_once(": used ") else {
            return false;
        };
        let Some((used, bound)) = rest.split_once(" of ") else {
            return false;
        };
        matches!(which, "turns" | "spend" | "time") && decimal(used) && decimal(bound)
    })
}

fn decimal(value: &str) -> bool {
    !value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()) && value.parse::<u64>().is_ok()
}

fn backend(value: &smith_events::Backend) -> &'static str {
    match value {
        smith_events::Backend::Direct => "Direct",
        smith_events::Backend::ClaudeCode => "ClaudeCode",
        smith_events::Backend::Unknown(_) => "unknown",
    }
}

fn status(value: &smith_events::Status) -> &'static str {
    match value {
        smith_events::Status::Accepted => "Accepted",
        smith_events::Status::Parked => "Parked",
        smith_events::Status::Failed => "Failed",
        smith_events::Status::Refused => "Refused",
        smith_events::Status::Unknown(_) => "unknown",
    }
}

fn answer_class(value: &smith_events::AnswerClass) -> &'static str {
    match value {
        smith_events::AnswerClass::Model => "Model",
        smith_events::AnswerClass::Budget => "Budget",
        smith_events::AnswerClass::Policy => "Policy",
        smith_events::AnswerClass::Cancelled => "Cancelled",
        smith_events::AnswerClass::Agent => "Agent",
        smith_events::AnswerClass::Host => "Host",
        smith_events::AnswerClass::Stale => "Stale",
        smith_events::AnswerClass::Transcript => "Transcript",
        smith_events::AnswerClass::Busy => "Busy",
        smith_events::AnswerClass::Invalid => "Invalid",
        smith_events::AnswerClass::Unknown(_) => "unknown",
    }
}

fn account_state(value: &smith_events::AccountState) -> &'static str {
    match value {
        smith_events::AccountState::Limited => "Limited",
        smith_events::AccountState::Refused => "Refused",
        smith_events::AccountState::Unauthorized => "Unauthorized",
        smith_events::AccountState::Unknown(_) => "unknown",
    }
}

fn failure_class(value: &smith_events::FailureClass) -> &'static str {
    match value {
        smith_events::FailureClass::Overloaded => "Overloaded",
        smith_events::FailureClass::RateLimited => "RateLimited",
        smith_events::FailureClass::Exhausted => "Exhausted",
        smith_events::FailureClass::Refused => "Refused",
        smith_events::FailureClass::Unsupported => "Unsupported",
        smith_events::FailureClass::Unavailable => "Unavailable",
        smith_events::FailureClass::TimedOut => "TimedOut",
        smith_events::FailureClass::ContextTooLong => "ContextTooLong",
        smith_events::FailureClass::Invalid => "Invalid",
        smith_events::FailureClass::Unauthorized => "Unauthorized",
        smith_events::FailureClass::Limit => "Limit",
        smith_events::FailureClass::Protocol => "Protocol",
        smith_events::FailureClass::Cancelled => "Cancelled",
        smith_events::FailureClass::Unknown(_) => "unknown",
    }
}

fn evidence(value: &smith_events::Evidence) -> &'static str {
    match value {
        smith_events::Evidence::Unsent => "Unsent",
        smith_events::Evidence::MaybeSent => "MaybeSent",
        smith_events::Evidence::Response => "Response",
        smith_events::Evidence::Unknown(_) => "unknown",
    }
}

fn failure_phase(value: &smith_events::FailurePhase) -> &'static str {
    match value {
        smith_events::FailurePhase::Connect => "Connect",
        smith_events::FailurePhase::Handshake => "Handshake",
        smith_events::FailurePhase::Head => "Head",
        smith_events::FailurePhase::Idle => "Idle",
        smith_events::FailurePhase::Whole => "Whole",
        smith_events::FailurePhase::Unknown(_) => "unknown",
    }
}

fn conversation_end(value: &smith_events::ConversationEnd) -> &'static str {
    match value {
        smith_events::ConversationEnd::Closed => "Closed",
        smith_events::ConversationEnd::Budget => "Budget",
        smith_events::ConversationEnd::Failed => "Failed",
        smith_events::ConversationEnd::Refused => "Refused",
        smith_events::ConversationEnd::Transcript => "Transcript",
        smith_events::ConversationEnd::Overflow => "Overflow",
        smith_events::ConversationEnd::Unknown(_) => "unknown",
    }
}

fn notice_kind(value: &smith_events::NoticeKind) -> &str {
    match value {
        smith_events::NoticeKind::ReasoningDropped => "ReasoningDropped",
        smith_events::NoticeKind::CredentialUnauthorized => "CredentialUnauthorized",
        smith_events::NoticeKind::AccountLimited => "AccountLimited",
        smith_events::NoticeKind::AccountRefused => "AccountRefused",
        smith_events::NoticeKind::ForcedExit => "ForcedExit",
        smith_events::NoticeKind::TeardownAborted => "TeardownAborted",
        smith_events::NoticeKind::Unknown(kind) => {
            if !kind.is_empty()
                && kind.len() <= 32
                && kind.iter().all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'_')
            {
                std::str::from_utf8(kind).expect("ASCII identifier")
            } else {
                "unknown"
            }
        }
    }
}

fn verdict(value: &smith_events::Verdict) -> &'static str {
    match value {
        smith_events::Verdict::Read => "Read",
        smith_events::Verdict::Listed => "Listed",
        smith_events::Verdict::Found => "Found",
        smith_events::Verdict::Written => "Written",
        smith_events::Verdict::Removed => "Removed",
        smith_events::Verdict::NotEmpty => "NotEmpty",
        smith_events::Verdict::Edited => "Edited",
        smith_events::Verdict::Exited => "Exited",
        smith_events::Verdict::Conflict => "Conflict",
        smith_events::Verdict::Missing => "Missing",
        smith_events::Verdict::TooLarge => "TooLarge",
        smith_events::Verdict::TimedOut => "TimedOut",
        smith_events::Verdict::Running => "Running",
        smith_events::Verdict::Stopped => "Stopped",
        smith_events::Verdict::Failed => "Failed",
        smith_events::Verdict::Cancelled => "Cancelled",
        smith_events::Verdict::Ambiguous => "Ambiguous",
        smith_events::Verdict::Result => "Result",
        smith_events::Verdict::Error => "Error",
        smith_events::Verdict::Invalid => "Invalid",
        smith_events::Verdict::NotRun => "NotRun",
        smith_events::Verdict::Withdrawn => "Withdrawn",
        smith_events::Verdict::NotGranted => "NotGranted",
        smith_events::Verdict::Outside => "Outside",
        smith_events::Verdict::ReadOnly => "ReadOnly",
        smith_events::Verdict::TooLong => "TooLong",
        smith_events::Verdict::NotFound => "NotFound",
        smith_events::Verdict::NotFile => "NotFile",
        smith_events::Verdict::Linked => "Linked",
        smith_events::Verdict::Protected => "Protected",
        smith_events::Verdict::NotDirectory => "NotDirectory",
        smith_events::Verdict::NotRead => "NotRead",
        smith_events::Verdict::Stale => "Stale",
        smith_events::Verdict::NoMatch => "NoMatch",
        smith_events::Verdict::Unchanged => "Unchanged",
        smith_events::Verdict::Busy => "Busy",
        smith_events::Verdict::NulByte => "NulByte",
        smith_events::Verdict::Unknown(_) => "unknown",
    }
}

fn delivered(value: &smith_events::Delivered) -> &'static str {
    match value {
        smith_events::Delivered::Delivered => "Delivered",
        smith_events::Delivered::Nothing => "Nothing",
        smith_events::Delivered::Refused => "Refused",
        smith_events::Delivered::Failed => "Failed",
        smith_events::Delivered::Stale => "Stale",
        smith_events::Delivered::Unknown(_) => "unknown",
    }
}
