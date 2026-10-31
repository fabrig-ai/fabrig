//! Wire spellings and forward-value classification (smith/docs/design/protocol/events.md, 3).
//! Each pair holds the bytes and whether the peer supplied a forward value;
//! known words are independent of the configured metadata-string cap.

use crate::records::{
    AccountState, AnswerClass, Backend, BudgetStage, Capture, ChildStop, Containment, ConversationEnd,
    ConversationKind, Delivered, Delivery, Effect, Evidence, FailureClass, FailurePhase, Family, Forced, Form, Level,
    MessageRefusal, Mode, NoticeKind, Outcome, Role, ShareBound, SharePart, Source, Status, Stop, ToolChoice, Verdict,
};

impl Capture {
    pub(crate) fn spelled(&self) -> (&[u8], bool) {
        match self {
            Capture::None => (b"none", false),
            Capture::Calls => (b"calls", false),
            Capture::Everything => (b"everything", false),
            Capture::Unknown(text) => (text, true),
        }
    }
}

impl Delivery {
    pub(crate) fn spelled(&self) -> (&[u8], bool) {
        match self {
            Delivery::Complete => (b"complete", false),
            Delivery::BestEffort => (b"best_effort", false),
            Delivery::Unknown(text) => (text, true),
        }
    }
}

impl Mode {
    pub(crate) fn spelled(&self) -> (&[u8], bool) {
        match self {
            Mode::Agent => (b"agent", false),
            Mode::Exec => (b"exec", false),
            Mode::Chat => (b"chat", false),
            Mode::Unknown(text) => (text, true),
        }
    }
}

impl Containment {
    pub(crate) fn spelled(&self) -> (&[u8], bool) {
        match self {
            Containment::Groups => (b"groups", false),
            Containment::Trees => (b"trees", false),
            Containment::Unknown(text) => (text, true),
        }
    }
}

impl Family {
    pub(crate) fn spelled(&self) -> (&[u8], bool) {
        match self {
            Family::Inspect => (b"inspect", false),
            Family::Modify => (b"modify", false),
            Family::Shell => (b"shell", false),
            Family::SubAgents => (b"sub_agents", false),
            Family::Unknown(text) => (text, true),
        }
    }
}

impl Form {
    pub(crate) fn spelled(&self) -> (&[u8], bool) {
        match self {
            Form::Report => (b"report", false),
            Form::Verdict => (b"verdict", false),
            Form::Change => (b"change", false),
            Form::Failure => (b"failure", false),
            Form::Unknown(text) => (text, true),
        }
    }
}

impl Status {
    pub(crate) fn spelled(&self) -> (&[u8], bool) {
        match self {
            Status::Accepted => (b"accepted", false),
            Status::Parked => (b"parked", false),
            Status::Failed => (b"failed", false),
            Status::Refused => (b"refused", false),
            Status::Unknown(text) => (text, true),
        }
    }
}

impl ConversationKind {
    pub(crate) fn spelled(&self) -> (&[u8], bool) {
        match self {
            ConversationKind::Main => (b"main", false),
            ConversationKind::Child => (b"child", false),
            ConversationKind::Compaction => (b"compaction", false),
            ConversationKind::Unknown(text) => (text, true),
        }
    }
}

impl ConversationEnd {
    pub(crate) fn spelled(&self) -> (&[u8], bool) {
        match self {
            ConversationEnd::Closed => (b"closed", false),
            ConversationEnd::Budget => (b"budget", false),
            ConversationEnd::Failed => (b"failed", false),
            ConversationEnd::Refused => (b"refused", false),
            ConversationEnd::Transcript => (b"transcript", false),
            ConversationEnd::Overflow => (b"overflow", false),
            ConversationEnd::Unknown(text) => (text, true),
        }
    }
}

impl ToolChoice {
    pub(crate) fn spelled(&self) -> (&[u8], bool) {
        match self {
            ToolChoice::Auto => (b"auto", false),
            ToolChoice::None => (b"none", false),
            ToolChoice::Only => (b"only", false),
            ToolChoice::Unknown(text) => (text, true),
        }
    }
}

impl Outcome {
    pub(crate) fn spelled(&self) -> (&[u8], bool) {
        match self {
            Outcome::Completed => (b"completed", false),
            Outcome::Failed => (b"failed", false),
            Outcome::Cancelled => (b"cancelled", false),
            Outcome::Unknown(text) => (text, true),
        }
    }
}

impl Stop {
    pub(crate) fn spelled(&self) -> (&[u8], bool) {
        match self {
            Stop::End => (b"end", false),
            Stop::Tools => (b"tools", false),
            Stop::MaxTokens => (b"max_tokens", false),
            Stop::Refusal => (b"refusal", false),
            Stop::Unknown(text) => (text, true),
        }
    }
}

