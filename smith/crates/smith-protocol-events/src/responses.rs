//! Bounded joins between logical session facts and LLM hand-offs.
//! Each live conversation keeps its opening model, current intent and at most
//! one provider attempt and one temporary compaction scope. Hand-offs
//! retain their public conversation number until their terminal. Intents can be
//! revised before hand-off; only `request`
//! assigns a run-wide response number. Raw usage is counted before accounting;
//! rejected reports remain in nullable totals, with sticky per-field overflow.
//! A content cursor skips already recorded completions; original completion
//! blocks and observed transport measures wait for their terminal. Unknown
//! measures remain absent, and observed zero durations remain zero. No provider
//! replay bytes are retained. Typed diagnostics join the request terminal,
//! then its conversation and failed main answer; child diagnostics never supply
//! the main failure. Capture none keeps no provider diagnostic content.
//! Contracts: smith/docs/design/protocol/events.md, sections 3.4–3.5, 3.10–3.11 and 4.

#![expect(clippy::manual_map, reason = "the strict subset keeps bounded entries explicit and uses no closures")]

mod retry;

use alloc::boxed::Box;

use skein_lib::{List, Map, Time, Token};
use smith_domain::{llm, run, session};
use smith_events::{
    CompletionFailure, ConversationClosed, ConversationEnd, ConversationOpened, Event, Model, Outcome, Record,
    ResponseCompleted, ResponseStarted, Stop, Usage,
};

/// Public stream identity of one internal conversation owner.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Scope {
    pub(crate) run: u64,
    pub(crate) conversation: u64,
}

/// A bounded response join table, owned by the event sink.
#[derive(Debug)]
pub(crate) struct Book {
    conversations: Map<Token, Conversation>,
    next: u64,
    reported: crate::observations::Reported,
    run: Option<u64>,
    next_conversation: u64,
    main_failure: Option<RunFailure>,
}

#[derive(Debug)]
struct RunFailure {
    run: u64,
    model: Model,
    attempts: u64,
    failure: CompletionFailure,
}

#[derive(Debug)]
struct Conversation {
    run: u64,
    number: u64,
    thread: u64,
    model: Model,
    opened: Time,
    intent: Option<Intent>,
    attempt: Attempt,
    spent: u64,
    reported: crate::observations::Reported,
    admitted: bool,
    root_end: Option<(Time, run::facts::EndKind)>,
    recorded: Option<u32>,
    main: bool,
    attempts: u64,
    last_failure: Option<CompletionFailure>,
    compaction: Option<Compaction>,
}

#[derive(Debug)]
struct Compaction {
    charter_model: Model,
    number: u64,
    at: Time,
    usage: crate::observations::Reported,
    spent: u64,
    turns: u32,
}

#[derive(Clone, Copy, Debug)]
struct Intent {
    turn: u64,
    retry: u32,
    window: u32,
}

#[derive(Debug)]
struct Started {
    opening: retry::Opening,
    number: u64,
    conversation: u64,
    at: Time,
    owner: Token,
    observed: bool,
    received: Option<Received>,
    completion: Option<smith_events::Completion>,
    measures: Option<smith_protocol_llm::Measures>,
    failure: Option<CompletionFailure>,
}

#[derive(Clone, Copy, Debug)]
struct Received {
    usage: llm::Usage,
    oversized: u64,
    cut: u64,
    blocks: u64,
}

/// The hand-off's one terminal, completed only after accepted accounting.
#[derive(Debug)]
enum Attempt {
    /// No provider request was handed off, or its terminal was already recorded.
    Idle,
    /// A request was handed to the LLM component and owes one terminal.
    Started(Started),
    /// The completion answered; its accepted Used or end fact still follows.
    Answered { started: Started, at: Time, stop: Stop, blocks: u64, calls: u64, invalid: u64 },
    /// The failure's retry decision or session end still follows.
    Failed { started: Started, at: Time, failure: CompletionFailure },
}

impl Book {
    pub(crate) fn new(conversations: u32) -> Book {
        Book {
            conversations: Map::with_capacity(conversations),
            next: 1,
            reported: crate::observations::Reported::NONE,
            main_failure: None,
            run: None,
            next_conversation: 0,
        }
    }

    pub(crate) fn next_conversation(&mut self, run: u64) -> u64 {
        if self.run != Some(run) {
            self.run = Some(run);
            self.next_conversation = 0;
        }
        let number = self.next_conversation;
        self.next_conversation = number.checked_add(1).expect("conversation names cannot exhaust");
        number
    }

