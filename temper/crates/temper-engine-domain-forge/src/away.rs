//! Away spells and decisions (temper/docs/design/domain/forge.md, 5.8, 5.9).
use crate::{Answered, Domain, Key, Limits, NamedCall, Request, Stored};
use alloc::boxed::Box;
use skein_lib::{Env, List, Queue, ReplyTo, Token, Wall};

pub const AWAY_SUBJECT: u64 = 1 << 60;
pub const DECIDED_PER_PROJECT: usize = 64;
pub const DECIDED_QUESTION_BYTES: usize = 400;
pub const DECIDED_OPTION_BYTES: usize = 120;
pub const DECIDED_WHY_BYTES: usize = 400;
pub const DECIDED_OPTIONS: core::ops::RangeInclusive<usize> = 2..=6;
#[must_use]
pub fn decided_subject(project: u32) -> u64 {
    (1 << 62) | (1 << 35) | u64::from(project)
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum DecisionKind {
    Implementation,
    Retry,
    Wording,
    Dependency,
    Other,
}
impl DecisionKind {
    #[must_use]
    pub fn bit(self) -> Option<u8> {
        match self {
            Self::Implementation => Some(1),
            Self::Retry => Some(2),
            Self::Wording => Some(4),
            Self::Dependency => Some(8),
            Self::Other => None,
        }
    }
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Away {
    pub party: u64,
    pub project: u32,
    pub since: Wall,
    pub until: Option<Wall>,
    pub kinds: u8,
    pub landed: u64,
    pub off: Option<Wall>,
}
impl Away {
    #[must_use]
    pub fn on_at(&self, wall: Wall) -> bool {
        self.off.is_none()
            && match self.until {
                Some(until) => until > wall,
                None => true,
            }
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Decided {
    pub project: u32,
    pub number: u32,
    pub party: u64,
    pub task: u64,
    pub at: Wall,
    pub question: Box<[u8]>,
    pub options: Box<[Box<[u8]>]>,
    pub took: u8,
    pub why: Box<[u8]>,
    pub overturned: bool,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DecidedCall {
    pub question: Box<[u8]>,
    pub options: Box<[Box<[u8]>]>,
    pub took: u8,
    pub why: Box<[u8]>,
    pub kind: DecisionKind,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SetAwayAnswer {
    Away,
    TooLarge,
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum OverturnAnswer {
    Overturned,
    Already,
    Unknown,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DecisionView {
    pub number: u32,
    pub task: u64,
    pub at: Wall,
    pub question: Box<[u8]>,
    pub took: Box<[u8]>,
    pub why: Box<[u8]>,
    pub overturned: bool,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct AwaysView {
    pub rows: Box<[Away]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DecisionsView {
    pub project: u32,
    pub rows: Box<[DecisionView]>,
}

#[expect(clippy::too_many_arguments, reason = "the admitted switch request and decision environment")]
pub(crate) fn set_away(
    d: &mut Domain,
    env: &Env<Limits>,
    request: Token,
    token: Token,
    party: u64,
    project: u32,
    on: bool,
    until: Option<Wall>,
    kinds: u8,
    out: &mut Queue<Request>,
) {
    let key = (party, project);
    let held = d.aways.get(&key).copied();
    if held.is_none() && d.aways.len() == d.aways.capacity() {
        d.connect.answer(request, token, Answered::SetAway(SetAwayAnswer::TooLarge), out);
        return;
    }
    let landed = match d.project_facts.get(&project) {
        Some(row) => row.landed,
        None => 0,
    };
    let mut row = match held {
        Some(row) => row,
        None => Away { party, project, since: env.wall, until, kinds, landed, off: Some(env.wall) },
    };
    if on {
        if !row.on_at(env.wall) {
            row.since = env.wall;
            row.landed = landed;
            row.off = None;
        }
    } else if row.on_at(env.wall) {
        row.off = Some(env.wall);
    } else if row.off.is_none() {
        row.off = row.until;
    }
    row.until = until;
    row.kinds = kinds;
    if held != Some(row) {
        d.aways.insert(key, row).expect("away room checked");
        out.push(Request::Save { record: Stored::Away(row), release: None, read_afresh: None });
        out.push(Request::Marked { tasks: Box::new([AWAY_SUBJECT]) });
    }
    d.connect.answer(request, token, Answered::SetAway(SetAwayAnswer::Away), out);
}

#[expect(clippy::too_many_arguments, reason = "the named call and its caller cross the boundary whole")]
pub(crate) fn decided(
    d: &mut Domain,
    env: &Env<Limits>,
    to: ReplyTo,
    key: NamedCall,
    party: u64,
    project: u32,
    task: u64,
    call: DecidedCall,
    out: &mut Queue<Request>,
) {
    let mut allowed = false;
    if let Some(row) = d.aways.get(&(party, project))
        && let Some(bit) = call.kind.bit()
    {
        allowed = row.on_at(env.wall) && row.kinds & bit != 0;
    }
    let took = match call.options.get(usize::from(call.took)) {
        Some(words) if allowed => Some(words.clone()),
        Some(_) | None => None,
    };
    if took.is_some() {
        let mut count = 0_usize;
        let mut lowest = None;
        let mut highest = 0_u32;
        for ((_, number), _) in d.decisions.range((project, 0)..=(project, u32::MAX)) {
            if lowest.is_none() {
                lowest = Some(*number);
            }
            highest = *number;
            count = count.checked_add(1).expect("bounded decision rows");
        }
        let number = highest.checked_add(1).expect("project decision number fits u32");
        if count == DECIDED_PER_PROJECT {
            let number = lowest.expect("full project has a lowest decision");
            d.decisions.remove(&(project, number));
            out.push(Request::Erase { key: Key::Decided { project, number } });
        }
        let row = Decided {
            project,
            number,
            party,
            task,
            at: env.wall,
            question: call.question,
            options: call.options,
            took: call.took,
            why: call.why,
            overturned: false,
        };
        d.decisions.insert((project, number), row.clone()).expect("admitted project's decision room");
        out.push(Request::Save { record: Stored::Decided(row), release: None, read_afresh: None });
        out.push(Request::Marked { tasks: Box::new([decided_subject(project)]) });
    }
    out.push(Request::Decided { to, key, took });
}

pub(crate) fn overturn(
    d: &mut Domain,
    request: Token,
    token: Token,
    party: u64,
    project: u32,
    number: u32,
    out: &mut Queue<Request>,
) {
    let answer = match d.decisions.get_mut(&(project, number)) {
        Some(row) if row.party == party => {
            if row.overturned {
                OverturnAnswer::Already
            } else {
                row.overturned = true;
                out.push(Request::Save { record: Stored::Decided(row.clone()), release: None, read_afresh: None });
                out.push(Request::Marked { tasks: Box::new([decided_subject(project)]) });
                OverturnAnswer::Overturned
            }
        }
        Some(_) | None => OverturnAnswer::Unknown,
    };
    d.connect.answer(request, token, Answered::Overturn(answer), out);
}

pub(crate) fn erase_project(d: &mut Domain, project: u32, out: &mut Queue<Request>) {
    let mut erases = List::with_capacity(d.aways.len());
    for ((party, held), _) in &d.aways {
        if *held == project {
            erases.push(*party).expect("bounded away rows");
        }
    }
    if !erases.is_empty() {
        for party in erases.into_boxed() {
            d.aways.remove(&(party, project));
            out.push(Request::Erase { key: Key::Away { party, project } });
        }
        out.push(Request::Marked { tasks: Box::new([AWAY_SUBJECT]) });
    }
    let mut erases = List::with_capacity(u32::try_from(DECIDED_PER_PROJECT).expect("decision bound fits"));
    for ((_, number), _) in d.decisions.range((project, 0)..=(project, u32::MAX)) {
        erases.push(*number).expect("bounded project decisions");
    }
    if !erases.is_empty() {
        for number in erases.into_boxed() {
            d.decisions.remove(&(project, number));
            out.push(Request::Erase { key: Key::Decided { project, number } });
        }
        out.push(Request::Marked { tasks: Box::new([decided_subject(project)]) });
    }
}

impl Domain {
    pub(crate) fn aways_view(&self, party: u64) -> AwaysView {
        let mut rows = List::with_capacity(self.aways.len());
        for (_, row) in self.aways.range((party, 0)..=(party, u32::MAX)) {
            rows.push(*row).expect("bounded away rows");
        }
        AwaysView { rows: rows.into_boxed() }
    }
    pub(crate) fn decisions_view(&self, project: u32, party: u64) -> DecisionsView {
        let mut rows = List::with_capacity(u32::try_from(DECIDED_PER_PROJECT).expect("decision bound fits"));
        for (_, row) in self.decisions.range((project, 0)..=(project, u32::MAX)) {
            if row.party == party {
                let took = row.options.get(usize::from(row.took)).expect("restored taken option checked").clone();
                rows.push(DecisionView {
                    number: row.number,
                    task: row.task,
                    at: row.at,
                    question: row.question.clone(),
                    took,
                    why: row.why.clone(),
                    overturned: row.overturned,
                })
                .expect("bounded project decisions");
            }
        }
        DecisionsView { project, rows: rows.into_boxed() }
    }
}
