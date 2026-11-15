//! Each document expires independently of visibility (temper/docs/design/client/web/public.md, 2).
use super::{Doc, Heard, PublicLimits, PublicRead};
use crate::address::Public;
use jig_client_protocol_wire::public_v1 as jig;
use skein_lib::{Duration, Queue, Reader, Time};
use temper_client_protocol_wire::public_v1 as forge;

#[derive(Debug)]
pub(super) enum Document {
    Projects(jig::PublicProjects),
    Board(jig::PublicBoard),
    Card(jig::PublicCard),
    Change(jig::PublicApplication, forge::PublicChange),
    Landings(jig::PublicApplication, forge::PublicLandings),
    Landing(jig::PublicApplication, forge::PublicLanding),
}
#[derive(Debug)]
struct Snapshot {
    document: Document,
    until: Time,
    read_at: Time,
}
#[derive(Debug)]
enum State {
    Reading,
    Unavailable,
    Unknown { until: Time },
    Fresh(Snapshot),
    Retained(Snapshot),
}
#[derive(Debug)]
enum Fetch {
    Dormant,
    Ready(Time),
    Pending,
}
#[derive(Debug)]
pub(super) struct Slot {
    doc: Doc,
    state: State,
    fetch: Fetch,
    backoff: Duration,
}
const SECOND: Duration = Duration::from_secs(1);
const MINUTE: Duration = Duration::from_secs(60);
impl Slot {
    pub fn idle(doc: Doc) -> Self {
        Self { doc, state: State::Reading, fetch: Fetch::Dormant, backoff: Duration::ZERO }
    }
    pub fn new(doc: Doc) -> Self {
        Self { fetch: Fetch::Ready(Time::ZERO), ..Self::idle(doc) }
    }
    pub fn doc(&self) -> Doc {
        self.doc
    }
    pub fn document(&self) -> Option<&Document> {
        match &self.state {
            State::Fresh(snapshot) | State::Retained(snapshot) => Some(&snapshot.document),
            State::Reading | State::Unavailable | State::Unknown { .. } => None,
        }
    }
    pub fn as_of(&self, now: Time) -> Option<Duration> {
        match &self.state {
            State::Retained(snapshot) => Some(now.saturating_since(snapshot.read_at)),
            State::Fresh(_) | State::Reading | State::Unavailable | State::Unknown { .. } => None,
        }
    }
    pub fn unavailable(&self) -> bool {
        match self.state {
            State::Unavailable => true,
            State::Fresh(_) | State::Retained(_) | State::Reading | State::Unknown { .. } => false,
        }
    }
    pub fn unknown(&self) -> bool {
        match self.state {
            State::Unknown { .. } => true,
            State::Fresh(_) | State::Retained(_) | State::Reading | State::Unavailable => false,
        }
    }
    pub fn reading(&self) -> bool {
        match self.state {
            State::Reading => true,
            State::Fresh(_) | State::Retained(_) | State::Unknown { .. } | State::Unavailable => false,
        }
    }
    pub fn wake(&mut self, now: Time) {
        let expired = match &self.state {
            State::Fresh(snapshot) | State::Retained(snapshot) => now >= snapshot.until,
            State::Unknown { until } => now >= *until,
            State::Reading | State::Unavailable => true,
        };
        match self.fetch {
            Fetch::Ready(_) if expired => self.fetch = Fetch::Ready(now),
            Fetch::Ready(_) | Fetch::Pending | Fetch::Dormant => {}
        }
    }
    pub fn expire(&mut self, now: Time) {
        let state = core::mem::replace(&mut self.state, State::Unavailable);
        self.state = match state {
            State::Fresh(snapshot) | State::Retained(snapshot) if snapshot.until <= now => State::Unavailable,
            state @ (State::Fresh(_)
            | State::Retained(_)
            | State::Reading
            | State::Unavailable
            | State::Unknown { .. }) => state,
        };
    }
    pub fn deadline(&self, visible: bool) -> Option<Time> {
        let expires = match &self.state {
            State::Fresh(snapshot) | State::Retained(snapshot) => Some(snapshot.until),
            State::Reading | State::Unavailable | State::Unknown { .. } => None,
        };
        let read = match self.fetch {
            Fetch::Ready(at) if visible => Some(at),
            Fetch::Ready(_) | Fetch::Pending | Fetch::Dormant => None,
        };
        skein_lib::earliest(expires, read)
    }
    pub fn read(&mut self, now: Time, page: &Public, out: &mut Queue<PublicRead>) {
        match self.fetch {
            Fetch::Ready(at) if at <= now => {
                self.fetch = Fetch::Pending;
                out.push(self.doc.read(page));
            }
            Fetch::Ready(_) | Fetch::Pending | Fetch::Dormant => {}
        }
    }
    pub fn heard(&mut self, now: Time, heard: Heard, limits: &PublicLimits) {
        match self.fetch {
            Fetch::Pending | Fetch::Ready(_) => {}
            Fetch::Dormant => return,
        }
        match heard {
            Heard::Document { body, fresh } => match decode(self.doc, &body, limits) {
                Some(document) => {
                    self.state = State::Fresh(Snapshot { document, until: now.saturating_add(fresh), read_at: now });
                    self.backoff = Duration::ZERO;
                    self.fetch = Fetch::Ready(now.saturating_add(if fresh == Duration::ZERO { SECOND } else { fresh }));
                    self.expire(now);
                }
                None => self.refused(now, SECOND),
            },
            Heard::Unknown { fresh } => {
                self.state = State::Unknown { until: now.saturating_add(fresh) };
                self.fetch = Fetch::Ready(now.saturating_add(if fresh == Duration::ZERO { SECOND } else { fresh }));
            }
            Heard::Busy { retry } => self.refused(now, retry),
            Heard::NotServed | Heard::Failed => self.refused(now, SECOND),
        }
    }
    fn refused(&mut self, now: Time, retry: Duration) {
        let state = core::mem::replace(&mut self.state, State::Unavailable);
        self.state = match state {
            State::Fresh(snapshot) | State::Retained(snapshot) => State::Retained(snapshot),
            State::Reading | State::Unknown { .. } | State::Unavailable => State::Unavailable,
        };
        self.backoff = retry.max(SECOND).max(self.backoff.saturating_mul(2)).min(MINUTE);
        self.fetch = Fetch::Ready(now.saturating_add(self.backoff));
    }
}

