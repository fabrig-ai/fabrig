//! How work runs' parts, reaches, landing boxes and budgets
//! (temper/docs/design/client/pages/settings.md, 2, *Its parts* and *What reaches you*).
use alloc::boxed::Box;
use jig_client::contract as c;
use skein_lib::{List, Map, Queue, Wall};
use temper_client_protocol_wire::v1 as w;
use temper_protocol_reaches::{Class, Classes, Kind, Subject};

/// The project's `ways` read, as the page sees it.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Read {
    Reading,
    Ended,
    /// The stock way the project runs, none for its own.
    Shown {
        current: Option<u32>,
    },
}

/// What How work runs' parts are decided from.
#[expect(clippy::struct_excessive_bools, reason = "the page gathers independent facts for its parts")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Facts {
    pub period: Option<Period>,
    pub standing: Box<[Standing]>,
    /// `me` lists Ask temper's template in the project.
    pub asking: bool,
    pub read: Read,
    /// Kind 7 read, with Goals shown.
    pub goals: bool,
    /// The page has read at least one project role.
    pub roles: bool,
    /// The default goal template's budget, none with no template or kind 7 unread.
    pub budget: Option<c::TemplateBudget>,
    /// Kind 8 read for the project: `root.forge().classes(project)` answers.
    pub classes: bool,
    /// `me` lists two projects or more.
    pub several: bool,
}

/// The heading's way.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Head {
    Reading,
    Way(u32),
    Own,
}

/// The path section's state.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Path {
    Reading,
    Ended,
    Shown,
}

/// A goal budget row's figure.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Budget {
    Amount(u64),
    UpTo(u64),
}

/// What How work runs draws (client/pages/settings.md, 2, *Its parts*).
#[expect(clippy::struct_excessive_bools, reason = "each independent part has its own visibility rule")]
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Parts {
    pub period: Option<Period>,
    pub standing: Box<[Standing]>,
    pub head: Head,
    /// "Ask temper to change this".
    pub ask: bool,
    pub path: Path,
    pub roles: bool,
    /// A goal's amount or cap, while Goals is shown.
    pub budget: Option<Budget>,
    /// What reaches you draws its columns: `facts.classes`.
    pub reaches: bool,
    /// Its head says "each project has its own": `facts.several`.
    pub several: bool,
}

#[must_use]
pub fn parts(facts: &Facts) -> Parts {
    let (head, path) = match facts.read {
        Read::Reading => (Head::Reading, Path::Reading),
        Read::Ended => (Head::Reading, Path::Ended),
        Read::Shown { current: Some(index) } => (Head::Way(index), Path::Shown),
        Read::Shown { current: None } => (Head::Own, Path::Shown),
    };
    let budget = if facts.goals {
        match facts.budget {
            Some(c::TemplateBudget::Amount(amount)) => Some(Budget::Amount(amount)),
            Some(c::TemplateBudget::AsAllowed { cap: Some(amount) }) => Some(Budget::UpTo(amount)),
            Some(c::TemplateBudget::Given | c::TemplateBudget::AsAllowed { cap: None }) | None => None,
        }
    } else {
        None
    };
    Parts {
        period: facts.period,
        standing: facts.standing.clone(),
        head,
        ask: facts.asking,
        path,
        roles: facts.roles,
        budget,
        reaches: facts.classes,
        several: facts.several,
    }
}

