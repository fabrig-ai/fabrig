//! A person's Away facts (temper/docs/design/client/forge/records.md).
use super::{Limits, Request, Side, wire_limits};
use alloc::boxed::Box;
use jig_client::{Ask, Family, Handle, Latest, Link, Told, Watchable, contract as c};
use skein_lib::{List, Queue, Wall};
use temper_client_protocol_wire::{self as wire, v1 as w};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct AwayRow {
    pub project: u32,
    pub since: Wall,
    pub until: Option<Wall>,
    pub landed: u64,
    pub off: Option<Wall>,
}
impl AwayRow {
    #[must_use]
    pub fn on_at(&self, wall: Wall) -> bool {
        self.off.is_none()
            && match self.until {
                Some(until) => wall < until,
                None => true,
            }
    }
    /// When the spell ended: off, or until once past.
    #[must_use]
    pub fn ended(&self, wall: Wall) -> Option<Wall> {
        match self.off {
            Some(off) => Some(off),
            None => match self.until {
                Some(until) if until <= wall => Some(until),
                Some(_) | None => None,
            },
        }
    }
}

/// The first 08:00 after wall in the person's zone (temper/docs/design/client/pages/README.md, Away).
#[must_use]
pub fn next_eight(wall: Wall, zone_minutes: i32) -> Wall {
    const MINUTE: i128 = 60_000_000_000;
    const DAY: i128 = 86_400_000_000_000;
    const EIGHT: i128 = 28_800_000_000_000;
    let zone = i128::from(zone_minutes).checked_mul(MINUTE).expect("i32 minutes fit i128");
    let local = i128::from(wall.as_nanos()).checked_add(zone).expect("wall and zone fit i128");
    let today = local
        .div_euclid(DAY)
        .checked_mul(DAY)
        .expect("local day fits i128")
        .checked_add(EIGHT)
        .expect("local eight fits i128");
    let next = if today > local { today } else { today.checked_add(DAY).expect("next day fits i128") };
    let universal = next.checked_sub(zone).expect("universal eight fits i128");
    Wall::from_nanos(u64::try_from(universal).unwrap_or(u64::MAX))
}

/// An ended watch stays ended until the next sign-in.
#[derive(Debug)]
pub enum State {
    Unknown,
    Rows(Box<[AwayRow]>),
    Ended,
}

#[must_use]
pub fn watch(limits: &Limits) -> Option<Watchable> {
    let bounds = wire_limits(limits)?;
    let record = w::Watching::new(&bounds, w::WatchingParts { watch: w::Watch::Away }).ok()?;
    Some(Watchable::Family {
        scope: c::Scope::Own,
        window: c::Window { rows: 0, after: None },
        record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::watching(&record) },
    })
}

/// Set the project's Away row in the project's scope.
#[must_use]
pub fn ask(project: u32, on: bool, until: Option<Wall>, limits: &Limits) -> Option<Ask> {
    let bounds = wire_limits(limits)?;
    let until = match until {
        Some(wall) => Some(wall.as_nanos()),
        None => None,
    };
    let value = w::SetAway::new(&bounds, w::SetAwayParts { project, on, until }).ok()?;
    let record = w::Requesting::new(&bounds, w::RequestingParts { request: w::Request::SetAway(value) }).ok()?;
    Some(Ask::Family {
        scope: c::Scope::Project(project),
        record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::requesting(&record) },
    })
}
impl Side {
    /// None before delivery, for a missing row, or after the watch ends.
    #[must_use]
    pub fn away(&self, project: u32) -> Option<AwayRow> {
        match &self.away {
            State::Rows(rows) => {
                for row in rows {
                    if row.project == project {
                        return Some(*row);
                    }
                }
                None
            }
            State::Unknown | State::Ended => None,
        }
    }
    pub(crate) fn away_available(&self) -> bool {
        match self.away {
            State::Rows(_) => true,
            State::Unknown | State::Ended => false,
        }
    }
}

pub(crate) fn handle() -> Handle {
    Handle { asker: crate::owner::Owner::Forge.asker(), number: super::me::AWAY }
}

pub(crate) fn configure(side: &mut Side, limits: Limits, out: &mut Queue<Request>) {
    if side.link == Link::SignedOut || side.link == Link::OutOfDate {
        end(side, out);
        side.away = State::Unknown;
        side.changed();
        return;
    }
    if side.party.is_some()
        && !side.away_open
        && let State::Unknown = side.away
    {
        if let Some(subject) = watch(&limits) {
            side.away_open = true;
            out.push(Request::Client { asked: jig_client::Event::Watch { handle: handle(), subject } });
        } else {
            side.away = State::Ended;
        }
        side.changed();
    }
}

