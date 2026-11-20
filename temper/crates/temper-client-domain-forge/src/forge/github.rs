//! An App's editable id and one-shot key (client/forge/deployment.md).
use super::{Limits, Request, Served, Side, forges};
use alloc::boxed::Box;
use jig_client::{Answered, Ask, Event, Family, Key, Outcome, Told, contract as c};
use skein_lib::{Queue, Secret};
use temper_client_protocol_wire::v1::Record;
use temper_client_protocol_wire::{self as wire, v1 as w};

#[derive(Debug)]
#[expect(clippy::partial_pub_fields, reason = "the view reads presentation while the one-shot key stays private")]
pub struct Draft {
    pub id: Box<[u8]>,
    pub written: u64,
    pub key_written: u64,
    pub checked: bool,
    pub too_long: bool,
    key: Option<Secret>,
}
impl Draft {
    pub(crate) fn new() -> Self {
        Self { id: Box::default(), written: 0, key_written: 0, checked: false, too_long: false, key: None }
    }
    #[must_use]
    pub fn has_key(&self) -> bool {
        match &self.key {
            Some(key) => !forges::trimmed_app_key(key.expose()).is_empty(),
            None => false,
        }
    }
    #[must_use]
    pub fn number(&self) -> Option<u64> {
        let mut number = 0_u64;
        if self.id.is_empty() {
            return None;
        }
        for byte in &self.id {
            if !byte.is_ascii_digit() {
                return None;
            }
            number = number.checked_mul(10)?.checked_add(u64::from(byte.checked_sub(b'0')?))?;
        }
        if number == 0 { None } else { Some(number) }
    }
    pub(crate) fn clear(&mut self) {
        self.key = None;
        self.key_written = self.key_written.checked_add(1).expect("key writes fit");
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answer {
    Connecting,
    Link(Box<[u8]>),
    Page(super::family::Page),
    NoAccess,
    NotFound,
    Refused(w::ForgeRefusal),
    Entrance(c::EntranceRefusal),
    Unknown,
}
#[derive(Debug)]
pub struct Pending {
    pub forge: u16,
    pub key: Key,
    pub kind: Option<WayKind>,
    pub answer: Option<Answer>,
    pub way: Option<Way>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WayKind {
    Connect,
    Install,
}
/// A GitHub departure lives only in this client (client/forge/deployment.md, GitHub's way, in a new tab).
#[derive(Debug)]
pub struct Way {
    pub address: Option<Box<[u8]>>,
    pub refused: bool,
    pub accepted: Box<[u64]>,
    pub cleared: bool,
    pub app: WayApp,
}
/// The App at the press, or the missing App observed being made since it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WayApp {
    AtPress,
    Missing,
    Made,
}
impl Pending {
    /// A connect that has made its missing App reopens through a fresh Install press.
    #[must_use]
    pub fn install_again(&self) -> Option<u16> {
        let way = self.way.as_ref()?;
        if self.kind == Some(WayKind::Connect) && way.app == WayApp::Made && self.forge != 0 && self.answer.is_some() {
            Some(self.forge)
        } else {
            None
        }
    }
}
pub(crate) fn write(side: &mut Side, text: Box<[u8]>, limits: Limits) {
    if let Some(pending) = &side.app
        && pending.answer.is_none()
    {
        return;
    }
    if text.len() > usize::try_from(limits.field_bytes).expect("field bound fits") {
        return;
    }
    side.app_form.id = text;
    side.app_form.written = side.app_form.written.checked_add(1).expect("id writes fit");
    side.app_form.checked = false;
    side.app = None;
    side.changed();
}
pub(crate) fn key(side: &mut Side, value: Secret) {
    if let Some(pending) = &side.app
        && pending.answer.is_none()
    {
        return;
    }
    side.app_form.key = None;
    side.app = None;
    side.app_form.too_long = value.len() > forges::PASTED_PAYLOAD;
    if !value.is_empty() && !side.app_form.too_long {
        side.app_form.key = Some(value);
    }
    side.app_form.checked = false;
    side.changed();
}
pub(crate) fn send(side: &mut Side, limits: Limits, forge: u16, installing: bool, out: &mut Queue<Request>) {
    if !available(side) {
        return;
    }
    let bounds = super::wire_limits(&limits).expect("checked forge limits");
    let request = if installing {
        w::Request::Install(w::Install { forge, back: None })
    } else {
        side.app_form.checked = true;
        let Some(id) = side.app_form.number() else {
            side.changed();
            return;
        };
        if !side.app_form.has_key() {
            side.changed();
            return;
        }
        let Some(key) = &side.app_form.key else {
            side.changed();
            return;
        };
        let pem = key.expose();
        if forges::restored_app_key(pem).is_none() {
            side.app_form.too_long = !pem.is_empty();
            side.changed();
            return;
        }
        w::Request::Connect(w::Connect {
            forge: forges::making(side, forge, &bounds, None).expect("valid target"),
            how: w::How::GithubApp(w::GitHubApp { id, read_only: None }),
        })
    };
    dispatch(side, &bounds, forge, request, if installing { Some(WayKind::Install) } else { None }, out);
}
fn available(side: &Side) -> bool {
    side.party.is_some()
        && side.served != Served::OutOfDate
        && match &side.app {
            Some(pending) => pending.answer.is_some(),
            None => true,
        }
}
pub(crate) fn manifest(side: &mut Side, limits: Limits, forge: u16, out: &mut Queue<Request>) {
    if !available(side) {
        return;
    }
    let bounds = super::wire_limits(&limits).expect("checked forge limits");
    let how = w::How::GithubManifest(w::GitHubManifest { back: None });
    let request = w::Request::Connect(w::Connect {
        forge: forges::making(side, forge, &bounds, None).expect("valid target"),
        how,
    });
    dispatch(side, &bounds, forge, request, Some(WayKind::Connect), out);
}
fn dispatch(
    side: &mut Side,
    bounds: &w::Limits,
    forge: u16,
    request: w::Request,
    kind: Option<WayKind>,
    out: &mut Queue<Request>,
) {
    let record = w::Requesting { request };
    let request = Ask::Family {
        scope: c::Scope::Deployment,
        record: Family {
            family: wire::FORGE,
            version: wire::CLIENT_VERSION,
            bytes: record.encode(bounds).expect("request fits"),
        },
    };
    side.serial = side.serial.checked_add(1).expect("forge keys fit");
    let key = Key { asker: crate::owner::Owner::Forge.asker(), number: side.serial };
    let event = if kind.is_some() {
        Event::Send { key, name: None, request }
    } else {
        let value = side.app_form.key.take().expect("validated key");
        let pem = value.expose();
        let value = forges::restored_app_key(pem).expect("validated bundle");
        side.app_form.clear();
        // A fresh request name, never reused with another pasted value.
        let name = crate::key::named(
            b"temper-app",
            &crate::key::Name::new(skein_lib::Decimal::of(side.serial).as_bytes()).expect("serial name fits"),
        );
        Event::Carry { key, name, request, value: Some(value) }
    };
    let way = if kind.is_some() {
        let mut accepted = skein_lib::List::uncapped();
        let mut app = WayApp::Missing;
        if let Some(row) = super::connect::row(forge, &side.forges, super::connect::playground(side)) {
            if row.app.is_some() {
                app = WayApp::AtPress;
            }
            for installation in &row.installations {
                if !installation.stray && !installation.suspended {
                    accepted.push(installation.id).expect("accepted ids fit");
                }
            }
        }
        out.push(Request::OpenTab { key });
        Some(Way { address: None, refused: false, accepted: accepted.into_boxed(), cleared: false, app })
    } else {
        None
    };
    side.app = Some(Pending { forge, key, kind, answer: None, way });
    out.push(Request::Client { asked: event });
    side.changed();
}
pub fn told(
    side: &mut Side,
    client: &jig_client::Client,
    told: &Told,
    limits: Limits,
    out: &mut Queue<Request>,
) -> bool {
    let Some(pending) = &mut side.app else {
        return false;
    };
    let outcome = match told {
        Told::Answered { key, outcome } if *key == pending.key => outcome,
        Told::Pending { key } if *key == pending.key => {
            if let Some(value) = client.pending(*key)
                && value.progress == jig_client::Progress::Unknown
                && pending.answer.is_none()
            {
                pending.answer = Some(Answer::Unknown);
                pending.way = None;
                if pending.kind.is_some() {
                    out.push(Request::CloseTab { key: pending.key });
                }
                side.changed();
            }
            return true;
        }
        Told::Link { .. }
        | Told::Pending { .. }
        | Told::Answered { .. }
        | Told::Kept { .. }
        | Told::Dropped { .. }
        | Told::Changed { .. }
        | Told::Text { .. }
        | Told::Ended { .. }
        | Told::Page { .. } => return false,
    };
    if pending.answer.is_some() {
        return true;
    }
    let answer = decode(outcome, pending.kind.is_some(), limits).unwrap_or(Answer::Unknown);
    if pending.kind.is_some() {
        match &answer {
            Answer::Link(to) => {
                if let Some(way) = &mut pending.way {
                    way.address = Some(to.clone());
                    out.push(Request::PointTab {
                        key: pending.key,
                        to: to.clone(),
                        on_origin: pending.kind == Some(WayKind::Connect),
                    });
                }
            }
            Answer::Connecting
            | Answer::Page(_)
            | Answer::NoAccess
            | Answer::NotFound
            | Answer::Refused(_)
            | Answer::Entrance(_)
            | Answer::Unknown => {
                pending.way = None;
                out.push(Request::CloseTab { key: pending.key });
            }
        }
    }
    let accepted = match &answer {
        Answer::Connecting | Answer::Link(_) | Answer::Page(_) => true,
        Answer::NoAccess | Answer::NotFound | Answer::Refused(_) | Answer::Entrance(_) | Answer::Unknown => false,
    };
    pending.answer = Some(answer);
    if accepted
        && pending.forge == 0
        && let Some(new) = &mut side.setup_form.making
        && new.kind == forges::Kind::GitHub
    {
        new.accepted = true;
    }
    forges::bind_new(side, out);
    side.changed();
    true
}
/// The manifest's path and query on the page's own origin. Its path follows the web
/// platform's origin-path policy, shared by the note and shell (client/forge/deployment.md, The press).
#[must_use]
pub fn origin_path(address: &[u8]) -> Option<&[u8]> {
    let tail = match address.strip_prefix(b"https://") {
        Some(tail) => tail,
        None => address.strip_prefix(b"http://")?,
    };
    let slash = skein_lib::bytes::find(tail, b"/")?;
    let authority = tail.get(..slash)?;
    if authority.is_empty() || authority.contains(&b'?') || authority.contains(&b'#') || authority.contains(&b'\\') {
        return None;
    }
    let path = tail.get(slash..)?;
    if path.starts_with(b"//") || path.contains(&b'\\') {
        return None;
    }
    for byte in path {
        if !(0x21..=0x7e).contains(byte) {
            return None;
        }
    }
    Some(path)
}

pub(crate) fn again(side: &mut Side, to: Box<[u8]>, out: &mut Queue<Request>) {
    let Some(pending) = &side.app else { return };
    let Some(way) = &pending.way else { return };
    if pending.install_again().is_some()
        || pending.kind != Some(WayKind::Connect)
        || way.address.as_deref() != Some(to.as_ref())
        || origin_path(&to).is_none()
    {
        return;
    }
    out.push(Request::OpenTab { key: pending.key });
    out.push(Request::PointTab { key: pending.key, to, on_origin: true });
}

/// A refused shell tab still leaves the answered address as the person's own link.
pub fn refused(side: &mut Side, key: Key) -> bool {
    let Some(pending) = &mut side.app else { return false };
    if pending.key != key {
        return false;
    }
    let Some(way) = &mut pending.way else { return false };
    way.refused = true;
    side.changed();
    true
}
/// Only a delivered document can clear the press's old ending or finish its way.
pub(crate) fn watched(side: &mut Side, out: &mut Queue<Request>) {
    let Some(pending) = &mut side.app else { return };
    let Some(way) = &mut pending.way else { return };
    let forges::State::Rows(rows) = &side.forges else { return };
    for row in rows {
        let target = if pending.forge == 0 {
            match &side.setup_form.making {
                Some(making) => row.kind == forges::Kind::GitHub && !making.seen.contains(&row.number),
                None => false,
            }
        } else {
            row.number == pending.forge
        };
        if !target {
            continue;
        }
        if pending.kind == Some(WayKind::Connect) && way.app == WayApp::Missing && row.app.is_some() {
            way.app = WayApp::Made;
        }
        let mut added = false;
        for installation in &row.installations {
            if !installation.stray && !installation.suspended && !way.accepted.contains(&installation.id) {
                added = true;
            }
        }
        if row.ended.is_none() {
            way.cleared = true;
        }
        if added || (way.cleared && row.ended.is_some()) {
            pending.way = None;
            if pending.answer.is_none() {
                out.push(Request::CloseTab { key: pending.key });
            }
            out.push(Request::GitHubEnded { forge: row.number });
        }
        return;
    }
}
pub(crate) fn decode(outcome: &Outcome, linking: bool, limits: Limits) -> Option<Answer> {
    let Outcome::Answered(answer) = outcome else {
        return Some(Answer::Unknown);
    };
    match answer {
        Answered::Jig(c::Answer::Refused { refusal, .. }) => {
            Some(Answer::Entrance(c::EntranceRefusal::new(refusal.clone()).ok()?))
        }
        Answered::Family(record) if record.family == wire::FORGE && record.version == wire::CLIENT_VERSION => {
            let mut reader = skein_lib::Reader::new(&record.bytes);
            let answer = w::Answer::decode(&super::wire_limits(&limits)?, &mut reader).ok()?;
            if !reader.is_empty() {
                return None;
            }
            match answer.outcome {
                w::Outcome::Connecting => Some(Answer::Connecting),
                w::Outcome::ConnectRefused(refusal) => Some(Answer::Refused(refusal.why.clone())),
                w::Outcome::Link(link) if linking => Some(Answer::Link(link.address)),
                w::Outcome::Unknown => Some(Answer::Unknown),
                w::Outcome::Page(page) => Some(Answer::Page(super::family::page(page))),
                w::Outcome::ForgeRefused(refusal) => match refusal.reason {
                    w::ForgeReason::NoAccess => Some(Answer::NoAccess),
                    w::ForgeReason::NotFound => Some(Answer::NotFound),
                    w::ForgeReason::NoSuchForge
                    | w::ForgeReason::AnotherProject
                    | w::ForgeReason::HomeHeld
                    | w::ForgeReason::PrefixOccupied
                    | w::ForgeReason::Refused
                    | w::ForgeReason::InFlight
                    | w::ForgeReason::TooLarge
                    | w::ForgeReason::Unreachable
                    | w::ForgeReason::InvalidAnswer => None,
                },
                w::Outcome::Accepted(_)
                | w::Outcome::MakeKept
                | w::Outcome::MakeRefused(_)
                | w::Outcome::Adopted(_)
                | w::Outcome::CoreRefused(_)
                | w::Outcome::SettingUp
                | w::Outcome::SetUpRefused(_)
                | w::Outcome::Running
                | w::Outcome::StartRefused(_)
                | w::Outcome::StartFailed(_)
                | w::Outcome::Link(_)
                | w::Outcome::Removed
                | w::Outcome::Looked
                | w::Outcome::Dismissed
                | w::Outcome::RemoveRefused(_)
                | w::Outcome::Reset
                | w::Outcome::DeviceAdded(_)
                | w::Outcome::DeviceRemoved
                | w::Outcome::DeviceRefused(_)
                | w::Outcome::Away
                | w::Outcome::Tried
                | w::Outcome::Filed(_)
                | w::Outcome::Handed
                | w::Outcome::Triaged
                | w::Outcome::FindingRefused(_)
                | w::Outcome::Drafted(_)
                | w::Outcome::DraftRefused(_)
                | w::Outcome::Follow(_)
                | w::Outcome::Opened(_)
                | w::Outcome::OpenRefused(_)
                | w::Outcome::Tier(_)
                | w::Outcome::NotTheirs
                | w::Outcome::Overturned
                | w::Outcome::OverturnRefused(_)
                | w::Outcome::Checkpointed(_)
                | w::Outcome::CheckpointRefused(_)
                | w::Outcome::EndRefused(_) => None,
            }
        }
        Answered::Jig(_) | Answered::Family(_) => None,
    }
}
