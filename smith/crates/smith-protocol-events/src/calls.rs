//! Bounded provider call joins for event starts and terminals.
//! The book keeps the session-issued owner, provider identity, public
//! conversation scope and start time until the terminal fact. It never owns
//! result content or a rendered result. Selected provider arguments wait in a
//! bounded input book until their start. Workspace caps arrive from
//! the tool's terminal; delegated content remains outside this projection.
//! Contracts: smith/docs/design/protocol/events.md, sections 3.1, 3.6 and 5; smith/docs/design/domain/run.md, section 11.

mod builtin;
mod jobs;

use alloc::boxed::Box;
#[cfg(test)]
use skein_lib::List;
use skein_lib::{Map, Time, Token};
use smith_domain::{session, tools};
use smith_events::{Event, Record, ToolCompleted, ToolStarted};

use crate::responses::Scope;
use crate::sinks::Sinks;

#[derive(Debug)]
struct Call {
    metadata: session::ToolCall,
    scope: Scope,
    at: Time,
    delivery: Option<smith_events::Delivered>,
    clamped: bool,
    omitted_bytes: u64,
}

/// Session batch capacities and aggregate provider metadata ownership.
#[derive(Clone, Copy)]
struct Bounds {
    slots: u32,
    bytes: u64,
}

impl Bounds {
    fn derive(limits: &session::Limits) -> Option<Bounds> {
        // The source session run slab holds two batches per session;
        // Provider identities are unchanged by application decoding and share
        // the raw answer's aggregate bytes. Decoded calls and replay envelopes
        // are never retained in this book.
        Some(Bounds {
            slots: limits.sessions.checked_mul(limits.parallel_tools)?.checked_mul(2)?,
            bytes: u64::from(limits.sessions).checked_mul(2)?.checked_mul(limits.answer_bytes)?,
        })
    }
}

/// One bounded set of provider calls which still owe a terminal.
#[derive(Debug)]
pub(crate) struct Book {
    calls: Map<Token, Call>,
    bytes: u64,
    bound: u64,
    children: crate::children::Book,
    landing: Option<Token>,
    inputs: crate::inputs::Book,
    builtin: builtin::Book,
}

impl Book {
    pub(crate) fn new(limits: &session::Limits, capture: crate::sinks::Capture) -> Book {
        let bounds = Bounds::derive(limits).expect("admitted session batch bounds fit");
        Book {
            calls: Map::with_capacity(bounds.slots),
            bytes: 0,
            bound: bounds.bytes,
            children: crate::children::Book::new(bounds.slots),
            landing: None,
            inputs: crate::inputs::Book::new(limits, capture),
            builtin: builtin::Book::new(limits),
        }
    }

    pub(crate) fn received(&mut self, owner: Token, completion: &smith_domain::llm::Completion) {
        self.inputs.received(owner, completion);
    }

    /// Join a validated root ask to its already observed provider call.
    pub(crate) fn called(&mut self, owner: Token, scope: Scope, ask: smith_domain::run::facts::Asked) {
        let call = self.calls.get(&owner).expect("session delegation precedes the causal root ask");
        assert_eq!(call.scope.run, scope.run, "the root call belongs to its provider activation");
        assert_eq!(call.scope.conversation, scope.conversation, "the root call belongs to its provider asker");
        match ask {
            smith_domain::run::facts::Asked::Deliver | smith_domain::run::facts::Asked::Finish => {
                assert!(self.landing.is_none(), "main's landing is an exclusive writer");
                self.landing = Some(owner);
            }
            smith_domain::run::facts::Asked::Wait
            | smith_domain::run::facts::Asked::Host
            | smith_domain::run::facts::Asked::SubAgent => {}
        }
    }

    /// Provider identity retained until this call's terminal drains.
    pub(crate) fn id(&self, owner: Token) -> Box<[u8]> {
        self.calls.get(&owner).expect("the returning child retains its provider call").metadata.id.clone()
    }

    /// Provider identity of the call which opened this child.
    pub(crate) fn child_opened(
        &mut self,
        owner: Token,
        number: u64,
        call_owner: Token,
        parent: Scope,
        granted: smith_domain::run::Granted,
        at: Time,
    ) -> Box<[u8]> {
        let call = self.calls.get_mut(&call_owner).expect("delegated sub-agent call precedes opening");
        assert_eq!(call.scope.run, parent.run, "the child and its call belong to the same activation");
        assert_eq!(call.scope.conversation, parent.conversation, "the child keeps its provider asker");
        call.clamped = lowered(granted.turns_set) || lowered(granted.time_set);
        self.children.opened(owner, number, call_owner, at);
        call.metadata.id.clone()
    }

