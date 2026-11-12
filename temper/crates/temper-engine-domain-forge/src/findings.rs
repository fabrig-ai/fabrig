//! Findings (temper/docs/design/domain/findings.md, 2, 6, 9; domain/areas.md, 3 to 7).
use alloc::boxed::Box;
use crate::{Domain, Key, Limits, Request, Stored, settings::{AreaOwner, ProjectAreas}};
use skein_lib::{List, Queue, Wall};
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Holder { Task(u64), Party(u64), Role(u32) }
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Filer { Task(u64), Party(u64) }
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FindingKind { Qa, Flake, Red, Review, Feedback }
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Severity { Low, Medium, High }
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum FindingState { New, Routed, Backlog, Dropped }
impl FindingState {
    #[must_use]
    pub const fn ended(self) -> bool {
        match self { Self::Routed | Self::Dropped => true, Self::New | Self::Backlog => false }
    }
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Finding {
    pub project: u32, pub number: u32, pub kind: FindingKind, pub severity: Severity,
    pub area: Option<Box<[u8]>>, pub state: FindingState, pub title: Box<[u8]>, pub body: Box<[u8]>,
    pub filer: Filer, pub at: Wall, pub why: Option<Box<[u8]>>, pub hander: Option<Filer>,
    pub goal: Option<u64>, pub reason: Option<Box<[u8]>>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FileFinding {
    pub project: u32, pub kind: FindingKind, pub severity: Severity,
    pub area: Option<Box<[u8]>>, pub title: Box<[u8]>, pub body: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct HandFinding { pub project: u32, pub number: u32, pub area: Box<[u8]>, pub why: Box<[u8]> }
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct TriageFinding { pub project: u32, pub number: u32, pub act: Triage }
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Triage { Route(u64), Backlog, Drop(Box<[u8]>) }
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum FindingRefusal { NoArea(Box<[Box<[u8]>]>), Full(u32), NotYours(Holder), NoFinding, NoGoal, Ended, Words(Box<[u8]>) }
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum FileFindingAnswer { Filed { number: u32 }, Refused(FindingRefusal) }
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum HandFindingAnswer { Handed, Refused(FindingRefusal) }
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum TriageFindingAnswer { Triaged, Refused(FindingRefusal) }
/// The three-step rule; standing comes from the root's snapshot of the core.
#[must_use]
pub fn owner(area: Option<&[u8]>, areas: &ProjectAreas, triager: Option<AreaOwner>, stands: &impl Fn(AreaOwner) -> Option<Holder>) -> Holder {
    for row in &areas.areas {
        if area == Some(row.name.as_ref()) && let Some(holder) = stands(row.owner) { return holder; }
    }
    if let Some(triager) = triager && let Some(holder) = stands(triager) { return holder; }
    Holder::Role(0)
}
/// Filters are part of the subject identity; each write marks every filter.
#[must_use]
pub fn findings_subject(project: u32, state: Option<FindingState>, mine: bool) -> u64 {
    let state = match state { None => 0, Some(FindingState::New) => 1, Some(FindingState::Routed) => 2, Some(FindingState::Backlog) => 3, Some(FindingState::Dropped) => 4 };
    (1 << 59) | (state << 33) | (u64::from(mine) << 32) | u64::from(project)
}
#[must_use]
pub fn findings_filter(subject: u64, project: u32) -> Option<(Option<FindingState>, bool)> {
    for state in [None, Some(FindingState::New), Some(FindingState::Routed), Some(FindingState::Backlog), Some(FindingState::Dropped)] {
        for mine in [false, true] { if subject == findings_subject(project, state, mine) { return Some((state, mine)); } }
    }
    None
}
pub fn mark_findings(project: u32, out: &mut Queue<Request>) {
    let mut subjects = List::with_capacity(10);
    for state in [None, Some(FindingState::New), Some(FindingState::Routed), Some(FindingState::Backlog), Some(FindingState::Dropped)] {
        for mine in [false, true] { subjects.push(findings_subject(project, state, mine)).expect("ten finding subjects"); }
    }
    out.push(Request::Marked { tasks: subjects.into_boxed() });
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FindingRow {
    pub number: u32, pub kind: FindingKind, pub severity: Severity, pub area: Option<Box<[u8]>>,
    pub state: FindingState, pub title: Box<[u8]>, pub holder: Holder, pub at: Wall, pub why: Option<Box<[u8]>>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct AreaRow { pub name: Box<[u8]>, pub words: Box<[u8]>, pub owner: AreaOwner, pub holder: Option<Holder> }
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Findings {
    pub project: u32, pub rows: Box<[FindingRow]>, pub areas: Box<[AreaRow]>,
    pub triager: Option<AreaOwner>, pub holder: Holder, pub cut: bool,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FindingAnswer { pub row: FindingRow, pub body: Box<[u8]>, pub filer: Filer, pub goal: Option<u64>, pub reason: Option<Box<[u8]>> }
impl Finding {
    #[must_use]
    pub fn row(&self, holder: Holder) -> FindingRow {
        FindingRow { number: self.number, kind: self.kind, severity: self.severity, area: self.area.clone(), state: self.state, title: self.title.clone(), holder, at: self.at, why: self.why.clone() }
    }
}
/// A judged write is committed after the root admits the receiving subscription.
#[derive(Hash, Debug)]
pub struct FindingWrite { pub row: Finding, pub erase: Option<u32> }
impl Domain {
    #[must_use]
    pub fn project_areas(&self, project: u32) -> ProjectAreas {
        if let Some(crate::settings::SizedValue { value: crate::settings::Value::ProjectAreas(areas), .. }) = self.setting(12, project, 1) { return areas.clone(); }
        ProjectAreas { areas: Box::default() }
    }
    #[must_use]
    pub fn project_triager(&self, project: u32) -> Option<AreaOwner> {
        if let Some(crate::settings::SizedValue { value: crate::settings::Value::ProjectTriager(triager), .. }) = self.setting(13, project, 1) { return Some(triager.owner); }
        None
    }
    #[must_use]
    pub fn finding(&self, project: u32, number: u32) -> Option<&Finding> { self.findings.get(&(project, number)) }
    pub fn findings(&self, project: u32) -> impl DoubleEndedIterator<Item = &Finding> {
        self.findings.range((project, 0)..=(project, u32::MAX)).map(|(_, row)| row)
    }
    fn finding_area(&self, project: u32, name: &[u8]) -> Result<(), FindingRefusal> {
        let areas = self.project_areas(project);
        let mut names = List::with_capacity(16);
        for area in areas.areas {
            if area.name.as_ref() == name { return Ok(()); }
            names.push(area.name).expect("bounded areas");
        }
        Err(FindingRefusal::NoArea(names.into_boxed()))
    }
    pub fn prepare_file_finding(&self, limits: &Limits, at: Wall, filer: Filer, call: FileFinding) -> Result<FindingWrite, FindingRefusal> {
        if !crate::settings::words_fit(&call.title, 200) { return Err(FindingRefusal::Words(b"title".as_slice().into())); }
        if !body_fits(&call.body) { return Err(FindingRefusal::Words(b"body".as_slice().into())); }
        if let Some(area) = &call.area { self.finding_area(call.project, area)?; }
        let mut highest = 0_u32; let mut count = 0_u32; let mut new = 0_u32; let mut oldest = None;
        for row in self.findings(call.project) {
            highest = row.number; count = count.saturating_add(1);
            if row.state == FindingState::New { new = new.saturating_add(1); }
            if row.state.ended() {
                match oldest { Some((at, _)) if at <= row.at => {}, Some(_) | None => oldest = Some((row.at, row.number)) }
            }
        }
        let erase = if count >= limits.findings_per_project {
            let Some((_, number)) = oldest else { return Err(FindingRefusal::Full(new)); }; Some(number)
        } else { None };
        if erase.is_none() && self.findings.len() == self.findings.capacity() { return Err(FindingRefusal::Full(new)); }
        let Some(number) = highest.checked_add(1) else { return Err(FindingRefusal::Words(b"finding number exhausted".as_slice().into())); };
        Ok(FindingWrite { row: Finding { project: call.project, number, kind: call.kind, severity: call.severity, area: call.area, state: FindingState::New, title: call.title, body: call.body, filer, at, why: None, hander: None, goal: None, reason: None }, erase })
    }
    pub fn prepare_hand_finding(&self, filer: Filer, call: HandFinding) -> Result<FindingWrite, FindingRefusal> {
        let row = self.finding(call.project, call.number).ok_or(FindingRefusal::NoFinding)?;
        if row.state.ended() { return Err(FindingRefusal::Ended); }
        self.finding_area(call.project, &call.area)?;
        if !crate::settings::words_fit(&call.why, 200) { return Err(FindingRefusal::Words(b"why".as_slice().into())); }
        let mut row = row.clone();
        row.area = Some(call.area); row.why = Some(call.why); row.hander = Some(filer); row.state = FindingState::New;
        Ok(FindingWrite { row, erase: None })
    }
    pub fn prepare_triage_finding(&self, call: TriageFinding) -> Result<FindingWrite, FindingRefusal> {
        let mut row = self.finding(call.project, call.number).ok_or(FindingRefusal::NoFinding)?.clone();
        if row.state.ended() { return Err(FindingRefusal::Ended); }
        match call.act {
            Triage::Route(goal) => { row.state = FindingState::Routed; row.goal = Some(goal); }
            Triage::Backlog => row.state = FindingState::Backlog,
            Triage::Drop(reason) => {
                if !crate::settings::words_fit(&reason, 400) { return Err(FindingRefusal::Words(b"reason".as_slice().into())); }
                row.state = FindingState::Dropped; row.reason = Some(reason);
            }
        }
        Ok(FindingWrite { row, erase: None })
    }
    pub fn commit_finding(&mut self, write: FindingWrite, out: &mut Queue<Request>) {
        let FindingWrite { row, erase } = write;
        if let Some(number) = erase { self.findings.remove(&(row.project, number)); out.push(Request::Erase { key: Key::Finding { project: row.project, number } }); }
        let project = row.project;
        self.findings.insert((project, row.number), row.clone()).expect("judged finding room");
        out.push(Request::Save { record: Stored::Finding(row), release: None, read_afresh: None });
        mark_findings(project, out);
    }
    pub fn answer_finding(&mut self, request: skein_lib::Token, token: skein_lib::Token, answer: crate::Answered, out: &mut Queue<Request>) { self.connect.answer(request, token, answer, out); }
}
#[must_use]
#[expect(clippy::disallowed_methods, reason = "the forge judges UTF-8 at its finding boundary")]
pub fn body_fits(body: &[u8]) -> bool { body.len() <= 4096 && core::str::from_utf8(body).is_ok() }
pub(crate) fn erase_project(d: &mut Domain, project: u32, out: &mut Queue<Request>) {
    let mut numbers = List::with_capacity(d.findings.len());
    for row in d.findings(project) { numbers.push(row.number).expect("bounded findings"); }
    for number in numbers.into_boxed() { d.findings.remove(&(project, number)); out.push(Request::Erase { key: Key::Finding { project, number } }); }
    mark_findings(project, out);
}
