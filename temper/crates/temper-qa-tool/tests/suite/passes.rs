//! Pass rules over records, not commands (qa/ledger.md, 4.8–4.16).
use super::{case, landing, session};
use temper_qa_tool::{CaseState, Exit, FindingInput, FindingState, Ledger, OutcomeResult, PassStart, PassState};

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
    ledger.start_pass("places/1", &["web".into()], "high", 4).expect("start")
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
fn roundtrip(ledger: &Ledger) -> Ledger {
    Ledger::decode(&ledger.encode().expect("encode")).expect("decode")
}

#[test]
fn start_takes_newest_reached_unheld_build_and_all_compatible_cases() {
    let mut ledger = Ledger::default();
    queue(&mut ledger, "work", "web");
    queue(&mut ledger, "home", "web");
    queue(&mut ledger, "chat", "tui");
    let before = ledger.clone();
    assert_eq!(ledger.start_pass("places/1", &["web".into()], "high", 4), None);
    assert_eq!(ledger, before);
    build(&mut ledger, "A", &[1], 5);
    build(&mut ledger, "B", &[1, 2, 3], 9);
    build(&mut ledger, "unknown", &[1, 2, 3], 12);
    ledger.builds.last_mut().expect("unknown").holds = None;
    let pass = start(&mut ledger);
    assert_eq!(pass.build, "B");
    assert_eq!((pass.pass, pass.session), (1, 4));
    assert_eq!(ledger.builds[1].held, Some(1));
    assert_eq!(ledger.passes[0].state, PassState::Running);
    assert_eq!(ledger.passes[0].cases, [1, 2]);
    assert_eq!(ledger.passes[0].sessions, [pass.session]);
    assert_eq!((ledger.cases[0].state, ledger.cases[0].pass), (CaseState::Assigned, Some(1)));
    assert_eq!((ledger.cases[1].state, ledger.cases[1].pass), (CaseState::Assigned, Some(1)));
    assert_eq!((ledger.cases[2].state, ledger.cases[2].pass), (CaseState::Queued, None));
    assert_eq!(ledger.sessions.last().expect("session").effort, "high");
    let before = ledger.clone();
    assert!(ledger.start_pass("places/1", &["web".into()], "high", 5).is_none());
    assert_eq!(ledger, before);
    assert!(!ledger.deletable_builds().iter().any(|build| build.commit == "B"));
    assert!(ledger.mark_build_deleted("B", 5).is_err());
    ledger.record_outcome(pass.session, 1, "passed", "", &[], 5).expect("passed");
    ledger.record_outcome(pass.session, 2, "passed", "", &[], 5).expect("passed");
    ledger.end_pass_session(pass.session, Exit::Code(0), None, 6, "qa", &[]).expect("end");
    assert_eq!(ledger.passes[0].state, PassState::Ended);
    assert_eq!(ledger.builds[1].held, None);
    // Tui remains queued, so this newest reached build stays until a newer reached one exists.
    assert!(!ledger.deletable_builds().iter().any(|build| build.commit == "B"));
    build(&mut ledger, "C", &[1, 2, 3], 13);
    ledger.mark_build_deleted("B", 7).expect("released build deletable under 4.6");
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one scenario checks refusal atomicity before both alternatives of finding matching"
)]
fn recording_validates_commands_and_links_the_oldest_open_match() {
    let mut ledger = Ledger::default();
    queue(&mut ledger, "work", "web");
    queue(&mut ledger, "home", "web");
    build(&mut ledger, "A", &[1, 2], 5);
    let pass = start(&mut ledger);
    assert_eq!(ledger.pass_command_session(1).expect_err("ended triage").to_string(), "session 1 is not running");
    let third = u64::try_from(ledger.landings.len()).expect("length") + 1;
    landing(&mut ledger, third);
    let triage = session(&mut ledger, third);
    assert_eq!(
        ledger.pass_command_session(triage).expect_err("kind").to_string(),
        format!("session {triage} is a triage session")
    );
    assert_eq!(
        ledger.command_session(pass.session).expect_err("kind").to_string(),
        format!("session {} is a pass session", pass.session)
    );
    let mut invalid = ledger.clone();
    invalid.passes[0].state = PassState::Waiting;
    assert_eq!(
        invalid.pass_command_session(pass.session).expect_err("pass not running").to_string(),
        "pass 1 is waiting"
    );
    let before = ledger.clone();
    for (result, notes, findings, expected) in [
        ("unknown", "", vec![], "outcome refused: unknown is none of passed, failed, blocked"),
        ("blocked", "  ", vec![], "outcome refused: a blocked case says why"),
        ("failed", "", vec![], "outcome refused: a failed case names a finding of its own"),
        ("passed", "", vec![9], "outcome refused: finding 9 is not case 1's"),
    ] {
        assert_eq!(
            ledger.record_outcome(pass.session, 1, result, notes, &findings, 5).expect_err("refused").to_string(),
            expected
        );
        assert_eq!(ledger, before);
    }
    assert_eq!(
        ledger.record_outcome(pass.session, 9, "passed", "", &[], 5).expect_err("case").to_string(),
        "case 9 is not assigned to pass 1"
    );
    assert_eq!(
        ledger.record_check(pass.session, 1, "gone", "", 5).expect_err("first").to_string(),
        "check refused: pass 1's first session has no finding to check"
    );
    assert_eq!(ledger.record_note(pass.session, " ", 5).expect_err("empty").to_string(), "note refused: no words");
    for (field, expected) in [
        ("title", "finding refused: no title"),
        ("severity", "finding refused: severity urgent is none of low, medium, high"),
        ("setup", "finding refused: setup missing is not in pass 1's setups; setups: recipe"),
        ("lines", "finding refused: line 1 is empty or spans lines"),
    ] {
        let mut input = finding(1);
        match field {
            "title" => input.title.clear(),
            "severity" => input.severity = "urgent".into(),
            "setup" => input.setup = "missing".into(),
            "lines" => input.lines = vec!["a\nb".into()],
            _ => unreachable!(),
        }
        assert_eq!(
            ledger
                .record_finding(pass.session, &input, &["recipe".into()], "product", 5)
                .expect_err("invalid")
                .to_string(),
            expected
        );
        assert_eq!(ledger, before);
    }
    let first = ledger.record_finding(pass.session, &finding(1), &[], "product", 5).expect("open");
    assert_eq!(first.to_string(), "finding 1 recorded");
    assert_eq!(
        (ledger.findings[0].surface.as_str(), ledger.findings[0].page.as_str(), ledger.findings[0].landings.as_slice()),
        ("web", "work", &[1][..])
    );
    let second =
        ledger.record_finding(pass.session, &finding(2), &[], "product", 5).expect("same setup/lines, other page");
    assert_eq!(second.to_string(), "finding 2 linked to open finding 1 of pass 1");
    let mut different = finding(1);
    different.lines = vec!["other line".into()];
    different.setup = "recipe".into();
    different.expected = " \na goal starts\t".into();
    let third =
        ledger.record_finding(pass.session, &different, &["recipe".into()], "product", 5).expect("trimmed expectation");
    assert_eq!(third.linked, Some((1, 1)));
    different.expected = "another expectation".into();
    assert_eq!(
        ledger.record_finding(pass.session, &different, &["recipe".into()], "product", 5).expect("new fault").linked,
        None
    );
    different.expected = "a goal starts".into();
    assert_eq!(
        ledger.record_finding(pass.session, &different, &["recipe".into()], "product", 5).expect("two matches").linked,
        Some((1, 1))
    );
    assert_eq!(
        ledger.record_outcome(pass.session, 1, "failed", "", &[2], 5).expect_err("other case").to_string(),
        "outcome refused: finding 2 is not case 1's"
    );
    ledger.record_outcome(pass.session, 1, "failed", "", &[1], 5).expect("append");
    ledger.record_outcome(pass.session, 1, "passed", "last wins", &[], 6).expect("append");
    assert_eq!(ledger.cases[0].outcomes.len(), 2);
    assert_eq!(ledger.cases[0].state, CaseState::Assigned);
    ledger.record_note(pass.session, "off-path note", 6).expect("note");
    assert_eq!(ledger.notes[0].words, "off-path note");
    ledger.end_pass_session(pass.session, Exit::Lost, None, 7, "qa", &[]).expect("end");
    assert_eq!(ledger.cases[0].state, CaseState::Passed);
    assert_eq!(ledger.cases[1].state, CaseState::Queued);
    assert_eq!(ledger.cases[1].blocks, 0);
    assert_eq!(ledger.passes[0].state, PassState::Waiting);
    assert_eq!(
        ledger.pass_command_session(pass.session).expect_err("ended").to_string(),
        format!("session {} is not running", pass.session)
    );
}

