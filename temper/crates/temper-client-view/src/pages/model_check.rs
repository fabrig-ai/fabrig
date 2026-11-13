//! Model’s line in the welcome’s pane (temper/docs/design/client/pages/README.md, *The try*; temper/docs/design/domain/trial.md, 6.1–6.4).
use crate::{
    accounts::{Bindings, Meaning},
    flow::{self, Control, Mark},
    look,
    words::{Shell, Word, Words},
};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_lib::List;
use skein_ui::{Builder, Role, phrases::Hole};
use temper_client_domain::forge::trials;
use temper_client_domain::{
    Address, Domain, Step,
    pages::{
        accounts::Way,
        connect_model::{self, Act, ProgramCheck},
        welcome,
    },
};

pub(crate) enum First {
    Subscription,
    Plan,
    Key,
}
pub(crate) enum Answer {
    SignIn { way: Way },
}
pub(crate) enum State {
    Running { at: usize },
    Usable,
    Answered { seconds: u64 },
    Stopped { at: usize, why: Box<[u8]>, answer: Option<Answer> },
}
pub(crate) struct Trial {
    pub(crate) account: Box<[u8]>,
    pub(crate) account_number: Option<u32>,
    pub(crate) who: Box<[u8]>,
    pub(crate) detail: Option<Box<[u8]>>,
    pub(crate) first: First,
    pub(crate) state: State,
}
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
/// A failed trial for Home (domain/trial.md, 6.3); its caller is wired by home-line.
pub(crate) fn stopped(root: &Domain, w: &Words) -> Option<(Box<[u8]>, u32)> {
    if root.forge().trial_asked().is_some() {
        return None;
    }
    for row in root.forge().trials() {
        match row.state {
            trials::State::Failed { .. } => return Some((row_trial(root, w, row).who, row.account)),
            trials::State::Running | trials::State::Answered { .. } => {}
        }
    }
    None
}
fn identity_detail(account: &[u8], full: Option<Box<[u8]>>) -> Option<Box<[u8]>> {
    let full = full?;
    let rest = full.strip_prefix(account)?.strip_prefix(" · ".as_bytes())?;
    Some(rest.into())
}
#[expect(clippy::manual_find, reason = "bounded source and account selection uses explicit loops")]
fn account_line(root: &Domain, number: u32) -> Option<&c::AccountLine> {
    for row in &temper_client_domain::pages::accounts::document(root.client())?.accounts {
        if row.number == number {
            return Some(row);
        }
    }
    None
}
fn first_source(row: &c::AccountLine) -> Option<&c::AccountSource> {
    for source in &row.sources {
        if source.usable {
            return Some(source);
        }
    }
    row.sources.get(0)
}
fn sign_in(row: &c::AccountLine, source: &c::AccountSource) -> Option<Answer> {
    let provider = row.provider?;
    match source.kind {
        c::AccountKind::Program => Some(Answer::SignIn { way: Way::Program { provider } }),
        c::AccountKind::SignIn => Some(Answer::SignIn { way: Way::SignIn { provider, plan: row.plan } }),
        c::AccountKind::Key | c::AccountKind::None => None,
    }
}
fn row_trial(root: &Domain, w: &Words, row: &trials::Row) -> Trial {
    let number = row.took.unwrap_or(row.account);
    let account = account_line(root, number);
    let source = match account {
        Some(account) => first_source(account),
        None => None,
    };
    let first = match source {
        Some(source) => match source.kind {
            c::AccountKind::Program => First::Subscription,
            c::AccountKind::SignIn => First::Plan,
            c::AccountKind::Key | c::AccountKind::None => First::Key,
        },
        None => First::Key,
    };
    let (name, detail, who, answer) = match (account, source) {
        (Some(account), Some(source)) => {
            let name = super::accounts::account_words(root, w, account, source, false);
            let detail = identity_detail(&name, Some(super::accounts::account_words(root, w, account, source, true)));
            let provider = match &account.provider_name {
                Some(name) => name.bytes(),
                None => account.display.bytes(),
            };
            let who = crate::words::shown(crate::accounts::words::provider_words(provider).1, w);
            (name, detail, who, sign_in(account, source))
        }
        (Some(_), None) | (None, Some(_) | None) => {
            let name = crate::accounts::words::say(w, crate::accounts::words::Word::Account, &[]);
            (name.clone(), None, name, None)
        }
    };
    let at = match first {
        First::Subscription => match row.step {
            trials::Step::Starts => 1,
            trials::Step::Answers => 2,
        },
        First::Plan | First::Key => 1,
    };
    let state = match row.state {
        trials::State::Running => match root.forge().trial_wait() {
            Some(wait) if !crate::choice::wait_settles(wait) => {
                State::Stopped { at, why: crate::choice::wait_why(root, w, wait), answer }
            }
            Some(_) | None => State::Running { at },
        },
        trials::State::Answered { seconds } => State::Answered { seconds: u64::from(seconds) },
        trials::State::Failed { why } => {
            let (why, answering) = failure(root, w, why, &who, answer);
            State::Stopped { at, why, answer: answering }
        }
    };
    Trial { account: name, account_number: Some(row.account), who, detail, first, state }
}
fn failure(
    root: &Domain,
    w: &Words,
    why: trials::Why,
    who: &[u8],
    answer: Option<Answer>,
) -> (Box<[u8]>, Option<Answer>) {
    match why {
        trials::Why::Waited => match root.forge().trial_wait() {
            Some(wait) => (crate::choice::wait_why(root, w, wait), answer),
            None => (named(w, Word::WelcomeTryWaited, who), None),
        },
        trials::Why::Ended { class: trials::Class::Permanent } => (named(w, Word::WelcomeTryRefused, who), answer),
        trials::Why::Ended { class: trials::Class::Transient | trials::Class::Infrastructure } => {
            (named(w, Word::WelcomeTryBusy, who), None)
        }
        trials::Why::Ended { class: trials::Class::Run | trials::Class::Agent | trials::Class::Invalid } => {
            (named(w, Word::WelcomeTryRun, who), None)
        }
        trials::Why::Ended { class: trials::Class::Lost } => (w.say(Word::WelcomeTryLost, &[]), None),
        trials::Why::Ended { class: trials::Class::Refused } | trials::Why::Refused => {
            (w.say(Word::WelcomeTryCouldNotStart, &[]), None)
        }
    }
}
/// The page determines where the quiet sign-in link goes (domain/trial.md, 6.2).
pub(crate) enum Place {
    Welcome,
    Row,
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
