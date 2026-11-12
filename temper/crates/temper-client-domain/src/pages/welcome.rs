//! First-run facts (temper/docs/design/client/pages/README.md).
use super::{
    accounts::{self, PasteState, SignInState, Way},
    connect_model,
};
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
/// What the welcome's Model step draws (client/pages/README.md, Model).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModelStep {
    /// A model usable, at any way or none.
    Usable,
    /// No way: the plans' cards.
    Choose,
    /// Claude Code holds the terminal, or its source's check is under way.
    TerminalLogin,
    /// A sign-in's way, not yet asked or ended with no model.
    SignIn {
        way: Way,
    },
    /// A sign-in asked, its tab not gone yet.
    Away {
        way: Way,
    },
    /// A paste accepted.
    Finishing {
        way: Way,
    },
    Key {
        way: Way,
    },
    Program {
        way: Way,
    },
}
/// None until the page and `me` are in.
#[must_use]
pub fn model_step(root: &Domain) -> Option<ModelStep> {
    let page = root.pages().connect_model()?;
    accounts::current_me(root.client())?;
    if let connect_model::LoginState::Waiting { .. } = page.login_state(root.client()) {
        return Some(ModelStep::TerminalLogin);
    }
    if model_usable(root) == Some(true) {
        return Some(ModelStep::Usable);
    }
    let Some(way) = connect_model::way(root.pages(), root.client()) else {
        return Some(ModelStep::Choose);
    };
    Some(match way {
        Way::Key { .. } => ModelStep::Key { way },
        Way::Program { .. } => ModelStep::Program { way },
        Way::SignIn { .. } => {
            if page.request.paste_state(root.client()) == PasteState::Finishing {
                ModelStep::Finishing { way }
            } else {
                match page.request.sign_in_state(root.client()) {
                    SignInState::Started { .. } | SignInState::Signing { .. } => ModelStep::Away { way },
                    SignInState::Unknown | SignInState::Ended { .. } => ModelStep::SignIn { way },
                }
            }
        }
    })
}
/// A step's facts: the address is current, and only another done step is ticked (3.5).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct StepMark {
    pub current: bool,
    pub ticked: bool,
}
/// Welcome, Forge, Model, Ready, in order (client/pages/README.md, Its steps).
#[must_use]
pub fn step_marks(at: crate::Step, forge_kept: bool, model_usable: bool) -> [StepMark; 4] {
    let current: u8 = match at {
        crate::Step::Welcome | crate::Step::WelcomeAsked => 0,
        crate::Step::Forge { .. } => 1,
        crate::Step::Model { .. } => 2,
        crate::Step::Ready => 3,
    };
    [
        StepMark { current: current == 0, ticked: current != 0 },
        StepMark { current: current == 1, ticked: forge_kept && current != 1 },
        StepMark { current: current == 2, ticked: model_usable && current != 2 },
        StepMark { current: current == 3, ticked: false },
    ]
}
/// A real forge is kept, even under way or cut short: what `decision` calls kept.
#[must_use]
pub fn forge_kept(root: &Domain) -> bool {
    let rows = match root.forges() {
        Forges::Rows(rows) => rows,
        Forges::Unknown | Forges::Ended => return false,
    };
    for row in rows {
        if row.mode != forges::Mode::Fake {
            return true;
        }
    }
    false
}
/// Where "Go on setting up" and Forge's and Model's "Continue" go: Ready with a usable model, else Model.
#[must_use]
pub fn go_on(root: &Domain) -> crate::Step {
    if model_counts(root) == Some(true) { crate::Step::Ready } else { crate::Step::Model { way: None } }
}
/// The store's first-run state (temper/docs/design/client/pages/README.md).
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
    let model = model_counts(root)?;
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
/// The person signed in by the printed link: the forges document says so,
/// or the welcome's decision is past its first step, or `me` names a
/// project (temper/docs/design/client/tui/box.md).
#[must_use]
pub fn signed_in(root: &Domain) -> bool {
    if root.forge().linked() == Some(true) {
        return true;
    }
    match decision(root) {
        Some(FirstRun::Unfinished { .. } | FirstRun::Whole) => return true,
        Some(FirstRun::Empty) | None => {}
    }
    match accounts::current_me(root.client()) {
        Some(me) => !me.projects.is_empty(),
        None => false,
    }
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
/// What Ready adopts (temper/docs/design/client/pages/README.md): a first repository made and adopted nowhere.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct First {
    pub forge: u16,
    pub org: Box<[u8]>,
    pub name: Box<[u8]>,
}
/// The offered name (temper/docs/design/client/pages/README.md).
#[must_use]
pub fn offered(first: Option<&First>, copies: Option<&forge::making::Copies>) -> Box<[u8]> {
    let name = match first {
        Some(first) => first.name.clone(),
        None => {
            let Some(copies) = copies else { return OFFERED_NAME.into() };
            let Some(address) = copies.addresses.first() else { return OFFERED_NAME.into() };
            let Some((_, name)) = forge::making::target(address) else { return OFFERED_NAME.into() };
            name
        }
    };
    if name.is_empty() || name.len() > usize::try_from(NAME_BYTES).expect("name bound fits") {
        return OFFERED_NAME.into();
    }
    let mut name = name;
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
/// Ready's adoption, or Home's again (temper/docs/design/client/pages/README.md).
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

/// `model_usable`, or a declaration whose host’s check is out (client/pages/README.md, *The try*).
#[must_use]
pub fn model_counts(root: &Domain) -> Option<bool> {
    if let Some(page) = root.pages().connect_model() {
        match page.check {
            connect_model::ProgramCheck::Declaring | connect_model::ProgramCheck::Checking { .. } => return Some(true),
            connect_model::ProgramCheck::Idle | connect_model::ProgramCheck::NotYet => {}
        }
    }
    model_usable(root)
}
