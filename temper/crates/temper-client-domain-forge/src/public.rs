//! Anonymous page reading and freshness (temper/docs/design/client/web/public.md, 2).
use crate::address::Public;
use alloc::boxed::Box;
use jig_client_protocol_wire::public_v1 as jig;
use skein_lib::{Duration, Queue, Time};
use temper_client_protocol_wire::public_v1 as forge;

mod document;
use document::{Document, Slot};

/// Receiving bounds for the two public wire families, including jig's envelope.
#[derive(Clone, Copy, Debug)]
pub struct PublicLimits {
    pub jig: jig::Limits,
    pub forge: forge::Limits,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Doc {
    JigProjects,
    JigBoard,
    JigCard,
    ForgeChange,
    ForgeLandings,
    ForgeLanding,
}
#[derive(Debug)]
pub enum Heard {
    Document { body: Box<[u8]>, fresh: Duration },
    Unknown { fresh: Duration },
    Busy { retry: Duration },
    NotServed,
    Failed,
}
#[derive(Debug)]
pub enum PublicEvent {
    Open { page: Public },
    Heard { doc: Doc, heard: Heard },
    Visible { visible: bool },
    Due,
}
#[derive(Debug, PartialEq, Eq)]
pub struct PublicRead {
    pub doc: Doc,
    pub owner: u16,
    pub version: u16,
    pub kind: u16,
    pub ids: [Option<u64>; 2],
}
/// A document's current record or state; application identifiers stay in their envelope.
#[derive(Debug, PartialEq, Eq)]
pub enum Shown<'a> {
    Reading,
    Unavailable,
    Unknown,
    Projects {
        projects: &'a jig::PublicProjects,
        as_of: Option<Duration>,
    },
    Board {
        board: &'a jig::PublicBoard,
        landings: Option<(&'a jig::PublicApplication, &'a forge::PublicLandings)>,
        reading: bool,
        unavailable: bool,
        as_of: Option<Duration>,
        companion_as_of: Option<Duration>,
    },
    Task {
        card: &'a jig::PublicCard,
        change: Option<(&'a jig::PublicApplication, &'a forge::PublicChange)>,
        reading: bool,
        unavailable: bool,
        as_of: Option<Duration>,
        companion_as_of: Option<Duration>,
    },
    Landings {
        application: &'a jig::PublicApplication,
        landings: &'a forge::PublicLandings,
        as_of: Option<Duration>,
    },
    Landing {
        application: &'a jig::PublicApplication,
        landing: &'a forge::PublicLanding,
        as_of: Option<Duration>,
    },
}
/// One page's documents and their monotonic freshness and retry timers.
#[derive(Debug)]
pub struct Reader {
    limits: PublicLimits,
    page: Public,
    main: Slot,
    second: Option<Slot>,
    visible: bool,
    now: Time,
}
impl Reader {
    #[must_use]
    pub fn new(limits: &PublicLimits) -> Self {
        Self {
            limits: *limits,
            page: Public::Projects,
            main: Slot::idle(Doc::JigProjects),
            second: None,
            visible: true,
            now: Time::ZERO,
        }
    }
    pub fn step(&mut self, now: Time, event: PublicEvent, out: &mut Queue<PublicRead>) {
        self.now = now;
        self.main.expire(now);
        if let Some(second) = &mut self.second {
            second.expire(now);
        }
        match event {
            PublicEvent::Open { page } => {
                let (main, second) = match page {
                    Public::Projects => (Slot::new(Doc::JigProjects), None),
                    Public::Board { .. } => (Slot::new(Doc::JigBoard), Some(Slot::new(Doc::ForgeLandings))),
                    Public::Task { .. } => (Slot::new(Doc::JigCard), Some(Slot::new(Doc::ForgeChange))),
                    Public::Landings { .. } => (Slot::new(Doc::ForgeLandings), None),
                    Public::Landing { .. } => (Slot::new(Doc::ForgeLanding), None),
                };
                self.page = page;
                self.main = main;
                self.second = second;
            }
            PublicEvent::Heard { doc, heard } => {
                if self.main.doc() == doc {
                    self.main.heard(now, heard, &self.limits);
                    if self.main.unknown()
                        && let Some(second) = &mut self.second
                    {
                        *second = Slot::new(second.doc());
                    }
                } else if !self.main.unknown()
                    && let Some(second) = &mut self.second
                    && second.doc() == doc
                {
                    second.heard(now, heard, &self.limits);
                }
            }
            PublicEvent::Visible { visible } => {
                if visible && !self.visible {
                    self.main.wake(now);
                    if let Some(second) = &mut self.second {
                        second.wake(now);
                    }
                }
                self.visible = visible;
            }
            PublicEvent::Due => {}
        }
        if self.visible {
            self.main.read(now, &self.page, out);
            if !self.main.unknown()
                && let Some(second) = &mut self.second
            {
                second.read(now, &self.page, out);
            }
        }
    }
    #[must_use]
    pub fn deadline(&self) -> Option<Time> {
        let mut next = self.main.deadline(self.visible);
        if let Some(second) = &self.second {
            next = skein_lib::earliest(next, second.deadline(self.visible && !self.main.unknown()));
        }
        next
    }
    #[must_use]
    pub const fn page(&self) -> &Public {
        &self.page
    }
    #[must_use]
    pub fn shown(&self) -> Shown<'_> {
        let second = match &self.second {
            Some(slot) => slot.document(),
            None => None,
        };
        let as_of = self.main.as_of(self.now);
        let (unavailable, reading, companion_as_of) = match &self.second {
            Some(slot) => (slot.unavailable(), slot.reading(), slot.as_of(self.now)),
            None => (false, false, None),
        };
        match self.main.document() {
            Some(Document::Projects(projects)) => Shown::Projects { projects, as_of },
            Some(Document::Board(board)) => Shown::Board {
                board,
                as_of,
                companion_as_of,
                reading,
                unavailable,
                landings: match second {
                    Some(Document::Landings(a, d)) => Some((a, d)),
                    Some(
                        Document::Projects(_)
                        | Document::Board(_)
                        | Document::Card(_)
                        | Document::Change(_, _)
                        | Document::Landing(_, _),
                    )
                    | None => None,
                },
            },
            Some(Document::Card(card)) => Shown::Task {
                card,
                as_of,
                companion_as_of,
                reading,
                unavailable,
                change: match second {
                    Some(Document::Change(a, d)) => Some((a, d)),
                    Some(
                        Document::Projects(_)
                        | Document::Board(_)
                        | Document::Card(_)
                        | Document::Landings(_, _)
                        | Document::Landing(_, _),
                    )
                    | None => None,
                },
            },
            Some(Document::Landings(application, landings)) => Shown::Landings { application, landings, as_of },
            Some(Document::Landing(application, landing)) => Shown::Landing { application, landing, as_of },
            Some(Document::Change(_, _)) => unreachable!("a change accompanies a card"),
            None => {
                if self.main.unknown() {
                    Shown::Unknown
                } else if self.main.reading() {
                    Shown::Reading
                } else {
                    Shown::Unavailable
                }
            }
        }
    }
}

