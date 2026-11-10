//! Connect or replace a pasted App while preserving identity (forge/setup/github.md, 1.8).
use super::{
    app::{self, Kept},
    checks::{self, Lacking},
    given::{self, Given, Refused},
    hook::{self, Hook},
    key,
    write::Write,
};
use crate::github::{Reply, reply, unexpected};
use crate::{
    call::Auth,
    item::{self, Item},
    plan::{self, Event, Need, Outcome, Stage, Why},
};
use alloc::boxed::Box;
use core::num::NonZeroU64;
use skein_forge::github::{
    document::{AppOwner, Document},
    request::Request,
};
use skein_lib::{Decimal, Queue, Secret};

#[derive(Debug)]
struct Checked {
    key: Secret,
    slug: Box<[u8]>,
    bot: u64,
    lacking: Box<[Lacking]>,
    owner: Option<AppOwner>,
}
#[derive(Debug)]
enum State {
    Begin,
    Pasted,
    Id(Secret),
    User(Secret),
    App { key: Secret, user: Option<NonZeroU64> },
    Bot { key: Secret, slug: Box<[u8]>, lacking: Box<[Lacking]>, owner: Option<AppOwner>, user: Option<NonZeroU64> },
    Hook { child: Hook, checked: Checked },
    Writing { write: Write, slug: Box<[u8]>, lacking: Box<[Lacking]>, owner: Option<AppOwner> },
    Ended,
}
/// Reads Pasted once, proves both identities before hook repair, and waits for
/// every durable identity write before Done(Lacking) (forge/setup/github.md, 1.8, forge/setup/README.md, 4.4).
#[derive(Debug)]
pub struct Paste {
    given: Given,
    app: NonZeroU64,
    state: State,
}
impl Paste {
    /// Checks the given values before copying them; no callback state is needed (5.8).
    pub fn new(given: &Given, app: NonZeroU64) -> Result<Self, Refused> {
        given::check(given)?;
        Ok(Self { given: given.clone(), app, state: State::Begin })
    }
    fn stop(&self, subject: &[u8], why: Why) -> Need {
        plan::stop(self.given.forge, Stage::App, subject, why)
    }
    fn hook(&mut self, mut child: Hook, checked: Checked, event: Event) -> Need {
        let mut out = Queue::uncapped();
        hook::step(&mut child, event, &mut out);
        match out.pop() {
            Some(hook::Progress::Pending(need @ Need::Stop(_))) => need,
            Some(hook::Progress::Pending(
                need @ (Need::Read(_) | Need::Write { .. } | Need::Send { .. } | Need::Draw(_)),
            )) => {
                self.state = State::Hook { child, checked };
                need
            }
            Some(hook::Progress::Complete) => {
                let write = Write::new(self.app.get(), checked.slug.clone(), checked.key, checked.bot);
                self.writing(write, checked.slug, checked.lacking, checked.owner, Event::Begin)
            }
            Some(hook::Progress::Pending(Need::Done(_))) | None => self.stop(&checked.slug, Why::Unexpected),
        }
    }
    fn writing(
        &mut self,
        mut write: Write,
        slug: Box<[u8]>,
        lacking: Box<[Lacking]>,
        owner: Option<AppOwner>,
        event: Event,
    ) -> Need {
        match write.advance(event) {
            Ok(Some(need)) => {
                self.state = State::Writing { write, slug, lacking, owner };
                need
            }
            Ok(None) => Need::Done(Outcome::Lacking { lacking, owner }),
            Err(why) => self.stop(&slug, why),
        }
    }
    fn proved(&mut self, key: Secret, user: Option<NonZeroU64>, event: Event) -> Need {
        let subject = Decimal::of(self.app.get());
        let app = match reply(event) {
            Ok(Reply::Document(Document::App(app))) => app,
            Ok(Reply::Unsigned) => return self.stop(b"key", Why::KeyForm),
            Ok(reply @ (Reply::Empty | Reply::Status(_) | Reply::Down | Reply::Document(_) | Reply::Rows(_))) => {
                return self.stop(subject.as_bytes(), unexpected(reply));
            }
            Err(why) => return self.stop(subject.as_bytes(), why),
        };
        if app.id != self.app.get() {
            return self.stop(subject.as_bytes(), Why::OtherApp);
        }
        if item::app_slug(&app.slug, self.given.limits.name_bytes).is_none() {
            return self.stop(subject.as_bytes(), Why::Unexpected);
        }
        let lacking = checks::lacks(&app);
        let request = Request::BotUser { slug: app.slug.clone() }.into();
        self.state = State::Bot { key, slug: app.slug, lacking, owner: app.owner, user };
        Need::Send { request, auth: Auth::None }
    }
    fn advance(&mut self, state: State, event: Event) -> Need {
        match state {
            State::Begin => match event.begin() {
                Ok(()) => {
                    self.state = State::Pasted;
                    Need::Read(Item::Pasted)
                }
                Err(why) => self.stop(b"key", why),
            },
            State::Pasted => match event.read() {
                Ok(None) => self.stop(b"key", Why::Missing),
                Ok(Some(pem)) => {
                    if pem.is_empty() {
                        return self.stop(b"key", Why::Missing);
                    }
                    let key = key::pem(&pem, self.given.limits.key_bytes);
                    drop(pem);
                    let Some(key) = key else { return self.stop(b"key", Why::KeyForm) };
                    self.state = State::Id(key);
                    Need::Read(Item::AppId)
                }
                Err(why) => self.stop(b"key", why),
            },
            State::Id(key) => match event.read() {
                Ok(read) => {
                    match optional_id(read) {
                        Ok(Some(id)) => {
                            if id != self.app {
                                return self.stop(Decimal::of(self.app.get()).as_bytes(), Why::OtherApp);
                            }
                        }
                        Ok(None) => (),
                        Err(why) => return self.stop(b"id", why),
                    }
                    self.state = State::User(key);
                    Need::Read(Item::User)
                }
                Err(why) => self.stop(b"id", why),
            },
            State::User(key) => match event.read() {
                Ok(read) => {
                    let user = match optional_id(read) {
                        Ok(user) => user,
                        Err(why) => return self.stop(b"user", why),
                    };
                    let auth = Kept::new(self.app, key.clone()).auth();
                    self.state = State::App { key, user };
                    Need::Send { request: Request::App.into(), auth }
                }
                Err(why) => self.stop(b"user", why),
            },
            State::App { key, user } => self.proved(key, user, event),
            State::Bot { key, slug, lacking, owner, user } => match app::bot(event) {
                Ok(bot) => {
                    if let Some(user) = user
                        && user != bot
                    {
                        return self.stop(&slug, Why::OtherUser);
                    }
                    let checked = Checked { key, slug, bot: bot.get(), lacking, owner };
                    let child = Hook::new(
                        self.given.forge,
                        Kept::new(self.app, checked.key.clone()),
                        &checked.slug,
                        self.given.repair_hook,
                    );
                    self.hook(child, checked, Event::Begin)
                }
                Err(why) => self.stop(&slug, why),
            },
            State::Hook { child, checked } => self.hook(child, checked, event),
            State::Writing { write, slug, lacking, owner } => self.writing(write, slug, lacking, owner, event),
            State::Ended => unreachable!("ended paste is not advanced"),
        }
    }
}
/// A kept identity is optional, but malformed bytes are Unexpected (forge/setup/github.md, 1.8).
fn optional_id(read: Option<Secret>) -> Result<Option<NonZeroU64>, Why> {
    match read {
        Some(value) => match item::user(value.expose()) {
            Some(id) => Ok(Some(id)),
            None => Err(Why::Unexpected),
        },
        None => Ok(None),
    }
}
/// Consumes one event and emits its next need, emitting at most one need.
/// Terminal paths release credentials; ended machines are silent (5.8).
pub fn step(machine: &mut Paste, event: Event, out: &mut Queue<Need>) {
    let state = core::mem::replace(&mut machine.state, State::Ended);
    match state {
        State::Ended => (),
        State::Begin
        | State::Pasted
        | State::Id(_)
        | State::User(_)
        | State::App { .. }
        | State::Bot { .. }
        | State::Hook { .. }
        | State::Writing { .. } => out.push(machine.advance(state, event)),
    }
}
/// Releases every credential and interrupts the current step once (5.8, 6.3).
pub(crate) fn halt(machine: &mut Paste, why: Why, out: &mut Queue<Need>) {
    let state = core::mem::replace(&mut machine.state, State::Ended);
    let need = match state {
        State::Begin | State::Pasted => machine.stop(b"key", why),
        State::Id(_) => machine.stop(b"id", why),
        State::User(_) => machine.stop(b"user", why),
        State::App { .. } => machine.stop(Decimal::of(machine.app.get()).as_bytes(), why),
        State::Bot { slug, .. } | State::Writing { slug, .. } => machine.stop(&slug, why),
        State::Hook { checked, .. } => machine.stop(&checked.slug, why),
        State::Ended => return,
    };
    out.push(need);
}
