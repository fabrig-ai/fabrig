//! Connect one more model account; shared with the welcome (temper/docs/design/client/pages/README.md).
use super::{
    Pages, Request as Output,
    accounts::{self, Broken, Way, request::Request},
};
use crate::{Address, ModelWay, Step, WayKind};
use alloc::boxed::Box;
use jig_client::{Client, Handle, Link, Readable, Readout, Told, contract as c};
use skein_lib::{Queue, RequestKey};
use temper_client_protocol_wire::{self as wire, v1 as w};
/// A program’s "I’ve signed in" (temper/docs/design/client/pages/README.md, A way’s page).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProgramCheck {
    Idle,
    /// `DeclareProgram` out.
    Declaring,
    /// `CheckProgram` out, or answered and not yet settled.
    Checking {
        source: u32,
    },
    /// Settled with the source not usable.
    NotYet,
}
#[derive(Debug)]
#[expect(
    clippy::partial_pub_fields,
    reason = "request state is public; the tab refusal and environment have read-only accessors"
)]
pub struct ConnectModel {
    pub holder: c::AskAccountHolder,
    pub check: ProgramCheck,
    /// The way last pressed, for a check’s retry and return to sign-in.
    pub tried: Option<Way>,
    pub program: Option<Box<[u8]>>,
    pub request: Request,
    /// Whether the scripted model was serving when this connection was sent.
    pub playground: bool,
    pub(crate) tab_refused: bool,
    program_environment: Option<Box<[u8]>>,
    environment_read: Option<Handle>,
    environment_asked: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Act {
    Holder { holder: c::AskAccountHolder },
    SignIn { name: RequestKey, way: Way },
    SignedIn { name: RequestKey, way: Way },
    Finish { name: RequestKey },
    ProgramName { value: Box<[u8]> },
    Submit { name: RequestKey },
    Cancel,
}
#[must_use]
pub const fn open(address: &Address) -> bool {
    match address {
        Address::ConnectModel { .. } | Address::Welcome { step: Step::Model { .. } } => true,
        Address::Forges
        | Address::ConnectForge { .. }
        | Address::AllHome
        | Address::Inbox { .. }
        | Address::Chats { .. }
        | Address::NewChat { .. }
        | Address::Told
        | Address::Remembered
        | Address::Members
        | Address::Limits
        | Address::Accounts
        | Address::Welcome { .. }
        | Address::Chat { .. }
        | Address::Task { .. }
        | Address::Home { .. }
        | Address::Changes { .. }
        | Address::Team { .. }
        | Address::Settings { .. }
        | Address::Models { .. }
        | Address::Builders { .. }
        | Address::Role { .. }
        | Address::Rule { .. }
        | Address::Repositories { .. }
        | Address::Work { .. }
        | Address::Goals { .. }
        | Address::Ask { .. }
        | Address::Adopt { .. } => false,
    }
}
#[must_use]
pub const fn model_way(way: Way) -> ModelWay {
    ModelWay {
        provider: way.provider(),
        source: None,
        kind: match way {
            Way::SignIn { .. } => WayKind::SignIn,
            Way::Program { .. } => WayKind::Program,
            Way::Key { .. } => WayKind::Key,
        },
    }
}
/// Whether this person holds a source of this provider and kind
/// (temper/docs/design/client/pages/README.md, `ConnectModel`, Choose).
#[must_use]
pub fn connected(root: &crate::Domain, way: Way) -> bool {
    let Some(me) = accounts::current_me(root.client()) else { return false };
    let Some(doc) = accounts::document(root.client()) else { return false };
    let kind = match way {
        Way::SignIn { .. } => c::AccountKind::SignIn,
        Way::Program { .. } => c::AccountKind::Program,
        Way::Key { .. } => c::AccountKind::Key,
    };
    for account in &doc.accounts {
        if account.holder == c::AccountHolder::Party(me.party.number) && account.provider == Some(way.provider()) {
            for source in &account.sources {
                if source.kind == kind {
                    return true;
                }
            }
        }
    }
    false
}
/// The first unconnected offered way; none when every way is connected (client/pages/README.md).
#[must_use]
pub fn default_way(ways: &skein_lib::List<Way>, root: &crate::Domain) -> Option<Way> {
    for way in ways {
        if !connected(root, *way) {
            return Some(*way);
        }
    }
    None
}
#[must_use]
pub fn address(current: &Address, way: Option<ModelWay>) -> Address {
    match current {
        Address::Welcome { .. } => Address::Welcome { step: Step::Model { way } },
        Address::Forges
        | Address::ConnectForge { .. }
        | Address::AllHome
        | Address::Inbox { .. }
        | Address::Chats { .. }
        | Address::NewChat { .. }
        | Address::Told
        | Address::Remembered
        | Address::Members
        | Address::Limits
        | Address::Accounts
        | Address::ConnectModel { .. }
        | Address::Chat { .. }
        | Address::Task { .. }
        | Address::Home { .. }
        | Address::Changes { .. }
        | Address::Team { .. }
        | Address::Settings { .. }
        | Address::Models { .. }
        | Address::Builders { .. }
        | Address::Role { .. }
        | Address::Rule { .. }
        | Address::Repositories { .. }
        | Address::Work { .. }
        | Address::Goals { .. }
        | Address::Ask { .. }
        | Address::Adopt { .. } => Address::ConnectModel { way },
    }
}
#[must_use]
pub fn way(pages: &Pages, client: &Client) -> Option<Way> {
    let chosen = match pages.address() {
        Address::ConnectModel { way } | Address::Welcome { step: Step::Model { way } } => (*way)?,
        Address::Forges
        | Address::ConnectForge { .. }
        | Address::AllHome
        | Address::Inbox { .. }
        | Address::Chats { .. }
        | Address::NewChat { .. }
        | Address::Told
        | Address::Remembered
        | Address::Members
        | Address::Limits
        | Address::Accounts
        | Address::Welcome { .. }
        | Address::Chat { .. }
        | Address::Task { .. }
        | Address::Home { .. }
        | Address::Changes { .. }
        | Address::Team { .. }
        | Address::Settings { .. }
        | Address::Models { .. }
        | Address::Builders { .. }
        | Address::Role { .. }
        | Address::Rule { .. }
        | Address::Repositories { .. }
        | Address::Work { .. }
        | Address::Goals { .. }
        | Address::Ask { .. }
        | Address::Adopt { .. } => return None,
    };
    let me = accounts::current_me(client)?;
    let page = pages.connect_model()?;
    for way in &accounts::ways(&me.providers, pages.playground_provider, page.holder) {
        if model_way(*way).provider == chosen.provider && model_way(*way).kind == chosen.kind {
            return Some(*way);
        }
    }
    None
}
impl ConnectModel {
    /// The owner's `program_environment`, empty until answered (temper/docs/design/client/forge/records.md).
    #[must_use]
    pub fn program_environment(&self) -> &[u8] {
        match &self.program_environment {
            Some(environment) => environment,
            None => b"",
        }
    }