/// Gather from `me`, the forge's kinds 7 and 8, ways and roles held for the address.
#[must_use]
pub fn facts(root: &crate::Domain, project: u32) -> Facts {
    use crate::{Address, forge::stock_ways::State};
    let pages = root.pages();
    let me = pages.me(root.client());
    let asking = match me {
        Some(me) => jig_client::shapes::template(me, project, super::chats::ASKING_TEMPLATE).is_some(),
        None => false,
    };
    let mut read = Read::Reading;
    if (pages.address() == &(Address::Ask { project }) || pages.address() == &(Address::Settings { project }))
        && let Some(ways) = pages.stock_ways()
    {
        read = match ways.state() {
            State::Loading => Read::Reading,
            State::Ended => Read::Ended,
            State::Shown(panel) => Read::Shown { current: *panel.current() },
        };
    }
    let mut goals = false;
    let mut budget = None;
    if let Some(ways) = root.forge().ways(project) {
        goals = ways.goals;
        if let Some(me) = me
            && let Some(number) = crate::forge::ways::default_way(me, project, &ways)
            && let Some(template) = jig_client::shapes::template(me, project, number)
        {
            budget = Some(template.budget);
        }
    }
    let roles = if pages.address() == &(Address::Settings { project }) {
        match pages.roles() {
            Some(roles) => match roles.read() {
                super::roles::Read::Drawn(w::View::Roles(view)) => !view.roles().is_empty(),
                super::roles::Read::Reading
                | super::roles::Read::Unreadable
                | super::roles::Read::Drawn(
                    w::View::Charter(_)
                    | w::View::Rule(_)
                    | w::View::NotDrawn(_)
                    | w::View::Way(_)
                    | w::View::Path(_)
                    | w::View::Proposal(_),
                ) => false,
            },
            None => false,
        }
    } else {
        false
    };
    let classes = root.forge().classes(project).is_some();
    let several = match me {
        Some(me) => me.projects.len() >= 2,
        None => false,
    };
    let period = match pages.team.document(project) {
        Some(document) => period(document),
        None => None,
    };
    let standing = match &pages.how {
        Some(open) => open.standing(root.client(), project),
        None => Box::default(),
    };
    Facts { period, standing, asking, read, goals, roles, budget, classes, several }
}

/// A subject's line in one of What reaches you's columns.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Line {
    pub subject: Subject,
    /// The item's place in the table, from 1; 0 for a subject with no item
    /// of its own: a movable kind or a set with none, and every fixed kind.
    pub item: u8,
    /// The line's class: its item's, `AskAway` with no item, `Wait` for a fixed kind.
    pub class: Class,
}

/// What reaches you's three columns (client/pages/settings.md, 2, *What reaches you*).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Reaches {
    /// "Agents decide, then tell you": the lines at `Decide`, in the settings order.
    pub decide: Box<[Line]>,
    /// "Asks you": the lines at `AskAway`, then those at `Ask`, each in the settings order.
    pub ask: Box<[Line]>,
    /// "Always waits for you": the fixed kinds in `Kind::ALL`'s order, item 0, `Wait`.
    pub wait: Box<[Line]>,
}

#[must_use]
pub fn reaches(classes: &Classes) -> Reaches {
    let mut decide = List::uncapped();
    let mut ask = List::uncapped();
    let mut wait = List::uncapped();
    settings_lines(classes, Class::Decide, &mut decide);
    settings_lines(classes, Class::AskAway, &mut ask);
    settings_lines(classes, Class::Ask, &mut ask);
    for kind in Kind::ALL {
        if kind.fixed() {
            wait.push(Line { subject: Subject::Kind(kind), item: 0, class: Class::Wait }).expect("fixed kinds fit");
        }
    }
    Reaches { decide: decide.into_boxed(), ask: ask.into_boxed(), wait: wait.into_boxed() }
}

fn settings_lines(classes: &Classes, class: Class, lines: &mut List<Line>) {
    for kind in Kind::ALL {
        if kind.fixed() {
            continue;
        }
        let subject = Subject::Kind(kind);
        let mut own = None;
        for (index, item) in classes.items.iter().enumerate() {
            if item.subject == subject && item.words.is_empty() {
                own = Some((index, item));
                break;
            }
        }
        match own {
            Some((index, item)) => {
                if item.class == class {
                    item_line(lines, subject, index, class);
                }
            }
            None => {
                if class == Class::AskAway {
                    lines.push(Line { subject, item: 0, class }).expect("settings lines grow");
                }
            }
        }
    }
    for (index, item) in classes.items.iter().enumerate() {
        let shown = match item.subject {
            Subject::Kind(kind) => !kind.fixed() && !item.words.is_empty(),
            Subject::Set(number) => classes.set(number).is_some(),
        };
        if shown && item.class == class {
            item_line(lines, item.subject, index, class);
        }
    }
    for set in &classes.sets {
        let subject = Subject::Set(set.number);
        let mut own = false;
        for item in &classes.items {
            if item.subject == subject && item.words.is_empty() {
                own = true;
                break;
            }
        }
        if !own && class == Class::AskAway {
            lines.push(Line { subject, item: 0, class }).expect("settings lines grow");
        }
    }
}

fn item_line(lines: &mut List<Line>, subject: Subject, index: usize, class: Class) {
    if let Ok(item) = u8::try_from(index.saturating_add(1)) {
        lines.push(Line { subject, item, class }).expect("settings lines grow");
    }
}

