//! Home's facts, watches and entry look (temper/docs/design/client/pages/README.md, Home).
use super::{Request, welcome};
use crate::{Domain, forge, owner::Owner};
use alloc::boxed::Box;
use forge::dismissed::{self, PersonFact};
use jig_client::{Client, Handle, Key, Latest, Told, contract as c};
use skein_lib::{Env, List, Queue, Wall, bytes};
use temper_client_protocol_wire::{source, v1 as w};

#[derive(Debug)]
#[expect(clippy::partial_pub_fields, reason = "settings is a public read; request state belongs to this page")]
pub struct Home {
    pub settings: Handle,
    project: u32,
    status: Option<Handle>,
    document: Option<Box<w::ProjectStatus>>,
    pub(super) look: Look,
    pub add: Option<AddSheet>,
    skip: Option<Key>,
    skipped: bool,
    skip_refused: bool,
    copies: List<Key>,
}
#[derive(Debug)]
pub(super) enum Look {
    NotSent,
    Out(Key),
    Answered,
}
impl Home {
    pub(super) fn new(project: u32, number: u64, limits: forge::Limits, out: &mut Queue<Request>) -> Self {
        let mut home = Self {
            add: None,
            settings: forge::ways::handle(),
            project,
            status: None,
            document: None,
            look: Look::NotSent,
            skip: None,
            skipped: false,
            skip_refused: false,
            copies: List::uncapped(),
        };
        if let Some(subject) = forge::status::watch(project, &limits) {
            let handle =
                Handle { asker: Owner::Pages.asker(), number: number.checked_add(1).expect("page handles fit") };
            home.status = Some(handle);
            out.push(Request::Client(jig_client::Event::Watch { handle, subject }));
        }
        home
    }
    pub(super) fn leave(&self, out: &mut Queue<Request>) {
        if let Some(handle) = self.status {
            out.push(Request::Client(jig_client::Event::Unwatch { handle }));
        }
    }
    pub(super) fn skip(&mut self, project: u32, limits: forge::Limits, serial: &mut u64, out: &mut Queue<Request>) {
        if self.skip.is_some() {
            return;
        }
        let Some(request) = dismissed::ask(PersonFact::ToBegin { project }, &limits) else { return };
        *serial = serial.checked_add(1).expect("page keys fit");
        let key = Key { asker: Owner::Pages.asker(), number: *serial };
        self.skip = Some(key);
        self.skip_refused = false;
        out.push(Request::Client(jig_client::Event::Send { key, name: None, request }));
    }
    pub(super) fn told(
        &mut self,
        told: &Told,
        client: &Client,
        limits: forge::Limits,
        out: &mut Queue<Request>,
    ) -> bool {
        match told {
            Told::Answered { key, outcome } => {
                for index in 0..self.copies.len() {
                    if self.copies.get(index) == Some(key) {
                        self.copies.swap_remove(index).expect("held copy key");
                        if let Some(sheet) = &mut self.add
                            && sheet.out.is_some()
                        {
                            if let Err(refused) = add_answer(outcome, limits) {
                                sheet.refused = Some(refused);
                            }
                            if self.copies.is_empty() {
                                sheet.out = None;
                                if sheet.refused.is_none() {
                                    self.add = None;
                                }
                            }
                        }
                        return true;
                    }
                }
            }
            Told::Pending { key } => {
                for held in &self.copies {
                    if held == key {
                        return true;
                    }
                }
            }
            Told::Changed { .. }
            | Told::Ended { .. }
            | Told::Link { .. }
            | Told::Kept { .. }
            | Told::Dropped { .. }
            | Told::Text { .. }
            | Told::Page { .. } => {}
        }
        match told {
            Told::Changed { handle } if Some(*handle) == self.status => {
                self.document = match client.document(*handle) {
                    Some(document) if document.state == jig_client::State::Ended => None,
                    Some(document) => match document.latest {
                        Some(Latest::Family(record)) => match forge::status::document(record, limits) {
                            Some(document) if document.project() == self.project => Some(Box::new(document)),
                            Some(_) | None => None,
                        },
                        Some(Latest::Jig(_)) | None => None,
                    },
                    None => None,
                };
                true
            }
            Told::Ended { handle, .. } if Some(*handle) == self.status => {
                self.status = None;
                self.document = None;
                out.push(Request::Client(jig_client::Event::Unwatch { handle: *handle }));
                true
            }
            Told::Pending { key } if self.looking(*key) => true,
            Told::Answered { key, outcome } if self.looking(*key) => {
                match forge::status::answer(outcome, limits) {
                    forge::status::Answer::Looked | forge::status::Answer::Unknown | forge::status::Answer::Refused => {
                        self.look = Look::Answered;
                    }
                }
                true
            }
            Told::Changed { handle } | Told::Ended { handle, .. } => *handle == self.settings,
            Told::Pending { key } if Some(*key) == self.skip => true,
            Told::Answered { key, outcome } if Some(*key) == self.skip => {
                self.skip = None;
                match dismissed::answer(outcome, limits) {
                    dismissed::Answer::Dismissed => self.skipped = true,
                    dismissed::Answer::Unknown => {}
                    dismissed::Answer::Refused => self.skip_refused = true,
                }
                true
            }
            Told::Link { .. }
            | Told::Pending { .. }
            | Told::Answered { .. }
            | Told::Kept { .. }
            | Told::Dropped { .. }
            | Told::Page { .. }
            | Told::Text { .. } => false,
        }
    }
    fn looking(&self, key: Key) -> bool {
        match self.look {
            Look::Out(ours) => ours == key,
            Look::NotSent | Look::Answered => false,
        }
    }
    pub(super) fn shown_status(&self) -> Status {
        match self.look {
            Look::Answered => match &self.document {
                Some(document) => Status::Shown(document.clone()),
                None => Status::Unknown,
            },
            Look::NotSent | Look::Out(_) => Status::Unknown,
        }
    }
    pub(super) fn copying(&self) -> bool {
        !self.copies.is_empty()
    }
    pub(super) fn keep_copy(&mut self, key: Key) {
        self.copies.push(key).expect("internal copy keys");
    }
}

