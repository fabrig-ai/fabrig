//! Model’s line in the welcome’s pane (temper/docs/design/client/pages/README.md, *The try*; temper/docs/design/domain/trial.md, 6.1–6.4).
pub(crate) use crate::common::model_check::{Answer, First, State, Trial, identity_detail, named, row_trial};
use crate::{
    accounts::{Bindings, Meaning},
    flow::{self, Control, Mark},
    look,
    words::{Shell, Word, Words},
};
use alloc::boxed::Box;
use skein_lib::List;
use skein_ui::{Builder, Role, phrases::Hole};
use temper_client_domain::forge::trials;
use temper_client_domain::{
    Address, Domain, Step,
    pages::{
        connect_model::{self, Act, ProgramCheck},
        welcome,
    },
};

/// The host’s check, then the newest trial (temper/docs/design/domain/trial.md, 6.1).
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
                account_number: None,
                who: b"Claude".as_slice().into(),
                detail: None,
                first: First::Subscription,
                state,
            });
        }
    }
    if let Some(account) = root.forge().trial_asked() {
        return Some(row_trial(
            root,
            w,
            &trials::Row { account, task: None, took: None, step: trials::Step::Starts, state: trials::State::Running },
        ));
    }
    if let Some(row) = root.forge().trials().first() {
        return of_account(root, w, row.took.unwrap_or(row.account));
    }
    if welcome::model_usable(root) != Some(true) {
        return None;
    }
    let account = match super::accounts::first_usable_kind(root, w) {
        Some(account) => account,
        None => crate::choice::new_chat_model(root, w),
    };
    let detail = identity_detail(&account, super::accounts::first_usable_short(root, w));
    Some(Trial {
        account,
        account_number: None,
        who: Box::new([]),
        detail,
        first: First::Subscription,
        state: State::Usable,
    })
}
/// An account's own line follows the account the trial took (domain/trial.md, 6.2).
pub(crate) fn of_account(root: &Domain, w: &Words, account: u32) -> Option<Trial> {
    if root.forge().trial_asked() == Some(account) {
        return Some(row_trial(
            root,
            w,
            &trials::Row { account, task: None, took: None, step: trials::Step::Starts, state: trials::State::Running },
        ));
    }
    for row in root.forge().trials() {
        if row.took.unwrap_or(row.account) == account {
            return Some(row_trial(root, w, row));
        }
    }
    None
}
/// The page determines where the quiet sign-in link goes (domain/trial.md, 6.2).
pub(crate) enum Place {
    Welcome,
    Row,
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
pub(crate) fn line(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, trial: &Trial, place: Place) {
    let mut details: List<Box<[u8]>> = List::with_capacity(2);
    let (name, mark) = match &trial.state {
        State::Running { at } => {
            let step = step(w, trial, *at);
            let trying = named(w, Word::WelcomeTryStep, &step);
            let mut name = w.say(Word::WelcomeTryLine, &[Hole::Name(trial.account.clone()), Hole::Name(step)]);
            details.push(trying).expect("one running detail");
            match trial.first {
                First::Key => {
                    let spend = w.say(Word::WelcomeTrySpend, &[]);
                    name = crate::accounts::words::join(&[&name, " · ".as_bytes(), &spend]);
                    details.push(spend).expect("key spend detail fits");
                }
                First::Subscription | First::Plan => {}
            }
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
        if let Some(detail) = details.get(0) {
            b.name(detail);
        }
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
        retry(b, root, w, bindings, trial);
        let (word, way) = match answer {
            Some(Answer::SignIn { way }) => (Word::WelcomeSignInAgain, Some(connect_model::model_way(*way))),
            None => (Word::WelcomeAllWays, None),
        };
        let address = match place {
            Place::Welcome => Some(Address::Welcome { step: Step::Model { way } }),
            Place::Row => match way {
                Some(way) => Some(Address::ConnectModel { way: Some(way) }),
                None => None,
            },
        };
        if let Some(address) = address
            && let Some(to) = bindings.insert(Meaning::Go(address))
        {
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
fn retry(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, trial: &Trial) {
    if let Some(account) = trial.account_number {
        if root.forge().trial_asked().is_none() {
            for row in root.forge().trials() {
                if row.account == account
                    && let trials::State::Failed { .. } = row.state
                {
                    if let Some(to) = bindings.insert(Meaning::Try { account }) {
                        flow::control(b, Control::Button { name: &w.say(Word::WelcomeTryAgain, &[]), to });
                    }
                    break;
                }
            }
        }
    } else if let Some(page) = root.pages().connect_model()
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