    pub(crate) fn opened(&mut self, at: Time, owner: Token, opening: &ConversationOpened) {
        self.run = Some(opening.run);
        self.next_conversation =
            self.next_conversation.max(opening.conversation.checked_add(1).expect("bounded names"));
        let old = self
            .conversations
            .insert(
                owner,
                Conversation {
                    run: opening.run,
                    number: opening.conversation,
                    thread: opening.thread,
                    model: opening.model.clone(),
                    opened: at,
                    intent: None,
                    attempt: Attempt::Idle,
                    spent: 0,
                    reported: crate::observations::Reported::NONE,
                    admitted: false,
                    root_end: None,
                    recorded: None,
                    main: match opening.kind {
                        smith_events::ConversationKind::Main => true,
                        smith_events::ConversationKind::Child
                        | smith_events::ConversationKind::Compaction
                        | smith_events::ConversationKind::Unknown(_) => false,
                    },
                    attempts: 0,
                    last_failure: None,
                    compaction: None,
                },
            )
            .expect("one join row per bounded live conversation");
        assert!(old.is_none(), "each conversation opens once");
    }

    pub(crate) fn starting(&mut self) {
        assert!(self.conversations.is_empty(), "prior run's conversations settled");
        self.next = 1;
        self.run = None;
        self.next_conversation = 0;
        self.reported = crate::observations::Reported::NONE;
        self.main_failure = None;
    }

    pub(crate) fn scope(&self, owner: Token) -> Option<Scope> {
        let conversation = self.conversations.get(&owner)?;
        Some(Scope { run: conversation.run, conversation: conversation.number })
    }

    pub(crate) fn request(&mut self, now: Time, origin: Time, request_owner: Token, prompt: &llm::Prompt) -> Record {
        let mut owner = None;
        for (name, conversation) in &self.conversations {
            if conversation.thread == u64::from(prompt.affinity.thread) {
                assert!(owner.is_none(), "one live conversation per affinity ordinal");
                owner = Some(*name);
            }
        }
        let owner = owner.expect("the opening fact is drained before its request");
        let conversation = self.conversations.get_mut(&owner).expect("found above");
        assert_eq!(conversation.model.model.as_ref(), prompt.model.as_ref(), "the request uses its opening's model");
        match &conversation.attempt {
            Attempt::Idle => {}
            Attempt::Started(_) | Attempt::Answered { .. } | Attempt::Failed { .. } => {
                unreachable!("a conversation hands off one provider request at a time")
            }
        }
        let intent = conversation.intent.take().expect("each request has a surviving logical intent");
        let number = self.next;
        self.next = self.next.checked_add(1).expect("a process cannot exhaust response names");
        conversation.attempts = u64::from(intent.retry).checked_add(1).expect("u32 retry count plus one fits u64");
        conversation.last_failure = None;
        let response_conversation = match &conversation.compaction {
            Some(compaction) => compaction.number,
            None => conversation.number,
        };
        conversation.attempt = Attempt::Started(Started {
            opening: retry::Opening {
                intent,
                output_tokens: u64::from(prompt.max_tokens),
                tool_choice: crate::request_tool_choice(&prompt.choice),
            },
            number,
            conversation: response_conversation,
            at: now,
            owner: request_owner,
            observed: false,
            received: None,
            completion: None,
            measures: None,
            failure: None,
        });
        Record {
            t_ms: millis(now, origin),
            event: Event::ResponseStarted(ResponseStarted {
                run: conversation.run,
                conversation: response_conversation,
                response: number,
                turn: match &conversation.compaction {
                    Some(_) => 1,
                    None => intent.turn,
                },
                attempt: u64::from(intent.retry),
                window: u64::from(intent.window),
                model: conversation.model.clone(),
                messages: u64::try_from(prompt.messages.len()).expect("admitted prompt messages"),
                output_tokens: u64::from(prompt.max_tokens),
                tool_choice: crate::request_tool_choice(&prompt.choice),
                prompt: None,
            }),
        }
    }

    fn owner_for(&self, request_owner: Token) -> Option<Token> {
        for (owner, conversation) in &self.conversations {
            let started = match &conversation.attempt {
                Attempt::Idle => continue,
                Attempt::Started(started) | Attempt::Answered { started, .. } | Attempt::Failed { started, .. } => {
                    started
                }
            };
            if started.owner == request_owner {
                return Some(*owner);
            }
        }
        None
    }

    pub(crate) fn failed(
        &mut self,
        request_owner: Token,
        failure: llm::Failure,
        evidence: llm::Evidence,
        detail: &[u8],
        capture: crate::sinks::Capture,
        report: &llm::Report,
    ) {
        let owner = self.owner_for(request_owner).expect("failure belongs to a request");
        let conversation = self.conversations.get_mut(&owner).expect("found above");
        let mut observed = completion_failure(failure, evidence);
        observed.field = provider_field(&report.field);
        observed.code = provider_field(&report.code);
        observed.request_id = provider_field(&report.request);
        observed.detail = match capture {
            crate::sinks::Capture::None => None,
            crate::sinks::Capture::Calls | crate::sinks::Capture::Everything => Some(detail.into()),
        };
        match &mut conversation.attempt {
            Attempt::Started(started) => {
                assert!(started.failure.is_none(), "one failure observation per terminal");
                started.failure = Some(observed);
            }
            Attempt::Idle | Attempt::Answered { .. } | Attempt::Failed { .. } => {
                unreachable!("failure precedes its domain fact")
            }
        }
    }

