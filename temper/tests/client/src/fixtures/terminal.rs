//! Terminal roots beside the browser reading root (client/testing.md, joint-world stories).
use super::{PARTY, chat_page, chats_page, phrasebooks, view_bounds, world_with};
use crate::{
    app::{Root, Temper, View},
    front::Joint,
};
use jig_client_kit::harness::{ClientId, World};
use temper_client_domain::Limits;

#[must_use]
pub fn limits() -> Limits {
    temper_client_shell_terminal::limits::native().client
}

/// The real loop's `open_reading`: native bounds with a 2,048-byte window.
#[must_use]
pub fn reading_limits() -> Limits {
    let mut limits = limits();
    limits.client.domain.text = 256;
    limits.client.domain.long_text = 1024;
    limits.client.domain.turn_text = 1024;
    limits.window_bytes = Some(super::READING_BYTES);
    limits
}

#[must_use]
pub fn world(seed: u64) -> World<Temper, Joint> {
    world_with(seed, Joint::with_whole_turns(seed, &limits()))
}

#[must_use]
pub fn reading_world(seed: u64) -> World<Temper, Joint> {
    world_with(seed, Joint::with_reading_pages(seed, &reading_limits()))
}

#[must_use]
pub fn open_chats_world(seed: u64) -> World<Temper, Joint> {
    world_with(seed, Joint::open_chats(seed, limits()))
}

pub fn chats_client(world: &mut World<Temper, Joint>) -> ClientId {
    let limits = limits();
    world.front().next_client_limits(limits);
    world.client_with(PARTY, chats_page(), limits, root)
}

pub fn client(world: &mut World<Temper, Joint>, task: u64) -> ClientId {
    let limits = limits();
    world.front().next_client_limits(limits);
    world.client_with(PARTY, chat_page(task), limits, root)
}

pub fn reading_client(world: &mut World<Temper, Joint>, task: u64) -> ClientId {
    let limits = reading_limits();
    world.front().next_client_limits(limits);
    world.client_with(PARTY, chat_page(task), limits, reading_root)
}

fn root(seed: u64) -> (Root, View) {
    root_with(seed, limits())
}

fn reading_root(seed: u64) -> (Root, View) {
    root_with(seed, reading_limits())
}

fn root_with(seed: u64, limits: Limits) -> (Root, View) {
    let words = phrasebooks(&limits);
    let bounds = view_bounds(&limits, &words);
    (
        Root::Person(Box::new(temper_client_domain::unlinked(&limits, seed))),
        View::Person(Box::new(temper_client_view::View::new(&bounds, &limits, words, super::FORGE))),
    )
}
