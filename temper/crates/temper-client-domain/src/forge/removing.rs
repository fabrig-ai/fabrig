//! Removing a forge whole, confirmed on its row (temper/docs/design/draft/playground-reset.md, 4).
use super::{Limits, Request, Served, Side, forges, listed, wire_limits};
use jig_client::{Answered, Ask, Family, contract as c};
use skein_lib::{Queue, Reader, RequestProgress, Token};
use temper_client_protocol_wire::{self as wire, v1 as w};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum RemoveAnswer {
    Removing,
    NoSuchForge,
    Playground,
    Unknown,
    EntranceRefused(c::EntranceRefusal),
}
#[derive(Debug)]
pub struct Removal {
    pub forge: u16,
    pub confirming: bool,
    pub start_over: bool,
    pub request: Option<Token>,
    pub progress: Option<RequestProgress>,
    pub answer: Option<RemoveAnswer>,
}
#[must_use]
pub fn ask(forge: u16, limits: &Limits) -> Option<Ask> {
    if forge == 0 {
        return None;
    }
    let limits = wire_limits(limits)?;
    let request = w::Request::RemoveForge(w::RemoveForge::new(&limits, w::RemoveForgeParts { forge }).ok()?);
    let record = w::Requesting::new(&limits, w::RequestingParts { request }).ok()?;
    Some(Ask::Family {
        scope: c::Scope::Deployment,
        record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::requesting(&record) },
    })
}
#[must_use]
pub fn answer(answer: &Answered, limits: &Limits) -> Option<RemoveAnswer> {
    let record = match answer {
        Answered::Jig(c::Answer::Refused { refusal, .. }) => {
            return Some(RemoveAnswer::EntranceRefused(c::EntranceRefusal::new(refusal.clone()).ok()?));
        }
        Answered::Jig(c::Answer::Done(_)) => return None,
        Answered::Family(record) => record,
    };
    if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
        return None;
    }
    let mut reader = Reader::new(&record.bytes);
    let answer = w::Answer::decode(&wire_limits(limits)?, &mut reader).ok()?;
    if !reader.is_empty() {
        return None;
    }
    match answer.into_parts().outcome {
        w::Outcome::Removing => Some(RemoveAnswer::Removing),
        w::Outcome::ConnectRefused(refused) if *refused.why() == w::ForgeRefusal::NoSuchForge => {
            Some(RemoveAnswer::NoSuchForge)
        }
        w::Outcome::Playground => Some(RemoveAnswer::Playground),
        w::Outcome::Unknown => Some(RemoveAnswer::Unknown),
        w::Outcome::Adopted(_)
        | w::Outcome::CoreRefused(_)
        | w::Outcome::ForgeRefused(_)
        | w::Outcome::SettingUp
        | w::Outcome::SetUpRefused(_)
        | w::Outcome::Running
        | w::Outcome::StartRefused(_)
        | w::Outcome::StartFailed(_)
        | w::Outcome::Returning
        | w::Outcome::Connecting
        | w::Outcome::Link(_)
        | w::Outcome::ConnectRefused(_)
        | w::Outcome::Page(_)
        | w::Outcome::Accepted(_)
        | w::Outcome::Made(_)
        | w::Outcome::Full
        | w::Outcome::NotOffered
        | w::Outcome::Reset => None,
    }
}
fn listed(side: &Side, forge: u16) -> bool {
    for row in &listed::rows(&side.forges, side.demo()) {
        if row.number == forge {
            return true;
        }
    }
    false
}
/// Accepted removal still runs until the document drops its row.
pub(crate) fn busy(side: &Side) -> bool {
    match &side.removal {
        Some(removal) => removal.progress.is_some() || removal.answer == Some(RemoveAnswer::Removing),
        None => false,
    }
}
pub(crate) fn confirm(side: &mut Side, forge: u16, start_over: bool) {
    if busy(side) || !listed(side, forge) {
        return;
    }
    side.removal = Some(Removal { forge, start_over, confirming: true, request: None, progress: None, answer: None });
    side.changed();
}
pub(crate) fn keep(side: &mut Side) {
    if let Some(removal) = &side.removal
        && removal.confirming
        && removal.request.is_none()
    {
        side.removal = None;
        side.changed();
    }
}
pub(crate) fn send(side: &mut Side, limits: Limits, forge: u16, out: &mut Queue<Request>) {
    if side.party.is_none() || side.served == Served::OutOfDate || busy(side) || !listed(side, forge) {
        return;
    }
    let Some(removal) = &side.removal else { return };
    if removal.forge != forge || !removal.confirming {
        return;
    }
    let Some(ask) = ask(forge, &limits) else { return };
    side.serial = side.serial.checked_add(1).expect("forge request serial fits");
    let request = Token::new(side.serial);
    let start_over = removal.start_over;
    side.removal = Some(Removal {
        forge,
        start_over,
        confirming: false,
        request: Some(request),
        progress: Some(RequestProgress::Parked),
        answer: None,
    });
    out.push(Request::Send { request, name: None, ask });
    side.changed();
}
pub(crate) fn prune(side: &mut Side) {
    if let forges::State::Rows(_) = side.forges
        && let Some(removal) = &side.removal
        && removal.progress.is_none()
        && !listed(side, removal.forge)
    {
        side.removal = None;
        side.changed();
    }
}
pub(crate) fn told(side: &mut Side, client: &jig_client::Client, value: &jig_client::Told, limits: Limits) -> bool {
    use jig_client::{Outcome, Progress, Told};
    let Some(removal) = &mut side.removal else { return false };
    if removal.progress.is_none() {
        return false;
    }
    let Some(request) = removal.request else { return false };
    let key = jig_client::Key { asker: crate::owner::Owner::Forge.asker(), number: request.raw() };
    let answer = match value {
        Told::Pending { key: pending } if *pending == key => {
            let Some(pending) = client.pending(key) else { return true };
            let progress = match pending.progress {
                Progress::InFlight => RequestProgress::InFlight,
                Progress::Retrying { .. } => RequestProgress::Retrying,
                Progress::NotServed | Progress::Parked => RequestProgress::Parked,
                Progress::Unknown => RequestProgress::Unknown,
            };
            if progress != RequestProgress::Unknown {
                removal.progress =
                    Some(if side.served == Served::OutOfDate { RequestProgress::Parked } else { progress });
                side.changed();
                return true;
            }
            RemoveAnswer::Unknown
        }
        Told::Answered { key: answered, outcome } if *answered == key => match outcome {
            Outcome::Answered(answered) => answer(answered, &limits).unwrap_or(RemoveAnswer::Unknown),
            Outcome::Unknown
            | Outcome::Limit
            | Outcome::Held
            | Outcome::NotKept
            | Outcome::Unsent
            | Outcome::Full { .. }
            | Outcome::Lost
            | Outcome::Unconfirmed => RemoveAnswer::Unknown,
        },
        Told::Link { .. }
        | Told::Changed { .. }
        | Told::Ended { .. }
        | Told::Page { .. }
        | Told::Text { .. }
        | Told::Pending { .. }
        | Told::Answered { .. }
        | Told::Kept { .. }
        | Told::Dropped { .. } => return false,
    };
    removal.progress = None;
    removal.answer = Some(answer);
    prune(side);
    side.changed();
    true
}