#[test]
fn empty_sessions_block_twice_but_a_note_leaves_missing_outcomes_uncounted() {
    let mut ledger = Ledger::default();
    queue(&mut ledger, "work", "web");
    build(&mut ledger, "A", &[1], 5);
    let pass = start(&mut ledger);
    ledger.record_note(pass.session, "a note", 5).expect("note");
    ledger.end_pass_session(pass.session, Exit::Lost, None, 6, "qa", &["recipe".into()]).expect("end");
    assert_eq!(ledger.cases[0].state, CaseState::Queued);
    assert_eq!(ledger.cases[0].blocks, 0);
    assert_eq!(ledger.cases[0].pass, None);
    assert!(ledger.lines.is_empty());
    assert_eq!(ledger.passes[0].requests[0].findings, Vec::<u64>::new());
    ledger.file_request(1, 1, None, 7).expect("recipes-only file");
    assert_eq!(ledger.passes[0].requests[0].goal, None);
    let pass = start(&mut ledger);
    ledger.end_pass_session(pass.session, Exit::Lost, None, 7, "qa", &[]).expect("empty end");
    assert_eq!(ledger.cases[0].state, CaseState::Queued);
    assert_eq!(ledger.cases[0].blocks, 1);
    assert_eq!(ledger.cases[0].outcomes.last().expect("blocked").result, OutcomeResult::Blocked);
    assert_eq!(
        ledger.lines[0].words,
        format!("pass 2 (places/1, build A): session {} recorded nothing; its cases count as blocked", pass.session)
    );
    let pass = start(&mut ledger);
    ledger.record_outcome(pass.session, 1, "blocked", "no browser", &[], 8).expect("block");
    ledger.end_pass_session(pass.session, Exit::Code(0), None, 9, "qa", &[]).expect("end");
    assert_eq!(ledger.cases[0].state, CaseState::Blocked);
    assert_eq!(ledger.cases[0].blocks, 2);
    assert_eq!(ledger.lines[1].words, "case 1 (work on web; landings 1) blocked twice: no browser");
    assert!(ledger.start_pass("places/1", &["web".into()], "high", 10).is_none());
    ledger.mark_build_deleted("A", 10).expect("released and no queued case");
}

