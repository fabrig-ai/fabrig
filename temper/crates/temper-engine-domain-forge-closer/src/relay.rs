//! One question at a time, without a closer allowance (temper/docs/design/domain/forge/branches.md, 2).
use crate::{Closer, Running, Step, first_line};
use alloc::boxed::Box;
use skein_lib::{Decimal, List, Wall, bytes};

pub const FALLBACK: &[u8] =
    b"This question reached no one: the goal's closer decides nothing and relays only the questions its open steps ask.";

const NO_WORDS: &[u8] = b"This closer takes no words: it relays its open steps' questions and their answers alone. To answer, message form=answer with the question's number.";

/// Feedback for both relays (temper/docs/design/domain/forge/change/notices.md, 3).
#[must_use]
pub fn answer_refusal(target: u64, named: u64, open: Option<(u64, &[u8])>) -> Box<[u8]> {
    let mut text = List::uncapped();
    let task = Decimal::of(target);
    if let Some((number, line)) = open {
        let question = Decimal::of(number);
        append(&mut text, b"Question ");
        append(&mut text, Decimal::of(named).as_bytes());
        append(&mut text, b" is not open at task ");
        append(&mut text, task.as_bytes());
        append(&mut text, b"; its question to you is ");
        append(&mut text, question.as_bytes());
        append(&mut text, b": ");
        append(&mut text, line);
        append(&mut text, b". Answer it: message target=");
        append(&mut text, task.as_bytes());
        append(&mut text, b", form=answer, question=");
        append(&mut text, question.as_bytes());
        append(&mut text, b".");
    } else {
        append(&mut text, b"Task ");
        append(&mut text, task.as_bytes());
        append(&mut text, b" has no question open to you; read_task on task ");
        append(&mut text, task.as_bytes());
        append(&mut text, b" shows its questions.");
    }
    text.into_boxed()
}

/// One waiting slot per possible open step, apart from the standing question.
pub const OPEN_STEPS: usize = 2;

/// The changes to check for liveness, even when a settlement interrupts a landing.
#[must_use]
pub fn steps(closer: &Closer) -> [Option<Running>; OPEN_STEPS] {
    let landing = match (closer.running, closer.landing) {
        (Some(running), Some(landing)) if running.change == landing => None,
        (Some(_) | None, Some(landing)) => Some(Running { step: Step::Land, change: landing }),
        (Some(_) | None, None) => None,
    };
    [closer.running, landing]
}

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub enum Stands {
    Taken,
    Asked { question: u64, answer: Option<Box<[u8]>>, held: Option<Held> },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct Held {
    pub since: Wall,
    /// Older answers never sent a held notice and do not send a delivered notice.
    pub notified: bool,
}
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Question {
    pub asker: u64,
    pub number: u64,
    pub step: Step,
    pub words: Box<[u8]>,
    pub stands: Stands,
}
impl Question {
    fn held_bytes(&self) -> Option<u64> {
        let words = skein_lib::len_u64(self.words.len());
        match &self.stands {
            Stands::Taken | Stands::Asked { answer: None, .. } => Some(words),
            Stands::Asked { answer: Some(answer), .. } => words.checked_add(skein_lib::len_u64(answer.len())),
        }
    }
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
    pub waiting: Box<[Question]>,
}
impl Relay {
    #[must_use]
    #[expect(clippy::new_without_default, reason = "step state has a named constructor")]
    pub fn new() -> Self {
        Self { open: None, replies: Box::new([]), waiting: Box::new([]) }
    }
    /// Only the goal can answer a standing question, and only once.
    #[must_use]
    pub fn open_to(&self, goal: u64, from: u64) -> Option<(u64, &[u8])> {
        if from == goal
            && let Some(open) = &self.open
            && let Stands::Asked { question, answer: None, .. } = open.stands
        {
            Some((question, first_line(&open.words)))
        } else {
            None
        }
    }
    fn promote(&mut self) {
        let waiting = core::mem::replace(&mut self.waiting, Box::new([]));
        let mut next = None;
        let mut rest = List::uncapped();
        for question in waiting {
            if next.is_none() {
                next = Some(question);
            } else {
                rest.push(question).expect("waiting question suffix");
            }
        }
        self.open = next;
        self.waiting = rest.into_boxed();
    }
    fn keep(&mut self, question: Question) {
        if self.open.is_none() && self.waiting.is_empty() {
            self.open = Some(question);
            return;
        }
        let waiting = core::mem::replace(&mut self.waiting, Box::new([]));
        // Like replies, retained questions come from the admitted task inbox.
        let mut next = List::uncapped();
        for waiting in waiting {
            next.push(waiting).expect("admitted waiting questions");
        }
        next.push(question).expect("admitted input");
        self.waiting = next.into_boxed();
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
            total = total.checked_add(open.held_bytes()?)?;
        }
        let waiting = skein_lib::len_u64(self.waiting.len()).checked_mul(u64::try_from(size_of::<Question>()).ok()?)?;
        total = total.checked_add(waiting)?;
        for question in &self.waiting {
            total = total.checked_add(question.held_bytes()?)?;
        }
        Some(total)
    }
}

