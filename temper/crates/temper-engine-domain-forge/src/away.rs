//! Away spells and decisions (temper/docs/design/domain/forge/attention.md, 4, 5).
use crate::{Answered, Domain, Key, Limits, NamedCall, Request, Stored};
use alloc::boxed::Box;
use skein_lib::{Env, List, Queue, ReplyTo, Token, Wall};
use temper_protocol_reaches::{Ask, Kind, Reach, judge};

pub const AWAY_SUBJECT: u64 = 1 << 60;
pub const DECIDED_DAY_NS: u64 = 86_400_000_000_000;
pub const DECIDED_KEPT_NS: u64 = 604_800_000_000_000;
pub const DECISIONS_SHOWN: usize = 128;
pub const DECIDED_QUESTION_BYTES: usize = 400;
pub const DECIDED_OPTION_BYTES: usize = 120;
pub const DECIDED_WHY_BYTES: usize = 400;
pub const DECIDED_OPTIONS: core::ops::RangeInclusive<usize> = 2..=6;
#[must_use]
pub fn decided_subject(project: u32) -> u64 {
    (1 << 62) | (1 << 35) | u64::from(project)
}
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Away {
    pub party: u64,
    pub project: u32,
    pub since: Wall,
    pub until: Option<Wall>,
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
    pub took: Option<u8>,
    pub why: Box<[u8]>,
    pub overturned: bool,
    pub reach: Reach,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DecidedCall {
    pub question: Box<[u8]>,
    pub options: Box<[Box<[u8]>]>,
    pub took: u8,
    pub why: Box<[u8]>,
    pub kind: Kind,
    pub paths: Box<[Box<[u8]>]>,
    pub under: u8,
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
    pub reach: Reach,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct AskedView {
    pub number: u32,
    pub task: u64,
    pub at: Wall,
    pub reach: Reach,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct AwaysView {
    pub rows: Box<[Away]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct DecisionsView {
    pub project: u32,
    pub rows: Box<[DecisionView]>,
    pub asked: Box<[AskedView]>,
    pub cut: bool,
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
        None => Away { party, project, since: env.wall, until, landed, off: Some(env.wall) },
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
    always: bool,
    call: DecidedCall,
    out: &mut Queue<Request>,
) {
    let Some(words) = call.options.get(usize::from(call.took)) else {
        out.push(Request::Decided { to, key, took: None, reach: None });
        return;
    };
    let away = match d.aways.get(&(party, project)) {
        Some(row) => row.on_at(env.wall),
        None => false,
    };
    let reach = judge(
        &d.project_classes(project),
        &Ask { kind: call.kind, paths: &call.paths, under: call.under, away, always },
    );
    let took = if reach.decided() { Some(words.clone()) } else { None };
    let mut erases = List::with_capacity(env.limits.decided_per_project);
    let mut count = 0_u32;
    let mut highest = 0_u32;
    let mut oldest = None;
    for ((_, number), row) in d.decisions.range((project, 0)..=(project, u32::MAX)) {
        highest = *number;
        let age = env.wall.as_nanos().saturating_sub(row.at.as_nanos());
        if age > DECIDED_KEPT_NS {
            erases.push(*number).expect("bounded project decisions");
        } else {
            count = count.checked_add(1).expect("bounded decision rows");
            if age > DECIDED_DAY_NS {
                match oldest {
                    Some((at, _)) if at <= row.at => {}
                    Some(_) | None => oldest = Some((row.at, *number)),
                }
            }
        }
    }
    let swept = !erases.is_empty();
    for number in erases.into_boxed() {
        d.decisions.remove(&(project, number));
        out.push(Request::Erase { key: Key::Decided { project, number } });
    }
    if count >= env.limits.decided_per_project {
        let Some((_, number)) = oldest else {
            if swept {
                out.push(Request::Marked { tasks: Box::new([decided_subject(project)]) });
            }
            out.push(Request::Decided { to, key, took: None, reach: Some(reach) });
            return;
        };
        d.decisions.remove(&(project, number));
        out.push(Request::Erase { key: Key::Decided { project, number } });
    }
    let number = highest.checked_add(1).expect("project decision number fits u32");
    let (options, index, why) =
        if took.is_some() { (call.options, Some(call.took), call.why) } else { (Box::default(), None, Box::default()) };
    let row = Decided {
        project,
        number,
        party,
        task,
        at: env.wall,
        question: call.question,
        options,
        took: index,
        why,
        overturned: false,
        reach,
    };
    d.decisions.insert((project, number), row.clone()).expect("admitted project's decision room");
    out.push(Request::Save { record: Stored::Decided(row), release: None, read_afresh: None });
    out.push(Request::Marked { tasks: Box::new([decided_subject(project)]) });
    out.push(Request::Decided { to, key, took, reach: Some(reach) });
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
        Some(row) if row.party == party && row.took.is_some() => {
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
    crate::findings::erase_project(d, project, out);
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
    let mut erases = List::with_capacity(d.decisions.len());
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
        let shown = u32::try_from(DECISIONS_SHOWN).expect("document bound fits");
        let mut rows = List::with_capacity(shown);
        let mut asked = List::with_capacity(shown);
        let mut cut = false;
        for (_, row) in self.decisions.range((project, 0)..=(project, u32::MAX)).rev() {
            if row.party != party {
                continue;
            }
            match row.took {
                Some(index) => {
                    if rows.len() == shown {
                        cut = true;
                        continue;
                    }
                    let took = row.options.get(usize::from(index)).expect("restored taken option checked").clone();
                    rows.push(DecisionView {
                        number: row.number,
                        task: row.task,
                        at: row.at,
                        question: row.question.clone(),
                        took,
                        why: row.why.clone(),
                        overturned: row.overturned,
                        reach: row.reach,
                    })
                    .expect("document room checked");
                }
                None => {
                    if asked.len() == shown {
                        cut = true;
                        continue;
                    }
                    asked
                        .push(AskedView { number: row.number, task: row.task, at: row.at, reach: row.reach })
                        .expect("document room checked");
                }
            }
        }
        let mut rows = rows.into_boxed();
        let mut asked = asked.into_boxed();
        rows.reverse();
        asked.reverse();
        DecisionsView { project, rows, asked, cut }
    }
}
