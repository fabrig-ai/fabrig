//! Tests of work pages (temper/docs/design/client/view-crates.md, 4.3).
pub(crate) mod fixtures;
pub use fixtures::documents;
mod all_home;
mod change;
mod changes;
mod chat;
mod chat_line;
mod chats;
mod goal;
mod goals;
mod home_cards;
mod needs;
mod playground;
mod start_goal;
mod task;
mod tell;
mod work;
use fixtures::{cards, inbox, limits_helpers as limits};
