//! Ledger fixtures shared by the value and real-file tests.
use super::{case, landing, session};
use crate::{self as temper_qa_tool, Exit, FindingInput, Ledger, PassStart};

pub fn queue(ledger: &mut Ledger, page: &str, surface: &str) -> u64 {
    let number = u64::try_from(ledger.landings.len()).expect("length") + 1;
    landing(ledger, number);
    let session = session(ledger, number);
    let mut input = case();
    input.page = page.into();
    input.surface = surface.into();
    let answer = ledger.add_case(session, &input, 1).expect("case");
    ledger.end_session(session, Exit::Code(0), None, 2, "qa").expect("triage end");
    match answer {
        temper_qa_tool::CaseAnswer::Queued(number)
        | temper_qa_tool::CaseAnswer::Joined { number, .. }
        | temper_qa_tool::CaseAnswer::Waiting { number, .. } => number,
    }
}

pub fn build(ledger: &mut Ledger, name: &str, holds: &[u64], order: u64) {
    ledger.keep_build(name, 3).expect("build");
    ledger.set_build_holds(name, holds.to_vec(), order).expect("holds");
}

pub fn start(ledger: &mut Ledger) -> PassStart {
    ledger.start_pass("places/1", temper_qa_tool::Kind::Box, &["web".into()], "high", 4).expect("start")
}

pub fn finding(case: u64) -> FindingInput {
    FindingInput {
        case,
        title: "Start does nothing".into(),
        severity: "medium".into(),
        area: None,
        setup: "fresh".into(),
        lines: vec!["web press button Start".into()],
        expected: "a goal starts".into(),
        seen: "no goal".into(),
        evidence: vec!["look.png".into()],
    }
}

pub fn waiting() -> (Ledger, PassStart) {
    let mut ledger = Ledger::default();
    queue(&mut ledger, "work", "web");
    build(&mut ledger, "A", &[1], 5);
    let pass = start(&mut ledger);
    ledger.record_finding(pass.session, &finding(1), &[], "product", 5).expect("finding");
    ledger.record_outcome(pass.session, 1, "failed", "no goal", &[1], 6).expect("outcome");
    ledger.end_pass_session(pass.session, Exit::Bound, Some(42), 7, "qa", &["recipe".into()]).expect("end");
    (ledger, pass)
}

/// Two ctl cases on the same page, separated only by setup kind.
pub fn queue_kinds(ledger: &mut Ledger) {
    landing(ledger, 1);
    let session = ledger
        .start_session(
            1,
            "low",
            [
                ("ctl".into(), vec!["task".into()]),
                ("web".into(), vec!["work".into()]),
                ("tui".into(), vec!["chat".into()]),
            ]
            .into(),
            vec!["welcome".into(), "turn-landing".into()],
            vec!["turn-landing".into()],
            0,
        )
        .expect("triage");
    let mut input = case();
    input.surface = "ctl".into();
    input.page = "task".into();
    input.setup = "turn-landing".into();
    ledger.add_case(session, &input, 1).expect("deployment case");
    input.setup = "welcome".into();
    ledger.add_case(session, &input, 1).expect("box case");
    ledger.end_session(session, Exit::Code(0), None, 2, "qa").expect("triaged");
}
