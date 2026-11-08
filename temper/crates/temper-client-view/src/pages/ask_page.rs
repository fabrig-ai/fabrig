//! Ask temper before its first words (temper/docs/design/client/README.md, 3.5).
use crate::{
    frame::Bindings,
    look, paths,
    words::{Word, Words},
};
use alloc::boxed::Box;
use skein_lib::{List, bytes::copy_of};
use skein_ui::{Builder, Role, Run, State, phrases::Hole};
use temper_client_domain::forge::stock_ways::{State as WaysState, StockWays, WayAct};
use temper_client_domain::{Address, Domain, pages::Act};
use temper_client_protocol_wire::v1 as wire;

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
    let tabs = ways.tabs();
    b.open(Role::Tabs);
    b.name(&w.say(Word::WaysTitle, &[]));
    b.style(look::WAYS_TABS);
    for (index, tab) in tabs.iter().enumerate() {
        let index = u32::try_from(index).expect("wire bounds tabs");
        b.open(Role::Tab);
        if ways.shown() == Some(index) {
            b.state(State::Selected);
        }
        b.bind(bindings.insert(Act::WayTab { project, tab: index }));
        b.text(Run::Plain, &tab.name);
        if tab.current {
            b.open(Role::Text);
            b.style(look::WAYS_NOW);
            b.text(Run::Plain, &w.say(Word::WaysNow, &[]));
            b.close();
        }
        b.close();
    }
    b.close();
    b.open(Role::Paragraph);
    b.name(&w.say(Word::WaysAxis, &[]));
    b.style(look::WAYS_AXIS);
    for word in [Word::WaysYou, Word::WaysAgents] {
        b.text(Run::Plain, &w.say(word, &[]));
    }
    b.close();
    if let Some(index) = ways.shown()
        && let Some(tab) = tabs.get(index)
    {
        b.open(Role::Group);
        b.name(&tab.name);
        b.style(look::WAYS_PANEL);
        if !tab.line.is_empty() {
            paragraph(b, &tab.line, look::WAYS_SAYS);
        }
        path(b, &tab.path, ways.open(), w, bindings, project);
        if let Some(open) = ways.open()
            && let Some(stop) = tab.path.stops().get(open)
        {
            card(b, stop, w, bindings, project);
        }
        foot(b, ways, index, w, bindings, project);
        if let Some(refused) = ways.refused() {
            b.open(Role::Alert);
            b.style(look::WAYS_REFUSED);
            crate::cards::refusal(b, w, refused);
            b.close();
        }
        b.close();
    }
}

fn path(b: &mut Builder, path: &wire::BranchPath, open: Option<u32>, w: &Words, bindings: &mut Bindings, project: u32) {
    let mut stops = List::with_capacity(paths::STOPS);
    let mut acts = List::with_capacity(paths::STOPS);
    for (index, stop) in path.stops().iter().enumerate() {
        let index = u32::try_from(index).expect("wire bounds stops");
        let (mark, act) = match stop.actor() {
            wire::Actor::Person => (paths::Mark::Person, Some(bindings.insert(Act::WayStop { project, stop: index }))),
            wire::Actor::Agent => (paths::Mark::Agent, Some(bindings.insert(Act::WayStop { project, stop: index }))),
            wire::Actor::Code => (paths::Mark::Code, Some(bindings.insert(Act::WayStop { project, stop: index }))),
            wire::Actor::Bar => (paths::Mark::Bar, None),
        };
        stops.push(paths::Stop { mark, words: stop.words() }).expect("wire bounds stops");
        acts.push(act).expect("one act per stop");
    }
    let pressed =
        paths::Pressed { drawing: bindings.insert(Act::WayStop { project, stop: 0 }), stops: acts.as_slice(), open };
    paths::path(b, &w.say(Word::WaysPath, &[]), stops.as_slice(), Some(&pressed)).expect("wire bounds path");
}

