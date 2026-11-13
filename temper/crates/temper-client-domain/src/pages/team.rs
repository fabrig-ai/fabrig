//! Team: a project's people and what waits on each
//! (temper/docs/design/client/pages/team.md, 1).
use super::Request;
use crate::{Address, Limits, owner::Owner};
use alloc::boxed::Box;
use jig_client::{Client, Handle, Latest, Readable, Readout, Told, Watchable, contract as c};
use skein_lib::Queue;
use skein_lib::{Decimal, Duration, List, Set, Wall};
use temper_client_protocol_wire::v1 as w;

mod areas;
pub mod follows;
pub mod hand;
pub use areas::{Area, AreaName, AreaPart, Areas};
mod rules;
mod standing;
pub mod wakes;
pub use rules::{Rules, asked};
pub(crate) use standing::now;
pub use standing::{Card, Now, Role, StartedBy};
pub(crate) mod sessions;
pub use sessions::Session;
/// A project's areas from its findings document (client/pages/talk.md, 3).
#[derive(Debug, PartialEq, Eq)]
pub struct HeldAreas {
    pub rows: Box<[HeldArea]>,
    pub triager: Option<Holder>,
}
#[derive(Debug, PartialEq, Eq)]
pub struct HeldArea {
    pub name: Box<[u8]>,
    pub holder: Option<Holder>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Holder {
    Task(u64),
    Party(u64),
    Role(u32),
}
#[must_use]
pub fn areas(root: &crate::Domain, project: u32) -> Option<HeldAreas> {
    let document = root.pages.needs.findings.document(project)?;
    let mut rows = List::with_capacity(document.areas().len());
    for area in document.areas() {
        rows.push(HeldArea {
            name: area.name().into(),
            holder: match area.holder() {
                Some(value) => Some(area_holder(value)),
                None => None,
            },
        })
        .expect("document's areas fit");
    }
    Some(HeldAreas {
        rows: rows.into_boxed(),
        triager: if document.triager().is_some() { Some(area_holder(document.holder())) } else { None },
    })
}
pub(super) fn area_holder(value: &w::Holder) -> Holder {
    match value {
        w::Holder::Task(task) => Holder::Task(task.number()),
        w::Holder::Party(party) => Holder::Party(party.number()),
        w::Holder::Role(role) => Holder::Role(role.number()),
    }
}

/// The standing roles watch’s window.
pub const ROLES: u32 = 16;

/// The project's open period lasts seven days (client/pages/talk.md, 2).
#[must_use]
pub fn weekly(root: &crate::Domain, project: u32) -> bool {
    let Some(document) = root.pages().team.document(project) else { return false };
    let Some(period) = &document.current_period else { return false };
    if period.closed {
        return false;
    }
    match (period.began, period.next) {
        (Some(began), Some(next)) => next.as_nanos().checked_sub(began.as_nanos()) == Some(604_800_000_000_000),
        (Some(_), None) | (None, Some(_) | None) => false,
    }
}

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
    /// Sign-offs in the week; `None` while the changes watch is unread or unavailable.
    pub signed: Option<Signed>,
}

/// A person's distinct changes signed in the inclusive seven-day window.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Signed {
    pub count: u32,
    pub floor: bool,
}

/// Team's facts: plain values its reads give.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Facts {
    /// The reader holds a role in this project, from me.
    pub member: bool,
    /// The project's members of kind person, the reader first, then in the document's order, each once.
    pub people: List<Person>,
    /// The project's document was cut: its members may be more.
    pub cut: bool,
    /// The project's areas and routing holder, once the findings document answers.
    pub areas: Option<Areas>,
    pub follows: List<follows::Follow>,
    pub roles: List<Role>,
    pub roles_cut: bool,
    /// The address’s requested role; falls back to the first listed.
    pub selected: Option<u64>,
    pub card: Option<Box<Card>>,
}

