//! One page of an accepted installation's repositories (forge/setup/github.md, 1.9).
use super::{
    app,
    given::{self, Given, Place, Refused},
};
use crate::github::{Reply, reply, unexpected};
use crate::{
    call::Auth,
    item::Item,
    plan::{self, Event, Need, Outcome, Stage, Why},
    reachable::Reached,
};
use alloc::boxed::Box;
use core::num::NonZeroU64;
use skein_forge::github::{document, request};
use skein_lib::{Decimal, List, Queue, usize_of};

#[derive(Debug)]
enum State {
    GitHubBegin { installations: u32 },
    GitHubAccepted { installations: u32 },
    GitHubId,
    GitHubKey(NonZeroU64),
    GitHubMint(app::Kept),
    GitHubPage,
    Ended,
}
/// One picker page; the installation's token lasts only until the page answers.
#[derive(Debug)]
pub struct Reachable {
    forge: u16,
    installation: u64,
    page: u32,
    limit: u32,
    state: State,
}
impl Reachable {
    /// GitHub's picker for an accepted installation; pages start at one.
    pub fn new(given: &Given, installation: u64, page: u32) -> Result<Self, Refused> {
        given::check(given)?;
        if page == 0 {
            return Err(Refused { place: Place::Page, other: None });
        }
        Ok(Self {
            forge: given.forge,
            installation,
            page,
            limit: core::cmp::min(given.limits.reachable, skein_forge::github::pin::PAGE_CAP),
            state: State::GitHubBegin { installations: given.limits.installations },
        })
    }

    fn advance(&mut self, state: State, event: Event) -> Need {
        match state {
            State::GitHubBegin { installations } => match event.begin() {
                Ok(()) => {
                    self.state = State::GitHubAccepted { installations };
                    Need::Read(Item::Accepted)
                }
                Err(why) => self.installation_stop(why),
            },
            State::GitHubAccepted { installations } => match event.read() {
                Ok(read) => {
                    let accepted = match read {
                        Some(value) => crate::item::accepted(value.expose(), installations),
                        None => None,
                    };
                    let found = match accepted {
                        Some(ids) => ids.contains(&self.installation),
                        None => false,
                    };
                    if !found {
                        return self.installation_stop(Why::Missing);
                    }
                    self.state = State::GitHubId;
                    Need::Read(Item::AppId)
                }
                Err(why) => self.installation_stop(why),
            },
            State::GitHubId => match event.read() {
                Ok(read) => match app::id_of(read) {
                    Some(id) => {
                        self.state = State::GitHubKey(id);
                        Need::Read(Item::AppKey)
                    }
                    None => self.stop(Stage::App, b"id", Why::Missing),
                },
                Err(why) => self.stop(Stage::App, b"id", why),
            },
            State::GitHubKey(id) => match event.read() {
                Ok(Some(key)) => {
                    let app = app::Kept::new(id, key);
                    let auth = app.auth();
                    self.state = State::GitHubMint(app);
                    Need::Send {
                        request: request::Request::Mint {
                            installation: self.installation,
                            scope: request::Scope::Full,
                        }
                        .into(),
                        auth,
                    }
                }
                Ok(None) => self.stop(Stage::App, b"key", Why::Missing),
                Err(why) => self.stop(Stage::App, b"key", why),
            },
            State::GitHubMint(app) => {
                drop(app);
                self.mint(event)
            }
            State::GitHubPage => self.page(event),
            State::Ended => unreachable!("terminal machine is not advanced"),
        }
    }

    fn mint(&mut self, event: Event) -> Need {
        match reply(event) {
            Ok(Reply::Document(document::Document::Minted(minted))) => {
                self.state = State::GitHubPage;
                Need::Send {
                    request: request::Request::Reachable { page: self.page, per_page: self.limit }.into(),
                    auth: Auth::Bearer(minted.token),
                }
            }
            Ok(Reply::Unsigned) => self.stop(Stage::App, b"key", Why::KeyForm),
            Ok(Reply::Status(404)) => self.installation_stop(Why::Missing),
            Ok(other @ (Reply::Empty | Reply::Status(_) | Reply::Down | Reply::Document(_) | Reply::Rows(_))) => {
                self.installation_stop(unexpected(other))
            }
            Err(why) => self.installation_stop(why),
        }
    }

    fn page(&mut self, event: Event) -> Need {
        match reply(event) {
            Ok(Reply::Rows(rows)) => match self.rows(rows) {
                Ok(need) => need,
                Err(why) => self.installation_stop(why),
            },
            Ok(Reply::Unsigned) => self.installation_stop(Why::Unexpected),
            Ok(other @ (Reply::Empty | Reply::Status(_) | Reply::Down | Reply::Document(_))) => {
                self.installation_stop(unexpected(other))
            }
            Err(why) => self.installation_stop(why),
        }
    }

    fn stop(&mut self, stage: Stage, subject: &[u8], why: Why) -> Need {
        self.state = State::Ended;
        plan::stop(self.forge, stage, subject, why)
    }

    fn installation_stop(&mut self, why: Why) -> Need {
        let subject = Decimal::of(self.installation);
        self.stop(Stage::Installation, subject.as_bytes(), why)
    }

    fn rows(&mut self, rows: Box<[document::Row]>) -> Result<Need, Why> {
        let limit = usize_of(self.limit);
        if rows.len() > limit {
            return Err(Why::Unexpected);
        }
        let more = rows.len() == limit;
        let mut reached = List::with_capacity(u32::try_from(rows.len()).expect("bounded page"));
        for row in rows {
            let row = match row {
                document::Row::Reachable(row) => row,
                document::Row::Log(_)
                | document::Row::Installation(_)
                | document::Row::Branch(_)
                | document::Row::SkippedBranch
                | document::Row::Issue(_)
                | document::Row::Comment(_)
                | document::Row::Pull(_)
                | document::Row::Context(_)
                | document::Row::Run(_)
                | document::Row::Collaborator(_)
                | document::Row::Remark(_)
                | document::Row::Review(_)
                | document::Row::CommitMessage(_)
                | document::Row::Commit(_)
                | document::Row::Rule(_)
                | document::Row::Entry { .. }
                | document::Row::File(_) => return Err(Why::Unexpected),
            };
            reached
                .push(Reached {
                    id: row.id,
                    owner_id: row.owner,
                    owner: row.login,
                    name: row.name,
                    private: row.private,
                })
                .expect("bounded page");
        }
        self.state = State::Ended;
        Ok(Need::Done(Outcome::Reachable { rows: reached.into_boxed(), more }))
    }
}

/// Consumes one event and emits at most one need; terminal machines ignore later events.
pub fn step(machine: &mut Reachable, event: Event, out: &mut Queue<Need>) {
    let event = plan::normal(event);
    let state = core::mem::replace(&mut machine.state, State::Ended);
    match state {
        State::Ended => (),
        State::GitHubBegin { .. }
        | State::GitHubAccepted { .. }
        | State::GitHubId
        | State::GitHubKey(_)
        | State::GitHubMint(_)
        | State::GitHubPage => {
            let need = machine.advance(state, event);
            out.push(need);
        }
    }
}