    pub(crate) fn answer_failure(&mut self, record: &mut smith_events::RunCompleted) {
        let Some(observed) = self.main_failure.take() else { return };
        if observed.run != record.run {
            return;
        }
        let Some(failure) = &mut record.failure else { return };
        match &failure.class {
            smith_events::AnswerClass::Model => {}
            smith_events::AnswerClass::Budget
            | smith_events::AnswerClass::Policy
            | smith_events::AnswerClass::Cancelled
            | smith_events::AnswerClass::Agent
            | smith_events::AnswerClass::Host
            | smith_events::AnswerClass::Stale
            | smith_events::AnswerClass::Transcript
            | smith_events::AnswerClass::Busy
            | smith_events::AnswerClass::Invalid
            | smith_events::AnswerClass::Unknown(_) => return,
        }
        let Some(completion) = &failure.completion else { return };
        assert_eq!(completion.class, observed.failure.class, "the main terminal supplies the failed answer");
        assert_eq!(
            completion.evidence, observed.failure.evidence,
            "the main answer agrees with its transport evidence"
        );
        failure.endpoint = Some(observed.model.endpoint);
        failure.model = Some(observed.model.model);
        failure.attempts = Some(observed.attempts);
        failure.completion = Some(observed.failure);
    }

    pub(crate) fn observation(&mut self, request_owner: Token) -> smith_protocol_llm::Observation {
        let owner = self.owner_for(request_owner).expect("request retains its conversation");
        let conversation = self.conversations.get_mut(&owner).expect("found above");
        if let Attempt::Started(started) = &mut conversation.attempt {
            started.observed = true;
        }
        match conversation.recorded {
            Some(messages) => smith_protocol_llm::Observation { first: false, messages },
            None => smith_protocol_llm::Observation { first: true, messages: 0 },
        }
    }

    pub(crate) fn observed(&self, request_owner: Token) -> bool {
        let Some(owner) = self.owner_for(request_owner) else {
            return false;
        };
        match &self.conversations.get(&owner).expect("found above").attempt {
            Attempt::Started(started) => started.observed,
            Attempt::Idle | Attempt::Answered { .. } | Attempt::Failed { .. } => false,
        }
    }

    pub(crate) fn prompt_recorded(&mut self, request_owner: Token, messages: u32) {
        let owner = self.owner_for(request_owner).expect("prompt belongs to a request");
        self.conversations.get_mut(&owner).expect("found above").recorded = Some(messages);
    }

    pub(crate) fn completion(&mut self, request_owner: Token, completion: smith_events::Completion) {
        let owner = self.owner_for(request_owner).expect("completion belongs to a request");
        let conversation = self.conversations.get_mut(&owner).expect("found above");
        let started = match &mut conversation.attempt {
            Attempt::Started(started) => started,
            Attempt::Idle | Attempt::Answered { .. } | Attempt::Failed { .. } => {
                unreachable!("capture precedes the domain terminal")
            }
        };
        assert!(started.completion.is_none(), "one observed completion per request");
        started.completion = Some(completion);
    }

    pub(crate) fn measured(&mut self, request_owner: Token, measures: smith_protocol_llm::Measures) {
        let owner = self.owner_for(request_owner).expect("transport measures belong to a request");
        let conversation = self.conversations.get_mut(&owner).expect("found above");
        let started = match &mut conversation.attempt {
            Attempt::Started(started) => started,
            Attempt::Idle | Attempt::Answered { .. } | Attempt::Failed { .. } => {
                unreachable!("transport measures precede the domain terminal")
            }
        };
        assert!(started.measures.is_none(), "one connection terminal per request");
        started.measures = Some(measures);
    }

    pub(crate) fn delta(&self, at: Time, origin: Time, request_owner: Token, block: u32, text: Box<[u8]>) -> Record {
        let owner = self.owner_for(request_owner).expect("delta belongs to a request");
        let conversation = self.conversations.get(&owner).expect("found above");
        let (response, response_conversation) = match &conversation.attempt {
            Attempt::Started(started) => (started.number, started.conversation),
            Attempt::Idle | Attempt::Answered { .. } | Attempt::Failed { .. } => {
                unreachable!("deltas precede the domain terminal")
            }
        };
        Record {
            t_ms: millis(at, origin),
            event: Event::TextDelta(smith_events::TextDelta {
                run: conversation.run,
                conversation: response_conversation,
                response,
                block: u64::from(block),
                text,
            }),
        }
    }

