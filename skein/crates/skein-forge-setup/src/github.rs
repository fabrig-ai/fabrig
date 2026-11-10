//! GitHub's owner-provided values and plans (forge/setup/github.md, 1).
pub mod accept;
pub(crate) mod app;
pub mod checks;
pub mod convert;
pub mod given;
pub(crate) mod hook;
pub mod install;
pub(crate) mod installations;
mod key;
pub mod limits;
pub mod manifest;
pub use checks::Lacking;
pub mod reachable;
pub mod reconcile;

pub mod paste;
mod write;

use crate::plan::{self, Event, Why};
use alloc::boxed::Box;
use skein_forge::github::document::{Document, Row};

/// The answers this dialect's plans read (forge/setup/forgejo.md, 1.2).
#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) enum Reply {
    Document(Document),
    Rows(Box<[Row]>),
    Empty,
    Status(u16),
    Down,
    Unsigned,
}
/// A plan's reply, or why it ends at its current stage and subject (4.2).
pub(crate) fn reply(event: Event) -> Result<Reply, Why> {
    match event {
        Event::Reply(reply) => match reply {
            plan::Reply::GitHubDocument(value) => Ok(Reply::Document(value)),
            plan::Reply::GitHubRows(value) => Ok(Reply::Rows(value)),
            plan::Reply::Empty => Ok(Reply::Empty),
            plan::Reply::Status(status) => Ok(Reply::Status(status)),
            plan::Reply::Down => Ok(Reply::Down),
            plan::Reply::Unsigned => Ok(Reply::Unsigned),
            plan::Reply::Document(_) | plan::Reply::Rows(_) => Err(Why::Unexpected),
        },
        Event::Begin | Event::Read(_) | Event::Written | Event::Drawn(_) => Err(Why::Unexpected),
    }
}
pub(crate) fn unexpected(reply: Reply) -> Why {
    match reply {
        Reply::Status(status) => Why::Refused(status),
        Reply::Down => Why::Down,
        Reply::Unsigned => Why::KeyForm,
        Reply::Document(_) | Reply::Rows(_) | Reply::Empty => Why::Unexpected,
    }
}