    pub(crate) fn child_received(&mut self, owner: Token, usage: smith_domain::llm::Usage) {
        self.children.received(owner, usage);
    }

    pub(crate) fn child_ended(&mut self, owner: Token, end: smith_domain::run::facts::EndKind, at: Time) {
        self.children.ended(owner, end, at);
    }

    pub(crate) fn returned(&mut self, owner: Token, result: smith_domain::run::facts::Return) {
        self.children.returned(owner, result);
        match result {
            smith_domain::run::facts::Return::Answered { cut, .. } => {
                self.calls.get_mut(&owner).expect("child retains its enclosing provider call").omitted_bytes = cut;
            }
            smith_domain::run::facts::Return::Crossed
            | smith_domain::run::facts::Return::Waited
            | smith_domain::run::facts::Return::Withheld
            | smith_domain::run::facts::Return::HostAnswered
            | smith_domain::run::facts::Return::HostTooLarge
            | smith_domain::run::facts::Return::HostUnknown
            | smith_domain::run::facts::Return::HostRejected
            | smith_domain::run::facts::Return::Delivered
            | smith_domain::run::facts::Return::Nothing
            | smith_domain::run::facts::Return::DeliveryRefused
            | smith_domain::run::facts::Return::Accepted
            | smith_domain::run::facts::Return::Rejected
            | smith_domain::run::facts::Return::ChecksFailed
            | smith_domain::run::facts::Return::Stale
            | smith_domain::run::facts::Return::DeliveryFailed
            | smith_domain::run::facts::Return::Cancelled
            | smith_domain::run::facts::Return::TimedOut
            | smith_domain::run::facts::Return::Busy
            | smith_domain::run::facts::Return::Unanswered { .. }
            | smith_domain::run::facts::Return::Refused => {}
        }
        if self.landing == Some(owner) {
            self.landing = None;
        }
        if let Some(delivery) = returned_delivery(result) {
            let call = self.calls.get_mut(&owner).expect("root terminal keeps its provider call");
            call.delivery = Some(delivery);
        }
    }

    /// Root deliveries are serialized main writes; no name or text parser joins them.
    pub(crate) fn delivered(&mut self, status: smith_domain::run::DeliveryStatus) {
        let owner = self.landing.expect("delivery belongs to the exclusive main landing");
        let call = self.calls.get_mut(&owner).expect("landing retains its provider call");
        call.delivery = Some(match status {
            smith_domain::run::DeliveryStatus::Delivered => smith_events::Delivered::Delivered,
            smith_domain::run::DeliveryStatus::Nothing => smith_events::Delivered::Nothing,
            smith_domain::run::DeliveryStatus::Refused => smith_events::Delivered::Refused,
            smith_domain::run::DeliveryStatus::Failed(_) => smith_events::Delivered::Failed,
            smith_domain::run::DeliveryStatus::Stale => smith_events::Delivered::Stale,
        });
    }