    pub(crate) fn received(&mut self, owner: Token, completion: &llm::Completion) -> Token {
        let mut found = None;
        for (name, conversation) in &self.conversations {
            match &conversation.attempt {
                Attempt::Started(started) if started.owner == owner => {
                    found = Some(*name);
                    break;
                }
                Attempt::Idle | Attempt::Started(_) | Attempt::Answered { .. } | Attempt::Failed { .. } => {}
            }
        }
        let conversation_owner = found.expect("typed answer retains its hand-off identity");
        let conversation = self.conversations.get_mut(&conversation_owner).expect("found above");
        conversation.reported.add(completion.usage);
        if let Some(compaction) = &mut conversation.compaction {
            compaction.usage.add(completion.usage);
        }
        self.reported.add(completion.usage);
        match &mut conversation.attempt {
            Attempt::Started(started) => {
                let (oversized, cut) = rejected_calls(completion);
                assert!(started.received.is_none(), "one typed component answer per hand-off");
                let blocks = u64::try_from(completion.content.len())
                    .expect("bounded blocks")
                    .checked_add(u64::try_from(completion.reasoning_dropped.len()).expect("bounded drops"))
                    .expect("bounded original completion blocks");
                started.received = Some(Received { usage: completion.usage, oversized, cut, blocks });
            }
            Attempt::Idle | Attempt::Answered { .. } | Attempt::Failed { .. } => {
                unreachable!("found active hand-off above")
            }
        }
        conversation.recorded = match conversation.recorded {
            Some(messages) => {
                Some(messages.checked_add(1).expect("the received assistant message fits admitted history"))
            }
            None => None,
        };
        conversation_owner
    }

    pub(crate) fn reported(&self) -> Usage {
        self.reported.usage()
    }

    pub(crate) fn root_ended(&mut self, at: Time, owner: Token, end: run::facts::EndKind) {
        // SessionEnded can have already closed a child in a previous drain.
        if let Some(conversation) = self.conversations.get_mut(&owner) {
            conversation.root_end = Some((at, end));
        }
    }

    /// Called only after the complete gathered fact batch has been drained.
    /// A root refusal before session admission still closes its opening once.
    pub(crate) fn facts_drained(&mut self, origin: Time) -> Option<Record> {
        let mut refused = None;
        for (owner, conversation) in &self.conversations {
            if !conversation.admitted && conversation.root_end.is_some() {
                refused = Some(*owner);
                break;
            }
        }
        let owner = refused?;
        let conversation = self.conversations.remove(&owner).expect("selected above");
        let (at, end) = conversation.root_end.expect("selected above");
        let end = match end {
            run::facts::EndKind::Busy | run::facts::EndKind::Invalid => ConversationEnd::Refused,
            run::facts::EndKind::TranscriptRefused { .. } => ConversationEnd::Transcript,
            run::facts::EndKind::PriceOverflow
            | run::facts::EndKind::UsageOverflow
            | run::facts::EndKind::Receiving(_)
            | run::facts::EndKind::Closed
            | run::facts::EndKind::Fault
            | run::facts::EndKind::Budget(_) => {
                unreachable!("other conversation ends follow a session admission or its Ended fact")
            }
        };
        Some(Record {
            t_ms: millis(at, origin),
            event: Event::ConversationClosed(ConversationClosed {
                run: conversation.run,
                conversation: conversation.number,
                end,
                which: None,
                failure: None,
                turns: 0,
                usage: conversation.reported.usage(),
                spent: 0,
                duration_ms: millis(at, conversation.opened),
            }),
        })
    }