fn card(b: &mut Builder, stop: &wire::Stop, w: &Words, bindings: &mut Bindings, project: u32) {
    let mut heading = copy_of(stop.words());
    if let Some(first) = heading.first_mut() {
        first.make_ascii_uppercase();
    }
    let card = stop.card();
    b.open(Role::Region);
    b.name(&heading);
    b.style(look::WAYS_ZOOM);
    b.open(Role::Slot);
    b.style(look::WAYS_ZOOM_HEAD);
    text(b, Role::Heading { level: 3 }, &heading, None);
    b.text(Run::Plain, card.who());
    let address = match stop.opens() {
        wire::Opens::Charter(c) => Some(Address::Role { project, charter: c.charter() }),
        wire::Opens::Gate(g) => Some(Address::Role { project, charter: g.charter() }),
        wire::Opens::Rule(r) => Some(Address::Rule { project, kind: r.kind(), number: r.number() }),
        wire::Opens::Nothing | wire::Opens::Tier(_) | wire::Opens::Queue | wire::Opens::Turn => None,
    };
    if let Some(address) = address {
        let label = w.say(Word::WaysOpen, &[Hole::Name(copy_of(card.name()))]);
        crate::link(b, &label, bindings.insert(Act::Go { address }));
        b.text(Run::Plain, &label);
        b.close();
    }
    b.close();
    b.open(Role::Group);
    for fact in card.facts() {
        b.open(Role::Slot);
        text(b, Role::Term, fact.label(), None);
        text(b, Role::Description, fact.value(), None);
        b.close();
    }
    b.close();
    b.close();
}

fn foot(b: &mut Builder, ways: &StockWays, index: u32, w: &Words, bindings: &mut Bindings, project: u32) {
    b.open(Role::Slot);
    b.style(look::WAYS_FOOT);
    b.open(Role::Paragraph);
    b.name(&w.say(Word::WaysLegend, &[]));
    b.style(look::WAYS_LEGEND);
    for (word, style) in [
        (Word::WaysAgent, look::WAYS_KIND_AGENT),
        (Word::WaysPerson, look::WAYS_KIND_PERSON),
        (Word::WaysCode, look::WAYS_KIND_CODE),
        (Word::WaysLine, look::WAYS_KIND_LINE),
    ] {
        b.open(Role::Text);
        b.style(style);
        b.text(Run::Plain, &w.say(word, &[]));
        b.close();
    }
    b.close();
    if let Some(act) = ways.act(index) {
        b.open(Role::Slot);
        b.style(look::WAYS_ACTS);
        let (word, act) = match act {
            WayAct::Use => (Word::WaysUse, Act::UseWay { project, way: index }),
            WayAct::Keep => (Word::WaysKeep, Act::KeepWay { project }),
        };
        b.open(Role::Button);
        b.style(look::WAYS_USE);
        b.name(&w.say(word, &[]));
        b.bind(bindings.insert(act));
        if ways.sending() {
            b.state(State::Disabled);
        }
        b.close();
        b.close();
    }
    b.close();
}

fn composer(b: &mut Builder, root: &Domain, w: &Words, bindings: &mut Bindings, pending: bool) {
    let Some(chats) = root.pages().chats() else { return };
    let terminal = super::terminal::shown(root, w);
    let available = match (root.pages().me(root.client()), root.pages().project()) {
        (Some(me), Some(project)) => {
            jig_client::shapes::template(me, project, temper_client_domain::pages::chats::ASKING_TEMPLATE).is_some()
        }
        (Some(_), None) | (None, Some(_) | None) => false,
    };
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
    if !available || pending && (!terminal || !super::terminal::commands::typing(root)) {
        bindings.block_submissions();
    } else {
        b.state(State::SubmitsOnEnter);
    }
    b.bind(bindings.insert(Act::ChatWords { words: Box::default() }));
    b.close();
    if !terminal {
        b.open(Role::Button);
        b.name(&w.say(Word::Send, &[]));
        if !available || pending || chats.words.is_empty() {
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
    super::session::refusals(b, root, w);
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
