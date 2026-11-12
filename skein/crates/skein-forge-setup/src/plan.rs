//! The events, needs and outcomes shared by setup plans (skein/docs/design/forge/setup/forgejo.md, 1.2).
use crate::{
    call::{Auth, Call},
    item::Item,
};
use alloc::boxed::Box;
use core::num::NonZeroU64;
use skein_forge::forgejo::document::{Document, Row};
use skein_forge::github::document::AppOwner;
use skein_lib::{Secret, bytes};
/// One effect for the host, or the run's terminal outcome.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Need {
    /// The host signs `Auth::App` per request, so a plan never holds a JWT.
    Send {
        request: Call,
        auth: Auth,
    },
    Read(Item),
    Write {
        item: Item,
        bytes: Secret,
    },
    Draw(u32),
    Done(Outcome),
    Stop(Failure),
}
/// What a plan hands its owner when it ends (forge/setup/forgejo.md, 1.2).
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Outcome {
    /// Nothing to serve or show.
    Nothing,
    /// The first repository, ready for adoption (4.9).
    Created { id: NonZeroU64, branch: Box<[u8]> },
    /// A public repository copied and ready for adoption (4.10).
    Copied { id: NonZeroU64, branch: Box<[u8]> },
    /// The form target, manifest JSON and the host-given state they were made for (5.1).
    Manifest { state: Box<[u8]>, target: Box<[u8]>, manifest: Box<[u8]> },
    /// The App's missing wanted access, possibly empty, and its account (5.6 and 5.8).
    Lacking { lacking: Box<[crate::github::Lacking]>, owner: Option<AppOwner> },
    /// The App a manifest made, and its account (5.2).
    Made { owner: Option<AppOwner> },
    /// The given person's user id; none when the account is gone (4.3, step 7).
    Person { id: Option<NonZeroU64> },
    /// The install link and its host-issued Accept state (5.4 and 5.7).
    Install { state: Box<[u8]>, url: Box<[u8]> },
    /// The user a connected Forgejo's tokens belong to (4.6).
    Identity { id: u64, login: Box<[u8]> },
    /// One page of repositories for the picker (4.8).
    Reachable { rows: Box<[crate::reachable::Reached]>, more: bool },
}
/// The outcome of the outstanding need.
#[derive(Clone, PartialEq, Eq, Debug)]
#[cfg_attr(
    not(target_arch = "wasm32"),
    expect(clippy::large_enum_variant, reason = "a reply's document moves whole, as the store's records do")
)]
pub enum Event {
    Begin,
    Read(Option<Secret>),
    Reply(Reply),
    Written,
    Drawn(Secret),
}
/// An HTTP answer decoded by the host.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Reply {
    Document(Box<Document>),
    /// A page of rows: a list, or a repository search that answered `ok`.
    Rows(Box<[Row]>),
    Empty,
    Status(u16),
    Down,
    GitHubDocument(skein_forge::github::document::Document),
    GitHubRows(Box<[skein_forge::github::document::Row]>),
    /// The host could not sign the App's JWT: the request was not sent.
    Unsigned,
}
/// Names the stopped step, never its credentials.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Failure {
    pub forge: u16,
    pub step: Stage,
    pub subject: Box<[u8]>,
    pub why: Why,
}
/// Setup's steps: the bot's id is in Bot; Repository is a repository's access and hook (4.5).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Stage {
    Admin,
    Bot,
    Token,
    Org,
    Repository,
    File,
    App,
    Installation,
    State,
    Person,
}
/// Why the forge could not be reconciled.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Why {
    Missing,
    Refused(u16),
    NotAdmin,
    BotIsAdmin,
    Unmarked,
    OtherUser,
    /// The answered App id is not the id kept or given.
    OtherApp,
    /// The key is not an `RSA PRIVATE KEY` PEM, exceeds `key_bytes`, or will not sign.
    KeyForm,
    /// The state is unknown, used, expired, or belongs to another forge or plan.
    Unknown,
    /// Another run of the forge is live, or every host slot is held (6.5).
    Busy,
    /// The installation listing exceeds `pages`; the strays seen were
    /// uninstalled and the accepted set was left unpruned.
    Crowded,
    /// The copy destination belongs to another run (4.10).
    Taken,
    /// The copied source has no default branch (4.10).
    Empty,
    /// The copy exceeded its whole-run deadline (4.10).
    TimedOut,
    Down,
    Unexpected,
}
/// A plan's stop, naming its stage and subject without credentials.
pub(crate) fn stop(forge: u16, step: Stage, subject: &[u8], why: Why) -> Need {
    Need::Stop(Failure { forge, step, subject: bytes::copy_of(subject), why })
}

impl Event {
    pub(crate) fn begin(self) -> Result<(), Why> {
        match self {
            Self::Begin => Ok(()),
            Self::Read(_) | Self::Reply(_) | Self::Written | Self::Drawn(_) => Err(Why::Unexpected),
        }
    }
    pub(crate) fn read(self) -> Result<Option<Secret>, Why> {
        match self {
            Self::Read(value) => Ok(value),
            Self::Begin | Self::Reply(_) | Self::Written | Self::Drawn(_) => Err(Why::Unexpected),
        }
    }
    pub(crate) fn written(self) -> Result<(), Why> {
        match self {
            Self::Written => Ok(()),
            Self::Begin | Self::Read(_) | Self::Reply(_) | Self::Drawn(_) => Err(Why::Unexpected),
        }
    }
    pub(crate) fn drawn(self) -> Result<Secret, Why> {
        match self {
            Self::Drawn(value) => Ok(value),
            Self::Begin | Self::Read(_) | Self::Reply(_) | Self::Written => Err(Why::Unexpected),
        }
    }
}
/// An empty secret read is no secret: `Read(Some(empty))` is `Read(None)`.
pub(crate) fn normal(event: Event) -> Event {
    match event {
        Event::Read(Some(value)) if value.is_empty() => Event::Read(None),
        other @ (Event::Begin | Event::Read(_) | Event::Reply(_) | Event::Written | Event::Drawn(_)) => other,
    }
}
