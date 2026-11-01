//! A forge's removal from the client: one request at a time (temper/docs/design/draft/playground-reset.md, 4).
use super::{Limits, Request, Served, Side, forges, wire_limits};
use jig_client::{Answered, Ask, Family, Outcome, Told, contract as c};
use skein_lib::{Queue, Token};
use temper_client_protocol_wire::{self as wire, v1 as w};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RemoveRefusal {
    NotOffered,
    Role,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum State {
    Confirming(u16),
    Removing { forge: u16, request: Token },
    Refused { forge: u16, why: RemoveRefusal },
    Removed(u16),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Answer {
    Removing,
    Refused(RemoveRefusal),
    Unknown,
}
impl Side {
    #[must_use]
    pub fn removal(&self, forge: u16) -> Option<&State> {
        let state = self.remove.as_ref()?;
        let number = match state {
            State::Confirming(forge)
            | State::Removing { forge, .. }
            | State::Refused { forge, .. }
            | State::Removed(forge) => *forge,
        };
        if forge == number { Some(state) } else { None }
    }
}
fn busy(side: &Side) -> bool {
    match side.remove {
        Some(State::Removing { .. } | State::Removed(_)) => true,
        Some(State::Confirming(_) | State::Refused { .. }) | None => false,
    }
}
pub(super) fn confirm(side: &mut Side, forge: u16) {
    refresh(side);
    if busy(side) {
        return;
    }
    side.remove = Some(State::Confirming(forge));
    side.changed();
}
pub(super) fn cancel(side: &mut Side) {
    match side.remove {
        Some(State::Confirming(_)) => {
            side.remove = None;
            side.changed();
        }
        Some(State::Removing { .. } | State::Refused { .. } | State::Removed(_)) | None => {}
    }
}
/// Shared with the welcome's unconfirmed Start over (draft 6.4).
pub fn request(side: &mut Side, forge: u16, limits: Limits, out: &mut Queue<Request>) {
    refresh(side);
    if busy(side) || side.party.is_none() || side.served == Served::OutOfDate {
        return;
    }
    let Some(ask) = ask(forge, &limits) else { return };
    side.serial = side.serial.checked_add(1).expect("forge request serial fits");
    let request = Token::new(side.serial);
    side.remove = Some(State::Removing { forge, request });
    out.push(Request::Send { request, name: None, ask });
    side.changed();
}
pub(super) fn refresh(side: &mut Side) {
    let forge = match side.remove {
        Some(State::Removed(forge)) => forge,
        Some(State::Confirming(_) | State::Removing { .. } | State::Refused { .. }) | None => return,
    };
    let rows = match &side.forges {
        forges::State::Rows(rows, ..) => rows,
        forges::State::Unknown | forges::State::Ended => return,
    };
    for row in rows {
        if row.number == forge {
            return;
        }
    }
    side.remove = None;
    side.changed();
}
pub(crate) fn told(side: &mut Side, client: &jig_client::Client, told: &Told, limits: Limits) -> bool {
    let (forge, request) = match side.remove {
        Some(State::Removing { forge, request }) => (forge, request),
        Some(State::Confirming(_) | State::Refused { .. } | State::Removed(_)) | None => return false,
    };
    let (key, result) = match told {
        Told::Answered { key, outcome } => (*key, Some(outcome)),
        Told::Pending { key } => (*key, None),
        Told::Link { .. }
        | Told::Changed { .. }
        | Told::Ended { .. }
        | Told::Text { .. }
        | Told::Page { .. }
        | Told::Kept { .. }
        | Told::Dropped { .. } => return false,
    };
    if key.asker != crate::owner::Owner::Forge.asker() || key.number != request.raw() {
        return false;
    }
    let result = match result {
        Some(Outcome::Answered(value)) => answer(value, &limits),
        Some(
            Outcome::Unknown
            | Outcome::Limit
            | Outcome::Held
            | Outcome::NotKept
            | Outcome::Unsent
            | Outcome::Full { .. }
            | Outcome::Lost
            | Outcome::Unconfirmed,
        ) => Some(Answer::Unknown),
        None => {
            let Some(pending) = client.pending(key) else { return true };
            if pending.progress != jig_client::Progress::Unknown {
                return true;
            }
            Some(Answer::Unknown)
        }
    };
    side.remove = match result {
        Some(Answer::Removing) => Some(State::Removed(forge)),
        Some(Answer::Refused(why)) => Some(State::Refused { forge, why }),
        Some(Answer::Unknown) | None => None,
    };
    refresh(side);
    side.changed();
    true
}
#[must_use]
pub fn ask(forge: u16, limits: &Limits) -> Option<Ask> {
    let limits = wire_limits(limits)?;
    let value = w::RemoveForge::new(&limits, w::RemoveForgeParts { forge }).ok()?;
    let record = w::Requesting::new(&limits, w::RequestingParts { request: w::Request::RemoveForge(value) }).ok()?;
    Some(Ask::Family {
        scope: c::Scope::Deployment,
        record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::requesting(&record) },
    })
}
#[must_use]
pub fn answer(value: &Answered, limits: &Limits) -> Option<Answer> {
    let record = match value {
        Answered::Family(record) => record,
        Answered::Jig(c::Answer::Refused { refusal, .. }) => {
            return Some(match refusal {
                c::Refusal::NotOffered => Answer::Refused(RemoveRefusal::NotOffered),
                c::Refusal::Role { .. } => Answer::Refused(RemoveRefusal::Role),
                c::Refusal::Stale(_)
                | c::Refusal::Full(_)
                | c::Refusal::Account { .. }
                | c::Refusal::AccountMove { .. }
                | c::Refusal::SignInRedirect { .. }
                | c::Refusal::NoEscalationHolder
                | c::Refusal::NameTaken
                | c::Refusal::Malformed { .. }
                | c::Refusal::NotWaiting
                | c::Refusal::NoFurther
                | c::Refusal::Authority { .. }
                | c::Refusal::Limit { .. }
                | c::Refusal::Moved { .. }
                | c::Refusal::Ended { .. }
                | c::Refusal::Unknown
                | c::Refusal::KeyConflict
                | c::Refusal::Circle { .. }
                | c::Refusal::Dependency { .. }
                | c::Refusal::Kept { .. }
                | c::Refusal::Provider
                | c::Refusal::Taken { .. }
                | c::Refusal::Llm { .. } => Answer::Unknown,
            });
        }
        Answered::Jig(c::Answer::Done(_)) => return None,
    };
    if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
        return None;
    }
    let mut reader = skein_lib::Reader::new(&record.bytes);
    let value = w::Answer::decode(&wire_limits(limits)?, &mut reader).ok()?;
    if !reader.is_empty() {
        return None;
    }
    Some(match value.into_parts().outcome {
        w::Outcome::Removing => Answer::Removing,
        w::Outcome::RemoveRefused(value) => match value.into_parts().why {
            w::RemoveRefusal::NotOffered => Answer::Refused(RemoveRefusal::NotOffered),
            w::RemoveRefusal::Role => Answer::Refused(RemoveRefusal::Role),
            w::RemoveRefusal::NoSuchForge => Answer::Removing,
        },
        w::Outcome::Unknown => Answer::Unknown,
        w::Outcome::Connecting
        | w::Outcome::Link(_)
        | w::Outcome::ConnectRefused(_)
        | w::Outcome::Page(_)
        | w::Outcome::Accepted(_)
        | w::Outcome::Adopted(_)
        | w::Outcome::CoreRefused(_)
        | w::Outcome::ForgeRefused(_)
        | w::Outcome::SettingUp
        | w::Outcome::SetUpRefused(_)
        | w::Outcome::Running
        | w::Outcome::StartRefused(_)
        | w::Outcome::StartFailed(_)
        | w::Outcome::Reset
        | w::Outcome::EndRefused(_)
        | w::Outcome::Returning => return None,
    })
}
