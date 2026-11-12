//! A project's stock ways, drawn on Ask temper's page and How work runs
//! (temper/docs/design/client/pages/README.md and temper/docs/design/client/pages/settings.md, 2 and temper/docs/design/client/pages/chat/conversation.md, 3).
use crate::{
    frame::Bindings,
    look, paths,
    words::{Word, Words},
};
use skein_lib::{List, bytes::copy_of};
use skein_ui::{Builder, Role, Run, State, phrases::Hole};
use temper_client_domain::forge::stock_ways::{StockWays, WayAct};
use temper_client_domain::{Address, pages::Act};
use temper_client_protocol_wire::v1 as wire;

/// The tabs control "Ways of working", a tab a way, "now" on the current one, and
/// the axis line under it.
pub(crate) fn tabs(b: &mut Builder, ways: &StockWays, w: &Words, bindings: &mut Bindings, project: u32) {
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
}

/// The path and its stop acts, with the open stop marked.
pub(crate) fn drawing(
    b: &mut Builder,
    path: &wire::BranchPath,
    open: Option<u32>,
    w: &Words,
    bindings: &mut Bindings,
    project: u32,
) {
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
        stops
            .push(paths::Stop { mark, words: stop.words(), needed: stop.needed(), over: stop.over() })
            .expect("wire bounds stops");
        acts.push(act).expect("one act per stop");
    }
    let pressed =
        paths::Pressed { drawing: bindings.insert(Act::WayStop { project, stop: 0 }), stops: acts.as_slice(), open };
    paths::path(b, &w.say(Word::WaysPath, &[]), stops.as_slice(), Some(&pressed)).expect("wire bounds path");
}

/// Who acts and the dotted line's authority boundary.
pub(crate) fn legend(b: &mut Builder, w: &Words) {
    b.open(Role::Paragraph);
    b.name(&w.say(Word::WaysLegend, &[]));
    b.style(look::WAYS_LEGEND);
    for (word, style) in [
        (Word::WaysAgent, look::WAYS_KIND_AGENT),
        (Word::WaysMaybe, look::WAYS_KIND_MAYBE),
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
}

/// The open stop's card: heading, `who`, "Open NAME ›" where the stop opens a
/// charter, a gate or a rule, and its facts.
pub(crate) fn card(b: &mut Builder, stop: &wire::Stop, w: &Words, bindings: &mut Bindings, project: u32) {
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

/// The shown way's act, "Use this" or "Keep it", where `ways.act(index)` gives one.
pub(crate) fn act(b: &mut Builder, ways: &StockWays, index: u32, w: &Words, bindings: &mut Bindings, project: u32) {
    if ways.act(index).is_some() {
        b.open(Role::Slot);
        b.style(look::WAYS_ACTS);
        act_button(b, ways, index, w, bindings, project);
        b.close();
    }
}

/// The way's button, within the calling page's acts row.
pub(crate) fn act_button(b: &mut Builder, ways: &StockWays, index: u32, w: &Words, bindings: &mut Bindings, project: u32) {
    if let Some(act) = ways.act(index) {
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
    }
}

/// The refusal of the last act, where there is one.
pub(crate) fn refused(b: &mut Builder, ways: &StockWays, w: &Words) {
    if let Some(refused) = ways.refused() {
        b.open(Role::Alert);
        b.style(look::WAYS_REFUSED);
        crate::cards::refusal(b, w, refused);
        b.close();
    }
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