impl Doc {
    fn read(self, page: &Public) -> PublicRead {
        let ids = match *page {
            Public::Projects => [None, None],
            Public::Board { project } | Public::Landings { project } => [Some(project), None],
            Public::Task { project, task } | Public::Landing { project, task } => [Some(project), Some(task)],
        };
        let (owner, version, kind) = match self {
            Self::JigProjects => {
                (0, jig_client_protocol_wire::PUBLIC_VERSION, jig_client_protocol_wire::PUBLIC_PROJECTS)
            }
            Self::JigBoard => (0, jig_client_protocol_wire::PUBLIC_VERSION, jig_client_protocol_wire::PUBLIC_BOARD),
            Self::JigCard => (0, jig_client_protocol_wire::PUBLIC_VERSION, jig_client_protocol_wire::PUBLIC_CARD),
            Self::ForgeChange => (forge_owner(), forge::PUBLIC_VERSION, forge::PUBLIC_CHANGE),
            Self::ForgeLandings => (forge_owner(), forge::PUBLIC_VERSION, forge::PUBLIC_LANDINGS),
            Self::ForgeLanding => (forge_owner(), forge::PUBLIC_VERSION, forge::PUBLIC_LANDING),
        };
        PublicRead { doc: self, owner, version, kind, ids }
    }
}
fn forge_owner() -> u16 {
    temper_client_protocol_wire::number(temper_client_protocol_wire::Owner::Forge)
        .checked_add(1)
        .expect("forge owner fits")
}
