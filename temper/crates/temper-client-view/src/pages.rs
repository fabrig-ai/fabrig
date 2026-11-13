//! temper’s own page content.
pub(crate) use crate::common::{project_name, serves_pages, chat_line, ways};
pub use temper_client_view_work::{pages::chats, pages::goals, pages::home};
pub(crate) use temper_client_view_work::{pages::agent, pages::all_home, pages::approval, pages::ask, pages::ask_page, pages::change, pages::changes, pages::chat, pages::chat_sheet, pages::decided, pages::document, pages::findings, pages::goal, pages::home_cards, pages::inbox, pages::needs, pages::new_chat, pages::only_you, pages::session, pages::start_goal, pages::task, pages::tell, pages::terminal, pages::work};
pub(crate) use temper_client_view_setup::{pages::accounts, pages::account_programs, pages::builders, pages::claude_login, pages::connect_model, pages::deployment, pages::how, pages::model_check, pages::models, pages::remembered, pages::roles, pages::settings, pages::sign_in_wait, pages::team, pages::told, pages::welcome, pages::welcome_copy, pages::welcome_model};