pub(crate) fn configured(client: &Client, project: u32) -> bool {
    let Some(document) = client.document(forge::ways::handle()) else { return false };
    if document.state == jig_client::State::Ended {
        return false;
    }
    let Some(Latest::Jig(c::Delivery::Settings(settings))) = document.latest else { return false };
    for line in &settings.settings {
        let own = match line.setting {
            c::Setting::Application { scope, .. } => scope == project,
            c::Setting::Charter(_)
            | c::Setting::Template(_)
            | c::Setting::Policy(_)
            | c::Setting::Model(_)
            | c::Setting::Rules
            | c::Setting::ProjectPolicy { .. } => false,
        };
        if own && line.follows == c::SettingFollows::Change {
            return true;
        }
    }
    false
}

/// The recent Away spell shaping Home's intro (client/pages/home.md, 1, Decided for you).
#[derive(Debug, PartialEq, Eq)]
pub struct Spell {
    pub since: Wall,
    pub end: Option<Wall>,
    pub landed: Option<u64>,
}

/// The repository sheet belongs to this Home (client/pages/README.md).
#[derive(Debug, PartialEq, Eq)]
pub struct AddSheet {
    pub start: forge::forges::Start,
    pub org: Box<[u8]>,
    pub name: Box<[u8]>,
    pub addresses: Box<[u8]>,
    pub refused: Option<AddRefused>,
    pub out: Option<Key>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AddField {
    Org,
    Name,
    Addresses,
}
#[derive(Debug, PartialEq, Eq)]
pub enum AddRefused {
    Blank,
    Copies(forge::making::CopiesRefused),
    Invalid(forge::making::Refused),
    Make(w::MakeRefusal),
    Unknown,
}
impl AddSheet {
    pub(super) fn new(start: forge::forges::Start) -> Self {
        Self {
            start,
            org: b"work".as_slice().into(),
            name: b"hello".as_slice().into(),
            addresses: Box::default(),
            refused: None,
            out: None,
        }
    }
    pub(super) fn write(&mut self, field: AddField, words: Box<[u8]>, limits: forge::Limits) {
        if self.out.is_some() || words.len() > usize::try_from(limits.field_bytes).expect("field bound fits") {
            return;
        }
        match field {
            AddField::Org => self.org = words,
            AddField::Name => self.name = words,
            AddField::Addresses => self.addresses = words,
        }
        self.refused = None;
    }
    pub(super) fn requests(&self, forge: u16, project: u32) -> Result<List<w::MakeRepository>, AddRefused> {
        use forge::{forges::Start, making};
        let mut requests = List::uncapped();
        match self.start {
            Start::Empty => {
                let org = self.org.trim_ascii();
                let name = self.name.trim_ascii();
                if org.is_empty() || name.is_empty() {
                    return Err(AddRefused::Blank);
                }
                let into = w::MakeInto::new(&w::CEILINGS, w::MakeIntoParts { project, home: true })
                    .expect("project into fits");
                let request = match making::ask(forge, org, name, None, Some(into)) {
                    Ok(request) => request,
                    Err(why) => return Err(AddRefused::Invalid(why)),
                };
                requests.push(request).expect("one empty repository");
            }
            Start::Copies => {
                let addresses = match making::parse_copies(&self.addresses) {
                    Ok(addresses) => addresses,
                    Err(why) => return Err(AddRefused::Copies(why)),
                };
                for address in &addresses {
                    let Some((org, name)) = making::target(address) else { return Err(AddRefused::Unknown) };
                    let into = w::MakeInto::new(&w::CEILINGS, w::MakeIntoParts { project, home: requests.is_empty() })
                        .expect("project into fits");
                    let request = match making::ask(forge, &org, &name, Some(address), Some(into)) {
                        Ok(request) => request,
                        Err(why) => return Err(AddRefused::Invalid(why)),
                    };
                    requests.push(request).expect("admitted copies");
                }
            }
            Start::Nothing => return Err(AddRefused::Unknown),
        }
        Ok(requests)
    }
}
fn add_answer(outcome: &jig_client::Outcome, limits: forge::Limits) -> Result<(), AddRefused> {
    use temper_client_protocol_wire as wire;
    if let jig_client::Outcome::Answered(jig_client::Answered::Family(record)) = outcome
        && record.family == wire::FORGE
        && record.version == wire::CLIENT_VERSION
        && let Some(bounds) = forge::wire_limits(&limits)
    {
        let mut reader = skein_lib::Reader::new(&record.bytes);
        if let Ok(answer) = w::Answer::decode(&bounds, &mut reader)
            && reader.is_empty()
        {
            if answer.outcome() == &w::Outcome::MakeKept {
                return Ok(());
            }
            if let w::Outcome::MakeRefused(refused) = answer.outcome() {
                return Err(AddRefused::Make(refused.why().clone()));
            }
        }
    }
    Err(AddRefused::Unknown)
}

/// What needs the person in this project (temper/docs/design/client/pages/README.md, Home).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Needs {
    Unknown,
    /// The project's cards of the needs' groups; `cut` where the read was cut.
    Count {
        count: u32,
        cut: bool,
    },
}

