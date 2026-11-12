//! Recording commands' value operations (qa/ledger.md, 4.9–4.11 and 4.14).
use crate::ledger::next_number;
use crate::{Ledger, Outcome, OutcomeResult, Refusal, refused};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FindingState {
    Open,
    Linked,
    Closed,
    Handed,
}
impl FindingState {
    #[must_use]
    pub const fn name(self) -> &'static str {
        match self {
            Self::Open => "open",
            Self::Linked => "linked",
            Self::Closed => "closed",
            Self::Handed => "handed",
        }
    }
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Finding {
    pub number: u64,
    pub pass: u64,
    pub case: u64,
    pub session: u64,
    pub time: u64,
    pub title: String,
    pub severity: String,
    pub area: String,
    pub setup: String,
    pub lines: Vec<String>,
    pub expected: String,
    pub seen: String,
    pub evidence: Vec<String>,
    pub surface: String,
    pub page: String,
    pub landings: Vec<u64>,
    pub state: FindingState,
    pub to: Option<u64>,
    pub closed: Option<String>,
    pub closed_time: Option<u64>,
    pub goals: Vec<u64>,
    pub checks: Vec<Check>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Check {
    pub session: u64,
    pub time: u64,
    pub commit: String,
    pub holds: bool,
    pub notes: String,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Note {
    pub number: u64,
    pub pass: u64,
    pub session: u64,
    pub time: u64,
    pub words: String,
}
/// Evidence is already copied; these are its kept file names, not source paths.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FindingInput {
    pub case: u64,
    pub title: String,
    pub severity: String,
    pub area: Option<String>,
    pub setup: String,
    pub lines: Vec<String>,
    pub expected: String,
    pub seen: String,
    pub evidence: Vec<String>,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FindingAnswer {
    pub number: u64,
    pub linked: Option<(u64, u64)>,
}
impl std::fmt::Display for FindingAnswer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.linked {
            Some((number, pass)) => write!(f, "finding {} linked to open finding {number} of pass {pass}", self.number),
            None => write!(f, "finding {} recorded", self.number),
        }
    }
}
impl FindingInput {
    pub(crate) fn check(&self, pass: u64, setups: &[String]) -> Result<(), Refusal> {
        if self.case == 0 {
            return Err(refused("finding refused: no case"));
        }
        for (field, empty) in [
            ("title", self.title.trim().is_empty()),
            ("severity", self.severity.is_empty()),
            ("setup", self.setup.trim().is_empty()),
            ("lines", self.lines.is_empty()),
            ("expected", self.expected.trim().is_empty()),
            ("seen", self.seen.trim().is_empty()),
        ] {
            if empty {
                return Err(refused(format!("finding refused: no {field}")));
            }
        }
        if !["low", "medium", "high"].contains(&self.severity.as_str()) {
            return Err(refused(format!("finding refused: severity {} is none of low, medium, high", self.severity)));
        }
        if self.setup != "fresh" && !setups.contains(&self.setup) {
            return Err(refused(format!(
                "finding refused: setup {} is not in pass {pass}'s setups; setups: {}",
                self.setup,
                setups.join(", ")
            )));
        }
        for (index, line) in self.lines.iter().enumerate() {
            if line.trim().is_empty() || line.contains(['\n', '\r']) {
                return Err(refused(format!(
                    "finding refused: line {} is empty or spans lines",
                    index.saturating_add(1)
                )));
            }
        }
        Ok(())
    }
}
impl Ledger {
    pub fn record_outcome(
        &mut self,
        session: u64,
        case: u64,
        result: &str,
        notes: &str,
        findings: &[u64],
        now: u64,
    ) -> Result<(), Refusal> {
        let record = self.pass_command_case(session, case)?;
        let pass = record.pass.expect("assigned case");
        let result = OutcomeResult::parse(result)?;
        if result == OutcomeResult::Blocked && notes.trim().is_empty() {
            return Err(refused("outcome refused: a blocked case says why"));
        }
        if result == OutcomeResult::Failed && findings.is_empty() {
            return Err(refused("outcome refused: a failed case names a finding of its own"));
        }
        for number in findings {
            if !self
                .findings
                .iter()
                .any(|finding| finding.number == *number && finding.case == case && finding.pass == pass)
            {
                return Err(refused(format!("outcome refused: finding {number} is not case {case}'s")));
            }
        }
        self.cases.iter_mut().find(|record| record.number == case).expect("checked case").outcomes.push(Outcome {
            session,
            time: now,
            result,
            notes: notes.into(),
            findings: findings.to_vec(),
        });
        Ok(())
    }