/// What Team draws (temper/docs/design/client/pages/team.md, 1, Its parts).
/// The head names a shared built-in role only for two or more people in a whole
/// document. A person's line is drawn while their `waits` is `Some`.
#[derive(Clone, Debug, PartialEq, Eq)]
#[expect(clippy::struct_excessive_bools, reason = "one flag per independently drawn Team part")]
pub struct Parts {
    /// People: a person is listed.
    pub people: bool,
    /// People's head's shared role; never `Policy`.
    pub shared: Option<c::HeldRole>,
    pub areas: bool,
    pub hand_area: bool,
    /// Named areas held by the selected role, excluding Anything else.
    pub give_areas: u32,
    pub areas_table: bool,
    pub areas_empty: bool,
    pub who_routes: bool,
    pub lede_areas: bool,
    pub area_rows: List<AreaPart>,
    /// Listed roles that hold no table row, in watch order.
    pub standing: List<u64>,
    /// The person's table rows, by party, never an empty list.
    pub box_areas: List<(u64, List<AreaName>)>,
    /// The selected card's table rows, in table order.
    pub owns: List<AreaName>,
    pub talk: bool,
    /// Positive sign-off counts, in words, by party.
    pub signed: List<(u64, Box<[u8]>)>,
    pub follows: List<follows::Follow>,
    pub card_follows: List<follows::Follow>,
    pub standing_roles: bool,
    pub roles_cut: bool,
    pub card: Option<u64>,
    pub pause: bool,
    pub resume: bool,
    pub now: bool,
    pub sessions: bool,
    pub report: bool,
    pub rules: bool,
    pub wakes: bool,
    pub wakes_cut: bool,
    pub right: bool,
}

impl Default for Parts {
    fn default() -> Self {
        Self {
            signed: List::with_capacity(0),
            follows: List::with_capacity(0),
            card_follows: List::with_capacity(0),
            people: false,
            shared: None,
            areas: false,
            hand_area: false,
            give_areas: 0,
            areas_table: false,
            areas_empty: false,
            who_routes: false,
            lede_areas: false,
            area_rows: List::with_capacity(0),
            standing: List::with_capacity(0),
            box_areas: List::with_capacity(0),
            owns: List::with_capacity(0),
            talk: false,
            standing_roles: false,
            roles_cut: false,
            card: None,
            pause: false,
            resume: false,
            now: false,
            sessions: false,
            report: false,
            rules: false,
            wakes: false,
            wakes_cut: false,
            right: false,
        }
    }
}

