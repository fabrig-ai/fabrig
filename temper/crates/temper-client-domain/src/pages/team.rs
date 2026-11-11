//! Team: a project's people and what waits on each
//! (temper/docs/design/client/pages/team.md, 1).
use super::Request;
use crate::{Address, Limits, owner::Owner};
use alloc::boxed::Box;
use jig_client::{Client, Handle, Latest, Readable, Readout, Told, Watchable, contract as c};
use skein_lib::List;
use skein_lib::Queue;

mod rules;
mod standing;
pub mod wakes;
pub use rules::{Rules, asked};
pub use standing::{Card, Now, Role, StartedBy};
pub(crate) mod sessions;
pub use sessions::Session;
/// The standing roles watch’s window.
pub const ROLES: u32 = 16;

/// Other people whose waits are read, one `tasks{query}` watch each.
pub const OTHERS: u32 = 8;

/// What waits on one person in the project, by kind; each count a floor while `cut`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Waits {
    pub questions: u32,
    pub reviews: u32,
    pub proposals: u32,
    /// Escalations: a held task that reached them.
    pub held: u32,
    pub person_tasks: u32,
    pub cut: bool,
}
#[expect(clippy::derivable_impls, reason = "Team’s contract names each zero count and its uncut default explicitly")]
impl Default for Waits {
    fn default() -> Self {
        Self { questions: 0, reviews: 0, proposals: 0, held: 0, person_tasks: 0, cut: false }
    }
}
impl Waits {
    /// The five counts summed, saturating.
    #[must_use]
    pub const fn total(&self) -> u32 {
        self.questions
            .saturating_add(self.reviews)
            .saturating_add(self.proposals)
            .saturating_add(self.held)
            .saturating_add(self.person_tasks)
    }
}

/// One person of the project as Team draws them.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Person {
    pub party: u64,
    /// The person reading.
    pub you: bool,
    /// The member's name, or its login where the name is empty.
    pub name: Box<[u8]>,
    pub role: c::HeldRole,
    /// What waits on them; `None` while unread or past `OTHERS`: no line is drawn.
    pub waits: Option<Waits>,
}

/// Team's facts: plain values its reads give.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Facts {
    /// The project's members of kind person, the reader first, then in the document's order, each once.
    pub people: List<Person>,
    /// The project's document was cut: its members may be more.
    pub cut: bool,
    /// The project's triager, as its line names it; no read gives it yet.
    pub triager: Option<Box<[u8]>>,
    pub roles: List<Role>,
    pub roles_cut: bool,
    /// The address’s requested role; falls back to the first listed.
    pub selected: Option<u64>,
    pub card: Option<Box<Card>>,
}

/// What Team draws (temper/docs/design/client/pages/team.md, 1, Its parts).
/// The head names a shared built-in role only for two or more people in a whole
/// document. A person's line is drawn while their `waits` is `Some`.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[expect(clippy::struct_excessive_bools, reason = "one flag per independently drawn Team part")]
pub struct Parts {
    /// People: a person is listed.
    pub people: bool,
    /// People's head's shared role; never `Policy`.
    pub shared: Option<c::HeldRole>,
    /// The triager's line: a triager is read.
    pub triager: bool,
    pub standing_roles: bool,
    pub roles_cut: bool,
    pub card: Option<u64>,
    pub pause: bool,
    pub resume: bool,
    pub now: bool,
    pub sessions: bool,
    pub rules: bool,
    pub wakes: bool,
    pub wakes_cut: bool,
    pub right: bool,
}

#[expect(clippy::derivable_impls, reason = "parts name each undrawn default explicitly in the step subset")]
impl Default for Parts {
    fn default() -> Self {
        Self {
            people: false,
            shared: None,
            triager: false,
            standing_roles: false,
            roles_cut: false,
            card: None,
            pause: false,
            resume: false,
            now: false,
            sessions: false,
            rules: false,
            wakes: false,
            wakes_cut: false,
            right: false,
        }
    }
}

