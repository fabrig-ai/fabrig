//! The person's findings in Needs you (temper/docs/design/client/pages/needs-you.md, 1).
use super::{Pages, Request, inbox::Inbox, needs};
use crate::{
    Domain, Limits,
    address::{Filter, Kind},
    forge,
    owner::Owner,
};
use alloc::boxed::Box;
use jig_client::{Client, Handle, Key, Latest, Told, contract as c};
use skein_lib::{Env, List, Queue, Wall};
use temper_client_protocol_wire::v1 as w;

pub const PROJECTS: u32 = 4;
pub const ROWS: u32 = 128;
pub const CARDS: u32 = PROJECTS.checked_mul(ROWS).expect("finding rows fit");
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Id {
    pub project: u32,
    pub number: u32,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Mode {
    Closed,
    Hand,
    Route,
    Drop,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Act {
    Open { id: Id },
    Show { id: Id, mode: Mode },
    Area { id: Id, words: Box<[u8]> },
    Why { id: Id, words: Box<[u8]> },
    Goal { id: Id, task: u64 },
    Reason { id: Id, words: Box<[u8]> },
    Hand { id: Id },
    Route { id: Id },
    Backlog { id: Id },
    Drop { id: Id },
}
#[derive(Debug)]
pub struct Card {
    pub id: Id,
    pub row: w::FindingRow,
    pub mode: Mode,
    pub area: Box<[u8]>,
    pub why: Box<[u8]>,
    pub why_beyond: u64,
    pub goal: Option<u64>,
    pub reason: Box<[u8]>,
    pub reason_beyond: u64,
    pub pending: Option<Key>,
    pub refusal: Option<w::FindingRefusal>,
    /// A send with an unknown outcome waits for the subject; it is never retried automatically.
    pub unconfirmed: bool,
}
impl Card {
    #[must_use]
    pub fn can_hand(&self) -> bool {
        self.pending.is_none() && !self.area.is_empty() && self.why_beyond == 0 && line(&self.why, 200)
    }
    #[must_use]
    pub fn can_drop(&self) -> bool {
        self.pending.is_none()
            && self.reason_beyond == 0
            && !self.reason.trim_ascii().is_empty()
            && self.reason.len() <= 400
    }
}
#[derive(Debug)]
struct Watched {
    project: u32,
    handle: Handle,
    goals_handle: Option<Handle>,
    goals: Option<Box<c::Tasks>>,
    document: Option<w::Findings>,
    cards: List<Card>,
}
#[derive(Debug)]
#[expect(
    clippy::partial_pub_fields,
    reason = "project watches are private; count truncation and the shown card are view facts"
)]
pub struct Store {
    projects: List<Watched>,
    pub more: bool,
    pub shown: Option<Id>,
    // Session watches count down below the four fixed handles; page-local reads grow up.
    next: u64,
}
impl Store {
    pub(super) fn new() -> Self {
        Self {
            projects: List::with_capacity(PROJECTS),
            more: false,
            shown: None,
            next: u64::MAX.checked_sub(4).expect("reserved findings handles"),
        }
    }
    #[must_use]
    pub fn cards(&self) -> Box<[&Card]> {
        let mut cards = List::with_capacity(CARDS);
        for project in &self.projects {
            for card in &project.cards {
                cards.push(card).expect("finding rows bounded");
            }
        }
        cards.into_boxed()
    }
    #[must_use]
    pub fn areas(&self, id: Id) -> &[w::AreaRow] {
        for project in &self.projects {
            if project.project == id.project
                && let Some(document) = &project.document
            {
                return document.areas().as_slice();
            }
        }
        &[]
    }
    #[must_use]
    pub fn goals(&self, id: Id) -> &[c::TaskLine] {
        for project in &self.projects {
            if project.project == id.project
                && let Some(goals) = &project.goals
            {
                return goals.rows.as_slice();
            }
        }
        &[]
    }
    #[must_use]
    pub fn cut(&self, project: Option<u32>) -> bool {
        if project.is_none() && self.more {
            return true;
        }
        for held in &self.projects {
            if (project.is_none() || project == Some(held.project))
                && let Some(document) = &held.document
                && document.cut()
            {
                return true;
            }
        }
        false
    }
    #[must_use]
    pub fn cut_for(&self, filter: Filter) -> bool {
        if filter.goal.is_some() || (filter.kind.is_some() && filter.kind != Some(Kind::Finding)) {
            return false;
        }
        self.cut(filter.project)
    }
    #[must_use]
    pub fn count(&self, project: Option<u32>) -> u32 {
        let mut count = 0_u32;
        for held in &self.projects {
            if project.is_none() || project == Some(held.project) {
                count = count.checked_add(held.cards.len()).expect("finding rows bounded");
            }
        }
        count
    }
    fn card_mut(&mut self, id: Id) -> Option<&mut Card> {
        for project in self.projects.iter_mut() {
            if project.project == id.project {
                for card in project.cards.iter_mut() {
                    if card.id == id {
                        return Some(card);
                    }
                }
            }
        }
        None
    }
    #[expect(clippy::too_many_lines, reason = "each finding tell stays beside its watch or request correlation")]
    pub(super) fn told(&mut self, told: &Told, client: &Client, limits: forge::Limits) -> bool {
        for project in self.projects.iter_mut() {
            if let Told::Changed { handle } = told {
                if *handle == project.handle {
                    if let Some(document) = client.document(*handle)
                        && let Some(Latest::Family(record)) = document.latest
                        && let Some(document) = forge::findings::document(record, &limits)
                        && document.project() == project.project
                    {
                        let mut cards = List::with_capacity(ROWS);
                        for row in document.rows() {
                            if *row.state() != w::FindingState::New {
                                continue;
                            }
                            let mut held = None;
                            for (index, card) in project.cards.iter().enumerate() {
                                if card.id.number == row.number() {
                                    held = Some(u32::try_from(index).expect("bounded rows"));
                                    break;
                                }
                            }
                            let card = match held {
                                Some(index) => {
                                    let mut card = project.cards.swap_remove(index).expect("held finding");
                                    card.row = row.clone();
                                    card.unconfirmed = false;
                                    card
                                }
                                None => Card {
                                    id: Id { project: project.project, number: row.number() },
                                    row: row.clone(),
                                    mode: Mode::Closed,
                                    area: Box::default(),
                                    why: Box::default(),
                                    why_beyond: 0,
                                    goal: None,
                                    reason: Box::default(),
                                    reason_beyond: 0,
                                    pending: None,
                                    refusal: None,
                                    unconfirmed: false,
                                },
                            };
                            cards.push(card).expect("document rows bounded");
                        }
                        project.cards = cards;
                        project.document = Some(document);
                    }
                    return true;
                }
                if project.goals_handle == Some(*handle) {
                    project.goals = match client.document(*handle) {
                        Some(document) => match document.latest {
                            Some(Latest::Jig(c::Delivery::Tasks(rows))) => Some(rows.clone()),
                            Some(Latest::Family(_) | Latest::Jig(_)) | None => None,
                        },
                        None => None,
                    };
                    return true;
                }
            }
            if let Told::Ended { handle, .. } = told {
                if *handle == project.handle {
                    project.cards.clear();
                    project.document = None;
                    return true;
                }
                if project.goals_handle == Some(*handle) {
                    project.goals = None;
                    return true;
                }
            }
            for card in project.cards.iter_mut() {
                let outcome = match told {
                    Told::Answered { key, outcome } if card.pending == Some(*key) => Some(outcome),
                    Told::Pending { key } if card.pending == Some(*key) => {
                        if let Some(pending) = client.pending(*key)
                            && pending.progress == jig_client::Progress::Unknown
                        {
                            Some(&jig_client::Outcome::Unknown)
                        } else {
                            return true;
                        }
                    }
                    Told::Answered { .. }
                    | Told::Pending { .. }
                    | Told::Changed { .. }
                    | Told::Ended { .. }
                    | Told::Link { .. }
                    | Told::Kept { .. }
                    | Told::Dropped { .. }
                    | Told::Text { .. }
                    | Told::Page { .. } => None,
                };
                if let Some(outcome) = outcome {
                    card.pending = None;
                    card.refusal =
                        if let Some(w::Outcome::FindingRefused(refused)) = forge::findings::outcome(outcome, limits) {
                            Some(refused.reason().clone())
                        } else {
                            None
                        };
                    card.unconfirmed = card.refusal.is_none();
                    return true;
                }
            }
        }
        false
    }
}
#[derive(Clone, Copy, Debug)]
pub enum Need<'a> {
    Existing(needs::Need<'a>),
    Finding(&'a Card),
}
impl Need<'_> {
    #[must_use]
    pub fn same(self, other: Self) -> bool {
        match (self, other) {
            (Self::Existing(left), Self::Existing(right)) => left.object() == right.object(),
            (Self::Finding(left), Self::Finding(right)) => left.id == right.id,
            (Self::Existing(_), Self::Finding(_)) | (Self::Finding(_), Self::Existing(_)) => false,
        }
    }
    #[must_use]
    pub fn open(self) -> super::Act {
        match self {
            Self::Existing(need) => super::Act::Needs(needs::Act::Open { object: need.object() }),
            Self::Finding(card) => super::Act::Finding(Act::Open { id: card.id }),
        }
    }
    #[must_use]
    pub fn ready(self, root: &Domain) -> bool {
        match self {
            Self::Existing(need) => needs::ready(root, need),
            Self::Finding(_) => true,
        }
    }
    fn at(self) -> Wall {
        match self {
            Self::Finding(card) => Wall::from_nanos(card.row.at()),
            Self::Existing(needs::Need::Approval(row)) => row.since,
            Self::Existing(needs::Need::Card(card)) => match &card.page {
                Some(c::ReadValue::Question(row)) => row.since,
                Some(c::ReadValue::PersonTask(row)) => row.since.unwrap_or(Wall::EPOCH),
                Some(c::ReadValue::Proposal(row)) => row.waiting.since,
                Some(c::ReadValue::Escalation(row)) => match &row.task.hold {
                    Some(hold) => hold.hold_wait_since.unwrap_or(Wall::EPOCH),
                    None => Wall::EPOCH,
                },
                Some(_) | None => Wall::EPOCH,
            },
        }
    }
}
impl needs::Session {
    #[must_use]
    pub const fn findings(&self) -> &Store {
        &self.findings
    }
    #[must_use]
    pub fn all<'a>(&'a self, inbox: &'a Inbox, approvals: &'a forge::approvals::Approvals) -> Box<[Need<'a>]> {
        if !inbox.known() {
            return Box::default();
        }
        let existing = self.needs(inbox, approvals);
        let mut rows = List::with_capacity(
            u32::try_from(existing.len())
                .expect("needs fit")
                .checked_add(self.findings.count(None))
                .expect("needs fit"),
        );
        for need in &existing {
            rows.push(Need::Existing(*need)).expect("needs fit");
        }
        if inbox.filter.goal.is_none() && (inbox.filter.kind.is_none() || inbox.filter.kind == Some(Kind::Finding)) {
            for card in &self.findings.cards() {
                if inbox.filter.project.is_none() || inbox.filter.project == Some(card.id.project) {
                    rows.push(Need::Finding(card)).expect("needs fit");
                }
            }
        }
        // The browser list is oldest wait first, with Later marks last.
        for index in 1..rows.len() {
            let mut at = index;
            while at > 0 {
                let previous = at.checked_sub(1).expect("positive index");
                let left = *rows.get(previous).expect("bounded needs");
                let right = *rows.get(at).expect("bounded needs");
                let left_later = if let Need::Existing(need) = left { self.later(need.object()) } else { false };
                let right_later = if let Need::Existing(need) = right { self.later(need.object()) } else { false };
                if (!left_later && right_later) || (left_later == right_later && left.at() <= right.at()) {
                    break;
                }
                *rows.get_mut(previous).expect("bounded needs") = right;
                *rows.get_mut(at).expect("bounded needs") = left;
                at = previous;
            }
        }
        rows.into_boxed()
    }
    #[must_use]
    pub fn shown_all<'a>(&'a self, inbox: &'a Inbox, approvals: &'a forge::approvals::Approvals) -> Option<Need<'a>> {
        let rows = self.all(inbox, approvals);
        for need in &rows {
            match need {
                Need::Finding(card) if self.findings.shown == Some(card.id) => return Some(*need),
                Need::Existing(need) if self.findings.shown.is_none() && self.at == Some(need.object()) => {
                    return Some(Need::Existing(*need));
                }
                Need::Finding(_) | Need::Existing(_) => {}
            }
        }
        rows.first().copied()
    }
}

