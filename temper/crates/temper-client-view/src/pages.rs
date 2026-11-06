//! temper's own page content.
use alloc::boxed::Box;
mod account_programs;
pub(crate) mod accounts;
pub(crate) mod agent;
pub(crate) mod ask;
pub(crate) mod builders;
pub(crate) mod change;
pub(crate) mod changes;
pub(crate) mod chat;
pub(crate) mod chat_sheet;
pub mod chats;
pub(crate) mod configure;
pub(crate) mod connect_model;
pub(crate) mod document;
pub(crate) mod goal;
pub mod goals;
pub mod home;
pub(crate) mod inbox;
pub(crate) mod model_check;
pub(crate) mod needs;
pub(crate) mod new_chat;
pub(crate) mod session;
pub(crate) mod settings;
mod sign_in_wait;
pub(crate) mod start_goal;
pub(crate) mod task;
pub(crate) mod welcome;
mod welcome_model;

pub(crate) fn project_name(me: Option<&jig_client::contract::Me>, project: u32) -> Option<Box<[u8]>> {
    for row in &me?.projects {
        if row.project == project {
            return Some(skein_lib::bytes::copy_of(row.name.bytes()));
        }
    }
    None
}

/// The playground's forge has no external pages (temper/docs/design/client/README.md, 4.8).
pub(crate) fn serves_pages(root: &temper_client_domain::Domain, forge: u16) -> bool {
    match root.forge().playground() {
        Some(playground) => playground.forge != forge,
        None => true,
    }
}