/// What Team's page shows of the project.
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Shown {
    /// No answer yet: the intro, and "People" with no line under it.
    Reading,
    /// The project's document answered.
    Read(Facts),
    /// A refused or failed read: "This project's people can't be read now."
    Unreadable,
    /// Roles are read independently of the project's people.
    PeopleUnread { facts: Box<Facts>, failed: bool },
}
impl Shown {
    #[must_use]
    pub fn facts(&self) -> Option<&Facts> {
        match self {
            Self::Read(facts) => Some(facts),
            Self::PeopleUnread { facts, .. } => Some(facts),
            Self::Reading | Self::Unreadable => None,
        }
    }
}

#[must_use]
pub fn parts(facts: &Facts) -> Parts {
    let card = selected(&facts.roles, facts.selected);
    let detail = match &facts.card {
        Some(detail) if Some(detail.task) == card => Some(detail),
        Some(_) | None => None,
    };
    let mut parts = Parts {
        people: !facts.people.is_empty(),
        shared: shared(facts),
        triager: facts.triager.is_some(),
        standing_roles: !facts.roles.is_empty(),
        roles_cut: !facts.roles.is_empty() && facts.roles_cut,
        card,
        pause: false,
        resume: false,
        now: false,
        sessions: false,
        rules: false,
        wakes: false,
        wakes_cut: false,
        right: false,
    };
    if let Some(detail) = detail {
        if let Some(rules) = &detail.rules {
            parts.rules = !rules.asked.is_empty();
        }
        parts.wakes = !detail.wakes.is_empty();
        parts.wakes_cut = parts.wakes && detail.wakes_cut;
        parts.sessions = !detail.sessions.is_empty();
        parts.right = parts.wakes || parts.sessions;
        parts.pause = detail.hold.is_none() && detail.pause;
        parts.resume = detail.hold.is_some() && detail.paused && detail.resume;
        if let Some(now) = &detail.now {
            parts.now = !now.body.is_empty();
        }
    }
    parts
}

fn shared(facts: &Facts) -> Option<c::HeldRole> {
    if facts.cut || facts.people.len() < 2 {
        return None;
    }
    let mut shared = None;
    for person in &facts.people {
        match person.role {
            c::HeldRole::Owner | c::HeldRole::Maintainer | c::HeldRole::Member | c::HeldRole::Observer => {}
            c::HeldRole::Policy { role: _ } => return None,
        }
        if let Some(role) = shared
            && role != person.role
        {
            return None;
        }
        shared = Some(person.role);
    }
    shared
}

/// Team's facts from the project's document, the reader's party, what waits on
/// the reader, and what waits on others by party; pure.
#[must_use]
pub fn gather(project: &c::Project, you: u64, yours: Option<Waits>, others: &[(u64, Waits)]) -> Facts {
    let mut people: List<Person> = List::with_capacity(project.members.len());
    for reader in [true, false] {
        for member in &project.members {
            match member.kind {
                c::PartyKind::Person => {}
                c::PartyKind::Service => continue,
            }
            if (member.party == you) != reader {
                continue;
            }
            let mut listed = false;
            for person in &people {
                if person.party == member.party {
                    listed = true;
                    break;
                }
            }
            if listed {
                continue;
            }
            let mut waits = None;
            if reader {
                waits = yours;
            } else {
                for (party, counts) in others {
                    if *party == member.party {
                        waits = Some(*counts);
                        break;
                    }
                }
            }
            let name = if member.name.bytes().is_empty() { member.login.bytes() } else { member.name.bytes() };
            people
                .push(Person { party: member.party, you: reader, name: name.into(), role: member.role, waits })
                .expect("at most one person per document member");
        }
    }
    Facts {
        people,
        cut: project.cut,
        triager: None,
        roles: List::with_capacity(0),
        roles_cut: false,
        selected: None,
        card: None,
    }
}

