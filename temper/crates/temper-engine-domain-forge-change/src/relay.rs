//! A change's relay of its agents' questions (temper/docs/design/domain/forge.md, 8.11).
use alloc::boxed::Box;
use skein_lib::{List, Writer, bytes};

use crate::{Limits, Open, QUESTION_LINE, QUESTIONS_OPEN, Questions, Route, Stands};

/// A question one of the change's own agents asked, taken at this step.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Asked<'a> {
    pub asker: u64,
    pub number: u64,
    pub words: &'a [u8],
}

/// How the open question's foreman ended.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Settled<'a> {
    Answer(&'a [u8]),
    Finish(&'a [u8]),
    Escalate(&'a [u8]),
    Failed(&'a [u8]),
}

/// What a step heard for the relay.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct RelayHeard<'a> {
    pub asked: &'a [Asked<'a>],
    pub foreman: Option<(u64, Settled<'a>)>,
    /// The requester's answer to the change's question, its words.
    pub answer: Option<&'a [u8]>,
    /// Askers among the open questions' that have ended.
    pub ended: &'a [u64],
    /// The change's task is kept by a pending dependency.
    pub kept: bool,
    /// The change was released from its questions hold since its last step.
    pub released: bool,
}

/// What the root applies for a change's relay.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RelayAct {
    /// Delegate a foreman for the first open question.
    Foreman { asker: u64, number: u64 },
    /// Answer the asker's question with these words.
    Answer { asker: u64, number: u64, words: Box<[u8]> },
    /// Ask the requester, with the foreman's reasons if any.
    Ask { asker: u64, number: u64, reasons: Box<[u8]> },
    /// Cancel a foreman whose asker ended.
    Cancel { task: u64 },
    /// Hold the change past its bound.
    Hold,
}

/// The relay's next record and the acts to commit with it.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Relayed {
    pub questions: Questions,
    pub acts: Box<[RelayAct]>,
    /// Work is left for a later step because message room ran out.
    pub more: bool,
}

/// Decide the relay purely; `room` bounds this step's `Answer` and `Ask` acts.
#[must_use]
pub fn relay(questions: &Questions, heard: &RelayHeard<'_>, limits: &Limits, mut room: usize) -> Relayed {
    if questions.route == Route::None {
        return Relayed { questions: questions.clone(), acts: Box::new([]), more: false };
    }
    let mut acts = List::uncapped();
    let mut open = take_questions(questions, heard, &mut acts);
    let mut heard = *heard;
    let mut taken = questions.taken;
    let mut done: usize = 0;
    let mut more = false;
    for question in &mut open {
        if question.stands == Stands::Asked
            && let Some(answer) = heard.answer
        {
            if heard.ended.contains(&question.asker) {
                heard.answer = None;
                done = done.saturating_add(1);
                continue;
            }
            if heard.kept {
                question.stands = Stands::Held { answer: bytes::copy_of(answer) };
                break;
            }
        }
        let Some(act) = next_act(question, &heard, limits, taken, questions.route) else { break };
        match &act {
            RelayAct::Answer { .. } | RelayAct::Ask { .. } => {
                if room == 0 {
                    more = true;
                    break;
                }
                room = room.saturating_sub(1);
            }
            RelayAct::Foreman { .. } | RelayAct::Cancel { .. } | RelayAct::Hold => {}
        }
        if question.stands == Stands::Taken && act != RelayAct::Hold {
            taken = if heard.released { 1 } else { taken.saturating_add(1) };
            heard.released = false;
        }
        match &act {
            RelayAct::Answer { .. } => {
                match question.stands {
                    Stands::Foreman { .. } => heard.foreman = None,
                    Stands::Asked => heard.answer = None,
                    Stands::Held { .. } => {}
                    Stands::Taken => unreachable!("a taken question is delegated or passed up"),
                }
                done = done.saturating_add(1);
                acts.push(act).expect("relay acts fit a list");
            }
            RelayAct::Ask { .. } => {
                question.stands = Stands::Asked;
                acts.push(act).expect("relay acts fit a list");
                break;
            }
            RelayAct::Foreman { .. } => {
                question.stands = Stands::Foreman { task: None };
                acts.push(act).expect("relay acts fit a list");
                break;
            }
            RelayAct::Hold => {
                acts.push(act).expect("relay acts fit a list");
                break;
            }
            RelayAct::Cancel { .. } => unreachable!("ended askers are removed before working the head"),
        }
    }
    let remaining = open.get(done..).expect("only answered heads leave the open questions");
    Relayed {
        questions: Questions { route: questions.route, taken, open: Box::from(remaining) },
        acts: acts.into_boxed(),
        more,
    }
}

