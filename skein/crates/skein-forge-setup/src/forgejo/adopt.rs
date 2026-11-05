//! Gives the bot access before its owner's first call (forge-setup.md, 4.5, steps 1 to 4).
use crate::forgejo::admin::{self, Admin};
use crate::{
    call::Auth,
    forgejo::access::{Access, Fail, Move, Shared},
    given::{self, Given, Place, Refused},
    item::Item,
    plan::{self, Event, Need, Outcome, Stage, Why},
    reconcile,
};
use alloc::boxed::Box;
use skein_forge::forgejo::request::{Repo, Request};
use skein_lib::{Queue, Secret};

#[derive(Debug)]
enum State {
    Admin(Admin),
    Access,
    Ended,
}
/// One adoption; the admin's token is held only until the plan ends.
#[derive(Debug)]
pub struct Adopt {
    forge: u16,
    admin_name: Box<[u8]>,
    state: State,
    admin: Option<Secret>,
    access: Access,
    shared: Shared,
}
impl Adopt {
    /// Refuses an invalid given or repository zero, at index zero in a list of one.
    pub fn new(given: &Given, repository: u64) -> Result<Self, Refused> {
        given::check(given)?;
        if repository == 0 {
            return Err(Refused { place: Place::Repository(0), other: None });
        }
        Ok(Self::with_access(given, Access::new(repository, &given.bot_login, Self::url(given))))
    }
    /// Refuses an invalid given; path bytes are bounded by the request's encoding.
    pub fn at(given: &Given, owner: Box<[u8]>, name: Box<[u8]>) -> Result<Self, Refused> {
        given::check(given)?;
        Ok(Self::with_access(given, Access::named(Repo { owner, name }, &given.bot_login, Self::url(given))))
    }
    fn url(given: &Given) -> Option<Box<[u8]>> {
        match given.listener {
            Some(address) => reconcile::hook_url(address, given.forge),
            None => None,
        }
    }
    fn with_access(given: &Given, access: Access) -> Self {
        Self {
            forge: given.forge,
            admin_name: given.admin_name.clone(),
            state: State::Admin(Admin::Begin),
            admin: None,
            shared: Shared { cap: None, secret: None },
            access,
        }
    }
    fn send(&self, request: Request) -> Need {
        Need::Send {
            request: request.into(),
            auth: Auth::Token(self.admin.clone().expect("admin checked before repository")),
        }
    }
    fn stop(&mut self, step: Stage, why: Why) -> Need {
        self.state = State::Ended;
        self.admin = None;
        self.shared.secret = None;
        let subject = match step {
            Stage::Admin => self.admin_name.clone(),
            Stage::Repository => self.access.subject(),
            Stage::File | Stage::Bot | Stage::Token | Stage::Org | Stage::App | Stage::Installation | Stage::State => {
                unreachable!("adoption's two stages")
            }
        };
        plan::stop(self.forge, step, &subject, why)
    }
    fn advance(&mut self, state: State, event: Event) -> Need {
        match state {
            State::Admin(mut admin) => {
                let movement = admin::step(&mut admin, event);
                self.state = State::Admin(admin);
                match movement {
                    Ok(admin::Move::Read) => Need::Read(Item::AdminToken),
                    Ok(admin::Move::Token(token)) => {
                        self.admin = Some(token);
                        self.send(Request::User)
                    }
                    Ok(admin::Move::Passed(_)) => {
                        self.state = State::Access;
                        let request = self.access.begin();
                        self.send(request)
                    }
                    Err(why) => self.stop(Stage::Admin, why),
                }
            }
            State::Access => {
                self.state = State::Access;
                match self.access.event(&mut self.shared, event) {
                    Ok(Move::Send(request)) => self.send(request),
                    Ok(Move::Read(item)) => Need::Read(item),
                    Ok(Move::Draw(count)) => Need::Draw(count),
                    Ok(Move::Write { item, bytes }) => Need::Write { item, bytes },
                    Ok(Move::Done) => {
                        self.state = State::Ended;
                        self.admin = None;
                        self.shared.secret = None;
                        Need::Done(Outcome::Nothing)
                    }
                    Err(Fail::Gone) => self.stop(Stage::Repository, Why::Missing),
                    Err(Fail::Why(why)) => self.stop(Stage::Repository, why),
                }
            }
            State::Ended => unreachable!("terminal machine is not advanced"),
        }
    }
}
/// Consumes one event and emits at most one need; terminal machines ignore later events.
pub fn step(machine: &mut Adopt, event: Event, out: &mut Queue<Need>) {
    let event = plan::normal(event);
    let state = core::mem::replace(&mut machine.state, State::Ended);
    match state {
        State::Ended => (),
        State::Admin(_) | State::Access => {
            let need = machine.advance(state, event);
            out.push(need);
        }
    }
}
