//! A person's facts (temper/docs/design/client/forge/records.md).
use super::{Limits, Request, Side, wire_limits};
use alloc::boxed::Box;
use jig_client::{Ask, Family, Handle, Latest, Link, Told, Watchable, contract as c};
use skein_lib::{List, Queue, Wall};
use temper_client_protocol_wire::{self as wire, v1 as w};
use temper_protocol_reaches::{Subject, subject_from, subject_record};

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
/// Facts the party keeps (temper/docs/design/client/forge/records.md).
pub enum PersonFact {
    /// "Skip" on the project's "To begin".
    ToBegin { project: u32 },
    /// The person started `goal` from `chat`'s document.
    GoalFrom { project: u32, chat: u64, goal: u64, at: u64 },
    /// The person kept asking for this reach's subject.
    KeepAsking { project: u32, subject: Subject, at: Wall },
}

/// An ended watch stays ended until the next sign-in.
#[derive(Debug)]
pub enum State {
    Unknown,
    Rows(Box<[PersonFact]>),
    Ended,
}

#[must_use]
pub fn watch(limits: &Limits) -> Option<Watchable> {
    let bounds = wire_limits(limits)?;
    let record = w::Watching::new(&bounds, w::WatchingParts { watch: w::Watch::Dismissed }).ok()?;
    Some(Watchable::Family {
        scope: c::Scope::Own,
        window: c::Window { rows: 0, after: None },
        record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::watching(&record) },
    })
}

/// Save a fact in its project's scope.
#[must_use]
pub fn ask(what: PersonFact, limits: &Limits) -> Option<Ask> {
    let bounds = wire_limits(limits)?;
    let (project, what) = match what {
        PersonFact::ToBegin { project } => {
            (project, w::PersonFact::ToBegin(w::ToBegin::new(&bounds, w::ToBeginParts { project }).ok()?))
        }
        PersonFact::GoalFrom { project, chat, goal, at } => (
            project,
            w::PersonFact::GoalFrom(w::GoalFrom::new(&bounds, w::GoalFromParts { project, chat, goal, at }).ok()?),
        ),
        PersonFact::KeepAsking { project, subject, at } => (
            project,
            w::PersonFact::KeepAsking(
                w::KeepAsking::new(
                    &bounds,
                    w::KeepAskingParts { project, subject: subject_record(subject), at: at.as_nanos() },
                )
                .ok()?,
            ),
        ),
    };
    let dismiss = w::Dismiss::new(&bounds, w::DismissParts { what }).ok()?;
    let record = w::Requesting::new(&bounds, w::RequestingParts { request: w::Request::Dismiss(dismiss) }).ok()?;
    Some(Ask::Family {
        scope: c::Scope::Project(project),
        record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::requesting(&record) },
    })
}

impl Side {
    /// The person's dismissal rows have been delivered.
    #[must_use]
    pub fn dismissals_delivered(&self) -> bool {
        match self.dismissed {
            State::Rows(_) => true,
            State::Unknown | State::Ended => false,
        }
    }

    /// The newest held Keep asking of this project's subject.
    #[must_use]
    pub fn keep_asking(&self, project: u32, subject: Subject) -> Option<Wall> {
        let rows = match &self.dismissed {
            State::Rows(rows) => rows.as_ref(),
            State::Unknown | State::Ended => return None,
        };
        let mut newest = None;
        for fact in rows {
            match *fact {
                PersonFact::KeepAsking { project: held_project, subject: held_subject, at }
                    if held_project == project && held_subject == subject =>
                {
                    newest = Some(match newest {
                        Some(held) => Wall::max(held, at),
                        None => at,
                    });
                }
                PersonFact::KeepAsking { .. } | PersonFact::ToBegin { .. } | PersonFact::GoalFrom { .. } => {}
            }
        }
        newest
    }

    /// The dismissals have been delivered or their watch has ended.
    #[must_use]
    pub fn settled(&self) -> bool {
        match self.dismissed {
            State::Unknown => false,
            State::Rows(_) | State::Ended => true,
        }
    }

