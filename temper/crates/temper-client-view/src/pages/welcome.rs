//! First-run pages (temper/docs/design/client/README.md, 3.5).
use crate::{
    frame::Bindings,
    look,
    words::{Word, Words},
};
use alloc::boxed::Box;
use skein_ui::{Builder, Role, Run, State, phrases::Hole};
use temper_client_domain::{
    Address, Domain, Step,
    forge::{
        connect, deployment,
        forges::{Connection, FirstState},
    },
    pages::Act,
};

fn text(b: &mut Builder, w: &Words, role: Role, word: Word) {
    b.open(role);
    if word == Word::WelcomeTitle {
        b.style(look::WELCOME_INTRO);
    }
    b.name(&w.say(word, &[]));
    w.text(b, word, &[]);
    b.close();
}
fn paragraph(b: &mut Builder, w: &Words, word: Word, style: skein_ui::Style) {
    b.open(Role::Paragraph);
    b.style(style);
    b.name(&w.say(word, &[]));
    w.text(b, word, &[]);
    b.close();
}
fn lead(b: &mut Builder, w: &Words) {
    paragraph(b, w, Word::WelcomeLead, look::WELCOME_TAGLINE);
    let describe = w.say(Word::WelcomeDescribe, &[]);
    let what = w.say(Word::WelcomeWhat, &[]);
    let want_and = w.say(Word::WelcomeWantAnd, &[]);
    let how = w.say(Word::WelcomeHow, &[]);
    let want_done = w.say(Word::WelcomeWantDone, &[]);
    b.open(Role::Paragraph);
    b.style(look::WELCOME_LEAD);
    b.name(&crate::accounts::words::join(&[&describe, &what, &want_and, &how, &want_done]));
    b.open(Role::Text);
    b.run(Run::Plain, &describe);
    b.run(Run::Strong, &what);
    b.run(Run::Plain, &want_and);
    b.run(Run::Strong, &how);
    b.run(Run::Plain, &want_done);
    b.close();
    b.close();
}
fn control(b: &mut Builder, w: &Words, bindings: &mut Bindings, word: Word, act: Act) {
    let name = w.say(word, &[]);
    let binding = bindings.insert(act.clone());
    if let Act::Go { .. } = act {
        crate::link(b, &name, binding);
        b.text(Run::Plain, &name);
    } else {
        b.open(Role::Button);
        b.name(&name);
        b.bind(binding);
    }
    if word == Word::WelcomeSetup || word == Word::WelcomeGoOn {
        b.style(look::FLOW_PRIMARY);
    } else if word == Word::WelcomePlayground {
        b.style(look::FLOW_SECONDARY);
    } else if word != Word::WelcomePlaygroundInstead {
        b.style(look::FLOW_QUIET);
    }
    b.close();
}
fn go(b: &mut Builder, w: &Words, bindings: &mut Bindings, word: Word, step: Step) {
    control(b, w, bindings, word, Act::Go { address: Address::Welcome { step } });
}
/// The welcome's steps, in its frame's top bar (3.5).
pub(crate) fn steps(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, current: Step) {
    use temper_client_domain::pages::welcome;
    let marks = welcome::step_marks(current, welcome::forge_kept(root), welcome::model_usable(root) == Some(true));
    let [welcome, forge, model, ready] = marks;
    b.open(Role::List);
    b.name(&w.say(Word::WelcomeSteps, &[]));
    b.style(look::FLOW_STEPS);
    for (word, step, mark) in [
        (Word::WelcomeStep, Step::WelcomeAsked, welcome),
        (Word::WelcomeForgeStep, Step::Forge { forge: None }, forge),
        (Word::WelcomeModelStep, Step::Model { way: None }, model),
        (Word::WelcomeReadyStep, Step::Ready, ready),
    ] {
        b.open(Role::Item);
        b.style(look::FLOW_STEP);
        if mark.current {
            b.state(State::Current);
        }
        if mark.ticked {
            b.style(look::FLOW_DONE);
        }
        let name = w.say(word, &[]);
        crate::link(b, &name, bindings.insert(Act::Go { address: Address::Welcome { step } }));
        if mark.ticked && w.shell == crate::words::Shell::Terminal {
            b.text(Run::Plain, "✓ ".as_bytes());
        }
        b.text(Run::Plain, &name);
        b.close();
        b.close();
    }
    b.close();
}
fn card(b: &mut Builder, w: &Words, title: Word) {
    b.open(Role::Region);
    b.style(look::WELCOME_CARD);
    b.name(&w.say(title, &[]));
    text(b, w, Role::Heading { level: 2 }, title);
}
fn setup_description(b: &mut Builder, w: &Words) {
    b.open(Role::Slot);
    b.style(look::WELCOME_SETUP_DESCRIPTION);
    b.open(Role::Paragraph);
    w.text(b, Word::WelcomeSetupDescription, &[]);
    b.close();
    b.close();
}