/// Team's open project and its bounded reads (client/pages/README.md, Team, Its reads).
#[derive(Debug)]
pub struct Team {
    project: Option<u32>,
    handle: Option<Handle>,
    document: Option<Box<c::Project>>,
    reading: Reading,
    others: List<Other>,
    standing: standing::Standing,
    sessions: sessions::Reader,
}
#[derive(Debug, PartialEq, Eq)]
enum Reading {
    Idle,
    Asked,
    Refresh,
    Failed,
}
#[derive(Debug)]
struct Other {
    party: u64,
    handle: Handle,
    waits: Option<Waits>,
}
impl Default for Team {
    fn default() -> Self {
        Self {
            project: None,
            handle: None,
            document: None,
            reading: Reading::Idle,
            others: List::with_capacity(OTHERS),
            standing: standing::Standing::default(),
            sessions: sessions::Reader::default(),
        }
    }
}
impl Team {
    #[expect(
        clippy::match_like_matches_macro,
        reason = "the step crate excludes matches!; its address check uses if let"
    )]
    pub(crate) fn configure(
        &mut self,
        address: &Address,
        you: u64,
        env: &skein_lib::Env<Limits>,
        client: &Client,
        next: &mut u64,
        out: &mut Queue<Request>,
    ) {
        let Some(project) = super::address_project(address) else {
            self.leave(out);
            return;
        };
        if self.project != Some(project) {
            self.leave(out);
            self.project = Some(project);
            self.reading = Reading::Refresh;
        }
        if self.reading == Reading::Refresh {
            if let Some(handle) = self.handle.take() {
                out.push(Request::Client(jig_client::Event::Unwatch { handle }));
            }
            let handle = new_handle(next);
            self.handle = Some(handle);
            self.reading = Reading::Asked;
            out.push(Request::Client(jig_client::Event::Read {
                handle,
                read: Readable::Jig(c::Read::Project { project }),
            }));
        }
        let on_team = if let Address::Team { .. } = address { true } else { false };
        self.configure_others(you, on_team, &env.limits, next, out);
        self.standing.configure(project, address, next, out);
        let wanted = if on_team { self.standing.activity(client) } else { None };
        self.sessions.configure(wanted, env.wall, next, out);
        self.standing.configure_rules(client, env.limits.forge, next, out);
    }
    fn configure_others(&mut self, you: u64, on_team: bool, limits: &Limits, next: &mut u64, out: &mut Queue<Request>) {
        let mut wanted = List::with_capacity(OTHERS);
        if on_team && let Some(document) = &self.document {
            for person in &gather(document, you, None, &[]).people {
                if !person.you && wanted.len() < OTHERS {
                    wanted.push(person.party).expect("at most OTHERS people");
                }
            }
        }
        let old = core::mem::replace(&mut self.others, List::with_capacity(OTHERS));
        for other in old.into_boxed() {
            if wanted.as_slice().contains(&other.party) {
                self.others.push(other).expect("at most OTHERS watches");
            } else {
                out.push(Request::Client(jig_client::Event::Unwatch { handle: other.handle }));
            }
        }
        let Some(project) = self.project else { return };
        for party in &wanted {
            let mut held = false;
            for other in &self.others {
                if other.party == *party {
                    held = true;
                    break;
                }
            }
            if held {
                continue;
            }
            let handle = new_handle(next);
            self.others.push(Other { party: *party, handle, waits: None }).expect("at most OTHERS watches");
            let query = c::TaskQuery {
                project: Some(project),
                requester: Some(c::Requester::Person(*party)),
                root: None,
                parent: None,
                label: None,
                phase: None,
                executor: None,
                tracked: None,
                order: c::QueryOrder::Activity,
            };
            out.push(Request::Client(jig_client::Event::Watch {
                handle,
                subject: Watchable::Jig(c::Subject::Tasks { query, window: limits.client.domain.rows }),
            }));
        }
    }
    pub(crate) fn leave(&mut self, out: &mut Queue<Request>) {
        if let Some(handle) = self.handle {
            out.push(Request::Client(jig_client::Event::Unwatch { handle }));
        }
        for other in &self.others {
            out.push(Request::Client(jig_client::Event::Unwatch { handle: other.handle }));
        }
        self.sessions.close(out);
        self.standing.leave(out);
        *self = Self::default();
    }
    pub(super) fn told(
        &mut self,
        env: &skein_lib::Env<Limits>,
        client: &Client,
        told: &Told,
        out: &mut Queue<Request>,
    ) -> bool {
        if self.sessions.told(env.now, env.wall, told, out) {
            return true;
        }
        if self.standing.rules_told(told, env.limits.forge) {
            return true;
        }
        if self.standing.told(client, told, out) {
            return true;
        }
        match told {
            Told::Changed { handle } if *handle == super::me_handle() && self.project.is_some() => {
                if self.reading != Reading::Asked {
                    self.reading = Reading::Refresh;
                }
                true
            }
            Told::Page { handle, page } if self.handle == Some(*handle) && self.reading == Reading::Asked => {
                self.document = match page {
                    Readout::Jig(c::ReadResult::Value(c::ReadValue::Project(document)))
                        if self.project == Some(document.number) =>
                    {
                        Some(document.clone())
                    }
                    Readout::Jig(_) | Readout::Family { .. } | Readout::Limit | Readout::NotServed { .. } => None,
                };
                self.reading = if self.document.is_some() { Reading::Idle } else { Reading::Failed };
                true
            }
            Told::Ended { handle, .. } if self.handle == Some(*handle) => {
                self.document = None;
                self.reading = Reading::Failed;
                true
            }
            Told::Changed { handle } => {
                for other in self.others.iter_mut() {
                    if other.handle == *handle {
                        other.waits = other_waits(client, *handle, other.party);
                        return true;
                    }
                }
                false
            }
            Told::Ended { handle, .. } => {
                for other in self.others.iter_mut() {
                    if other.handle == *handle {
                        other.waits = None;
                        return true;
                    }
                }
                false
            }
            Told::Page { .. }
            | Told::Pending { .. }
            | Told::Answered { .. }
            | Told::Kept { .. }
            | Told::Dropped { .. }
            | Told::Link { .. }
            | Told::Text { .. } => false,
        }
    }
}
fn new_handle(next: &mut u64) -> Handle {
    let handle = Handle { asker: Owner::Pages.asker(), number: *next };
    *next = next.checked_add(1).expect("page handles fit");
    handle
}
fn other_waits(client: &Client, handle: Handle, party: u64) -> Option<Waits> {
    let document = client.document(handle)?;
    if document.state == jig_client::State::Ended {
        return None;
    }
    let Latest::Jig(c::Delivery::Tasks(tasks)) = document.latest? else { return None };
    let mut waits = Waits { cut: tasks.more, ..Waits::default() };
    for line in &tasks.rows {
        if let c::WaitsOn::Party { whom: c::Whom::Person(whom), kind, .. } = line.waits_on
            && whom == party
            && line.ended_at.is_none()
        {
            match kind {
                c::WaitedFor::Question => waits.questions = waits.questions.saturating_add(1),
                c::WaitedFor::Proposal => waits.proposals = waits.proposals.saturating_add(1),
                c::WaitedFor::Escalation => waits.held = waits.held.saturating_add(1),
                c::WaitedFor::PersonTask => waits.person_tasks = waits.person_tasks.saturating_add(1),
            }
        }
    }
    Some(waits)
}
/// Team's page for the project, from its document and delivered waits.
#[must_use]
pub fn shown(root: &crate::Domain, project: u32) -> Shown {
    shown_at(root, project, root.pages.team.sessions.wall())
}
/// Project the card's sessions at the view's wall, including week expiry between deliveries.
#[must_use]
pub fn shown_at(root: &crate::Domain, project: u32, wall: skein_lib::Wall) -> Shown {
    let state = &root.pages.team;
    if state.project != Some(project) {
        return Shown::Reading;
    }
    let you = match root.pages.me(root.client()) {
        Some(me) => me.party.number,
        None => 0,
    };
    let mut others = List::with_capacity(OTHERS);
    for other in &state.others {
        if let Some(waits) = other.waits {
            others.push((other.party, waits)).expect("at most OTHERS waits");
        }
    }
    let mut facts = if let Some(document) = &state.document {
        gather(document, you, yours(root, project), others.as_slice())
    } else {
        Facts {
            people: List::with_capacity(0),
            cut: false,
            triager: None,
            roles: List::with_capacity(0),
            roles_cut: false,
            selected: None,
            card: None,
        }
    };
    state.standing.gather(root.client(), state.document.as_deref(), you, &mut facts);
    if let Some(card) = &mut facts.card {
        card.sessions = state.sessions.rows(wall);
    }
    if state.document.is_some() {
        Shown::Read(facts)
    } else if facts.roles.is_empty() {
        if state.reading == Reading::Failed { Shown::Unreadable } else { Shown::Reading }
    } else {
        Shown::PeopleUnread { facts: Box::new(facts), failed: state.reading == Reading::Failed }
    }
}
/// The project's people counted for the rail while its document is read.
#[must_use]
pub fn people(root: &crate::Domain, project: u32) -> Option<u32> {
    let state = &root.pages.team;
    if state.project != Some(project) {
        return None;
    }
    let you = match root.pages.me(root.client()) {
        Some(me) => me.party.number,
        None => 0,
    };
    Some(gather(state.document.as_deref()?, you, None, &[]).people.len())
}
fn yours(root: &crate::Domain, project: u32) -> Option<Waits> {
    let client = root.client();
    let document = client.document(super::needs::handle())?;
    if document.state == jig_client::State::Ended {
        return None;
    }
    let Latest::Jig(c::Delivery::Inbox(inbox)) = document.latest? else { return None };
    let session = root.pages.needs();
    let approvals = root.forge().approvals();
    let mut waits = Waits { cut: session.count(client, approvals, project).cut, ..Waits::default() };
    for row in &inbox.needs {
        if row.shows_in(project) && !session.answered_here(row.card_key()) {
            match row.group() {
                c::InboxGroup::Questions => waits.questions = waits.questions.saturating_add(1),
                c::InboxGroup::Proposals => waits.proposals = waits.proposals.saturating_add(1),
                c::InboxGroup::Escalations => waits.held = waits.held.saturating_add(1),
                c::InboxGroup::PersonTasks => waits.person_tasks = waits.person_tasks.saturating_add(1),
                c::InboxGroup::Replies | c::InboxGroup::Results => {}
            }
        }
    }
    for row in approvals.rows() {
        if row.project == project {
            waits.reviews = waits.reviews.saturating_add(1);
        }
    }
    Some(waits)
}

/// One selected listed role, otherwise the first listed.
fn selected(roles: &List<Role>, requested: Option<u64>) -> Option<u64> {
    for role in roles {
        if Some(role.task) == requested {
            return requested;
        }
    }
    if let Some(role) = roles.get(0) { Some(role.task) } else { None }
}
/// The live standing roles counted for the rail.
#[must_use]
pub fn standing_roles(root: &crate::Domain, project: u32) -> Option<u32> {
    if root.pages.team.project != Some(project) {
        return None;
    }
    root.pages.team.standing.count()
}
impl Team {
    pub(super) fn send(
        &mut self,
        client: &Client,
        kind: super::task::Sending,
        serial: &mut u64,
        out: &mut Queue<Request>,
    ) {
        self.standing.send(client, kind, serial, out);
    }
    pub(super) fn toasted(&mut self, told: &Told) -> Option<super::needs::Toasted> {
        self.standing.toasted(told)
    }
}

impl Team {
    pub(super) fn fire(&mut self, now: skein_lib::Time, out: &mut Queue<Request>) {
        self.sessions.fire(now, out);
    }
    pub(super) const fn next_deadline(&self) -> Option<skein_lib::Time> {
        self.sessions.next_deadline()
    }
}
