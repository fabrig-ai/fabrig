//! Runtime setup entries (temper/docs/design/draft/playground-reset.md, 3).
use alloc::boxed::Box;
use core::net::SocketAddr;
/// A Forgejo account temper names.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Account {
    pub login: Box<[u8]>,
    pub email: Box<[u8]>,
}
/// An owned forge's runner, including the PATH of its jobs.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Runner {
    pub binary: Box<[u8]>,
    pub labels: Box<[Box<[u8]>]>,
    pub capacity: u32,
    pub path: Box<[u8]>,
}
/// What temper runs for one forge.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Owned {
    pub public: Option<Origin>,
    pub fetch: Fetch,
    pub forgejo: Box<[u8]>,
    pub runner: Option<Runner>,
    pub listen: SocketAddr,
    pub person: Account,
}
/// Which pinned binaries the fetch supplies (protocol/setup.md, 6.6).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Fetch {
    pub forgejo: bool,
    pub runner: bool,
    pub mirror: Option<Mirror>,
}
/// Alternate origins; the release paths and digests stay pinned.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Mirror {
    pub forgejo: Origin,
    pub runner: Origin,
}
/// An HTTPS authority, without a path.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Origin {
    pub host: Box<[u8]>,
    pub port: u16,
}
/// One of a fake forge's repositories.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Repository {
    pub name: Box<[u8]>,
    pub landing: Box<[u8]>,
}
/// The demo's fake forge, which temper runs (protocol/setup.md, 8.4).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Fake {
    pub listen: SocketAddr,
    pub git: Box<[u8]>,
    pub owner: Box<[u8]>,
    pub repositories: Box<[Repository]>,
}
/// A person's Forgejo or one temper runs.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum Mode {
    Managed { admin_token: Option<Box<[u8]>> },
    Owned(Owned),
    Fake(Fake),
}
/// One forge's directory, bot, orgs and mode.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Entry {
    pub forge: u16,
    pub directory: Box<[u8]>,
    pub bot: Account,
    pub orgs: Box<[Box<[u8]>]>,
    pub mode: Mode,
    pub asked: bool,
}
/// The setup list, with at most one entry per forge.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Setup {
    pub entries: Box<[Entry]>,
}
impl Setup {
    /// Finds the entry for a forge number.
    #[must_use]
    pub fn entry(&self, number: u16) -> Option<&Entry> {
        let mut found = None;
        for e in &self.entries {
            if e.forge == number {
                found = Some(e);
            }
        }
        found
    }
}
