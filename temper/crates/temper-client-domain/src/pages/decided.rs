//! Decided for you: the open page's projects' decisions (temper/docs/design/client/pages/README.md, Decided for you).
use super::Pages;
use crate::{Address, Domain, Limits, Request, forge, owner::Owner};
use alloc::boxed::Box;
use jig_client::{Client, Handle, Latest, Readable, Readout, Told, contract as c};
use skein_lib::{Env, List, Queue, Wall, bytes};
use temper_client_protocol_wire::v1 as w;

pub const DECIDED_PROJECTS: usize = 4;
pub const DECIDED_SHOWN: usize = 7;
pub(super) const DAY: u64 = 86_400_000_000_000;
#[derive(Debug, PartialEq, Eq)]
pub struct Shown {
    pub showing: u32,
    pub more: bool,
    pub rows: Box<[DecisionLine]>,
    pub expires: Wall,
}
#[derive(Debug, PartialEq, Eq)]
pub struct DecisionLine {
    pub project: u32,
    pub number: u32,
    pub task: u64,
    pub title: Option<Box<[u8]>>,
    pub at: Wall,
    pub question: Box<[u8]>,
    pub took: Box<[u8]>,
    pub why: Box<[u8]>,
    pub overturned: bool,
    pub state: DecisionTask,
    pub reach: Option<temper_protocol_reaches::Reach>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum DecisionTask {
    Unknown,
    Live,
    Done(Wall),
    Ended(Wall),
}
/// A project's `decided` document as the open page holds it.
#[derive(Clone, Copy, Debug)]
pub enum Document<'a> {
    Unwatched,
    Unknown,
    Ended,
    Delivered(&'a w::Decisions),
}
#[derive(Debug)]
struct Watched {
    project: u32,
    handle: Option<Handle>,
    document: Option<Box<w::Decisions>>,
}
#[derive(Debug)]
struct DecisionRead {
    task: u64,
    handle: Handle,
    title: Option<Box<[u8]>>,
    state: DecisionTask,
}
#[derive(Debug)]
pub(super) struct Store {
    projects: List<Watched>,
    reads: List<DecisionRead>,
    more: bool,
}
impl Store {
    pub(super) fn new() -> Self {
        Self {
            projects: List::with_capacity(u32::try_from(DECIDED_PROJECTS).expect("decision bound fits")),
            reads: List::with_capacity(u32::try_from(DECIDED_SHOWN).expect("decision bound fits")),
            more: false,
        }
    }
    pub(super) fn told(
        &mut self,
        told: &Told,
        client: &Client,
        limits: forge::Limits,
        out: &mut Queue<super::Request>,
    ) -> bool {
        for project in self.projects.iter_mut() {
            match told {
                Told::Changed { handle } if project.handle == Some(*handle) => {
                    project.document = match client.document(*handle) {
                        Some(document) if document.state != jig_client::State::Ended => match document.latest {
                            Some(Latest::Family(record)) => match forge::decided::document(record, limits) {
                                Some(document) if document.project() == project.project => Some(Box::new(document)),
                                Some(_) | None => None,
                            },
                            Some(Latest::Jig(_)) | None => None,
                        },
                        Some(_) | None => None,
                    };
                    return true;
                }
                Told::Ended { handle, .. } if project.handle == Some(*handle) => {
                    project.handle = None;
                    project.document = None;
                    out.push(super::Request::Client(jig_client::Event::Unwatch { handle: *handle }));
                    return true;
                }
                Told::Changed { .. }
                | Told::Ended { .. }
                | Told::Link { .. }
                | Told::Pending { .. }
                | Told::Answered { .. }
                | Told::Kept { .. }
                | Told::Dropped { .. }
                | Told::Text { .. }
                | Told::Page { .. } => {}
            }
        }
        match told {
            Told::Page { handle, page } => {
                for row in self.reads.iter_mut() {
                    if row.handle == *handle {
                        let task = task_read(page, row.task);
                        row.title = match task {
                            Some((task, _)) => Some(bytes::copy_of(task.chip.title.bytes())),
                            None => None,
                        };
                        row.state = match task {
                            Some((task, ended)) => decision_task(task, ended),
                            None => DecisionTask::Unknown,
                        };
                        return true;
                    }
                }
            }
            Told::Changed { .. }
            | Told::Ended { .. }
            | Told::Link { .. }
            | Told::Pending { .. }
            | Told::Answered { .. }
            | Told::Kept { .. }
            | Told::Dropped { .. }
            | Told::Text { .. } => {}
        }
        false
    }
    fn read_tasks(&mut self, shown: Option<&Shown>, next: &mut u64, out: &mut Queue<super::Request>) {
        let rows = match shown {
            Some(shown) => shown.rows.as_ref(),
            None => &[],
        };
        let mut kept = List::with_capacity(u32::try_from(DECIDED_SHOWN).expect("decision bound fits"));
        for read in core::mem::replace(
            &mut self.reads,
            List::with_capacity(u32::try_from(DECIDED_SHOWN).expect("decision bound fits")),
        )
        .into_boxed()
        {
            let mut retain = false;
            for row in rows {
                retain |= row.task == read.task;
            }
            if retain {
                kept.push(read).expect("seven drawn reads");
            } else {
                out.push(super::Request::Client(jig_client::Event::Unwatch { handle: read.handle }));
            }
        }
        self.reads = kept;
        for row in rows {
            let mut held = false;
            for read in &self.reads {
                held |= read.task == row.task;
            }
            if held {
                continue;
            }
            let handle = Handle { asker: Owner::Pages.asker(), number: *next };
            *next = next.checked_add(1).expect("page handles fit");
            self.reads
                .push(DecisionRead { task: row.task, handle, title: None, state: DecisionTask::Unknown })
                .expect("seven drawn reads");
            out.push(super::Request::Client(jig_client::Event::Read {
                handle,
                read: Readable::Jig(c::Read::Task { task: row.task }),
            }));
        }
    }
}
fn task_read(page: &Readout, task: u64) -> Option<(&c::Task, Option<Wall>)> {
    match page {
        Readout::Jig(c::ReadResult::Value(c::ReadValue::Task(document))) if document.chip.task == task => {
            Some((document, None))
        }
        Readout::Jig(c::ReadResult::Value(c::ReadValue::Ended(ended))) if ended.matches(c::ObjectKey::Task(task)) => {
            match &ended.value {
                c::EndedValue::Task { document, .. } => Some((document, ended.at)),
                c::EndedValue::Proposal(_)
                | c::EndedValue::PersonProposal { .. }
                | c::EndedValue::Escalation { .. }
                | c::EndedValue::Question { .. }
                | c::EndedValue::PersonTask { .. } => None,
            }
        }
        Readout::Jig(_) | Readout::Family { .. } | Readout::Limit | Readout::NotServed { .. } => None,
    }
}
fn decision_task(task: &c::Task, ended: Option<Wall>) -> DecisionTask {
    match task.chip.details.phase {
        c::ChipPhase::Ended => {
            let Some(at) = ended.or(task.chip.details.details.ended_at) else { return DecisionTask::Unknown };
            match task.chip.details.details.ending {
                Some(c::ChipEnding::Done(_)) => DecisionTask::Done(at),
                Some(c::ChipEnding::Failed(_) | c::ChipEnding::Cancelled { .. }) | None => DecisionTask::Ended(at),
            }
        }
        c::ChipPhase::Waiting { .. }
        | c::ChipPhase::Active(_)
        | c::ChipPhase::Person { .. }
        | c::ChipPhase::Closing(_)
        | c::ChipPhase::Held { .. } => DecisionTask::Live,
    }
}

impl Pages {
    /// The rows showing at the wall, of one project or, with `None`, of every watched one, the session's marks applied.
    #[must_use]
    pub fn decided(&self, project: Option<u32>, wall: Wall) -> Option<Shown> {
        let mut showing = 0_u32;
        let mut expires = Wall::from_nanos(u64::MAX);
        let mut more = project.is_none() && self.decided.more;
        for held in &self.decided.projects {
            if let Some(project) = project
                && project != held.project
            {
                continue;
            }
            let Some(document) = &held.document else { continue };
            more |= document.cut();
            for row in document.rows() {
                if wall.as_nanos().saturating_sub(row.at()) < DAY {
                    showing = showing.checked_add(1).expect("four bounded documents");
                    expires = expires.min(Wall::from_nanos(row.at().saturating_add(DAY)));
                }
            }
        }
        if showing == 0 {
            return None;
        }
        let mut rows = List::with_capacity(u32::try_from(DECIDED_SHOWN).expect("decision bound fits"));
        let mut before = None;
        // Select the newest seven across four bounded documents, then draw oldest first.
        for _index in 0..DECIDED_SHOWN {
            let mut newest: Option<(u32, &w::Decision)> = None;
            for held in &self.decided.projects {
                if let Some(project) = project
                    && project != held.project
                {
                    continue;
                }
                let Some(document) = &held.document else { continue };
                for row in document.rows() {
                    let key = (row.at(), held.project, row.number());
                    if wall.as_nanos().saturating_sub(row.at()) >= DAY {
                        continue;
                    }
                    if let Some(before) = before
                        && key >= before
                    {
                        continue;
                    }
                    let later = match newest {
                        Some((project, held)) => key > (held.at(), project, held.number()),
                        None => true,
                    };
                    if later {
                        newest = Some((held.project, row));
                    }
                }
            }
            let Some((project, row)) = newest else { break };
            before = Some((row.at(), project, row.number()));
            let mut title = None;
            let mut state = DecisionTask::Unknown;
            for read in &self.decided.reads {
                if read.task == row.task() {
                    title.clone_from(&read.title);
                    state = read.state;
                    break;
                }
            }
            rows.push(DecisionLine {
                project,
                number: row.number(),
                task: row.task(),
                title,
                state,
                at: Wall::from_nanos(row.at()),
                question: row.question().into(),
                took: row.took().into(),
                why: row.why().into(),
                overturned: row.overturned() || self.overturns.marked(project, row.number()),
                reach: temper_protocol_reaches::reach_from(row.reach()),
            })
            .expect("seven shown decisions");
        }
        let mut rows = rows.into_boxed();
        rows.reverse();
        Some(Shown { showing, more, rows, expires })
    }
    /// A project's document, for a need's reason (`needs-why`).
    #[must_use]
    pub fn decisions(&self, project: u32) -> Document<'_> {
        for held in &self.decided.projects {
            if held.project != project {
                continue;
            }
            return match (&held.document, held.handle) {
                (Some(document), _) => Document::Delivered(document),
                (None, Some(_)) => Document::Unknown,
                (None, None) => Document::Ended,
            };
        }
        Document::Unwatched
    }
}
fn wanted(root: &Domain) -> List<u32> {
    let pages = &root.pages;
    let mut wanted = List::with_capacity(u32::try_from(DECIDED_PROJECTS).expect("decision bound fits"));
    if !pages.watching {
        return wanted;
    }
    match pages.address {
        Address::Home { project } => {
            wanted.push(project).expect("one Home project");
        }
        Address::AllHome | Address::Inbox { .. } => {
            let project = if let Address::Inbox { filter } = pages.address { filter.project } else { None };
            if let Some(project) = project {
                wanted.push(project).expect("one filtered project");
            } else {
                if let Some(inbox) = &pages.inbox {
                    let project = match pages.needs.shown(inbox, root.forge().approvals()) {
                        Some(super::needs::Need::Card(card)) => card.project,
                        Some(super::needs::Need::Approval(row)) => Some(row.project),
                        None => None,
                    };
                    if let Some(project) = project {
                        wanted.push(project).expect("shown need first");
                    }
                }
                if let Some(me) = pages.me(root.client()) {
                    for row in &me.projects {
                        if usize::try_from(wanted.len()).expect("four projects") == DECIDED_PROJECTS {
                            break;
                        }
                        if !wanted.as_slice().contains(&row.project) {
                            wanted.push(row.project).expect("four projects");
                        }
                    }
                }
            }
        }
        Address::NewChat { .. }
        | Address::Team { .. }
        | Address::Chats { .. }
        | Address::Accounts
        | Address::Forges
        | Address::ConnectForge { .. }
        | Address::ConnectModel { .. }
        | Address::Welcome { .. }
        | Address::Chat { .. }
        | Address::Task { .. }
        | Address::Changes { .. }
        | Address::Settings { .. }
        | Address::Repositories { .. }
        | Address::Models { .. }
        | Address::Builders { .. }
        | Address::Role { .. }
        | Address::Rule { .. }
        | Address::Goals { .. }
        | Address::Ask { .. }
        | Address::Work { .. }
        | Address::Told
        | Address::Remembered
        | Address::Members
        | Address::Limits
        | Address::Adopt { .. } => {}
    }
    wanted
}
/// Keep watches and task reads while their projects and rows remain on the open page.
pub(crate) fn reconcile(root: &mut Domain, env: &Env<Limits>, out: &mut Queue<Request>) {
    let wanted = wanted(root);
    let more = match root.pages.me(root.client()) {
        Some(me) => me.projects.len() > wanted.len(),
        None => false,
    };
    let beginning = match root.pages.address {
        Address::Home { project } => super::home::facts(root, project, env.wall).to_begin(),
        Address::AllHome
        | Address::Inbox { .. }
        | Address::NewChat { .. }
        | Address::Team { .. }
        | Address::Chats { .. }
        | Address::Accounts
        | Address::Forges
        | Address::ConnectForge { .. }
        | Address::ConnectModel { .. }
        | Address::Welcome { .. }
        | Address::Chat { .. }
        | Address::Task { .. }
        | Address::Changes { .. }
        | Address::Settings { .. }
        | Address::Repositories { .. }
        | Address::Models { .. }
        | Address::Builders { .. }
        | Address::Role { .. }
        | Address::Rule { .. }
        | Address::Goals { .. }
        | Address::Ask { .. }
        | Address::Work { .. }
        | Address::Told
        | Address::Remembered
        | Address::Members
        | Address::Limits
        | Address::Adopt { .. } => false,
    };
    let pages = &mut root.pages;
    if let Some(inbox) = &pages.inbox {
        pages.next = pages.next.max(inbox.next);
    }
    let mut requests = Queue::uncapped();
    let mut kept = List::with_capacity(u32::try_from(DECIDED_PROJECTS).expect("decision bound fits"));
    for held in core::mem::replace(
        &mut pages.decided.projects,
        List::with_capacity(u32::try_from(DECIDED_PROJECTS).expect("decision bound fits")),
    )
    .into_boxed()
    {
        if wanted.as_slice().contains(&held.project) {
            kept.push(held).expect("four projects");
        } else if let Some(handle) = held.handle {
            requests.push(super::Request::Client(jig_client::Event::Unwatch { handle }));
        }
    }
    for project in wanted.into_boxed() {
        let mut present = false;
        for held in &kept {
            present |= held.project == project;
        }
        if present {
            continue;
        }
        let handle = if let Some(subject) = forge::decided::watch(project, &env.limits.forge) {
            let handle = Handle { asker: Owner::Pages.asker(), number: pages.next };
            pages.next = pages.next.checked_add(1).expect("page handles fit");
            requests.push(super::Request::Client(jig_client::Event::Watch { handle, subject }));
            Some(handle)
        } else {
            None
        };
        kept.push(Watched { project, handle, document: None }).expect("four projects");
    }
    pages.decided.projects = kept;
    pages.decided.more = more;
    let shown = if beginning { None } else { pages.decided(None, env.wall) };
    pages.decided.read_tasks(shown.as_ref(), &mut pages.next, &mut requests);
    if let Some(inbox) = &mut pages.inbox {
        inbox.next = pages.next;
    }
    for _index in 0..requests.len() {
        match requests.pop().expect("bounded reconciliation queue") {
            super::Request::Client(event) => crate::route::client(root, env, event, out),
            super::Request::Push(_)
            | super::Request::Screen { .. }
            | super::Request::KeepLook { .. }
            | super::Request::Show { .. }
            | super::Request::Replace { .. }
            | super::Request::OpenTab { .. } => unreachable!("decided reconciles client requests"),
        }
    }
}