    /// Whether the owner’s environment was decoded, including an empty default (client/forge/records.md).
    #[must_use]
    pub const fn program_environment_answered(&self) -> bool {
        self.program_environment.is_some()
    }

    pub(crate) fn read_environment(&mut self, address: &Address, link: Link, next: &mut u64, out: &mut Queue<Output>) {
        let program = match chosen_way(address) {
            Some(way) => way.kind == WayKind::Program,
            None => false,
        };
        if !program || self.environment_asked || link != Link::Live {
            return;
        }
        let handle = Handle { asker: crate::owner::Owner::Pages.asker(), number: *next };
        *next = next.checked_add(1).expect("pages handles fit");
        self.environment_read = Some(handle);
        self.environment_asked = true;
        let record = w::Reading::new(&w::CEILINGS, w::ReadingParts { read: w::Read::ProgramEnvironment })
            .expect("fieldless program environment read");
        out.push(Output::Client(jig_client::Event::Read {
            handle,
            read: Readable::Family {
                scope: c::Scope::Deployment,
                window: c::Window { rows: 0, after: None },
                record: jig_client::Family {
                    family: wire::FORGE,
                    version: wire::CLIENT_VERSION,
                    bytes: wire::reading(&record),
                },
            },
        }));
    }
    pub(crate) fn environment_told(&mut self, told: &Told) -> bool {
        match told {
            Told::Page { handle, page } if Some(*handle) == self.environment_read => {
                self.environment_read = None;
                self.program_environment = match page {
                    Readout::Family { family, record: Some(record) } if *family == wire::FORGE => {
                        match wire::decode_program_environment_answer(record, &w::CEILINGS) {
                            Some(record) => Some(record.into_parts().environment),
                            None => None,
                        }
                    }
                    Readout::Family { .. } | Readout::NotServed { .. } | Readout::Limit | Readout::Jig(_) => None,
                };
                true
            }
            Told::Page { .. }
            | Told::Link { .. }
            | Told::Changed { .. }
            | Told::Ended { .. }
            | Told::Text { .. }
            | Told::Pending { .. }
            | Told::Answered { .. }
            | Told::Kept { .. }
            | Told::Dropped { .. } => false,
        }
    }

