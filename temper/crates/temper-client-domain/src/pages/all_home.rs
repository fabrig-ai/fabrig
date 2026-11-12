//! Home on All projects (temper/docs/design/client/pages/README.md, Home on All projects).
use super::{
    Request,
    decided::DecisionLine,
    home::{self, Home, Needs, Status},
    parts,
};
use crate::{Domain, address::Filter, forge, owner::Owner};
use alloc::boxed::Box;
use jig_client::{Client, Key, Told, contract::Me};
use skein_lib::{Env, List, Map, Queue, Wall, bytes};

pub const ACROSS: u32 = 16;
#[derive(Debug, PartialEq, Eq)]
pub struct Facts {
    pub projects: Box<[Project]>,
    pub needs: Needs,
    pub decided: Box<[Decided]>,
    pub showing: u32,
    pub away: Option<Spell>,
}
#[derive(Debug, PartialEq, Eq)]
pub struct Project {
    pub project: u32,
    pub name: Box<[u8]>,
    pub status: Status,
}
#[derive(Debug, PartialEq, Eq)]
pub struct Decided {
    pub project: u32,
    pub line: DecisionLine,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Spell {
    pub since: Wall,
    pub off: Wall,
}
#[derive(Debug, PartialEq, Eq)]
pub struct Intro {
    pub last_looked: Option<Wall>,
    pub away: Option<Spell>,
    pub landed: Landed,
    pub needs: Needs,
    pub brief: Option<Brief>,
}
#[derive(Debug, PartialEq, Eq)]
pub enum Landed {
    Nothing,
    Last { at: Wall },
    In { project: u32, name: Box<[u8]>, count: u64 },
    Across { count: u64, projects: u32 },
}
#[derive(Debug, PartialEq, Eq)]
pub struct Brief {
    pub lines: Box<[BriefLine]>,
    pub at: Wall,
}
#[derive(Debug, PartialEq, Eq)]
pub struct BriefLine {
    pub project: u32,
    pub name: Box<[u8]>,
    pub sentence: Box<[u8]>,
}
impl Facts {
    #[must_use]
    pub fn parts_facts(&self) -> parts::HomeFacts {
        parts::HomeFacts {
            across: true,
            projects: u32::try_from(self.projects.len()).expect("project rows fit"),
            brief: brief(self).is_some(),
            open: match self.needs {
                Needs::Count { count, .. } => count,
                Needs::Unknown => 0,
            },
            showing: self.showing,
            to_begin: false,
        }
    }
}
/// Reads shared by Home on All projects and Needs you, in `me`'s order.
#[derive(Debug)]
pub(crate) struct AllHome {
    homes: List<(u32, Home)>,
    statuses: bool,
    project: Option<u32>,
}
impl AllHome {
    pub(super) fn new(statuses: bool, project: Option<u32>) -> Self {
        Self { homes: List::with_capacity(ACROSS), statuses, project }
    }
    pub(super) fn leave(&self, out: &mut Queue<Request>) {
        for (_, home) in &self.homes {
            home.leave(out);
        }
    }
    pub(super) fn configure(
        &mut self,
        me: Option<&Me>,
        next: &mut u64,
        limits: forge::Limits,
        out: &mut Queue<Request>,
    ) {
        if !self.statuses {
            return;
        }
        let mut homes = List::with_capacity(ACROSS);
        if let Some(me) = me {
            for row in &me.projects {
                if self.project.is_some() && self.project != Some(row.project) {
                    continue;
                }
                if homes.len() == ACROSS {
                    break;
                }
                let mut found = None;
                for (index, (project, _)) in self.homes.iter().enumerate() {
                    if *project == row.project {
                        found = Some(u32::try_from(index).expect("project index fits"));
                        break;
                    }
                }
                let home = match found {
                    Some(index) => self.homes.swap_remove(index).expect("held project").1,
                    None => {
                        let number = *next;
                        *next = next.checked_add(2).expect("page handles fit");
                        Home::new(row.project, number, limits, out)
                    }
                };
                homes.push((row.project, home)).expect("first ACROSS projects fit");
            }
        }
        self.leave(out);
        self.homes = homes;
    }
    pub(super) fn told(
        &mut self,
        told: &Told,
        client: &Client,
        limits: forge::Limits,
        out: &mut Queue<Request>,
    ) -> bool {
        for (_, home) in self.homes.iter_mut() {
            if home.told(told, client, limits, out) {
                return true;
            }
        }
        false
    }
    fn rows(&self, root: &Domain, wall: Wall) -> (Box<[Decided]>, u32) {
        match root.pages().decided(self.project, wall) {
            Some(shown) => {
                let mut rows = List::with_capacity(7);
                for line in shown.rows {
                    rows.push(Decided { project: line.project, line }).expect("seven drawn decisions");
                }
                (rows.into_boxed(), shown.showing)
            }
            None => (Box::default(), 0),
        }
    }
}
fn reads(root: &Domain) -> Option<&AllHome> {
    match root.pages.address() {
        crate::Address::AllHome => root.pages.all_home.as_ref(),
        crate::Address::Inbox { .. } => root.pages.needs_decided.as_ref(),
        crate::Address::Home { .. }
        | crate::Address::Team { .. }
        | crate::Address::Ask { .. }
        | crate::Address::NewChat { .. }
        | crate::Address::Chats { .. }
        | crate::Address::Work { .. }
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
        | crate::Address::Settings { .. }
        | crate::Address::Role { .. }
        | crate::Address::Rule { .. }
        | crate::Address::Repositories { .. }
        | crate::Address::Models { .. }
        | crate::Address::Builders { .. }
        | crate::Address::Goals { .. }
        | crate::Address::Adopt { .. } => None,
    }
}
#[must_use]
pub fn decided(root: &Domain, wall: Wall) -> Box<[Decided]> {
    match reads(root) {
        Some(reads) => reads.rows(root, wall).0,
        None => Box::default(),
    }
}
pub(super) fn showing(root: &Domain, wall: Wall) -> u32 {
    match reads(root) {
        Some(reads) => reads.rows(root, wall).1,
        None => 0,
    }
}
#[must_use]
pub fn facts(root: &Domain, wall: Wall) -> Facts {
    let mut projects = List::with_capacity(ACROSS);
    if let Some(me) = root.pages().me(root.client()) {
        for row in &me.projects {
            if projects.len() == ACROSS {
                break;
            }
            let mut status = Status::Unknown;
            if let Some(reads) = &root.pages.all_home {
                for (project, home) in &reads.homes {
                    if *project == row.project {
                        status = home.shown_status();
                        break;
                    }
                }
            }
            projects
                .push(Project { project: row.project, name: bytes::copy_of(row.name.bytes()), status })
                .expect("first ACROSS projects fit");
        }
    }
    let needs = match root.pages().inbox() {
        Some(inbox) if inbox.filter == Filter::ALL && inbox.known() => Needs::Count {
            count: u32::try_from(root.pages().needs().all(inbox, root.forge().approvals()).len())
                .expect("inbox cards fit"),
            cut: *inbox.cut.first().expect("needs cut") || root.forge().approvals().more() > 0,
        },
        Some(_) | None => Needs::Unknown,
    };
    let mut previous = None;
    for project in &projects {
        match &project.status {
            Status::Shown(document) => previous = previous.max(*document.previous()),
            Status::Unknown => {}
        }
    }
    let mut away: Option<Spell> = None;
    for project in &projects {
        if let Some(row) = root.forge().away(project.project)
            && let Some(off) = row.ended(wall)
            && match previous {
                Some(previous) => off.as_nanos() > previous,
                None => true,
            }
            && match away {
                Some(held) => (off, row.since) > (held.off, held.since),
                None => true,
            }
        {
            away = Some(Spell { since: row.since, off });
        }
    }
    let (decided, showing) = match &root.pages.all_home {
        Some(reads) => reads.rows(root, wall),
        None => (Box::default(), 0),
    };
    Facts { projects: projects.into_boxed(), needs, decided, showing, away }
}
/// Reconcile only the seven displayed tasks, then send each settled project's entry look.
pub(super) fn look(root: &mut Domain, env: &Env<crate::Limits>, out: &mut Queue<crate::Request>) {
    let Some(reads) = reads(root) else { return };
    let mut settled = List::with_capacity(ACROSS);
    if reads.statuses
        && root.forge.settled()
        && (forge::ways::ended(root.forge()) || home::delivered(root.client(), forge::ways::handle()))
    {
        for (project, _) in &reads.homes {
            if home::repository(root, *project) != home::Repository::Unknown {
                settled.push(*project).expect("read projects fit");
            }
        }
    }
    let mut requests = Queue::uncapped();
    let pages = &mut root.pages;
    if let Some(inbox) = &pages.inbox {
        pages.next = pages.next.max(inbox.next);
    }
    let reads = match &mut pages.all_home {
        Some(reads) => Some(reads),
        None => pages.needs_decided.as_mut(),
    };
    if let Some(reads) = reads {
        for (project, home) in reads.homes.iter_mut() {
            let unsent = match home.look {
                home::Look::NotSent => true,
                home::Look::Out(_) | home::Look::Answered => false,
            };
            if settled.as_slice().contains(project)
                && unsent
                && let Some(request) = forge::status::ask(*project, &env.limits.forge)
            {
                pages.serial = pages.serial.checked_add(1).expect("page keys fit");
                let key = Key { asker: Owner::Pages.asker(), number: pages.serial };
                home.look = home::Look::Out(key);
                requests.push(Request::Client(jig_client::Event::Send { key, name: None, request }));
            }
        }
    }
    if let Some(inbox) = &mut pages.inbox {
        inbox.next = pages.next;
    }
    for _index in 0..requests.len() {
        match requests.pop().expect("bounded project requests") {
            Request::Client(event) => crate::route::client(root, env, event, out),
            Request::Screen { .. }
            | Request::KeepLook { .. }
            | Request::Show { .. }
            | Request::Replace { .. }
            | Request::OpenTab { .. } => unreachable!("project reads are client events"),
        }
    }
}
#[must_use]
pub fn intro(facts: &Facts) -> Intro {
    let mut last_looked = None;
    let mut last_landed = None;
    let mut count = 0_u64;
    let mut projects = 0_u32;
    let mut landed = Landed::Nothing;
    for row in &facts.projects {
        match &row.status {
            Status::Unknown => {}
            Status::Shown(document) => {
                if let Some(at) = document.previous() {
                    last_looked = last_looked.max(Some(Wall::from_nanos(*at)));
                }
                if document.landed() > 0
                    && let Some(at) = document.last_landed()
                {
                    last_landed = last_landed.max(Some(Wall::from_nanos(*at)));
                }
                let since = document.landed().saturating_sub(document.seen());
                if since > 0 {
                    count = count.saturating_add(since);
                    projects = projects.checked_add(1).expect("project rows fit");
                    landed = if projects == 1 {
                        Landed::In { project: row.project, name: row.name.clone(), count }
                    } else {
                        Landed::Across { count, projects }
                    };
                }
            }
        }
    }
    if projects == 0
        && let Some(at) = last_landed
    {
        landed = Landed::Last { at };
    }
    Intro { last_looked, away: facts.away, landed, needs: facts.needs, brief: brief(facts) }
}
#[must_use]
pub fn brief(facts: &Facts) -> Option<Brief> {
    let mut joined = Map::with_capacity(u32::try_from(facts.projects.len()).expect("project rows fit"));
    for row in &facts.projects {
        match &row.status {
            Status::Unknown => {}
            Status::Shown(document) => {
                if let Some(brief) = document.brief()
                    && (brief.landed() > document.seen()
                        || match document.previous() {
                            Some(previous) => brief.at() > *previous,
                            None => false,
                        })
                {
                    let line = BriefLine {
                        project: row.project,
                        name: row.name.clone(),
                        sentence: bytes::copy_of(first_sentence(brief.lede())),
                    };
                    joined.insert((brief.at(), row.project), line).expect("one brief per project fits");
                }
            }
        }
    }
    let (&(at, _), _) = joined.last()?;
    let mut lines = List::with_capacity(4);
    for _index in 0_u32..4 {
        let Some((&key, _)) = joined.last() else { break };
        let line = joined.remove(&key).expect("joined brief");
        lines.push(line).expect("four joined briefs fit");
    }
    Some(Brief { lines: lines.into_boxed(), at: Wall::from_nanos(at) })
}
#[must_use]
pub fn first_sentence(lede: &[u8]) -> &[u8] {
    for (index, byte) in lede.iter().enumerate() {
        let end = index.checked_add(1).expect("lede length fits");
        if (*byte == b'.' || *byte == b'!' || *byte == b'?') && (end == lede.len() || lede.get(end) == Some(&b' ')) {
            return lede.get(..end).expect("sentence within lede");
        }
    }
    lede
}