/// Budgets' period and rows (client/pages/settings.md, 2, *Budgets*).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Period {
    pub week: bool,
    pub next: Option<Wall>,
}
#[must_use]
pub fn period(project: &c::Project) -> Option<Period> {
    let current = project.current_period.as_ref()?;
    let week = match (current.began, current.next) {
        (Some(began), Some(next)) => next.as_nanos().checked_sub(began.as_nanos()) == Some(604_800_000_000_000),
        (Some(_) | None, None) | (None, Some(_)) => false,
    };
    Some(Period { week, next: current.next })
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Standing {
    pub number: u64,
    pub title: Box<[u8]>,
    pub left: u64,
    pub budget: u64,
}
#[must_use]
pub fn standing(tasks: &[c::TaskLine], project: u32) -> Box<[Standing]> {
    let mut rows = Map::uncapped();
    for task in tasks {
        if task.project == project && task.label == 3 && task.ended_at.is_none() {
            rows.insert(
                task.chip.task,
                Standing {
                    number: task.chip.task,
                    title: task.chip.title.bytes().into(),
                    left: task.budget.left,
                    budget: task.budget.numbers.budget,
                },
            )
            .expect("delivered rows fit");
        }
    }
    let mut ordered = List::uncapped();
    for (_, row) in &rows {
        ordered.push(row.clone()).expect("rows grow");
    }
    ordered.into_boxed()
}
/// Boxes after a landing (client/pages/settings.md, 2, *What follows a landing*).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Landed {
    pub follows: Box<[Follow]>,
    pub tiers: Option<Tiers>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Owner {
    You,
    Named(Box<[u8]>),
    Unknown,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Follow {
    pub source: Box<[u8]>,
    pub source_branch: Box<[u8]>,
    pub target: Box<[u8]>,
    pub target_branch: Box<[u8]>,
    pub paths: Box<[Box<[u8]>]>,
    pub owner: Owner,
    pub code_starts: bool,
    pub workflow: Box<[u8]>,
    pub lag: u32,
    pub parity: Box<[u8]>,
    pub stopped: Option<Box<[u8]>>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tier {
    pub name: Box<[u8]>,
    pub checks: Option<Box<[u8]>>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Tiers {
    pub branch: Box<[u8]>,
    pub tiers: Box<[Tier]>,
}
#[must_use]
pub fn follows(
    panel: Option<&w::FollowsPanel>,
    path: Option<&w::BranchPath>,
    members: Option<&[c::ProjectMember]>,
    reader: u64,
) -> Landed {
    let mut follows = List::uncapped();
    if let Some(panel) = panel {
        for row in panel.follows() {
            let setting = row.setting();
            let w::FollowOwner::Person(person) = setting.owner();
            let owner = follow_owner(person.person(), members, reader);
            let code_starts = match setting.start() {
                w::FollowStart::Code => true,
                w::FollowStart::Owner => false,
            };
            follows
                .push(Follow {
                    source: row.source().into(),
                    source_branch: row.source_branch().into(),
                    target: row.target().into(),
                    target_branch: row.target_branch().into(),
                    paths: setting.source().paths().clone().into_boxed(),
                    owner,
                    code_starts,
                    workflow: setting.verification().workflow().into(),
                    lag: row.lag(),
                    parity: crate::forge::follows::parity(row.parity().as_ref()),
                    stopped: row.stopped().clone(),
                })
                .expect("boxes grow");
        }
    }
    let mut tiers = List::uncapped();
    if let Some(path) = path {
        for stop in path.stops() {
            if stop.step() == &w::PathStep::Verify
                && let w::Opens::Tier(_) = stop.opens()
            {
                let mut checks = None;
                for fact in stop.card().facts() {
                    if fact.label() == b"Checks" {
                        checks = Some(fact.value().into());
                        break;
                    }
                }
                tiers.push(Tier { name: stop.card().name().into(), checks }).expect("tiers grow");
            }
        }
    }
    let tiers = if tiers.is_empty() {
        None
    } else {
        Some(Tiers { branch: path.expect("tiers came from a path").branch().into(), tiers: tiers.into_boxed() })
    };
    Landed { follows: follows.into_boxed(), tiers }
}
fn follow_owner(owner: u64, members: Option<&[c::ProjectMember]>, reader: u64) -> Owner {
    if owner == reader {
        return Owner::You;
    }
    if let Some(members) = members {
        for member in members {
            if member.party == owner {
                let name = if member.name.bytes().is_empty() { member.login.bytes() } else { member.name.bytes() };
                return Owner::Named(name.into());
            }
        }
    }
    Owner::Unknown
}
/// The running way, not the inspected tab (client/pages/settings.md, 2, *Its parts*).
#[must_use]
pub fn landed(root: &crate::Domain, project: u32) -> Landed {
    let pages = root.pages();
    if pages.address() != &(crate::Address::Settings { project }) {
        return follows(None, None, None, 0);
    }
    let panel = match &pages.how {
        Some(open) => open.panel(root.client()),
        None => None,
    };
    let path = match pages.stock_ways() {
        Some(ways) => ways.current_path(),
        None => None,
    };
    let members = match pages.team.document(project) {
        Some(document) => Some(document.members.as_slice()),
        None => None,
    };
    let reader = match pages.me(root.client()) {
        Some(me) => me.party.number,
        None => 0,
    };
    follows(panel.as_ref(), path, members, reader)
}
/// Settings-only watches (client/pages/settings.md, 2, *What the client holds*).
#[derive(Debug)]
pub(crate) struct Open {
    follows: Option<jig_client::Handle>,
    standing: jig_client::Handle,
    limits: crate::forge::Limits,
}
impl Open {
    pub(crate) fn new(project: u32, next: &mut u64, limits: &crate::Limits, out: &mut Queue<super::Request>) -> Self {
        let mut follows = None;
        if let Some(subject) = crate::forge::follows::watch(project, &limits.forge) {
            let handle = handle(next);
            out.push(super::Request::Client(jig_client::Event::Watch { handle, subject }));
            follows = Some(handle);
        }
        let standing = handle(next);
        let query = c::TaskQuery {
            project: Some(project),
            label: Some(3),
            order: c::QueryOrder::Number,
            requester: None,
            root: None,
            parent: None,
            phase: None,
            executor: None,
            tracked: None,
        };
        out.push(super::Request::Client(jig_client::Event::Watch {
            handle: standing,
            subject: jig_client::Watchable::Jig(c::Subject::Tasks { query, window: limits.client.domain.rows }),
        }));
        Self { follows, standing, limits: limits.forge }
    }
    pub(crate) fn leave(&self, out: &mut Queue<super::Request>) {
        if let Some(handle) = self.follows {
            out.push(super::Request::Client(jig_client::Event::Unwatch { handle }));
        }
        out.push(super::Request::Client(jig_client::Event::Unwatch { handle: self.standing }));
    }
    pub(crate) fn told(&self, told: &jig_client::Told) -> bool {
        let handle = match told {
            jig_client::Told::Changed { handle } | jig_client::Told::Ended { handle, .. } => *handle,
            jig_client::Told::Link { .. }
            | jig_client::Told::Kept { .. }
            | jig_client::Told::Answered { .. }
            | jig_client::Told::Pending { .. }
            | jig_client::Told::Dropped { .. }
            | jig_client::Told::Text { .. }
            | jig_client::Told::Page { .. } => return false,
        };
        Some(handle) == self.follows || handle == self.standing
    }
    fn panel(&self, client: &jig_client::Client) -> Option<w::FollowsPanel> {
        let doc = client.document(self.follows?)?;
        if doc.state == jig_client::State::Ended {
            return None;
        }
        let jig_client::Latest::Family(record) = doc.latest? else { return None };
        temper_client_protocol_wire::decode_follows_panel(&record.bytes, &crate::forge::wire_limits(&self.limits)?)
    }
    fn standing(&self, client: &jig_client::Client, project: u32) -> Box<[Standing]> {
        if let Some(doc) = client.document(self.standing)
            && doc.state != jig_client::State::Ended
            && let Some(jig_client::Latest::Jig(c::Delivery::Tasks(tasks))) = doc.latest
        {
            return standing(tasks.rows.as_slice(), project);
        }
        Box::default()
    }
}
fn handle(next: &mut u64) -> jig_client::Handle {
    let handle = jig_client::Handle { asker: crate::owner::Owner::Pages.asker(), number: *next };
    *next = next.checked_add(1).expect("page handles fit");
    handle
}
