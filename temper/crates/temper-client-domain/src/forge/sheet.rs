//! Add repositories: reads, selections and sends (temper/docs/design/client/forge/family.md, 4 and 5 and temper/docs/design/client/forge/records.md
//! ; temper/docs/design/client/forge/repositories.md, 1).
use super::{
    Field, Limits, Request, Side, Typed,
    forges::{Connection, Kind, Mode, Row, State},
};
use alloc::boxed::Box;
use skein_lib::{List, Queue, Token, bytes};
use temper_client_protocol_wire::{self as wire, v1 as w};

pub const PICKER_ROWS: usize = 2000;
const CAP: u32 = 2000;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Source {
    Forge,
    Empty,
    Copy,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pick {
    pub id: u64,
    pub owner_id: u64,
    pub owner: Box<[u8]>,
    pub name: Box<[u8]>,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Reading {
    Idle,
    Asked { key: Token, page: u32 },
    Refused(Refused),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Refused {
    NotConnected,
    Busy,
    NoAnswer,
    AdminMissing,
    NotAdmin { name: Box<[u8]> },
    Role,
    Words(Box<[u8]>),
}
#[derive(Debug)]
#[expect(clippy::partial_pub_fields, reason = "the view reads the sheet; read identity and send queues stay private")]
pub struct Sheet {
    pub project: u32,
    pub source: Source,
    pub forge: Option<u16>,
    pub installation: Option<u64>,
    pub target: Option<u16>,
    pub find: Field,
    pub rows: List<w::Reached>,
    pub page: u32,
    pub more: bool,
    pub reading: Reading,
    pub picks: List<Pick>,
    pub not_listed: (Field, Field),
    pub org: Field,
    pub name: Field,
    pub addresses: Field,
    pub sent: Sent,
    adopt_when_read: Option<Pick>,
    read_forge: Option<u16>,
    read_installation: Option<u64>,
    home: bool,
    making: List<w::MakeRepository>,
    not_listed_edited: bool,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Sent {
    Idle,
    Adopting { left: List<Pick> },
    Making { left: u32 },
    Refused(Box<[u8]>),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Act {
    Source(Source),
    Forge(u16),
    Installation(u64),
    Find(Box<[u8]>),
    Pick { id: u64, on: bool },
    NearEnd,
    NotListed { owner: Option<Box<[u8]>>, name: Option<Box<[u8]>> },
    Org(Box<[u8]>),
    Name(Box<[u8]>),
    Addresses(Box<[u8]>),
    Target(u16),
    UseIt { line: Box<[u8]> },
    AdoptIt { org: Box<[u8]>, name: Box<[u8]> },
    CopyAgain { forge: u16, org: Box<[u8]>, name: Box<[u8]> },
    AdoptAgain { forge: u16, org: Box<[u8]>, name: Box<[u8]> },
    MakeAgain { forge: u16, org: Box<[u8]>, name: Box<[u8]> },
    Dismiss { forge: u16, org: Box<[u8]>, name: Box<[u8]> },
    Send,
}
#[derive(Debug)]
pub struct Shown<'a> {
    pub row: &'a w::Reached,
    pub mark: Mark,
    pub picked: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mark {
    None,
    Private,
    InProject,
    Copying,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Count {
    Hidden,
    SoFar(u32),
    Found { found: u32, of: u32 },
    Searching(u32),
    All(u32),
    Suspended,
    Full,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Remedy {
    Grant { owner: Box<[u8]>, name: Box<[u8]>, page: Box<[u8]> },
    NotInstalled { owner: Box<[u8]> },
    Collaborator { login: Box<[u8]>, owner: Box<[u8]>, name: Box<[u8]> },
}
#[derive(Debug, PartialEq, Eq)]
pub enum Button {
    Adopt { count: u32, first: Box<[u8]> },
    Make { full: Box<[u8]> },
    Copy { count: u32, first: Box<[u8]> },
}

impl Sheet {
    #[must_use]
    pub fn new(project: u32, rows: &State) -> Self {
        let mut sheet = Self {
            project,
            source: Source::Forge,
            forge: None,
            installation: None,
            target: None,
            find: field(b""),
            rows: List::with_capacity(CAP),
            page: 0,
            more: true,
            reading: Reading::Idle,
            picks: List::with_capacity(CAP),
            not_listed: (field(b""), field(b"")),
            org: field(b"work"),
            name: field(b""),
            addresses: field(b""),
            sent: Sent::Idle,
            adopt_when_read: None,
            read_forge: None,
            read_installation: None,
            home: true,
            making: List::with_capacity(CAP),
            not_listed_edited: false,
        };
        sheet.defaults(rows);
        sheet
    }
    fn defaults(&mut self, rows: &State) {
        let mut forge = None;
        let mut target = None;
        for row in &super::listed::rows(rows, None) {
            if let Connection::Connected { .. } = row.connection
                && (forge.is_none() || forge > Some(row.number))
            {
                forge = Some(row.number);
            }
            if makes(row) && (target.is_none() || target > Some(row.number)) {
                target = Some(row.number);
            }
            for repository in &row.repositories {
                if repository.project == self.project {
                    self.home = false;
                }
            }
        }
        if self.forge.is_none() {
            self.forge = forge;
        }
        if self.target.is_none() {
            self.target = target;
        }
        if self.installation.is_none() {
            self.installation = first_installation(self.forge, rows);
        }
    }
    fn reset(&mut self) {
        self.rows.clear();
        self.picks.clear();
        put(&mut self.find, Box::new([]), 0);
        self.page = 0;
        self.more = true;
        self.reading = Reading::Idle;
        put(&mut self.not_listed.0, Box::new([]), 0);
        put(&mut self.not_listed.1, Box::new([]), 0);
        self.not_listed_edited = false;
        self.adopt_when_read = None;
    }
    fn chosen(&self) -> Option<u16> {
        match self.source {
            Source::Forge => self.forge,
            Source::Copy => self.target,
            Source::Empty => None,
        }
    }
}
fn field(text: &[u8]) -> Field {
    super::form::field(text, Typed::Bytes)
}
fn put(field: &mut Field, text: Box<[u8]>, bound: u32) {
    field.written = field.written.checked_add(1).expect("field writes fit");
    if text.len() > skein_lib::usize_of(bound) {
        field.broken = Some(super::Broken::Long { bound });
    } else {
        field.text = text;
        field.broken = None;
    }
}
#[must_use]
pub fn makes(row: &Row) -> bool {
    row.kind == Kind::Forgejo && (row.mode == Mode::Owned || row.mode == Mode::Managed)
}
#[must_use]
pub fn row(rows: &State, forge: Option<u16>) -> Option<&Row> {
    if let State::Rows(rows) = rows {
        for row in rows {
            if Some(row.number) == forge {
                return Some(row);
            }
        }
    }
    None
}
fn first_installation(forge: Option<u16>, rows: &State) -> Option<u64> {
    let row = row(rows, forge)?;
    if row.kind != Kind::GitHub {
        return None;
    }
    for installation in &row.installations {
        if !installation.stray && installation.account.is_some() {
            return Some(installation.id);
        }
    }
    None
}
fn suspended(sheet: &Sheet, rows: &State) -> bool {
    if let Some(row) = row(rows, sheet.forge) {
        for installation in &row.installations {
            if Some(installation.id) == sheet.installation {
                return installation.suspended;
            }
        }
    }
    false
}
#[must_use]
pub fn long(sheet: &Sheet) -> bool {
    sheet.rows.len() > 8 || sheet.more
}
fn wanted(sheet: &Sheet, row: &w::Reached) -> bool {
    let full = bytes::concat(&[row.owner().login(), b"/", row.name()]);
    let needle = sheet.find.text.as_ref().trim_ascii();
    if needle.is_empty() {
        return true;
    }
    for part in full.windows(needle.len()) {
        if part.eq_ignore_ascii_case(needle) {
            return true;
        }
    }
    false
}
#[must_use]
pub fn shown<'a>(sheet: &'a Sheet, project: u32, rows: &State) -> List<Shown<'a>> {
    let mut shown = List::with_capacity(CAP);
    for reached in &sheet.rows {
        if wanted(sheet, reached) {
            let mut mark = if reached.private() { Mark::Private } else { Mark::None };
            if let Some(row) = row(rows, sheet.forge) {
                for making in &row.makings {
                    if making.org.as_ref() == reached.owner().login() && making.name.as_ref() == reached.name() {
                        let taken = match &making.state {
                            w::MakingState::Failed(failed) => *failed.reason() == w::MakeFailure::Taken,
                            w::MakingState::Waiting
                            | w::MakingState::Making
                            | w::MakingState::Made(_)
                            | w::MakingState::Adopting(_) => false,
                        };
                        if !taken {
                            mark = Mark::Copying;
                        }
                    }
                }
                for repository in &row.repositories {
                    if repository.project == project
                        && repository.owner.as_ref() == reached.owner().login()
                        && repository.name.as_ref() == reached.name()
                    {
                        mark = Mark::InProject;
                    }
                }
            }
            let mut picked = false;
            for pick in &sheet.picks {
                picked |= pick.id == reached.id();
            }
            shown.push(Shown { row: reached, mark, picked }).expect("bounded rows");
        }
    }
    shown
}
#[must_use]
pub fn count(sheet: &Sheet, rows: &State) -> Count {
    if suspended(sheet, rows) {
        return Count::Suspended;
    }
    if sheet.rows.len() == CAP {
        return Count::Full;
    }
    if !long(sheet) {
        return Count::Hidden;
    }
    if sheet.find.text.trim_ascii().is_empty() {
        if sheet.more { Count::SoFar(sheet.rows.len()) } else { Count::All(sheet.rows.len()) }
    } else if sheet.more {
        Count::Searching(sheet.rows.len())
    } else {
        let found = shown(sheet, sheet.project, rows).len();
        if found == 0 { Count::Hidden } else { Count::Found { found, of: sheet.rows.len() } }
    }
}
#[must_use]
pub fn remedy(sheet: &Sheet, row: &Row) -> Option<Remedy> {
    let owner = sheet.not_listed.0.text.trim_ascii();
    let name = sheet.not_listed.1.text.trim_ascii();
    if owner.is_empty() || name.is_empty() {
        return None;
    }
    for reached in &sheet.rows {
        if reached.owner().login() == owner && reached.name() == name {
            return None;
        }
    }
    match row.kind {
        Kind::GitHub => {
            for installation in &row.installations {
                if !installation.stray
                    && let Some(account) = &installation.account
                    && account.login.eq_ignore_ascii_case(owner)
                {
                    let page = match super::forges::slug(row) {
                        Some(slug) if !slug.is_empty() => {
                            bytes::concat(&[b"https://github.com/apps/", slug, b"/installations/new"])
                        }
                        Some(_) | None => Box::from(b"".as_slice()),
                    };
                    return Some(Remedy::Grant { owner: Box::from(owner), name: Box::from(name), page });
                }
            }
            Some(Remedy::NotInstalled { owner: Box::from(owner) })
        }
        Kind::Forgejo => match row.mode {
            Mode::Owned | Mode::Managed | Mode::Fake => None,
            Mode::Connected => match &row.connection {
                Connection::Connected { login, .. } => {
                    Some(Remedy::Collaborator { login: login.clone(), owner: Box::from(owner), name: Box::from(name) })
                }
                Connection::NotConnected
                | Connection::NotSetUp
                | Connection::SettingUp { .. }
                | Connection::SetUpFailed { .. }
                | Connection::Connecting
                | Connection::Stopped { .. } => None,
            },
        },
    }
}
#[must_use]
pub fn lines(sheet: &Sheet, rows: &State) -> List<super::making::Line> {
    super::making::lines(&sheet.addresses.text, &|address, org, name| {
        if let Some(row) = row(rows, sheet.target) {
            for repository in &row.repositories {
                if repository.owner.as_ref() == org && repository.name.as_ref() == name {
                    return true;
                }
            }
            let mut replaceable = false;
            for making in &row.makings {
                if making.org.as_ref() == org && making.name.as_ref() == name {
                    match &making.state {
                        w::MakingState::Failed(failed) => match failed.reason() {
                            w::MakeFailure::Taken => {}
                            w::MakeFailure::Empty
                            | w::MakeFailure::TimedOut
                            | w::MakeFailure::Refused(_)
                            | w::MakeFailure::NotAllowed
                            | w::MakeFailure::Other => match &making.from {
                                w::MakeFrom::Empty => {}
                                w::MakeFrom::Copy(copy) => {
                                    if let Ok(source) = wire::source::parse(copy.address()) {
                                        replaceable |= wire::source::address(&source).as_ref() == address;
                                    }
                                }
                            },
                        },
                        w::MakingState::Waiting
                        | w::MakingState::Making
                        | w::MakingState::Made(_)
                        | w::MakingState::Adopting(_) => return true,
                    }
                }
            }
            // temper/docs/design/client/forge/family.md, 4: copying again replaces
            // the same source's empty repository.
            if replaceable {
                return false;
            }
        }
        for reached in &sheet.rows {
            if reached.owner().login() == org && reached.name() == name {
                return true;
            }
        }
        false
    })
}
#[must_use]
pub fn button(sheet: &Sheet, form: &super::Form, rows: &State) -> (Button, bool) {
    let idle = form.pending.is_none() && form.confirmed && form.served != super::Served::OutOfDate;
    match sheet.source {
        Source::Forge => {
            let mut count = sheet.picks.len();
            let mut first: Box<[u8]> = Box::from(b"".as_slice());
            if let Some(pick) = sheet.picks.get(0) {
                first = bytes::concat(&[&pick.owner, b"/", &pick.name]);
            }
            let named =
                !sheet.not_listed.0.text.trim_ascii().is_empty() && !sheet.not_listed.1.text.trim_ascii().is_empty();
            if named {
                count = count.checked_add(1).expect("bounded picks");
                if first.is_empty() {
                    first = bytes::concat(&[&sheet.not_listed.0.text, b"/", &sheet.not_listed.1.text]);
                }
            }
            let mut enabled = idle && count > 0 && !suspended(sheet, rows);
            if let Some(row) = row(rows, sheet.forge) {
                enabled &= matches_connection(&row.connection);
                if named && form.role != super::Role::Context {
                    enabled &= remedy(sheet, row).is_none();
                }
            } else {
                enabled = false;
            }
            (Button::Adopt { count, first }, enabled)
        }
        Source::Empty => {
            let full = bytes::concat(&[&sheet.org.text, b"/", &sheet.name.text]);
            let valid =
                super::making::ask(sheet.target.unwrap_or(0), &sheet.org.text, &sheet.name.text, None, None).is_ok();
            (
                Button::Make { full },
                idle && sheet.target.is_some() && valid && sheet.org.broken.is_none() && sheet.name.broken.is_none(),
            )
        }
        Source::Copy => {
            let lines = lines(sheet, rows);
            let mut valid = !lines.is_empty() && sheet.addresses.broken.is_none();
            let mut first: Box<[u8]> = Box::from(b"".as_slice());
            for line in &lines {
                match line {
                    super::making::Line::To { org, name, .. } => {
                        if first.is_empty() {
                            first = bytes::concat(&[org, b"/", name]);
                        }
                    }
                    super::making::Line::Bad { raw } | super::making::Line::NotAHost { raw, .. } => {
                        if first.is_empty() {
                            first = super::words::lossy(raw);
                        }
                        valid = false;
                    }
                    super::making::Line::Taken { org, name, .. } => {
                        if first.is_empty() {
                            first = bytes::concat(&[org, b"/", name]);
                        }
                        valid = false;
                    }
                }
            }
            (Button::Copy { count: lines.len(), first }, idle && sheet.target.is_some() && valid)
        }
    }
}
fn matches_connection(connection: &Connection) -> bool {
    match connection {
        Connection::Connected { .. } => true,
        Connection::NotConnected
        | Connection::NotSetUp
        | Connection::SettingUp { .. }
        | Connection::SetUpFailed { .. }
        | Connection::Connecting
        | Connection::Stopped { .. } => false,
    }
}

pub(crate) fn progress(side: &mut Side, limits: Limits, out: &mut Queue<Request>) {
    if let State::Rows(_) = &side.forges {
        let mut kept = List::uncapped();
        for dismissal in &side.sheet_dismissals {
            if making(side, dismissal.forge, &dismissal.org, &dismissal.name).is_some() {
                kept.push(dismissal.clone()).expect("uncapped dismissals");
            }
        }
        side.sheet_dismissals = kept;
    }
    let Some(sheet) = &mut side.sheet else { return };
    sheet.defaults(&side.forges);
    if let Some(form) = &mut side.form
        && form.pending.is_none()
    {
        form.home.on = sheet.home && form.home.on;
    }
    if sheet.page == 0 {
        read(side, limits, out);
    }
}
fn read(side: &mut Side, limits: Limits, out: &mut Queue<Request>) {
    let Some(sheet) = &mut side.sheet else { return };
    if side.served == super::Served::OutOfDate || !sheet.more || sheet.rows.len() == CAP {
        return;
    }
    match sheet.reading {
        Reading::Asked { .. } | Reading::Refused(_) => return,
        Reading::Idle => {}
    }
    let Some(forge) = sheet.chosen() else { return };
    let installation = match sheet.source {
        Source::Forge => sheet.installation,
        Source::Copy | Source::Empty => None,
    };
    let page = sheet.page.checked_add(1).expect("bounded pages");
    let Some(bounds) = super::wire_limits(&limits) else { return };
    let Some(record) = super::family::reachable(forge, installation, page, &bounds) else { return };
    side.serial = side.serial.checked_add(1).expect("request serial fits");
    let key = Token::new(side.serial);
    sheet.reading = Reading::Asked { key, page };
    sheet.read_forge = Some(forge);
    sheet.read_installation = installation;
    out.push(Request::Send {
        request: key,
        name: None,
        ask: jig_client::Ask::Family { scope: jig_client::contract::Scope::Deployment, record },
    });
}
#[expect(clippy::too_many_lines, reason = "the sheet contract lists its independent field acts in one dispatch")]
pub(crate) fn act(side: &mut Side, limits: Limits, act: Act, out: &mut Queue<Request>) {
    match &act {
        Act::Dismiss { forge, org, name } => {
            dismiss(side, limits, *forge, org, name, out);
            return;
        }
        Act::CopyAgain { forge, org, name }
        | Act::AdoptAgain { forge, org, name }
        | Act::MakeAgain { forge, org, name } => {
            let Some(making) = making(side, *forge, org, name) else { return };
            let Some(into) = &making.into else { return };
            let w::MakingState::Failed(_) = making.state else { return };
            let project = into.project();
            side.sheet_again = Some(act);
            out.push(Request::Show { address: super::Address::Adopt { project } });
            return;
        }
        Act::Source(_)
        | Act::Forge(_)
        | Act::Installation(_)
        | Act::Find(_)
        | Act::Pick { .. }
        | Act::NearEnd
        | Act::NotListed { .. }
        | Act::Org(_)
        | Act::Name(_)
        | Act::Addresses(_)
        | Act::Target(_)
        | Act::UseIt { .. }
        | Act::AdoptIt { .. }
        | Act::Send => {}
    }
    let Some(sheet) = &mut side.sheet else { return };
    if let Some(form) = &side.form
        && form.pending.is_some()
    {
        return;
    }
    match act {
        Act::CopyAgain { .. } | Act::AdoptAgain { .. } | Act::MakeAgain { .. } | Act::Dismiss { .. } => return,
        Act::Send => {
            send(side, limits, out);
            return;
        }
        Act::Source(source) => {
            if source != Source::Forge && row(&side.forges, sheet.target).is_none() {
                return;
            }
            if sheet.source != source {
                sheet.source = source;
                sheet.reset();
            }
        }
        Act::Forge(forge) => {
            sheet.forge = Some(forge);
            sheet.installation = first_installation(sheet.forge, &side.forges);
            sheet.reset();
        }
        Act::Installation(installation) => {
            sheet.installation = Some(installation);
            sheet.reset();
        }
        Act::Target(target) => {
            if let Some(row) = row(&side.forges, Some(target))
                && makes(row)
            {
                sheet.target = Some(target);
                sheet.reset();
            }
        }
        Act::Find(text) => {
            put(&mut sheet.find, text, limits.field_bytes);
            fill_not_listed(sheet, &side.forges, limits.field_bytes);
        }
        Act::Pick { id, on } => {
            if on {
                for shown in &shown(sheet, sheet.project, &side.forges) {
                    if shown.row.id() == id
                        && !shown.picked
                        && (shown.mark == Mark::None || shown.mark == Mark::Private)
                    {
                        sheet.picks.push(pick(shown.row)).expect("bounded picks");
                        break;
                    }
                }
            } else {
                remove_pick(&mut sheet.picks, id);
            }
        }
        Act::NearEnd => {
            read(side, limits, out);
            side.changed();
            return;
        }
        Act::NotListed { owner, name } => {
            sheet.not_listed_edited = true;
            if let Some(owner) = owner {
                put(&mut sheet.not_listed.0, owner, limits.field_bytes);
            }
            if let Some(name) = name {
                put(&mut sheet.not_listed.1, name, limits.field_bytes);
            }
        }
        Act::Org(text) => put(&mut sheet.org, text, limits.field_bytes),
        Act::Name(text) => put(&mut sheet.name, text, limits.field_bytes),
        Act::Addresses(text) => put(&mut sheet.addresses, text, limits.field_bytes),
        Act::UseIt { line } => {
            let parsed = lines(sheet, &side.forges);
            for candidate in &parsed {
                if let super::making::Line::NotAHost { raw, line: fixed } = candidate
                    && *fixed == line
                {
                    rewrite(&mut sheet.addresses, raw, Some(fixed), limits.field_bytes);
                }
            }
        }
        Act::AdoptIt { org, name } => {
            let parsed = lines(sheet, &side.forges);
            for candidate in &parsed {
                if let super::making::Line::Taken { raw, org: taken_org, name: taken_name } = candidate
                    && *taken_org == org
                    && *taken_name == name
                {
                    rewrite(&mut sheet.addresses, raw, None, limits.field_bytes);
                }
            }
            sheet.source = Source::Forge;
            sheet.forge = sheet.target;
            sheet.installation = None;
            sheet.reset();
            sheet.adopt_when_read = Some(Pick { id: 0, owner_id: 0, owner: org, name });
        }
    }
    sheet.sent = Sent::Idle;
    let search = !sheet.find.text.trim_ascii().is_empty();
    if sheet.page == 0 || search || sheet.adopt_when_read.is_some() {
        read(side, limits, out);
    }
    side.changed();
}
fn pick(row: &w::Reached) -> Pick {
    Pick {
        id: row.id(),
        owner_id: row.owner().id(),
        owner: Box::from(row.owner().login()),
        name: Box::from(row.name()),
    }
}
fn remove_pick(picks: &mut List<Pick>, id: u64) {
    let mut kept = List::with_capacity(CAP);
    for pick in picks.as_slice() {
        if pick.id != id {
            kept.push(pick.clone()).expect("bounded picks");
        }
    }
    *picks = kept;
}
#[expect(clippy::disallowed_methods, reason = "line repair compares trimmed Unicode input as the parser does")]
fn rewrite(field: &mut Field, raw: &[u8], replacement: Option<&[u8]>, bound: u32) {
    let Ok(text) = core::str::from_utf8(&field.text) else { return };
    let capacity = bound.checked_mul(2).expect("line fragments fit");
    let mut parts = List::with_capacity(capacity);
    let mut first = true;
    for line in text.split('\n') {
        let value = if line.trim().as_bytes() == raw {
            match replacement {
                Some(value) => value,
                None => continue,
            }
        } else {
            line.as_bytes()
        };
        if !first {
            parts.push(b"\n".as_slice()).expect("bounded separators");
        }
        first = false;
        parts.push(value).expect("bounded lines");
    }
    let text = bytes::concat(parts.as_slice());
    put(field, text, bound);
}

/// Receive only the read currently owned by this sheet. Earlier choices' keys are retired.
pub(crate) fn received(
    side: &mut Side,
    key: Token,
    answer: Result<w::Page, Refused>,
    limits: Limits,
    out: &mut Queue<Request>,
) {
    let Some(sheet) = &mut side.sheet else { return };
    let page = match sheet.reading {
        Reading::Asked { key: pending, page } if key == pending => page,
        Reading::Asked { .. } | Reading::Idle | Reading::Refused(_) => return,
    };
    let installation = if sheet.source == Source::Forge { sheet.installation } else { None };
    if sheet.read_forge != sheet.chosen() || sheet.read_installation != installation {
        return;
    }
    match answer {
        Ok(answer) => {
            let answer = answer.into_parts();
            sheet.more = answer.more && !answer.rows.is_empty();
            sheet.page = page;
            sheet.reading = Reading::Idle;
            for row in answer.rows.into_boxed() {
                if sheet.rows.push(row).is_err() {
                    break;
                }
            }
            if let Some(intent) = &sheet.adopt_when_read {
                for row in &sheet.rows {
                    if row.owner().login() == intent.owner.as_ref() && row.name() == intent.name.as_ref() {
                        sheet.picks.push(pick(row)).expect("one pick after reset");
                        sheet.adopt_when_read = None;
                        break;
                    }
                }
            }
            fill_not_listed(sheet, &side.forges, limits.field_bytes);
            if !sheet.find.text.trim_ascii().is_empty() || sheet.adopt_when_read.is_some() {
                read(side, limits, out);
            }
        }
        Err(why) => sheet.reading = Reading::Refused(why),
    }
    side.changed();
}
fn fill_not_listed(sheet: &mut Sheet, rows: &State, bound: u32) {
    if sheet.not_listed_edited {
        return;
    }
    if sheet.more || sheet.find.text.trim_ascii().is_empty() || !shown(sheet, sheet.project, rows).is_empty() {
        put(&mut sheet.not_listed.0, Box::new([]), 0);
        put(&mut sheet.not_listed.1, Box::new([]), 0);
        return;
    }
    let typed = sheet.find.text.trim_ascii();
    if let Some(slash) = bytes::find(typed, b"/") {
        let after = slash.checked_add(1).expect("slash fits");
        put(&mut sheet.not_listed.0, Box::from(typed.get(..slash).expect("found slash")), bound);
        put(&mut sheet.not_listed.1, Box::from(typed.get(after..).expect("found slash")), bound);
    } else {
        if let Some(row) = row(rows, sheet.forge) {
            for installation in &row.installations {
                if Some(installation.id) == sheet.installation
                    && let Some(account) = &installation.account
                {
                    put(&mut sheet.not_listed.0, account.login.clone(), bound);
                }
            }
        }
        put(&mut sheet.not_listed.1, Box::from(typed), bound);
    }
}
fn send(side: &mut Side, limits: Limits, out: &mut Queue<Request>) {
    let Some(sheet) = &mut side.sheet else { return };
    let Some(form) = &mut side.form else { return };
    if !button(sheet, form, &side.forges).1 {
        return;
    }
    match sheet.source {
        Source::Forge => {
            let mut left = sheet.picks.clone();
            if !sheet.not_listed.0.text.trim_ascii().is_empty() && !sheet.not_listed.1.text.trim_ascii().is_empty() {
                let named = Pick {
                    id: 0,
                    owner_id: 0,
                    owner: Box::from(sheet.not_listed.0.text.trim_ascii()),
                    name: Box::from(sheet.not_listed.1.text.trim_ascii()),
                };
                let mut duplicate = false;
                for pick in &left {
                    duplicate |= pick.owner == named.owner && pick.name == named.name;
                }
                if !duplicate && left.push(named).is_err() {
                    return;
                }
            }
            sheet.sent = Sent::Adopting { left };
        }
        Source::Empty | Source::Copy => {
            let Some(forge) = sheet.target else { return };
            sheet.making.clear();
            let mut requests = List::with_capacity(CAP);
            match sheet.source {
                Source::Empty => {
                    let into =
                        w::MakeInto::new(&w::CEILINGS, w::MakeIntoParts { project: sheet.project, home: sheet.home })
                            .expect("project fits");
                    let Ok(request) = super::making::ask(forge, &sheet.org.text, &sheet.name.text, None, Some(into))
                    else {
                        return;
                    };
                    requests.push(request).expect("one request");
                }
                Source::Copy => {
                    for line in &lines(sheet, &side.forges) {
                        match line {
                            super::making::Line::To { address, org, name } => {
                                let into = w::MakeInto::new(
                                    &w::CEILINGS,
                                    w::MakeIntoParts {
                                        project: sheet.project,
                                        home: sheet.home && requests.is_empty(),
                                    },
                                )
                                .expect("project fits");
                                let Ok(request) = super::making::ask(forge, org, name, Some(address), Some(into))
                                else {
                                    return;
                                };
                                if requests.push(request).is_err() {
                                    return;
                                }
                            }
                            super::making::Line::Bad { .. }
                            | super::making::Line::NotAHost { .. }
                            | super::making::Line::Taken { .. } => return,
                        }
                    }
                }
                Source::Forge => unreachable!("making sources"),
            }
            sheet.sent = Sent::Making { left: requests.len() };
            sheet.making = requests;
        }
    }
    send_next(side, limits, out);
    side.changed();
}
fn send_next(side: &mut Side, limits: Limits, out: &mut Queue<Request>) {
    let Some(sheet) = &mut side.sheet else { return };
    let Some(form) = &mut side.form else { return };
    let ask = match &sheet.sent {
        Sent::Adopting { left } => {
            let Some(pick) = left.get(0) else {
                close(sheet.project, out);
                return;
            };
            let Some(forge) = sheet.forge else { return };
            let number = skein_lib::Decimal::of(u64::from(forge));
            for (text, value, typed) in [
                (super::Text::Forge, number.as_bytes(), Typed::Forge(Some(forge))),
                (super::Text::Owner, pick.owner.as_ref(), Typed::Bytes),
                (super::Text::Name, pick.name.as_ref(), Typed::Bytes),
            ] {
                super::write(form, text, Box::from(value), typed, limits.field_bytes, limits.checks);
            }
            form.home.on = sheet.home && form.home.on && left.len() == 1;
            super::form::mark(form, limits.checks);
            let Some(adoption) = super::adoption(form, limits.checks) else {
                sheet.sent = Sent::Idle;
                return;
            };
            let Some(ask) = super::ask(&adoption, &limits) else { return };
            ask
        }
        Sent::Making { left } => {
            if *left == 0 {
                close(sheet.project, out);
                return;
            }
            let Some(request) = sheet.making.get(0) else { return };
            let Some(ask) = super::make_repository(request, &limits) else { return };
            ask
        }
        Sent::Idle | Sent::Refused(_) => return,
    };
    side.serial = side.serial.checked_add(1).expect("request serial fits");
    let request = Token::new(side.serial);
    form.pending = Some((request, skein_lib::RequestProgress::Parked));
    out.push(Request::Send { request, name: None, ask });
}
fn close(project: u32, out: &mut Queue<Request>) {
    out.push(Request::Repositories { project });
}
fn pending_is(form: &super::Form, request: Token) -> bool {
    match form.pending {
        Some((key, _)) => key == request,
        None => false,
    }
}

pub(crate) fn adopted(
    side: &mut Side,
    request: Token,
    answer: &super::Answer,
    limits: Limits,
    out: &mut Queue<Request>,
) -> bool {
    let Some(sheet) = &mut side.sheet else { return false };
    let Some(form) = &mut side.form else { return false };
    if !pending_is(form, request) {
        return false;
    }
    let Sent::Adopting { left } = &mut sheet.sent else { return false };
    form.pending = None;
    match answer {
        super::Answer::Adopted(_) => {
            if let Some(done) = left.get(0) {
                remove_pick(&mut sheet.picks, done.id);
                if sheet.not_listed.0.text == done.owner && sheet.not_listed.1.text == done.name {
                    put(&mut sheet.not_listed.0, Box::new([]), 0);
                    put(&mut sheet.not_listed.1, Box::new([]), 0);
                    sheet.not_listed_edited = false;
                }
            }
            let mut rest = List::with_capacity(CAP);
            for pick in left.as_slice().get(1..).unwrap_or(&[]) {
                rest.push(pick.clone()).expect("bounded picks");
            }
            *left = rest;
            sheet.home = false;
            form.home.on = false;
        }
        super::Answer::CoreRefused(_)
        | super::Answer::ForgeRefused(_)
        | super::Answer::EntranceRefused(_)
        | super::Answer::Unknown => {
            sheet.sent = Sent::Refused(super::words::sheet_adoption(answer));
        }
    }
    send_next(side, limits, out);
    side.changed();
    true
}
#[expect(clippy::disallowed_methods, reason = "a kept copy is removed by parsing the person-entered address line")]
pub(crate) fn made(
    side: &mut Side,
    request: Token,
    answer: Result<(), Box<[u8]>>,
    limits: Limits,
    out: &mut Queue<Request>,
) {
    let Some(sheet) = &mut side.sheet else { return };
    let Some(form) = &mut side.form else { return };
    if !pending_is(form, request) {
        return;
    }
    let Sent::Making { left } = &mut sheet.sent else { return };
    form.pending = None;
    match answer {
        Err(words) => sheet.sent = Sent::Refused(words),
        Ok(()) => {
            if let Some(done) = sheet.making.get(0) {
                match done.from() {
                    w::MakeFrom::Copy(copy) => {
                        let Ok(text) = core::str::from_utf8(&sheet.addresses.text) else { return };
                        let mut raw = None;
                        for line in text.split('\n') {
                            let parsed = super::making::parse_copies(line.as_bytes());
                            if let Ok(parsed) = parsed
                                && let Some(address) = parsed.get(0)
                                && address.as_ref() == copy.address()
                            {
                                raw = Some(Box::from(line.as_bytes().trim_ascii()));
                                break;
                            }
                        }
                        if let Some(raw) = raw {
                            rewrite(&mut sheet.addresses, &raw, None, limits.field_bytes);
                        }
                    }
                    w::MakeFrom::Empty => put(&mut sheet.name, Box::new([]), 0),
                }
            }
            let mut rest = List::with_capacity(CAP);
            for request in sheet.making.as_slice().get(1..).unwrap_or(&[]) {
                rest.push(request.clone()).expect("bounded requests");
            }
            sheet.making = rest;
            *left = left.checked_sub(1).expect("one pending making");
            sheet.home = false;
        }
    }
    send_next(side, limits, out);
    side.changed();
}

/// A dismissal waits for the watch to remove the row; refusals belong to that row.
#[derive(Clone, Debug)]
pub struct Dismissal {
    pub forge: u16,
    pub org: Box<[u8]>,
    pub name: Box<[u8]>,
    pub pending: Option<Token>,
    pub words: Option<Box<[u8]>>,
}
#[expect(clippy::manual_find, reason = "step code searches with a loop rather than a closure")]
pub(crate) fn dismissal<'a>(side: &'a Side, forge: u16, org: &[u8], name: &[u8]) -> Option<&'a Dismissal> {
    for dismissal in &side.sheet_dismissals {
        if dismissal.forge == forge && dismissal.org.as_ref() == org && dismissal.name.as_ref() == name {
            return Some(dismissal);
        }
    }
    None
}
#[expect(clippy::manual_find, reason = "step code searches with a loop rather than a closure")]
fn making<'a>(side: &'a Side, forge: u16, org: &[u8], name: &[u8]) -> Option<&'a super::making::Making> {
    for making in &row(&side.forges, Some(forge))?.makings {
        if making.org.as_ref() == org && making.name.as_ref() == name {
            return Some(making);
        }
    }
    None
}
fn dismiss(side: &mut Side, limits: Limits, forge: u16, org: &[u8], name: &[u8], out: &mut Queue<Request>) {
    let Some(making) = making(side, forge, org, name) else { return };
    let w::MakingState::Failed(_) = making.state else { return };
    if let Some(dismissal) = dismissal(side, forge, org, name)
        && dismissal.pending.is_some()
    {
        return;
    }
    let project = match &making.into {
        Some(into) => Some(into.project()),
        None => None,
    };
    let Some(bounds) = super::wire_limits(&limits) else { return };
    let Some(record) = super::dismiss_making(forge, org, name, project, &bounds) else { return };
    let scope = match project {
        Some(project) => jig_client::contract::Scope::Project(project),
        None => jig_client::contract::Scope::Deployment,
    };
    side.serial = side.serial.checked_add(1).expect("request serial fits");
    let request = Token::new(side.serial);
    let mut found = false;
    for dismissal in side.sheet_dismissals.iter_mut() {
        if dismissal.forge == forge && dismissal.org.as_ref() == org && dismissal.name.as_ref() == name {
            dismissal.pending = Some(request);
            dismissal.words = None;
            found = true;
        }
    }
    if !found {
        side.sheet_dismissals
            .push(Dismissal { forge, org: Box::from(org), name: Box::from(name), pending: Some(request), words: None })
            .expect("uncapped dismissals");
    }
    out.push(Request::Send { request, name: None, ask: jig_client::Ask::Family { scope, record } });
    side.changed();
}
pub(crate) fn again(side: &mut Side, limits: Limits, act: Act, out: &mut Queue<Request>) {
    let (forge, org, name, source) = match act {
        Act::CopyAgain { forge, org, name } => (forge, org, name, Source::Copy),
        Act::AdoptAgain { forge, org, name } => (forge, org, name, Source::Forge),
        Act::MakeAgain { forge, org, name } => (forge, org, name, Source::Empty),
        Act::Dismiss { .. }
        | Act::Source(_)
        | Act::Forge(_)
        | Act::Installation(_)
        | Act::Find(_)
        | Act::Pick { .. }
        | Act::NearEnd
        | Act::NotListed { .. }
        | Act::Org(_)
        | Act::Name(_)
        | Act::Addresses(_)
        | Act::Target(_)
        | Act::UseIt { .. }
        | Act::AdoptIt { .. }
        | Act::Send => return,
    };
    let Some(making) = making(side, forge, &org, &name).cloned() else { return };
    let Some(sheet) = &mut side.sheet else { return };
    let Some(into) = &making.into else { return };
    if into.project() != sheet.project {
        return;
    }
    sheet.source = source;
    sheet.target = Some(forge);
    sheet.forge = Some(forge);
    sheet.installation = None;
    sheet.reset();
    match source {
        Source::Copy => {
            if let w::MakeFrom::Copy(copy) = making.from {
                put(&mut sheet.addresses, Box::from(copy.address()), limits.field_bytes);
            }
        }
        Source::Empty => {
            put(&mut sheet.org, org, limits.field_bytes);
            put(&mut sheet.name, name, limits.field_bytes);
        }
        Source::Forge => sheet.adopt_when_read = Some(Pick { id: 0, owner_id: 0, owner: org, name }),
    }
    read(side, limits, out);
    side.changed();
}

/// Reads and making answers are decoded here before the adoption family's terminal.
pub(crate) fn told(
    side: &mut Side,
    client: &jig_client::Client,
    told: &jig_client::Told,
    limits: Limits,
    out: &mut Queue<Request>,
) -> bool {
    let (key, outcome) = match told {
        jig_client::Told::Answered { key, outcome } => (*key, Some(outcome)),
        jig_client::Told::Pending { key } => {
            if match client.pending(*key) {
                Some(pending) => pending.progress != jig_client::Progress::Unknown,
                None => true,
            } {
                return false;
            }
            (*key, None)
        }
        jig_client::Told::Link { .. }
        | jig_client::Told::Kept { .. }
        | jig_client::Told::Dropped { .. }
        | jig_client::Told::Changed { .. }
        | jig_client::Told::Ended { .. }
        | jig_client::Told::Text { .. }
        | jig_client::Told::Page { .. } => return false,
    };
    let token = Token::new(key.number);
    for dismissal in side.sheet_dismissals.iter_mut() {
        if dismissal.pending == Some(token) {
            dismissal.pending = None;
            dismissal.words = match decode(outcome, limits) {
                Some(w::Outcome::MakeKept) => None,
                Some(value) => Some(super::words::sheet_outcome(&value)),
                None => Some(Box::from(b"The forge did not answer.".as_slice())),
            };
            side.changed();
            return true;
        }
    }
    let Some(sheet) = &side.sheet else { return false };
    let reading = match sheet.reading {
        Reading::Asked { key, .. } => key == token,
        Reading::Idle | Reading::Refused(_) => false,
    };
    let making = match sheet.sent {
        Sent::Making { .. } => match &side.form {
            Some(form) => pending_is(form, token),
            None => false,
        },
        Sent::Idle | Sent::Adopting { .. } | Sent::Refused(_) => false,
    };
    if !reading && !making {
        return false;
    }
    let decoded = decode(outcome, limits);
    if reading {
        let answer = match decoded {
            Some(w::Outcome::Page(page)) => Ok(page),
            Some(w::Outcome::ConnectRefused(why)) => Err(connect_refused(why.why().clone())),
            Some(value) => Err(Refused::Words(super::words::sheet_outcome(&value))),
            None => Err(Refused::NoAnswer),
        };
        received(side, token, answer, limits, out);
    } else {
        let answer = match decoded {
            Some(w::Outcome::MakeKept) => Ok(()),
            Some(value) => Err(super::words::sheet_outcome(&value)),
            None => Err(Box::from(b"The forge did not answer.".as_slice())),
        };
        made(side, token, answer, limits, out);
    }
    true
}
fn decode(outcome: Option<&jig_client::Outcome>, limits: Limits) -> Option<w::Outcome> {
    let jig_client::Outcome::Answered(jig_client::Answered::Family(record)) = outcome? else { return None };
    if record.family != wire::FORGE || record.version != wire::CLIENT_VERSION {
        return None;
    }
    let mut reader = skein_lib::Reader::new(&record.bytes);
    let answer = w::Answer::decode(&super::wire_limits(&limits)?, &mut reader).ok()?;
    if !reader.is_empty() {
        return None;
    }
    Some(answer.into_parts().outcome)
}
fn connect_refused(why: w::ForgeRefusal) -> Refused {
    match why {
        w::ForgeRefusal::Role => Refused::Role,
        w::ForgeRefusal::NotConnected => Refused::NotConnected,
        w::ForgeRefusal::Busy => Refused::Busy,
        w::ForgeRefusal::NoSuchForge
        | w::ForgeRefusal::NotTaken
        | w::ForgeRefusal::Connected
        | w::ForgeRefusal::NoKey
        | w::ForgeRefusal::FirstIsBot => Refused::NoAnswer,
    }
}

#[cfg(test)]
#[path = "../tests/sheet.rs"]
mod tests;
