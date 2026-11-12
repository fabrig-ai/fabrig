//! Forgejo's owner-provided values and plans (forge/setup/forgejo.md, 1).
mod access;
mod admin;
pub mod adopt;
pub mod check;
pub mod copy;
pub mod create;
pub mod given;
pub mod limits;
pub mod reachable;
pub mod reconcile;
pub mod take_over;

use crate::plan::{self, Event, Why};
use alloc::boxed::Box;
use skein_forge::forgejo::document::{Document, Row};

/// The answers this dialect's plans read (forge/setup/forgejo.md, 1.2).
#[derive(Clone, PartialEq, Eq, Debug)]
pub(crate) enum Reply {
    Document(Box<Document>),
    Rows(Box<[Row]>),
    Empty,
    Status(u16),
    Down,
}
/// A plan's reply, or why it ends at its current stage and subject (4.2).
pub(crate) fn reply(event: Event) -> Result<Reply, Why> {
    match event {
        Event::Reply(reply) => match reply {
            plan::Reply::Document(value) => Ok(Reply::Document(value)),
            plan::Reply::Rows(value) => Ok(Reply::Rows(value)),
            plan::Reply::Empty => Ok(Reply::Empty),
            plan::Reply::Status(status) => Ok(Reply::Status(status)),
            plan::Reply::Down => Ok(Reply::Down),
            plan::Reply::Unsigned => Err(Why::KeyForm),
            plan::Reply::GitHubDocument(_) | plan::Reply::GitHubRows(_) => Err(Why::Unexpected),
        },
        Event::Begin | Event::Read(_) | Event::Written | Event::Drawn(_) => Err(Why::Unexpected),
    }
}
pub(crate) fn unexpected(reply: Reply) -> Why {
    match reply {
        Reply::Status(status) => Why::Refused(status),
        Reply::Down => Why::Down,
        Reply::Document(_) | Reply::Rows(_) | Reply::Empty => Why::Unexpected,
    }
}