fn decode(doc: Doc, bytes: &[u8], limits: &PublicLimits) -> Option<Document> {
    let mut reader = Reader::new(bytes);
    match doc {
        Doc::JigProjects => Some(Document::Projects(jig::PublicProjects::decode(&limits.jig, &mut reader).ok()?)),
        Doc::JigBoard => Some(Document::Board(jig::PublicBoard::decode(&limits.jig, &mut reader).ok()?)),
        Doc::JigCard => Some(Document::Card(jig::PublicCard::decode(&limits.jig, &mut reader).ok()?)),
        Doc::ForgeChange | Doc::ForgeLandings | Doc::ForgeLanding => {
            let application = jig::PublicApplication::decode(&limits.jig, &mut reader).ok()?;
            let mut reader = Reader::new(application.record());
            match doc {
                Doc::ForgeChange => {
                    let record = forge::PublicChange::decode(&limits.forge, &mut reader).ok()?;
                    application.tasks().get(u32::from(record.task()))?;
                    Some(Document::Change(application, record))
                }
                Doc::ForgeLandings => {
                    let record = forge::PublicLandings::decode(&limits.forge, &mut reader).ok()?;
                    if let Some(task) = record.turn() {
                        application.tasks().get(u32::from(*task))?;
                    }
                    for landing in record.landings() {
                        for task in landing.members() {
                            application.tasks().get(u32::from(*task))?;
                        }
                    }
                    for queue in record.queues() {
                        for change in queue.changes() {
                            application.tasks().get(u32::from(change.task()))?;
                            application.tasks().get(u32::from(change.goal()))?;
                        }
                    }
                    Some(Document::Landings(application, record))
                }
                Doc::ForgeLanding => {
                    let record = forge::PublicLanding::decode(&limits.forge, &mut reader).ok()?;
                    for member in record.members() {
                        application.tasks().get(u32::from(member.task()))?;
                    }
                    Some(Document::Landing(application, record))
                }
                Doc::JigProjects | Doc::JigBoard | Doc::JigCard => unreachable!("jig records decoded above"),
            }
        }
    }
}
