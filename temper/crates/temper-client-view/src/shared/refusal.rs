//! Send refusals (temper/docs/design/client/tui/pages/chat.md, 2; temper/docs/design/client/pages/README.md).
use crate::{cards::P, words::Words};
use alloc::boxed::Box;
use jig_client::{Answered, Outcome, contract as c};
use skein_lib::List;
use skein_ui::phrases::Hole;
use temper_client_domain::{Address, Domain, pages::chat::Sending};

/// Words beneath the composer's last send (temper/docs/design/client/tui/pages/chat.md, 2).
/// Explanatory paragraphs follow the status with a space between them.
#[must_use]
pub fn refusal(root: &Domain, w: &Words) -> Option<Box<[u8]>> {
    let mut parts = List::with_capacity(3);
    if let Some(question) = super::question(root, w)
        && let Some(outcome) = root.pages().needs().refused(question.object)
        && let Some(feedback) = outcome_feedback(outcome, w, true)
    {
        parts.push(feedback.words()).expect("one question's feedback");
    }
    for feedback in &composer_refusals(root, w) {
        parts.push(feedback.words()).expect("one send's feedback");
    }
    if parts.is_empty() {
        return None;
    }
    let mut joined: Box<[u8]> = Box::default();
    for part in &parts {
        joined = if joined.is_empty() { part.clone() } else { crate::accounts::words::join(&[&joined, b" ", part]) };
    }
    Some(joined)
}

/// The start sheet's error and funding explanation, without its browser frame (client/pages/goal.md, 3).
#[must_use]
pub fn start_goal_refusal(root: &Domain, words: &Words) -> Option<Box<[u8]>> {
    use temper_client_domain::pages::start_goal::{self, Failure, Unfunded};
    let start = root.pages().start()?;
    let error = match &start.error {
        Some(Failure::Refused(why)) => Some(refused(why, words, false).words()),
        Some(Failure::NotOffered(_)) => Some(refused(&c::Refusal::NotOffered, words, false).words()),
        Some(Failure::TooLong) => Some(
            refused(&c::Refusal::Limit { which: c::Limit::Words { limit: words.name_bytes } }, words, false).words(),
        ),
        Some(Failure::Outcome(outcome)) => match outcome_feedback(outcome, words, true) {
            Some(feedback) => Some(feedback.words()),
            None => None,
        },
        None => None,
    };
    let unfunded = match root.pages().me(root.client()) {
        Some(me) => start_goal::unfunded(me, start.project, start.way),
        None => None,
    };
    let funding = match unfunded {
        Some(why) => Some(words.say(
            match why {
                Unfunded::Held => crate::words::Word::StartGoalNoFundsHeld,
                Unfunded::Empty => crate::words::Word::StartGoalNoFunds,
            },
            &[],
        )),
        None => None,
    };
    match (error, funding) {
        (Some(error), Some(funding)) => Some(crate::accounts::words::join(&[&error, b" ", &funding])),
        (Some(error), None) => Some(error),
        (None, funding) => funding,
    }
}

/// A send's status and its explanatory paragraph (temper/docs/design/client/pages/README.md).
#[derive(Debug)]
pub(crate) struct Feedback {
    pub head: Box<[u8]>,
    pub refused: bool,
    pub after: Option<Box<[u8]>>,
}
impl Feedback {
    fn words(&self) -> Box<[u8]> {
        match &self.after {
            Some(after) => crate::accounts::words::join(&[&self.head, b" ", after]),
            None => self.head.clone(),
        }
    }
}
fn refused(value: &c::Refusal, words: &Words, answer: bool) -> Feedback {
    let hole = match value {
        c::Refusal::Forgotten { .. } => unreachable!("a restore: restore-ctl"),
        c::Refusal::Judged { why, .. } => Some(Hole::Name(why.bytes().into())),
        c::Refusal::Account { account, .. } => Some(Hole::Number(u64::from(*account))),
        c::Refusal::Taken { by } => Some(Hole::Number(*by)),
        c::Refusal::Llm { model, .. } => Some(Hole::Name(words.model_name(model.bytes()))),
        c::Refusal::Stale(stale) => Some(Hole::Number(u64::from(stale.newest))),
        c::Refusal::Full(full) => Some(Hole::Number(u64::from(full.revision))),
        c::Refusal::AccountMove { .. }
        | c::Refusal::SignInRedirect { .. }
        | c::Refusal::NoEscalationHolder
        | c::Refusal::NameTaken
        | c::Refusal::Malformed { .. }
        | c::Refusal::Role { .. }
        | c::Refusal::NotWaiting
        | c::Refusal::NoFurther
        | c::Refusal::Authority { .. }
        | c::Refusal::Limit { .. }
        | c::Refusal::Moved { .. }
        | c::Refusal::Ended { .. }
        | c::Refusal::Unknown
        | c::Refusal::ProgramNameReserved
        | c::Refusal::NotOffered
        | c::Refusal::KeyConflict
        | c::Refusal::LiveTasks { .. }
        | c::Refusal::Circle { .. }
        | c::Refusal::Dependency { .. }
        | c::Refusal::Kept { .. }
        | c::Refusal::Provider => None,
    };
    let head = words.say_word(jig_client::words::Word::refusal(value), hole.as_slice());
    let head = live_tasks_words(head, value);
    let after = if let c::Refusal::Role { held: None } = value {
        Some(words.say_card(P::NoHeldRole, &[]))
    } else if answer && let c::Refusal::Moved { .. } = value {
        Some(words.say_card(P::LookAgain, &[]))
    } else {
        None
    };
    Feedback { head, refused: true, after }
}
/// Append the live roots as task names without changing jig's refusal phrase.
pub(crate) fn live_tasks_words(mut head: Box<[u8]>, value: &c::Refusal) -> Box<[u8]> {
    if let c::Refusal::LiveTasks { tasks, more } = value {
        let mut first = true;
        for task in tasks {
            let number = skein_lib::Decimal::of(*task);
            head = crate::accounts::words::join(&[&head, if first { b" T" } else { b", T" }, number.as_bytes()]);
            first = false;
        }
        if *more != 0 {
            let number = skein_lib::Decimal::of(u64::from(*more));
            head = crate::accounts::words::join(&[&head, b", and ", number.as_bytes(), b" more"]);
        }
    }
    head
}

