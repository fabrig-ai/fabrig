//! Gaps are filed for people and closed by kept landings (qa/gaps.md, 2–7; qa/ledger.md, 4.18).
use crate::ledger::next_number;
use crate::{Answer, Build, BuildState, Case, CaseState, Ledger, Outcome, OutcomeResult, Refusal, refused};

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Gap {
    pub number: u64,
    pub kind: String,
    pub why: String,
    pub owner: String,
    pub landing: u64,
    pub case: Option<u64>,
    pub session: u64,
    pub time: u64,
    pub state: GapState,
    pub filed: Option<u64>,
    pub closed: Option<GapClosed>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum GapState {
    Open,
    Filed,
    Closed,
}
impl GapState {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Filed => "filed",
            Self::Closed => "closed",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct GapClosed {
    pub landing: u64,
    pub time: u64,
}
fn check(kind: &str, why: &str) -> Result<(), Refusal> {
    if !["ctl", "fake", "qa"].contains(&kind) {
        return Err(refused(format!("gap refused: {kind} is none of ctl, fake, qa")));
    }
    if why.trim().is_empty() {
        return Err(refused("gap refused: no why"));
    }
    Ok(())
}
impl Ledger {
    pub fn triage_gap(&mut self, session: u64, kind: &str, why: &str, owner: &str, now: u64) -> Result<u64, Refusal> {
        let number = self.command_session(session)?.landing.expect("triage landing");
        check(kind, why)?;
        let landing = self.landings.iter_mut().find(|landing| landing.number == number).expect("session landing");
        if let Some(Answer::None { .. }) = landing.answer {
            return Err(refused(format!("landing {number} is answered no QA; a gap is refused")));
        }
        if landing.answer != Some(Answer::Cases) {
            landing.answer = Some(Answer::Gap);
        }
        let number = next_number(self.gaps.iter().map(|gap| gap.number));
        self.gaps.push(Gap {
            number,
            kind: kind.into(),
            why: why.into(),
            owner: owner.into(),
            landing: landing.number,
            case: None,
            session,
            time: now,
            state: GapState::Open,
            filed: None,
            closed: None,
        });
        Ok(number)
    }

    pub fn pass_gap(
        &mut self,
        session: u64,
        case: u64,
        kind: &str,
        why: &str,
        owner: &str,
        now: u64,
    ) -> Result<u64, Refusal> {
        let landing = self.pass_command_case(session, case)?.asks.first().expect("case ask").landing;
        check(kind, why)?;
        let number = next_number(self.gaps.iter().map(|gap| gap.number));
        self.gaps.push(Gap {
            number,
            kind: kind.into(),
            why: why.into(),
            owner: owner.into(),
            landing,
            case: Some(case),
            session,
            time: now,
            state: GapState::Open,
            filed: None,
            closed: None,
        });
        let case = self.cases.iter_mut().find(|record| record.number == case).expect("assigned case");
        case.outcomes.push(Outcome {
            session,
            time: now,
            result: OutcomeResult::Gap,
            notes: why.into(),
            findings: Vec::new(),
        });
        case.state = CaseState::Gap;
        Ok(number)
    }

    pub(crate) fn close_gap(&mut self, number: u64, landing: u64, now: u64) {
        let Some(gap) = self.gaps.iter_mut().find(|gap| gap.number == number && gap.state != GapState::Closed) else {
            return;
        };
        gap.state = GapState::Closed;
        gap.closed = Some(GapClosed { landing, time: now });
        if let Some(number) = gap.case {
            let case = self.cases.iter_mut().find(|case| case.number == number).expect("gap case");
            case.state = CaseState::Queued;
            case.pass = None;
            case.outcomes.clear();
            case.queued = now;
            case.line = false;
        } else {
            let landing = self.landings.iter_mut().find(|landing| landing.number == gap.landing).expect("gap landing");
            landing.answer = None;
            landing.triaged = None;
            landing.sessions = 0;
            landing.unanswered = false;
        }
    }

    pub(crate) fn gap_build_holds(&self, case: &Case, build: &Build) -> bool {
        self.gaps.iter().filter(|gap| gap.case == Some(case.number)).all(|gap| {
            gap.closed
                .as_ref()
                .is_some_and(|closed| build.holds.as_ref().is_some_and(|holds| holds.contains(&closed.landing)))
        })
    }

    pub(crate) fn triage_build(&self, landing: u64) -> Option<&Build> {
        let gaps: Vec<_> = self
            .gaps
            .iter()
            .filter(|gap| gap.case.is_none() && gap.landing == landing && gap.closed.is_some())
            .collect();
        if gaps.is_empty() {
            return None;
        }
        self.builds
            .iter()
            .filter(|build| {
                build.state == BuildState::Kept
                    && gaps.iter().all(|gap| {
                        build
                            .holds
                            .as_ref()
                            .is_some_and(|holds| holds.contains(&gap.closed.as_ref().expect("closed gap").landing))
                    })
            })
            .max_by_key(|build| build.order)
    }

    #[must_use]
    pub fn gaps_text(&self, open: bool) -> String {
        use std::fmt::Write;
        let mut text = String::new();
        for gap in self.gaps.iter().rev().filter(|gap| !open || gap.state != GapState::Closed) {
            let case = gap.case.map_or_else(String::new, |case| format!(" case {case}"));
            let mut line = format!(
                "{} {} {} {} landing {}{case}: {}",
                gap.number,
                gap.kind,
                gap.owner,
                gap.state.name(),
                gap.landing,
                gap.why
            );
            line = line.replace(['\n', '\r'], " ");
            let mut end = line.len().min(200);
            while !line.is_char_boundary(end) {
                end = end.saturating_sub(1);
            }
            line.truncate(end);
            writeln!(text, "{line}").expect("string");
        }
        text
    }
}
