//! Views in committed replies and the chat document (temper/docs/design/client/pages/chat/conversation.md, 3).
use crate::{
    look, paths,
    words::{Word, Words},
};
use skein_lib::{List, bytes::copy_of};
use skein_ui::{Builder, Role, Run, phrases::Hole};
use temper_client_domain::{
    pages::chat::{Chat, Shown},
    views::Part,
};
use temper_client_protocol_wire::v1 as w;

#[expect(clippy::disallowed_methods, reason = "the protocol validated the text; the view splitter borrows it as str")]
pub(crate) fn parts(bytes: &[u8]) -> List<Part<'_>> {
    let text = core::str::from_utf8(bytes).expect("committed text is UTF-8");
    temper_client_domain::views::parts(text)
}

pub(crate) fn read(b: &mut Builder, bytes: &[u8], heading: u8, chat: Option<&Chat>, words: &Words) {
    for part in &parts(bytes) {
        match part {
            Part::Text(text) => skein_ui::markdown::read(text.as_bytes(), heading, b),
            Part::View(address) => {
                let shown = match chat {
                    Some(chat) => chat.view(address),
                    None => None,
                };
                block(b, address, shown, words);
            }
        }
    }
}

pub(crate) fn block(b: &mut Builder, address: &[u8], shown: Option<&Shown>, words: &Words) {
    match shown {
        Some(Shown::Asking) | None => {
            b.open(Role::Paragraph);
            b.style(look::VIEW_MUTED);
            b.text(Run::Plain, &words.say(Word::ViewAsking, &[Hole::Name(copy_of(address))]));
            b.close();
        }
        Some(Shown::Failed) => failed(b, address, words),
        Some(Shown::Drawn(view)) => drawn(b, address, view, words),
    }
}

fn failed(b: &mut Builder, address: &[u8], words: &Words) {
    line(b, Role::Paragraph, &words.say(Word::ViewFailed, &[Hole::Name(copy_of(address))]));
}

fn drawn(b: &mut Builder, address: &[u8], view: &w::View, words: &Words) {
    match view {
        w::View::Way(way) => {
            b.open(Role::Group);
            b.name(way.name());
            b.style(look::VIEW);
            b.open(Role::Paragraph);
            b.text(Run::Strong, way.name());
            if way.current() {
                b.text(Run::Plain, &words.say(Word::ViewNow, &[]));
            }
            b.close();
            b.open(Role::Paragraph);
            b.style(look::VIEW_MUTED);
            b.text(Run::Plain, way.line());
            b.close();
            path(b, way.path(), &words.say(Word::ViewWay, &[]), address, words);
            b.close();
        }
        w::View::Path(value) => {
            let label = match value.repository() {
                Some(repository) => words.say(
                    Word::ViewRepositoryPath,
                    &[Hole::Name(copy_of(value.branch())), Hole::Name(repository.clone())],
                ),
                None => words.say(Word::ViewPath, &[Hole::Name(copy_of(value.branch()))]),
            };
            b.open(Role::Group);
            b.name(&label);
            b.style(look::VIEW);
            path(b, value, &label, address, words);
            b.close();
        }
        w::View::Proposal(proposal) => {
            for (word, value) in [(Word::ViewBefore, proposal.before()), (Word::ViewAfter, proposal.after())] {
                let label = words.say(word, &[]);
                b.open(Role::Group);
                b.name(&label);
                b.style(look::VIEW);
                line(b, Role::Heading { level: 3 }, &label);
                path(b, value, &label, address, words);
                b.close();
            }
        }
        w::View::Charter(charter) => card(b, charter.card()),
        w::View::Rule(rule) => card(b, rule.card()),
        w::View::Roles(roles) => {
            for role in roles.roles() {
                card(b, role.card());
            }
        }
        w::View::NotDrawn(value) => line(b, Role::Paragraph, value.words()),
    }
}

fn path(b: &mut Builder, path: &w::BranchPath, label: &[u8], address: &[u8], words: &Words) {
    let mut stops = List::with_capacity(paths::STOPS);
    for stop in path.stops() {
        let mark = match stop.actor() {
            w::Actor::Person => paths::Mark::Person,
            w::Actor::Agent => paths::Mark::Agent,
            w::Actor::Code => paths::Mark::Code,
            w::Actor::Bar => paths::Mark::Bar,
        };
        stops
            .push(paths::Stop { mark, words: stop.words(), needed: stop.needed(), over: stop.over() })
            .expect("wire bounds path stops");
    }
    if paths::path(b, label, stops.as_slice(), None).is_err() {
        failed(b, address, words);
    }
}

fn card(b: &mut Builder, card: &w::Card) {
    b.open(Role::Group);
    b.name(card.name());
    b.style(look::VIEW);
    b.style(look::VIEW_CARD);
    line(b, Role::Heading { level: 3 }, card.name());
    b.open(Role::Paragraph);
    b.style(look::VIEW_MUTED);
    b.text(Run::Plain, card.who());
    b.close();
    facts(b, card);
    b.close();
}

pub(crate) fn facts(b: &mut Builder, card: &w::Card) {
    b.open(Role::Group);
    for fact in card.facts() {
        b.open(Role::Slot);
        line(b, Role::Term, fact.label());
        line(b, Role::Description, fact.value());
        b.close();
    }
    b.close();
}

fn line(b: &mut Builder, role: Role, text: &[u8]) {
    b.open(role);
    b.name(text);
    b.text(Run::Plain, text);
    b.close();
}
