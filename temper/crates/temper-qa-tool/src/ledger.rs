//! Value operations, with time supplied by the caller (qa/ledger.md, 2 and 4).
use crate::{CaseInput, Refusal, refused};
use std::collections::BTreeMap;

pub type Pages = BTreeMap<String, Vec<String>>;

#[derive(Clone, Debug, Default, PartialEq, Eq)]
pub struct Ledger {
    pub head: Option<String>,
    pub landings: Vec<Landing>,
    pub cases: Vec<Case>,
    pub builds: Vec<Build>,
    pub sessions: Vec<Session>,
    pub lines: Vec<Line>,
    pub passes: Vec<crate::Pass>,
    pub findings: Vec<crate::Finding>,
    pub notes: Vec<crate::Note>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Landing {
    pub number: u64,
    pub commit: String,
    pub goal: u64,
    pub pull: Option<u64>,
    pub title: String,
    pub seen: u64,
    pub answer: Option<Answer>,
    pub triaged: Option<u64>,
    pub unanswered: bool,
    pub sessions: u64,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Answer {
    None { why: String },
    Cases,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Case {
    pub number: u64,
    pub surface: String,
    pub page: String,
    pub asks: Vec<Ask>,
    pub state: crate::CaseState,
    pub pass: Option<u64>,
    pub blocks: u64,
    pub outcomes: Vec<crate::Outcome>,
    pub queued: u64,
    pub line: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Ask {
    pub landing: u64,
    pub setup: String,
    pub adds: String,
    pub steps: Vec<crate::Step>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Build {
    pub commit: String,
    pub kept: u64,
    pub state: BuildState,
    pub deleted: Option<u64>,
    pub holds: Option<Vec<u64>>,
    pub order: Option<u64>,
    pub held: Option<u64>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BuildState {
    Kept,
    Deleted,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Session {
    pub number: u64,
    pub kind: crate::SessionKind,
    pub landing: Option<u64>,
    pub pass: Option<u64>,
    pub before: Option<crate::Before>,
    pub effort: String,
    pub started: u64,
    pub ended: Option<u64>,
    pub exit: Option<Exit>,
    pub tokens: Option<u64>,
    pub pages: Pages,
    pub setups: Vec<String>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Exit {
    Code(i32),
    Signal(i32),
    Bound,
    Lost,
    Unstarted,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Line {
    pub number: u64,
    pub time: u64,
    pub area: String,
    pub words: String,
    pub given: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum CaseAnswer {
    Queued(u64),
    Joined { number: u64, landings: Vec<u64> },
    Waiting { number: u64, pass: u64, finding: u64 },
}
impl std::fmt::Display for CaseAnswer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Queued(number) => write!(f, "case {number} queued"),
            Self::Waiting { number, pass, finding } => {
                write!(f, "case {number} joined pass {pass}, waiting on finding {finding}")
            }
            Self::Joined { number, landings } => write!(f, "case {number} joined: landings {}", numbers(landings)),
        }
    }
}
impl Case {
    #[must_use]
    pub fn landings(&self) -> Vec<u64> {
        let mut numbers = Vec::new();
        for ask in &self.asks {
            if !numbers.contains(&ask.landing) {
                numbers.push(ask.landing);
            }
        }
        numbers
    }
}

impl Ledger {
    pub fn record_head(&mut self, commit: &str) {
        self.head = Some(commit.into());
    }

    /// A duplicate commit records nothing and returns no new number.
    pub fn record_landing(&mut self, commit: &str, goal: u64, pull: Option<u64>, title: &str, now: u64) -> Option<u64> {
        if self.landings.iter().any(|landing| landing.commit == commit) {
            return None;
        }
        let number = next_number(self.landings.iter().map(|landing| landing.number));
        self.landings.push(Landing {
            number,
            commit: commit.into(),
            goal,
            pull,
            title: title.into(),
            seen: now,
            answer: None,
            triaged: None,
            unanswered: false,
            sessions: 0,
        });
        Some(number)
    }

    #[must_use]
    pub fn next_landing(&self) -> Option<&Landing> {
        self.landings.iter().find(|landing| landing.triaged.is_none())
    }

    pub fn start_session(
        &mut self,
        landing: u64,
        effort: &str,
        pages: Pages,
        setups: Vec<String>,
        now: u64,
    ) -> Result<u64, Refusal> {
        if let Some(session) =
            self.running_sessions().into_iter().find(|session| session.kind == crate::SessionKind::Triage)
        {
            return Err(refused(format!("session {} is running", session.number)));
        }
        if self.next_landing().map(|next| next.number) != Some(landing) {
            return Err(refused(format!("landing {landing} is not next")));
        }
        let number = next_number(self.sessions.iter().map(|session| session.number));
        self.sessions.push(Session {
            number,
            kind: crate::SessionKind::Triage,
            landing: Some(landing),
            pass: None,
            before: None,
            effort: effort.into(),
            started: now,
            ended: None,
            exit: None,
            tokens: None,
            pages,
            setups,
        });
        Ok(number)
    }

    /// Check both the live session and the landing order before any command records.
    pub fn command_session(&self, number: u64) -> Result<&Session, Refusal> {
        let session = self
            .sessions
            .iter()
            .find(|session| session.number == number && session.ended.is_none())
            .ok_or_else(|| refused(format!("session {number} is not running")))?;
        if session.kind != crate::SessionKind::Triage {
            return Err(refused(format!("session {number} is a pass session")));
        }
        let landing = session.landing.expect("triage landing");
        match self.next_landing() {
            Some(next) if next.number == landing => Ok(session),
            Some(next) => Err(refused(format!(
                "session {number} triages landing {}, which is not next: landing {} is not triaged",
                landing, next.number
            ))),
            None => Err(refused(format!("session {number} is not running"))),
        }
    }

    pub fn answer_none(&mut self, session: u64, why: &str) -> Result<u64, Refusal> {
        let number = self.command_session(session)?.landing.expect("triage landing");
        if why.trim().is_empty() {
            return Err(refused("no QA needs a line saying why"));
        }
        let landing = self.landings.iter_mut().find(|landing| landing.number == number).expect("session's landing");
        if landing.answer == Some(Answer::Cases) {
            return Err(refused(format!("landing {number} has cases; no QA is refused")));
        }
        landing.answer = Some(Answer::None { why: why.into() });
        Ok(number)
    }

    pub fn add_case(&mut self, session: u64, input: &CaseInput, now: u64) -> Result<CaseAnswer, Refusal> {
        let session = self.command_session(session)?;
        let number = session.landing.expect("triage landing");
        if let Some(Answer::None { .. }) =
            self.landings.iter().find(|landing| landing.number == number).and_then(|landing| landing.answer.as_ref())
        {
            return Err(refused(format!("landing {number} is answered no QA; a case is refused")));
        }
        input.check(session)?;
        let ask =
            Ask { landing: number, setup: input.setup.clone(), adds: input.adds.clone(), steps: input.steps.clone() };
        let waiting = self.waiting_match(input);
        let answer = if let Some((pass, finding)) = waiting {
            let number = next_number(self.cases.iter().map(|case| case.number));
            self.cases.push(Case {
                number,
                surface: input.surface.clone(),
                page: input.page.clone(),
                asks: vec![ask],
                state: crate::CaseState::Assigned,
                pass: Some(pass),
                blocks: 0,
                outcomes: Vec::new(),
                queued: now,
                line: false,
            });
            self.passes.iter_mut().find(|record| record.number == pass).expect("waiting pass").cases.push(number);
            CaseAnswer::Waiting { number, pass, finding }
        } else if let Some(case) = self.cases.iter_mut().find(|case| {
            case.state == crate::CaseState::Queued && case.surface == input.surface && case.page == input.page
        }) {
            case.asks.push(ask);
            CaseAnswer::Joined { number: case.number, landings: case.landings() }
        } else {
            let number = next_number(self.cases.iter().map(|case| case.number));
            self.cases.push(Case {
                number,
                surface: input.surface.clone(),
                page: input.page.clone(),
                asks: vec![ask],
                state: crate::CaseState::Queued,
                pass: None,
                blocks: 0,
                outcomes: Vec::new(),
                queued: now,
                line: false,
            });
            CaseAnswer::Queued(number)
        };
        self.landings.iter_mut().find(|landing| landing.number == number).expect("session's landing").answer =
            Some(Answer::Cases);
        Ok(answer)
    }

    pub fn end_session(
        &mut self,
        number: u64,
        exit: Exit,
        tokens: Option<u64>,
        now: u64,
        area: &str,
    ) -> Result<(), Refusal> {
        let session = self
            .sessions
            .iter()
            .find(|session| session.number == number && session.ended.is_none())
            .ok_or_else(|| refused(format!("session {number} is not running")))?;
        if session.kind == crate::SessionKind::Pass {
            return self.end_pass_session(number, exit, tokens, now, area, &[]);
        }
        let landing_number = session.landing.expect("triage landing");
        let landing = self
            .landings
            .iter_mut()
            .find(|landing| landing.number == landing_number)
            .ok_or_else(|| refused(format!("session {number}: no landing {landing_number}")))?;
        let mut words = None;
        if exit != Exit::Unstarted {
            if landing.answer.is_some() {
                landing.triaged = Some(now);
            } else {
                landing.sessions = landing.sessions.saturating_add(1);
                if landing.sessions == 2 {
                    landing.triaged = Some(now);
                    landing.unanswered = true;
                    words = Some(format!(
                        "landing {} (goal {}, {}): triage recorded nothing in two sessions; triaged as unanswered",
                        landing.number,
                        landing.goal,
                        short(&landing.commit)
                    ));
                }
            }
        }
        let session = self.sessions.iter_mut().find(|session| session.number == number).expect("checked session");
        session.ended = Some(now);
        session.exit = Some(exit);
        session.tokens = tokens;
        if let Some(words) = words {
            self.add_line(area, &words, now);
        }
        Ok(())
    }

    /// Build records are unique by commit, including after deletion.
    pub fn keep_build(&mut self, commit: &str, now: u64) -> Result<(), Refusal> {
        if self.builds.iter().any(|build| build.commit == commit) {
            return Err(refused(format!("build {commit} is already recorded")));
        }
        self.builds.push(Build {
            commit: commit.into(),
            kept: now,
            state: BuildState::Kept,
            deleted: None,
            holds: None,
            order: None,
            held: None,
        });
        Ok(())
    }

    /// Record a published build anew, moving a deleted record to the kept order's end
    /// (qa/builds.md, 3).
    pub fn record_build_kept(&mut self, commit: &str, now: u64) -> Result<(), Refusal> {
        self.check_build_keep(commit)?;
        self.builds.retain(|build| build.commit != commit);
        self.keep_build(commit, now)
    }

    pub(crate) fn check_build_keep(&self, commit: &str) -> Result<(), Refusal> {
        if self.builds.iter().any(|build| build.commit == commit && build.state == BuildState::Kept) {
            return Err(refused(format!("build {} is kept already", short(commit))));
        }
        Ok(())
    }

    pub fn set_build_holds(&mut self, commit: &str, holds: Vec<u64>, order: u64) -> Result<(), Refusal> {
        if holds.iter().any(|number| !self.landings.iter().any(|landing| landing.number == *number)) {
            return Err(refused(format!("build {commit}: holds an unknown landing")));
        }
        let build = self
            .builds
            .iter_mut()
            .find(|build| build.commit == commit && build.state == BuildState::Kept)
            .ok_or_else(|| refused(format!("build {commit} is not kept")))?;
        build.holds = Some(holds);
        build.order = Some(order);
        Ok(())
    }

    #[must_use]
    pub fn builds_without_holds(&self) -> Vec<&Build> {
        self.builds.iter().filter(|build| build.state == BuildState::Kept && build.holds.is_none()).collect()
    }

    #[must_use]
    pub fn reached(&self, build: &Build) -> bool {
        build.state == BuildState::Kept
            && build.holds.as_ref().is_some_and(|holds| {
                holds.iter().all(|number| {
                    self.landings.iter().any(|landing| landing.number == *number && landing.triaged.is_some())
                })
            })
    }

    #[must_use]
    pub fn deletable_builds(&self) -> Vec<&Build> {
        self.builds
            .iter()
            .filter(|build| {
                build.held.is_none()
                    && self.reached(build)
                    && (self.builds.iter().any(|newer| {
                        self.reached(newer) && newer.order.zip(build.order).is_some_and(|(newer, older)| newer > older)
                    }) || !self.cases.iter().any(|case| {
                        case.state == crate::CaseState::Queued
                            && case
                                .landings()
                                .iter()
                                .any(|number| build.holds.as_ref().is_some_and(|holds| holds.contains(number)))
                    }))
            })
            .collect()
    }

    pub fn mark_build_deleted(&mut self, commit: &str, now: u64) -> Result<(), Refusal> {
        if !self.deletable_builds().iter().any(|build| build.commit == commit) {
            return Err(refused(format!("build {commit} is not deletable")));
        }
        let build = self.builds.iter_mut().find(|build| build.commit == commit).expect("deletable build");
        build.state = BuildState::Deleted;
        build.deleted = Some(now);
        Ok(())
    }

    /// Record each waiting case's line once; returns the new line numbers.
    pub fn waiting_lines(&mut self, bound: u64, now: u64, area: &str) -> Vec<u64> {
        let waiting: Vec<_> = self.cases.iter().filter(|case| case.state == crate::CaseState::Queued && !case.line && now.saturating_sub(case.queued) >= bound &&
            !self.builds.iter().any(|build| self.reached(build) && case.landings().iter().all(|number| build.holds.as_ref().is_some_and(|holds| holds.contains(number)))))
            .map(|case| (case.number, format!("case {} ({} on {}; landings {}) has waited past {bound} seconds for a kept build holding its landings", case.number, case.page, case.surface, numbers(&case.landings())))).collect();
        let mut lines = Vec::new();
        for (number, words) in waiting {
            self.cases.iter_mut().find(|case| case.number == number).expect("waiting case").line = true;
            lines.push(self.add_line(area, &words, now));
        }
        lines
    }

    pub fn add_line(&mut self, area: &str, words: &str, now: u64) -> u64 {
        let number = next_number(self.lines.iter().map(|line| line.number));
        self.lines.push(Line { number, time: now, area: area.into(), words: words.into(), given: false });
        number
    }
    #[must_use]
    pub fn ungiven_lines(&self) -> Vec<&Line> {
        self.lines.iter().filter(|line| !line.given).collect()
    }
    pub fn mark_line_given(&mut self, number: u64) -> Result<(), Refusal> {
        let line = self
            .lines
            .iter_mut()
            .find(|line| line.number == number)
            .ok_or_else(|| refused(format!("no line {number}")))?;
        line.given = true;
        Ok(())
    }
    #[must_use]
    pub fn running_sessions(&self) -> Vec<&Session> {
        self.sessions.iter().filter(|session| session.ended.is_none()).collect()
    }
}

pub(crate) fn next_number(numbers: impl Iterator<Item = u64>) -> u64 {
    numbers.max().unwrap_or(0).checked_add(1).expect("ledger numbers fit u64")
}
pub(crate) fn numbers(values: &[u64]) -> String {
    values.iter().map(u64::to_string).collect::<Vec<_>>().join(", ")
}
pub(crate) fn short(commit: &str) -> &str {
    commit.get(..12).unwrap_or(commit)
}