#[test]
fn failed_linked_cases_move_to_the_other_live_pass_for_its_resume() {
    let (mut ledger, first) = waiting();
    queue(&mut ledger, "home", "web");
    build(&mut ledger, "B", &[1, 2], 9);
    let second = start(&mut ledger);
    let linked = ledger.record_finding(second.session, &finding(2), &[], "product", 10).expect("linked");
    assert_eq!(linked.linked, Some((1, first.pass)));
    ledger.record_outcome(second.session, 2, "failed", "same fault", &[2], 11).expect("failed");
    ledger.end_pass_session(second.session, Exit::Code(0), None, 12, "qa", &[]).expect("end");
    assert_eq!((ledger.cases[1].state, ledger.cases[1].pass), (CaseState::Failed, Some(first.pass)));
    assert_eq!(ledger.passes[0].cases, [1, 2]);
    assert_eq!(ledger.passes[1].state, PassState::Ended);
    ledger.file_request(first.pass, 1, Some(100), 13).expect("file");
    ledger.end_goal(100, 14).expect("goal end");
    let resumed = ledger.resume_pass("places/1", &["web".into()], "high", 15).expect("resume");
    assert_eq!(resumed.pass, first.pass);
    assert_eq!(ledger.cases[1].state, CaseState::Assigned);
    // Linked targets no longer open cannot keep a new failed case in a pass.
    ledger.record_check(resumed.session, 1, "gone", "fixed", 16).expect("check");
    ledger.record_note(resumed.session, "recorded", 16).expect("note");
    ledger.end_pass_session(resumed.session, Exit::Code(0), None, 17, "qa", &[]).expect("end");
    assert_eq!(ledger.cases[1].state, CaseState::Queued);
}

