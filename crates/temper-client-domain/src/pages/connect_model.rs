//! Connect one more model account; shared with the welcome's Model step.
use super::{
    Pages, Request as Output,
    accounts::{self, Broken, Way, request::Request},
};
use crate::{Address, ModelWay, Step, WayKind};
use alloc::boxed::Box;
use jig_client::{Client, contract as c};
use skein_lib::{Queue, RequestKey};
#[derive(Debug)]
pub struct ConnectModel {
    pub holder: c::AccountHolder,
    pub program: Option<Box<[u8]>>,
    pub request: Request,
    /// Whether the scripted model was serving when this connection was sent.
    pub playground: bool,
    pub elsewhere: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Act {
    Holder { holder: c::AccountHolder },
    SignIn { name: RequestKey, way: Way },
    SignInElsewhere { name: RequestKey, way: Way },
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
        | Address::Inbox { .. }
        | Address::Chats { .. }
        | Address::Accounts
        | Address::Welcome { .. }
        | Address::Chat { .. }
        | Address::Task { .. }
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
#[must_use]
pub fn address(current: &Address, way: Option<ModelWay>) -> Address {
    match current {
        Address::Welcome { .. } => Address::Welcome { step: Step::Model { way } },
        Address::Forges
        | Address::ConnectForge { .. }
        | Address::Inbox { .. }
        | Address::Chats { .. }
        | Address::Accounts
        | Address::ConnectModel { .. }
        | Address::Chat { .. }
        | Address::Task { .. }
        | Address::Adopt { .. } => Address::ConnectModel { way },
    }
}
#[must_use]
pub fn way(pages: &Pages, client: &Client) -> Option<Way> {
    let chosen = match pages.address() {
        Address::ConnectModel { way } | Address::Welcome { step: Step::Model { way } } => (*way)?,
        Address::Forges
        | Address::ConnectForge { .. }
        | Address::Inbox { .. }
        | Address::Chats { .. }
        | Address::Accounts
        | Address::Welcome { .. }
        | Address::Chat { .. }
        | Address::Task { .. }
        | Address::Adopt { .. } => return None,
    };
    let me = accounts::current_me(client)?;
    let page = pages.connect_model()?;
    for way in &accounts::ways(&me.providers, pages.demo_provider, page.holder) {
        if model_way(*way).provider == chosen.provider && model_way(*way).kind == chosen.kind {
            return Some(*way);
        }
    }
    None
}
impl ConnectModel {
    #[must_use]
    pub const fn elsewhere(&self) -> bool {
        self.elsewhere
    }
    pub(crate) fn changing_way(&mut self, current: &Address, next: &Address) {
        if let Address::ConnectModel { way: Some(old) } | Address::Welcome { step: Step::Model { way: Some(old) } } =
            current
            && old.kind == WayKind::SignIn
        {
            let next = if let Address::ConnectModel { way } | Address::Welcome { step: Step::Model { way } } = next {
                *way
            } else {
                None
            };
            if next != Some(*old) {
                self.leave_way();
            }
        }
    }
    fn leave_way(&mut self) {
        self.elsewhere = false;
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
        reason = "the page resolves its way against borrowed identity, address and demo facts"
    )]
    pub(crate) fn act(
        &mut self,
        client: &Client,
        current: &Address,
        chosen: Option<Way>,
        demo: Option<u32>,
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
        let elsewhere = match &act {
            Act::SignInElsewhere { .. } => true,
            Act::SignIn { .. }
            | Act::Holder { .. }
            | Act::ProgramName { .. }
            | Act::Submit { .. }
            | Act::Finish { .. }
            | Act::Cancel => false,
        };
        let Some(me) = accounts::current_me(client) else { return };
        match act {
            Act::Holder { holder } => {
                if holder == c::AccountHolder::Deployment && me.deployment_role != Some(c::HeldRole::Owner) {
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
            Act::Cancel => self.request.cancel(),
            Act::SignIn { name, way } | Act::SignInElsewhere { name, way } => {
                if model_way(way).kind != WayKind::SignIn
                    || !accounts::ways(&me.providers, demo, self.holder).as_slice().contains(&way)
                {
                    return;
                }
                self.elsewhere = elsewhere;
                self.request.send(
                    name,
                    c::Ask::SignIn { holder: c::AccountHolder::Own, provider: way.provider() },
                    false,
                    serial,
                    out,
                );
                out.push(Output::Show { address: address(current, Some(model_way(way))) });
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
                        let name = self.program_name(client);
                        if !c::valid_program_name(&name) {
                            self.request.broken = Some(Broken::ProgramName);
                            return;
                        }
                        c::AccountChange::DeclareProgram { provider, join: None, name }
                    }
                    Way::SignIn { .. } => return,
                };
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
            holder: c::AccountHolder::Own,
            program: None,
            request: Request::default(),
            playground: false,
            elsewhere: false,
        }
    }
}