impl Source {
    pub(crate) fn spelled(&self) -> (&[u8], bool) {
        match self {
            Source::Workspace => (b"workspace", false),
            Source::Run => (b"run", false),
            Source::Host => (b"host", false),
            Source::Unknown(text) => (text, true),
        }
    }
}

impl Effect {
    pub(crate) fn spelled(&self) -> (&[u8], bool) {
        match self {
            Effect::Read => (b"read", false),
            Effect::Write => (b"write", false),
            Effect::Unknown(text) => (text, true),
        }
    }
}

impl Verdict {
    pub(crate) fn spelled(&self) -> (&[u8], bool) {
        match self {
            Verdict::Read => (b"read", false),
            Verdict::Listed => (b"listed", false),
            Verdict::Found => (b"found", false),
            Verdict::Removed => (b"removed", false),
            Verdict::NotEmpty => (b"not_empty", false),
            Verdict::Written => (b"written", false),
            Verdict::Edited => (b"edited", false),
            Verdict::Exited => (b"exited", false),
            Verdict::Conflict => (b"conflict", false),
            Verdict::Missing => (b"missing", false),
            Verdict::TooLarge => (b"too_large", false),
            Verdict::TimedOut => (b"timed_out", false),
            Verdict::Running => (b"running", false),
            Verdict::Stopped => (b"stopped", false),
            Verdict::Failed => (b"failed", false),
            Verdict::Cancelled => (b"cancelled", false),
            Verdict::Ambiguous => (b"ambiguous", false),
            Verdict::Result => (b"result", false),
            Verdict::Error => (b"error", false),
            Verdict::Invalid => (b"invalid", false),
            Verdict::NotRun => (b"not_run", false),
            Verdict::Withdrawn => (b"withdrawn", false),
            Verdict::NotGranted => (b"not_granted", false),
            Verdict::Outside => (b"outside", false),
            Verdict::ReadOnly => (b"read_only", false),
            Verdict::TooLong => (b"too_long", false),
            Verdict::NotFound => (b"not_found", false),
            Verdict::NotFile => (b"not_file", false),
            Verdict::Linked => (b"linked", false),
            Verdict::Protected => (b"protected", false),
            Verdict::NotDirectory => (b"not_directory", false),
            Verdict::NotRead => (b"not_read", false),
            Verdict::Stale => (b"stale", false),
            Verdict::NoMatch => (b"no_match", false),
            Verdict::Unchanged => (b"unchanged", false),
            Verdict::Busy => (b"busy", false),
            Verdict::NulByte => (b"nul_byte", false),
            Verdict::Unknown(text) => (text, true),
        }
    }
}

impl Delivered {
    pub(crate) fn spelled(&self) -> (&[u8], bool) {
        match self {
            Delivered::Delivered => (b"delivered", false),
            Delivered::Nothing => (b"nothing", false),
            Delivered::Refused => (b"refused", false),
            Delivered::Failed => (b"failed", false),
            Delivered::Stale => (b"stale", false),
            Delivered::Unknown(text) => (text, true),
        }
    }
}

impl FailureClass {
    pub(crate) fn spelled(&self) -> (&[u8], bool) {
        match self {
            FailureClass::Overloaded => (b"overloaded", false),
            FailureClass::RateLimited => (b"rate_limited", false),
            FailureClass::Exhausted => (b"exhausted", false),
            FailureClass::Refused => (b"refused", false),
            FailureClass::Unsupported => (b"unsupported", false),
            FailureClass::Unavailable => (b"unavailable", false),
            FailureClass::TimedOut => (b"timed_out", false),
            FailureClass::ContextTooLong => (b"context_too_long", false),
            FailureClass::Invalid => (b"invalid", false),
            FailureClass::Unauthorized => (b"unauthorized", false),
            FailureClass::Limit => (b"limit", false),
            FailureClass::Protocol => (b"protocol", false),
            FailureClass::Cancelled => (b"cancelled", false),
            FailureClass::Unknown(text) => (text, true),
        }
    }
}

impl Evidence {
    pub(crate) fn spelled(&self) -> (&[u8], bool) {
        match self {
            Evidence::Unsent => (b"unsent", false),
            Evidence::MaybeSent => (b"maybe_sent", false),
            Evidence::Response => (b"response", false),
            Evidence::Unknown(text) => (text, true),
        }
    }
}

