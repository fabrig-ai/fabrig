//! Findings (temper/docs/design/domain/findings.md, 2, 6, 9; domain/areas.md, 3 to 7).
use crate::{
    Domain, Key, Limits, Request, Stored,
    settings::{AreaOwner, ProjectAreas},
};
use alloc::boxed::Box;
use skein_lib::{List, Map, Queue, Wall};
pub use temper_engine_domain_forge_vocabulary::findings::*;
static NO_FINDINGS: Map<(u32, u32), Finding> = Map::with_capacity(0);
impl Domain {
    pub(crate) fn finding_rows(&self) -> &Map<(u32, u32), Finding> {
        match &self.findings {
            Some(rows) => rows,
            None => &NO_FINDINGS,
        }
    }
    pub(crate) fn finding_rows_mut(&mut self, limits: &Limits) -> &mut Map<(u32, u32), Finding> {
        if self.findings.is_none() {
            self.findings = Some(Box::new(Map::with_capacity(limits.findings())));
        }
        self.findings.as_deref_mut().expect("finding table opened")
    }
    #[must_use]
    pub fn project_areas(&self, project: u32) -> ProjectAreas {
        if let Some(crate::settings::SizedValue { value: crate::settings::Value::ProjectAreas(areas), .. }) =
            self.setting(14, project, 1)
        {
            return areas.clone();
        }
        ProjectAreas { areas: Box::default() }
    }
    #[must_use]
    pub fn project_triager(&self, project: u32) -> Option<AreaOwner> {
        if let Some(crate::settings::SizedValue { value: crate::settings::Value::ProjectTriager(triager), .. }) =
            self.setting(15, project, 1)
        {
            return Some(triager.owner);
        }
        None
    }
    #[must_use]
    pub fn finding(&self, project: u32, number: u32) -> Option<&Finding> {
        self.finding_rows().get(&(project, number))
    }
    pub fn findings(&self, project: u32) -> impl DoubleEndedIterator<Item = (&(u32, u32), &Finding)> {
        self.finding_rows().range((project, 0)..=(project, u32::MAX))
    }
    fn finding_area(&self, project: u32, name: &[u8]) -> Result<(), FindingRefusal> {
        let areas = self.project_areas(project);
        let mut names = List::with_capacity(16);
        for area in areas.areas {
            if area.name.as_ref() == name {
                return Ok(());
            }
            names.push(area.name).expect("bounded areas");
        }
        Err(FindingRefusal::NoArea(names.into_boxed()))
    }
    pub fn prepare_file_finding(
        &self,
        limits: &Limits,
        at: Wall,
        filer: Filer,
        call: FileFinding,
    ) -> Result<FindingWrite, FindingRefusal> {
        if !crate::settings::words_fit(&call.title, 200) {
            return Err(FindingRefusal::Words(b"title".as_slice().into()));
        }
        if !body_fits(&call.body) {
            return Err(FindingRefusal::Words(b"body".as_slice().into()));
        }
        if let Some(area) = &call.area {
            self.finding_area(call.project, area)?;
        }
        let mut highest = 0_u32;
        let mut count = 0_u32;
        let mut new = 0_u32;
        let mut oldest = None;
        for (_, row) in self.findings(call.project) {
            highest = row.number;
            count = count.saturating_add(1);
            if row.state == FindingState::New {
                new = new.saturating_add(1);
            }
            if row.state.ended() {
                match oldest {
                    Some((at, _)) if at <= row.at => {}
                    Some(_) | None => oldest = Some((row.at, row.number)),
                }
            }
        }
        let erase = if count >= limits.findings_per_project {
            let Some((_, number)) = oldest else {
                return Err(FindingRefusal::Full(new));
            };
            Some(number)
        } else {
            None
        };
        if erase.is_none() && self.finding_rows().len() >= limits.findings() {
            return Err(FindingRefusal::Full(new));
        }
        let Some(number) = highest.checked_add(1) else {
            return Err(FindingRefusal::Words(b"finding number exhausted".as_slice().into()));
        };
        Ok(FindingWrite {
            row: Finding {
                project: call.project,
                number,
                kind: call.kind,
                severity: call.severity,
                area: call.area,
                state: FindingState::New,
                title: call.title,
                body: call.body,
                filer,
                at,
                why: None,
                hander: None,
                goal: None,
                reason: None,
                signature: None,
                seen: None,
            },
            erase,
        })
    }
    /// A delivery check counts only an equal, non-dropped flake of this project (findings.md, 4).
    #[expect(clippy::too_many_arguments, reason = "the check report supplies its filing identity and words")]
    pub fn prepare_check_flake(
        &self,
        limits: &Limits,
        at: Wall,
        filer: Filer,
        project: u32,
        signature: FindingSignature,
        title: Box<[u8]>,
        body: Box<[u8]>,
    ) -> Result<CheckFlake, FindingRefusal> {
        if !signature.fits() {
            return Err(FindingRefusal::Words(b"signature".as_slice().into()));
        }
        for (_, row) in self.findings(project) {
            if row.kind == FindingKind::Flake
                && row.state != FindingState::Dropped
                && row.signature.as_ref() == Some(&signature)
            {
                let mut row = row.clone();
                row.seen = Some(row.seen.unwrap_or(0).saturating_add(1));
                return Ok(CheckFlake::Counted(FindingWrite { row, erase: None }));
            }
        }
        let mut write = self.prepare_file_finding(
            limits,
            at,
            filer,
            FileFinding { project, kind: FindingKind::Flake, severity: Severity::Low, area: None, title, body },
        )?;
        write.row.signature = Some(signature);
        write.row.seen = Some(1);
        Ok(CheckFlake::Filed(write))
    }
    pub fn prepare_hand_finding(&self, filer: Filer, call: HandFinding) -> Result<FindingWrite, FindingRefusal> {
        let row = self.finding(call.project, call.number).ok_or(FindingRefusal::NoFinding)?;
        if row.state.ended() {
            return Err(FindingRefusal::Ended);
        }
        self.finding_area(call.project, &call.area)?;
        if !crate::settings::words_fit(&call.why, 200) {
            return Err(FindingRefusal::Words(b"why".as_slice().into()));
        }
        let mut row = row.clone();
        row.area = Some(call.area);
        row.why = Some(call.why);
        row.hander = Some(filer);
        row.state = FindingState::New;
        Ok(FindingWrite { row, erase: None })
    }
    pub fn prepare_triage_finding(&self, call: TriageFinding) -> Result<FindingWrite, FindingRefusal> {
        let mut row = self.finding(call.project, call.number).ok_or(FindingRefusal::NoFinding)?.clone();
        if row.state.ended() {
            return Err(FindingRefusal::Ended);
        }
        match call.act {
            Triage::Route(goal) => {
                row.state = FindingState::Routed;
                row.goal = Some(goal);
            }
            Triage::Backlog => row.state = FindingState::Backlog,
            Triage::Drop(reason) => {
                if !reason_fits(&reason) {
                    return Err(FindingRefusal::Words(b"reason".as_slice().into()));
                }
                row.state = FindingState::Dropped;
                row.reason = Some(reason);
            }
        }
        Ok(FindingWrite { row, erase: None })
    }
    pub fn commit_finding(&mut self, limits: &Limits, write: FindingWrite, out: &mut Queue<Request>) {
        let FindingWrite { row, erase } = write;
        if let Some(number) = erase {
            self.finding_rows_mut(limits).remove(&(row.project, number));
            out.push(Request::Erase { key: Key::Finding { project: row.project, number } });
        }
        let project = row.project;
        self.finding_rows_mut(limits).insert((project, row.number), row.clone()).expect("judged finding room");
        out.push(Request::Save { record: Stored::Finding(row), release: None, read_afresh: None });
        mark_findings(project, out);
    }
    pub fn answer_finding(
        &mut self,
        request: skein_lib::Token,
        token: skein_lib::Token,
        answer: crate::Answered,
        out: &mut Queue<Request>,
    ) {
        self.connect.answer(request, token, answer, out);
    }
}
pub(crate) fn erase_project(d: &mut Domain, project: u32, out: &mut Queue<Request>) {
    let mut numbers = List::with_capacity(d.finding_rows().len());
    for (_, row) in d.findings(project) {
        numbers.push(row.number).expect("bounded findings");
    }
    if numbers.is_empty() {
        return;
    }
    for number in numbers.into_boxed() {
        d.findings.as_mut().expect("erasing held findings").remove(&(project, number));
        out.push(Request::Erase { key: Key::Finding { project, number } });
    }
    if d.finding_rows().is_empty() {
        d.findings = None;
    }
    mark_findings(project, out);
}
