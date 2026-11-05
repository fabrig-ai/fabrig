//! Home's facts from the session's settings watch (temper/docs/design/client/README.md, 3.5, Home).
use super::{Request, welcome};
use crate::{Domain, forge, owner::Owner};
use alloc::boxed::Box;
use forge::dismissed::{self, Dismissal};
use jig_client::{Client, Handle, Key, Latest, Told, contract as c};
use skein_lib::Queue;

#[derive(Debug)]
#[expect(clippy::partial_pub_fields, reason = "settings is a public read; skip state belongs to this page")]
pub struct Home {
    pub settings: Handle,
    skip: Option<Key>,
    skipped: bool,
    skip_refused: bool,
}
impl Home {
    pub(super) fn new() -> Self {
        Self { settings: forge::ways::handle(), skip: None, skipped: false, skip_refused: false }
    }
    pub(super) fn skip(&mut self, project: u32, limits: forge::Limits, serial: &mut u64, out: &mut Queue<Request>) {
        if self.skip.is_some() {
            return;
        }
        let Some(request) = dismissed::ask(Dismissal::ToBegin { project }, &limits) else { return };
        *serial = serial.checked_add(1).expect("page keys fit");
        let key = Key { asker: Owner::Pages.asker(), number: *serial };
        self.skip = Some(key);
        self.skip_refused = false;
        out.push(Request::Client(jig_client::Event::Send { key, name: None, request }));
    }
    pub(super) fn told(&mut self, told: &Told, limits: forge::Limits) -> bool {
        match told {
            Told::Changed { handle } | Told::Ended { handle, .. } => *handle == self.settings,
            Told::Pending { key } if Some(*key) == self.skip => true,
            Told::Answered { key, outcome } if Some(*key) == self.skip => {
                self.skip = None;
                match dismissed::answer(outcome, limits) {
                    dismissed::Answer::Dismissed => self.skipped = true,
                    dismissed::Answer::Unknown => {}
                    dismissed::Answer::Refused => self.skip_refused = true,
                }
                true
            }
            Told::Link { .. }
            | Told::Pending { .. }
            | Told::Answered { .. }
            | Told::Kept { .. }
            | Told::Dropped { .. }
            | Told::Page { .. }
            | Told::Text { .. } => false,
        }
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

/// What the person's skip says (temper/docs/design/client/README.md, 3.5, To begin).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Skip {
    /// The dismissals aren't delivered, or their watch was refused or ended.
    Unknown,
    /// The dismissals name the project, or this page's request is out or answered.
    Skipped,
    NotSkipped,
}

/// The facts Home draws (temper/docs/design/client/README.md, 3.5, Home).
#[derive(Debug, PartialEq, Eq)]
#[expect(clippy::struct_excessive_bools, reason = "Home exposes the independent facts of To begin's contract")]
pub struct Facts {
    pub configured: bool,
    pub repository: Repository,
    pub chat_asked: bool,
    pub needs: Needs,
    pub delivered: bool,
    pub skip: Skip,
    pub skip_refused: bool,
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
    #[must_use]
    pub fn to_begin(&self) -> bool {
        self.delivered && self.skip == Skip::NotSkipped && self.left() > 0
    }

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
    let home = if root.pages().address() == &(crate::Address::Home { project }) { root.pages().home() } else { None };
    let (configuring, chat_asked) = match root.pages().chats() {
        Some(chats) => chats.chat_counts(root.client(), root.pages().me(root.client()), project),
        None => (false, false),
    };
    let configured = configuring
        || match home {
            Some(home) => home.configured(root.client()),
            None => false,
        };
    let repository = repository(root, project);
    let delivered = repository != Repository::Unknown
        && match root.forges() {
            forge::forges::State::Unknown => false,
            forge::forges::State::Rows(_) | forge::forges::State::Ended => true,
        }
        && match home {
            Some(home) => forge::ways::ended(root.forge()) || delivered(root.client(), home.settings),
            None => false,
        }
        && match root.pages().chats() {
            Some(chats) => chats.delivered(root.client(), project),
            None => false,
        };
    let mut skip = match root.forge().dismissed(Dismissal::ToBegin { project }) {
        Some(true) => Skip::Skipped,
        Some(false) => Skip::NotSkipped,
        None => Skip::Unknown,
    };
    let mut skip_refused = false;
    if let Some(home) = home {
        if home.skip.is_some() || home.skipped {
            skip = Skip::Skipped;
        }
        skip_refused = home.skip_refused;
    }
    let needs = match root.pages().inbox() {
        Some(inbox) if inbox.filter.project == Some(project) && inbox.delivered(root.client()) => Needs::Count {
            count: u32::try_from(inbox.needs().count()).expect("inbox cards fit"),
            cut: *inbox.cut.first().expect("needs cut"),
        },
        Some(_) | None => Needs::Unknown,
    };
    Facts { configured, repository, chat_asked, needs, delivered, skip, skip_refused }
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

pub(super) fn delivered(client: &Client, handle: Handle) -> bool {
    match client.document(handle) {
        Some(document) => document.state == jig_client::State::Ended || document.latest.is_some(),
        None => false,
    }
}