/// A failed send in jig's words (temper/docs/design/client/pages/README.md).
pub(crate) fn outcome_feedback(outcome: &Outcome, words: &Words, answer: bool) -> Option<Feedback> {
    let head = match outcome {
        Outcome::Answered(Answered::Jig(c::Answer::Refused { refusal, .. })) => {
            return Some(refused(refusal, words, answer));
        }
        Outcome::Answered(Answered::Jig(c::Answer::Done(_)) | Answered::Family(_)) => return None,
        Outcome::Full { held, room_at } => {
            words.say_card(P::AnswersFull, &[Hole::Number(u64::from(*held)), Hole::Wall(*room_at)])
        }
        Outcome::NotKept | Outcome::Held => words.say_card(P::RequestNotKept, &[]),
        Outcome::Unknown => words.say_card(P::Unknown, &[]),
        Outcome::Limit => words.say_card(P::RequestsInFlight, &[]),
        Outcome::Unconfirmed => words.say_card(P::RequestUnconfirmed, &[]),
        Outcome::Unsent | Outcome::Lost => unreachable!("temper's requests carry no value"),
    };
    Some(Feedback { head, refused: false, after: None })
}
/// Start and message feedback; question feedback stays beside its list (temper/docs/design/client/pages/README.md).
pub(crate) fn composer_refusals(root: &Domain, w: &Words) -> List<Feedback> {
    let mut feedback = List::with_capacity(2);
    if let Some(chat) = root.pages().chat() {
        if chat.outcome_kind == Some(Sending::Message)
            && let Some(outcome) = &chat.outcome
            && let Some(line) = outcome_feedback(outcome, w, false)
        {
            feedback.push(line).expect("one message outcome");
        }
    } else {
        match root.pages().address() {
            Address::Ask { .. } | Address::Home { .. } | Address::NewChat { .. } => {
                if let Some(chats) = root.pages().chats() {
                    if chats.refusal.is_some() {
                        feedback.push(refused(&c::Refusal::NotOffered, w, false)).expect("one local refusal");
                    }
                    if let Some(outcome) = &chats.outcome
                        && let Some(line) = outcome_feedback(outcome, w, false)
                    {
                        feedback.push(line).expect("one start outcome");
                    }
                }
            }
            Address::Chat { .. }
            | Address::AllHome
            | Address::Inbox { .. }
            | Address::Chats { .. }
            // page-view: draw Remembered.
            | Address::Told
            | Address::Remembered
            | Address::Members
            | Address::Limits
            | Address::Accounts
            | Address::Forges
            | Address::ConnectForge { .. }
            | Address::ConnectModel { .. }
            | Address::Welcome { .. }
            | Address::Task { .. }
            | Address::Work { .. }
            | Address::Goals { .. }
            | Address::Changes { .. }
            | Address::Team { .. }
            | Address::Settings { .. }
            | Address::Models { .. }
            | Address::Builders { .. }
            | Address::Role { .. }
            | Address::Rule { .. }
            | Address::Repositories { .. }
            | Address::Adopt { .. } => {}
        }
    }
    feedback
}