/// What the person's skip says (temper/docs/design/client/pages/README.md, To begin).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Skip {
    /// The dismissals aren't delivered, or their watch was refused or ended.
    Unknown,
    /// The dismissals name the project, or this page's request is out or answered.
    Skipped,
    NotSkipped,
}

/// The facts Home draws (temper/docs/design/client/pages/README.md, Home).
#[derive(Debug, PartialEq, Eq)]
pub struct Facts {
    pub configured: bool,
    pub repository: Repository,
    pub needs: Needs,
    pub delivered: bool,
    pub skip: Skip,
    pub skip_refused: bool,
    pub status: Status,
    pub decided: Option<super::decided::Shown>,
    pub spell: Option<Spell>,
}
/// Status stays unknown until this entry's look is answered and its document delivered.
#[derive(Debug, PartialEq, Eq)]
pub enum Status {
    Unknown,
    Shown(Box<w::ProjectStatus>),
}
#[derive(Debug, PartialEq, Eq)]
pub enum Repository {
    Unknown,
    /// Copies being made into this project, a line each (client/pages/README.md, Home).
    Copying {
        lines: Box<[CopyLine]>,
    },
    /// The rows' copies in: their names, in the order asked.
    CopiesIn {
        names: Box<[Box<[u8]>]>,
    },
    /// The rows' copies not all asked, after a restart.
    CopyOffer {
        forge: u16,
        addresses: Box<[Box<[u8]>]>,
    },
    In {
        owner: Box<[u8]>,
        name: Box<[u8]>,
    },
    Adopting(welcome::First),
    Offer {
        first: welcome::First,
        refused: Option<forge::Answer>,
    },
    GitHub {
        account: Option<Box<[u8]>>,
    },
    Add {
        forge: u16,
    },
    None,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CopyLine {
    pub org: Box<[u8]>,
    pub name: Box<[u8]>,
    pub state: CopyState,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CopyState {
    Running { host: Box<[u8]> },
    Making,
    Waiting,
    Adopting,
    In,
    Failed { words: Box<[u8]> },
}
impl Facts {
    #[must_use]
    pub fn parts_facts(&self) -> super::parts::HomeFacts {
        super::parts::HomeFacts {
            across: false,
            follows: 0,
            projects: 0,
            brief: self.brief().is_some(),
            open: match self.needs {
                Needs::Count { count, .. } => count,
                Needs::Unknown => 0,
            },
            showing: match &self.decided {
                Some(away) => away.showing,
                None => 0,
            },
            to_begin: self.to_begin(),
        }
    }

    /// Changes landed since the previous sitting, zero until status is shown.
    #[must_use]
    pub fn landed_since(&self) -> u64 {
        match &self.status {
            Status::Shown(document) => document.landed().saturating_sub(document.seen()),
            Status::Unknown => 0,
        }
    }
    /// The brief that covers new landings or was written since the previous sitting.
    #[must_use]
    pub fn brief(&self) -> Option<&w::StatusBrief> {
        match &self.status {
            Status::Shown(document) => match document.brief() {
                Some(brief)
                    if brief.landed() > document.seen()
                        || match document.previous() {
                            Some(previous) => brief.at() > *previous,
                            None => false,
                        } =>
                {
                    Some(brief)
                }
                Some(_) | None => None,
            },
            Status::Unknown => None,
        }
    }

    /// The latest landing, when the project has landed anything.
    #[must_use]
    pub fn last_landed(&self) -> Option<Wall> {
        match &self.status {
            Status::Shown(document) if document.landed() > 0 => match document.last_landed() {
                Some(at) => Some(Wall::from_nanos(*at)),
                None => None,
            },
            Status::Shown(_) | Status::Unknown => None,
        }
    }

    #[must_use]
    pub fn to_begin(&self) -> bool {
        self.delivered && self.skip == Skip::NotSkipped && self.left() > 0
    }

    /// The two steps left: step 1 counts unless its repositories are in.
    #[must_use]
    pub fn left(&self) -> u8 {
        let repository = match self.repository {
            Repository::In { .. } | Repository::CopiesIn { .. } => false,
            Repository::Unknown
            | Repository::Copying { .. }
            | Repository::CopyOffer { .. }
            | Repository::Adopting(_)
            | Repository::Offer { .. }
            | Repository::GitHub { .. }
            | Repository::Add { .. }
            | Repository::None => true,
        };
        u8::from(!self.configured).checked_add(u8::from(repository)).expect("two steps fit")
    }
}
/// Whether To begin shows for the rail: on Home and on Ask temper's page alone
/// (temper/docs/design/client/view.md, 3, *While To begin shows*).
#[must_use]
pub fn beginning(root: &Domain, project: u32, wall: Wall) -> bool {
    (root.pages().address() == &(crate::Address::Home { project })
        || root.pages().address() == &(crate::Address::Ask { project }))
        && facts(root, project, wall).to_begin()
}
#[must_use]
pub fn facts(root: &Domain, project: u32, wall: Wall) -> Facts {
    let home = if root.pages().address() == &(crate::Address::Home { project }) { root.pages().home() } else { None };
    let asking = match root.pages().chats() {
        Some(chats) => chats.chat_counts(root.client(), root.pages().me(root.client()), project),
        None => false,
    };
    let configured = asking || configured(root.client(), project);
    let repository = repository(root, project);
    let delivered = repository != Repository::Unknown
        && match root.forges() {
            forge::forges::State::Unknown => false,
            forge::forges::State::Rows(_) | forge::forges::State::Ended => true,
        }
        && (forge::ways::ended(root.forge()) || delivered(root.client(), forge::ways::handle()))
        && match root.pages().chats() {
            Some(chats) => chats.delivered(root.client(), project),
            None => false,
        };
    let mut skip = match root.forge().dismissed(PersonFact::ToBegin { project }) {
        Some(true) => Skip::Skipped,
        Some(false) => Skip::NotSkipped,
        None => Skip::Unknown,
    };
    let mut skip_refused = false;
    if let Some(home) = home {
        if home.skip.is_some() || home.skipped {
            skip = Skip::Skipped;
        }
        skip_refused = home.skip_refused;
    }
    let needs = match root.pages().inbox() {
        Some(inbox) if inbox.filter.project == Some(project) && inbox.known() => Needs::Count {
            count: u32::try_from(root.pages().needs().all(inbox, root.forge().approvals()).len())
                .expect("inbox cards fit"),
            cut: *inbox.cut.first().expect("needs cut")
                || root.forge().approvals().more() > 0
                || root.pages().needs().findings().cut_for(inbox.filter),
        },
        Some(_) | None => Needs::Unknown,
    };
    let status = match home {
        Some(home) => home.shown_status(),
        None => Status::Unknown,
    };
    let mut facts =
        Facts { configured, repository, needs, delivered, skip, skip_refused, status, decided: None, spell: None };
    if !facts.to_begin() && home.is_some() {
        facts.decided = root.pages().decided(Some(project), wall);
        if let Some(row) = root.forge().away(project) {
            let end = row.ended(wall);
            let recent = match end {
                Some(end) => wall.as_nanos().saturating_sub(end.as_nanos()) < super::decided::DAY,
                None => true,
            };
            if recent {
                let landed = match home {
                    Some(home) => match &home.document {
                        Some(status) => Some(status.landed().saturating_sub(row.landed)),
                        None => None,
                    },
                    None => None,
                };
                facts.spell = Some(Spell { since: row.since, end, landed });
            }
        }
    }
    facts
}
#[must_use]
pub fn repository(root: &Domain, project: u32) -> Repository {
    use forge::listed::Held;
    if let Some(copies) = copy_repository(root.forges(), forge::connect::playground(root.forge()), project) {
        return copies;
    }
    match forge::listed::held(root.forges(), root.forge().playground(), project) {
        Held::One { owner, name, .. } | Held::Several { owner, name, .. } => {
            return Repository::In { owner: owner.into(), name: name.into() };
        }
        Held::Unknown => {
            return match root.forges() {
                forge::forges::State::Ended => Repository::None,
                forge::forges::State::Unknown | forge::forges::State::Rows(_) => Repository::Unknown,
            };
        }
        Held::None => {}
    }
    if let Some(adopting) = root.pages().adopting()
        && adopting.project == project
        && adopting.out.is_some()
    {
        return Repository::Adopting(adopting.first.clone());
    }
    if let Some((forge, org, name)) = forge::listed::unadopted_first(root.forges()) {
        let first = welcome::First { forge, org: org.into(), name: name.into() };
        let mut refused = None;
        if let Some(adopting) = root.pages().adopting()
            && adopting.project == project
            && adopting.first == first
            && let Some(welcome::AdoptionEnded::Refused(answer)) = &adopting.ended
        {
            refused = Some(answer.clone());
        }
        return Repository::Offer { first, refused };
    }
    for row in &forge::listed::rows(root.forges(), forge::connect::playground(root.forge())) {
        if row.kind == forge::forges::Kind::GitHub
            && let forge::forges::Connection::Connected { .. } = row.connection
        {
            let mut account = None;
            for installation in &row.installations {
                if !installation.stray && !installation.suspended {
                    if let Some(accepted) = &installation.account {
                        account = Some(accepted.login.clone());
                    }
                    break;
                }
            }
            return Repository::GitHub { account };
        }
    }
    match add_forge(root.forges(), forge::connect::playground(root.forge())) {
        Some(forge) => Repository::Add { forge },
        None => Repository::None,
    }
}

pub(super) fn can_add(state: &forge::forges::State, project: u32) -> bool {
    if forge::listed::held(state, None, project) != forge::listed::Held::None
        || forge::listed::unadopted_first(state).is_some()
        || copy_repository(state, None, project).is_some()
    {
        return false;
    }
    for row in &forge::listed::rows(state, None) {
        if row.kind == forge::forges::Kind::GitHub
            && let forge::forges::Connection::Connected { .. } = row.connection
        {
            return false;
        }
    }
    add_forge(state, None).is_some()
}

pub(super) fn add_forge(state: &forge::forges::State, playground: Option<u16>) -> Option<u16> {
    use forge::forges::{Connection, Kind, Mode};
    let mut found = None;
    for row in &forge::listed::rows(state, playground) {
        if row.kind == Kind::Forgejo
            && (row.mode == Mode::Owned || row.mode == Mode::Managed)
            && let Connection::Connected { .. } = row.connection
            && match found {
                Some(number) => row.number < number,
                None => true,
            }
        {
            found = Some(row.number);
        }
    }
    found
}

/// The making rows take precedence over held repositories (client/pages/README.md, Home).
fn copy_repository(state: &forge::forges::State, playground: Option<u16>, project: u32) -> Option<Repository> {
    use forge::making;
    let copies = making::copies(state, playground);
    let rows = forge::listed::rows(state, playground);
    let mut lines = List::uncapped();
    let mut names = List::uncapped();
    let mut unasked = List::uncapped();
    let mut active = false;
    if let Some(copies) = &copies {
        for row in &rows {
            if row.number != copies.forge {
                continue;
            }
            for address in &copies.addresses {
                let Some((org, name)) = making::target(address) else { continue };
                let mut found = false;
                for making in &row.makings {
                    if making::is_target(making, address) {
                        found = true;
                        if let Some(into) = &making.into
                            && into.project() == project
                        {
                            active = true;
                            lines.push(copy_line(making)).expect("admitted making rows");
                        }
                        break;
                    }
                }
                if found {
                    continue;
                }
                for repository in &row.repositories {
                    if repository.owner == org && repository.name == name {
                        found = true;
                        if repository.project == project {
                            names.push(bytes::concat(&[&org, b"/", &name])).expect("admitted copies");
                            lines
                                .push(CopyLine { org: org.clone(), name: name.clone(), state: CopyState::In })
                                .expect("admitted copies");
                        }
                        break;
                    }
                }
                if !found {
                    unasked.push(address.clone()).expect("admitted copy addresses");
                    lines.push(CopyLine { org, name, state: CopyState::Waiting }).expect("admitted copies");
                }
            }
        }
    }
    let mut held_are_copies = true;
    for row in &rows {
        for making in &row.makings {
            let Some(into) = &making.into else { continue };
            if into.project() != project {
                continue;
            }
            let mut intended = false;
            if let Some(copies) = &copies
                && copies.forge == row.number
            {
                for address in &copies.addresses {
                    intended |= making::is_target(making, address);
                }
            }
            if !intended {
                active = true;
                lines.push(copy_line(making)).expect("admitted making rows");
            }
        }
        for repository in &row.repositories {
            if repository.project != project {
                continue;
            }
            let mut intended = false;
            if let Some(copies) = &copies
                && copies.forge == row.number
            {
                for address in &copies.addresses {
                    if let Some((org, name)) = making::target(address) {
                        intended |= repository.owner == org && repository.name == name;
                    }
                }
            }
            held_are_copies &= intended;
        }
    }
    if active {
        return Some(Repository::Copying { lines: lines.into_boxed() });
    }
    if !unasked.is_empty()
        && let Some(copies) = copies
    {
        return Some(Repository::CopyOffer { forge: copies.forge, addresses: unasked.into_boxed() });
    }
    if held_are_copies && !names.is_empty() {
        return Some(Repository::CopiesIn { names: names.into_boxed() });
    }
    None
}
fn copy_line(making: &forge::making::Making) -> CopyLine {
    let state = match &making.state {
        w::MakingState::Waiting => match making.from {
            w::MakeFrom::Empty => CopyState::Making,
            w::MakeFrom::Copy(_) => CopyState::Waiting,
        },
        w::MakingState::Making => {
            let host = match &making.from {
                w::MakeFrom::Copy(copy) => match source::parse(copy.address()) {
                    Ok(source) => source.host.into(),
                    Err(_) => b"?".as_slice().into(),
                },
                w::MakeFrom::Empty => {
                    return CopyLine { org: making.org.clone(), name: making.name.clone(), state: CopyState::Making };
                }
            };
            CopyState::Running { host }
        }
        w::MakingState::Made(_) | w::MakingState::Adopting(_) => CopyState::Adopting,
        w::MakingState::Failed(_) => CopyState::Failed { words: forge::making::phrase(making) },
    };
    CopyLine { org: making.org.clone(), name: making.name.clone(), state }
}

pub(super) fn delivered(client: &Client, handle: Handle) -> bool {
    match client.document(handle) {
        Some(document) => document.state == jig_client::State::Ended || document.latest.is_some(),
        None => false,
    }
}

/// Send this entry's look once the facts and dismissals settle and To begin is gone.
pub(crate) fn look(root: &mut Domain, env: &Env<crate::Limits>, out: &mut Queue<crate::Request>) {
    super::all_home::look(root, env, out);
    let project = match root.pages.address() {
        crate::Address::Home { project } => *project,
        crate::Address::NewChat { .. }
        | crate::Address::AllHome
        | crate::Address::Inbox { .. }
        | crate::Address::Chats { .. }
        | crate::Address::Remembered
        | crate::Address::Members
        | crate::Address::Limits
        | crate::Address::Accounts
        | crate::Address::Forges
        | crate::Address::ConnectForge { .. }
        | crate::Address::ConnectModel { .. }
        | crate::Address::Welcome { .. }
        | crate::Address::Chat { .. }
        | crate::Address::Task { .. }
        | crate::Address::Changes { .. }
        | crate::Address::Team { .. }
        | crate::Address::Settings { .. }
        | crate::Address::Repositories { .. }
        | crate::Address::Models { .. }
        | crate::Address::Builders { .. }
        | crate::Address::Work { .. }
        | crate::Address::Role { .. }
        | crate::Address::Rule { .. }
        | crate::Address::Goals { .. }
        | crate::Address::Ask { .. }
        | crate::Address::Adopt { .. } => return,
    };
    let facts = facts(root, project, env.wall);
    let Some(home) = root.pages.home.as_ref() else { return };
    match home.look {
        Look::NotSent => {}
        Look::Out(_) | Look::Answered => return,
    }
    if !facts.delivered || !root.forge.settled() || facts.to_begin() {
        return;
    }
    let Some(request) = forge::status::ask(project, &env.limits.forge) else { return };
    root.pages.serial = root.pages.serial.checked_add(1).expect("page keys fit");
    let key = Key { asker: Owner::Pages.asker(), number: root.pages.serial };
    root.pages.home.as_mut().expect("Home is open").look = Look::Out(key);
    crate::route::client(root, env, jig_client::Event::Send { key, name: None, request }, out);
}
