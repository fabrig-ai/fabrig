//! First-run facts (temper/docs/design/client/README.md, 3.5).
use super::accounts;
use crate::{
    Domain,
    forge::{
        self,
        forges::{self, State as Forges},
    },
};
use alloc::boxed::Box;
use jig_client::{Latest, State};
#[derive(Debug)]
pub struct Welcome {
    pub decided: bool,
}
impl Welcome {
    pub(crate) const fn new() -> Self {
        Self { decided: false }
    }
}
/// The store's first-run state (temper/docs/design/client/README.md, 3.5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FirstRun {
    Empty,
    Unfinished { step: crate::Step },
    Whole,
}
/// None while the session's deployment, playground, forge or account facts are missing.
#[must_use]
pub fn decision(root: &Domain) -> Option<FirstRun> {
    root.forge().deployment()?;
    let me = accounts::current_me(root.client())?;
    if !root.forge().playground_answered() {
        return None;
    }
    let model = model_usable(root)?;
    let rows = match root.forges() {
        Forges::Unknown => return None,
        Forges::Ended => None,
        Forges::Rows(rows) => Some(rows),
    };
    let mut kept = false;
    let mut connected = false;
    if let Some(rows) = rows {
        for row in rows {
            if row.mode != forges::Mode::Fake {
                kept = true;
                connected |= forge_connected(row);
            }
        }
    }
    Some(if connected && model && !me.projects.is_empty() {
        FirstRun::Whole
    } else if !kept && !model {
        FirstRun::Empty
    } else if !connected {
        FirstRun::Unfinished { step: crate::Step::Forge { forge: None } }
    } else if !model {
        FirstRun::Unfinished { step: crate::Step::Model { way: None } }
    } else {
        FirstRun::Unfinished { step: crate::Step::Ready }
    })
}
/// A real connected forge, including GitHub's accepted installation (3.5).
#[must_use]
pub fn forge_connected(row: &forges::Row) -> bool {
    let connected = match row.connection {
        forges::Connection::Connected { .. } => true,
        forges::Connection::NotConnected
        | forges::Connection::NotSetUp
        | forges::Connection::SettingUp { .. }
        | forges::Connection::SetUpFailed { .. }
        | forges::Connection::Connecting
        | forges::Connection::Stopped { .. } => false,
    };
    row.mode != forges::Mode::Fake && connected && (row.kind != forges::Kind::GitHub || forges::accepted(row))
}
/// What Ready adopts (temper/docs/design/client/README.md, 3.5): a first repository made and adopted nowhere.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct First {
    pub forge: u16,
    pub org: Box<[u8]>,
    pub name: Box<[u8]>,
}
/// The offered name (temper/docs/design/client/README.md, 3.5).
#[must_use]
pub fn offered(first: Option<&First>) -> Box<[u8]> {
    let Some(first) = first else { return OFFERED_NAME.into() };
    if first.name.is_empty() || first.name.len() > usize::try_from(NAME_BYTES).expect("name bound fits") {
        return OFFERED_NAME.into();
    }
    let mut name = first.name.clone();
    if let Some(byte) = name.first_mut() {
        byte.make_ascii_uppercase();
    }
    name
}
/// Ready's first project: its written name, request and last ending,
/// which survive navigation (3.5).
#[derive(Debug)]
pub struct DefaultProject {
    /// What the person wrote in the field; none until they write.
    pub written: Option<Box<[u8]>>,
    /// Create was pressed with no name written.
    pub unnamed: bool,
    pub making: Option<jig_client::Key>,
    pub attempt: u64,
    pub ended: Option<Ended>,
    pub project: Option<u32>,
}
impl DefaultProject {
    pub(crate) fn new() -> Self {
        Self { written: None, unnamed: false, making: None, attempt: 0, ended: None, project: None }
    }
    /// The field's value: what was written, else the offered name.
    #[must_use]
    pub fn shown<'a>(&'a self, offered: &'a [u8]) -> &'a [u8] {
        match &self.written {
            Some(written) => written,
            None => offered,
        }
    }
    /// What Create asks for: `shown` without its outer blanks.
    #[must_use]
    pub fn asked<'a>(&'a self, offered: &'a [u8]) -> &'a [u8] {
        self.shown(offered).trim_ascii()
    }
}
/// Ready's adoption, or Home's again (temper/docs/design/client/README.md, 3.5).
#[derive(Debug, PartialEq, Eq)]
pub struct Adopting {
    pub project: u32,
    pub first: First,
    /// The request out, if one is.
    pub out: Option<jig_client::Key>,
    /// Each attempt after a refusal names a key of its own.
    pub attempt: u64,
    pub ended: Option<AdoptionEnded>,
}
#[derive(Debug, PartialEq, Eq)]
pub enum AdoptionEnded {
    Refused(forge::Answer),
    Unknown,
}
/// The first project's name until the person writes another (3.5).
pub const OFFERED_NAME: &[u8] = b"My project";
/// jig's project name bound (jig-domain-people's `NAME_BYTES`).
pub(crate) const NAME_BYTES: u32 = 64;
#[derive(Debug, PartialEq, Eq)]
pub enum Ended {
    Refused(jig_client::contract::Refusal),
    Unknown,
}
/// A real account alone keeps the playground offered (3.5).
#[must_use]
pub fn playground_offered(root: &Domain) -> Option<bool> {
    if !root.forge().deployment()?.welcome {
        return Some(false);
    }
    if !root.forge().playground_answered() {
        return None;
    }
    match root.forges() {
        Forges::Unknown => None,
        Forges::Ended => Some(true),
        Forges::Rows(rows) => {
            for row in rows {
                if row.mode != forges::Mode::Fake {
                    return Some(false);
                }
            }
            Some(true)
        }
    }
}

/// The welcome reads model facts, including an ended watch.
#[must_use]
pub fn model_usable(root: &Domain) -> Option<bool> {
    let document = root.client().document(accounts::handle())?;
    let accounts = if document.state == State::Ended {
        None
    } else if let Some(Latest::Jig(jig_client::contract::Delivery::Accounts(accounts))) = document.latest {
        Some(accounts.as_ref())
    } else {
        return None;
    };
    let Some(accounts) = accounts else { return Some(false) };
    let Some(provider) = root.forge().playground_provider() else { return Some(accounts.usable) };
    Some(forge::model::other_model_usable(
        accounts,
        forge::model::owner_deployment(root.forge(), root.client()),
        provider,
    ))
}