#[expect(
    clippy::too_many_lines,
    reason = "the dispatcher keeps each finding act with its validation and forge request"
)]
pub(super) fn act(pages: &mut Pages, env: &Env<Limits>, act: Act, out: &mut Queue<Request>) {
    let id = match &act {
        Act::Open { id }
        | Act::Show { id, .. }
        | Act::Area { id, .. }
        | Act::Why { id, .. }
        | Act::Goal { id, .. }
        | Act::Reason { id, .. }
        | Act::Hand { id }
        | Act::Route { id }
        | Act::Backlog { id }
        | Act::Drop { id } => *id,
    };
    if let Act::Open { .. } = act {
        pages.needs.findings.shown = Some(id);
        if let crate::Address::Inbox { .. } = pages.address {
            return;
        }
        out.push(Request::Show { address: crate::Address::Inbox { filter: Filter::ALL } });
        return;
    }
    if let Act::Show { .. } = &act {
        pages.needs.findings.shown = Some(id);
    }
    let Some(bounds) = forge::wire_limits(&env.limits.forge) else { return };
    let mut goal_valid = false;
    if let Act::Route { .. } | Act::Goal { .. } = &act {
        let task = if let Act::Goal { task, .. } = &act {
            Some(*task)
        } else {
            match pages.needs.findings.card_mut(id) {
                Some(card) => card.goal,
                None => None,
            }
        };
        for row in pages.needs.findings.goals(id) {
            goal_valid |= Some(row.chip.task) == task;
        }
    }
    let mut area_valid = false;
    if let Act::Area { words, .. } = &act {
        for area in pages.needs.findings.areas(id) {
            area_valid |= area.name() == words.as_ref();
        }
    }
    let Some(card) = pages.needs.findings.card_mut(id) else { return };
    if card.pending.is_some() {
        return;
    }
    let triage = match act {
        Act::Open { .. } => return,
        Act::Show { mode, .. } => {
            card.mode = mode;
            return;
        }
        Act::Area { words, .. } => {
            if area_valid {
                card.area = words;
                card.refusal = None;
            }
            return;
        }
        Act::Why { words, .. } => {
            (card.why, card.why_beyond) = super::composer::held(words, 200);
            card.refusal = None;
            return;
        }
        Act::Reason { words, .. } => {
            (card.reason, card.reason_beyond) = super::composer::held(words, 400);
            card.refusal = None;
            return;
        }
        Act::Goal { task, .. } => {
            if goal_valid {
                card.goal = Some(task);
                card.refusal = None;
            }
            return;
        }
        Act::Hand { .. } => {
            if !card.can_hand() {
                return;
            }
            let Ok(hand) = w::HandFinding::new(
                &bounds,
                w::HandFindingParts {
                    project: id.project,
                    number: id.number,
                    area: card.area.clone(),
                    why: card.why.clone(),
                },
            ) else {
                return;
            };
            send(card, w::Request::HandFinding(hand), &mut pages.serial, env, out);
            return;
        }
        Act::Route { .. } => {
            if !goal_valid {
                return;
            }
            let Some(task) = card.goal else { return };
            let Ok(route) = w::FindingRoute::new(&bounds, w::FindingRouteParts { task }) else { return };
            w::Triage::Route(route)
        }
        Act::Backlog { .. } => w::Triage::Backlog,
        Act::Drop { .. } => {
            if !card.can_drop() {
                return;
            }
            let Ok(drop) = w::FindingDrop::new(&bounds, w::FindingDropParts { reason: card.reason.clone() }) else {
                return;
            };
            w::Triage::Drop(drop)
        }
    };
    let Ok(triage) =
        w::TriageFinding::new(&bounds, w::TriageFindingParts { project: id.project, number: id.number, act: triage })
    else {
        return;
    };
    send(card, w::Request::TriageFinding(triage), &mut pages.serial, env, out);
}
fn line(words: &[u8], max: usize) -> bool {
    !words.trim_ascii().is_empty() && words.len() <= max && !words.contains(&b'\n') && !words.contains(&b'\r')
}
fn send(card: &mut Card, request: w::Request, serial: &mut u64, env: &Env<Limits>, out: &mut Queue<Request>) {
    let Some(request) = forge::findings::request(card.id.project, request, env.limits.forge) else { return };
    *serial = serial.checked_add(1).expect("page keys fit");
    let key = Key { asker: Owner::Pages.asker(), number: *serial };
    card.pending = Some(key);
    card.refusal = None;
    card.unconfirmed = false;
    out.push(Request::Client(jig_client::Event::Send { key, name: None, request }));
}

