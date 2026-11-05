//! The configuring chat before its first words (temper/docs/design/client/README.md, 3.5).
use crate::{
    frame::Bindings,
    look,
    words::{Shell, Word, Words},
};
use alloc::boxed::Box;
use skein_ui::{Builder, Role, Run, State, phrases::Hole};
use temper_client_domain::{Address, Domain, pages::Act};

pub(crate) fn main(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32, text_limit: u32) {
    if root.pages().me(root.client()).is_some()
        && super::project_name(root.pages().me(root.client()), project).is_none()
    {
        let mut times = crate::words::Times::new(skein_lib::Wall::EPOCH);
        super::home::main(b, root, w, bindings, project, &mut times, 0);
        return;
    }
    b.open(Role::Main);
    b.style(look::MAIN);
    b.style(look::SESSION);
    top(b, root, w, bindings, project, &w.say(Word::ConfigureTitle, &[]));
    b.close();
    b.open(Role::Region);
    b.name(&w.say_card(crate::cards::P::ConversationLog, &[]));
    b.style(look::CONVO);
    b.open(Role::Slot);
    b.style(look::THREAD);
    b.open(Role::Slot);
    b.style(look::STARTER);
    let name = match super::project_name(root.pages().me(root.client()), project) {
        Some(name) => name,
        None => w.say(Word::Project, &[Hole::Number(u64::from(project))]),
    };
    let question = w.say(Word::ConfigureQuestion, &[Hole::Name(name)]);
    text(b, Role::Heading { level: 2 }, &question, None);
    paragraph(b, &w.say(Word::ConfigureLede, &[]), look::STARTER_LEDE);
    b.open(Role::List);
    b.name(&w.say(Word::ConfigureSets, &[]));
    b.style(look::STARTER_SETS);
    for (term, line) in [
        (Word::ConfigureWorkTerm, Word::ConfigureWorkLine),
        (Word::ConfigureModelsTerm, Word::ConfigureModelsLine),
        (Word::ConfigureApprovalsTerm, Word::ConfigureApprovalsLine),
    ] {
        b.open(Role::Item);
        pair(b, w, term, line);
        b.close();
    }
    b.close();
    paragraph(b, &w.say(Word::ConfigureNote, &[]), look::STARTER_NOTE);
    b.open(Role::Region);
    let ways = w.say(Word::ConfigureWays, &[]);
    b.name(&ways);
    b.style(look::STARTER_WAYS);
    if w.shell == Shell::Browser {
        text(b, Role::Heading { level: 3 }, &ways, None);
    }
    b.open(Role::List);
    b.style(look::STARTERS);
    let pending = match root.pages().chats() {
        Some(chats) => chats.start.is_some(),
        None => false,
    };
    for (title, line, sent) in [
        (Word::ConfigureAloneTitle, Word::ConfigureAloneLine, Word::ConfigureAlone),
        (Word::ConfigureDesignTitle, Word::ConfigureDesignLine, Word::ConfigureDesign),
        (Word::ConfigureModelsTitle, Word::ConfigureModelsCardLine, Word::ConfigureModels),
    ] {
        b.open(Role::Item);
        b.open(Role::Button);
        pair(b, w, title, line);
        starter(b, w, bindings, sent, pending);
        b.close();
        b.close();
    }
    b.close();
    b.open(Role::Button);
    b.name(&w.say(Word::ConfigureUnsureTitle, &[]));
    b.style(look::STARTER_UNSURE);
    starter(b, w, bindings, Word::ConfigureUnsure, pending);
    b.close();
    b.close();
    b.close();
    b.close();
    composer(b, root, w, bindings, pending, text_limit);
    b.close();
    b.close();
}

fn paragraph(b: &mut Builder, words: &[u8], style: skein_ui::Style) {
    b.open(Role::Paragraph);
    b.style(style);
    b.text(Run::Plain, words);
    b.close();
}

fn pair(b: &mut Builder, w: &Words, title: Word, line: Word) {
    let title = w.say(title, &[]);
    let line = w.say(line, &[]);
    match w.shell {
        Shell::Browser => {
            b.text(Run::Plain, &title);
            b.text(Run::Plain, &line);
        }
        Shell::Terminal => {
            let words = w.say(Word::HomeAskLine, &[Hole::Name(title), Hole::Name(line)]);
            b.text(Run::Plain, &words);
        }
    }
}

