//! Findings (temper/docs/design/domain/findings.md, 2, 6, 9; domain/areas.md, 3 to 7).
use crate::{
    Domain, Key, Limits, Request, Stored,
    settings::{AreaOwner, ProjectAreas},
};
use alloc::boxed::Box;
use skein_lib::{List, Map, Queue, Wall};
pub use temper_engine_domain_forge_vocabulary::findings::*;
pub(crate) mod judging;
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
    /// One open flake per test suffices; count each chosen finding once (landing-regate.md, 2).
    pub(crate) fn gate_flakes(
        &self,
        project: u32,
        repository: skein_forge::api::Repository,
        gate: &[u8],
        tests: &[crate::ReceiptTest],
    ) -> Option<Box<[u32]>> {
        if tests.is_empty() || tests.len() > 64 {
            return None;
        }
        let mut numbers = List::with_capacity(64);
        for test in tests {
            let mut matched = None;
            for (_, row) in self.findings(project) {
                if row.kind != FindingKind::Flake || row.state == FindingState::Dropped {
                    continue;
                }
                let Some(signature) = &row.signature else { continue };
                if signature.repository == repository
                    && (signature.source.as_ref() == gate || signature.source.as_ref() == b"check")
                    && signature.ids.contains(&test.key)
                {
                    matched = Some(row.number);
                    break;
                }
            }
            let number = matched?;
            if !numbers.as_slice().contains(&number) {
                numbers.push(number).expect("one finding per test");
            }
        }
        Some(numbers.into_boxed())
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
            if row.erasable() {
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
                blame: None,
                repair: None,
                cleared: None,
            },
            erase,
        })
    }
    /// Free slots followed by erasable rows, matching `prepare_file_finding`'s admission.
    pub(crate) fn verdict_room(&self, limits: &Limits, project: u32) -> u32 {
        let mut count = 0_u32;
        let mut erasable = 0_u32;
        for (_, finding) in self.findings(project) {
            count = count.saturating_add(1);
            erasable = erasable.saturating_add(u32::from(finding.erasable()));
        }
        let free = limits.findings_per_project.saturating_sub(count);
        let global = limits.findings().saturating_sub(self.finding_rows().len());
        // Replacement is admitted only once the project's own bound is reached.
        if global < free { global } else { free.saturating_add(erasable) }
    }
    /// A landing tier supplies its signature and blame (domain/red-findings.md, 9).
    pub fn prepare_verdict_filing(
        &self,
        limits: &Limits,
        wall: Wall,
        filer: Filer,
        filing: &VerdictFiling,
    ) -> Result<FindingWrite, FindingRefusal> {
        if !filing.signature.fits() {
            return Err(FindingRefusal::Words(b"signature".as_slice().into()));
        }
        let mut write = self.prepare_file_finding(
            limits,
            wall,
            filer,
            FileFinding {
                project: filing.project,
                kind: filing.kind,
                severity: filing.severity,
                area: None,
                title: filing.title.clone(),
                body: filing.body.clone(),
            },
        )?;
        write.row.signature = Some(filing.signature.clone());
        write.row.blame.clone_from(&filing.blame);
        if filing.kind == FindingKind::Flake {
            write.row.seen = Some(1);
        }
        Ok(write)
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
    pub fn prepare_triage_finding(&self, by: Filer, call: TriageFinding) -> Result<FindingWrite, FindingRefusal> {
        let mut row = self.finding(call.project, call.number).ok_or(FindingRefusal::NoFinding)?.clone();
        match &call.act {
            Triage::Route(_) | Triage::Backlog | Triage::Drop(_) => {
                if row.state.ended() {
                    return Err(FindingRefusal::Ended);
                }
            }
            Triage::Repair(_) | Triage::Blame(_) => {}
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
            Triage::Repair(_) => {
                return Err(FindingRefusal::Words(crate::tiers::joined(&[
                    b"repairs are off in project ",
                    skein_lib::Decimal::of(u64::from(call.project)).as_bytes(),
                ])));
            }
            Triage::Blame(act) => row.blame = Some(self.moved_blame(&row, by, act)?),
        }
        Ok(FindingWrite { row, erase: None })
    }
    /// A move reads only today's tier settings and the retained log (domain/red-repairs.md, 3).
    fn moved_blame(&self, row: &Finding, by: Filer, act: BlameAct) -> Result<FindingBlame, FindingRefusal> {
        let address = crate::tiers::joined(&[
            b"finding ",
            skein_lib::Decimal::of(u64::from(row.project)).as_bytes(),
            b"/",
            skein_lib::Decimal::of(u64::from(row.number)).as_bytes(),
        ]);
        if row.kind != FindingKind::Red || row.blame.is_none() {
            return Err(FindingRefusal::Words(crate::tiers::joined(&[
                &address,
                b" has no blame: it is no red filed under landed",
            ])));
        }
        if row.cleared.is_some() {
            return Err(FindingRefusal::Words(crate::tiers::joined(&[&address, b" is cleared"])));
        }
        if !crate::settings::words_fit(&act.why, 200) {
            return Err(FindingRefusal::Words(b"why".as_slice().into()));
        }
        let landings: Box<[BlamedLanding]> = match act.landing {
            None => Box::default(),
            Some(commit) => {
                let signature = row.signature.as_ref().expect("landed finding signature");
                let mut branch = None;
                for tier in crate::tiers::configured_all(self, signature.repository) {
                    if tier.name == signature.source {
                        branch = crate::tiers::followed(self, signature.repository, tier);
                        break;
                    }
                }
                let Some(branch) = branch else {
                    return Err(FindingRefusal::Words(crate::tiers::joined(&[
                        &address,
                        b"'s tier ",
                        &signature.source,
                        b" has no followed branch",
                    ])));
                };
                let mut found = None;
                if let Some(log) = self.landing_log(signature.repository) {
                    for landing in &log.landings {
                        if landing.commit == commit && landing.branch == branch {
                            found = Some(blamed_landing(commit, &landing.changes, self.landing.limits.landing_members));
                            break;
                        }
                    }
                }
                let Some(landing) = found else {
                    let digits = crate::commit_text(&commit);
                    return Err(FindingRefusal::Words(crate::tiers::joined(&[
                        b"landing ",
                        digits.get(..12).expect("commit digits"),
                        b" is not one the landing log holds on ",
                        &branch,
                    ])));
                };
                Box::new([landing])
            }
        };
        Ok(FindingBlame { strength: BlameStrength::Moved, landings, moved: Some(BlameMove { by, why: act.why }) })
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
/// Filing and moves retain each goal once in member order (domain/red-findings.md, 3).
pub(crate) fn blamed_landing(
    commit: skein_forge::api::Commit,
    members: &[crate::LoggedChange],
    bound: u32,
) -> BlamedLanding {
    let mut goals = List::with_capacity(bound);
    for member in members {
        let Some(goal) = member.goal else { continue };
        let mut seen = false;
        for kept in &goals {
            let kept: &BlamedGoal = kept;
            seen |= kept.goal == goal;
        }
        if !seen && goals.room() > 0 {
            goals.push(BlamedGoal { goal, milestone: member.milestone, person: member.person }).expect("goal room");
        }
    }
    BlamedLanding { commit, goals: goals.into_boxed() }
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
