//! The hand-off sheet and give-back confirmation (client/pages/team.md, 1).
use crate::{
    bindings::Bindings,
    cards::text,
    look,
    words::{Word, Words},
};
use alloc::boxed::Box;
use skein_ui::{Builder, Role, Run, Shortcut, State, phrases::Hole};
use temper_client_domain::{
    Domain,
    pages::{
        Act as PageAct,
        team::hand::{Act, Failure, NameProblem, Open, Stock},
    },
};

pub fn sheet(b: &mut Builder, root: &Domain, open: &Open, w: &Words, bindings: &mut Bindings) {
    let title = if let Some(give) = &open.give {
        if give.names.len() == 1 {
            w.say(Word::GiveTitle, &[Hole::Name(give.names.get(0).expect("one area").clone())])
        } else {
            w.say(Word::GiveManyTitle, &[Hole::Number(u64::from(give.names.len()))])
        }
    } else {
        w.say(Word::HandTitle, &[])
    };
    b.open(Role::Dialog);
    b.style(look::SHEET);
    b.name(&title);
    b.open(Role::Form);
    b.name(&title);
    b.open(Role::Slot);
    text(b, Role::Heading { level: 2 }, &title);
    b.close();
    if let Some(give) = &open.give {
        let mut owner = w.say(Word::MemberUnnamed, &[Hole::Number(give.party)]);
        for person in &open.facts.people {
            if person.party == give.party {
                owner = if person.you { w.say(Word::TeamWhomYou, &[]) } else { person.name.clone() };
                break;
            }
        }
        text(
            b,
            Role::Paragraph,
            &w.say(if give.names.len() == 1 { Word::GiveOwner } else { Word::GiveOwners }, &[Hole::Name(owner)]),
        );
    } else {
        areas(b, root, open, w, bindings);
        roles(b, open, w, bindings);
        if open.stock.is_some() {
            budget(b, root, open, w, bindings);
        }
    }
    if let Some(failure) = &open.failure {
        b.open(Role::Slot);
        b.style(look::SHEET_ERROR);
        match failure {
            Failure::Read => w.text(b, Word::HandReadFailed, &[]),
            Failure::Outcome(outcome) => crate::confirm::outcome_words(b, w, outcome),
        }
        b.close();
    }
    foot(b, open, w, bindings);
    b.close();
    b.close();
}
fn areas(b: &mut Builder, root: &Domain, open: &Open, w: &Words, bindings: &mut Bindings) {
    b.open(Role::ChoiceGroup);
    b.style(look::SHEET_FIELDSET);
    b.name(&w.say(Word::HandWhichArea, &[]));
    text(b, Role::Heading { level: 3 }, &w.say(Word::HandWhichArea, &[]));
    if let Some(areas) = &open.facts.areas {
        for area in &areas.rows {
            b.open(Role::Slot);
            b.style(look::SHEET_OPTION);
            choice(
                b,
                &area.name,
                Act::Area(Some(area.name.clone())),
                open.area.as_ref() == Some(&area.name),
                open.locked(),
                bindings,
            );
            b.open(Role::Text);
            b.text(Run::Strong, &area.name);
            b.text(Run::Plain, &area.words);
            b.close();
            let owner = match &area.holder {
                Some(holder) => super::areas::holder_name(w, &open.facts, holder),
                None => super::areas::absent(w, &area.owner, &open.project_name),
            };
            b.open(Role::Text);
            b.style(look::SHEET_MUTED);
            b.text(Run::Plain, &owner);
            b.close();
            b.close();
        }
        if areas.rows.len() < 16 {
            choice(b, &w.say(Word::HandNew, &[]), Act::Area(None), open.area.is_none(), open.locked(), bindings);
        }
    }
    b.close();
    if open.area.is_none() {
        field(b, root, open, w, true, bindings);
        field(b, root, open, w, false, bindings);
    }
}
fn field(b: &mut Builder, root: &Domain, open: &Open, w: &Words, name: bool, bindings: &mut Bindings) {
    b.open(Role::Slot);
    b.style(look::GOAL_FIELD);
    b.open(Role::TextField);
    b.style(look::GOAL_TEXT);
    b.name(&w.say(if name { Word::HandName } else { Word::HandCovers }, &[]));
    b.value(if name { &open.name } else { &open.words }, temper_client_domain::pages::shown(root.pages()));
    b.bind(bindings.insert(PageAct::HandArea(if name {
        Act::Name(Box::default())
    } else {
        Act::Words(Box::default())
    })));
    if open.locked() {
        b.state(State::Disabled);
    }
    b.close();
    if open.judged {
        let error = if name {
            match open.name_problem() {
                Some(NameProblem::Shape) => Some(w.say(Word::HandNameShape, &[])),
                Some(NameProblem::Duplicate) => Some(
                    w.say(Word::HandDuplicate, &[Hole::Name(open.project_name.clone()), Hole::Name(open.name.clone())]),
                ),
                None => None,
            }
        } else if open.words_valid() {
            None
        } else {
            Some(w.say(Word::HandWordsShape, &[]))
        };
        if let Some(error) = error {
            b.open(Role::Text);
            b.style(look::SHEET_ERROR);
            b.text(Run::Plain, &error);
            b.close();
        }
    }
    b.close();
}
fn roles(b: &mut Builder, open: &Open, w: &Words, bindings: &mut Bindings) {
    b.open(Role::ChoiceGroup);
    b.style(look::SHEET_FIELDSET);
    b.name(&w.say(Word::HandWhichRole, &[]));
    text(b, Role::Heading { level: 3 }, &w.say(Word::HandWhichRole, &[]));
    for role in &open.facts.roles {
        choice(b, &role.name, Act::Role(Some(role.task)), open.role == Some(role.task), open.locked(), bindings);
    }
    for (stock, title, line) in [
        (Stock::Prover, Word::HandProver, Word::HandProverLine),
        (Stock::Porter, Word::HandPorter, Word::HandPorterLine),
    ] {
        b.open(Role::Slot);
        b.style(look::SHEET_OPTION);
        choice(b, &w.say(title, &[]), Act::Stock(stock), open.stock == Some(stock), open.locked(), bindings);
        text(b, Role::Paragraph, &w.say(line, &[]));
        b.close();
    }
    choice(
        b,
        &w.say(Word::HandWrite, &[]),
        Act::Role(None),
        open.role.is_none() && open.stock.is_none(),
        open.locked(),
        bindings,
    );
    b.close();
}
fn budget(b: &mut Builder, root: &Domain, open: &Open, w: &Words, bindings: &mut Bindings) {
    b.open(Role::Slot);
    b.style(look::GOAL_FIELD);
    b.open(Role::TextField);
    b.style(look::GOAL_TEXT);
    b.name(&w.say(Word::HandBudget, &[]));
    b.value(&open.budget, temper_client_domain::pages::shown(root.pages()));
    b.bind(bindings.insert(PageAct::HandArea(Act::Budget(Box::default()))));
    if open.busy() {
        b.state(State::Disabled);
    }
    b.close();
    if open.judged && open.budget_value().is_none() {
        b.open(Role::Text);
        b.style(look::SHEET_ERROR);
        w.text(b, Word::HandBudgetShape, &[]);
        b.close();
    }
    b.close();
}
fn choice(b: &mut Builder, title: &[u8], act: Act, selected: bool, busy: bool, bindings: &mut Bindings) {
    b.open(Role::Choice);
    b.name(title);
    b.bind(bindings.insert(PageAct::HandArea(act)));
    if selected {
        b.state(State::Selected);
    }
    if busy {
        b.state(State::Disabled);
    }
    b.close();
}
fn foot(b: &mut Builder, open: &Open, w: &Words, bindings: &mut Bindings) {
    let plural = match &open.give {
        Some(give) => give.names.len() > 1,
        None => false,
    };
    let keep =
        if open.give.is_some() { if plural { Word::GiveKeepMany } else { Word::GiveKeep } } else { Word::HandNotNow };
    let send = if open.give.is_none() && open.role.is_none() && open.stock.is_none() {
        Word::HandAsk
    } else if !open.apply {
        Word::StopAskingPropose
    } else if open.give.is_some() {
        if plural { Word::GiveSendMany } else { Word::GiveSend }
    } else {
        Word::HandOver
    };
    b.open(Role::Slot);
    b.style(look::SHEET_FOOT);
    b.key(skein_ui::Key { owner: look::owner(), kind: 92, a: u64::from(open.project), b: 0 });
    b.open(Role::Button);
    b.style(look::GOAL_BTN);
    b.style(look::GOAL_QUIET);
    b.name(&w.say(keep, &[]));
    b.bind(bindings.insert(PageAct::HandArea(Act::Close)));
    b.shortcut(Shortcut::ESCAPE);
    b.close();
    b.open(Role::Button);
    b.style(look::GOAL_BTN);
    b.style(look::GOAL_PRIMARY);
    b.name(&w.say(send, &[]));
    b.bind(bindings.insert(PageAct::HandArea(Act::Send)));
    if open.busy() {
        b.state(State::Disabled);
    }
    b.close();
    b.close();
}
