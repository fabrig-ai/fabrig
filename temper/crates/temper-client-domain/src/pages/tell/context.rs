//! The page left and the web composer's outcome (client/pages/needs-you.md, 1, Tell temper).
use super::{About, State};
use alloc::boxed::Box;
use crate::{Address, pages::{Pages, needs::Toasted}};
use jig_client::Client;
use skein_lib::Time;

#[expect(clippy::match_like_matches_macro, reason = "the strict subset names the address variants without matches!")]
pub(in crate::pages) fn showing(address: &Address) -> bool {
    if let Address::Tell { .. } | Address::Inbox { filter: crate::address::Filter { project: Some(_), .. } } = address { true } else { false }
}
pub(in crate::pages) fn remember(pages: &mut Pages, client: &Client) {
    let mut project = if let Address::Adopt { project } = &pages.address { Some(*project) } else { super::super::address_project(&pages.address) };
    let title = match &pages.address {
        Address::Inbox { .. } | Address::Tell { .. } | Address::Ask { .. } => return,
        Address::Task { .. } => match task_document(pages, client) {
            Some(task) => { project = project.or(Some(task.project)); Some(task.chip.title.bytes().into()) }
            None => None,
        },
        Address::Chat { .. } | Address::Talk { .. } => match chat_document(pages, client) {
            Some(task) => {
                if let Address::Chat { .. } = pages.address
                    && let Some(me) = pages.me(client)
                    && super::super::chats::asking(me, task.project, &task.chip.executor)
                { return; }
                project = project.or(Some(task.project));
                Some(task.chip.title.bytes().into())
            }
            None => None,
        },
        Address::Team { role: Some(task), .. } => pages.team.role_title(*task),
        Address::Team { role: None, .. } | Address::Home { .. } | Address::AllHome
        | Address::Work { .. } | Address::Goals { .. } | Address::Changes { .. }
        | Address::Chats { .. } | Address::NewChat { .. } | Address::Settings { .. }
        | Address::Repositories { .. } | Address::Models { .. } | Address::Builders { .. }
        | Address::Role { .. } | Address::Rule { .. } | Address::Adopt { .. }
        | Address::Members | Address::Remembered | Address::Told | Address::Limits
        | Address::Accounts | Address::Forges | Address::ConnectForge { .. }
        | Address::ConnectModel { .. } | Address::Welcome { .. } => None,
    };
    pages.was_on = Some(About { address: pages.address.clone(), title });
    pages.was_on_project = project;
}
pub(in crate::pages) fn outcome(pages: &mut Pages, now: Time) {
    let Some(tell) = pages.tell.last() else { return };
    if tell.serial <= pages.tell_said { return; }
    match &tell.state {
        State::Sending => return,
        State::Filed { .. } => {
            pages.tell_words = Box::default();
            pages.needs.say(Toasted::TellFiled { project: tell.project, goes: tell.goes.clone() }, now);
        }
        State::Refused(_) | State::Lost | State::Long { .. } => {}
    }
    pages.tell_said = tell.serial;
}

fn task_document<'a>(pages: &'a Pages, client: &'a Client) -> Option<&'a jig_client::contract::Task> { pages.task.as_ref()?.document(client) }
fn chat_document<'a>(pages: &'a Pages, client: &'a Client) -> Option<&'a jig_client::contract::Task> { pages.chat.as_ref()?.task(client) }
