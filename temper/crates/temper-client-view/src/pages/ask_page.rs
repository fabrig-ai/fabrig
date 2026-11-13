//! Ask temper before its first words (temper/docs/design/client/pages/README.md).
use crate::{
    frame::Bindings,
    look,
    words::{Word, Words},
};
use alloc::boxed::Box;
use skein_ui::{Builder, Role, Run, State, phrases::Hole};
use temper_client_domain::forge::stock_ways::{State as WaysState, StockWays};
use temper_client_domain::{Domain, pages::Act};

pub(crate) fn main(
    b: &mut Builder,
    root: &Domain,
    w: &Words,
    bindings: &mut Bindings,
    accounts: &mut crate::accounts::Bindings,
    project: u32,
    _text_limit: u32,
) {
    if root.pages().me(root.client()).is_some()
        && super::project_name(root.pages().me(root.client()), project).is_none()
    {
        let mut times = crate::words::Times::new(skein_lib::Wall::EPOCH);
        super::home::main(b, root, w, bindings, accounts, project, &mut times, 0, &crate::forge_sign_in::Forges::new());
        return;
    }
    let terminal = super::terminal::shown(root, w);
    b.open(Role::Main);
    if terminal {
        super::terminal::style(b, root.pages().look());
    }
    b.style(look::MAIN);
    b.style(look::SESSION);
    if terminal {
        let heading = w.say(Word::AskTitle, &[]);
        super::terminal::top(
            b,
            root,
            w,
            bindings,
            super::terminal::Top {
                project: Some(project),
                heading: &heading,
                name: &heading,
                hint: None,
                number: None,
                draft: super::terminal::Draft::None,
            },
        );
    } else {
        super::session::top(
            b,
            root,
            w,
            bindings,
            Some(project),
            super::session::Path::Home,
            super::session::Heading::Words(&w.say(Word::AskTitle, &[])),
        );
        super::session::right(b);
        super::session::to_terminal(b, w, bindings);
        b.close();
        b.close();
    }
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
    b.open(Role::Paragraph);
    b.style(look::STARTER_LEDE);
    b.style(look::WAYS_LEDE);
    b.text(Run::Plain, &w.say(Word::WaysLede, &[Hole::Name(name)]));
    b.close();
    b.open(Role::Region);
    b.name(&w.say(Word::WaysTitle, &[]));
    b.style(look::STARTER_WAYS);
    b.style(look::WAYS);
    if let Some(ways) = root.pages().stock_ways() {
        opening(b, ways, w, bindings, project);
    }
    b.close();
    b.close();
    if terminal {
        super::terminal::notes(b, root, w);
    }
    b.close();
    let pending = match root.pages().chats() {
        Some(chats) => chats.start.is_some(),
        None => false,
    };
    composer(b, root, w, bindings, pending);
    if !terminal {
        crate::link(b, &w.say(Word::TellTitle, &[]), bindings.insert(Act::Go { address: temper_client_domain::Address::Tell { project } }));
        b.style(look::TELL_LINK);
        b.text(Run::Plain, &w.say(Word::TellTitle, &[]));
        b.close();
    }
    b.close();
    b.close();
}

fn paragraph(b: &mut Builder, words: &[u8], style: skein_ui::Style) {
    b.open(Role::Paragraph);
    b.style(style);
    b.text(Run::Plain, words);
    b.close();
}

fn opening(b: &mut Builder, ways: &StockWays, w: &Words, bindings: &mut Bindings, project: u32) {
    match ways.state() {
        WaysState::Loading => return,
        WaysState::Ended => {
            paragraph(b, &w.say(Word::WaysEnded, &[]), look::WAYS_SAYS);
            return;
        }
        WaysState::Shown(_) => {}
    }
    super::ways::tabs(b, ways, w, bindings, project);
    let tabs = ways.tabs();
    if let Some(index) = ways.shown()
        && let Some(tab) = tabs.get(index)
    {
        b.open(Role::Group);
        b.name(&tab.name);
        b.style(look::WAYS_PANEL);
        if !tab.line.is_empty() {
            paragraph(b, &tab.line, look::WAYS_SAYS);
        }
        super::ways::drawing(b, &tab.path, ways.open(), w, bindings, project);
        if let Some(open) = ways.open()
            && let Some(stop) = tab.path.stops().get(open)
        {
            super::ways::card(b, stop, w, bindings, project);
        }
        foot(b, ways, index, w, bindings, project);
        super::ways::refused(b, ways, w);
        b.close();
    }
}

fn foot(b: &mut Builder, ways: &StockWays, index: u32, w: &Words, bindings: &mut Bindings, project: u32) {
    b.open(Role::Slot);
    b.style(look::WAYS_FOOT);
    super::ways::legend(b, w);
    super::ways::act(b, ways, index, w, bindings, project);
    b.close();
}

fn readiness(root: &Domain) -> (bool, u64) {
    let Some(me) = root.pages().me(root.client()) else { return (false, 0) };
    let Some(project) = root.pages().project() else { return (false, 0) };
    let Some(chats) = root.pages().chats() else { return (false, 0) };
    (
        jig_client::shapes::template(me, project, temper_client_domain::pages::chats::ASKING_TEMPLATE).is_some(),
        temper_client_domain::pages::composer::over(&chats.words, chats.beyond, me.words_bytes),
    )
}

fn composer(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, pending: bool) {
    let Some(chats) = root.pages().chats() else { return };
    let terminal = super::terminal::shown(root, w);
    let (available, over) = readiness(root);
    super::session::foot(b, &w.say(Word::AskTitle, &[]));
    if terminal {
        super::terminal::prompt(b, w);
    }
    b.open(Role::TextArea);
    let name = w.say(Word::AskWords, &[]);
    b.name(&name);
    b.placeholder(&name);
    if !available {
        b.state(State::Disabled);
    }
    b.value(&chats.words, temper_client_domain::pages::shown(root.pages()));
    if root.pages().switcher().renaming().is_none() {
        b.focus(if terminal { temper_client_domain::pages::shown(root.pages()) } else { 0 });
    }
    if !available || over > 0 || pending && (!terminal || !super::terminal::commands::typing(root)) {
        bindings.block_submissions();
    } else {
        b.state(State::SubmitsOnEnter);
    }
    b.bind(bindings.insert(Act::ChatWords { words: Box::default() }));
    b.close();
    if !terminal {
        b.open(Role::Button);
        b.name(&w.say(Word::Send, &[]));
        if !available || pending || over > 0 || chats.words.is_empty() {
            b.state(State::Disabled);
        }
        b.bind(bindings.insert(Act::StartChat));
        b.close();
    }
    if terminal {
        b.close();
        super::terminal::under(b, root, None, w, bindings, super::terminal::Hint::Commands);
    } else {
        super::session::foot_row(b, root, None, w, bindings, Word::ChatSendHint);
    }
    if !available {
        paragraph(b, &w.say(Word::AskUnavailable, &[]), look::STATUS_QUIET);
    }
    super::session::over(b, w, over);
    super::session::refusals(b, root, w);
    b.close();
}
