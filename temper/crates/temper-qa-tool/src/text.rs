//! The person's text (qa/ledger.md, 6). UTC dates use March-based Gregorian eras.
use crate::ledger::{numbers, short};
use crate::{Answer, BuildState, Exit, Ledger, SessionKind};
use std::fmt::Write;

impl Ledger {
    #[must_use]
    pub fn show(&self) -> String {
        let mut text = String::from("landings:\n");
        for landing in &self.landings {
            let state = if landing.unanswered {
                "triaged unanswered".into()
            } else if let Some(time) = landing.triaged {
                format!("triaged {}", utc_minute(time))
            } else {
                "untriaged".into()
            };
            let answer = match &landing.answer {
                None => String::new(),
                Some(Answer::None { why }) => format!("; no QA: {why}"),
                Some(Answer::Cases) => "; cases".into(),
            };
            writeln!(
                text,
                "  {} {} goal {} {}: {state}{answer}",
                landing.number,
                short(&landing.commit),
                landing.goal,
                landing.title
            )
            .expect("String write");
        }
        text.push_str("\ncases:\n");
        for case in &self.cases {
            writeln!(
                text,
                "  {} {} {} {}; pass {}; landings {}",
                case.number,
                case.surface,
                case.page,
                case.state.name(),
                case.pass.map_or_else(|| "none".into(), |number| number.to_string()),
                numbers(&case.landings())
            )
            .expect("String write");
        }
        text.push_str("\nbuilds:\n");
        for build in &self.builds {
            let state = match build.state {
                BuildState::Kept => "kept",
                BuildState::Deleted => "deleted",
            };
            let holds = build.holds.as_ref().map_or_else(|| "unknown".into(), |holds| numbers(holds));
            writeln!(
                text,
                "  {} {state}; landings {holds}; held {}",
                short(&build.commit),
                build.held.map_or_else(|| "none".into(), |number| number.to_string())
            )
            .expect("String write");
        }
        text.push_str("\nsessions:\n");
        for session in &self.sessions {
            let ended = session.ended.map_or_else(|| "running".into(), utc_minute);
            let exit = session.exit.map_or_else(|| "running".into(), exit_text);
            let target = match session.kind {
                SessionKind::Triage => format!("triage landing {}", session.landing.expect("triage landing")),
                SessionKind::Pass => format!("pass {}", session.pass.expect("pass session")),
            };
            writeln!(
                text,
                "  {} {target} {} {} - {ended}; {exit}",
                session.number,
                session.effort,
                utc_minute(session.started)
            )
            .expect("String write");
        }
        self.show_passes(&mut text);
        text
    }
    fn show_passes(&self, text: &mut String) {
        text.push_str("\npasses:\n");
        for pass in &self.passes {
            writeln!(
                text,
                "  {} {} {} {}; cases {}; goal {}",
                pass.number,
                pass.place,
                short(&pass.build),
                pass.state.name(),
                numbers(&pass.cases),
                pass.requests
                    .last()
                    .and_then(|request| request.goal)
                    .map_or_else(|| "none".into(), |goal| goal.to_string())
            )
            .expect("String write");
        }
        text.push_str("\nfindings:\n");
        for finding in &self.findings {
            writeln!(
                text,
                "  {} pass {} case {} {} {} {}",
                finding.number,
                finding.pass,
                finding.case,
                finding.severity,
                finding.state.name(),
                finding.title
            )
            .expect("String write");
        }
        text.push_str("\nnotes:\n");
        for note in &self.notes {
            writeln!(text, "  {} pass {} {}", note.number, note.pass, note.words).expect("String write");
        }
    }
    #[must_use]
    pub fn lines_text(&self) -> String {
        let mut text = String::new();
        for line in &self.lines {
            writeln!(text, "{} {} {}: {}", line.number, utc_minute(line.time), line.area, line.words)
                .expect("String write");
        }
        text
    }
}
fn exit_text(exit: Exit) -> String {
    match exit {
        Exit::Code(code) => format!("code {code}"),
        Exit::Signal(signal) => format!("signal {signal}"),
        Exit::Bound => "bound".into(),
        Exit::Lost => "lost".into(),
        Exit::Unstarted => "unstarted".into(),
    }
}
// The ledger counts whole seconds, rather than Wall's bounded nanoseconds.
fn utc_minute(seconds: u64) -> String {
    let days = seconds / 86_400 + 719_468;
    let era = days / 146_097;
    let doe = days % 146_097;
    let yoe = (doe - doe / 1_460 + doe / 36_524 - doe / 146_096) / 365;
    let doy = doe - (365 * yoe + yoe / 4 - yoe / 100);
    let mp = (5 * doy + 2) / 153;
    let day = doy - (153 * mp + 2) / 5 + 1;
    let month = if mp < 10 { mp + 3 } else { mp - 9 };
    let year = era * 400 + yoe + u64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}T{:02}:{:02}Z", seconds % 86_400 / 3600, seconds % 3600 / 60)
}