impl Parts {
    /// A person's positive sign-off count, in words.
    #[must_use]
    pub fn signed(&self, party: u64) -> Option<&[u8]> {
        for (whom, words) in &self.signed {
            if *whom == party {
                return Some(words);
            }
        }
        None
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
    let mut signed = List::with_capacity(facts.people.len());
    for person in &facts.people {
        if let Some(count) = person.signed
            && count.count > 0
        {
            let number = if count.floor {
                super::signing::join(&[Decimal::of(u64::from(count.count)).as_bytes(), b"+"])
            } else {
                super::signing::count(count.count)
            };
            signed
                .push((person.party, super::signing::join(&[b"Signed ", &number, b" last week."])))
                .expect("one sentence per person");
        }
    }
    let card = selected(&facts.roles, facts.selected);
    let detail = match &facts.card {
        Some(detail) if Some(detail.task) == card => Some(detail),
        Some(_) | None => None,
    };
    let (follows, card_follows) = follows::parts(facts, card);
    let mut parts = Parts {
        signed,
        follows,
        card_follows,
        people: !facts.people.is_empty(),
        shared: shared(facts),
        roles_cut: facts.roles_cut,
        card,
        pause: false,
        resume: false,
        now: false,
        sessions: false,
        report: false,
        rules: false,
        wakes: false,
        wakes_cut: false,
        right: false,
        ..Parts::default()
    };
    areas::parts(facts, &mut parts);
    parts.hand_area = parts.areas && facts.member;
    if facts.member
        && let Some(task) = parts.card
        && let Some(areas) = &facts.areas
    {
        for area in &areas.rows {
            if let Some(w::Holder::Task(holder)) = &area.holder
                && holder.number() == task
            {
                parts.give_areas = parts.give_areas.saturating_add(1);
            }
        }
    }
    if let Some(detail) = detail {
        if let Some(rules) = &detail.rules {
            parts.rules = !rules.asked.is_empty();
        }
        parts.wakes = !detail.wakes.is_empty();
        parts.wakes_cut = parts.wakes && detail.wakes_cut;
        parts.sessions = !detail.sessions.is_empty();
        if let Some(report) = &detail.report {
            parts.report = !report.body.is_empty();
        }
        parts.right = parts.wakes || parts.sessions || parts.report;
        parts.talk = detail.talk;
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
/// the reader, what waits on others by party, and the landed changes at the wall; pure.
#[must_use]
pub fn gather(
    project: &c::Project,
    you: u64,
    yours: Option<Waits>,
    others: &[(u64, Waits)],
    changes: Option<&w::ChangesPanel>,
    wall: Wall,
) -> Facts {
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
                .push(Person {
                    party: member.party,
                    you: reader,
                    name: name.into(),
                    role: member.role,
                    waits,
                    signed: signed_count(changes, member.party, wall),
                })
                .expect("at most one person per document member");
        }
    }
    let mut member = false;
    for row in &project.members {
        member |= row.party == you;
    }
    Facts {
        member,
        people,
        cut: project.cut,
        areas: None,
        follows: List::with_capacity(0),
        roles: List::with_capacity(0),
        roles_cut: false,
        selected: None,
        card: None,
    }
}

fn signed_count(changes: Option<&w::ChangesPanel>, party: u64, wall: Wall) -> Option<Signed> {
    let changes = changes?;
    let start = Wall::from_nanos(wall.as_nanos().saturating_sub(Duration::from_secs(604_800).as_nanos()));
    let mut tasks = Set::with_capacity(changes.landed_default().len());
    let mut oldest = None;
    for row in changes.landed_default() {
        let at = Wall::from_nanos(row.at());
        oldest = Some(match oldest {
            Some(held) => at.min(held),
            None => at,
        });
        if at < start || at > wall {
            continue;
        }
        for signer in row.signed() {
            if signer.party() == party {
                tasks.insert(row.task()).expect("one key per listed change");
                break;
            }
        }
    }
    let floor = changes.landed_default_more() > 0
        && match oldest {
            Some(at) => at >= start && at <= wall,
            None => false,
        };
    Some(Signed { count: tasks.len(), floor })
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
    changes_handle: Option<Handle>,
    changes: Option<Box<w::ChangesPanel>>,
    areas_handle: Option<Handle>,
    areas: Option<Areas>,
    follows_handle: Option<Handle>,
    follows: Option<Box<w::FollowsPanel>>,
}
#[derive(Debug, PartialEq, Eq)]
enum Reading {
    Idle,
    Asked,
    Refresh,
    Failed,
    Gone,
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
            changes_handle: None,
            changes: None,
            areas_handle: None,
            areas: None,
            follows_handle: None,
            follows: None,
        }
    }
}
impl Team {
    pub(crate) fn role_title(&self, task: u64) -> Option<Box<[u8]>> {
        self.standing.title(task)
    }
    pub(crate) fn document(&self, project: u32) -> Option<&c::Project> {
        if self.project == Some(project) { self.document.as_deref() } else { None }
    }
    #[expect(
        clippy::match_like_matches_macro,
        reason = "the step crate excludes matches!; its address check uses if let"
    )]
    #[expect(clippy::too_many_arguments, reason = "launchpad enables Home's Now reads, not Team's other reads")]
    pub(crate) fn configure(
        &mut self,
        address: &Address,
        launchpad: bool,
        you: u64,
        env: &skein_lib::Env<Limits>,
        client: &Client,
        next: &mut u64,
        out: &mut Queue<Request>,
    ) {
        let project =
            if let Address::Inbox { filter } = address { filter.project } else { super::address_project(address) };
        let Some(project) = project else {
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
        if on_team && self.changes_handle.is_none() {
            if let Some(subject) = crate::forge::changes::watch(project, &env.limits.forge) {
                let handle = new_handle(next);
                self.changes_handle = Some(handle);
                out.push(Request::Client(jig_client::Event::Watch { handle, subject }));
            }
        } else if !on_team {
            if let Some(handle) = self.changes_handle.take() {
                out.push(Request::Client(jig_client::Event::Unwatch { handle }));
            }
            self.changes = None;
        }
        if on_team && self.areas_handle.is_none() {
            if let Some(subject) =
                crate::forge::findings::watching(project, Some(w::FindingState::New), false, &env.limits.forge)
            {
                let handle = new_handle(next);
                self.areas_handle = Some(handle);
                out.push(Request::Client(jig_client::Event::Watch { handle, subject }));
            }
        } else if !on_team {
            if let Some(handle) = self.areas_handle.take() {
                out.push(Request::Client(jig_client::Event::Unwatch { handle }));
            }
            self.areas = None;
        }
        if on_team && self.follows_handle.is_none() {
            if let Some(subject) = crate::forge::follows::watch(project, &env.limits.forge) {
                let handle = new_handle(next);
                self.follows_handle = Some(handle);
                out.push(Request::Client(jig_client::Event::Watch { handle, subject }));
            }
        } else if !on_team {
            if let Some(handle) = self.follows_handle.take() {
                out.push(Request::Client(jig_client::Event::Unwatch { handle }));
            }
            self.follows = None;
        }
        self.configure_others(you, on_team, &env.limits, next, out);
        self.standing.configure(project, address, launchpad, next, out);
        let wanted = if on_team { self.standing.activity(client) } else { None };
        self.sessions.configure(wanted, env.wall, next, out);
        self.standing.configure_rules(client, env.limits.forge, next, out);
    }
    fn configure_others(&mut self, you: u64, on_team: bool, limits: &Limits, next: &mut u64, out: &mut Queue<Request>) {
        let mut wanted = List::with_capacity(OTHERS);
        if on_team && let Some(document) = &self.document {
            for person in &gather(document, you, None, &[], None, Wall::EPOCH).people {
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
        if let Some(handle) = self.changes_handle {
            out.push(Request::Client(jig_client::Event::Unwatch { handle }));
        }
        if let Some(handle) = self.areas_handle {
            out.push(Request::Client(jig_client::Event::Unwatch { handle }));
        }
        if let Some(handle) = self.follows_handle {
            out.push(Request::Client(jig_client::Event::Unwatch { handle }));
        }
        *self = Self::default();
    }
    #[expect(clippy::too_many_lines, reason = "Team correlates its independent people, changes and areas reads")]
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
            Told::Changed { handle } if Some(*handle) == self.areas_handle => {
                let document = match client.document(*handle) {
                    Some(document) if document.state == jig_client::State::Ended => None,
                    Some(document) => match document.latest {
                        Some(Latest::Family(record)) => crate::forge::findings::document(record, &env.limits.forge),
                        Some(Latest::Jig(_)) => None,
                        None => return true,
                    },
                    None => return true,
                };
                self.areas = match document {
                    Some(document) if Some(document.project()) == self.project => Some(areas::gather(&document)),
                    Some(_) | None => None,
                };
                true
            }
            Told::Ended { handle, .. } if Some(*handle) == self.areas_handle => {
                self.areas = None;
                true
            }
            Told::Changed { handle } if Some(*handle) == self.follows_handle => {
                self.follows = match follows_panel(client, *handle, env.limits.forge, self.project) {
                    Some(panel) => Some(Box::new(panel)),
                    None => None,
                };
                true
            }
            Told::Ended { handle, .. } if Some(*handle) == self.follows_handle => {
                self.follows = None;
                true
            }
            Told::Changed { handle } if Some(*handle) == self.changes_handle => {
                let panel = match client.document(*handle) {
                    Some(document) if document.state == jig_client::State::Ended => None,
                    Some(document) => match document.latest {
                        Some(Latest::Family(record)) => crate::forge::changes::panel(record, env.limits.forge),
                        Some(Latest::Jig(_)) => None,
                        None => return true,
                    },
                    None => return true,
                };
                self.changes = match panel {
                    Some(panel) if Some(panel.project()) == self.project => Some(Box::new(panel)),
                    Some(_) | None => None,
                };
                true
            }
            Told::Ended { handle, .. } if Some(*handle) == self.changes_handle => {
                self.changes = None;
                true
            }
            Told::Changed { handle } if *handle == super::me_handle() && self.project.is_some() => {
                if self.reading != Reading::Asked
                    && let Some(me) = super::accounts::current_me(client)
                    && super::contains(me, self.project)
                {
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
                self.reading = if self.document.is_some() {
                    Reading::Idle
                } else if let Readout::Jig(c::ReadResult::None(c::ReadNone::Gone)) = page {
                    Reading::Gone
                } else {
                    Reading::Failed
                };
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
fn follows_panel(
    client: &Client,
    handle: Handle,
    limits: crate::forge::Limits,
    project: Option<u32>,
) -> Option<w::FollowsPanel> {
    let document = client.document(handle)?;
    if document.state == jig_client::State::Ended {
        return None;
    }
    let Latest::Family(record) = document.latest? else { return None };
    let panel = temper_client_protocol_wire::decode_follows_panel(&record.bytes, &crate::forge::wire_limits(&limits)?)?;
    if Some(panel.project()) == project { Some(panel) } else { None }
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
pub fn shown_at(root: &crate::Domain, project: u32, wall: Wall) -> Shown {
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
        gather(document, you, yours(root, project), others.as_slice(), state.changes.as_deref(), wall)
    } else {
        Facts {
            member: false,
            people: List::with_capacity(0),
            cut: false,
            areas: None,
            follows: List::with_capacity(0),
            roles: List::with_capacity(0),
            roles_cut: false,
            selected: None,
            card: None,
        }
    };
    facts.member = hand::member(root, project);
    facts.areas.clone_from(&state.areas);
    facts.follows = follows::gather(state.follows.as_deref());
    state.standing.gather(root.client(), state.document.as_deref(), you, &mut facts);
    if let Some(card) = &mut facts.card {
        card.sessions = state.sessions.rows(wall);
    }
    if state.document.is_some() {
        Shown::Read(facts)
    } else if facts.roles.is_empty() && !facts.roles_cut && facts.areas.is_none() {
        if state.reading == Reading::Failed || state.reading == Reading::Gone {
            Shown::Unreadable
        } else {
            Shown::Reading
        }
    } else {
        Shown::PeopleUnread {
            facts: Box::new(facts),
            failed: state.reading == Reading::Failed || state.reading == Reading::Gone,
        }
    }
}
/// An unlisted project's read outcome (client/pages/home.md, 1, A deleted project).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Missing {
    Reading,
    Deleted,
    NotOurs,
}

/// Home and How work runs share the rail's project read, including its answer
/// while the address keeps the same project.
#[must_use]
pub fn missing(root: &crate::Domain, project: u32) -> Missing {
    let state = &root.pages.team;
    if state.project != Some(project) {
        return Missing::Reading;
    }
    match state.reading {
        Reading::Asked | Reading::Refresh => Missing::Reading,
        Reading::Gone => Missing::Deleted,
        Reading::Idle | Reading::Failed => Missing::NotOurs,
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
    Some(gather(state.document.as_deref()?, you, None, &[], None, Wall::EPOCH).people.len())
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