#[expect(clippy::too_many_arguments, reason = "welcome composes the forge and accounts flows with their bindings")]
pub(crate) fn main(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    accounts: &mut crate::accounts::Bindings,
    forge: &mut crate::forge::bindings::Bindings,
    step: Step,
    text_per_node: u32,
    sign_in: &crate::forge_sign_in::Forges,
) {
    b.open(Role::Main);
    b.style(look::MAIN);
    b.style(look::WELCOME);
    let Some(deployment::Answer { welcome: true, .. }) = root.forge().deployment() else {
        b.close();
        return;
    };
    let playground = temper_client_domain::pages::welcome::playground_offered(root) == Some(true);
    match step {
        Step::Welcome | Step::WelcomeAsked => {
            text(b, w, Role::Heading { level: 1 }, Word::WelcomeTitle);
            if let Some(page) = root.pages().welcome()
                && page.decided
            {
                lead(b, w);
                b.open(Role::Slot);
                b.style(look::WELCOME_CHOICES);
                card(b, w, Word::WelcomeSetup);
                setup_description(b, w);
                paragraph(b, w, Word::WelcomeSetupTime, look::WELCOME_MUTED);
                if temper_client_domain::pages::welcome::forge_kept(root) {
                    go(b, w, bindings, Word::WelcomeGoOn, temper_client_domain::pages::welcome::go_on(root));
                } else {
                    go(b, w, bindings, Word::WelcomeSetup, Step::Forge { forge: None });
                }
                b.close();
                if playground {
                    card(b, w, Word::WelcomePlayground);
                    text(b, w, Role::Paragraph, Word::WelcomePlaygroundDescription);
                    paragraph(b, w, Word::WelcomePlaygroundCost, look::WELCOME_MUTED);
                    control(b, w, bindings, Word::WelcomePlayground, Act::Playground);
                    b.close();
                }
                b.close();
                if playground {
                    paragraph(b, w, Word::WelcomeAfter, look::WELCOME_AFTER);
                }
            }
        }
        Step::Forge { forge: forge_choice } => {
            text(b, w, Role::Heading { level: 1 }, Word::WelcomeForgeTitle);
            let playground = connect::playground(root.forge());
            let unfinished =
                forge_choice.is_none() && connect::unfinished(connect::records(root.forges()), playground).is_some();
            let kept = match crate::forge::connect::personal(root) {
                Some(row) => !unfinished && connect::status_shown(row, playground),
                None => false,
            };
            paragraph(b, w, if kept { Word::WelcomeForgeLeadKept } else { Word::WelcomeForgeLead }, look::WELCOME_LEAD);
            crate::forge::connect::welcome(b, root, w, bindings, forge, forge_choice, sign_in);
        }
        Step::Model { .. } => {
            super::welcome_model::step(b, root, w, accounts, text_per_node);
            if let Some(row) = crate::forge::connect::personal(root) {
                b.open(Role::Complementary);
                b.style(look::WELCOME_STATUS);
                b.name(&w.say(Word::WelcomeForgeStep, &[]));
                crate::forge::connect::welcome_line(b, root, w, row);
                b.close();
            }
        }
        Step::Ready => {
            ready(b, root, w, bindings, accounts, forge, sign_in);
        }
    }
    if let Step::Forge { .. } | Step::Model { .. } | Step::Ready = step {
        b.open(Role::Slot);
        b.style(look::WELCOME_FOOT);
        let back = match step {
            Step::Model { way: Some(_) } | Step::Ready => Step::Model { way: None },
            Step::Forge { forge: Some(_) } | Step::Model { way: None } => Step::Forge { forge: None },
            Step::Welcome | Step::WelcomeAsked | Step::Forge { forge: None } => Step::Welcome,
        };
        go(b, w, bindings, Word::WelcomeBack, back);
        if playground {
            control(b, w, bindings, Word::WelcomePlaygroundInstead, Act::Playground);
        }
        b.close();
    }
    b.close();
}