    /// At most two records: an unanswered completion's terminal, then its conversation end.
    #[expect(
        clippy::too_many_lines,
        reason = "one exhaustive session-fact join; each entrance keeps at most two records"
    )]
    pub(crate) fn fact(&mut self, origin: Time, fact: &session::Fact) -> List<Record> {
        let mut records = List::with_capacity(2);
        match fact.kind.clone() {
            session::FactKind::CompactionStarted {
                opener, model, model_bytes, model_window, output, effort, ..
            } => {
                let run = self.conversations.get(&opener).expect("compaction follows opening").run;
                let number = self.next_conversation(run);
                let conversation = self.conversations.get_mut(&opener).expect("found above");
                if let Some(record) = terminal(conversation, origin, unknown_usage(), 0, None) {
                    records.push(record).expect("the prior failed response settles before the summary request");
                }
                assert!(conversation.compaction.is_none(), "one active compaction per conversation");
                let effort = match effort {
                    Some((bytes, length)) => Some(Box::from(
                        bytes.get(..usize::try_from(length).expect("bounded effort")).expect("admitted effort"),
                    )),
                    None => None,
                };
                let model = Model {
                    account: conversation.model.account,
                    backend: conversation.model.backend.clone(),
                    endpoint: conversation.model.endpoint.clone(),
                    model: Box::from(
                        model.get(..usize::try_from(model_bytes).expect("bounded model")).expect("admitted model"),
                    ),
                    window_tokens: u64::from(model_window),
                    output_tokens: u64::from(output),
                    effort,
                };
                let charter_model = core::mem::replace(&mut conversation.model, model);
                conversation.compaction = Some(Compaction {
                    charter_model,
                    number,
                    at: fact.at,
                    usage: crate::observations::Reported::NONE,
                    spent: 0,
                    turns: 0,
                });
                records
                    .push(Record {
                        t_ms: millis(fact.at, origin),
                        event: Event::ConversationOpened(ConversationOpened {
                            run,
                            conversation: number,
                            thread: conversation.thread,
                            model: conversation.model.clone(),
                            kind: smith_events::ConversationKind::Compaction,
                            parent: Some(conversation.number),
                            call: None,
                            requested: None,
                            share: None,
                            clamped: List::with_capacity(0),
                        }),
                    })
                    .expect("one compaction opening");
            }
            session::FactKind::CompactionEnded { opener, end, .. } => {
                let conversation = self.conversations.get_mut(&opener).expect("end follows opening");
                if let Some(record) = terminal(conversation, origin, unknown_usage(), 0, None) {
                    records.push(record).expect("the summary response settles before its conversation");
                }
                let compaction = conversation.compaction.take().expect("one compaction terminal");
                match end {
                    session::Compacted::Opened => {
                        conversation.recorded = None;
                        conversation.model = compaction.charter_model;
                    }
                    session::Compacted::Failed => {}
                }
                records
                    .push(Record {
                        t_ms: millis(fact.at, origin),
                        event: Event::ConversationClosed(ConversationClosed {
                            run: conversation.run,
                            conversation: compaction.number,
                            end: match end {
                                session::Compacted::Opened => ConversationEnd::Closed,
                                session::Compacted::Failed => ConversationEnd::Failed,
                            },
                            which: None,
                            failure: match end {
                                session::Compacted::Opened => None,
                                session::Compacted::Failed => conversation.last_failure.clone(),
                            },
                            turns: u64::from(compaction.turns),
                            usage: compaction.usage.usage(),
                            spent: compaction.spent,
                            duration_ms: millis(fact.at, compaction.at),
                        }),
                    })
                    .expect("one compaction terminal");
            }
            session::FactKind::Opened { opener } => {
                self.conversations.get_mut(&opener).expect("session admission follows opening").admitted = true;
            }
            session::FactKind::CompletionStarted { opener, attempt, window, .. } => {
                let turn = fact.response.expect("intent carries its session turn").number;
                let conversation = self.conversations.get_mut(&opener).expect("intent follows opening");
                conversation.intent = Some(Intent { turn, retry: attempt, window });
            }
            session::FactKind::CompletionAnswered { opener, stop, blocks, calls, invalid } => {
                let conversation = self.conversations.get_mut(&opener).expect("answer follows opening");
                let started = take_started(&mut conversation.attempt);
                match started {
                    Some(started) => {
                        conversation.attempt = Attempt::Answered {
                            started,
                            at: fact.at,
                            stop: stop_kind(stop),
                            blocks: u64::from(blocks),
                            calls: u64::from(calls),
                            invalid: u64::from(invalid),
                        }
                    }
                    None => unreachable!("a provider answer follows a hand-off"),
                }
            }
            session::FactKind::Used { opener, usage } => {
                let conversation = self.conversations.get_mut(&opener).expect("charge follows opening");
                let spent =
                    fact.response.expect("charge has response identity").spent.expect("Used has the accepted charge");
                conversation.spent = conversation
                    .spent
                    .checked_add(spent)
                    .expect("session accepted this representable cumulative spend");
                if let Some(compaction) = &mut conversation.compaction {
                    compaction.spent = spent;
                    compaction.turns = 1;
                }
                let record = terminal(conversation, origin, crate::reported_usage(usage), spent, None)
                    .expect("an accepted charge follows its pending answer");
                records.push(record).expect("one response terminal");
            }
            session::FactKind::CompletionFailed { opener, failure, evidence } => {
                let conversation = self.conversations.get_mut(&opener).expect("failure follows opening");
                let started = take_started(&mut conversation.attempt);
                match started {
                    Some(mut started) => {
                        let projected = completion_failure(failure, evidence);
                        let observed = match started.failure.take() {
                            Some(observed) => {
                                assert_eq!(
                                    observed.class, projected.class,
                                    "the domain fact preserves the failure class"
                                );
                                assert_eq!(
                                    observed.evidence, projected.evidence,
                                    "the domain fact preserves transport evidence"
                                );
                                observed
                            }
                            None => projected,
                        };
                        conversation.last_failure = Some(observed.clone());
                        conversation.attempt = Attempt::Failed { started, at: fact.at, failure: observed };
                    }
                    None => unreachable!("a component failure follows a hand-off"),
                }
            }
            session::FactKind::CompletionRetried { opener, delay, .. } => {
                let conversation = self.conversations.get_mut(&opener).expect("retry follows opening");
                let record = terminal(conversation, origin, unknown_usage(), 0, Some(delay.as_millis()))
                    .expect("a retry follows one pending failure");
                records.push(record).expect("one response terminal");
            }
            session::FactKind::CompletionCancelled { opener } => {
                let conversation = self.conversations.get_mut(&opener).expect("cancellation follows opening");
                let started = take_started(&mut conversation.attempt);
                // The domain can abandon an unsent logical intent; that is not a provider attempt.
                if let Some(started) = started {
                    records.push(cancelled(conversation, origin, started, fact.at)).expect("one cancellation");
                }
            }
            session::FactKind::Ended { opener, end, turns, usage: _ } => {
                let Some(mut conversation) = self.conversations.remove(&opener) else { return records };
                if let Some(record) = terminal(&mut conversation, origin, unknown_usage(), 0, None) {
                    records.push(record).expect("one pending terminal before end");
                }
                if let session::End::Failed {
                    failure: llm::Failure::Limit { which: llm::Limit::History, .. },
                    attempts,
                    ..
                } = end
                {
                    conversation.attempts = u64::from(attempts);
                }
                let (end, which, mut failure) = conversation_end(end);
                if failure.is_some() {
                    if let Some(observed) = conversation.last_failure.take() {
                        failure = Some(end_failure(failure.expect("typed failed terminal"), observed));
                    }
                    if conversation.main {
                        self.main_failure = match &failure {
                            Some(failure) => Some(RunFailure {
                                run: conversation.run,
                                model: conversation.model.clone(),
                                attempts: conversation.attempts,
                                failure: failure.clone(),
                            }),
                            None => None,
                        };
                    }
                }
                records
                    .push(Record {
                        t_ms: millis(fact.at, origin),
                        event: Event::ConversationClosed(ConversationClosed {
                            run: conversation.run,
                            conversation: conversation.number,
                            end,
                            which: match which {
                                Some(which) => Some(which.into()),
                                None => None,
                            },
                            failure,
                            turns: u64::from(turns),
                            usage: conversation.reported.usage(),
                            spent: conversation.spent,
                            duration_ms: millis(fact.at, conversation.opened),
                        }),
                    })
                    .expect("one conversation terminal");
            }
            session::FactKind::JobEnded { .. }
            | session::FactKind::ReasoningDropped { .. }
            | session::FactKind::Tools { .. }
            | session::FactKind::BuiltinStarted { .. }
            | session::FactKind::BuiltinAnswered { .. }
            | session::FactKind::DelegateStarted { .. }
            | session::FactKind::DelegateAnswered { .. }
            | session::FactKind::DelegateCancelled { .. }
            | session::FactKind::DelegateWithheld { .. }
            | session::FactKind::Yielded { .. } => {}
        }
        records
    }
}

