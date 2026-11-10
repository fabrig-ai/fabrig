//! The App shared by GitHub's plans (forge/setup/README.md, 4.4).
use crate::github::{Reply, reply};
use crate::{
    call::Auth,
    item,
    plan::{Event, Why},
};
use core::num::NonZeroU64;
use skein_lib::Secret;

/// The App a plan acts as: its id and its key (forge/setup/README.md, 4.4).
#[derive(Clone, Debug)]
pub(crate) struct Kept {
    id: u64,
    key: Secret,
}
impl Kept {
    pub(crate) fn new(id: NonZeroU64, key: Secret) -> Self {
        Self { id: id.get(), key }
    }
    pub(crate) fn id(&self) -> u64 {
        self.id
    }
    /// The host signs a fresh JWT with the key for each send.
    pub(crate) fn auth(&self) -> Auth {
        Auth::App { id: self.id(), key: self.key.clone() }
    }
}
/// An absent or refused App id is no id.
pub(crate) fn id_of(read: Option<Secret>) -> Option<NonZeroU64> {
    item::app_id(read?.expose())
}
/// The unauthenticated bot read's nonzero identity (forge/setup/github.md, 1.3).
pub(crate) fn bot(event: Event) -> Result<NonZeroU64, Why> {
    match reply(event)? {
        Reply::Document(skein_forge::github::document::Document::BotUser(id)) => {
            NonZeroU64::new(id).ok_or(Why::Unexpected)
        }
        Reply::Status(404) => Err(Why::Missing),
        Reply::Status(status) => Err(Why::Refused(status)),
        Reply::Down => Err(Why::Down),
        Reply::Empty | Reply::Document(_) | Reply::Rows(_) | Reply::Unsigned => Err(Why::Unexpected),
    }
}