/// A checklist line: its name, strong, then what it says of it, as the
/// greenfield mockup draws "Forge Forgejo on this machine · connected as
/// temper" (3.5).
fn labelled(b: &mut Builder, w: &Words, label: Word, detail: &[u8]) {
    let label = w.say(label, &[]);
    b.open(Role::Paragraph);
    b.name(&crate::accounts::words::join(&[&label, b" ", detail]));
    b.open(Role::Text);
    b.run(Run::Strong, &label);
    b.run(Run::Plain, b" ");
    b.run(Run::Plain, detail);
    b.close();
    b.close();
}
fn model_line(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    accounts: &mut crate::accounts::Bindings,
) {
    if let Some(usable) = temper_client_domain::pages::welcome::model_usable(root) {
        b.open(Role::Item);
        if usable {
            b.style(look::FLOW_DONE);
            // The account, "ChatGPT plan · owner@example.com"; the model and
            // its effort are chosen elsewhere.
            let account = match super::accounts::first_usable_short(root, w) {
                Some(account) => account,
                None => crate::choice::new_chat_model(root, w),
            };
            labelled(b, w, Word::WelcomeModelStep, &account);
        } else if let Some(provider) = super::accounts::own_signing_name(root, w) {
            b.style(look::WELCOME_CURRENT);
            labelled(b, w, Word::WelcomeModelStep, &w.say(Word::WelcomeModelSigningIn, &[Hole::Name(provider)]));
        } else {
            let label = w.say(Word::WelcomeModelStep, &[]);
            let none = w.say(Word::WelcomeNoModel, &[]);
            b.open(Role::Slot);
            b.style(look::ACCOUNT_LINE);
            b.name(&crate::accounts::words::join(&[&label, b" ", &none]));
            b.open(Role::Text);
            b.run(Run::Strong, &label);
            b.run(Run::Plain, b" ");
            b.run(Run::Plain, &none);
            b.run(Run::Plain, b" \xc2\xb7 ");
            b.close();
            go(b, w, bindings, Word::WelcomeModelTitle, Step::Model { way: None });
            b.close();
            b.open(Role::Slot);
            b.style(look::WELCOME_END);
            super::connect_model::welcome_end(b, root, w, accounts);
            b.close();
        }
        if usable {
            text(b, w, Role::Status, Word::FlowDone);
        }
        b.close();
    }
}
fn connected(root: &Domain) -> bool {
    if let temper_client_domain::forge::forges::State::Rows(rows) = root.forges() {
        for row in rows {
            if temper_client_domain::pages::welcome::forge_connected(row) {
                return true;
            }
        }
    }
    false
}
fn ready(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    accounts: &mut crate::accounts::Bindings,
    forge: &mut crate::forge::bindings::Bindings,
    sign_in: &crate::forge_sign_in::Forges,
) {
    use crate::forge::connect;
    let done = connected(root) && temper_client_domain::pages::welcome::model_usable(root) == Some(true);
    if done {
        text(b, w, Role::Heading { level: 1 }, Word::WelcomeReadyStep);
        ready_lead(b, root, w);
    } else {
        text(b, w, Role::Heading { level: 1 }, Word::WelcomeReadyTitle);
        paragraph(b, w, Word::WelcomeDefaultWait, look::WELCOME_LEAD);
    }
    b.open(Role::List);
    b.style(look::FLOW_PROGRESS);
    b.style(look::WELCOME_CHECKLIST);
    b.open(Role::Item);
    let row = connect::personal(root);
    if let Some(row) = row {
        let underway = temper_client_domain::forge::connect::underway(row);
        if underway {
            b.style(look::WELCOME_CURRENT);
        }
        if temper_client_domain::pages::welcome::forge_connected(row) {
            if !underway {
                b.style(look::FLOW_DONE);
            }
            if let Connection::Connected { login, .. } = &row.connection {
                let login = w.say(Word::WelcomeForgeDoneDetail, &[Hole::Name(skein_lib::bytes::copy_of(login))]);
                let mut detail =
                    crate::accounts::words::join(&[&connect::name(w, row, root), " · ".as_bytes(), &login]);
                if let Some(first) = &row.first {
                    match first.state {
                        FirstState::Made { .. } => {
                            let repository = w.say(
                                Word::WelcomeForgeDoneRepository,
                                &[Hole::Name(temper_client_domain::forge::words::full_name(
                                    &first.org,
                                    &first.repository,
                                ))],
                            );
                            detail = crate::accounts::words::join(&[&detail, " · ".as_bytes(), &repository]);
                        }
                        FirstState::Making | FirstState::Failed { .. } => {}
                    }
                }
                labelled(b, w, Word::WelcomeForgeStep, &detail);
            }
            crate::forge::forges::open_link(b, w, row, sign_in, Some(look::FLOW_QUIET));
            text(b, w, Role::Status, Word::FlowDone);
        } else {
            let detail = connect::status_words(root, w, row);
            labelled(b, w, Word::WelcomeForgeStep, &detail);
            connect::each_step(b, root, w, row);
        }
    } else {
        let label = w.say(Word::WelcomeForgeStep, &[]);
        let none = w.say(Word::WelcomeNoForge, &[]);
        b.open(Role::Slot);
        b.style(look::ACCOUNT_LINE);
        b.name(&crate::accounts::words::join(&[&label, b" ", &none]));
        b.open(Role::Text);
        b.run(Run::Strong, &label);
        b.run(Run::Plain, b" ");
        b.run(Run::Plain, &none);
        b.run(Run::Plain, " · ".as_bytes());
        b.close();
        go(b, w, bindings, Word::WelcomeForgeTitle, Step::Forge { forge: None });
        b.close();
    }
    b.close();
    model_line(b, root, w, bindings, accounts);
    b.close();
    if let Some(row) = row {
        connect::setup_ending(b, root, w, forge, row);
    }
    first_project(b, root, w, bindings, done);
    if let Some(row) = row
        && temper_client_domain::forge::connect::underway(row)
        && !temper_client_domain::pages::welcome::forge_connected(row)
    {
        paragraph(b, w, Word::WelcomeForgeStillSettingUp, look::FLOW_FACTS);
    }
}
fn ready_lead(b: &mut Builder, root: &Domain, w: &Words) {
    let (word, hole) = if let Some(first) = root.pages().welcome_first() {
        let name = temper_client_domain::forge::words::full_name(&first.org, &first.name);
        (Word::WelcomeReadyRepositoryLead, Some(Hole::Name(name)))
    } else if let Some(copies) = root.pages().welcome_copies() {
        if copies.addresses.len() > 1 {
            (
                Word::WelcomeReadyCopiesLead,
                Some(Hole::Number(u64::try_from(copies.addresses.len()).expect("copy count fits"))),
            )
        } else if let Some(address) = copies.addresses.first()
            && let Some((org, name)) = temper_client_domain::forge::making::target(address)
        {
            (
                Word::WelcomeReadyRepositoryLead,
                Some(Hole::Name(temper_client_domain::forge::words::full_name(&org, &name))),
            )
        } else {
            (Word::WelcomeReadyLead, None)
        }
    } else {
        (Word::WelcomeReadyLead, None)
    };
    let mut holes = skein_lib::List::with_capacity(1);
    if let Some(hole) = hole {
        holes.push(hole).expect("one lead hole");
    }
    b.open(Role::Paragraph);
    b.style(look::WELCOME_LEAD);
    b.name(&w.say(word, holes.as_slice()));
    w.text(b, word, holes.as_slice());
    b.close();
}
/// Ready's form: the offered or written name, then Create, which makes
/// the project and opens it (3.5).
fn first_project(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, done: bool) {
    use crate::flow::{self, Control};
    let state = root.pages().default_project();
    let offered =
        temper_client_domain::pages::welcome::offered(root.pages().welcome_first(), root.pages().welcome_copies());
    let label = w.say(Word::WelcomeDefaultProject, &[]);
    b.open(Role::Form);
    b.style(look::WELCOME_PROJECT);
    b.name(&label);
    b.open(Role::TextField);
    b.name(&label);
    b.value(state.shown(&offered), temper_client_domain::pages::shown(root.pages()));
    if state.making.is_some() {
        b.state(State::Disabled);
    } else if done {
        b.state(State::SubmitsOnEnter);
    }
    if state.unnamed {
        b.state(State::Invalid);
    }
    b.bind(bindings.insert(Act::DefaultName { words: Box::default() }));
    b.close();
    if state.making.is_some() {
        let making = w.say(Word::WelcomeMakingDefault, &[Hole::Name(state.asked(&offered).into())]);
        flow::control(b, Control::PrimaryDisabled { name: &making });
    } else {
        let create = w.say(Word::WelcomeCreateProject, &[]);
        flow::control(
            b,
            if done {
                Control::PrimaryButton { name: &create, to: bindings.insert(Act::MakeDefault) }
            } else {
                Control::PrimaryDisabled { name: &create }
            },
        );
    }
    if state.unnamed {
        let holes = &[Hole::Name(offered)];
        b.open(Role::Alert);
        b.name(&w.say(Word::WelcomeDefaultUnnamed, holes));
        w.text(b, Word::WelcomeDefaultUnnamed, holes);
        b.close();
    }
    match &state.ended {
        Some(temper_client_domain::pages::welcome::Ended::Refused(refusal)) => {
            b.open(Role::Alert);
            w.text(b, Word::WelcomeDefaultRefused, &[]);
            w.refusal(b, refusal, w.name_bytes);
            b.close();
        }
        Some(temper_client_domain::pages::welcome::Ended::Unknown) => {
            text(b, w, Role::Alert, Word::WelcomeDefaultUnknown);
        }
        None => {}
    }
    paragraph(b, w, Word::WelcomeRenameLater, look::WELCOME_AFTER);
    b.close();
}