fn rejected_calls(completion: &llm::Completion) -> (u64, u64) {
    let mut oversized = 0_u64;
    let mut cut = 0_u64;
    for block in &completion.content {
        match block {
            llm::Said::ToolCall { call, .. } => match call {
                llm::Decoded::Invalid { problem } => match problem {
                    llm::Problem::Oversize { .. } => oversized = oversized.checked_add(1).expect("bounded blocks"),
                    llm::Problem::CutOff { .. } => cut = cut.checked_add(1).expect("bounded blocks"),
                    llm::Problem::Nesting
                    | llm::Problem::UnknownTool
                    | llm::Problem::NotAnObject
                    | llm::Problem::Missing { .. }
                    | llm::Problem::WrongType { .. }
                    | llm::Problem::BadValue { .. }
                    | llm::Problem::TooLarge
                    | llm::Problem::JobsFull { .. }
                    | llm::Problem::UnknownJob { .. } => {}
                },
                llm::Decoded::Builtin { .. } | llm::Decoded::Owned { .. } | llm::Decoded::Served { .. } => {}
            },
            llm::Said::Opaque { .. } | llm::Said::Refusal { .. } | llm::Said::Text { .. } => {}
        }
    }
    (oversized, cut)
}

fn take_started(attempt: &mut Attempt) -> Option<Started> {
    match core::mem::replace(attempt, Attempt::Idle) {
        Attempt::Started(started) => Some(started),
        Attempt::Idle => None,
        Attempt::Answered { .. } | Attempt::Failed { .. } => unreachable!("one terminal per provider attempt"),
    }
}

fn terminal(
    conversation: &mut Conversation,
    origin: Time,
    usage: Usage,
    spent: u64,
    retry_ms: Option<u64>,
) -> Option<Record> {
    let (started, at, outcome, stop, blocks, calls, invalid, failure) =
        match core::mem::replace(&mut conversation.attempt, Attempt::Idle) {
            Attempt::Idle => return None,
            Attempt::Started(_) => unreachable!("a session end follows its provider terminal"),
            Attempt::Answered { started, at, stop, blocks, calls, invalid } => {
                (started, at, Outcome::Completed, Some(stop), blocks, calls, invalid, None)
            }
            Attempt::Failed { started, at, failure } => (started, at, Outcome::Failed, None, 0, 0, 0, Some(failure)),
        };
    let (usage, oversized, cut, blocks) = match started.received {
        Some(received) => (crate::reported_usage(received.usage), received.oversized, received.cut, received.blocks),
        None => (usage, 0, 0, blocks),
    };
    let measures = transport(started.measures);
    Some(Record {
        t_ms: millis(at, origin),
        event: Event::ResponseCompleted(ResponseCompleted {
            run: conversation.run,
            conversation: started.conversation,
            response: started.number,
            outcome,
            stop,
            blocks,
            calls,
            invalid,
            oversized,
            cut,
            usage,
            spent,
            request_bytes: measures.request_bytes,
            first_byte_ms: duration_ms(measures.first_byte),
            largest_gap_ms: duration_ms(measures.largest_gap),
            duration_ms: millis(at, started.at),
            failure,
            retry_ms,
            completion: started.completion,
        }),
    })
}

