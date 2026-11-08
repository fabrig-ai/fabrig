//! The first screen's chat stub, filled by tui-chat (client/README.md, 6.14).
use super::{Binds, launchpad};
use crate::{Tui, words::Words};
use skein_lib::Wall;
use skein_ui::Builder;
use temper_client_domain::Domain;

#[derive(Debug)]
pub(crate) struct Chat {
    pub first: launchpad::Launchpad,
}
pub(crate) fn read(root: &Domain, tui: &Tui, words: &Words, wall: Wall) -> Chat {
    Chat { first: launchpad::read(root, tui, words, wall) }
}
pub(crate) fn draw(b: &mut Builder, part: &Chat, words: &Words, binds: &mut Binds) {
    launchpad::draw(b, &part.first, words, binds);
}
