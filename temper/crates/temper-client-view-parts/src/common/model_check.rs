//! Shared setup items (temper/docs/design/client/view-crates.md, 3.3).
use crate::words::{Times, Word, Words};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_ui::phrases::Hole;
use temper_client_domain::{Domain, forge::trials, pages::accounts::Way};

#[derive(Debug)]
pub enum First {
    Subscription,
    Plan,
    Key,
}

#[derive(Debug)]
pub enum Answer {
    SignIn { way: Way },
    AllWays,
}

#[derive(Debug)]
pub enum State {
    Running { at: usize },
    Usable,
    Answered { seconds: u64 },
    Stopped { at: usize, why: Box<[u8]>, answer: Option<Answer> },
    Limited { passed: bool, why: Box<[u8]> },
}

#[derive(Debug)]
pub struct Trial {
    pub account: Box<[u8]>,
    pub account_number: Option<u32>,
    pub who: Box<[u8]>,
    pub detail: Option<Box<[u8]>>,
    pub first: First,
    pub state: State,
}

/// A failed or limited trial for Home (domain/trial.md, 6.3).
#[must_use]
pub fn stopped(root: &Domain, w: &Words, times: &mut Times) -> Option<(Box<[u8]>, u32)> {
    if root.forge().trial_asked().is_some() {
        return None;
    }
    for row in root.forge().trials() {
        match row.state {
            trials::State::Failed { .. } => {
                let trial = row_trial(root, w, row, times);
                return Some((named(w, Word::HomeTryStopped, &trial.who), row.account));
            }
            trials::State::Limited { .. } => {
                if let State::Limited { why, .. } = row_trial(root, w, row, times).state {
                    return Some((why, row.account));
                }
            }
            trials::State::Running | trials::State::Answered { .. } => {}
        }
    }
    None
}

#[must_use]
pub fn identity_detail(account: &[u8], full: Option<Box<[u8]>>) -> Option<Box<[u8]>> {
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

#[must_use]
pub fn row_trial(root: &Domain, w: &Words, row: &trials::Row, times: &mut Times) -> Trial {
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
                State::Stopped { at, why: crate::choice::wait_why(root, w, wait), answer: wait_answer(root, wait) }
            }
            Some(_) | None => State::Running { at },
        },
        trials::State::Answered { seconds } => State::Answered { seconds: u64::from(seconds) },
        trials::State::Limited { reset } => limited(w, times, &name, account, reset),
        trials::State::Failed { why } => {
            let (why, answering) = failure(root, w, why, &who, answer);
            State::Stopped { at, why, answer: answering }
        }
    };
    Trial { account: name, account_number: Some(row.account), who, detail, first, state }
}

fn limited(
    w: &Words,
    times: &mut Times,
    name: &[u8],
    account: Option<&c::AccountLine>,
    reset: Option<skein_lib::Wall>,
) -> State {
    let Some(reset) = reset else {
        return State::Limited { passed: false, why: named(w, Word::WelcomeTryLimitedUnknownReset, name) };
    };
    let Some(clock) = times.trial_reset(w, reset) else {
        return State::Limited { passed: true, why: named(w, Word::WelcomeTryLimitHasPassed, name) };
    };
    let window = w.say(limited_window(account), &[]);
    let why = w.say(Word::WelcomeTryLimitedUntil, &[Hole::Name(name.into()), Hole::Name(window), Hole::Name(clock)]);
    State::Limited { passed: false, why }
}

fn limited_window(account: Option<&c::AccountLine>) -> Word {
    let Some(account) = account else { return Word::WelcomeTryUsageLimit };
    let mut window = None;
    for fact in &account.facts {
        if fact.kind == c::FACT_USAGE
            && let c::AccountFactValue::Window(value) = &fact.value
            && value.state == c::WINDOW_LIMITED
        {
            if window.is_some() {
                return Word::WelcomeTryUsageLimit;
            }
            window = Some(fact.label.as_ref());
        }
    }
    match window {
        Some(b"five_hour") => Word::WelcomeTryFiveHourLimit,
        Some(b"seven_day") => Word::WelcomeTryWeeklyLimit,
        Some(_) | None => Word::WelcomeTryUsageLimit,
    }
}

fn wait_answer(root: &Domain, wait: &c::TaskRunWait) -> Option<Answer> {
    use c::{TaskSourceState as S, TaskWaitWhy as R};
    let c::TaskRunWait::Account { account, source, why } = *wait else { return None };
    if let R::Source(S::Limited { .. }) = why {
        return Some(Answer::AllWays);
    }
    let needs_sign_in = why == R::Host || why == R::Source(S::NeedsSignIn);
    let account = account_line(root, account?)?;
    for entry in &account.sources {
        if Some(entry.number) == source && (needs_sign_in || entry.expiring) {
            return sign_in(account, entry);
        }
    }
    None
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
            Some(wait) => (crate::choice::wait_why(root, w, wait), wait_answer(root, wait)),
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

#[must_use]
pub fn named(w: &Words, word: Word, name: &[u8]) -> Box<[u8]> {
    w.say(word, &[Hole::Name(skein_lib::bytes::copy_of(name))])
}
