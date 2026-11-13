//! Shared page facts (temper/docs/design/client/view-crates.md, 3.2).
pub mod accounts;
pub mod all_home;
pub mod approval;
pub mod ask;
pub mod change;
pub mod chat;
pub mod chat_line;
pub mod chats;
pub mod connect_model;
pub mod goals;
pub mod home;
pub mod how;
pub mod inbox;
pub mod model_check;
pub mod settings;
pub mod team;
pub mod terminal;
pub mod ways;
use alloc::boxed::Box;

#[must_use]
pub fn project_name(me: Option<&jig_client::contract::Me>, project: u32) -> Option<Box<[u8]>> {
    for row in &me?.projects {
        if row.project == project {
            return Some(skein_lib::bytes::copy_of(row.name.bytes()));
        }
    }
    None
}

/// The playground's forge has no external pages (temper/docs/design/client/forge/family.md, 7).
#[must_use]
pub fn serves_pages(root: &temper_client_domain::Domain, forge: u16) -> bool {
    match root.forge().playground() {
        Some(playground) => playground.forge != forge,
        None => true,
    }
}
