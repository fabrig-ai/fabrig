//! Model’s line in Ready’s pane: the try’s states (temper/docs/design/client/README.md, 3.5, *The try*).
use crate::{
    accounts::{Bindings, Meaning},
    flow::{self, Control, Mark},
    look,
    words::{Shell, Word, Words},
};
use alloc::boxed::Box;
use skein_lib::List;
use skein_ui::{Builder, Role, phrases::Hole};
use temper_client_domain::{
    Address, Domain, Step,
    pages::{
        accounts::Way,
        connect_model::{self, Act, ProgramCheck},
        welcome,
    },
};

#[cfg_attr(not(test), expect(dead_code, reason = "plan and key trials have no fact yet; tests draw their steps"))]
pub(crate) enum First {
    Subscription,
    Plan,
    Key,
}
pub(crate) enum Answer {
    SignIn { way: Way },
}
#[cfg_attr(not(test), expect(dead_code, reason = "no trial supplies elapsed time yet; tests draw Answered"))]
pub(crate) enum State {
    Running { at: usize },
    Usable,
    Answered { seconds: u64 },
    Stopped { at: usize, why: Box<[u8]>, answer: Option<Answer> },
}
pub(crate) struct Trial {
    pub(crate) account: Box<[u8]>,
    pub(crate) who: Box<[u8]>,
    pub(crate) detail: Option<Box<[u8]>>,
    pub(crate) first: First,
    pub(crate) state: State,
}
/// The host’s check and accounts alone, no trial being a fact yet.
/// The one function temper/docs/design/draft/model-check.md, 2 to 4, feeds.
pub(crate) fn trial(root: &Domain, w: &Words) -> Option<Trial> {
    if let Some(page) = root.pages().connect_model() {
        let state = match page.check {
            ProgramCheck::Idle => None,
            ProgramCheck::Declaring | ProgramCheck::Checking { .. } => Some(State::Running { at: 0 }),
            ProgramCheck::NotYet => Some(State::Stopped {
                at: 0,
                why: w.say(Word::WelcomeClaudeNotYet, &[]),
                answer: match page.tried {
                    Some(way) => Some(Answer::SignIn { way }),
                    None => None,
                },
            }),
        };
        if let Some(state) = state {
            return Some(Trial {
                account: crate::accounts::words::say(
                    w,
                    crate::accounts::words::Word::Subscription,
                    &[Hole::Name(b"Claude".as_slice().into())],
                ),
                who: b"Claude".as_slice().into(),
                detail: None,
                first: First::Subscription,
                state,
            });
        }
    }
    if welcome::model_usable(root) != Some(true) {
        return None;
    }
    let account = match super::accounts::first_usable_kind(root, w) {
        Some(account) => account,
        None => crate::choice::new_chat_model(root, w),
    };
    let detail = if let Some(full) = super::accounts::first_usable_short(root, w) {
        match full.strip_prefix(account.as_ref()).unwrap_or(b"").strip_prefix(" · ".as_bytes()) {
            Some(rest) => Some(rest.into()),
            None => None,
        }
    } else {
        None
    };
    Some(Trial { account, who: Box::new([]), detail, first: First::Subscription, state: State::Usable })
}
fn named(w: &Words, word: Word, name: &[u8]) -> Box<[u8]> {
    w.say(word, &[Hole::Name(skein_lib::bytes::copy_of(name))])
}
fn mark(state: &State, index: usize, first: &First) -> Mark {
    if index == 0 {
        match first {
            First::Plan | First::Key => return Mark::Done,
            First::Subscription => {}
        }
    }
    match state {
        State::Usable | State::Answered { .. } => Mark::Done,
        State::Running { at } | State::Stopped { at, .. } => {
            if index < *at {
                return Mark::Done;
            }
            if index > *at {
                return Mark::ToCome;
            }
            match state {
                State::Running { .. } => Mark::UnderWay,
                State::Stopped { .. } => Mark::Stopped,
                State::Usable | State::Answered { .. } => Mark::Done,
            }
        }
    }
}
fn progress(b: &mut Builder, w: &Words, trial: &Trial) {
    let first = match trial.first {
        First::Subscription => w.say(Word::WelcomeTrySubscription, &[]),
        First::Plan => named(w, Word::WelcomeTryPlan, &trial.who),
        First::Key => w.say(Word::WelcomeTryKey, &[]),
    };
    let starts = w.say(Word::WelcomeTryStarts, &[]);
    let answers = named(w, Word::WelcomeTryAnswers, &trial.who);
    let name = w.say(Word::ForgeProgress, &[]);
    match trial.first {
        First::Subscription => flow::progress(
            b,
            w,
            &name,
            &[
                (&first, mark(&trial.state, 0, &trial.first)),
                (&starts, mark(&trial.state, 1, &trial.first)),
                (&answers, mark(&trial.state, 2, &trial.first)),
            ],
        ),
        First::Plan | First::Key => flow::progress(
            b,
            w,
            &name,
            &[(&first, mark(&trial.state, 0, &trial.first)), (&answers, mark(&trial.state, 1, &trial.first))],
        ),
    }
}