pub(crate) fn told(
    side: &mut Side,
    client: &jig_client::Client,
    told: &Told,
    limits: Limits,
    out: &mut Queue<Request>,
) -> bool {
    match told {
        Told::Changed { handle: changed } if *changed == handle() => {
            if !side.away_open {
                return true;
            }
            side.away = match client.document(*changed) {
                Some(document) if document.state == jig_client::State::Ended => State::Ended,
                Some(document) => match document.latest {
                    Some(Latest::Family(record)) => match rows(record, limits) {
                        Some(rows) => State::Rows(rows),
                        None => State::Ended,
                    },
                    Some(Latest::Jig(_)) => State::Ended,
                    None => State::Unknown,
                },
                None => State::Unknown,
            };
            if let State::Ended = side.away {
                end(side, out);
            }
        }
        Told::Ended { handle: ended, .. } if *ended == handle() => {
            side.away = State::Ended;
            end(side, out);
        }
        Told::Link { .. }
        | Told::Changed { .. }
        | Told::Ended { .. }
        | Told::Page { .. }
        | Told::Text { .. }
        | Told::Pending { .. }
        | Told::Answered { .. }
        | Told::Kept { .. }
        | Told::Dropped { .. } => return false,
    }
    side.changed();
    true
}

fn rows(record: &Family, limits: Limits) -> Option<Box<[AwayRow]>> {
    if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
        return None;
    }
    let document = wire::decode_aways(&record.bytes, &wire_limits(&limits)?)?;
    let mut rows = List::with_capacity(w::CEILINGS.aways_rows);
    for row in document.into_parts().rows.into_boxed() {
        let row = row.into_parts();
        let until = match row.until {
            Some(wall) => Some(Wall::from_nanos(wall)),
            None => None,
        };
        let off = match row.off {
            Some(wall) => Some(Wall::from_nanos(wall)),
            None => None,
        };
        rows.push(AwayRow { project: row.project, since: Wall::from_nanos(row.since), until, landed: row.landed, off })
            .expect("wire bounds Away rows");
    }
    Some(rows.into_boxed())
}

fn end(side: &mut Side, out: &mut Queue<Request>) {
    if side.away_open {
        side.away_open = false;
        out.push(Request::Client { asked: jig_client::Event::Unwatch { handle: handle() } });
    }
}

pub(crate) enum Answer {
    Away,
    Unknown,
    Refused,
}

pub(crate) fn answer(outcome: &jig_client::Outcome, limits: Limits) -> Answer {
    match outcome {
        jig_client::Outcome::Unknown => Answer::Unknown,
        jig_client::Outcome::Answered(jig_client::Answered::Family(record)) => match answered(record, limits) {
            Some(answer) => answer,
            None => Answer::Refused,
        },
        jig_client::Outcome::Answered(jig_client::Answered::Jig(_))
        | jig_client::Outcome::NotKept
        | jig_client::Outcome::Limit
        | jig_client::Outcome::Unconfirmed
        | jig_client::Outcome::Held
        | jig_client::Outcome::Unsent
        | jig_client::Outcome::Full { .. }
        | jig_client::Outcome::Lost => Answer::Refused,
    }
}

fn answered(record: &Family, limits: Limits) -> Option<Answer> {
    if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
        return None;
    }
    let mut reader = skein_lib::Reader::new(&record.bytes);
    let answer = w::Answer::decode(&wire_limits(&limits)?, &mut reader).ok()?;
    if !reader.is_empty() {
        return None;
    }
    Some(match answer.into_parts().outcome {
        w::Outcome::Away => Answer::Away,
        w::Outcome::Unknown => Answer::Unknown,
        w::Outcome::Looked
        | w::Outcome::Page(_)
        | w::Outcome::Accepted(_)
        | w::Outcome::Connecting
        | w::Outcome::Link(_)
        | w::Outcome::ConnectRefused(_)
        | w::Outcome::Adopted(_)
        | w::Outcome::CoreRefused(_)
        | w::Outcome::ForgeRefused(_)
        | w::Outcome::Running
        | w::Outcome::StartRefused(_)
        | w::Outcome::StartFailed(_)
        | w::Outcome::SettingUp
        | w::Outcome::SetUpRefused(_)
        | w::Outcome::Removed
        | w::Outcome::RemoveRefused(_)
        | w::Outcome::Reset
        | w::Outcome::Dismissed
        | w::Outcome::Tried
        | w::Outcome::Follow(_)
        | w::Outcome::NotTheirs
        | w::Outcome::Overturned
        | w::Outcome::OverturnRefused(_)
        | w::Outcome::EndRefused(_)
        | w::Outcome::MakeKept
        | w::Outcome::MakeRefused(_) => Answer::Refused,
    })
}
