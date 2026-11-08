//! Concrete turns and content-free facts. The cursor tracks the told prefix; emitted turns resolve live calls into historical blocks.
//! Contract: smith/docs/design/domain/session.md, sections 3, 4 and 6.

use super::{
    Block, Box, Conversation, Decoded, End, FactKind, Facts, Id, List, Message, Queue, Request, Run, Session, Slab,
    ToolChoice, Usage, compaction, count, history,
};

/// What a transition tells, derived from the requests it made: a fact for
/// each but the cancels and the tools' operations, which the tools tell.
pub(super) fn tell(facts: &mut Facts, runs: &Slab<Run>, session: &mut Session, out: &Queue<Request>, mark: u32) {
    let opener = session.conversation.opener;
    let made = usize::try_from(mark).expect("a u32 fits in a usize");
    for request in out.iter().skip(made) {
        compaction::follow(&mut session.conversation, facts, request);
        let fact = match request {
            Request::Opened { opener, session: _ } => FactKind::Opened { opener: *opener },
            Request::Yielded { opener, stop, text: _ } => FactKind::Yielded { opener: *opener, stop: *stop },
            Request::Used { opener, usage } => FactKind::Used { opener: *opener, usage: *usage },
            Request::Ended { opener, end, turns, usage, .. } => {
                FactKind::Ended { opener: *opener, end: end_fact(end), turns: *turns, usage: *usage }
            }
            Request::Complete {
                owner: _,
                name,
                prompt,
                deadlines: _,
                max_completion_bytes: _,
                max_completion_blocks: _,
                max_failure_bytes: _,
            } => {
                let (messages, max_tokens) = (count(prompt.messages.len()), prompt.max_tokens);
                FactKind::CompletionStarted {
                    opener,
                    window: session.conversation.window.number,
                    attempt: name.retry,
                    messages,
                    max_tokens,
                    choice: choice_kind(&prompt.choice),
                }
            }
            Request::Delegate { owner, opener: _, call: _, deadline: _, origin: _ } => {
                let run = runs.get(Id::from_token(*owner)).expect("a run lives while its call is asked for");
                FactKind::DelegateStarted { opener, block: run.block }
            }
            Request::JobsCancelled { .. }
            | Request::Turn { .. }
            | Request::Priced { .. }
            | Request::Cancel { .. }
            | Request::Yield { .. }
            | Request::Withdraw { .. }
            | Request::Io { .. }
            | Request::Job { .. }
            | Request::CancelIo { .. } => {
                continue;
            }
        };
        match request {
            Request::Delegate { owner, .. } => {
                let run = runs.get(Id::from_token(*owner)).expect("a delegated call lives through its start");
                facts.push_call(facts.now(), fact, &run.metadata);
            }
            Request::Complete { .. } => facts.push_response(fact, next_response(&session.conversation), None),
            Request::Used { .. } => {
                let spent = session.conversation.spending.last_charge;
                facts.push_response(fact, u64::from(session.conversation.turns), Some(spent));
            }
            Request::Opened { .. } | Request::Yielded { .. } | Request::Ended { .. } => facts.push(fact),
            Request::JobsCancelled { .. }
            | Request::Turn { .. }
            | Request::Priced { .. }
            | Request::Cancel { .. }
            | Request::Yield { .. }
            | Request::Withdraw { .. }
            | Request::Io { .. }
            | Request::Job { .. }
            | Request::CancelIo { .. } => unreachable!("requests without facts were skipped"),
        }
    }
}

/// The logical tool selection without retaining descriptor tickets in a fact.
pub(super) fn choice_kind(choice: &ToolChoice) -> crate::facts::Choice {
    match choice {
        ToolChoice::Auto => crate::facts::Choice::Auto,
        ToolChoice::None => crate::facts::Choice::None,
        ToolChoice::Only(_) => crate::facts::Choice::Only,
    }
}

pub(super) fn tell_turn(conversation: &mut Conversation, out: &mut Queue<Request>) -> Option<End> {
    if !conversation.builtin.is_empty() {
        return None;
    }
    let Cursor { dialect, sequence, told, pending } = &mut conversation.cursor;
    let usage = pending.take()?;
    let spent = conversation.spending.total;
    let Some(next) = sequence.checked_add(1) else {
        return Some(history::sequence_limit());
    };
    let messages = copied_messages(&conversation.transcript, *told);
    *sequence = next;
    *told = conversation.transcript.len();
    let dialect = *dialect;
    let kind = opening_kind(conversation, next);
    let turn = crate::record::Turn {
        version: crate::record::VERSION,
        kind,
        endpoint: Some(conversation.endpoint),
        model: conversation.model.clone(),
        effort: conversation.effort.clone(),
        dialect,
        sequence: next,
        usage,
        spent,
        jobs: conversation.jobs.highest,
        messages,
    };
    out.push(Request::Turn { opener: conversation.opener, turn });
    None
}

pub(super) fn next_sequence(conversation: &Conversation) -> u32 {
    conversation.cursor.sequence.checked_add(1).expect("checked before tool effects")
}

pub(super) fn next_response(conversation: &Conversation) -> u64 {
    u64::from(conversation.turns).checked_add(1).expect("a u32 completion count fits in u64")
}

/// Facts carry terminal classification only; diagnostics go to the opener's terminal.
pub(super) fn end_fact(end: &End) -> End {
    match end {
        End::Failed { failure, evidence, attempts, .. } => {
            End::Failed { failure: *failure, evidence: *evidence, attempts: *attempts, detail: Box::default() }
        }
        End::Busy
        | End::Invalid
        | End::Closed
        | End::Budget { .. }
        | End::NoSummary { .. }
        | End::TranscriptRefused { .. }
        | End::PriceOverflow
        | End::UsageOverflow => end.clone(),
    }
}

/// Position and pending usage of the next concrete transcript turn.
#[derive(Debug)]
pub(super) struct Cursor {
    pub(super) dialect: u32,
    pub(super) sequence: u32,
    pub(super) told: u32,
    pub(super) pending: Option<Usage>,
}

fn copied_messages(transcript: &List<Message>, told: u32) -> Box<[Message]> {
    let mut messages = List::with_capacity(transcript.len().saturating_sub(told));
    for message in transcript.iter().skip(usize::try_from(told).expect("u32 fits")) {
        let mut blocks = List::with_capacity(count(message.content.len()));
        for block in &message.content {
            let block = match block {
                Block::ToolCall(call) => Block::ToolCall(crate::llm::ToolCall {
                    id: call.id.clone(),
                    name: call.name.clone(),
                    input: call.input.clone(),
                    call: Decoded::Historical,
                    replay: call.replay.clone(),
                }),
                Block::Text { .. } | Block::Refusal { .. } | Block::Opaque { .. } | Block::ToolResult { .. } => {
                    block.clone()
                }
            };
            blocks.push(block).expect("one copied block per original");
        }
        messages
            .push(Message { role: message.role, content: blocks.into_boxed() })
            .expect("one copied message per original");
    }
    messages.into_boxed()
}

fn opening_kind(conversation: &mut Conversation, next: u32) -> crate::record::Kind {
    if !conversation.window_turn && next != 1 {
        return crate::record::Kind::Ordinary;
    }
    let opener = Box::new(crate::record::Opener {
        system: Some(conversation.system.clone()),
        affinity: conversation.affinity,
        account: conversation.account,
        backend: conversation.backend.clone(),
    });
    if conversation.window_turn {
        conversation.window_turn = false;
        crate::record::Kind::Window(opener)
    } else {
        crate::record::Kind::First(opener)
    }
}
