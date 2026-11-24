//! Pure relay messages and judgements
//! (temper/docs/design/domain/forge/change/questions.md, 1).
use crate as forge;
use crate::routing::{append, takes_no_messages};
use alloc::boxed::Box;
use jig_domain_core_vocabulary::StepMessage;
use jig_domain_tasks as tasks;
use skein_lib::List;
use temper_engine_domain_forge_change as change;

pub fn word_message(act: &change::WordAct, requester: tasks::Party, route: change::Route, bound: u32) -> StepMessage {
    let mut words = List::uncapped();
    let target = match act {
        change::WordAct::Pass { producer, from, words: sent } => {
            append(&mut words, b"From task ");
            append(&mut words, skein_lib::Decimal::of(*from).as_bytes());
            append(&mut words, b", your change's requester:\n\n");
            append(&mut words, sent);
            *producer
        }
        change::WordAct::Tell { requester, producer, line } => {
            append(&mut words, b"Passed to task ");
            append(&mut words, skein_lib::Decimal::of(*producer).as_bytes());
            append(&mut words, b", this change's producer.");
            if let Some(line) = line {
                append(&mut words, b" Its question still waits, and words do not answer it: ");
                append(&mut words, line);
                append(
                    &mut words,
                    b". Answer it with message form=answer; read_task on this change gives the question's number.",
                );
            }
            *requester
        }
        change::WordAct::Fallback { requester: target } => {
            return StepMessage {
                target: *target,
                kind: tasks::MessageKind::Words,
                words: takes_no_messages(requester, route, bound),
            };
        }
        change::WordAct::NotPassed { requester, words: reply } => {
            append(&mut words, reply);
            *requester
        }
    };
    StepMessage {
        target,
        kind: tasks::MessageKind::Words,
        words: forge::text::fit(words.as_slice(), usize::try_from(bound).expect("message bound"), "…".as_bytes()),
    }
}

pub fn question_words(words: &[u8], reasons: &[u8], bound: u32) -> Box<[u8]> {
    let mut text = List::uncapped();
    append(&mut text, words);
    if !reasons.is_empty() {
        append(&mut text, b"\n\n");
        append(&mut text, reasons);
    }
    forge::text::fit(text.as_slice(), usize::try_from(bound).expect("message bound"), "…".as_bytes())
}

pub fn left_words(asker: u64, number: u64, line: &[u8], answered: bool, bound: u32) -> Box<[u8]> {
    let mut words = List::uncapped();
    append(&mut words, b"Task ");
    append(&mut words, skein_lib::Decimal::of(asker).as_bytes());
    if answered {
        append(&mut words, b" ended; your answer to its question ");
    } else {
        append(&mut words, b" ended with its question ");
    }
    append(&mut words, skein_lib::Decimal::of(number).as_bytes());
    if answered {
        append(&mut words, b" goes no further: ");
    } else {
        append(&mut words, b" open; it reached no one: ");
    }
    append(&mut words, line);
    forge::text::fit(words.as_slice(), usize::try_from(bound).expect("message bound"), "…".as_bytes())
}

pub fn hold_words(questions: &change::Questions, bound: u32) -> Box<[u8]> {
    let mut words = List::with_capacity(tasks::HOLD_ACCOUNT_BYTES);
    let bound = skein_lib::Decimal::of(u64::from(bound));
    append(&mut words, b"its questions are spent: ");
    append(&mut words, bound.as_bytes());
    append(&mut words, b" of ");
    append(&mut words, bound.as_bytes());
    append(&mut words, b" taken");
    if let Some(question) = questions.open.first() {
        append(&mut words, b"; question ");
        append(&mut words, skein_lib::Decimal::of(question.number).as_bytes());
        append(&mut words, b" of task ");
        append(&mut words, skein_lib::Decimal::of(question.asker).as_bytes());
        append(&mut words, b" waits; release it to pass the question up");
    }
    words.into_boxed()
}

pub fn has_ask(acts: &[change::RelayAct]) -> bool {
    for act in acts {
        match act {
            change::RelayAct::Ask { .. } => return true,
            change::RelayAct::Answer { .. } | change::RelayAct::Left { .. } | change::RelayAct::Hold => {}
        }
    }
    false
}

#[expect(clippy::manual_find, reason = "the programming model uses bounded loops instead of iterator closures")]
pub fn find_question(open: &[change::Open], asker: u64, number: u64) -> Option<&change::Open> {
    for question in open {
        if question.asker == asker && question.number == number {
            return Some(question);
        }
    }
    None
}

pub fn is_answer(kind: &tasks::MessageKind) -> bool {
    match kind {
        tasks::MessageKind::Answer { .. } => true,
        tasks::MessageKind::Question
        | tasks::MessageKind::PassedQuestion { .. }
        | tasks::MessageKind::PassedAnswer { .. }
        | tasks::MessageKind::Words
        | tasks::MessageKind::Result(_)
        | tasks::MessageKind::Escalation { .. }
        | tasks::MessageKind::Proposal { .. }
        | tasks::MessageKind::ProposalDecision { .. }
        | tasks::MessageKind::SettingsDecision { .. }
        | tasks::MessageKind::FinishRequested { .. }
        | tasks::MessageKind::Amendment { .. }
        | tasks::MessageKind::Met { .. }
        | tasks::MessageKind::Notice { .. }
        | tasks::MessageKind::QuestionAnswered { .. }
        | tasks::MessageKind::QuestionOwed
        | tasks::MessageKind::NothingAwaited
        | tasks::MessageKind::QuestionEnded { .. }
        | tasks::MessageKind::Timer { .. }
        | tasks::MessageKind::PersonStalled { .. }
        | tasks::MessageKind::EffectNotMade { .. }
        | tasks::MessageKind::News { .. } => false,
    }
}

pub fn asking_delegate(delegate: Option<(u64, change::Delegate)>, target: u64) -> bool {
    match delegate {
        Some((child, change::Delegate::Produce | change::Delegate::Repair(_) | change::Delegate::Resolve { .. })) => {
            child == target
        }
        Some((_, change::Delegate::Gate { .. } | change::Delegate::Merge { .. } | change::Delegate::Close)) | None => {
            false
        }
    }
}

pub fn questions_held(state: &change::State) -> bool {
    match state {
        change::State::Held { why, .. } => *why == change::Hold::Questions,
        change::State::Taking { .. }
        | change::State::Producing { .. }
        | change::State::Opening { .. }
        | change::State::Recreating { .. }
        | change::State::Reopening { .. }
        | change::State::Checking { .. }
        | change::State::Judging { .. }
        | change::State::Gating { .. }
        | change::State::CancellingGates { .. }
        | change::State::Ready { .. }
        | change::State::Preparing { .. }
        | change::State::Queued { .. }
        | change::State::First { .. }
        | change::State::Updating { .. }
        | change::State::WaitingProducer { .. }
        | change::State::Resolving { .. }
        | change::State::Repairing { .. }
        | change::State::Landing { .. }
        | change::State::Landed { .. }
        | change::State::Red { .. }
        | change::State::Cleared { .. }
        | change::State::Uncleared { .. }
        | change::State::NoChange { .. }
        | change::State::Ended => false,
    }
}