    /// Project only a fact with provider metadata; kit lifecycle is not a call.
    pub(crate) fn fact(&mut self, origin: Time, fact: &session::Fact, sinks: &mut Sinks) {
        self.children.session(fact);
        if self.builtin.fact(origin, fact, &mut self.inputs, sinks) {
            return;
        }
        let Some(opener) = self.call_opener(fact) else { return };
        let Some(metadata) = &fact.call else { return };
        let scope = sinks.scope(opener).expect("conversation opening precedes its provider call");
        match fact.kind {
            session::FactKind::Tools { fact: tools::FactKind::Started { .. }, .. }
            | session::FactKind::DelegateStarted { .. } => {
                sinks.record(&self.started(origin, fact.at, opener, scope, metadata));
            }
            session::FactKind::Tools {
                fact: tools::FactKind::Answered { verdict, bytes, job, omitted_bytes, clamped, .. },
                ..
            } => {
                // An entrance refusal settles the session's call in
                // the same step, without a lower work-start fact or IO.
                if !self.calls.contains_key(&metadata.owner) {
                    sinks.record(&self.started(origin, fact.at, opener, scope, metadata));
                }
                let terminal = Terminal {
                    verdict: verdict_kind(verdict),
                    job: jobs::number(job),
                    exit: command_exit(verdict),
                    bytes,
                    omitted_bytes,
                    clamped,
                };
                sinks.record(&self.completed(origin, fact.at, metadata, terminal));
            }
            session::FactKind::JobEnded { .. } => sinks.record(&jobs::ended(origin, fact, scope, metadata)),
            session::FactKind::DelegateAnswered { bytes, error, .. } => {
                let verdict = if error { smith_events::Verdict::Error } else { smith_events::Verdict::Result };
                sinks.record(&self.completed(
                    origin,
                    fact.at,
                    metadata,
                    Terminal { job: None, verdict, exit: None, bytes, omitted_bytes: 0, clamped: false },
                ));
            }
            session::FactKind::DelegateCancelled { .. } | session::FactKind::DelegateWithheld { .. } => {
                let verdict = smith_events::Verdict::Withdrawn;
                let terminal = Terminal { job: None, verdict, exit: None, bytes: 0, omitted_bytes: 0, clamped: false };
                sinks.record(&self.completed(origin, fact.at, metadata, terminal));
            }
            session::FactKind::BuiltinStarted { .. }
            | session::FactKind::BuiltinAnswered { .. }
            | session::FactKind::CompactionStarted { .. }
            | session::FactKind::CompactionEnded { .. }
            | session::FactKind::Opened { .. }
            | session::FactKind::CompletionStarted { .. }
            | session::FactKind::CompletionAnswered { .. }
            | session::FactKind::CompletionFailed { .. }
            | session::FactKind::CompletionCancelled { .. }
            | session::FactKind::CompletionRetried { .. }
            | session::FactKind::ReasoningDropped { .. }
            | session::FactKind::Yielded { .. }
            | session::FactKind::Used { .. }
            | session::FactKind::Ended { .. }
            | session::FactKind::Tools {
                fact:
                    tools::FactKind::Opened { .. }
                    | tools::FactKind::Refused { .. }
                    | tools::FactKind::Closing { .. }
                    | tools::FactKind::JobEnded { .. }
                    | tools::FactKind::Closed { .. },
                ..
            } => unreachable!("only selected call facts reach projection"),
        }
    }

    fn call_opener(&mut self, fact: &session::Fact) -> Option<Token> {
        match fact.kind {
            session::FactKind::Tools {
                opener,
                fact: tools::FactKind::Started { .. } | tools::FactKind::Answered { .. },
            }
            | session::FactKind::DelegateStarted { opener, .. }
            | session::FactKind::DelegateAnswered { opener, .. }
            | session::FactKind::DelegateCancelled { opener }
            | session::FactKind::DelegateWithheld { opener }
            | session::FactKind::JobEnded { opener, .. } => Some(opener),
            session::FactKind::Ended { opener, .. } => {
                self.inputs.ended(opener);
                None
            }
            session::FactKind::BuiltinStarted { .. }
            | session::FactKind::BuiltinAnswered { .. }
            | session::FactKind::CompactionStarted { .. }
            | session::FactKind::CompactionEnded { .. }
            | session::FactKind::Opened { .. }
            | session::FactKind::CompletionStarted { .. }
            | session::FactKind::CompletionAnswered { .. }
            | session::FactKind::CompletionFailed { .. }
            | session::FactKind::CompletionCancelled { .. }
            | session::FactKind::CompletionRetried { .. }
            | session::FactKind::ReasoningDropped { .. }
            | session::FactKind::Yielded { .. }
            | session::FactKind::Used { .. }
            | session::FactKind::Tools {
                fact:
                    tools::FactKind::Opened { .. }
                    | tools::FactKind::Refused { .. }
                    | tools::FactKind::Closing { .. }
                    | tools::FactKind::JobEnded { .. }
                    | tools::FactKind::Closed { .. },
                ..
            } => None,
        }
    }

