//! The first screen's bottom row, filled by tui-foot (client/README.md, 6.14).
use super::{Binds, text};
use crate::{
    Event, Local, Tui, look,
    words::{Word, Words},
};
use alloc::boxed::Box;
use jig_client::Link;
use skein_lib::Wall;
use skein_ui::{Binding, Builder, Modifier, Role, Shortcut, Style, phrases::Hole};
use temper_client_domain::Domain;
use temper_client_view::words::Word as Shared;

#[derive(Debug)]
pub(crate) struct Foot {
    pub signing_in: bool,
    pub state: Box<[u8]>,
    pub style: Style,
}
fn state(tui: &Tui, root: &Domain) -> (Shared, Style) {
    match root.client().link().state {
        Link::Live => {
            if root.pages().me(root.client()).is_some() {
                (Shared::Live, look::LIVE)
            } else {
                (Shared::Connecting, look::WAITING)
            }
        }
        Link::Starting => (Shared::Connecting, look::WAITING),
        Link::Offline => (Shared::Offline, look::DOWN),
        Link::OutOfDate => (Shared::OutOfDate, look::DOWN),
        Link::SignedOut => {
            if tui.local() == Local::Signing {
                (Shared::Connecting, look::WAITING)
            } else {
                (Shared::SignedOut, look::DOWN)
            }
        }
    }
}

pub(crate) fn read(root: &Domain, tui: &Tui, words: &Words, _wall: Wall) -> Foot {
    let (state, style) = state(tui, root);
    Foot {
        signing_in: root.client().link().state == Link::SignedOut && tui.local() != Local::Signing,
        state: words.say(Word::State, &[Hole::Name(words.view().say(state, &[]))]),
        style,
    }
}
pub(crate) fn draw(b: &mut Builder, part: &Foot, words: &Words, _binds: &mut Binds) -> Option<Event> {
    b.open(Role::ContentInfo);
    b.region(look::BOTTOM);
    if part.signing_in {
        text(b, Role::Paragraph, &words.say(Word::QuitKeys, &[]), Some(look::HINT));
    } else {
        b.open(Role::Button);
        b.name(&words.say(Word::Quit, &[]));
        b.bind(Binding { owner: look::OWNER, token: look::QUIT });
        b.shortcut(Shortcut { key: b'q', modifier: Modifier::Plain });
        b.style(look::HINT);
        b.close();
    }
    text(b, Role::Status, &part.state, Some(part.style));
    b.close();
    None
}
