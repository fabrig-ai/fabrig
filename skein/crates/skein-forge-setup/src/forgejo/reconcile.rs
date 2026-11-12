//! One forge reconciled from its answers, with one need outstanding (forge/setup/forgejo.md, 1.3).
mod hooks;
mod orgs;
mod tokens;
mod walk;
use crate::forgejo::Reply as Answer;
use crate::forgejo::admin::{self, Admin, Move};
use crate::forgejo::{reply, unexpected};
pub use crate::plan::{Event, Failure, Need, Outcome, Reply, Stage, Why};
use crate::{
    call::Auth,
    given::{self, Given, Refused},
    item::Item,
    plan,
};
use alloc::boxed::Box;
use core::num::NonZeroU64;
pub use hooks::hook_url;
use skein_forge::forgejo::{document::Document, request::Request};
use skein_lib::{Queue, Secret, Writer, bytes};
#[derive(Debug)]
enum State {
    Admin(Admin),
    Bot,
    Draw,
    Create,
    Tokens,
    Id(u64),
    Written,
    Orgs,
    Access,
    Person(Box<[u8]>),
    Ended,
}
/// One forge's state; credentials remain beside the machine's requests.
#[derive(Debug)]
pub struct Reconcile {
    forge: u16,
    person: Option<Box<[u8]>>,
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
            person: given.person.clone(),
            bot_login: given.bot_login.clone(),
            bot_email: given.bot_email.clone(),
            admin_name: given.admin_name.clone(),
            api_name: given.api_name.clone(),
            git_name: given.git_name.clone(),
            marker: given::marker(&given.word),
            admin: None,
            state: State::Admin(Admin::Begin),
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
            Stage::Person | Stage::Bot | Stage::Token | Stage::Org | Stage::Repository | Stage::File => {
                self.bot_login.clone()
            }
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
            State::Admin(admin) => self.admin(admin, event),
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
            State::Id(id) => match event.read() {
                Ok(value) => {
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
                Err(why) => self.stop(Stage::Bot, why),
            },
            State::Written => match event.written() {
                Ok(()) => self.start_orgs(),
                Err(why) => self.stop(Stage::Bot, why),
            },
            State::Orgs => orgs::advance(self, event),
            State::Access => walk::advance(self, event),
            State::Person(login) => match account(event) {
                Ok(user) => match NonZeroU64::new(user.id) {
                    Some(id) => self.done(Outcome::Person { id: Some(id) }),
                    None => self.stop_named(Stage::Person, &login, Why::Unexpected),
                },
                Err(Why::Refused(404)) => self.done(Outcome::Person { id: None }),
                Err(why) => self.stop_named(Stage::Person, &login, why),
            },
            State::Ended => unreachable!("terminal machine is not advanced"),
        }
    }
    fn admin(&mut self, mut admin: Admin, event: Event) -> Need {
        let movement = admin::step(&mut admin, event);
        self.state = State::Admin(admin);
        match movement {
            Ok(Move::Read) => Need::Read(Item::AdminToken),
            Ok(Move::Token(token)) => {
                self.admin = Some(token);
                self.send(Request::User)
            }
            Ok(Move::Passed(_)) => {
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
        match event.drawn() {
            Ok(value) if value.len() == 32 => {
                self.state = State::Create;
                self.send(Request::CreateUser {
                    login: self.bot_login.clone(),
                    email: self.bot_email.clone(),
                    full_name: bytes::copy_of(&self.marker),
                    password: hex(&value),
                    must_change_password: false,
                })
            }
            Ok(_) => self.stop(Stage::Bot, Why::Unexpected),
            Err(why) => self.stop(Stage::Bot, why),
        }
    }
}
pub(crate) fn account(event: Event) -> Result<skein_forge::forgejo::document::User, Why> {
    match reply(event)? {
        Answer::Document(document) => {
                    let Document::User(user) = *document else { return Err(Why::Unexpected) };
                    Ok(user)
                }
        other @ (Answer::Rows(_) | Answer::Empty | Answer::Status(_) | Answer::Down) => {
            Err(unexpected(other))
        }
    }
}
/// Consumes one event and emits at most one need; terminal machines ignore later events.
pub fn step(machine: &mut Reconcile, event: Event, out: &mut Queue<Need>) {
    let event = plan::normal(event);
    let state = core::mem::replace(&mut machine.state, State::Ended);
    match state {
        State::Ended => (),
        State::Admin(_)
        | State::Bot
        | State::Draw
        | State::Create
        | State::Tokens
        | State::Id(_)
        | State::Written
        | State::Orgs
        | State::Access
        | State::Person(_) => {
            let need = machine.advance(state, event);
            out.push(need);
        }
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
        match self.person.take() {
            Some(login) => {
                self.state = State::Person(login.clone());
                self.send(Request::UserOf { login })
            }
            None => self.done(Outcome::Nothing),
        }
    }
    fn done(&mut self, outcome: Outcome) -> Need {
        self.admin = None;
        self.shared.secret = None;
        self.state = State::Ended;
        Need::Done(outcome)
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
        State::Admin(Admin::Begin) => Event::Written,
        State::Admin(Admin::Read | Admin::Check)
        | State::Bot
        | State::Draw
        | State::Create
        | State::Tokens
        | State::Id(_)
        | State::Written
        | State::Orgs
        | State::Access
        | State::Person(_)
        | State::Ended => Event::Begin,
    };
    let mut needs = Queue::uncapped();
    step(machine, event, &mut needs);
    if let Some(Need::Stop(mut failure)) = needs.pop() {
        failure.why = why;
        out.push(Need::Stop(failure));
    }
}