    fn started(&mut self, origin: Time, at: Time, opener: Token, scope: Scope, metadata: &session::ToolCall) -> Record {
        let input = self.inputs.take(opener, metadata);
        let bytes = metadata_bytes(metadata);
        self.bytes = self.bytes.checked_add(bytes).expect("bounded call identity sum fits");
        assert!(self.bytes <= self.bound, "source batches bound copied provider identity");
        let old = self
            .calls
            .insert(
                metadata.owner,
                Call { metadata: metadata.clone(), scope, at, delivery: None, clamped: false, omitted_bytes: 0 },
            )
            .expect("source session run slab bounds active event calls");
        assert!(old.is_none(), "one provider start per session call owner");
        let source = match metadata.source {
            session::ToolSource::Workspace => smith_events::Source::Workspace,
            session::ToolSource::Run => smith_events::Source::Run,
            session::ToolSource::Host => smith_events::Source::Host,
        };
        let effect = match metadata.effect {
            tools::Effect::Read => smith_events::Effect::Read,
            tools::Effect::Write => smith_events::Effect::Write,
        };
        Record {
            t_ms: millis(at, origin),
            event: Event::ToolStarted(ToolStarted {
                run: scope.run,
                conversation: scope.conversation,
                call: metadata.id.clone(),
                tool: metadata.name.clone(),
                source,
                effect,
                deadline_ms: Some(millis(metadata.deadline, at)),
                input_bytes: metadata.input_bytes,
                input,
            }),
        }
    }

    fn completed(&mut self, origin: Time, at: Time, metadata: &session::ToolCall, terminal: Terminal) -> Record {
        let call = self.calls.remove(&metadata.owner).expect("terminal follows its one start");
        assert_eq!(call.metadata, *metadata, "terminal keeps its exact parent call metadata");
        self.bytes = self.bytes.checked_sub(metadata_bytes(&call.metadata)).expect("identity was retained at start");
        Record {
            t_ms: millis(at, origin),
            event: Event::ToolCompleted(ToolCompleted {
                run: call.scope.run,
                conversation: call.scope.conversation,
                call: call.metadata.id,
                tool: call.metadata.name,
                verdict: terminal.verdict,
                exit: terminal.exit,
                job: terminal.job,
                bytes: terminal.bytes,
                omitted_bytes: terminal.omitted_bytes.checked_add(call.omitted_bytes).expect("bounded tool omissions"),
                clamped: terminal.clamped || call.clamped,
                duration_ms: millis(at, call.at),
                child: self.children.take(metadata.owner),
                delivery: call.delivery,
            }),
        }
    }
}

struct Terminal {
    job: Option<u64>,
    verdict: smith_events::Verdict,
    exit: Option<smith_events::CommandExit>,
    bytes: u64,
    omitted_bytes: u64,
    clamped: bool,
}

fn lowered(set: smith_domain::run::Set) -> bool {
    match set {
        smith_domain::run::Set::Asked | smith_domain::run::Set::Pool => false,
        smith_domain::run::Set::Reserve | smith_domain::run::Set::Deadline => true,
    }
}

fn returned_delivery(result: smith_domain::run::facts::Return) -> Option<smith_events::Delivered> {
    match result {
        smith_domain::run::facts::Return::Delivered => Some(smith_events::Delivered::Delivered),
        smith_domain::run::facts::Return::Nothing => Some(smith_events::Delivered::Nothing),
        smith_domain::run::facts::Return::DeliveryRefused => Some(smith_events::Delivered::Refused),
        smith_domain::run::facts::Return::DeliveryFailed => Some(smith_events::Delivered::Failed),
        smith_domain::run::facts::Return::Stale => Some(smith_events::Delivered::Stale),
        smith_domain::run::facts::Return::Crossed
        | smith_domain::run::facts::Return::Waited
        | smith_domain::run::facts::Return::Withheld
        | smith_domain::run::facts::Return::HostAnswered
        | smith_domain::run::facts::Return::HostTooLarge
        | smith_domain::run::facts::Return::HostUnknown
        | smith_domain::run::facts::Return::HostRejected
        | smith_domain::run::facts::Return::Accepted
        | smith_domain::run::facts::Return::Rejected
        | smith_domain::run::facts::Return::ChecksFailed
        | smith_domain::run::facts::Return::Cancelled
        | smith_domain::run::facts::Return::TimedOut
        | smith_domain::run::facts::Return::Busy
        | smith_domain::run::facts::Return::Answered { .. }
        | smith_domain::run::facts::Return::Unanswered { .. }
        | smith_domain::run::facts::Return::Refused => None,
    }
}

fn millis(at: Time, since: Time) -> u64 {
    at.saturating_since(since).as_nanos() / 1_000_000
}

fn metadata_bytes(metadata: &session::ToolCall) -> u64 {
    let id = u64::try_from(metadata.id.len()).expect("bounded provider id fits u64");
    id.checked_add(u64::try_from(metadata.name.len()).expect("bounded tool name fits u64"))
        .expect("bounded provider identity fits u64")
}

