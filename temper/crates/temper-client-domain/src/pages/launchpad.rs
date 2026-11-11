//! The TUI's left-off reads (temper/docs/design/client/tui/launchpad.md, 2).
use super::Request;
use crate::{Limits, owner::Owner};
use jig_client::{Client, Handle, Latest, Told, Watchable, contract as c, shapes};
use skein_lib::Queue;

#[derive(Debug)]
pub struct LeftOff {
    project: u32,
    chats: Handle,
    goals: Handle,
    chat_query: Option<c::TaskQuery>,
    goal_query: Option<c::TaskQuery>,
}
impl LeftOff {
    pub(super) fn new(project: u32, next: &mut u64) -> Self {
        let chats = Handle { asker: Owner::Pages.asker(), number: *next };
        *next = next.checked_add(1).expect("page handles fit");
        let goals = Handle { asker: Owner::Pages.asker(), number: *next };
        *next = next.checked_add(1).expect("page handles fit");
        Self { project, chats, goals, chat_query: None, goal_query: None }
    }
    #[must_use]
    pub fn chats<'a>(&self, client: &'a Client) -> Option<&'a c::Tasks> {
        tasks(client, self.chats)
    }
    #[must_use]
    pub fn goals<'a>(&self, client: &'a Client) -> Option<&'a c::Tasks> {
        tasks(client, self.goals)
    }
    pub(super) fn configure(&mut self, me: Option<&c::Me>, limits: &Limits, out: &mut Queue<Request>) {
        let Some(me) = me else { return };
        for (template, tracked, handle, held) in [
            (shapes::CHAT, None, self.chats, &mut self.chat_query),
            (shapes::GOAL, Some(true), self.goals, &mut self.goal_query),
        ] {
            let query = match shapes::template(me, self.project, template) {
                Some(template) => Some(c::TaskQuery {
                    project: None,
                    requester: Some(c::Requester::Person(me.party.number)),
                    root: None,
                    parent: None,
                    label: Some(template.label),
                    phase: None,
                    executor: None,
                    tracked,
                    order: c::QueryOrder::Activity,
                }),
                None => None,
            };
            if *held == query {
                continue;
            }
            if held.is_some() {
                out.push(Request::Client(jig_client::Event::Unwatch { handle }));
            }
            *held = query;
            if let Some(query) = query {
                out.push(Request::Client(jig_client::Event::Watch {
                    handle,
                    subject: Watchable::Jig(c::Subject::Tasks { query, window: limits.client.domain.rows }),
                }));
            }
        }
    }
    pub(super) fn told(&self, told: &Told) -> bool {
        match told {
            Told::Changed { handle } | Told::Ended { handle, .. } => *handle == self.chats || *handle == self.goals,
            Told::Link { .. }
            | Told::Pending { .. }
            | Told::Answered { .. }
            | Told::Kept { .. }
            | Told::Dropped { .. }
            | Told::Text { .. }
            | Told::Page { .. } => false,
        }
    }
    pub(super) fn leave(&self, out: &mut Queue<Request>) {
        for (handle, query) in [(self.chats, self.chat_query), (self.goals, self.goal_query)] {
            if query.is_some() {
                out.push(Request::Client(jig_client::Event::Unwatch { handle }));
            }
        }
    }
}
fn tasks(client: &Client, handle: Handle) -> Option<&c::Tasks> {
    let document = client.document(handle)?;
    if document.state == jig_client::State::Ended {
        return None;
    }
    match document.latest {
        Some(Latest::Jig(c::Delivery::Tasks(tasks))) => Some(tasks),
        Some(Latest::Jig(_) | Latest::Family(_)) | None => None,
    }
}