    pub fn record_finding(
        &mut self,
        session: u64,
        input: &FindingInput,
        setups: &[String],
        default_area: &str,
        now: u64,
    ) -> Result<FindingAnswer, Refusal> {
        let pass = self.pass_command_session(session)?.pass.expect("pass session");
        input.check(pass, setups)?;
        let case = self.pass_command_case(session, input.case)?;
        let linked = self
            .findings
            .iter()
            .filter(|finding| {
                finding.state == FindingState::Open
                    && ((finding.setup == input.setup && finding.lines == input.lines)
                        || (finding.surface == case.surface
                            && finding.page == case.page
                            && finding.expected.trim() == input.expected.trim()))
            })
            .min_by_key(|finding| finding.number)
            .map(|finding| (finding.number, finding.pass));
        let finding = Finding {
            number: next_number(self.findings.iter().map(|finding| finding.number)),
            pass,
            case: case.number,
            session,
            time: now,
            title: input.title.clone(),
            severity: input.severity.clone(),
            area: input.area.clone().unwrap_or_else(|| default_area.into()),
            setup: input.setup.clone(),
            lines: input.lines.clone(),
            expected: input.expected.clone(),
            seen: input.seen.clone(),
            evidence: input.evidence.clone(),
            surface: case.surface.clone(),
            page: case.page.clone(),
            landings: case.landings(),
            state: if linked.is_some() { FindingState::Linked } else { FindingState::Open },
            to: linked.map(|(number, _)| number),
            closed: None,
            closed_time: None,
            goals: Vec::new(),
            checks: Vec::new(),
        };
        let number = finding.number;
        self.findings.push(finding);
        Ok(FindingAnswer { number, linked })
    }

    pub fn record_check(
        &mut self,
        session: u64,
        finding: u64,
        result: &str,
        notes: &str,
        now: u64,
    ) -> Result<(), Refusal> {
        let pass = self.pass_command_session(session)?.pass.expect("pass session");
        let pass = self.passes.iter().find(|record| record.number == pass).expect("session pass");
        if pass.sessions.first() == Some(&session) {
            return Err(refused(format!(
                "check refused: pass {}'s first session has no finding to check",
                pass.number
            )));
        }
        let holds = match result {
            "holds" => true,
            "gone" => false,
            other => return Err(refused(format!("check refused: {other} is neither holds nor gone"))),
        };
        let record = self
            .findings
            .iter_mut()
            .find(|record| record.number == finding && record.pass == pass.number && record.state == FindingState::Open)
            .ok_or_else(|| refused(format!("check refused: finding {finding} is not open in pass {}", pass.number)))?;
        record.checks.push(Check { session, time: now, commit: pass.build.clone(), holds, notes: notes.into() });
        Ok(())
    }

    pub fn record_note(&mut self, session: u64, words: &str, now: u64) -> Result<u64, Refusal> {
        let pass = self.pass_command_session(session)?.pass.expect("pass session");
        if words.trim().is_empty() {
            return Err(refused("note refused: no words"));
        }
        let number = next_number(self.notes.iter().map(|note| note.number));
        self.notes.push(Note { number, pass, session, time: now, words: words.into() });
        Ok(number)
    }
}