    /// `None` until the first document, or once the watch is refused or ended.
    #[must_use]
    pub fn dismissed(&self, what: PersonFact) -> Option<bool> {
        match &self.dismissed {
            State::Rows(rows) => Some(rows.contains(&what)),
            State::Unknown | State::Ended => None,
        }
    }
    /// Goals started from this chat, in the party's key order.
    #[expect(
        clippy::disallowed_methods,
        reason = "the person facts contract exposes a filtered iterator over the party's rows"
    )]
    pub fn goals_from(&self, project: u32, chat: u64) -> impl Iterator<Item = (u64, u64)> {
        let rows = match &self.dismissed {
            State::Rows(rows) => rows.as_ref(),
            State::Unknown | State::Ended => &[],
        };
        rows.iter().filter_map(move |fact| match *fact {
            PersonFact::GoalFrom { project: held_project, chat: held_chat, goal, at }
                if held_project == project && held_chat == chat =>
            {
                Some((goal, at))
            }
            PersonFact::ToBegin { .. } | PersonFact::GoalFrom { .. } | PersonFact::KeepAsking { .. } => None,
        })
    }
}

pub(crate) fn handle() -> Handle {
    Handle { asker: crate::owner::Owner::Forge.asker(), number: super::me::DISMISSED }
}

pub(crate) fn configure(side: &mut Side, limits: Limits, out: &mut Queue<Request>) {
    if side.link == Link::SignedOut || side.link == Link::OutOfDate {
        end(side, out);
        side.dismissed = State::Unknown;
        side.changed();
        return;
    }
    if side.party.is_some()
        && !side.dismissed_open
        && let State::Unknown = side.dismissed
    {
        if let Some(subject) = watch(&limits) {
            side.dismissed_open = true;
            out.push(Request::Client { asked: jig_client::Event::Watch { handle: handle(), subject } });
        } else {
            side.dismissed = State::Ended;
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
            if !side.dismissed_open {
                return true;
            }
            side.dismissed = match client.document(*changed) {
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
            if let State::Ended = side.dismissed {
                end(side, out);
            }
        }
        Told::Ended { handle: ended, .. } if *ended == handle() => {
            side.dismissed = State::Ended;
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

fn rows(record: &Family, limits: Limits) -> Option<Box<[PersonFact]>> {
    if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
        return None;
    }
    let document = wire::decode_dismissals(&record.bytes, &wire_limits(&limits)?)?;
    let mut rows = List::with_capacity(wire::DISMISSED);
    for what in document.into_parts().dismissed.into_boxed() {
        let what = match what {
            w::PersonFact::KeepAsking(value) => PersonFact::KeepAsking {
                project: value.project(),
                subject: subject_from(value.subject()),
                at: Wall::from_nanos(value.at()),
            },
            w::PersonFact::ToBegin(value) => PersonFact::ToBegin { project: value.project() },
            w::PersonFact::GoalFrom(value) => PersonFact::GoalFrom {
                project: value.project(),
                chat: value.chat(),
                goal: value.goal(),
                at: value.at(),
            },
        };
        rows.push(what).expect("wire bounds dismissal rows");
    }
    Some(rows.into_boxed())
}

fn end(side: &mut Side, out: &mut Queue<Request>) {
    if side.dismissed_open {
        side.dismissed_open = false;
        out.push(Request::Client { asked: jig_client::Event::Unwatch { handle: handle() } });
    }
}

pub(crate) enum Answer {
    Dismissed,
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
        w::Outcome::Dismissed => Answer::Dismissed,
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
        | w::Outcome::Away
        | w::Outcome::Tried
        | w::Outcome::Filed(_)
        | w::Outcome::Handed
        | w::Outcome::Triaged
        | w::Outcome::FindingRefused(_)
        | w::Outcome::NotTheirs
        | w::Outcome::Overturned
        | w::Outcome::OverturnRefused(_)
        | w::Outcome::EndRefused(_)
        | w::Outcome::MakeKept
        | w::Outcome::MakeRefused(_) => Answer::Refused,
    })
}
