//! A project's settings page and its deletion (temper/docs/design/draft/playground-reset.md, 5).
use super::Request;
use crate::{Address, ChatsFilter, owner::Owner};
use alloc::boxed::Box;
use jig_client::{Answered, Client, Key, Outcome, Told, contract as c};
use skein_lib::Queue;

#[derive(Debug)]
pub struct Settings {
    pub project: u32,
    pub typed: Box<[u8]>,
    pub deleting: Option<Key>,
    pub outcome: Option<Deletion>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Deletion {
    Deleted,
    Refused(c::Refusal),
    Unknown,
}

impl Settings {
    pub(crate) fn new(project: u32) -> Self {
        Self { project, typed: Box::default(), deleting: None, outcome: None }
    }

    pub(crate) fn delete(
        &mut self,
        me: Option<&c::Me>,
        playground: Option<u32>,
        serial: &mut u64,
        out: &mut Queue<Request>,
    ) {
        if self.deleting.is_some() || self.outcome == Some(Deletion::Deleted) || playground == Some(self.project) {
            return;
        }
        let Some(me) = me else { return };
        for row in &me.projects {
            if row.project == self.project && row.role == c::HeldRole::Owner && row.name.bytes() == &*self.typed {
                *serial = serial.checked_add(1).expect("pages keys fit");
                let key = Key { asker: Owner::Pages.asker(), number: *serial };
                self.deleting = Some(key);
                self.outcome = None;
                out.push(Request::Client(jig_client::Event::Send {
                    key,
                    name: None,
                    request: jig_client::Ask::Jig(Box::new(c::Ask::RemoveProject { project: self.project })),
                }));
                return;
            }
        }
    }

    pub(crate) fn told(&mut self, client: &Client, told: &Told, out: &mut Queue<Request>) -> bool {
        let outcome = match told {
            Told::Answered { key, outcome } if Some(*key) == self.deleting => outcome,
            Told::Pending { key } if Some(*key) == self.deleting => {
                if let Some(pending) = client.pending(*key)
                    && pending.progress == jig_client::Progress::Unknown
                {
                    self.deleting = None;
                    self.outcome = Some(Deletion::Unknown);
                }
                return true;
            }
            Told::Link { .. }
            | Told::Pending { .. }
            | Told::Answered { .. }
            | Told::Kept { .. }
            | Told::Dropped { .. }
            | Told::Changed { .. }
            | Told::Text { .. }
            | Told::Ended { .. }
            | Told::Page { .. } => return false,
        };
        self.deleting = None;
        self.outcome = Some(match outcome {
            Outcome::Answered(Answered::Jig(c::Answer::Done(c::Outcome::Owner(c::OwnerOutcome::ProjectRemoved {
                project,
            }))))
                if *project == self.project =>
            {
                out.push(Request::Show { address: Address::Chats { filter: ChatsFilter::All, start: false } });
                Deletion::Deleted
            }
            Outcome::Answered(Answered::Jig(c::Answer::Refused { refusal, .. })) => Deletion::Refused(refusal.clone()),
            Outcome::NotKept
            | Outcome::Answered(_)
            | Outcome::Unknown
            | Outcome::Limit
            | Outcome::Unconfirmed
            | Outcome::Held
            | Outcome::Unsent
            | Outcome::Full { .. }
            | Outcome::Lost => Deletion::Unknown,
        });
        true
    }
}