impl AnswerClass {
    pub(crate) fn spelled(&self) -> (&[u8], bool) {
        match self {
            AnswerClass::Model => (b"model", false),
            AnswerClass::Budget => (b"budget", false),
            AnswerClass::Policy => (b"policy", false),
            AnswerClass::Cancelled => (b"cancelled", false),
            AnswerClass::Agent => (b"agent", false),
            AnswerClass::Host => (b"host", false),
            AnswerClass::Stale => (b"stale", false),
            AnswerClass::Transcript => (b"transcript", false),
            AnswerClass::Busy => (b"busy", false),
            AnswerClass::Invalid => (b"invalid", false),
            AnswerClass::Unknown(text) => (text, true),
        }
    }
}

impl Level {
    pub(crate) fn spelled(&self) -> (&[u8], bool) {
        match self {
            Level::Info => (b"info", false),
            Level::Warning => (b"warning", false),
            Level::Unknown(text) => (text, true),
        }
    }
}

impl NoticeKind {
    pub(crate) fn spelled(&self) -> (&[u8], bool) {
        match self {
            NoticeKind::ReasoningDropped => (b"reasoning_dropped", false),
            NoticeKind::CredentialUnauthorized => (b"credential_unauthorized", false),
            NoticeKind::AccountLimited => (b"account_limited", false),
            NoticeKind::AccountRefused => (b"account_refused", false),
            NoticeKind::TeardownAborted => (b"teardown_aborted", false),
            NoticeKind::ForcedExit => (b"forced_exit", false),
            NoticeKind::Unknown(text) => (text, true),
        }
    }
}

impl Role {
    pub(crate) fn spelled(&self) -> (&[u8], bool) {
        match self {
            Role::User => (b"user", false),
            Role::Assistant => (b"assistant", false),
            Role::Unknown(text) => (text, true),
        }
    }
}

impl FailurePhase {
    pub(crate) fn spelled(&self) -> (&[u8], bool) {
        match self {
            FailurePhase::Connect => (b"connect", false),
            FailurePhase::Handshake => (b"handshake", false),
            FailurePhase::Head => (b"head", false),
            FailurePhase::Idle => (b"idle", false),
            FailurePhase::Whole => (b"whole", false),
            FailurePhase::Unknown(value) => (value, true),
        }
    }
}

impl Forced {
    pub(crate) fn spelled(&self) -> (&[u8], bool) {
        match self {
            Forced::No => (b"no", false),
            Forced::Terminated => (b"terminated", false),
            Forced::Killed => (b"killed", false),
            Forced::Unknown(value) => (value, true),
        }
    }
}

impl BudgetStage {
    pub(crate) fn spelled(&self) -> (&[u8], bool) {
        match self {
            BudgetStage::Ordinary => (b"ordinary", false),
            BudgetStage::WindDown => (b"wind_down", false),
            BudgetStage::Unknown(value) => (value, true),
        }
    }
}

impl SharePart {
    pub(crate) fn spelled(&self) -> (&[u8], bool) {
        match self {
            SharePart::Turns => (b"turns", false),
            SharePart::Time => (b"time", false),
            SharePart::Unknown(value) => (value, true),
        }
    }
}

impl ShareBound {
    pub(crate) fn spelled(&self) -> (&[u8], bool) {
        match self {
            ShareBound::Reserve => (b"reserve", false),
            ShareBound::Deadline => (b"deadline", false),
            ShareBound::Unknown(value) => (value, true),
        }
    }
}

impl Backend {
    pub(crate) fn spelled(&self) -> (&[u8], bool) {
        match self {
            Backend::Direct => (b"direct", false),
            Backend::ClaudeCode => (b"claude_code", false),
            Backend::Unknown(name) => (name, true),
        }
    }
}

impl AccountState {
    pub(crate) fn spelled(&self) -> (&[u8], bool) {
        match self {
            AccountState::Limited => (b"limited", false),
            AccountState::Refused => (b"refused", false),
            AccountState::Unauthorized => (b"unauthorized", false),
            AccountState::Unknown(value) => (value, true),
        }
    }
}

impl ChildStop {
    pub(crate) fn spelled(&self) -> (&[u8], bool) {
        match self {
            ChildStop::End => (b"end", false),
            ChildStop::Budget => (b"budget", false),
            ChildStop::Failed => (b"failed", false),
            ChildStop::Closed => (b"closed", false),
            ChildStop::Unknown(text) => (text, true),
        }
    }
}

impl MessageRefusal {
    pub(crate) fn spelled(&self) -> (&[u8], bool) {
        match self {
            MessageRefusal::TooLarge => (b"too_large", false),
            MessageRefusal::InboxFull => (b"inbox_full", false),
            MessageRefusal::NameInUse => (b"name_in_use", false),
            MessageRefusal::Ending => (b"ending", false),
            MessageRefusal::StartRefused => (b"start_refused", false),
            MessageRefusal::Unknown(text) => (text, true),
        }
    }
}