#[test]
fn resumes_joined_and_queued_cases_on_newest_build_then_closes_or_hands_findings() {
    let (mut ledger, pass) = waiting();
    assert_eq!(ledger.passes[0].state, PassState::Waiting);
    assert_eq!(ledger.builds[0].held, Some(pass.pass));
    assert_eq!(ledger.passes[0].requests[0].findings, [1]);
    assert_eq!(ledger.passes[0].requests[0].recipes, ["recipe"]);
    let before = ledger.clone();
    assert!(ledger.resume_pass("places/1", &["web".into()], "high", 8).is_none());
    assert_eq!(ledger, before);
    ledger.file_request(pass.pass, 1, Some(100), 8).expect("file");
    assert_eq!(ledger.findings[0].goals, [100]);
    let next = 2;
    landing(&mut ledger, next);
    let triage = session(&mut ledger, next);
    assert_eq!(
        ledger.add_case(triage, &case(), 9).expect("join waiting").to_string(),
        "case 2 joined pass 1, waiting on finding 1"
    );
    ledger.end_session(triage, Exit::Code(0), None, 10, "qa").expect("end");
    assert_eq!((ledger.cases[1].state, ledger.cases[1].pass), (CaseState::Assigned, Some(1)));
    queue(&mut ledger, "home", "web");
    let goal_landing = ledger.record_landing("goal-landing", 100, None, "fix", 11).expect("landing");
    let triage = session(&mut ledger, goal_landing);
    ledger.answer_none(triage, "fix check").expect("none");
    ledger.end_session(triage, Exit::Code(0), None, 12, "qa").expect("end");
    build(&mut ledger, "goal-landing", &[1, 2, 3, 4], 10);
    build(&mut ledger, "newer-than-landing", &[1, 2, 3, 4], 15);
    let resumed = ledger.resume_pass("places/2", &["web".into()], "high", 13).expect("resume");
    assert_eq!(resumed.build, "newer-than-landing");
    assert_eq!(ledger.builds[0].held, None);
    assert_eq!(ledger.builds[2].held, Some(1));
    assert_eq!(ledger.passes[0].cases, [1, 2, 3]);
    assert!(ledger.cases.iter().all(|case| case.state == CaseState::Assigned));
    ledger.mark_build_deleted("A", 14).expect("old released");
    queue(&mut ledger, "home", "web");
    assert_eq!(ledger.cases[3].state, CaseState::Queued);
    assert!(!ledger.passes[0].cases.contains(&4));
    assert_eq!(
        ledger.record_check(resumed.session, 1, "unknown", "", 14).expect_err("check result").to_string(),
        "check refused: unknown is neither holds nor gone"
    );
    assert_eq!(
        ledger.record_check(resumed.session, 999, "gone", "", 14).expect_err("not open").to_string(),
        "check refused: finding 999 is not open in pass 1"
    );
    let mut gone = ledger.clone();
    gone.record_check(resumed.session, 1, "holds", "first check", 14).expect("check");
    gone.record_check(resumed.session, 1, "gone", "fixed", 15).expect("last check");
    assert_eq!(gone.findings[0].state, FindingState::Open);
    gone.end_pass_session(resumed.session, Exit::Code(0), None, 16, "qa", &[]).expect("end");
    assert_eq!(gone.findings[0].state, FindingState::Closed);
    assert_eq!(gone.findings[0].closed.as_deref(), Some("newer-than-landing"));
    assert_eq!(gone.findings[0].closed_time, Some(16));
    assert_eq!(gone.passes[0].state, PassState::Ended);
    assert_eq!(gone.builds[2].held, None);
    ledger.record_note(resumed.session, "no check: still open", 14).expect("note");
    ledger.end_pass_session(resumed.session, Exit::Code(0), None, 15, "qa", &[]).expect("end");
    assert_eq!(
        ledger.cases[0].state,
        CaseState::Failed,
        "no outcome preserves a resumed failure with its open finding"
    );
    ledger.file_request(pass.pass, 2, Some(101), 16).expect("second file");
    ledger.end_goal(101, 17).expect("no landing");
    assert_eq!(ledger.passes[0].requests[1].ended, Some(17));
    assert_eq!(ledger.findings[0].goals, [100, 101]);
    let resumed = ledger.resume_pass("places/2", &["web".into()], "high", 18).expect("resume ended goal");
    ledger.record_check(resumed.session, 1, "holds", "still there", 19).expect("check");
    ledger.end_pass_session(resumed.session, Exit::Code(0), None, 20, "qa", &[]).expect("end");
    assert_eq!(ledger.findings[0].state, FindingState::Handed);
    assert_eq!(ledger.passes[0].state, PassState::Ended);
    assert_eq!(ledger.lines.last().expect("handed").area, "product");
    assert_eq!(
        ledger.lines.last().expect("handed").words,
        "finding 1 (Start does nothing; setup fresh) still holds after goals 100 and 101; handed on"
    );
}

