//! One forge reconciled from its answers, with one need outstanding (forge-setup.md, 4.3).
mod hooks;
mod orgs;
mod tokens;
mod walk;
pub use crate::plan::{Event, Failure, Need, Outcome, Reply, Stage, Why};
use crate::{
    call::Auth,
    given::{self, Given, Refused},
    item::Item,
    plan,
};
use alloc::boxed::Box;
pub use hooks::hook_url;
use skein_forge::forgejo::{document::Document, request::Request};
use skein_lib::{Queue, Secret, Writer, bytes};
pub(crate) use tokens::{reply, unexpected};
#[derive(Debug)]
enum State {
    Begin,
    AdminFile,
    Admin,
    Bot,
    Draw,
    Create,
    Tokens,
    Id(u64),
    Written,
    Orgs,
    Access,
    Ended,
}
/// One forge's state; credentials remain beside the machine's requests.
#[derive(Debug)]
pub struct Reconcile {
    forge: u16,
    bot_login: Box<[u8]>,
    bot_email: Box<[u8]>,
    admin_name: Box<[u8]>,
    api_name: Box<[u8]>,
    git_name: Box<[u8]>,
    marker: Box<[u8]>,
    admin: Option<Secret>,
    state: State,
    tokens: Option<tokens::Tokens>,
    orgs: Option<orgs::Orgs>,
    access: Option<walk::Walk>,
    shared: crate::forgejo::access::Shared,
    url: Option<Box<[u8]>>,
}
impl Reconcile {
    /// Starts one entry.
    pub fn new(given: &Given) -> Result<Self, Refused> {
        given::check(given)?;
        let url = match given.listener {
            Some(address) => hook_url(address, given.forge),
            None => None,
        };
        Ok(Self {
            forge: given.forge,
            bot_login: given.bot_login.clone(),
            bot_email: given.bot_email.clone(),
            admin_name: given.admin_name.clone(),
            api_name: given.api_name.clone(),
            git_name: given.git_name.clone(),
            marker: given::marker(&given.word),
            admin: None,
            state: State::Begin,
            tokens: None,
            orgs: orgs::Orgs::new(given),
            access: walk::Walk::new(given, url.clone()),
            shared: crate::forgejo::access::Shared { cap: None, secret: None },
            url,
        })
    }
    fn send(&self, request: Request) -> Need {
        Need::Send { request: request.into(), auth: Auth::Token(self.admin.clone().expect("admin checked before bot")) }
    }
    fn stop(&mut self, stage: Stage, why: Why) -> Need {
        let subject = match stage {
            Stage::Admin | Stage::App | Stage::Installation | Stage::State => self.admin_name.clone(),
            Stage::Bot | Stage::Token | Stage::Org | Stage::Repository | Stage::File => self.bot_login.clone(),
        };
        self.stop_named(stage, &subject, why)
    }
    fn id(&mut self, user: skein_forge::forgejo::document::User) -> Need {
        if user.id == 0 {
            return self.stop(Stage::Bot, Why::Unexpected);
        }
        if !marked(&user.full_name, &self.marker) {
            return self.stop(Stage::Bot, Why::Unmarked);
        }
        if user.is_admin {
            return self.stop(Stage::Bot, Why::BotIsAdmin);
        }
        self.tokens = Some(tokens::Tokens::new(user.id, &self.api_name, &self.git_name));
        self.state = State::Tokens;
        Need::Read(Item::ApiToken)
    }
    fn advance(&mut self, state: State, event: Event) -> Need {
        match state {
            State::Begin => match event {
                Event::Begin => {
                    self.state = State::AdminFile;
                    Need::Read(Item::AdminToken)
                }
                Event::Read(_) | Event::Reply(_) | Event::Written | Event::Drawn(_) => {
                    self.stop(Stage::Admin, Why::Unexpected)
                }
            },
            State::AdminFile => match event {
                Event::Read(Some(token)) if !token.is_empty() => {
                    self.admin = Some(token);
                    self.state = State::Admin;
                    self.send(Request::User)
                }
                Event::Read(_) => self.stop(Stage::Admin, Why::Missing),
                Event::Begin | Event::Reply(_) | Event::Written | Event::Drawn(_) => {
                    self.stop(Stage::Admin, Why::Unexpected)
                }
            },
            State::Admin => self.admin(event),
            State::Bot => self.bot(event),
            State::Draw => self.draw(event),
            State::Create => match account(event) {
                Ok(user) => self.id(user),
                Err(why) => self.stop(Stage::Bot, why),
            },
            State::Tokens => {
                self.state = State::Tokens;
                tokens::advance(self, event)
            }
            State::Id(id) => match event {
                Event::Read(value) => {
                    let bytes = id.to_le_bytes();
                    if match value {
                        Some(value) => value.expose() == bytes,
                        None => false,
                    } {
                        self.start_orgs()
                    } else {
                        self.state = State::Written;
                        Need::Write { item: Item::User, bytes: Secret::new(bytes::copy_of(&bytes)) }
                    }
                }
                Event::Begin | Event::Reply(_) | Event::Written | Event::Drawn(_) => {
                    self.stop(Stage::Bot, Why::Unexpected)
                }
            },
            State::Written => match event {
                Event::Written => self.start_orgs(),
                Event::Begin | Event::Read(_) | Event::Reply(_) | Event::Drawn(_) => {
                    self.stop(Stage::Bot, Why::Unexpected)
                }
            },
            State::Orgs => orgs::advance(self, event),
            State::Access => walk::advance(self, event),
            State::Ended => unreachable!("terminal machine is not advanced"),
        }
    }
    fn admin(&mut self, event: Event) -> Need {
        match account(event) {
            Ok(user) => {
                if !user.is_admin {
                    return self.stop(Stage::Admin, Why::NotAdmin);
                }
                self.state = State::Bot;
                self.send(Request::UserOf { login: self.bot_login.clone() })
            }
            Err(why) => self.stop(Stage::Admin, why),
        }
    }
    fn bot(&mut self, event: Event) -> Need {
        match account(event) {
            Ok(user) => self.id(user),
            Err(Why::Refused(404)) => {
                self.state = State::Draw;
                Need::Draw(32)
            }
            Err(why) => self.stop(Stage::Bot, why),
        }
    }
    fn draw(&mut self, event: Event) -> Need {
        match event {
            Event::Drawn(value) if value.len() == 32 => {
                self.state = State::Create;
                self.send(Request::CreateUser {
                    login: self.bot_login.clone(),
                    email: self.bot_email.clone(),
                    full_name: bytes::copy_of(&self.marker),
                    password: hex(&value),
                    must_change_password: false,
                })
            }
            Event::Begin | Event::Read(_) | Event::Reply(_) | Event::Written | Event::Drawn(_) => {
                self.stop(Stage::Bot, Why::Unexpected)
            }
        }
    }
}
pub(crate) fn account(event: Event) -> Result<skein_forge::forgejo::document::User, Why> {
    match event {
        Event::Reply(reply) => match reply {
            Reply::Document(document) => match document {
                Document::User(user) => Ok(user),
                Document::Token(_)
                | Document::Org(_)
                | Document::CreatedFile { .. }
                | Document::CreatedRepository(_)
                | Document::Hook(_)
                | Document::Team(_)
                | Document::ApiSettings(_)
                | Document::Repository(_)
                | Document::Search { .. }
                | Document::RepoSearch { .. }
                | Document::Listed { .. }
                | Document::Issue(_)
                | Document::Comment(_)
                | Document::Status(_)
                | Document::Review(_)
                | Document::Pull(_)
                | Document::CombinedStatus(_)
                | Document::Branch(_)
                | Document::Protection(_)
                | Document::Permission(_)
                | Document::Compare(_)
                | Document::Tree { .. }
                | Document::Runs { .. }
                | Document::Run(_)
                | Document::Job(_)
                | Document::Error(_) => Err(Why::Unexpected),
            },
            Reply::Status(status) => Err(Why::Refused(status)),
            Reply::Down => Err(Why::Down),
            Reply::Unsigned => Err(Why::KeyForm),
            Reply::Rows(_) | Reply::Empty | Reply::GithubDocument(_) | Reply::GithubRows(_) => Err(Why::Unexpected),
        },
        Event::Begin | Event::Read(_) | Event::Written | Event::Drawn(_) => Err(Why::Unexpected),
    }
}
/// Consumes one event and emits at most one need; terminal machines ignore later events.
pub fn step(machine: &mut Reconcile, event: Event, out: &mut Queue<Need>) {
    let event = normal(event);
    let state = core::mem::replace(&mut machine.state, State::Ended);
    match state {
        State::Ended => (),
        State::Begin
        | State::AdminFile
        | State::Admin
        | State::Bot
        | State::Draw
        | State::Create
        | State::Tokens
        | State::Id(_)
        | State::Written
        | State::Orgs
        | State::Access => {
            let need = machine.advance(state, event);
            out.push(need);
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

pub(crate) fn hex(value: &Secret) -> Secret {
    let mut out = Writer::new(64);
    for byte in value.expose() {
        for nibble in [byte >> 4_u8, byte & 15] {
            out.put(&[bytes::hex_digit(nibble)]).expect("32 bytes as 64 hex digits");
        }
    }
    Secret::new(out.finish())
}
impl Reconcile {
    fn start_orgs(&mut self) -> Need {
        match self.orgs.as_ref() {
            Some(orgs) => {
                let name = orgs.name().into();
                self.state = State::Orgs;
                self.send(Request::Org { name })
            }
            None => self.start_access(),
        }
    }
    fn start_access(&mut self) -> Need {
        match self.access.as_mut() {
            Some(walk) => {
                let request = walk.begin();
                self.state = State::Access;
                self.send(request)
            }
            None => self.finish(),
        }
    }
    fn finish(&mut self) -> Need {
        self.admin = None;
        self.shared.secret = None;
        self.state = State::Ended;
        Need::Done(Outcome::Nothing)
    }
    fn stop_named(&mut self, stage: Stage, name: &[u8], why: Why) -> Need {
        self.state = State::Ended;
        self.admin = None;
        self.shared.secret = None;
        plan::stop(self.forge, stage, name, why)
    }
}

fn marked(full_name: &[u8], marker: &[u8]) -> bool {
    for (at, candidate) in full_name.windows(marker.len()).enumerate() {
        if candidate != marker {
            continue;
        }
        let before = match at.checked_sub(1) {
            Some(index) => full_name.get(index),
            None => None,
        };
        let after = match at.checked_add(marker.len()) {
            Some(index) => full_name.get(index),
            None => None,
        };
        if edge(before) && edge(after) {
            return true;
        }
    }
    false
}
fn edge(byte: Option<&u8>) -> bool {
    match byte {
        None => true,
        Some(byte) => !(byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-'),
    }
}

/// Stops a live plan at its step with `why`; an ended plan pushes nothing.
pub(crate) fn halt(machine: &mut Reconcile, why: Why, out: &mut Queue<Need>) {
    let event = match machine.state {
        State::Begin => Event::Written,
        State::AdminFile
        | State::Admin
        | State::Bot
        | State::Draw
        | State::Create
        | State::Tokens
        | State::Id(_)
        | State::Written
        | State::Orgs
        | State::Access
        | State::Ended => Event::Begin,
    };
    let mut needs = Queue::uncapped();
    step(machine, event, &mut needs);
    if let Some(Need::Stop(mut failure)) = needs.pop() {
        failure.why = why;
        out.push(Need::Stop(failure));
    }
}