    /// Whether the shell refused this sign-in's tab (temper/docs/design/client/pages/README.md).
    #[must_use]
    pub const fn tab_refused(&self) -> bool {
        self.tab_refused || self.request.kept_refused()
    }
    pub(crate) fn changing_way(&mut self, current: &Address, next: &Address) {
        if let Address::ConnectModel { .. } = current
            && let Address::Welcome { .. } = next
        {
            self.program = None;
        }
        let old_way = chosen_way(current);
        let ready = Address::Welcome { step: Step::Ready };
        let choose = Address::Welcome { step: Step::Model { way: None } };
        if *next != ready && !(*current == ready && *next == choose) && old_way != chosen_way(next) {
            self.check = ProgramCheck::Idle;
        }
        if *current == ready && chosen_way(next).is_some() {
            self.check = ProgramCheck::Idle;
            self.request.cancel();
        }
        if let Address::ConnectModel { way: Some(old) } | Address::Welcome { step: Step::Model { way: Some(old) } } =
            current
            && old.kind == WayKind::SignIn
        {
            let next = if let Address::ConnectModel { way } | Address::Welcome { step: Step::Model { way } } = next {
                *way
            } else {
                None
            };
            let same_way = match next {
                Some(next) => next.provider == old.provider && next.kind == old.kind,
                None => false,
            };
            if !same_way {
                self.leave_way();
            }
        }
    }
    fn leave_way(&mut self) {
        self.request.clear_key();
    }
    pub(crate) fn leave(&mut self) {
        self.request.clear_key();
        *self = Self::default();
    }
    #[must_use]
    pub fn program_name(&self, client: &Client) -> Box<[u8]> {
        match &self.program {
            Some(name) => name.clone(),
            None => match accounts::current_me(client) {
                Some(me) => program_default(me.party.name.bytes()),
                None => Box::new([]),
            },
        }
    }
    #[expect(
        clippy::too_many_arguments,
        clippy::too_many_lines,
        reason = "the shared flow dispatches its acts against borrowed identity, address and playground facts"
    )]
    pub(crate) fn act(
        &mut self,
        client: &Client,
        current: &Address,
        chosen: Option<Way>,
        playground: Option<u32>,
        act: Act,
        serial: &mut u64,
        out: &mut Queue<Output>,
    ) {
        if let Some(p) = &self.request.pending
            && p.outcome.is_none()
        {
            return;
        }
        if self.request.paste.out() {
            return;
        }
        let Some(me) = accounts::current_me(client) else { return };
        match act {
            Act::Holder { holder } => {
                if holder == c::AskAccountHolder::Deployment && me.deployment_role != Some(c::HeldRole::Owner) {
                    return;
                }
                self.request.cancel();
                self.holder = holder;
            }
            Act::ProgramName { value } => {
                self.program = Some(Box::new([]));
                self.request.broken = Some(Broken::ProgramName);
                if value.len() <= 32 {
                    self.program = Some(value);
                    if c::valid_program_name(self.program.as_deref().expect("written name")) {
                        self.request.broken = None;
                    }
                }
            }
            Act::Finish { name } => self.request.finish(client, name, serial, out),
            Act::Cancel => {
                self.tab_refused = false;
                self.request.cancel();
            }
            Act::SignIn { name, way } => {
                if model_way(way).kind != WayKind::SignIn
                    || !accounts::ways(&me.providers, playground, self.holder).as_slice().contains(&way)
                {
                    return;
                }
                self.tab_refused = false;
                out.push(Output::OpenTab { provider: way.provider() });
                self.request.send(
                    name,
                    c::Ask::SignIn { holder: c::AskAccountHolder::Own, provider: way.provider() },
                    false,
                    serial,
                    out,
                );
                out.push(Output::Show { address: address(current, Some(model_way(way))) });
            }
            Act::SignedIn { name, way } => {
                if !(matches_welcome_program(current) && chosen == Some(way)
                    || *current == (Address::Welcome { step: Step::Ready }) && self.tried == Some(way))
                    || model_way(way).kind != WayKind::Program
                    || self.check == ProgramCheck::Declaring
                {
                    return;
                }
                if let ProgramCheck::Checking { .. } = self.check {
                    return;
                }
                let Some(doc) = accounts::document(client) else { return };
                let program = program_default(me.party.name.bytes());
                if let Some(source) = program_source(client, way.provider(), &program) {
                    self.send_check(name, source, serial, out);
                } else {
                    if doc.cut {
                        return;
                    }
                    for account in &doc.accounts {
                        if account.holder == c::AccountHolder::Party(me.party.number) && account.cut {
                            return;
                        }
                    }
                    self.check = ProgramCheck::Declaring;
                    self.request.send(
                        name,
                        c::Ask::AccountChange {
                            holder: c::AskAccountHolder::Own,
                            change: c::AccountChange::DeclareProgram {
                                provider: way.provider(),
                                join: None,
                                name: program,
                            },
                        },
                        false,
                        serial,
                        out,
                    );
                }
                self.tried = Some(way);
                if matches_welcome_program(current) {
                    out.push(Output::Replace { address: Address::Welcome { step: Step::Ready } });
                }
            }
            Act::Submit { name } => {
                let Some(way) = chosen else { return };
                let change = match way {
                    Way::Key { provider } => {
                        if !self.request.has_key() {
                            if self.request.broken != Some(Broken::Empty)
                                && self.request.broken != Some(Broken::TooLong)
                            {
                                self.request.broken = Some(Broken::ValueAgain);
                            }
                            return;
                        }
                        let Some(digest) = self.request.digest() else { return };
                        c::AccountChange::AddKey { provider, join: None, digest }
                    }
                    Way::Program { provider } => {
                        let program = self.program_name(client);
                        if !c::valid_program_name(&program) {
                            self.request.broken = Some(Broken::ProgramName);
                            return;
                        }
                        self.tried = Some(way);
                        if let Some(source) = program_source(client, provider, &program) {
                            self.send_check(name, source, serial, out);
                            return;
                        }
                        c::AccountChange::DeclareProgram { provider, join: None, name: program }
                    }
                    Way::SignIn { .. } => return,
                };
                if let Way::Program { .. } = way {
                    self.check = ProgramCheck::Declaring;
                    self.tried = Some(way);
                }
                let ask = c::Ask::AccountChange { holder: self.holder, change };
                if let Some(p) = &self.request.pending
                    && let Some(
                        jig_client::Outcome::Lost | jig_client::Outcome::Unsent | jig_client::Outcome::Full { .. },
                    ) = &p.outcome
                    && (p.name != Some(name) || p.ask != ask)
                {
                    self.request.clear_key();
                    self.request.broken = Some(Broken::ValueAgain);
                    return;
                }
                self.request.send(name, ask, model_way(way).kind == WayKind::Key, serial, out);
            }
        }
    }
}
fn chosen_way(address: &Address) -> Option<ModelWay> {
    if let Address::ConnectModel { way } | Address::Welcome { step: Step::Model { way } } = address {
        *way
    } else {
        None
    }
}
fn matches_welcome_program(address: &Address) -> bool {
    if let Address::Welcome { step: Step::Model { way: Some(way) } } = address {
        way.kind == WayKind::Program
    } else {
        false
    }
}
fn program_source(client: &Client, provider: u32, name: &[u8]) -> Option<u32> {
    let me = accounts::current_me(client)?;
    for account in &accounts::document(client)?.accounts {
        if account.holder == c::AccountHolder::Party(me.party.number) && account.provider == Some(provider) {
            for source in &account.sources {
                if source.kind == c::AccountKind::Program && source.name.as_deref() == Some(name) {
                    return Some(source.number);
                }
            }
        }
    }
    None
}
impl ConnectModel {
    fn send_check(&mut self, name: RequestKey, source: u32, serial: &mut u64, out: &mut Queue<Output>) {
        self.check = ProgramCheck::Checking { source };
        self.request.send(
            name,
            c::Ask::AccountChange {
                holder: c::AskAccountHolder::Own,
                change: c::AccountChange::CheckProgram { source },
            },
            false,
            serial,
            out,
        );
    }
    pub(crate) fn told(&mut self, client: &Client, told: &Told, serial: &mut u64, out: &mut Queue<Output>) -> bool {
        if !self.request.told(client, told) {
            return false;
        }
        let Told::Answered { outcome, .. } = told else { return true };
        if self.check == ProgramCheck::Declaring {
            let source = if let jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(
                c::Outcome::Document(c::DocumentOutcome::AccountProgramDeclared { source, .. }),
            ))) = outcome
            {
                Some(*source)
            } else if let jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused {
                refusal: c::Refusal::NameTaken,
                ..
            })) = outcome
            {
                if let Some(pending) = &self.request.pending
                    && let c::Ask::AccountChange {
                        change: c::AccountChange::DeclareProgram { provider, name, .. }, ..
                    } = &pending.ask
                {
                    program_source(client, *provider, name)
                } else {
                    None
                }
            } else {
                None
            };
            if let Some(source) = source {
                self.send_check(super::account_request_name(*serial), source, serial, out);
            } else {
                self.check = ProgramCheck::Idle;
            }
        } else if let ProgramCheck::Checking { .. } = self.check {
            if let jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Refused {
                refusal: c::Refusal::Limit { which: c::Limit::Admission { which: c::AdmissionLimit::Sources } },
                ..
            })) = outcome
            {
                // Another check of this source is still out; the watch supplies its end.
            } else if let jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(
                c::Outcome::Document(c::DocumentOutcome::AccountChanged),
            ))) = outcome
            {
                // The answer admits the check; settled is the hosts' observation.
            } else {
                self.check = ProgramCheck::Idle;
            }
        }
        true
    }
}

