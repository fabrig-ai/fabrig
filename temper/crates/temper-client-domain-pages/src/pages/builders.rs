//! The deployment's hosts as Settings' Builders tab reads them (client/pages/README.md, Settings).
use super::{Request, accounts};
use crate::owner::Owner;
use jig_client::{Client, Handle, Latest, State, contract as c};
use skein_lib::Queue;

#[derive(Debug)]
pub struct Builders {
    handle: Option<Handle>,
}

#[derive(Debug)]
pub enum Read<'a> {
    NoRole,
    Unknown,
    Hosts(&'a c::Deployment),
}

#[expect(clippy::derivable_impls, reason = "step state starts explicitly without a watch")]
impl Default for Builders {
    fn default() -> Self {
        Self { handle: None }
    }
}

impl Builders {
    pub(crate) fn configure(&mut self, role: bool, next: &mut u64, out: &mut Queue<Request>) {
        if role && self.handle.is_none() {
            let handle = Handle { asker: Owner::Pages.asker(), number: *next };
            *next = next.checked_add(1).expect("page handles fit");
            self.handle = Some(handle);
            out.push(Request::Client(jig_client::Event::Watch {
                handle,
                subject: jig_client::Watchable::Jig(c::Subject::Deployment),
            }));
        } else if !role {
            self.leave(out);
        }
    }

    pub(crate) fn leave(&mut self, out: &mut Queue<Request>) {
        if let Some(handle) = self.handle.take() {
            out.push(Request::Client(jig_client::Event::Unwatch { handle }));
        }
    }

    pub(super) fn watches(&self, handle: Handle) -> bool {
        self.handle == Some(handle)
    }

    #[must_use]
    pub fn read<'a>(&self, client: &'a Client) -> Read<'a> {
        let Some(me) = accounts::current_me(client) else { return Read::Unknown };
        if me.deployment_role.is_none() {
            return Read::NoRole;
        }
        let Some(handle) = self.handle else { return Read::Unknown };
        let Some(doc) = client.document(handle) else { return Read::Unknown };
        if doc.state != State::Live {
            return Read::Unknown;
        }
        if let Some(Latest::Jig(c::Delivery::Deployment(deployment))) = doc.latest {
            Read::Hosts(deployment)
        } else {
            Read::Unknown
        }
    }
}