fn cancelled(conversation: &Conversation, origin: Time, started: Started, at: Time) -> Record {
    let measures = transport(started.measures);
    Record {
        t_ms: millis(at, origin),
        event: Event::ResponseCompleted(ResponseCompleted {
            run: conversation.run,
            conversation: started.conversation,
            response: started.number,
            outcome: Outcome::Cancelled,
            stop: None,
            blocks: 0,
            calls: 0,
            invalid: 0,
            oversized: 0,
            cut: 0,
            usage: unknown_usage(),
            spent: 0,
            request_bytes: measures.request_bytes,
            first_byte_ms: duration_ms(measures.first_byte),
            largest_gap_ms: duration_ms(measures.largest_gap),
            duration_ms: millis(at, started.at),
            failure: None,
            retry_ms: None,
            completion: None,
        }),
    }
}

fn transport(measures: Option<smith_protocol_llm::Measures>) -> smith_protocol_llm::Measures {
    match measures {
        Some(measures) => measures,
        None => smith_protocol_llm::Measures::none(),
    }
}

fn duration_ms(duration: Option<skein_lib::Duration>) -> Option<u64> {
    match duration {
        Some(duration) => Some(duration.as_millis()),
        None => None,
    }
}

pub(crate) const fn unknown_usage() -> Usage {
    Usage {
        input_tokens: None,
        cache_read_tokens: None,
        cache_write_tokens: None,
        output_tokens: None,
        reasoning_tokens: None,
    }
}

pub(crate) fn millis(at: Time, origin: Time) -> u64 {
    at.saturating_since(origin).as_millis()
}

fn stop_kind(stop: llm::Stop) -> Stop {
    match stop {
        llm::Stop::EndTurn => Stop::End,
        llm::Stop::ToolUse => Stop::Tools,
        llm::Stop::MaxTokens => Stop::MaxTokens,
        llm::Stop::Refusal => Stop::Refusal,
    }
}

pub(crate) fn completion_failure(failure: llm::Failure, evidence: llm::Evidence) -> CompletionFailure {
    let (class, retry_after_ms) = match failure {
        llm::Failure::Limit { .. } | llm::Failure::ApplicationLimit => (smith_events::FailureClass::Limit, None),
        llm::Failure::Protocol => (smith_events::FailureClass::Protocol, None),
        llm::Failure::Cancelled => (smith_events::FailureClass::Cancelled, None),
        llm::Failure::Overloaded => (smith_events::FailureClass::Overloaded, None),
        llm::Failure::Unavailable => (smith_events::FailureClass::Unavailable, None),
        llm::Failure::TimedOut { .. } => (smith_events::FailureClass::TimedOut, None),
        llm::Failure::ContextTooLong => (smith_events::FailureClass::ContextTooLong, None),
        llm::Failure::Refused => (smith_events::FailureClass::Refused, None),
        llm::Failure::Unsupported => (smith_events::FailureClass::Unsupported, None),
        llm::Failure::Invalid | llm::Failure::Program { .. } | llm::Failure::SessionGone => {
            (smith_events::FailureClass::Invalid, None)
        }
        llm::Failure::SignedOut | llm::Failure::Unauthorized => (smith_events::FailureClass::Unauthorized, None),
        llm::Failure::RateLimited { retry_after } => {
            (smith_events::FailureClass::RateLimited, Some(retry_after.as_millis()))
        }
        llm::Failure::Exhausted { retry_after } => (
            smith_events::FailureClass::Exhausted,
            match retry_after {
                Some(reset) => Some(reset.as_millis()),
                None => None,
            },
        ),
    };
    let status = match evidence {
        llm::Evidence::Unsent | llm::Evidence::Unknown => None,
        llm::Evidence::Response { status } => match status {
            Some(status) => Some(u64::from(status)),
            None => None,
        },
    };
    let evidence = match evidence {
        llm::Evidence::Unsent => smith_events::Evidence::Unsent,
        llm::Evidence::Unknown => smith_events::Evidence::MaybeSent,
        llm::Evidence::Response { .. } => smith_events::Evidence::Response,
    };
    let phase = match failure {
        llm::Failure::TimedOut { phase } => Some(match phase {
            llm::Phase::Connect => smith_events::FailurePhase::Connect,
            llm::Phase::Handshake => smith_events::FailurePhase::Handshake,
            llm::Phase::Head => smith_events::FailurePhase::Head,
            llm::Phase::Idle => smith_events::FailurePhase::Idle,
            llm::Phase::Whole => smith_events::FailurePhase::Whole,
        }),
        llm::Failure::Limit { .. }
        | llm::Failure::ApplicationLimit
        | llm::Failure::Protocol
        | llm::Failure::Cancelled
        | llm::Failure::Overloaded
        | llm::Failure::Unavailable
        | llm::Failure::ContextTooLong
        | llm::Failure::Refused
        | llm::Failure::Program { .. }
        | llm::Failure::SessionGone
        | llm::Failure::SignedOut
        | llm::Failure::Unsupported
        | llm::Failure::Invalid
        | llm::Failure::Unauthorized
        | llm::Failure::RateLimited { .. }
        | llm::Failure::Exhausted { .. } => None,
    };
    let limit = match failure {
        llm::Failure::Limit { which, bound } => {
            Some(smith_events::FailureLimit { which: limit_name(which).into(), bound })
        }
        llm::Failure::ApplicationLimit
        | llm::Failure::Protocol
        | llm::Failure::Cancelled
        | llm::Failure::Overloaded
        | llm::Failure::Unavailable
        | llm::Failure::TimedOut { .. }
        | llm::Failure::ContextTooLong
        | llm::Failure::Refused
        | llm::Failure::Program { .. }
        | llm::Failure::SessionGone
        | llm::Failure::SignedOut
        | llm::Failure::Unsupported
        | llm::Failure::Invalid
        | llm::Failure::Unauthorized
        | llm::Failure::RateLimited { .. }
        | llm::Failure::Exhausted { .. } => None,
    };
    CompletionFailure {
        field: None,
        code: None,
        request_id: None,
        class,
        evidence,
        retry_after_ms,
        phase,
        limit,
        status,
        detail: None,
    }
}

