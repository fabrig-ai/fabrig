//! Home's facts and settings watch (temper/docs/design/client/README.md, 3.5, Home).
use super::{Request, welcome};
use crate::{Domain, forge, owner::Owner};
use alloc::boxed::Box;
use jig_client::{Client, Handle, Latest, Told, Watchable, contract as c};
use skein_lib::Queue;

#[derive(Debug)]
pub struct Home {
    pub settings: Handle,
}
impl Home {
    pub(super) fn new(number: u64, out: &mut Queue<Request>) -> Self {
        let home = Self { settings: Handle { asker: Owner::Pages.asker(), number } };
        out.push(Request::Client(jig_client::Event::Watch {
            handle: home.settings,
            subject: Watchable::Jig(c::Subject::Settings),
        }));
        home
    }
    pub(super) fn leave(&self, out: &mut Queue<Request>) {
        out.push(Request::Client(jig_client::Event::Unwatch { handle: self.settings }));
    }
    pub(super) fn told(&self, told: &Told) -> bool {
        if let Told::Changed { handle } | Told::Ended { handle, .. } = told { *handle == self.settings } else { false }
    }
    fn configured(&self, client: &Client) -> bool {
        let Some(document) = client.document(self.settings) else { return false };
        if document.state == jig_client::State::Ended {
            return false;
        }
        let Some(Latest::Jig(c::Delivery::Settings(settings))) = document.latest else { return false };
        for line in &settings.settings {
            if line.follows == c::SettingFollows::Change {
                return true;
            }
        }
        false
    }
}

/// What needs the person in this project (temper/docs/design/client/README.md, 3.5, Home).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Needs {
    Unknown,
    /// The project's cards of the needs' groups; `cut` where the read was cut.
    Count {
        count: u32,
        cut: bool,
    },
}

/// The facts Home draws (temper/docs/design/client/README.md, 3.5, Home).
#[derive(Debug, PartialEq, Eq)]
pub struct Facts {
    pub configured: bool,
    pub repository: Repository,
    pub chat_asked: bool,
    pub needs: Needs,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Repository {
    Unknown,
    In { owner: Box<[u8]>, name: Box<[u8]> },
    Adopting(welcome::First),
    Offer { first: welcome::First, refused: Option<forge::Answer> },
    GitHub { account: Option<Box<[u8]>> },
    None,
}
impl Facts {
    /// The steps left: step 2 counts unless it is `In`.
    #[must_use]
    pub fn left(&self) -> u8 {
        let repository = match self.repository {
            Repository::In { .. } => false,
            Repository::Unknown
            | Repository::Adopting(_)
            | Repository::Offer { .. }
            | Repository::GitHub { .. }
            | Repository::None => true,
        };
        u8::from(!self.configured)
            .checked_add(u8::from(repository))
            .expect("three steps fit")
            .checked_add(u8::from(!self.chat_asked))
            .expect("three steps fit")
    }
}
#[must_use]
pub fn facts(root: &Domain, project: u32) -> Facts {
    let configured = match root.pages().home() {
        Some(home) => home.configured(root.client()),
        None => false,
    };
    let chat_asked = match root.pages().chats() {
        Some(chats) => chats.asked(root.client(), project),
        None => false,
    };
    let needs = match root.pages().inbox() {
        Some(inbox) if inbox.filter.project == Some(project) && inbox.delivered(root.client()) => Needs::Count {
            count: u32::try_from(inbox.needs().count()).expect("inbox cards fit"),
            cut: *inbox.cut.first().expect("needs cut"),
        },
        Some(_) | None => Needs::Unknown,
    };
    Facts { configured, repository: repository(root, project), chat_asked, needs }
}
fn repository(root: &Domain, project: u32) -> Repository {
    use forge::listed::Held;
    match forge::listed::held(root.forges(), root.forge().playground(), project) {
        Held::One { owner, name } | Held::Several { owner, name, .. } => {
            return Repository::In { owner: owner.into(), name: name.into() };
        }
        Held::Unknown => {
            return match root.forges() {
                forge::forges::State::Ended => Repository::None,
                forge::forges::State::Unknown | forge::forges::State::Rows(_) => Repository::Unknown,
            };
        }
        Held::None => {}
    }
    if let Some(adopting) = root.pages().adopting()
        && adopting.project == project
        && adopting.out.is_some()
    {
        return Repository::Adopting(adopting.first.clone());
    }
    if let Some((forge, org, name)) = forge::listed::unadopted_first(root.forges()) {
        let first = welcome::First { forge, org: org.into(), name: name.into() };
        let mut refused = None;
        if let Some(adopting) = root.pages().adopting()
            && adopting.project == project
            && adopting.first == first
            && let Some(welcome::AdoptionEnded::Refused(answer)) = &adopting.ended
        {
            refused = Some(answer.clone());
        }
        return Repository::Offer { first, refused };
    }
    for row in &forge::listed::rows(root.forges(), forge::connect::playground(root.forge())) {
        if row.kind == forge::forges::Kind::GitHub
            && let forge::forges::Connection::Connected { .. } = row.connection
        {
            let mut account = None;
            for installation in &row.installations {
                if !installation.stray && !installation.suspended {
                    if let Some(accepted) = &installation.account {
                        account = Some(accepted.login.clone());
                    }
                    break;
                }
            }
            return Repository::GitHub { account };
        }
    }
    Repository::None
}
