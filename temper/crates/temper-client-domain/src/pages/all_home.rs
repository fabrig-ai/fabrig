//! Home on All projects (temper/docs/design/client/README.md, 3.5, Home on All projects).
use super::{
    home::{DecisionLine, Needs, Status},
    parts,
};
use crate::{Domain, address::Filter};
use alloc::boxed::Box;
use skein_lib::{List, Map, Wall, bytes};

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
#[must_use]
pub fn facts(root: &Domain, _wall: Wall) -> Facts {
    let mut projects = List::with_capacity(ACROSS);
    if let Some(me) = root.pages().me(root.client()) {
        for row in &me.projects {
            if projects.len() == ACROSS {
                break;
            }
            projects
                .push(Project { project: row.project, name: bytes::copy_of(row.name.bytes()), status: Status::Unknown })
                .expect("first ACROSS projects fit");
        }
    }
    let needs = match root.pages().inbox() {
        Some(inbox) if inbox.filter == Filter::ALL && inbox.known() => Needs::Count {
            count: u32::try_from(root.pages().needs().needs(inbox, root.forge().approvals()).len())
                .expect("inbox cards fit"),
            cut: *inbox.cut.first().expect("needs cut") || root.forge().approvals().more() > 0,
        },
        Some(_) | None => Needs::Unknown,
    };
    Facts { projects: projects.into_boxed(), needs, decided: Box::default(), showing: 0, away: None }
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
