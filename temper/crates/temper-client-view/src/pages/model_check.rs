//! The try of a model, its status box (temper/docs/design/client/README.md, 3.5, *The try*).
use crate::{
    accounts::{Bindings, Meaning},
    flow::{self, Control, Mark},
    look,
    words::{Shell, Word, Words},
};
use alloc::boxed::Box;
use skein_ui::{Builder, Role, Run, phrases::Hole};
use temper_client_domain::{
    Address, Domain, Step,
    pages::{
        accounts::Way,
        connect_model::{self, Act, ProgramCheck},
    },
};

/// The steps' first line: what the way's account has already done.
#[cfg_attr(
    not(test),
    expect(dead_code, reason = "a plan and a key have no trial fact yet; their boxes are drawn in tests")
)]
pub(crate) enum First {
    Subscription,
    Plan,
    Key,
}
/// The act that answers a reason.
pub(crate) enum Answer {
    SignIn,
}
#[cfg_attr(not(test), expect(dead_code, reason = "no trial supplies an answered fact yet; its box is drawn in tests"))]
pub(crate) enum State {
    /// `at` counts from the first step; `message`, whether a message goes.
    Running {
        at: usize,
        message: bool,
    },
    Answered {
        seconds: u64,
    },
    Stopped {
        at: usize,
        why: Box<[u8]>,
        answer: Option<Answer>,
    },
}
pub(crate) struct Trial {
    /// `Claude subscription`, `ChatGPT plan`, `OpenAI API key`.
    pub(crate) account: Box<[u8]>,
    /// `Claude`, `ChatGPT`.
    pub(crate) who: Box<[u8]>,
    pub(crate) first: First,
    pub(crate) state: State,
}
/// Which state the box draws: the host's check alone, no trial being a fact yet.
/// The one function temper/docs/design/draft/model-check.md, 2 to 4, feeds.
pub(crate) fn trial(root: &Domain, w: &Words) -> Option<Trial> {
    let Address::Welcome { step: Step::Model { .. } } = root.pages().address() else { return None };
    let Way::Program { .. } = connect_model::way(root.pages(), root.client())? else { return None };
    let state = match root.pages().connect_model()?.check {
        ProgramCheck::Idle => return None,
        ProgramCheck::Declaring | ProgramCheck::Checking { .. } => State::Running { at: 0, message: false },
        ProgramCheck::NotYet => {
            State::Stopped { at: 0, why: w.say(Word::WelcomeClaudeNotYet, &[]), answer: Some(Answer::SignIn) }
        }
    };
    Some(Trial {
        account: crate::accounts::words::say(
            w,
            crate::accounts::words::Word::Subscription,
            &[Hole::Name(Box::from(b"Claude".as_slice()))],
        ),
        who: Box::from(b"Claude".as_slice()),
        first: First::Subscription,
        state,
    })
}
/// Whether the last try stopped: no fact yet (temper/docs/design/draft/model-check.md, 4).
pub(crate) const fn last_try_failed() -> bool {
    false
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
        State::Answered { .. } => Mark::Done,
        State::Running { at, .. } | State::Stopped { at, .. } => {
            if index < *at {
                return Mark::Done;
            }
            if index > *at {
                return Mark::ToCome;
            }
            match state {
                State::Running { .. } => Mark::UnderWay,
                State::Stopped { .. } => Mark::Stopped,
                State::Answered { .. } => Mark::Done,
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
/// The box.
pub(crate) fn status(b: &mut Builder, w: &Words, trial: &Trial) {
    b.open(Role::Slot);
    b.style(look::WELCOME_FORGE_STATUS);
    match trial.state {
        State::Answered { .. } => b.style(look::FLOW_DONE),
        State::Running { .. } | State::Stopped { .. } => {}
    }
    b.style(look::WELCOME_CHECK);
    let (title, muted, role) = match trial.state {
        State::Running { message, .. } => (
            named(w, Word::WelcomeTrying, &trial.account),
            if message { Some(w.say(Word::WelcomeTryShort, &[])) } else { None },
            Role::Status,
        ),
        State::Answered { seconds } => {
            (trial.account.clone(), Some(w.say(Word::WelcomeTryAnswered, &[Hole::Number(seconds)])), Role::Status)
        }
        State::Stopped { .. } => {
            b.style(look::WELCOME_CHECK_STOPPED);
            (named(w, Word::WelcomeTryNoAnswer, &trial.who), None, Role::Alert)
        }
    };
    b.open(role);
    let name = match &muted {
        Some(muted) => crate::accounts::words::join(&[&title, b" ", muted]),
        None => title.clone(),
    };
    b.name(&name);
    match trial.state {
        State::Running { .. } => {
            if w.shell == Shell::Browser {
                b.open(Role::Slot);
                b.style(look::WELCOME_SPIN);
                b.close();
            }
        }
        State::Answered { .. } => {
            if w.shell == Shell::Browser {
                b.open(Role::Slot);
                b.style(look::WELCOME_OK);
                b.text(Run::Plain, "✓".as_bytes());
                b.close();
            } else {
                b.text(Run::Plain, "✓ ".as_bytes());
            }
        }
        State::Stopped { .. } => {}
    }
    b.open(Role::Text);
    b.run(Run::Strong, &title);
    b.close();
    if let Some(muted) = muted {
        if w.shell == Shell::Terminal {
            b.text(Run::Plain, " · ".as_bytes());
        }
        b.open(Role::Slot);
        b.style(look::FLOW_MUTED);
        b.text(Run::Plain, &muted);
        b.close();
    }
    b.close();
    if let State::Answered { .. } = trial.state {
        b.open(Role::Disclosure);
        b.name(&w.say(Word::ForgeProgress, &[]));
        progress(b, w, trial);
        b.close();
    } else {
        progress(b, w, trial);
    }
    if let State::Stopped { why, .. } = &trial.state {
        b.open(Role::Paragraph);
        b.style(look::WELCOME_CHECK_WHY);
        b.name(why);
        b.text(Run::Plain, why);
        b.close();
    }
    b.close();
}
/// A stopped try's controls, in a `WELCOME_ACTS` slot. `go_on` is where "Continue anyway" goes.
pub(crate) fn stopped_acts(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    trial: &Trial,
    way: Way,
    go_on: Address,
) {
    let State::Stopped { answer, .. } = &trial.state else { return };
    b.open(Role::Slot);
    b.style(look::WELCOME_ACTS);
    super::connect_model::button(
        b,
        bindings,
        &w.say(Word::WelcomeTryAgain, &[]),
        Act::SignedIn { name: root.pages().account_name(), way },
        false,
        true,
    );
    match answer {
        Some(Answer::SignIn) => super::connect_model::button(
            b,
            bindings,
            &w.say(Word::WelcomeSignInAgain, &[]),
            Act::SignInAgain,
            false,
            false,
        ),
        None => {
            let address = connect_model::address(root.pages().address(), None);
            if let Some(to) = bindings.insert(Meaning::Go(address)) {
                flow::control(b, Control::SecondaryLink { name: &w.say(Word::WelcomeAllWays, &[]), to });
            }
        }
    }
    if let Some(to) = bindings.insert(Meaning::Go(go_on)) {
        flow::control(b, Control::Link { name: &w.say(Word::WelcomeContinueAnyway, &[]), to });
    }
    b.close();
}
/// Ready's muted line after a try that stopped.
pub(crate) fn ready_note(b: &mut Builder, w: &Words) {
    b.open(Role::Slot);
    b.style(look::FLOW_MUTED);
    w.text(b, Word::WelcomeTryFailed, &[]);
    b.close();
}