fn take_questions(questions: &Questions, heard: &RelayHeard<'_>, acts: &mut List<RelayAct>) -> Box<[Open]> {
    let cap = u32::try_from(QUESTIONS_OPEN).expect("the open question bound fits a u32");
    let mut open = List::with_capacity(cap);
    for question in &questions.open {
        if heard.ended.contains(&question.asker) {
            match question.stands {
                Stands::Foreman { task } => {
                    if let Some(task) = task {
                        acts.push(RelayAct::Cancel { task }).expect("relay acts fit a list");
                    }
                    continue;
                }
                Stands::Taken | Stands::Held { .. } => continue,
                Stands::Asked => {}
            }
        }
        open.push(question.clone()).expect("the record fits the open question bound");
    }
    for asked in heard.asked {
        if open.room() == 0 {
            break;
        }
        let question = Open {
            asker: asked.asker,
            number: asked.number,
            line: first_line(asked.words),
            words: bytes::copy_of(asked.words),
            stands: Stands::Taken,
        };
        open.push(question).expect("open question room was checked");
    }
    open.into_boxed()
}

fn next_act(question: &Open, heard: &RelayHeard<'_>, limits: &Limits, taken: u32, route: Route) -> Option<RelayAct> {
    let Open { asker, number, stands, .. } = question;
    let asker = *asker;
    let number = *number;
    match stands {
        Stands::Taken => {
            let spent = taken >= limits.questions;
            if spent && !heard.released {
                return Some(RelayAct::Hold);
            }
            match route {
                Route::Foreman if !spent => Some(RelayAct::Foreman { asker, number }),
                Route::Foreman | Route::Up => Some(RelayAct::Ask { asker, number, reasons: Box::new([]) }),
                Route::None => unreachable!("a route of none returns before taking questions"),
            }
        }
        Stands::Foreman { task } => {
            let (settled_task, settled) = heard.foreman?;
            if let Some(task) = task
                && *task != settled_task
            {
                return None;
            }
            Some(match settled {
                Settled::Answer(words) => RelayAct::Answer { asker, number, words: bytes::copy_of(words) },
                Settled::Finish(words) => RelayAct::Answer {
                    asker,
                    number,
                    words: joined(b"Finish: the work done holds; list the rest under Left.\n", words),
                },
                Settled::Escalate(words) | Settled::Failed(words) => {
                    RelayAct::Ask { asker, number, reasons: joined(b"Foreman:\n", words) }
                }
            })
        }
        Stands::Asked => Some(RelayAct::Answer { asker, number, words: bytes::copy_of(heard.answer?) }),
        Stands::Held { answer } => {
            if heard.kept {
                None
            } else {
                Some(RelayAct::Answer { asker, number, words: answer.clone() })
            }
        }
    }
}

fn first_line(words: &[u8]) -> Box<[u8]> {
    let end = match bytes::find(words, b"\n") {
        Some(end) => end,
        None => words.len(),
    };
    let line = words.get(..end).expect("the first newline is within the words");
    if line.len() <= QUESTION_LINE {
        return bytes::copy_of(line);
    }
    let mark = "…".as_bytes();
    let prefix = bytes::utf8_prefix(line, QUESTION_LINE.saturating_sub(mark.len()));
    joined(prefix, mark)
}

fn joined(prefix: &[u8], words: &[u8]) -> Box<[u8]> {
    let length = prefix.len().checked_add(words.len()).expect("question text and its prefix fit usize");
    let mut out = Writer::new(length);
    out.put(prefix).expect("the prefix fits its sized writer");
    out.put(words).expect("the words fit their sized writer");
    out.finish()
}
