//! Views printed as terminal text (temper/docs/design/client/tui/pages/README.md).
use super::text;
use crate::{look, words::Words};
use alloc::boxed::Box;
use skein_ui::{Builder, Role, phrases::Hole};
use temper_client_domain::pages::chat::Shown;
use temper_client_protocol_wire::v1 as w;
use temper_client_view::words::Word;

// The actors' marks (temper/docs/design/client/tui/pages/chat.md, 1, a view).
const PERSON: &[u8] = "○".as_bytes();
const AGENT: &[u8] = "✦".as_bytes();
const CODE: &[u8] = "▪".as_bytes();
const BAR: &[u8] = "┃".as_bytes();

/// What the chat page holds for one view address, resolved at read time.
#[derive(Debug)]
pub(crate) enum Printed {
    Asking(Box<[u8]>),
    Failed(Box<[u8]>),
    Drawn(Box<w::View>),
}
pub(crate) fn read(address: &[u8], shown: Option<&Shown>) -> Printed {
    match shown {
        None | Some(Shown::Asking) => Printed::Asking(address.into()),
        Some(Shown::Failed) => Printed::Failed(address.into()),
        Some(Shown::Drawn(view)) => Printed::Drawn(view.clone()),
    }
}
pub(crate) fn draw(b: &mut Builder, printed: &Printed, words: &Words) {
    match printed {
        Printed::Asking(address) => text(
            b,
            Role::Paragraph,
            &words.view().say(Word::ViewAsking, &[Hole::Name(address.clone())]),
            Some(look::HINT),
        ),
        Printed::Failed(address) => {
            text(b, Role::Paragraph, &words.view().say(Word::ViewFailed, &[Hole::Name(address.clone())]), None);
        }
        Printed::Drawn(view) => match view.as_ref() {
            w::View::Way(way) => {
                b.open(Role::Group);
                b.style(look::SPACED);
                b.open(Role::Group);
                b.open(Role::Paragraph);
                text(b, Role::Text, way.name(), Some(look::NAME));
                if way.current() {
                    text(b, Role::Text, &words.view().say(Word::ViewNow, &[]), Some(look::HINT));
                }
                b.close();
                text(b, Role::Paragraph, way.line(), Some(look::HINT));
                b.close();
                b.open(Role::Group);
                print_path(b, way.path());
                b.close();
                b.close();
            }
            w::View::Path(path) => print_path(b, path),
            w::View::Proposal(proposal) => {
                b.open(Role::Group);
                b.style(look::SPACED);
                for (word, path) in [(Word::ViewBefore, proposal.before()), (Word::ViewAfter, proposal.after())] {
                    b.open(Role::Group);
                    text(b, Role::Paragraph, &words.view().say(word, &[]), Some(look::HEAD));
                    print_path(b, path);
                    b.close();
                }
                b.close();
            }
            w::View::Charter(charter) => card(b, charter.card()),
            w::View::Rule(rule) => card(b, rule.card()),
            w::View::Roles(roles) => {
                for role in roles.roles() {
                    card(b, role.card());
                }
            }
            w::View::NotDrawn(value) => text(b, Role::Paragraph, value.words(), None),
        },
    }
}
fn print_path(b: &mut Builder, path: &w::BranchPath) {
    for stop in path.stops() {
        let (mark, mark_style, word_style, indented) = match stop.actor() {
            w::Actor::Person => (PERSON, Some(look::SIGNAL), None, false),
            w::Actor::Agent => (AGENT, None, None, false),
            w::Actor::Code => (CODE, Some(look::HINT), Some(look::HINT), true),
            w::Actor::Bar => (BAR, Some(look::FAINT), None, true),
        };
        if indented {
            b.open(Role::Description);
            b.open(Role::Description);
        }
        b.open(Role::Paragraph);
        text(b, Role::Text, mark, mark_style);
        text(b, Role::Text, b" ", None);
        let words = if stop.over().is_empty() {
            skein_lib::bytes::copy_of(stop.words())
        } else {
            skein_lib::bytes::concat(&[stop.words(), b" (", stop.over(), b")"])
        };
        text(b, Role::Text, &words, word_style);
        b.close();
        if indented {
            b.close();
            b.close();
        }
    }
}
fn card(b: &mut Builder, card: &w::Card) {
    text(b, Role::Paragraph, card.name(), Some(look::NAME));
    text(b, Role::Paragraph, card.who(), Some(look::HINT));
    b.open(Role::Group);
    for fact in card.facts() {
        b.open(Role::Slot);
        text(b, Role::Term, fact.label(), None);
        text(b, Role::Description, fact.value(), None);
        b.close();
    }
    b.close();
}
