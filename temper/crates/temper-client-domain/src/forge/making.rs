//! Making a forge by kind, then continuing by number (temper/docs/design/draft/playground-reset.md, 3).
use super::{Limits, Request, Served, Side, forges, github, wire_limits};
use crate::{Address, Choice, Step};
use alloc::boxed::Box;
use jig_client::{Answered, Ask, Family, contract as c};
use skein_lib::{Queue, Reader, RequestProgress, Token};
use temper_client_protocol_wire::{self as wire, v1 as w};

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MakeChoice {
    Here,
    GitHub,
    Forgejo { api: Box<[u8]>, git: Box<[u8]>, web: Option<Box<[u8]>> },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Then {
    SetUp,
    SetUpAtOnce,
    ConnectManifest { back: Box<[u8]> },
    Open,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum MakeAnswer {
    Made { forge: u16 },
    Full,
    NotOffered,
    Unknown,
    EntranceRefused(c::EntranceRefusal),
}
#[derive(Debug)]
pub struct Making {
    pub choice: MakeChoice,
    pub then: Then,
    pub request: Token,
    pub progress: Option<RequestProgress>,
    pub answer: Option<MakeAnswer>,
}
#[derive(Debug)]
pub struct Draft {
    pub api: Box<[u8]>,
    pub git: Box<[u8]>,
    pub web: Box<[u8]>,
    pub written: [u64; 3],
    pub checked: bool,
}
impl Draft {
    pub(crate) fn new() -> Self {
        Self { api: Box::default(), git: Box::default(), web: Box::default(), written: [0; 3], checked: false }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Field {
    Api,
    Git,
    Web,
}
impl Field {
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Api => 0,
            Self::Git => 1,
            Self::Web => 2,
        }
    }
}
#[must_use]
pub fn ask(choice: MakeChoice, limits: &Limits) -> Option<Ask> {
    let limits = wire_limits(limits)?;
    let choice = match choice {
        MakeChoice::Here => w::ForgeChoice::Owned,
        MakeChoice::GitHub => w::ForgeChoice::Github,
        MakeChoice::Forgejo { api, git, web } => {
            let empty_web = match &web {
                Some(web) => web.is_empty(),
                None => false,
            };
            if api.is_empty() || git.is_empty() || empty_web {
                return None;
            }
            w::ForgeChoice::Forgejo(w::ForgejoChoice::new(&limits, w::ForgejoChoiceParts { api, git, web }).ok()?)
        }
    };
    let request = w::Request::MakeForge(w::MakeForge::new(&limits, w::MakeForgeParts { choice }).ok()?);
    let record = w::Requesting::new(&limits, w::RequestingParts { request }).ok()?;
    Some(Ask::Family {
        scope: c::Scope::Deployment,
        record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::requesting(&record) },
    })
}
#[must_use]
pub fn answer(answer: &Answered, limits: &Limits) -> Option<MakeAnswer> {
    let record = match answer {
        Answered::Jig(c::Answer::Refused { refusal, .. }) => {
            return Some(MakeAnswer::EntranceRefused(c::EntranceRefusal::new(refusal.clone()).ok()?));
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
    Some(match answer.into_parts().outcome {
        w::Outcome::Made(made) => MakeAnswer::Made { forge: made.forge() },
        w::Outcome::Full => MakeAnswer::Full,
        w::Outcome::NotOffered => MakeAnswer::NotOffered,
        w::Outcome::Unknown => MakeAnswer::Unknown,
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
        | w::Outcome::Removing
        | w::Outcome::Playground
        | w::Outcome::Reset => return None,
    })
}
pub(crate) fn busy(side: &Side) -> bool {
    match &side.making {
        Some(making) => making.progress.is_some(),
        None => false,
    }
}
pub(crate) fn send(side: &mut Side, limits: Limits, choice: MakeChoice, then: Then, out: &mut Queue<Request>) {
    if side.party.is_none() || side.served == Served::OutOfDate || busy(side) {
        return;
    }
    if then == Then::SetUp {
        side.setup_form.checked = true;
        side.changed();
        for field in &side.setup_form.fields {
            if field.is_empty() {
                return;
            }
        }
    }
    let Some(ask) = ask(choice.clone(), &limits) else { return };
    side.serial = side.serial.checked_add(1).expect("forge request serial fits");
    let request = Token::new(side.serial);
    side.making = Some(Making { choice, then, request, progress: Some(RequestProgress::Parked), answer: None });
    out.push(Request::Send { request, name: None, ask });
    side.changed();
}
pub(crate) fn write(side: &mut Side, limits: Limits, field: Field, text: Box<[u8]>) {
    if busy(side) || text.len() > usize::try_from(limits.field_bytes).expect("field bound fits") {
        return;
    }
    let draft = &mut side.elsewhere_form;
    match field {
        Field::Api => draft.api = text,
        Field::Git => draft.git = text,
        Field::Web => draft.web = text,
    }
    let written = draft.written.get_mut(field.index()).expect("three fields");
    *written = written.checked_add(1).expect("written count fits");
    draft.checked = false;
    side.changed();
}
pub(crate) fn elsewhere(side: &mut Side, limits: Limits, out: &mut Queue<Request>) {
    side.elsewhere_form.checked = true;
    side.changed();
    let draft = &side.elsewhere_form;
    if draft.api.is_empty() || draft.git.is_empty() {
        return;
    }
    let choice = MakeChoice::Forgejo {
        api: draft.api.clone(),
        git: draft.git.clone(),
        web: if draft.web.is_empty() { None } else { Some(draft.web.clone()) },
    };
    send(side, limits, choice, Then::Open, out);
}
pub(crate) fn told(
    side: &mut Side,
    client: &jig_client::Client,
    value: &jig_client::Told,
    limits: Limits,
    address: &Address,
    out: &mut Queue<Request>,
) -> bool {
    use jig_client::{Outcome, Progress, Told};
    let Some(making) = &mut side.making else { return false };
    if making.progress.is_none() {
        return false;
    }
    let key = jig_client::Key { asker: crate::owner::Owner::Forge.asker(), number: making.request.raw() };
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
                making.progress =
                    Some(if side.served == Served::OutOfDate { RequestProgress::Parked } else { progress });
                side.changed();
                return true;
            }
            MakeAnswer::Unknown
        }
        Told::Answered { key: answered, outcome } if *answered == key => match outcome {
            Outcome::Answered(answered) => answer(answered, &limits).unwrap_or(MakeAnswer::Unknown),
            Outcome::Unknown
            | Outcome::Limit
            | Outcome::Held
            | Outcome::NotKept
            | Outcome::Unsent
            | Outcome::Full { .. }
            | Outcome::Lost
            | Outcome::Unconfirmed => MakeAnswer::Unknown,
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
    making.progress = None;
    making.answer = Some(answer.clone());
    let then = making.then.clone();
    let choice = match making.choice {
        MakeChoice::Here => Choice::Here,
        MakeChoice::GitHub => Choice::GitHub,
        MakeChoice::Forgejo { .. } => Choice::Elsewhere,
    };
    if let MakeAnswer::Made { forge } = answer {
        match then {
            Then::SetUp => forges::set_up(side, limits, forge, out),
            Then::SetUpAtOnce => forges::set_up_at_once(side, limits, forge, out),
            Then::ConnectManifest { back } => github::manifest(side, limits, forge, back, out),
            Then::Open => {}
        }
        if side.making_address.as_ref() == Some(address)
            && let Some(address) = next_address(address, choice, forge)
        {
            out.push(Request::Replace { address });
        }
    }
    side.changed();
    true
}

fn next_address(address: &Address, choice: Choice, forge: u16) -> Option<Address> {
    match address {
        Address::ConnectForge { forge: at } if at.is_none() || *at == Some(choice) => {
            Some(Address::ConnectForge { forge: Some(Choice::Forge(forge)) })
        }
        Address::Welcome { step: Step::Forge { forge: at } } if at.is_none() || *at == Some(choice) => {
            Some(Address::Welcome { step: Step::Forge { forge: Some(Choice::Forge(forge)) } })
        }
        Address::Inbox { .. }
        | Address::Chats { .. }
        | Address::Accounts
        | Address::Forges
        | Address::ConnectForge { .. }
        | Address::ConnectModel { .. }
        | Address::Welcome { .. }
        | Address::Chat { .. }
        | Address::Task { .. }
        | Address::Adopt { .. } => None,
    }
}
