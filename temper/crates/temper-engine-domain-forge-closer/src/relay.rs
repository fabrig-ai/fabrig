//! One question at a time, without a closer allowance (temper/docs/design/domain/forge/branches.md, 2).
use crate::{Running, Step, first_line};
use alloc::boxed::Box;
use skein_lib::{Decimal, List, bytes};

pub const FALLBACK: &[u8] = b"The goal's closer decides nothing; it relays its own change's questions to its goal.";

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Stands {
    Taken,
    Asked { question: u64, answer: Option<Box<[u8]>> },
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Question {
    pub asker: u64,
    pub number: u64,
    pub step: Step,
    pub words: Box<[u8]>,
    pub stands: Stands,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Form {
    Words,
    Question,
    Answer { question: u64 },
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Message {
    pub target: u64,
    pub form: Form,
    pub words: Box<[u8]>,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Relay {
    pub open: Option<Question>,
    pub replies: Box<[Message]>,
}
impl Relay {
    #[must_use]
    #[expect(clippy::new_without_default, reason = "step state has a named constructor")]
    pub fn new() -> Self {
        Self { open: None, replies: Box::new([]) }
    }
    /// Owned payload, for store restore and reconciliation accounting.
    #[must_use]
    pub fn held_bytes(&self) -> Option<u64> {
        let mut total =
            skein_lib::len_u64(self.replies.len()).checked_mul(u64::try_from(size_of::<Message>()).ok()?)?;
        for reply in &self.replies {
            total = total.checked_add(skein_lib::len_u64(reply.words.len()))?;
        }
        if let Some(open) = &self.open {
            total = total.checked_add(skein_lib::len_u64(open.words.len()))?;
            match &open.stands {
                Stands::Taken | Stands::Asked { answer: None, .. } => {}
                Stands::Asked { answer: Some(words), .. } => {
                    total = total.checked_add(skein_lib::len_u64(words.len()))?;
                }
            }
        }
        Some(total)
    }
}

/// Retain every offered input before sending anything; only the live step may ask.
pub fn take(relay: &mut Relay, live: Option<Running>, goal: u64, from: u64, number: u64, form: &Form, words: &[u8]) {
    match form {
        Form::Question => {
            if let Some(live) = live
                && live.change == from
                && relay.open.is_none()
            {
                relay.open =
                    Some(Question { asker: from, number, step: live.step, words: words.into(), stands: Stands::Taken });
            } else {
                push(relay, Message { target: from, form: Form::Answer { question: number }, words: FALLBACK.into() });
            }
        }
        Form::Answer { question } => {
            if from == goal
                && let Some(open) = &mut relay.open
                && let Stands::Asked { question: asked, answer } = &mut open.stands
                && asked == question
                && answer.is_none()
            {
                *answer = Some(words.into());
            }
        }
        Form::Words => {
            if let Some(live) = live
                && live.change == from
            {
                let mut text = prefix(from);
                append(&mut text, b": ");
                append(&mut text, words);
                push(relay, Message { target: goal, form: Form::Words, words: text.into_boxed() });
            }
        }
    }
}
fn push(relay: &mut Relay, message: Message) {
    let replies = core::mem::replace(&mut relay.replies, Box::new([]));
    // Retained messages are bounded at the task inbox, not by a relay allowance.
    let mut next = List::uncapped();
    for reply in replies {
        next.push(reply).expect("admitted replies");
    }
    next.push(message).expect("admitted input");
    relay.replies = next.into_boxed();
}

#[derive(Debug)]
pub struct Relayed {
    pub relay: Relay,
    pub messages: Box<[Message]>,
    pub asked: bool,
    pub more: bool,
}

/// `ended` observes the open question's asker, even when the closer has moved to another step.
#[must_use]
#[expect(clippy::manual_map, reason = "step decisions use exhaustive matches, not closures")]
pub fn relay(state: &Relay, goal: u64, ended: bool, room: u32, bound: u32) -> Relayed {
    let mut state = state.clone();
    let mut messages = List::with_capacity(room);
    let mut sent = 0_usize;
    for reply in &state.replies {
        if messages.room() == 0 {
            break;
        }
        let mut reply = reply.clone();
        reply.words = fit(&reply.words, bound);
        messages.push(reply).expect("reply room checked");
        sent = sent.saturating_add(1);
    }
    state.replies = state.replies.get(sent..).expect("sent reply prefix").into();
    let mut asked = false;
    let mut more = !state.replies.is_empty();
    if let Some(open) = &state.open {
        let message = match &open.stands {
            Stands::Taken => Some(if ended {
                left(open, goal, None)
            } else {
                let mut text = prefix(open.asker);
                append(&mut text, b", the goal's ");
                append(
                    &mut text,
                    match open.step {
                        Step::Close => b"status settlement",
                        Step::Land => b"landing",
                    },
                );
                append(&mut text, b", asks:\n\n");
                append(&mut text, &open.words);
                Message { target: goal, form: Form::Question, words: text.into_boxed() }
            }),
            Stands::Asked { question, answer } => match answer {
                Some(words) => Some(if ended {
                    left(open, goal, Some(*question))
                } else {
                    Message { target: open.asker, form: Form::Answer { question: open.number }, words: words.clone() }
                }),
                None => None,
            },
        };
        if let Some(mut message) = message {
            if messages.room() == 0 {
                more = true;
            } else {
                asked = message.form == Form::Question;
                message.words = fit(&message.words, bound);
                messages.push(message).expect("question room checked");
                if !asked {
                    state.open = None;
                }
            }
        }
    }
    Relayed { relay: state, messages: messages.into_boxed(), asked, more }
}

/// Jig assigns the closer's question number at admission, never before it.
pub fn asked(relay: &mut Relay, number: u64) {
    let open = relay.open.as_mut().expect("an admitted ask keeps its question");
    open.stands = Stands::Asked { question: number, answer: None };
}
fn left(open: &Question, goal: u64, question: Option<u64>) -> Message {
    let mut text = prefix(open.asker);
    match question {
        None => append(&mut text, b" ended with its question open; it reached no one: "),
        Some(number) => {
            append(&mut text, b" ended; your answer to question ");
            append(&mut text, Decimal::of(number).as_bytes());
            append(&mut text, b" goes no further: ");
        }
    }
    append(&mut text, first_line(&open.words));
    Message { target: goal, form: Form::Words, words: text.into_boxed() }
}
fn prefix(change: u64) -> List<u8> {
    let mut text = List::uncapped();
    append(&mut text, b"Change ");
    append(&mut text, Decimal::of(change).as_bytes());
    text
}
fn append(text: &mut List<u8>, words: &[u8]) {
    for byte in words {
        text.push(*byte).expect("admitted message text");
    }
}
fn fit(words: &[u8], bound: u32) -> Box<[u8]> {
    let bound = usize::try_from(bound).expect("message bound fits usize");
    if words.len() <= bound {
        return words.into();
    }
    let mark = bytes::utf8_prefix("…".as_bytes(), bound);
    let mut text = List::uncapped();
    append(&mut text, bytes::utf8_prefix(words, bound.saturating_sub(mark.len())));
    append(&mut text, mark);
    text.into_boxed()
}

#[cfg(test)]
mod tests;