fn command_exit(verdict: tools::Verdict) -> Option<smith_events::CommandExit> {
    match verdict {
        tools::Verdict::Exited { exit } => match exit {
            tools::Exit::Code { code } => Some(smith_events::CommandExit::Code { code: i64::from(code) }),
            tools::Exit::Signal { signal } => Some(smith_events::CommandExit::Signal { signal: u64::from(signal) }),
            tools::Exit::TimedOut => None,
        },
        tools::Verdict::Running
        | tools::Verdict::Stopped
        | tools::Verdict::Read
        | tools::Verdict::Listed
        | tools::Verdict::Found
        | tools::Verdict::Removed
        | tools::Verdict::NotEmpty
        | tools::Verdict::Written { .. }
        | tools::Verdict::Edited { .. }
        | tools::Verdict::NotGranted
        | tools::Verdict::Outside
        | tools::Verdict::ReadOnly
        | tools::Verdict::TooLong
        | tools::Verdict::NotFound
        | tools::Verdict::NotFile
        | tools::Verdict::Linked
        | tools::Verdict::Protected
        | tools::Verdict::NotDirectory
        | tools::Verdict::TooLarge
        | tools::Verdict::NotRead
        | tools::Verdict::Stale
        | tools::Verdict::NoMatch { .. }
        | tools::Verdict::Ambiguous { .. }
        | tools::Verdict::Unchanged { .. }
        | tools::Verdict::Failed { .. }
        | tools::Verdict::TimedOut
        | tools::Verdict::Cancelled
        | tools::Verdict::Busy
        | tools::Verdict::NulByte => None,
    }
}

fn verdict_kind(verdict: tools::Verdict) -> smith_events::Verdict {
    match verdict {
        tools::Verdict::Running => smith_events::Verdict::Running,
        tools::Verdict::Stopped => smith_events::Verdict::Stopped,
        tools::Verdict::Read => smith_events::Verdict::Read,
        tools::Verdict::Listed => smith_events::Verdict::Listed,
        tools::Verdict::Found => smith_events::Verdict::Found,
        tools::Verdict::Written { .. } => smith_events::Verdict::Written,
        tools::Verdict::Edited { .. } => smith_events::Verdict::Edited,
        tools::Verdict::Exited { exit } => match exit {
            tools::Exit::Code { .. } | tools::Exit::Signal { .. } => smith_events::Verdict::Exited,
            tools::Exit::TimedOut => smith_events::Verdict::TimedOut,
        },
        tools::Verdict::NotGranted => smith_events::Verdict::NotGranted,
        tools::Verdict::Outside => smith_events::Verdict::Outside,
        tools::Verdict::ReadOnly => smith_events::Verdict::ReadOnly,
        tools::Verdict::TooLong => smith_events::Verdict::TooLong,
        tools::Verdict::NotFound => smith_events::Verdict::NotFound,
        tools::Verdict::Removed => smith_events::Verdict::Removed,
        tools::Verdict::NotEmpty => smith_events::Verdict::NotEmpty,
        tools::Verdict::NotFile => smith_events::Verdict::NotFile,
        tools::Verdict::Linked => smith_events::Verdict::Linked,
        tools::Verdict::Protected => smith_events::Verdict::Protected,
        tools::Verdict::NotDirectory => smith_events::Verdict::NotDirectory,
        tools::Verdict::TooLarge => smith_events::Verdict::TooLarge,
        tools::Verdict::NotRead => smith_events::Verdict::NotRead,
        tools::Verdict::Stale => smith_events::Verdict::Stale,
        tools::Verdict::NoMatch { .. } => smith_events::Verdict::NoMatch,
        tools::Verdict::Ambiguous { .. } => smith_events::Verdict::Ambiguous,
        tools::Verdict::Unchanged { .. } => smith_events::Verdict::Unchanged,
        tools::Verdict::Failed { .. } => smith_events::Verdict::Failed,
        tools::Verdict::TimedOut => smith_events::Verdict::TimedOut,
        tools::Verdict::Cancelled => smith_events::Verdict::Cancelled,
        tools::Verdict::Busy => smith_events::Verdict::Busy,
        tools::Verdict::NulByte => smith_events::Verdict::NulByte,
    }
}

/// Whether call joins and their metadata counter fit (skein/docs/foundation/programming-model.md, 6.3).
#[must_use]
pub(crate) fn valid(limits: &session::Limits) -> bool {
    Bounds::derive(limits).is_some() && builtin::valid(limits)
}

#[cfg(test)]
mod tests;