#[must_use]
pub const fn creation(ask: &c::Ask) -> bool {
    if let c::Ask::SignIn { .. } = ask {
        return true;
    }
    if let c::Ask::AccountChange {
        change: c::AccountChange::AddKey { join: None, .. } | c::AccountChange::DeclareProgram { join: None, .. },
        ..
    } = ask
    {
        return true;
    }
    false
}

pub(crate) fn program_default(person: &[u8]) -> Box<[u8]> {
    let mut length = 0;
    for (index, byte) in person.iter().take(25).enumerate() {
        if !byte.is_ascii_alphanumeric() && *byte != b'-' {
            break;
        }
        length = index.checked_add(1).expect("program prefix fits");
    }
    let mut name = skein_lib::Writer::new(7_usize.checked_add(length).expect("program name fits"));
    name.put(b"claude-").expect("measured prefix fits");
    for byte in person.iter().take(length) {
        name.put(&[byte.to_ascii_lowercase()]).expect("measured name fits");
    }
    name.finish()
}

impl Default for ConnectModel {
    fn default() -> Self {
        Self {
            holder: c::AskAccountHolder::Own,
            check: ProgramCheck::Idle,
            tried: None,
            program: None,
            request: Request::default(),
            playground: false,
            tab_refused: false,
            program_environment: None,
            environment_read: None,
            environment_asked: false,
        }
    }
}