fn limit_name(which: llm::Limit) -> &'static [u8] {
    match which {
        llm::Limit::Pool { .. } => b"pool",
        llm::Limit::History => b"history",
        llm::Limit::Window => b"window",
        llm::Limit::Call => b"call",
        llm::Limit::Message => b"message",
        llm::Limit::Path(which) => match which {
            llm::Cap::Request => b"request",
            llm::Cap::RequestHead => b"request_head",
            llm::Cap::ResponseHead => b"response_head",
            llm::Cap::ResponseFields => b"response_fields",
            llm::Cap::Skip => b"skip",
            llm::Cap::Field => b"field",
            llm::Cap::Retained => b"retained",
            llm::Cap::Tokens => b"tokens",
            llm::Cap::Depth => b"depth",
            llm::Cap::Strings => b"strings",
            llm::Cap::Number => b"number",
            llm::Cap::Reasoning => b"reasoning",
            llm::Cap::OutputItems => b"output_items",
            llm::Cap::Metadata => b"metadata",
            llm::Cap::Tools => b"tools",
            llm::Cap::HistoryItems => b"history_items",
            llm::Cap::Output => b"output",
            llm::Cap::Answer => b"answer",
            llm::Cap::ErrorBody => b"error_body",
            llm::Cap::AccessToken => b"access_token",
            llm::Cap::AccountId => b"account_id",
        },
    }
}

/// A derived window limit keeps the native failure that caused it (smith/docs/design/protocol/events.md, 3.10).
fn end_failure(mut projected: CompletionFailure, observed: CompletionFailure) -> CompletionFailure {
    match &projected.limit {
        Some(limit) if limit.which.as_ref() == b"window" => {
            let request_limit = match &observed.limit {
                Some(limit) => limit.which.as_ref() == b"request",
                None => false,
            };
            if projected.evidence == observed.evidence
                && (observed.class == smith_events::FailureClass::ContextTooLong
                    || observed.class == smith_events::FailureClass::Limit && request_limit)
            {
                projected.evidence = observed.evidence;
                projected.field = observed.field;
                projected.code = observed.code;
                projected.request_id = observed.request_id;
                projected.status = observed.status;
                projected.detail = observed.detail;
            }
            projected
        }
        Some(limit) if limit.which.as_ref() == b"history" => projected,
        Some(_) | None => observed,
    }
}

fn conversation_end(end: session::End) -> (ConversationEnd, Option<&'static [u8]>, Option<CompletionFailure>) {
    match end {
        session::End::Closed => (ConversationEnd::Closed, None, None),
        session::End::NoSummary { .. } => (ConversationEnd::Failed, None, None),
        session::End::Busy | session::End::Invalid => (ConversationEnd::Refused, None, None),
        session::End::Failed { failure, evidence, attempts: _, detail: _ } => {
            (ConversationEnd::Failed, None, Some(completion_failure(failure, evidence)))
        }
        session::End::Budget { spent } => {
            let which: &[u8] = match spent {
                session::Dimension::Turns => b"turns",
                session::Dimension::Input => b"input_tokens",
                session::Dimension::Output => b"output_tokens",
                session::Dimension::CacheRead => b"cache_read_tokens",
                session::Dimension::CacheWrite => b"cache_write_tokens",
                session::Dimension::Time => b"time",
                session::Dimension::Unit => b"spend",
            };
            (ConversationEnd::Budget, Some(which), None)
        }
        session::End::TranscriptRefused { .. } => (ConversationEnd::Transcript, None, None),
        session::End::PriceOverflow | session::End::UsageOverflow => (ConversationEnd::Overflow, None, None),
    }
}

fn provider_field(value: &[u8]) -> Option<Box<[u8]>> {
    if value.is_empty() { None } else { Some(value.into()) }
}

#[cfg(test)]
pub(crate) mod tests;

#[cfg(test)]
mod window_failure_tests;

#[cfg(test)]
mod model_change_tests;
