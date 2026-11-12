//! Atomic pass session endings (qa/ledger.md, 4.12 and 4.14).
use crate::ledger::{next_number, numbers, short};
use crate::{CaseState, Exit, FindingState, Ledger, Outcome, OutcomeResult, PassState, Refusal, Request};

impl Ledger {
    pub fn end_pass_session(
        &mut self,
        number: u64,
        exit: Exit,
        tokens: Option<u64>,
        now: u64,
        area: &str,
        recipes: &[String],
    ) -> Result<(), Refusal> {
        let session = self.pass_command_session(number)?.clone();
        let pass_number = session.pass.expect("pass session");
        let pass = self.passes.iter().find(|pass| pass.number == pass_number).expect("session's pass").clone();
        let first = pass.sessions.first() == Some(&number);
        if exit == Exit::Unstarted {
            self.undo_pass_start(number, now);
        } else {
            let recorded = self.cases.iter().flat_map(|case| &case.outcomes).any(|outcome| outcome.session == number)
                || self.findings.iter().any(|finding| {
                    finding.session == number || finding.checks.iter().any(|check| check.session == number)
                })
                || self.notes.iter().any(|note| note.session == number);
            if !recorded {
                for case in self
                    .cases
                    .iter_mut()
                    .filter(|case| case.pass == Some(pass_number) && case.state == CaseState::Assigned)
                {
                    case.outcomes.push(Outcome {
                        session: number,
                        time: now,
                        result: OutcomeResult::Blocked,
                        notes: "the pass's session recorded nothing".into(),
                        findings: Vec::new(),
                    });
                }
                self.add_line(area, &format!("pass {pass_number} ({}, build {}): session {number} recorded nothing; its cases count as blocked", pass.place, short(&pass.build)), now);
            }
            let cases: Vec<_> = self
                .cases
                .iter()
                .filter(|case| case.pass == Some(pass_number) && case.state == CaseState::Assigned)
                .map(|case| case.number)
                .collect();
            for case in cases {
                self.finish_case(case, number, now, area);
            }
            if !first {
                self.finish_round(pass_number, number, &pass.build, now);
            }
            let findings: Vec<_> = self
                .findings
                .iter()
                .filter(|finding| finding.pass == pass_number && finding.state == FindingState::Open)
                .map(|finding| finding.number)
                .collect();
            let waiting = !findings.is_empty();
            let pass = self.passes.iter_mut().find(|pass| pass.number == pass_number).expect("session's pass");
            if waiting {
                pass.state = PassState::Waiting;
            } else {
                pass.state = PassState::Ended;
                pass.ended = Some(now);
            }
            if waiting || !recipes.is_empty() {
                pass.requests.push(Request {
                    number: next_number(pass.requests.iter().map(|request| request.number)),
                    time: now,
                    findings,
                    recipes: recipes.to_vec(),
                    filed: None,
                    goal: None,
                    ended: None,
                });
            }
            if !waiting {
                for build in &mut self.builds {
                    if build.held == Some(pass_number) {
                        build.held = None;
                    }
                }
            }
        }
        let session = self.sessions.iter_mut().find(|session| session.number == number).expect("checked session");
        session.ended = Some(now);
        session.exit = Some(exit);
        session.tokens = tokens;
        Ok(())
    }

    fn undo_pass_start(&mut self, number: u64, now: u64) {
        let session = self.sessions.iter().find(|session| session.number == number).expect("checked session");
        let pass_number = session.pass.expect("pass session");
        let before = session.before.as_ref().expect("start/resume snapshot");
        for saved in &before.cases {
            let case = self.cases.iter_mut().find(|case| case.number == saved.number).expect("snapshot case");
            case.state = saved.state;
            case.pass = saved.pass;
        }
        let pass = self.passes.iter_mut().find(|pass| pass.number == pass_number).expect("session's pass");
        if let Some(state) = before.state {
            pass.state = state;
            pass.place.clone_from(before.place.as_ref().expect("prior place"));
            pass.build.clone_from(before.build.as_ref().expect("prior build"));
        } else {
            pass.state = PassState::Ended;
            pass.ended = Some(now);
        }
        for build in &mut self.builds {
            if build.held == Some(pass_number) {
                build.held = None;
            }
            if before.state.is_some() && build.commit == pass.build {
                build.held = Some(pass_number);
            }
        }
    }

