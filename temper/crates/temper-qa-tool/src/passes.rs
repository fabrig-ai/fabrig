//! Pass records and starts/resumes (qa/ledger.md, 2.3, 2.5, 2.7 and 4.8–4.16).
use crate::ledger::next_number;
use crate::{Build, Case, CaseInput, Ledger, Pages, Refusal, Session, refused};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum CaseState {
    Queued,
    Assigned,
    Passed,
    Failed,
    Blocked,
}
impl CaseState {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Queued => "queued",
            Self::Assigned => "assigned",
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Blocked => "blocked",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PassState {
    Running,
    Waiting,
    Ended,
}
impl PassState {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Waiting => "waiting",
            Self::Ended => "ended",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SessionKind {
    Triage,
    Pass,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum OutcomeResult {
    Passed,
    Failed,
    Blocked,
}
impl OutcomeResult {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Passed => "passed",
            Self::Failed => "failed",
            Self::Blocked => "blocked",
        }
    }
    pub(crate) fn parse(word: &str) -> Result<Self, Refusal> {
        match word {
            "passed" => Ok(Self::Passed),
            "failed" => Ok(Self::Failed),
            "blocked" => Ok(Self::Blocked),
            other => Err(refused(format!("outcome refused: {other} is none of passed, failed, blocked"))),
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Outcome {
    pub session: u64,
    pub time: u64,
    pub result: OutcomeResult,
    pub notes: String,
    pub findings: Vec<u64>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Pass {
    pub number: u64,
    pub place: String,
    pub build: String,
    pub state: PassState,
    pub started: u64,
    pub ended: Option<u64>,
    pub cases: Vec<u64>,
    pub sessions: Vec<u64>,
    pub requests: Vec<Request>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Request {
    pub number: u64,
    pub time: u64,
    pub findings: Vec<u64>,
    pub recipes: Vec<String>,
    pub filed: Option<u64>,
    pub goal: Option<u64>,
    pub ended: Option<u64>,
}
/// Restart-safe undo data, including a start's cases but no prior pass (ledger.md, 2.5).
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Before {
    pub state: Option<PassState>,
    pub place: Option<String>,
    pub build: Option<String>,
    pub cases: Vec<CaseBefore>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct CaseBefore {
    pub number: u64,
    pub state: CaseState,
    pub pass: Option<u64>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct PassStart {
    pub pass: u64,
    pub build: String,
    pub session: u64,
}

impl Ledger {
    fn takeable_build(&self, surfaces: &[String]) -> Option<&crate::Build> {
        self.builds.iter().filter(|build| {
            self.reached(build) && build.held.is_none() && build.order.is_some()
                && self.cases.iter().any(|case| eligible(case, build, surfaces))
        }).max_by_key(|build| build.order)
    }

    /// Called after the loop's resumes and starts (qa/ledger.md, 4.17).
    pub fn update_place_wait(&mut self, surfaces: &[String], free: bool, now: u64) {
        let pending = self.takeable_build(surfaces).is_some();
        if let Some(wait) = self.place_waits.last_mut().filter(|wait| wait.to.is_none()) {
            if !pending {
                wait.to = Some(now);
            }
        } else if pending && !free {
            self.place_waits.push(crate::PlaceWait { from: now, to: None });
        }
    }

    pub fn start_pass(&mut self, place: &str, surfaces: &[String], effort: &str, now: u64) -> Option<PassStart> {
        if self.passes.iter().any(|pass| pass.state == PassState::Running && pass.place == place) {
            return None;
        }
        let build = self.takeable_build(surfaces)?.commit.clone();
        let number = next_number(self.passes.iter().map(|pass| pass.number));
        self.passes.push(Pass {
            number,
            place: place.into(),
            build: build.clone(),
            state: PassState::Running,
            started: now,
            ended: None,
            cases: Vec::new(),
            sessions: Vec::new(),
            requests: Vec::new(),
        });
        let before = Before { state: None, place: None, build: None, cases: Vec::new() };
        Some(self.take_pass(number, &build, place, surfaces, effort, now, before))
    }

    /// Oldest eligible waiting pass first; never steals another pass's held build.
    pub fn resume_pass(&mut self, place: &str, surfaces: &[String], effort: &str, now: u64) -> Option<PassStart> {
        if self.passes.iter().any(|pass| pass.state == PassState::Running && pass.place == place) {
            return None;
        }
        let (number, build) = self.passes.iter().filter(|pass| pass.state == PassState::Waiting).find_map(|pass| {
            let request = pass.requests.last().filter(|request| request.filed.is_some())?;
            let goal = request.goal?;
            let landing = self.landings.iter().find(|landing| landing.goal == goal);
            if landing.is_none() && request.ended.is_none() {
                return None;
            }
            let build = self
                .builds
                .iter()
                .filter(|build| {
                    self.reached(build)
                        && build.order.is_some()
                        && (build.held.is_none() || build.held == Some(pass.number))
                        && landing.is_none_or(|landing| {
                            build.holds.as_ref().is_some_and(|holds| holds.contains(&landing.number))
                        })
                })
                .max_by_key(|build| build.order)?;
            Some((pass.number, build.commit.clone()))
        })?;
        let pass = self.passes.iter().find(|pass| pass.number == number).expect("waiting pass");
        let before = Before {
            state: Some(pass.state),
            place: Some(pass.place.clone()),
            build: Some(pass.build.clone()),
            cases: Vec::new(),
        };
        Some(self.take_pass(number, &build, place, surfaces, effort, now, before))
    }

    #[expect(clippy::too_many_arguments, reason = "one atomic start/resume with its supplied time and rollback data")]
    fn take_pass(
        &mut self,
        number: u64,
        build: &str,
        place: &str,
        surfaces: &[String],
        effort: &str,
        now: u64,
        mut before: Before,
    ) -> PassStart {
        let selected = self.builds.iter().find(|record| record.commit == build).expect("selected build");
        for case in &self.cases {
            if (case.pass == Some(number) && (case.state == CaseState::Failed || case.state == CaseState::Assigned))
                || eligible(case, selected, surfaces)
            {
                before.cases.push(CaseBefore { number: case.number, state: case.state, pass: case.pass });
            }
        }
        let session = next_number(self.sessions.iter().map(|session| session.number));
        let pass = self.passes.iter_mut().find(|pass| pass.number == number).expect("selected pass");
        pass.state = PassState::Running;
        pass.place = place.into();
        pass.build = build.into();
        pass.sessions.push(session);
        for saved in &before.cases {
            let case = self.cases.iter_mut().find(|case| case.number == saved.number).expect("taken case");
            case.state = CaseState::Assigned;
            case.pass = Some(number);
            if !pass.cases.contains(&case.number) {
                pass.cases.push(case.number);
            }
        }
        for record in &mut self.builds {
            if record.held == Some(number) {
                record.held = None;
            }
            if record.commit == build {
                record.held = Some(number);
            }
        }
        self.sessions.push(Session {
            number: session,
            kind: SessionKind::Pass,
            landing: None,
            pass: Some(number),
            before: Some(before),
            effort: effort.into(),
            started: now,
            ended: None,
            exit: None,
            tokens: None,
            pages: Pages::new(),
            setups: Vec::new(),
        });
        PassStart { pass: number, build: build.into(), session }
    }

    pub fn pass_command_session(&self, number: u64) -> Result<&Session, Refusal> {
        let session = self
            .sessions
            .iter()
            .find(|session| session.number == number && session.ended.is_none())
            .ok_or_else(|| refused(format!("session {number} is not running")))?;
        if session.kind != SessionKind::Pass {
            return Err(refused(format!("session {number} is a triage session")));
        }
        let pass = self.passes.iter().find(|pass| Some(pass.number) == session.pass).expect("session's pass");
        if pass.state != PassState::Running {
            return Err(refused(format!("pass {} is {}", pass.number, pass.state.name())));
        }
        Ok(session)
    }

    pub fn pass_command_case(&self, session: u64, number: u64) -> Result<&Case, Refusal> {
        let pass = self.pass_command_session(session)?.pass.expect("pass session");
        self.cases
            .iter()
            .find(|case| case.number == number && case.pass == Some(pass) && case.state == CaseState::Assigned)
            .ok_or_else(|| refused(format!("case {number} is not assigned to pass {pass}")))
    }

    pub(crate) fn waiting_match(&self, input: &CaseInput) -> Option<(u64, u64)> {
        self.passes.iter().filter(|pass| pass.state == PassState::Waiting).find_map(|pass| {
            self.findings
                .iter()
                .find(|finding| {
                    finding.pass == pass.number
                        && finding.state == crate::FindingState::Open
                        && finding.surface == input.surface
                        && finding.page == input.page
                        && finding.setup == input.setup
                })
                .map(|finding| (pass.number, finding.number))
        })
    }

    pub fn file_request(&mut self, pass: u64, request: u64, goal: Option<u64>, now: u64) -> Result<(), Refusal> {
        let request = self
            .passes
            .iter_mut()
            .find(|record| record.number == pass)
            .and_then(|pass| pass.requests.iter_mut().find(|record| record.number == request))
            .ok_or_else(|| refused(format!("no request {request} of pass {pass}")))?;
        if request.filed.is_some() {
            return Err(refused(format!("request {} of pass {pass} is filed", request.number)));
        }
        if !request.findings.is_empty() && goal.is_none() {
            return Err(refused("a request with findings needs a goal"));
        }
        if request.findings.is_empty() && goal.is_some() {
            return Err(refused("a request without findings has no goal"));
        }
        request.filed = Some(now);
        request.goal = goal;
        for number in &request.findings {
            let finding = self.findings.iter_mut().find(|finding| finding.number == *number).expect("request finding");
            if let Some(goal) = goal {
                finding.goals.push(goal);
            }
        }
        Ok(())
    }

    pub fn end_goal(&mut self, goal: u64, now: u64) -> Result<(), Refusal> {
        let mut found = false;
        for request in self.passes.iter_mut().flat_map(|pass| &mut pass.requests) {
            if request.goal == Some(goal) && request.filed.is_some() {
                request.ended = Some(now);
                found = true;
            }
        }
        if found { Ok(()) } else { Err(refused(format!("no filed request for goal {goal}"))) }
    }
}
fn eligible(case: &Case, build: &Build, surfaces: &[String]) -> bool {
    case.state == CaseState::Queued
        && surfaces.contains(&case.surface)
        && case.landings().iter().all(|number| build.holds.as_ref().is_some_and(|holds| holds.contains(number)))
}