#[test]
fn unstarted_start_and_resume_undo_from_persisted_before_records() {
    let mut ledger = Ledger::default();
    queue(&mut ledger, "work", "web");
    build(&mut ledger, "A", &[1], 5);
    let pass = start(&mut ledger);
    ledger = roundtrip(&ledger);
    ledger.end_pass_session(pass.session, Exit::Unstarted, None, 5, "qa", &[]).expect("unstarted start");
    assert_eq!((ledger.cases[0].state, ledger.cases[0].pass, ledger.cases[0].blocks), (CaseState::Queued, None, 0));
    assert_eq!(ledger.passes[0].state, PassState::Ended);
    assert_eq!(ledger.builds[0].held, None);
    assert!(ledger.lines.is_empty());
    let (mut ledger, pass) = waiting();
    queue(&mut ledger, "work", "web");
    queue(&mut ledger, "home", "web");
    ledger.file_request(pass.pass, 1, Some(100), 8).expect("file");
    ledger.end_goal(100, 9).expect("end goal");
    build(&mut ledger, "B", &[1, 2, 3], 9);
    let cases = ledger.cases.clone();
    let resumed = ledger.resume_pass("places/2", &["web".into()], "high", 10).expect("resume");
    ledger = roundtrip(&ledger);
    ledger.end_pass_session(resumed.session, Exit::Unstarted, None, 11, "qa", &[]).expect("undo resume");
    assert_eq!(ledger.cases, cases);
    assert_eq!(ledger.passes[0].state, PassState::Waiting);
    assert_eq!(ledger.passes[0].place, "places/1");
    assert_eq!(ledger.passes[0].build, "A");
    assert_eq!(ledger.builds[0].held, Some(pass.pass));
    assert_eq!(ledger.builds[1].held, None);
    assert!(ledger.lines.is_empty());
    assert_eq!(ledger.passes[0].requests.len(), 1);
}

#[test]
fn q1_decodes_without_q3_and_all_q3_records_roundtrip() {
    let q1 = br#"{"landings":[{"number":1,"commit":"0000000000000000000000000000000000000001","goal":11,"title":"a landing","seen":0,"answer":"cases","triaged":2,"sessions":0}],"cases":[{"number":1,"surface":"web","page":"work","asks":[{"landing":1,"setup":"fresh","adds":"","steps":[{"do":"open","expect":"page"}]}],"state":"queued","queued":1}],"builds":[{"commit":"A","kept":3,"state":"kept","holds":[1],"order":5}],"sessions":[{"number":1,"kind":"triage","landing":1,"effort":"low","started":0,"ended":2,"exit":{"code":0},"pages":{"web":["work"]},"setups":["fresh"]}],"lines":[]}"#;
    let old = Ledger::decode(q1).expect("Q1");
    assert_eq!(roundtrip(&old), old);
    assert!(old.passes.is_empty() && old.findings.is_empty() && old.notes.is_empty());
    assert_eq!(old.cases[0].state, CaseState::Queued);
    assert_eq!(old.cases[0].pass, None);
    assert_eq!(old.cases[0].blocks, 0);
    assert!(old.cases[0].outcomes.is_empty());
    assert_eq!(old.sessions[0].before, None);
    let (mut ledger, pass) = waiting();
    ledger.file_request(pass.pass, 1, Some(100), 8).expect("file");
    ledger.end_goal(100, 9).expect("goal");
    let resumed = ledger.resume_pass("places/2", &["web".into()], "high", 10).expect("resume");
    ledger.record_note(resumed.session, "a note", 11).expect("note");
    ledger.record_finding(resumed.session, &finding(1), &[], "product", 11).expect("linked");
    ledger.record_check(resumed.session, 1, "gone", "fixed", 12).expect("check");
    ledger.end_pass_session(resumed.session, Exit::Signal(15), Some(12), 13, "qa", &[]).expect("end");
    assert_eq!(roundtrip(&ledger), ledger);
    assert_eq!(ledger.findings[1].to, Some(1));
    assert_eq!(ledger.findings[0].checks[0].commit, "A");
    for pointer in [
        "/passes/0",
        "/passes/0/requests/0",
        "/findings/0",
        "/findings/0/checks/0",
        "/notes/0",
        "/sessions/2/before",
        "/cases/0/outcomes/0",
    ] {
        let mut document = ledger.document();
        document
            .pointer_mut(pointer)
            .expect("record")
            .as_object_mut()
            .expect("object")
            .insert("future".into(), serde_json::json!(true));
        assert!(
            Ledger::decode(&serde_json::to_vec(&document).expect("json"))
                .expect_err("unknown")
                .to_string()
                .contains("future"),
            "{pointer}"
        );
    }
}
