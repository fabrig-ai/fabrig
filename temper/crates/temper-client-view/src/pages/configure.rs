//! The configuring chat before its first words (temper/docs/design/client/README.md, 3.5).
use crate::{
    frame::Bindings,
    look,
    words::{Shell, Word, Words},
};
use alloc::boxed::Box;
use skein_ui::{Builder, Role, Run, State, phrases::Hole};
use temper_client_domain::{Domain, pages::Act};

#[expect(clippy::too_many_lines, reason = "one tree keeps the configuring opening, composer and reference in order")]
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
    let reference = reference(root);
    if let Some(mode) = reference {
        b.style(super::document::style(mode));
    }
    super::session::top(
        b,
        root,
        w,
        bindings,
        Some(project),
        super::session::Path::Settings,
        super::session::Heading::Words(&w.say(Word::ConfigureTitle, &[])),
    );
    super::session::right(b);
    byline(b, root, w, project);
    if let Some(mode) = reference {
        super::document::modes(b, Word::ReferenceGroup, mode, w, bindings);
    }
    b.close();
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
    let question = w.say(Word::ConfigureQuestion, &[Hole::Name(name.clone())]);
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
    let pending = match root.pages().chats() {
        Some(chats) => chats.start.is_some(),
        None => false,
    };
    b.open(Role::Slot);
    b.style(look::STARTER_HEAD);
    if w.shell == Shell::Browser {
        text(b, Role::Heading { level: 3 }, &ways, None);
    }
    b.open(Role::Button);
    b.name(&w.say(Word::ConfigureUnsureTitle, &[]));
    b.style(look::STARTER_UNSURE);
    starter(b, w, bindings, Word::ConfigureUnsure, &[], pending);
    b.close();
    b.close();
    b.open(Role::List);
    b.style(look::STARTERS);
    for (title, line, sent) in [
        (Word::ConfigureAloneTitle, Word::ConfigureAloneLine, Word::ConfigureAlone),
        (Word::ConfigureDesignTitle, Word::ConfigureDesignLine, Word::ConfigureDesign),
        (Word::ConfigureModelsTitle, Word::ConfigureModelsCardLine, Word::ConfigureModels),
    ] {
        b.open(Role::Item);
        b.open(Role::Button);
        pair(b, w, title, line);
        starter(b, w, bindings, sent, &[], pending);
        b.close();
        b.close();
    }
    b.close();
    own_way(b, w, bindings, name, pending);
    b.close();
    b.close();
    b.close();
    composer(b, root, w, bindings, pending, text_limit);
    b.close();
    if let Some(mode) = reference
        && mode != temper_client_domain::pages::chat::document::Mode::Chat
    {
        super::reference::pane(b, w);
    }
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

fn starter(b: &mut Builder, w: &Words, bindings: &mut Bindings, sent: Word, holes: &[Hole], pending: bool) {
    let words = w.say(sent, holes);
    if pending {
        b.state(State::Disabled);
    }
    b.bind(bindings.insert(Act::Starter { words }));
}

fn own_way(b: &mut Builder, w: &Words, bindings: &mut Bindings, name: Box<[u8]>, pending: bool) {
    b.open(Role::Group);
    b.name(&w.say(Word::ConfigureOwn, &[]));
    b.style(look::STARTER_OWN);
    b.open(Role::Slot);
    b.style(look::STARTER_OWN_TEXT);
    paragraph(b, &w.say(Word::ConfigureOwnTitle, &[]), look::STARTER_OWN_TITLE);
    b.open(Role::List);
    b.name(&w.say(Word::ConfigureOwnSteps, &[]));
    b.style(look::STARTER_OWN_STEPS);
    for step in [
        Word::ConfigureOwnDraft,
        Word::ConfigureOwnCut,
        Word::ConfigureOwnAgents,
        Word::ConfigureOwnBranch,
        Word::ConfigureOwnMain,
    ] {
        b.open(Role::Item);
        b.text(Run::Plain, &w.say(step, &[]));
        b.close();
    }
    b.close();
    b.close();
    b.open(Role::Slot);
    b.style(look::STARTER_OWN_ACTS);
    b.open(Role::Button);
    b.name(&w.say(Word::ConfigureOwnSee, &[]));
    b.bind(bindings.insert(Act::ShowReference));
    b.close();
    b.open(Role::Button);
    b.name(&w.say(Word::ConfigureOwnStart, &[]));
    starter(b, w, bindings, Word::ConfigureOwnSends, &[Hole::Name(name)], pending);
    b.close();
    b.close();
    b.close();
}

pub(crate) fn byline(b: &mut Builder, root: &Domain, w: &Words, project: u32) {
    let task = match root.pages().chat() {
        Some(chat) => chat.task(root.client()),
        None => None,
    };
    let line = crate::choice::byline_words(root, task, project, w);
    if !line.is_empty() {
        let line = match root.pages().chat() {
            Some(chat) => w.say(Word::BylineTask, &[Hole::Number(chat.task), Hole::Name(line)]),
            None => line,
        };
        text(b, Role::Paragraph, &line, Some(look::BYLINE));
    }
}

fn composer(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, pending: bool, text_limit: u32) {
    let Some(chats) = root.pages().chats() else { return };
    super::session::foot(b, &w.say(Word::ConfigureTitle, &[]));
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
    super::session::hint(b, w, chats.reference.is_some());
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

fn text(b: &mut Builder, role: Role, words: &[u8], style: Option<skein_ui::Style>) {
    b.open(role);
    if let Some(style) = style {
        b.style(style);
    }
    b.name(words);
    b.text(Run::Plain, words);
    b.close();
}

pub(crate) fn reference(root: &Domain) -> Option<temper_client_domain::pages::chat::document::Mode> {
    match root.pages().chats() {
        Some(chats) => chats.reference,
        None => None,
    }
}