/// Model’s line. `model_counts` decides the heading and act, independently of this line.
pub(crate) fn line(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, trial: &Trial) {
    let mut details: List<Box<[u8]>> = List::with_capacity(2);
    let (name, mark) = match &trial.state {
        State::Running { at } => {
            let step = step(w, trial, *at);
            let trying = named(w, Word::WelcomeTryStep, &step);
            let name = w.say(Word::WelcomeTryLine, &[Hole::Name(trial.account.clone()), Hole::Name(step)]);
            details.push(trying).expect("one running detail");
            (name, Mark::ToCome)
        }
        State::Usable | State::Answered { .. } => {
            if let Some(detail) = &trial.detail {
                details.push(detail.clone()).expect("identity detail fits");
            }
            if let State::Answered { seconds } = trial.state {
                details.push(w.say(Word::WelcomeTryAnswered, &[Hole::Number(seconds)])).expect("elapsed detail fits");
            }
            let mut name = trial.account.clone();
            for detail in &details {
                name = crate::accounts::words::join(&[&name, " · ".as_bytes(), detail]);
            }
            (name, Mark::Done)
        }
        State::Stopped { .. } => {
            let detail = named(w, Word::WelcomeTryNoAnswer, &trial.who);
            let name = crate::accounts::words::join(&[&trial.account, " · ".as_bytes(), &detail]);
            details.push(detail).expect("one stopped detail");
            (name, Mark::Stopped)
        }
    };
    let mut refs: List<&[u8]> = List::with_capacity(2);
    for detail in &details {
        refs.push(detail).expect("two details fit");
    }
    crate::forge::connect::stacked(b, w, &name, &trial.account, refs.as_slice(), mark);
    if let State::Running { .. } = trial.state
        && w.shell == Shell::Browser
    {
        b.open(Role::Progress);
        b.style(look::FLOW_BUSY);
        b.name(details.get(0).expect("running detail"));
        b.close();
    }
    if let State::Stopped { why, answer, .. } = &trial.state {
        b.open(Role::Paragraph);
        b.style(look::WELCOME_STATUS_WHY);
        b.name(why);
        b.text(skein_ui::Run::Plain, why);
        b.close();
        b.open(Role::Slot);
        b.style(look::WELCOME_STATUS_ACTS);
        if let Some(page) = root.pages().connect_model()
            && let Some(way) = page.tried
        {
            super::connect_model::button(
                b,
                bindings,
                &w.say(Word::WelcomeTryAgain, &[]),
                Act::SignedIn { name: root.pages().account_name(), way },
                false,
                false,
            );
        }
        let (word, way) = match answer {
            Some(Answer::SignIn { way }) => (Word::WelcomeSignInAgain, Some(connect_model::model_way(*way))),
            None => (Word::WelcomeAllWays, None),
        };
        if let Some(to) = bindings.insert(Meaning::Go(Address::Welcome { step: Step::Model { way } })) {
            flow::control(b, Control::Link { name: &w.say(word, &[]), to });
        }
        b.close();
    }
    match trial.state {
        State::Running { .. } | State::Stopped { .. } => {
            b.open(Role::Disclosure);
            b.name(&w.say(Word::ForgeProgress, &[]));
            progress(b, w, trial);
            b.close();
        }
        State::Usable | State::Answered { .. } => {}
    }
}
fn step(w: &Words, trial: &Trial, at: usize) -> Box<[u8]> {
    if at == 0 {
        return match trial.first {
            First::Subscription => w.say(Word::WelcomeTrySubscription, &[]),
            First::Plan => named(w, Word::WelcomeTryPlan, &trial.who),
            First::Key => w.say(Word::WelcomeTryKey, &[]),
        };
    }
    if let First::Subscription = trial.first
        && at == 1
    {
        return w.say(Word::WelcomeTryStarts, &[]);
    }
    named(w, Word::WelcomeTryAnswers, &trial.who)
}