fn starter(b: &mut Builder, w: &Words, bindings: &mut Bindings, sent: Word, pending: bool) {
    if pending {
        b.state(State::Disabled);
    }
    b.bind(bindings.insert(Act::Starter { words: w.say(sent, &[]) }));
}

/// Opens the configuring layout's top bar; its caller closes it (client/README.md, 3.5).
pub(crate) fn top(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, project: u32, heading: &[u8]) {
    // Banner is a page landmark in skein-ui; this bar belongs inside the main.
    b.open(Role::Slot);
    b.style(look::SESSION_TOP);
    b.open(Role::Navigation);
    b.name(&w.say(Word::ConfigureWhere, &[]));
    b.style(look::CRUMBS);
    link(b, bindings, &w.say(Word::RailSettings, &[]), Address::Settings { project }, Some(look::BACK));
    b.open(Role::Slot);
    b.style(look::TRAIL);
    let name = match super::project_name(root.pages().me(root.client()), project) {
        Some(name) => name,
        None => w.say(Word::Project, &[Hole::Number(u64::from(project))]),
    };
    link(b, bindings, &name, Address::Home { project }, None);
    link(b, bindings, &w.say(Word::RailSettings, &[]), Address::Settings { project }, None);
    b.close();
    text(b, Role::Heading { level: 1 }, heading, None);
    b.close();
    let account = super::accounts::first_usable_kind(root, w);
    let mut model = None;
    if let Some(me) = root.pages().me(root.client()) {
        for row in &me.projects {
            if row.project == project
                && let Some(first) = row.models.get(0)
            {
                model = Some(w.model_name(first.name.bytes()));
            }
        }
    }
    let line = match (&account, &model) {
        (Some(account), Some(model)) => {
            crate::accounts::words::join(&[account, &w.say(Word::ToolSeparator, &[]), model])
        }
        (Some(account), None) => account.clone(),
        (None, Some(model)) => model.clone(),
        (None, None) => Box::default(),
    };
    if !line.is_empty() {
        text(b, Role::Paragraph, &line, Some(look::BYLINE));
    }
}

fn composer(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, pending: bool, text_limit: u32) {
    let Some(chats) = root.pages().chats() else { return };
    foot(b, &w.say(Word::ConfigureTitle, &[]));
    b.open(Role::TextArea);
    let name = w.say(Word::ConfigureWords, &[]);
    b.name(&name);
    b.placeholder(&name);
    b.value(&chats.words, temper_client_domain::pages::shown(root.pages()));
    if root.pages().switcher().renaming().is_none() {
        b.focus(0);
    }
    if pending {
        bindings.block_submissions();
    } else {
        b.state(State::SubmitsOnEnter);
    }
    b.bind(bindings.insert(Act::ChatWords { words: Box::default() }));
    b.close();
    b.open(Role::Button);
    b.name(&w.say(Word::Send, &[]));
    if pending || chats.words.is_empty() {
        b.state(State::Disabled);
    }
    b.bind(bindings.insert(Act::StartChat));
    b.close();
    hint(b, w);
    if chats.refusal.is_some() {
        w.refusal(b, &jig_client::contract::Refusal::NotOffered, text_limit);
    }
    if let Some(outcome) = &chats.outcome
        && !super::chats::accepted(outcome)
    {
        super::chats::outcome_text(b, w, outcome, text_limit);
    }
    b.close();
}

/// Opens the shared composer form and its entry; the caller adds controls and closes it.
pub(crate) fn foot(b: &mut Builder, name: &[u8]) {
    b.open(Role::Form);
    b.name(name);
    b.style(look::SESSION_FOOT);
    b.open(Role::Slot);
    b.style(look::SESSION_COMPOSER);
}
/// Closes the entry and follows it with the shared keyboard hint.
pub(crate) fn hint(b: &mut Builder, w: &Words) {
    b.close();
    text(b, Role::Paragraph, &w.say(Word::ChatSendHint, &[]), Some(look::SESSION_HINT));
}

fn text(b: &mut Builder, role: Role, words: &[u8], style: Option<skein_ui::Style>) {
    b.open(role);
    if let Some(style) = style {
        b.style(style);
    }
    b.name(words);
    b.text(Run::Plain, words);
    b.close();
}
fn link(b: &mut Builder, bindings: &mut Bindings, words: &[u8], address: Address, style: Option<skein_ui::Style>) {
    crate::link(b, words, bindings.insert(Act::Go { address }));
    if let Some(style) = style {
        b.style(style);
    }
    b.text(Run::Plain, words);
    b.close();
}