/// A standalone way lands only after its own successful ending (client/pages/README.md).
pub(crate) fn added(pages: &Pages, client: &Client) -> Option<accounts::Added> {
    let way = way(pages, client)?;
    let page = pages.connect_model()?;
    let chosen = model_way(way);
    if chosen.kind == WayKind::SignIn {
        let added = accounts::signed_in(&page.request, client, Some(chosen.provider))?;
        return if added.provider == chosen.provider { Some(added) } else { None };
    }
    let pending = page.request.pending.as_ref()?;
    let jig_client::Outcome::Answered(jig_client::Answered::Jig(c::Answer::Done(c::Outcome::Document(outcome)))) =
        pending.outcome.as_ref()?
    else {
        return None;
    };
    match (&pending.ask, outcome) {
        (
            c::Ask::AccountChange { holder, change: c::AccountChange::AddKey { provider, join: None, .. } },
            c::DocumentOutcome::AccountKeyKept { .. },
        ) if *provider == chosen.provider && *holder == page.holder && chosen.kind == WayKind::Key => {
            Some(accounts::Added { provider: *provider, kind: WayKind::Key })
        }
        (
            c::Ask::AccountChange { change: c::AccountChange::CheckProgram { source }, .. },
            c::DocumentOutcome::AccountChanged,
        ) if page.tried == Some(way) => {
            let doc = accounts::document(client)?;
            for row in &doc.accounts {
                if row.provider == Some(chosen.provider) {
                    for login in &row.sources {
                        if login.number == *source && login.usable && login.kind == c::AccountKind::Program {
                            return Some(accounts::Added { provider: chosen.provider, kind: WayKind::Program });
                        }
                    }
                }
            }
            None
        }
        _ => None,
    }
}
