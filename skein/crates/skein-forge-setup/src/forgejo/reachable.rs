//! One page of a forge's repositories for the picker (forge/setup/forgejo.md, 1.8).
use crate::forgejo::admin::{self, Admin};
use crate::forgejo::{Reply, reply, unexpected};
use crate::{
    call::Auth,
    given::{self, Given, Place, Refused},
    item::Item,
    plan::{self, Event, Need, Outcome, Stage, Why},
};
use alloc::boxed::Box;
use skein_forge::forgejo::{
    document::{Document, Row},
    request::{Page, Request},
};
use skein_lib::{Decimal, List, Queue, Secret, bytes, usize_of};

/// One repository of a picker's page (forge/setup/forgejo.md, 1.8, step 4).
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Reached {
    pub id: u64,
    pub owner_id: u64,
    /// The owner's login.
    pub owner: Box<[u8]>,
    pub name: Box<[u8]>,
    pub private: bool,
}

#[derive(Debug)]
enum State {
    Begin,
    ApiRead,
    Admin(Admin),
    Settings,
    Search,
    Ended,
}

/// Whose token reads the page (forge/setup/forgejo.md, 1.8, step 1).
#[derive(Debug)]
enum Token {
    Admin,
    Api,
}

/// One picker page; credentials live only as long as their phases need them.
#[derive(Debug)]
pub struct Reachable {
    forge: u16,
    admin_name: Box<[u8]>,
    installation: Option<u64>,
    page: u32,
    limit: u32,
    state: State,
    token: Option<Secret>,
    listing: Token,
}

impl Reachable {
    /// Refuses invalid given values or a page of zero; pages start at one.
    pub fn new(given: &Given, installation: Option<u64>, page: u32) -> Result<Self, Refused> {
        given::check(given)?;
        if page == 0 {
            return Err(Refused { place: Place::Page, other: None });
        }
        Ok(Self {
            forge: given.forge,
            admin_name: given.admin_name.clone(),
            installation,
            page,
            limit: given.limits.reachable,
            state: State::Begin,
            token: None,
            listing: Token::Admin,
        })
    }

    fn send(&self, request: Request) -> Need {
        Need::Send {
            request: request.into(),
            auth: Auth::Token(self.token.clone().expect("listing token read before requests")),
        }
    }

    fn stop(&mut self, step: Stage, subject: Box<[u8]>, why: Why) -> Need {
        self.state = State::Ended;
        self.token = None;
        plan::stop(self.forge, step, &subject, why)
    }

    fn advance(&mut self, state: State, event: Event) -> Need {
        match state {
            State::Begin => match event.begin() {
                Ok(()) => {
                    if let Some(id) = self.installation {
                        let subject = bytes::copy_of(Decimal::of(id).as_bytes());
                        return self.stop(Stage::Installation, subject, Why::Unexpected);
                    }
                    self.admin(Admin::Begin, Event::Begin)
                }
                Err(why) => self.stop(Stage::Admin, self.admin_name.clone(), why),
            },
            State::Admin(admin) => self.admin(admin, event),
            State::ApiRead => match event.read() {
                Ok(Some(token)) => {
                    self.token = Some(token);
                    self.state = State::Settings;
                    self.send(Request::ApiSettings)
                }
                Ok(None) => self.stop(Stage::Token, bytes::copy_of(b"api"), Why::Missing),
                Err(why) => self.stop(Stage::Token, bytes::copy_of(b"api"), why),
            },
            State::Settings | State::Search => match self.repositories(state, event) {
                Ok(need) => need,
                Err(why) => {
                    let subject = match self.listing {
                        Token::Admin => self.admin_name.clone(),
                        Token::Api => bytes::copy_of(b"api"),
                    };
                    self.stop(Stage::Token, subject, why)
                }
            },
            State::Ended => unreachable!("terminal machine is not advanced"),
        }
    }

