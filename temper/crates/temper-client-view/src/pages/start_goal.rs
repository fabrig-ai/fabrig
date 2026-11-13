//! The draft's start sheet (temper/docs/design/client/pages/goal.md, 3).
use crate::{
    bindings::Bindings,
    look,
    words::{Word, Words},
};
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_ui::{Builder, Role, Run, Shortcut, State, phrases::Hole};
use temper_client_domain::{
    Address, Domain,
    pages::{
        Act, goal,
        start_goal::{self, Failure, StartGoal, Unfunded},
    },
};

pub(crate) fn sheet(b: &mut Builder, root: &Domain, start: &StartGoal, words: &Words, bindings: &mut Bindings) {
    let title = match &start.given {
        Some(path) => words.say(Word::StartGoalFrom, &[Hole::Name(goal::file_of(path).into())]),
        None => words.say(Word::StartGoalTitle, &[]),
    };
    b.open(Role::Dialog);
    b.style(look::SHEET);
    b.name(&title);
    b.open(Role::Form);
    b.name(&title);
    b.open(Role::Slot);
    crate::cards::text(b, Role::Heading { level: 2 }, &title);
    if start.given.is_some() {
        b.open(Role::Paragraph);
        b.style(look::SHEET_MUTED);
        words.text(b, Word::StartGoalLead, &[]);
        b.close();
    }
    b.close();
    if start.given.is_none() {
        b.open(Role::Slot);
        b.style(look::GOAL_FIELD);
        b.open(Role::TextField);
        b.style(look::GOAL_TEXT);
        b.name(&words.say(Word::StartGoalDraft, &[]));
        b.placeholder(&words.say(Word::StartGoalDraftHint, &[]));
        b.value(&start.draft, temper_client_domain::pages::shown(root.pages()));
        b.bind(bindings.insert(Act::StartDraft { words: Box::default() }));
        if start.pending.is_some() {
            b.state(State::Disabled);
        }
        b.close();
        b.close();
    }
    b.open(Role::Slot);
    b.style(look::GOAL_FIELD);
    b.open(Role::TextField);
    b.style(look::GOAL_TEXT);
    b.name(&words.say(Word::StartGoalName, &[]));
    b.value(&start.name, temper_client_domain::pages::shown(root.pages()));
    b.bind(bindings.insert(Act::StartName { words: Box::default() }));
    if start.pending.is_some() {
        b.state(State::Disabled);
    }
    b.close();
    b.close();
    ways(b, root, start, words, bindings);
    asks(b, start, words, bindings);
    if let Some(error) = &start.error {
        failure(b, words, error);
    }
    let unfunded = match root.pages().me(root.client()) {
        Some(me) => start_goal::unfunded(me, start.project, start.way),
        None => None,
    };
    if let Some(why) = unfunded {
        b.open(Role::Slot);
        b.style(look::SHEET_ERROR);
        let word = match why {
            Unfunded::Held => Word::StartGoalNoFundsHeld,
            Unfunded::Empty => Word::StartGoalNoFunds,
        };
        words.text(b, word, &[]);
        b.close();
    }
    b.open(Role::Slot);
    b.style(look::SHEET_FOOT);
    b.open(Role::Button);
    b.style(look::GOAL_BTN);
    b.style(look::GOAL_QUIET);
    b.name(&words.say(Word::Cancel, &[]));
    b.bind(bindings.insert(Act::StartCancel));
    b.shortcut(Shortcut::ESCAPE);
    b.close();
    b.open(Role::Button);
    b.style(look::GOAL_BTN);
    b.style(look::GOAL_PRIMARY);
    b.name(&words.say(Word::StartGoalRun, &[]));
    b.bind(bindings.insert(Act::StartGoal));
    if start.name.trim_ascii().is_empty()
        || start.draft.trim_ascii().is_empty()
        || start.pending.is_some()
        || unfunded.is_some()
    {
        b.state(State::Disabled);
    }
    b.close();
    b.close();
    b.close();
    b.close();
}

fn ways(b: &mut Builder, root: &Domain, start: &StartGoal, words: &Words, bindings: &mut Bindings) {
    b.open(Role::ChoiceGroup);
    b.style(look::SHEET_FIELDSET);
    b.name(&words.say(Word::GoalHowTitle, &[]));
    b.open(Role::Heading { level: 3 });
    b.style(look::SHEET_FIELDSET);
    words.text(b, Word::GoalHowTitle, &[]);
    b.close();
    if let Some(me) = root.pages().me(root.client()) {
        let default = start.default;
        for way in &super::goals::ways(me, start.project, words) {
            b.open(Role::Slot);
            b.style(look::SHEET_OPTION);
            b.open(Role::Choice);
            b.name(&way.name);
            b.bind(bindings.insert(Act::StartWay { template: way.number }));
            if start.way == Some(way.number) {
                b.state(State::Selected);
            }
            if start.pending.is_some() {
                b.state(State::Disabled);
            }
            b.close();
            b.open(Role::Text);
            b.style(look::SHEET_MUTED);
            if let Some(line) = &way.line {
                b.text(Run::Plain, line);
            }
            if default == Some(way.number)
                && let Some(project) = super::project_name(Some(me), start.project)
            {
                words.text(b, Word::StartGoalDefault, &[Hole::Name(project)]);
            }
            b.close();
            b.close();
        }
    }
    let name = words.say(Word::StartGoalWays, &[]);
    crate::bindings::link(b, &name, bindings.insert(Act::Go { address: Address::Settings { project: start.project } }));
    b.style(look::SHEET_LINK);
    b.text(Run::Plain, &name);
    b.close();
    b.close();
}

fn asks(b: &mut Builder, start: &StartGoal, words: &Words, bindings: &mut Bindings) {
    b.open(Role::ChoiceGroup);
    b.style(look::SHEET_FIELDSET);
    b.name(&words.say(Word::StartGoalAsks, &[]));
    b.open(Role::Heading { level: 3 });
    b.style(look::SHEET_FIELDSET);
    words.text(b, Word::StartGoalAsks, &[]);
    b.close();
    for (always, title, detail) in [
        (false, Word::StartGoalFollow, Word::StartGoalFollowDetail),
        (true, Word::StartGoalAlways, Word::StartGoalAlwaysDetail),
    ] {
        b.open(Role::Slot);
        b.style(look::SHEET_OPTION);
        b.open(Role::Choice);
        b.name(&words.say(title, &[]));
        b.bind(bindings.insert(Act::StartAsks { always }));
        if start.always == always {
            b.state(State::Selected);
        }
        if start.pending.is_some() {
            b.state(State::Disabled);
        }
        b.close();
        b.open(Role::Text);
        b.style(look::SHEET_MUTED);
        words.text(b, detail, &[]);
        b.close();
        b.close();
    }
    b.close();
}

fn failure(b: &mut Builder, words: &Words, failure: &Failure) {
    b.open(Role::Slot);
    b.style(look::SHEET_ERROR);
    match failure {
        Failure::Refused(refusal) => words.refusal(b, refusal, words.name_bytes),
        Failure::NotOffered(_) => words.refusal(b, &c::Refusal::NotOffered, words.name_bytes),
        Failure::Outcome(outcome) => crate::confirm::outcome_words(b, words, outcome),
        Failure::TooLong => words.refusal(
            b,
            &c::Refusal::Limit { which: c::Limit::Words { limit: words.name_bytes } },
            words.name_bytes,
        ),
    }
    b.close();
}
