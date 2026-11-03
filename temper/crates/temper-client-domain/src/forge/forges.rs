//! Kept forge records and their setup (temper/docs/design/client/forge.md, 5.4; temper/docs/design/client/README.md, 3.5).
use super::{family, wire_limits};
use alloc::boxed::Box;
use jig_client::{Answered, Ask, Family, Watchable, contract as c};
use temper_client_protocol_wire::{self as wire, v1 as w};
/// The longest setup reason rendered with one replacement per invalid byte.
pub const FAILURE_BYTES: u32 = wire::SET_UP_WHY_BYTES.checked_mul(3).expect("reason text fits");
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FailedStep {
    Fetch,
    Prepare,
    Run,
    Connect,
}

/// The wire bounds the session's forge rows.
pub const ROWS: u32 = wire::FORGES;
pub const INSTALLATIONS: u32 = wire::INSTALLATIONS;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Forgejo,
    GitHub,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Connected,
    Managed,
    Owned,
    Fake,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Step {
    Fetch,
    Prepare,
    Run,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Offer {
    NotOffered,
    NotUnderTheBox,
    Offered,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Connection {
    NotConnected,
    NotSetUp,
    SettingUp { step: Option<Step> },
    SetUpFailed { why: Box<[u8]>, at: Option<FailedStep> },
    Connecting,
    Connected { id: u64, login: Box<[u8]> },
    Stopped { reason: Box<[u8]> },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CreateStep {
    Admin,
    Org,
    Repository,
    File,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FirstState {
    Making,
    Made { id: u64 },
    Failed { step: CreateStep },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetupStage {
    Admin,
    Bot,
    Token,
    Org,
    Repository,
    File,
    App,
    Installation,
    State,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AdminStop {
    NotAdmin,
    Refused { status: u16, stage: SetupStage },
    Unmarked,
    Other,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct First {
    pub playground: bool,
    pub org: Box<[u8]>,
    pub repository: Box<[u8]>,
    pub state: FirstState,
    pub label: Box<[u8]>,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Account {
    pub id: u64,
    pub login: Box<[u8]>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Installation {
    pub id: u64,
    pub account: Option<Account>,
    pub suspended: bool,
    pub stray: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AppWay {
    Manifest,
    Pasted,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Adoption {
    pub owner: Box<[u8]>,
    pub name: Box<[u8]>,
    pub project: u32,
}
/// Public owned-forge versions and first-password file path (client/forge.md, 5.4).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Owned {
    pub forgejo: Box<[u8]>,
    pub runner: Option<Box<[u8]>>,
    pub password: Box<[u8]>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Row {
    pub read_only: bool,
    pub number: u16,
    pub kind: Kind,
    pub mode: Mode,
    pub connection: Connection,
    pub offer: Offer,
    pub first: Option<First>,
    pub installations: Box<[Installation]>,
    pub ended: Option<Box<[u8]>>,
    pub app: Option<AppWay>,
    pub web: Option<Box<[u8]>>,
    pub repositories: Box<[Adoption]>,
    pub owned: Option<Owned>,
    pub admin: bool,
    pub ended_stop: Option<AdminStop>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetUpOutcome {
    SettingUp,
    Refused(SetUpRefusal),
    Unknown,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetUpRefusal {
    NotOffered,
    NotUnderTheBox,
    Busy,
    Invalid,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum SetUpAnswer {
    Outcome(SetUpOutcome),
    EntranceRefused(c::EntranceRefusal),
}
#[must_use]
pub fn watch(limits: &super::Limits) -> Option<Watchable> {
    let limits = wire_limits(limits)?;
    let watching = w::Watching::new(&limits, w::WatchingParts { watch: w::Watch::Forges }).ok()?;
    Some(Watchable::Family {
        scope: c::Scope::Deployment,
        window: c::Window { rows: 0, after: None },
        record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::watching(&watching) },
    })
}
#[must_use]
pub fn ask(forge: u16, org: Box<[u8]>, repository: Box<[u8]>, playground: bool, limits: &super::Limits) -> Option<Ask> {
    let bounds = wire_limits(limits)?;
    let forge = w::Making::Kept(w::Kept::new(&bounds, w::KeptParts { forge }).ok()?);
    ask_target(forge, org, repository, playground, &bounds)
}
fn ask_target(
    forge: w::Making,
    org: Box<[u8]>,
    repository: Box<[u8]>,
    playground: bool,
    bounds: &w::Limits,
) -> Option<Ask> {
    let value = w::SetUp::new(bounds, w::SetUpParts { forge, org, repository, playground }).ok()?;
    request(w::Request::SetUp(value), bounds)
}
#[must_use]
pub fn answer(answer: &Answered, limits: &super::Limits) -> Option<SetUpAnswer> {
    match answer {
        Answered::Family(record) => Some(SetUpAnswer::Outcome(family::set_up_answer(record, &wire_limits(limits)?)?)),
        Answered::Jig(c::Answer::Refused { refusal, .. }) => {
            Some(SetUpAnswer::EntranceRefused(c::EntranceRefusal::new(refusal.clone()).ok()?))
        }
        Answered::Jig(c::Answer::Done(_)) => None,
    }
}
/// Invalid documents are refused by the watch's owner, which unwatches its handle.
#[must_use]
pub fn rows(record: &Family, limits: &super::Limits) -> Option<Box<[Row]>> {
    if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
        return None;
    }
    family::forges(&record.bytes, &wire_limits(limits)?)
}

/// The session keeps an ended watch until the next sign-in (client/README.md, 4.6).
#[derive(Debug)]
pub enum State {
    Unknown,
    Rows(Box<[Row]>),
    Ended,
}
/// The set up's editable fields (client/forge.md, 5.4).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SetupField {
    Organization,
    Repository,
}
impl SetupField {
    #[must_use]
    pub const fn index(self) -> usize {
        match self {
            Self::Organization => 0,
            Self::Repository => 1,
        }
    }
}
#[derive(Debug)]
pub struct Draft {
    pub removal: Option<Removal>,
    pub making: Option<New>,
    pub fields: [Box<[u8]>; 2],
    pub written: [u64; 2],
    pub checked: bool,
}
impl Draft {
    pub(crate) fn new() -> Self {
        Self {
            removal: None,
            making: None,
            fields: [Box::from(b"work".as_slice()), Box::from(b"hello".as_slice())],
            written: [0; 2],
            checked: false,
        }
    }
    #[must_use]
    pub fn required(&self, field: SetupField) -> bool {
        self.checked && self.text(field).is_empty()
    }
    #[must_use]
    pub fn text(&self, field: SetupField) -> &[u8] {
        self.fields.get(field.index()).expect("two setup fields")
    }
    #[must_use]
    pub fn written(&self, field: SetupField) -> u64 {
        *self.written.get(field.index()).expect("two setup fields")
    }
}
pub(crate) fn write(side: &mut super::Side, limits: super::Limits, field: SetupField, text: Box<[u8]>) {
    if text.len() > usize::try_from(limits.field_bytes).expect("field bytes fit") {
        return;
    }
    let draft = &mut side.setup_form;
    let index = field.index();
    *draft.fields.get_mut(index).expect("two setup fields") = text;
    let written = draft.written.get_mut(index).expect("two setup fields");
    *written = written.checked_add(1).expect("written count fits");
    draft.checked = false;
    if let Some(last) = &mut side.set_up {
        last.answer = None;
    }
    side.changed();
}
/// New records are identified from the watch, after the entrance answers.
#[derive(Debug)]
pub struct New {
    pub kind: Kind,
    pub mode: Mode,
    pub seen: Box<[u16]>,
    pub accepted: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum RemovalEffect {
    OwnedDirectory,
    ManagedBotAndHooksStay,
    GitHubAppStays,
    Playground,
}
#[derive(Debug)]
pub struct Removal {
    pub forge: u16,
    pub kind: Kind,
    pub mode: Mode,
    pub effect: RemovalEffect,
    pub request: Option<skein_lib::Token>,
    pub outcome: Option<jig_client::Outcome>,
    pub start_over: bool,
}
/// Actions whose words and confirmation are rendered by the view.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Act {
    SetUpOwned,
    SetUpOwnedAtOnce,
    GitHub { back: Box<[u8]> },
    Managed { place: Managed },
    Connected { place: Managed },
    PlaceWrite { field: super::admin::PlaceField, text: Box<[u8]> },
    ConnectManaged,
    ConnectTokens,
    OpenReplace { forge: u16 },
    CloseReplace { forge: u16 },
    Replace { forge: u16 },
    Remove { forge: u16 },
    ConfirmRemove,
    CancelRemove,
    StartOver { forge: u16 },
    GoOn { forge: u16, back: Box<[u8]> },
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Managed {
    pub host: Box<[u8]>,
    pub port: u16,
    pub tls: Option<Box<[u8]>>,
    pub git: Box<[u8]>,
    pub web: Option<Box<[u8]>>,
}
impl Managed {
    #[must_use]
    pub fn valid(&self) -> bool {
        !self.host.is_empty() && self.port != 0
    }
}
pub(crate) fn act(side: &mut super::Side, limits: super::Limits, act: Act, out: &mut skein_lib::Queue<super::Request>) {
    match act {
        Act::SetUpOwned => {
            if !can_make(side) || !can_set_up(side) || side.playground_running() {
                return;
            }
            new_record(side, Kind::Forgejo, Mode::Owned);
            set_up(side, limits, 0, out);
        }
        Act::SetUpOwnedAtOnce => {
            if !can_make(side) || !can_set_up(side) || side.playground_running() {
                return;
            }
            new_record(side, Kind::Forgejo, Mode::Owned);
            set_up_at_once(side, limits, 0, out);
        }
        Act::GitHub { back } => {
            if !can_make(side)
                || side.party.is_none()
                || side.playground_running()
                || side.served == super::Served::OutOfDate
            {
                return;
            }
            if let Some(pending) = &side.app
                && pending.answer.is_none()
            {
                return;
            }
            new_record(side, Kind::GitHub, Mode::Connected);
            super::github::manifest(side, limits, 0, back, out);
        }
        Act::Managed { place } => super::admin::send_new(side, limits, place, out),
        Act::Connected { place } => super::tokens::send_new(side, limits, place, out),
        Act::PlaceWrite { field, text } => super::admin::place_write(side, limits, field, text),
        Act::ConnectManaged => super::admin::connect_managed(side, limits, out),
        Act::OpenReplace { forge } => super::replace::open(side, forge),
        Act::CloseReplace { forge } => super::replace::close(side, forge),
        Act::Replace { forge } => super::replace::send(side, limits, forge, out),
        Act::ConnectTokens => super::tokens::connect_new(side, limits, out),
        Act::Remove { forge } => remove(side, forge),
        Act::ConfirmRemove => confirm_remove(side, limits, out),
        Act::CancelRemove => {
            if let Some(removal) = &side.setup_form.removal
                && removal.request.is_some()
            {
                return;
            }
            side.setup_form.removal = None;
            side.changed();
        }
        Act::StartOver { forge } => start_over(side, limits, forge, out),
        Act::GoOn { forge, back } => {
            let Some(row) = super::connect::row(forge, &side.forges, super::connect::playground(side)) else { return };
            if row.kind != Kind::GitHub {
                return;
            }
            match row.app {
                Some(AppWay::Manifest) => super::github::manifest(side, limits, forge, back, out),
                Some(AppWay::Pasted) => super::github::send(side, limits, forge, Some(back), out),
                None => {}
            }
        }
    }
}
pub(crate) fn can_make(side: &super::Side) -> bool {
    let Some(new) = &side.setup_form.making else { return true };
    if new.accepted {
        return false;
    }
    if let Some(last) = &side.set_up
        && last.forge.is_none()
        && last.progress.is_some()
    {
        return false;
    }
    if let Some(pending) = &side.admin
        && pending.forge == 0
        && pending.answer.is_none()
    {
        return false;
    }
    if let Some(pending) = &side.app
        && pending.forge == 0
        && pending.answer.is_none()
    {
        return false;
    }
    !super::tokens::busy(side)
}
pub(crate) fn new_record(side: &mut super::Side, kind: Kind, mode: Mode) {
    let mut seen = skein_lib::List::with_capacity(ROWS);
    if let State::Rows(rows) = &side.forges {
        for row in rows {
            seen.push(row.number).expect("bounded rows");
        }
    }
    side.setup_form.making = Some(New { kind, mode, seen: seen.into_boxed(), accepted: false });
}
pub(crate) fn bind_new(side: &mut super::Side, out: &mut skein_lib::Queue<super::Request>) {
    let Some(making) = &side.setup_form.making else { return };
    if !making.accepted {
        return;
    }
    let State::Rows(rows) = &side.forges else { return };
    for row in rows {
        if row.kind == making.kind && row.mode == making.mode && !making.seen.contains(&row.number) {
            if let Some(last) = &mut side.set_up
                && last.forge.is_none()
            {
                last.forge = Some(row.number);
            }
            if let Some(pending) = &mut side.admin
                && pending.forge == 0
                && making.kind == Kind::Forgejo
            {
                pending.forge = row.number;
            }
            if let Some(pending) = &mut side.tokens
                && pending.forge == 0
                && making.kind == Kind::Forgejo
                && making.mode == Mode::Connected
            {
                pending.forge = row.number;
                out.push(super::Request::ConnectForge { forge: row.number });
            }
            if let Some(pending) = &mut side.app
                && pending.forge == 0
                && making.kind == Kind::GitHub
            {
                pending.forge = row.number;
            }
            side.setup_form.making = None;
            return;
        }
    }
}
#[must_use]
pub(crate) fn making(side: &super::Side, forge: u16, bounds: &w::Limits, place: Option<Managed>) -> Option<w::Making> {
    if forge == 0
        && let Some(new) = &side.setup_form.making
    {
        let ask = match (new.kind, new.mode) {
            (Kind::GitHub, Mode::Connected | Mode::Managed | Mode::Owned | Mode::Fake) => w::ForgeAsk::Github,
            (Kind::Forgejo, Mode::Owned) => w::ForgeAsk::Owned,
            (Kind::Forgejo, Mode::Connected) => {
                let Managed { host, port, tls, git, web } = place?;
                w::ForgeAsk::Connected(
                    w::ManagedForge::new(bounds, w::ManagedForgeParts { host, port, tls, git, web, git_program: None })
                        .ok()?,
                )
            }
            (Kind::Forgejo, Mode::Managed | Mode::Fake) => return None,
        };
        return Some(w::Making::New(w::NewForge::new(bounds, w::NewForgeParts { forge: ask }).ok()?));
    }
    Some(w::Making::Kept(w::Kept::new(bounds, w::KeptParts { forge }).ok()?))
}
/// Collect the confirmation's facts without sending a request.
pub fn remove(side: &mut super::Side, forge: u16) {
    if side.party.is_none() || side.served == super::Served::OutOfDate {
        return;
    }
    if let Some(last) = &side.setup_form.removal
        && last.request.is_some()
        && last.outcome.is_none()
    {
        return;
    }
    let State::Rows(rows) = &side.forges else { return };
    for row in rows {
        if row.number != forge {
            continue;
        }
        let effect = match (row.kind, row.mode) {
            (Kind::GitHub, Mode::Connected | Mode::Managed | Mode::Owned | Mode::Fake) => RemovalEffect::GitHubAppStays,
            (Kind::Forgejo, Mode::Owned) => RemovalEffect::OwnedDirectory,
            (Kind::Forgejo, Mode::Connected | Mode::Managed) => RemovalEffect::ManagedBotAndHooksStay,
            (Kind::Forgejo, Mode::Fake) => RemovalEffect::Playground,
        };
        side.setup_form.removal = Some(Removal {
            forge,
            kind: row.kind,
            mode: row.mode,
            effect,
            request: None,
            outcome: None,
            start_over: false,
        });
        side.changed();
        return;
    }
}
pub fn start_over(
    side: &mut super::Side,
    limits: super::Limits,
    forge: u16,
    out: &mut skein_lib::Queue<super::Request>,
) {
    if super::connect::row(forge, &side.forges, super::connect::playground(side)).is_none() {
        return;
    }
    remove(side, forge);
    let Some(removal) = &mut side.setup_form.removal else { return };
    if removal.forge != forge || removal.request.is_some() {
        return;
    }
    removal.start_over = true;
    confirm_remove(side, limits, out);
}
pub fn confirm_remove(side: &mut super::Side, limits: super::Limits, out: &mut skein_lib::Queue<super::Request>) {
    if side.party.is_none() || side.served == super::Served::OutOfDate {
        return;
    }
    let Some(removal) = &mut side.setup_form.removal else { return };
    if removal.request.is_some() {
        return;
    }
    let Some(bounds) = wire_limits(&limits) else { return };
    let Ok(value) = w::RemoveForge::new(&bounds, w::RemoveForgeParts { forge: removal.forge }) else { return };
    let Some(ask) = request(w::Request::RemoveForge(value), &bounds) else { return };
    side.serial = side.serial.checked_add(1).expect("request serial fits");
    let token = skein_lib::Token::new(side.serial);
    removal.request = Some(token);
    out.push(super::Request::Send { request: token, name: None, ask });
    side.changed();
}
#[must_use]
pub(crate) fn request(request: w::Request, bounds: &w::Limits) -> Option<Ask> {
    let requesting = w::Requesting::new(bounds, w::RequestingParts { request }).ok()?;
    Some(Ask::Family {
        scope: c::Scope::Deployment,
        record: Family { family: wire::FORGE, version: wire::CLIENT_VERSION, bytes: wire::requesting(&requesting) },
    })
}
fn removed(outcome: &jig_client::Outcome, limits: super::Limits) -> bool {
    let jig_client::Outcome::Answered(Answered::Family(record)) = outcome else { return false };
    if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
        return false;
    }
    let Some(bounds) = wire_limits(&limits) else { return false };
    let mut reader = skein_lib::Reader::new(&record.bytes);
    let Ok(answer) = w::Answer::decode(&bounds, &mut reader) else { return false };
    reader.is_empty() && *answer.outcome() == w::Outcome::Removed
}
fn removal_told(
    side: &mut super::Side,
    value: &jig_client::Told,
    limits: super::Limits,
    out: &mut skein_lib::Queue<super::Request>,
) -> bool {
    let Some(removal) = &mut side.setup_form.removal else { return false };
    let Some(request) = removal.request else { return false };
    let key = jig_client::Key { asker: crate::owner::Owner::Forge.asker(), number: request.raw() };
    if let jig_client::Told::Answered { key: answered, outcome } = value
        && *answered == key
    {
        if removal.outcome.is_some() {
            return true;
        }
        removal.outcome = Some(outcome.clone());
        if removed(outcome, limits) && removal.start_over {
            side.set_up = None;
            side.setup_form.making = None;
            side.app = None;
            side.admin = None;
            side.replacements.clear();
            side.tokens_form.clear();
            side.tokens = None;
            out.push(super::Request::Leave { to: Box::from(b"/welcome?at=forge".as_slice()) });
        }
        side.changed();
        return true;
    }
    false
}
/// The last set up sent by this client, retained across page changes.
#[derive(Debug)]
pub struct SetUp {
    pub playground: bool,
    pub forge: Option<u16>,
    pub org: Box<[u8]>,
    pub repository: Box<[u8]>,
    pub request: skein_lib::Token,
    pub progress: Option<skein_lib::RequestProgress>,
    pub answer: Option<SetUpAnswer>,
}
pub(crate) fn handle() -> jig_client::Handle {
    jig_client::Handle { asker: crate::owner::Owner::Forge.asker(), number: super::me::FORGES }
}
pub(crate) fn configure(side: &mut super::Side, limits: super::Limits, out: &mut skein_lib::Queue<super::Request>) {
    use jig_client::Link;
    if side.link == Link::SignedOut || side.link == Link::OutOfDate {
        if side.forges_open {
            side.forges_open = false;
            out.push(super::Request::Client { asked: jig_client::Event::Unwatch { handle: handle() } });
        }
        side.forges = State::Unknown;
        if side.link == Link::SignedOut {
            side.set_up = None;
            side.setup_form.making = None;
            side.setup_form.removal = None;
            side.admin_form = super::admin::Draft::new();
            side.admin = None;
            side.replacements.clear();
            side.tokens_form.clear();
            side.tokens = None;
            side.app_form.clear();
            side.app = None;
        }
        side.changed();
        return;
    }
    if side.party.is_some()
        && !side.forges_open
        && let State::Unknown = side.forges
    {
        if let Some(subject) = watch(&limits) {
            side.forges_open = true;
            out.push(super::Request::Client { asked: jig_client::Event::Watch { handle: handle(), subject } });
        } else {
            side.forges = State::Ended;
        }
        side.changed();
    }
}
pub(crate) fn set_up(
    side: &mut super::Side,
    limits: super::Limits,
    forge: u16,
    out: &mut skein_lib::Queue<super::Request>,
) {
    if !can_set_up(side) {
        return;
    }
    let previous = retry_fields(side, forge);
    let draft = &mut side.setup_form;
    draft.checked = true;
    let (org, repository, playground) = match previous {
        Some(fields) => (fields.org, fields.repository, fields.playground),
        None => {
            if draft.fields[0].is_empty() || draft.fields[1].is_empty() {
                side.changed();
                return;
            }
            (draft.fields[0].clone(), draft.fields[1].clone(), false)
        }
    };
    send_setup(side, limits, forge, SetupValues { playground, org, repository }, out);
}
/// The welcome's set up under the box sends these, none of them asked
/// (client/README.md, 3.5).
pub const AT_ONCE_ORG: &[u8] = b"work";
pub const AT_ONCE_REPOSITORY: &[u8] = b"hello";

pub(crate) fn set_up_at_once(
    side: &mut super::Side,
    limits: super::Limits,
    forge: u16,
    out: &mut skein_lib::Queue<super::Request>,
) {
    if !can_set_up(side) {
        return;
    }
    send_setup(
        side,
        limits,
        forge,
        SetupValues { playground: false, org: Box::from(AT_ONCE_ORG), repository: Box::from(AT_ONCE_REPOSITORY) },
        out,
    );
}
fn can_set_up(side: &super::Side) -> bool {
    if side.party.is_none() || side.served == super::Served::OutOfDate {
        return false;
    }
    if let Some(last) = &side.set_up
        && last.progress.is_some()
    {
        return false;
    }
    true
}
fn send_setup(
    side: &mut super::Side,
    limits: super::Limits,
    forge: u16,
    values: SetupValues,
    out: &mut skein_lib::Queue<super::Request>,
) {
    let Some(_party) = side.party else { return };
    let SetupValues { playground, org, repository } = values;
    let Some(bounds) = wire_limits(&limits) else { return };
    let Some(target) = making(side, forge, &bounds, None) else { return };
    let Some(ask) = ask_target(target, org.clone(), repository.clone(), playground, &bounds) else {
        return;
    };
    side.serial = side.serial.checked_add(1).expect("forge request serial fits");
    let request = skein_lib::Token::new(side.serial);
    side.set_up = Some(Box::new(SetUp {
        playground,
        forge: if side.setup_form.making.is_some() { None } else { Some(forge) },
        org,
        repository,
        request,
        progress: Some(skein_lib::RequestProgress::Parked),
        answer: None,
    }));
    out.push(super::Request::Send { request, name: None, ask });
    side.changed();
}
struct SetupValues {
    playground: bool,
    org: Box<[u8]>,
    repository: Box<[u8]>,
}
fn retry_fields(side: &super::Side, forge: u16) -> Option<SetupValues> {
    let State::Rows(rows) = &side.forges else { return None };
    for row in rows {
        if row.number != forge || !super::connect::failed(row) {
            continue;
        }
        if let Some(last) = &side.set_up
            && last.forge == Some(forge)
        {
            return Some(SetupValues {
                playground: last.playground,
                org: last.org.clone(),
                repository: last.repository.clone(),
            });
        }
        if let Some(first) = &row.first {
            return Some(SetupValues {
                playground: first.playground,
                org: first.org.clone(),
                repository: first.repository.clone(),
            });
        }
    }
    None
}
pub(crate) fn told(
    side: &mut super::Side,
    client: &jig_client::Client,
    value: &jig_client::Told,
    limits: super::Limits,
    out: &mut skein_lib::Queue<super::Request>,
) -> bool {
    use jig_client::{Latest, Outcome, Progress, Told};
    if super::replace::told(side, client, value, limits) {
        return true;
    }
    if super::tokens::told(side, client, value, limits) {
        bind_new(side, out);
        return true;
    }
    if removal_told(side, value, limits, out) {
        return true;
    }
    match value {
        Told::Changed { handle: changed } if *changed == handle() => {
            side.forges = match client.document(*changed) {
                Some(document) if document.state == jig_client::State::Ended => State::Ended,
                Some(document) => match document.latest {
                    Some(Latest::Family(record)) => match rows(record, &limits) {
                        Some(rows) => State::Rows(rows),
                        None => State::Ended,
                    },
                    Some(Latest::Jig(_)) => State::Ended,
                    None => State::Unknown,
                },
                None => State::Unknown,
            };
            if let State::Ended = side.forges {
                end(side, out);
            }
        }
        Told::Ended { handle: ended, .. } if *ended == handle() => {
            side.forges = State::Ended;
            end(side, out);
        }
        Told::Pending { key } if setup_key(side, *key) => {
            let Some(last) = &mut side.set_up else { return true };
            let Some(pending) = client.pending(*key) else { return true };
            let progress = match pending.progress {
                Progress::InFlight => skein_lib::RequestProgress::InFlight,
                Progress::Retrying { .. } => skein_lib::RequestProgress::Retrying,
                Progress::NotServed | Progress::Parked => skein_lib::RequestProgress::Parked,
                Progress::Unknown => skein_lib::RequestProgress::Unknown,
            };
            if progress == skein_lib::RequestProgress::Unknown {
                last.progress = None;
                last.answer = Some(SetUpAnswer::Outcome(SetUpOutcome::Unknown));
            } else {
                last.progress = Some(if side.served == super::Served::OutOfDate {
                    skein_lib::RequestProgress::Parked
                } else {
                    progress
                });
            }
        }
        Told::Answered { key, outcome } if setup_key(side, *key) => {
            let Some(last) = &mut side.set_up else { return true };
            last.progress = None;
            last.answer = Some(match outcome {
                Outcome::Answered(answered) => {
                    answer(answered, &limits).unwrap_or(SetUpAnswer::Outcome(SetUpOutcome::Unknown))
                }
                Outcome::Unknown
                | Outcome::Limit
                | Outcome::Held
                | Outcome::NotKept
                | Outcome::Unsent
                | Outcome::Full { .. }
                | Outcome::Lost
                | Outcome::Unconfirmed => SetUpAnswer::Outcome(SetUpOutcome::Unknown),
            });
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
    if let Some(last) = &side.set_up
        && last.forge.is_none()
        && last.answer == Some(SetUpAnswer::Outcome(SetUpOutcome::SettingUp))
        && let Some(new) = &mut side.setup_form.making
    {
        new.accepted = true;
    }
    bind_new(side, out);
    super::admin::stopped(side);
    super::tokens::stopped(side);
    super::replace::stopped(side);
    side.changed();
    true
}
fn setup_key(side: &super::Side, key: jig_client::Key) -> bool {
    let Some(last) = &side.set_up else { return false };
    key.asker == crate::owner::Owner::Forge.asker() && key.number == last.request.raw()
}
fn end(side: &mut super::Side, out: &mut skein_lib::Queue<super::Request>) {
    if side.forges_open {
        side.forges_open = false;
        out.push(super::Request::Client { asked: jig_client::Event::Unwatch { handle: handle() } });
    }
}

/// Bundle a pasted App's PEM for jig's carried value (protocol/credentials.md, 2.2).
#[must_use]
pub fn pasted_app_key(pem: &[u8]) -> Option<skein_lib::Secret> {
    temper_protocol_records::encode_pasted(&[(11, pem)]).ok()
}

/// Restore only RSA armour separators stripped by a password input, then bundle it.
/// Body bytes stay untouched; Paste decides whether they form a key.
#[must_use]
pub fn restored_app_key(pem: &[u8]) -> Option<skein_lib::Secret> {
    const BEGIN: &[u8] = b"-----BEGIN RSA PRIVATE KEY-----";
    const END: &[u8] = b"-----END RSA PRIVATE KEY-----";
    let pem = trimmed_app_key(pem);
    let after_begin = pem.strip_prefix(BEGIN);
    let before_end = pem.strip_suffix(END);
    let begin_lf = match after_begin {
        Some(rest) => !rest.starts_with(b"\r") && !rest.starts_with(b"\n"),
        None => false,
    };
    let end_lf = match before_end {
        Some(rest) => !rest.ends_with(b"\n"),
        None => false,
    };
    let length = pem.len().checked_add(usize::from(begin_lf))?.checked_add(usize::from(end_lf))?;
    if length == 0 || length > PASTED_PAYLOAD {
        return None;
    }
    let mut out = skein_lib::Writer::new(length);
    let mut body = pem;
    if begin_lf {
        out.put(BEGIN).ok()?;
        out.put(b"\n").ok()?;
        body = body.strip_prefix(BEGIN)?;
    }
    if end_lf {
        out.put(body.strip_suffix(END)?).ok()?;
        out.put(b"\n").ok()?;
        out.put(END).ok()?;
    } else {
        out.put(body).ok()?;
    }
    pasted_app_key(&out.finish())
}
pub(crate) fn trimmed_app_key(mut pem: &[u8]) -> &[u8] {
    while let Some(b' ' | b'\t' | b'\r' | b'\n') = pem.first() {
        pem = pem.get(1..).expect("first byte exists");
    }
    while let Some(b' ' | b'\t' | b'\r' | b'\n') = pem.last() {
        pem = pem.get(..pem.len().checked_sub(1).expect("last byte exists")).expect("shorter slice");
    }
    pem
}

/// Stable names and words for forge connection refusals (client/README.md, 4.6).
#[must_use]
pub fn connect_refusal(why: &temper_client_protocol_wire::v1::ForgeRefusal) -> (&'static [u8], &'static [u8]) {
    match why {
        temper_client_protocol_wire::v1::ForgeRefusal::Role => (b"role", b"not the deployment's owner"),
        temper_client_protocol_wire::v1::ForgeRefusal::NoSuchForge => (b"no_such_forge", b"no such forge"),
        temper_client_protocol_wire::v1::ForgeRefusal::NotTaken => (b"not_taken", b"not taken by this forge"),
        temper_client_protocol_wire::v1::ForgeRefusal::Connected => {
            (b"connected", b"connected already: forge replace replaces its key or tokens")
        }
        temper_client_protocol_wire::v1::ForgeRefusal::NotConnected => {
            (b"not_connected", b"not connected: connect it first")
        }
        temper_client_protocol_wire::v1::ForgeRefusal::Busy => (b"busy", b"busy"),
        temper_client_protocol_wire::v1::ForgeRefusal::NoKey => (b"no_key", b"no key"),
        temper_client_protocol_wire::v1::ForgeRefusal::FirstIsBot => {
            (b"first_is_bot", b"the organization is the bot's login: name another organization")
        }
    }
}

/// One accepted, usable installation makes repositories reachable.
#[must_use]
pub fn accepted(row: &Row) -> bool {
    for installation in &row.installations {
        if !installation.stray && !installation.suspended {
            return true;
        }
    }
    false
}
#[must_use]
pub fn slug(row: &Row) -> Option<&[u8]> {
    if row.kind != Kind::GitHub {
        return None;
    }
    let Connection::Connected { login, .. } = &row.connection else {
        return None;
    };
    Some(login.strip_suffix(b"[bot]").unwrap_or(login))
}

/// Payload space within the existing carried-value budget after schema framing.
pub use temper_protocol_records::{PASTED_PAYLOAD, decode_pasted};

/// Bundle a site admin's token with the same family as a pasted App key.
#[must_use]
pub fn pasted_admin_token(token: &[u8]) -> Option<skein_lib::Secret> {
    temper_protocol_records::encode_pasted(&[(1, token)]).ok()
}

/// A connected Forgejo's pasted tokens and hooks' secret, one bundle
/// (protocol/credentials.md, 2.2): `ApiToken`, `GitToken`, `WebhookSecret`, each when given.
#[must_use]
pub fn pasted_tokens(api: Option<&[u8]>, git: Option<&[u8]>, secret: Option<&[u8]>) -> Option<skein_lib::Secret> {
    let mut items = skein_lib::List::with_capacity(3);
    for (code, value) in [(2, api), (3, git), (5, secret)] {
        if let Some(value) = value
            && !value.is_empty()
        {
            items.push((code, value)).expect("at most three items");
        }
    }
    temper_protocol_records::encode_pasted(items.as_slice()).ok()
}