    fn admin(&mut self, mut admin: Admin, event: Event) -> Need {
        let reading = match admin {
            Admin::Read => true,
            Admin::Begin | Admin::Check => false,
        };
        let movement = admin::step(&mut admin, event);
        self.state = State::Admin(admin);
        match movement {
            Ok(admin::Move::Read) => Need::Read(Item::AdminToken),
            Ok(admin::Move::Token(token)) => {
                self.token = Some(token);
                self.send(Request::User)
            }
            Ok(admin::Move::Passed(_)) => {
                self.state = State::Settings;
                self.send(Request::ApiSettings)
            }
            Err(Why::Missing) if reading => {
                self.listing = Token::Api;
                self.state = State::ApiRead;
                Need::Read(Item::ApiToken)
            }
            Err(why) => self.stop(Stage::Admin, self.admin_name.clone(), why),
        }
    }

    fn repositories(&mut self, state: State, event: Event) -> Result<Need, Why> {
        match state {
            State::Settings => match reply(event)? {
                Reply::Document(document) => {
                    let Document::ApiSettings(settings) = *document else { return Err(Why::Unexpected) };
                    if settings.max_response_items == 0 {
                        return Err(Why::Unexpected);
                    }
                    self.limit = core::cmp::min(self.limit, settings.max_response_items);
                    self.state = State::Search;
                    let page = Page { page: Some(self.page), limit: Some(self.limit) };
                    Ok(self.send(match self.listing {
                        Token::Admin => Request::RepoSearch { page },
                        Token::Api => Request::UserRepos { page },
                    }))
                }
                other @ (Reply::Rows(_) | Reply::Empty | Reply::Status(_) | Reply::Down) => Err(unexpected(other)),
            },
            State::Search => self.rows(reply(event)?),
            State::Begin | State::ApiRead | State::Admin(_) | State::Ended => {
                unreachable!("repository phases only")
            }
        }
    }

    fn rows(&mut self, answer: Reply) -> Result<Need, Why> {
        let rows = match answer {
            Reply::Rows(rows) => rows,
            other @ (Reply::Document(_) | Reply::Empty | Reply::Status(_) | Reply::Down) => {
                return Err(unexpected(other));
            }
        };
        let limit = usize_of(self.limit);
        if rows.len() > limit {
            return Err(Why::Unexpected);
        }
        let more = rows.len() == limit;
        let mut reached = List::with_capacity(u32::try_from(rows.len()).expect("bounded page"));
        for row in rows {
            let row = match (row, &self.listing) {
                (Row::Found(row), Token::Admin) | (Row::Reachable(row), Token::Api) => row,
                (Row::Found(_), Token::Api)
                | (Row::Reachable(_), Token::Admin)
                | (
                    Row::Hook(_)
                    | Row::Token(_)
                    | Row::Team(_)
                    | Row::User(_)
                    | Row::Issue(_)
                    | Row::Comment(_)
                    | Row::Status(_)
                    | Row::Review(_)
                    | Row::ReviewComment(_)
                    | Row::Branch(_)
                    | Row::Protection(_)
                    | Row::Pull(_)
                    | Row::Entry { .. }
                    | Row::File(_)
                    | Row::Commit(_)
                    | Row::Artifact(_)
                    | Row::CommitMessage(_)
                    | Row::TreeEntry(_)
                    | Row::Run(_)
                    | Row::Job(_)
                    | Row::Step(_),
                    Token::Admin | Token::Api,
                ) => return Err(Why::Unexpected),
            };
            if match self.listing {
                Token::Admin => true,
                Token::Api => row.push,
            } {
                reached
                    .push(Reached {
                        id: row.repository,
                        owner_id: row.owner,
                        owner: row.login,
                        name: row.name,
                        private: row.private,
                    })
                    .expect("bounded page");
            }
        }
        self.state = State::Ended;
        self.token = None;
        Ok(Need::Done(Outcome::Reachable { rows: reached.into_boxed(), more }))
    }
}

/// Consumes one event and emits at most one need; terminal machines ignore later events.
pub fn step(machine: &mut Reachable, event: Event, out: &mut Queue<Need>) {
    let event = plan::normal(event);
    let state = core::mem::replace(&mut machine.state, State::Ended);
    match state {
        State::Ended => (),
        State::Begin | State::ApiRead | State::Admin(_) | State::Settings | State::Search => {
            let need = machine.advance(state, event);
            out.push(need);
        }
    }
}