/// Retain every offered input before sending anything; returns whether this takes the goal's answer.
#[expect(clippy::too_many_arguments, reason = "the relay consumes one offered message at its task and goal")]
pub fn take(
    relay: &mut Relay,
    steps: &[Running],
    task: u64,
    goal: u64,
    from: u64,
    number: u64,
    form: &Form,
    words: &[u8],
) -> bool {
    let mut step = None;
    for running in steps {
        if running.change == from {
            step = Some(running.step);
            break;
        }
    }
    match form {
        Form::Question => {
            if let Some(step) = step
                && relay.waiting.len() < OPEN_STEPS
            {
                relay.keep(Question { asker: from, number, step, words: words.into(), stands: Stands::Taken });
            } else {
                // A procedure consumes its offered inbox prefix; it cannot leave this question unread.
                push(relay, Message { target: from, form: Form::Answer { question: number }, words: FALLBACK.into() });
            }
        }
        Form::Answer { question } => {
            let open = relay.open_to(goal, from);
            if let Some((asked, _)) = open
                && asked == *question
            {
                relay.open.as_mut().expect("a question stands").stands =
                    Stands::Asked { question: asked, answer: Some(words.into()), held: None };
                return true;
            }
            let words = answer_refusal(task, *question, open);
            push(relay, Message { target: from, form: Form::Words, words });
        }
        Form::Words => {
            if step.is_some() {
                let mut text = prefix(from);
                append(&mut text, b": ");
                append(&mut text, words);
                push(relay, Message { target: goal, form: Form::Words, words: text.into_boxed() });
            } else {
                push(relay, Message { target: from, form: Form::Words, words: NO_WORDS.into() });
            }
        }
    }
    false
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
pub struct Notice {
    pub question: u64,
    pub words: Box<[u8]>,
}

#[derive(Debug)]
pub struct Relayed {
    pub relay: Relay,
    pub messages: Box<[Message]>,
    /// Words to the goal in the question's reserved room, not the step's batch.
    pub notice: Option<Notice>,
    pub asked: bool,
    pub more: bool,
}

/// Ordinary message room: what is left in this step, and what a fresh step can use.
#[derive(Clone, Copy, Debug)]
pub struct Room {
    pub left: u32,
    pub batch: u32,
}

/// `ended` observes the open question's asker, even when the closer has moved to another step.
#[must_use]
pub fn relay(state: &Relay, goal: u64, ended: bool, room: Room, bound: u32, at: Wall, took_answer: bool) -> Relayed {
    let mut state = state.clone();
    let mut messages = List::with_capacity(room.left);
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
    let mut notice = None;
    if let Some(open) = &mut state.open {
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
            Stands::Asked { question, answer, .. } => match answer {
                Some(words) => Some(if ended {
                    left(open, goal, Some(*question))
                } else {
                    Message { target: open.asker, form: Form::Answer { question: open.number }, words: words.clone() }
                }),
                None => None,
            },
        };
        if let Some(mut message) = message {
            let delivery = delivery(open, ended);
            let needed = if delivery.is_some() { 2 } else { 1 };
            if messages.room() < needed {
                notice = hold(open, at, took_answer);
                let next_room = if !ended && notice.is_some() { 2 } else { needed };
                // With a one-message deployment batch a delivery pair cannot
                // make progress until a larger batch is configured or the asker ends.
                more |= room.batch >= next_room;
            } else {
                asked = message.form == Form::Question;
                message.words = fit(&message.words, bound);
                messages.push(message).expect("question room checked");
                if let Some((question, since)) = delivery {
                    let words = delivered_words(question, since, open.asker);
                    messages
                        .push(Message { target: goal, form: Form::Words, words: fit(&words, bound) })
                        .expect("answer and delivery notice room checked");
                }
                if !asked {
                    state.open = None;
                }
            }
        }
    }
    // A promoted question gets its own step and its own admission credit.
    if state.open.is_none() && !state.waiting.is_empty() {
        state.promote();
        more = true;
    }
    Relayed { relay: state, messages: messages.into_boxed(), notice, asked, more }
}

fn delivery(open: &Question, ended: bool) -> Option<(u64, Wall)> {
    if !ended && let Stands::Asked { question, held: Some(Held { since, notified: true }), .. } = open.stands {
        Some((question, since))
    } else {
        None
    }
}

fn hold(open: &mut Question, at: Wall, took_answer: bool) -> Option<Notice> {
    if let Stands::Asked { question, answer: Some(_), held } = &mut open.stands
        && held.is_none()
    {
        *held = Some(Held { since: at, notified: took_answer });
        if took_answer {
            let mut text = answer_prefix(*question);
            append(&mut text, b" is held until this closer has message room; held since ");
            append(&mut text, &at.utc_second());
            append(&mut text, b".");
            return Some(Notice { question: *question, words: text.into_boxed() });
        }
    }
    None
}

fn delivered_words(question: u64, since: Wall, change: u64) -> Box<[u8]> {
    let mut text = answer_prefix(question);
    append(&mut text, b", held since ");
    append(&mut text, &since.utc_second());
    append(&mut text, b", is delivered to task ");
    append(&mut text, Decimal::of(change).as_bytes());
    append(&mut text, b".");
    text.into_boxed()
}

fn answer_prefix(question: u64) -> List<u8> {
    let mut text = List::uncapped();
    append(&mut text, b"Your answer to question ");
    append(&mut text, Decimal::of(question).as_bytes());
    text
}

/// Jig assigns the closer's question number at admission, never before it.
pub fn asked(relay: &mut Relay, number: u64) {
    let open = relay.open.as_mut().expect("an admitted ask keeps its question");
    open.stands = Stands::Asked { question: number, answer: None, held: None };
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