    fn finish_case(&mut self, number: u64, session: u64, now: u64, area: &str) {
        let case = self.cases.iter().find(|case| case.number == number).expect("assigned case");
        let pass = case.pass.expect("assigned case's pass");
        let outcome = case.outcomes.iter().rev().find(|outcome| outcome.session == session).cloned();
        let mut line = None;
        let (state, owner) = match outcome {
            Some(Outcome { result: OutcomeResult::Passed, .. }) => (CaseState::Passed, Some(pass)),
            Some(Outcome { result: OutcomeResult::Blocked, notes, .. }) => {
                let blocks = case.blocks.saturating_add(1);
                if blocks >= 2 {
                    line = Some(format!(
                        "case {number} ({} on {}; landings {}) blocked twice: {notes}",
                        case.page,
                        case.surface,
                        numbers(&case.landings())
                    ));
                }
                let case = self.cases.iter_mut().find(|case| case.number == number).expect("assigned case");
                case.blocks = blocks;
                if blocks >= 2 { (CaseState::Blocked, Some(pass)) } else { (CaseState::Queued, None) }
            }
            Some(Outcome { result: OutcomeResult::Failed, findings, .. }) => self.failed_case(pass, &findings),
            None => {
                let previous = self
                    .sessions
                    .iter()
                    .find(|record| record.number == session)
                    .and_then(|record| record.before.as_ref())
                    .and_then(|before| before.cases.iter().find(|saved| saved.number == number));
                if previous.is_some_and(|saved| saved.state == CaseState::Failed)
                    && self.findings.iter().any(|finding| {
                        finding.case == number && finding.pass == pass && finding.state == FindingState::Open
                    })
                {
                    (CaseState::Failed, Some(pass))
                } else {
                    (CaseState::Queued, None)
                }
            }
        };
        let case = self.cases.iter_mut().find(|case| case.number == number).expect("assigned case");
        case.state = state;
        case.pass = owner;
        if let Some(owner) = owner {
            let pass = self.passes.iter_mut().find(|pass| pass.number == owner).expect("case owner");
            if !pass.cases.contains(&number) {
                pass.cases.push(number);
            }
        }
        if let Some(words) = line {
            self.add_line(area, &words, now);
        }
    }

    fn failed_case(&self, pass: u64, numbers: &[u64]) -> (CaseState, Option<u64>) {
        let findings: Vec<_> = numbers
            .iter()
            .filter_map(|number| self.findings.iter().find(|finding| finding.number == *number))
            .collect();
        if findings.iter().any(|finding| finding.pass == pass && finding.state == FindingState::Open) {
            return (CaseState::Failed, Some(pass));
        }
        let targets: Vec<_> = findings
            .iter()
            .filter_map(|finding| {
                if finding.state != FindingState::Linked {
                    return None;
                }
                self.findings.iter().find(|target| {
                    Some(target.number) == finding.to
                        && target.state == FindingState::Open
                        && target.pass != pass
                        && self.passes.iter().any(|pass| pass.number == target.pass && pass.state != PassState::Ended)
                })
            })
            .collect();
        if !targets.is_empty()
            && targets.len() == findings.len()
            && targets.iter().all(|target| target.pass == targets.first().expect("targets").pass)
        {
            return (
                CaseState::Failed,
                targets.iter().min_by_key(|finding| finding.number).map(|finding| finding.pass),
            );
        }
        (CaseState::Queued, None)
    }

    fn finish_round(&mut self, pass: u64, session: u64, commit: &str, now: u64) {
        let mut lines = Vec::new();
        for finding in
            self.findings.iter_mut().filter(|finding| finding.pass == pass && finding.state == FindingState::Open)
        {
            if finding.checks.iter().rev().find(|check| check.session == session).is_some_and(|check| !check.holds) {
                finding.state = FindingState::Closed;
                finding.closed = Some(commit.into());
                finding.closed_time = Some(now);
            } else if finding.goals.len() >= 2 {
                finding.state = FindingState::Handed;
                lines.push((
                    finding.area.clone(),
                    format!(
                        "finding {} ({}; setup {}) still holds after goals {} and {}; handed on",
                        finding.number,
                        finding.title,
                        finding.setup,
                        finding.goals.first().expect("two goals"),
                        finding.goals.get(1).expect("two goals")
                    ),
                ));
            }
        }
        for (area, words) in lines {
            self.add_line(&area, &words, now);
        }
    }
}
