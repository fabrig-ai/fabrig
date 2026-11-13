//! Shared page facts (temper/docs/design/client/view-crates.md, 3.2).
pub(crate) mod accounts;
pub(crate) mod all_home;
pub(crate) mod approval;
pub(crate) mod ask;
pub(crate) mod change;
pub(crate) mod chat;
pub(crate) mod chat_line;
pub(crate) mod chats;
pub(crate) mod connect_model;
pub(crate) mod goals;
pub(crate) mod home;
pub(crate) mod how;
pub(crate) mod inbox;
pub(crate) mod model_check;
pub(crate) mod settings;
pub(crate) mod team;
pub(crate) mod terminal;
pub(crate) mod ways;
use alloc::boxed::Box;

pub(crate) fn project_name(me: Option<&jig_client::contract::Me>, project: u32) -> Option<Box<[u8]>> {
    for row in &me?.projects {
        if row.project == project {
            return Some(skein_lib::bytes::copy_of(row.name.bytes()));
        }
    }
    None
}

/// The playground's forge has no external pages (temper/docs/design/client/forge/family.md, 7).
pub(crate) fn serves_pages(root: &temper_client_domain::Domain, forge: u16) -> bool {
    match root.forge().playground() {
        Some(playground) => playground.forge != forge,
        None => true,
    }
}