pub(crate) fn reconcile(root: &mut Domain, env: &Env<Limits>, out: &mut Queue<crate::Request>) {
    let wanted = super::decided::wanted(root);
    let more = match root.pages.me(root.client()) {
        Some(me) => wanted.len() == PROJECTS && me.projects.len() > wanted.len(),
        None => false,
    };
    let mut requests = Queue::uncapped();
    let pages = &mut root.pages;
    let mut kept = List::with_capacity(PROJECTS);
    for held in core::mem::replace(&mut pages.needs.findings.projects, List::with_capacity(PROJECTS)).into_boxed() {
        if wanted.as_slice().contains(&held.project) {
            kept.push(held).expect("four projects");
        } else {
            requests.push(jig_client::Event::Unwatch { handle: held.handle });
            if let Some(handle) = held.goals_handle {
                requests.push(jig_client::Event::Unwatch { handle });
            }
        }
    }
    for project in wanted.into_boxed() {
        let mut present = false;
        for held in &kept {
            present |= held.project == project;
        }
        if !present && let Some(subject) = forge::findings::watch(project, &env.limits.forge) {
            let handle = Handle { asker: Owner::Pages.asker(), number: pages.needs.findings.next };
            pages.needs.findings.next = pages.needs.findings.next.checked_sub(1).expect("finding handles fit");
            kept.push(Watched {
                project,
                handle,
                goals_handle: None,
                goals: None,
                document: None,
                cards: List::with_capacity(ROWS),
            })
            .expect("four projects");
            requests.push(jig_client::Event::Watch { handle, subject });
        }
    }
    pages.needs.findings.projects = kept;
    pages.needs.findings.more = more;
    // Goal menus are watched only after a person opens Route to a goal.
    let shown = pages.needs.findings.shown;
    for held in pages.needs.findings.projects.iter_mut() {
        let mut routing = false;
        for card in &held.cards {
            routing |= card.mode == Mode::Route && Some(card.id) == shown;
        }
        if routing && held.goals_handle.is_none() {
            if let Some(document) = root.client.document(super::me_handle())
                && let Some(Latest::Jig(c::Delivery::Me(me))) = document.latest
                && let Some(query) = super::goals::query(me, held.project)
            {
                let handle = Handle { asker: Owner::Pages.asker(), number: pages.needs.findings.next };
                pages.needs.findings.next = pages.needs.findings.next.checked_sub(1).expect("finding handles fit");
                held.goals_handle = Some(handle);
                requests.push(jig_client::Event::Watch {
                    handle,
                    subject: jig_client::Watchable::Jig(c::Subject::Tasks {
                        query,
                        window: env.limits.client.domain.rows,
                    }),
                });
            }
        } else if !routing && let Some(handle) = held.goals_handle.take() {
            held.goals = None;
            requests.push(jig_client::Event::Unwatch { handle });
        }
    }
    while let Some(event) = requests.pop() {
        crate::route::client(root, env, event, out);
    }
}
