//! Gap rules over ledger values (qa/gaps.md, 8).
use super::passes::{build, queue, start};
use super::{case, landing, session};
use temper_qa_tool::{Answer, CaseState, Exit, Ledger, PassState};

#[test]
fn triage_gaps_answer_the_landing_and_can_accompany_cases() {
    let mut ledger = Ledger::default();
    landing(&mut ledger, 1);
    let session = session(&mut ledger, 1);
    assert_eq!(ledger.triage_gap(session, "ctl", "no tool to see the runner", "tools", 1).expect("gap"), 1);
    assert_eq!(ledger.landings[0].answer, Some(Answer::Gap));
    ledger.add_case(session, &case(), 2).expect("case beside gap");
    ledger.triage_gap(session, "fake", "runner is absent", "fakes", 3).expect("second gap");
    assert_eq!(ledger.landings[0].answer, Some(Answer::Cases));
    ledger.end_session(session, Exit::Code(0), None, 4, "qa").expect("end");
    assert_eq!(ledger.landings[0].triaged, Some(4));
    assert_eq!(ledger.gaps.len(), 2);
    assert_eq!(Ledger::decode(&ledger.encode().expect("encode")).expect("decode"), ledger);
}

#[test]
fn a_pass_gap_leaves_the_queue_at_once_and_is_reported() {
    let mut ledger = Ledger::default();
    queue(&mut ledger, "work", "web");
    build(&mut ledger, "build", &[1], 1);
    let pass = start(&mut ledger);
    let number = ledger.pass_gap(pass.session, 1, "qa", "no setup reaches the pushed branch", "qa", 5).expect("gap");
    assert_eq!(ledger.cases[0].state, CaseState::Gap);
    ledger.end_pass_session(pass.session, Exit::Code(0), None, 6, "qa", &[]).expect("end");
    assert_eq!(ledger.passes[0].state, PassState::Ended);
    assert_eq!(ledger.cases[0].blocks, 0);
    assert_eq!(ledger.lines.len(), 0);
    let report = ledger.report(pass.pass).expect("report");
    assert_eq!(report["cases"][0]["outcome"], "gap");
    assert_eq!(report["cases"][0]["gap"], number);
    assert_eq!(report["gaps"][0]["why"], "no setup reaches the pushed branch");
    assert!(ledger.start_pass("places/1", temper_qa_tool::Kind::Box, &["web".into()], "high", 7).is_none());
}

#[test]
fn gap_refusals_record_nothing_for_either_session_kind() {
    let mut triage = Ledger::default();
    landing(&mut triage, 1);
    let triage_session = session(&mut triage, 1);
    let mut pass = Ledger::default();
    queue(&mut pass, "work", "web");
    build(&mut pass, "build", &[1], 1);
    let pass_session = start(&mut pass).session;
    queue(&mut pass, "home", "web");
    build(&mut pass, "other-build", &[1, 2], 2);
    pass.start_pass("places/2", temper_qa_tool::Kind::Box, &["web".into()], "high", 4).expect("other pass");
    for (kind, why, expected) in
        [("web", "no tool", "gap refused: web is none of ctl, fake, qa"), ("ctl", " \n", "gap refused: no why")]
    {
        let before = triage.clone();
        assert_eq!(triage.triage_gap(triage_session, kind, why, "qa", 1).expect_err("refused").to_string(), expected);
        assert_eq!(triage, before);
        let before = pass.clone();
        assert_eq!(pass.pass_gap(pass_session, 1, kind, why, "qa", 1).expect_err("refused").to_string(), expected);
        assert_eq!(pass, before);
    }
    let before = pass.clone();
    assert_eq!(
        pass.pass_gap(pass_session, 2, "qa", "missing state", "qa", 1).expect_err("not assigned").to_string(),
        "case 2 is not assigned to pass 1"
    );
    assert_eq!(pass, before);
    assert_eq!(
        pass.triage_gap(pass_session, "qa", "missing state", "qa", 1).expect_err("wrong kind").to_string(),
        "session 2 is a pass session"
    );
    assert_eq!(
        triage.pass_gap(triage_session, 1, "qa", "missing state", "qa", 1).expect_err("wrong kind").to_string(),
        "session 1 is a triage session"
    );
    triage.answer_none(triage_session, "internal").expect("none");
    let before = triage.clone();
    assert_eq!(
        triage.triage_gap(triage_session, "ctl", "no tool", "qa", 1).expect_err("none").to_string(),
        "landing 1 is answered no QA; a gap is refused"
    );
    assert_eq!(triage, before);
}

#[test]
fn old_ledgers_have_no_gaps_and_gap_records_refuse_unknown_fields() {
    let mut ledger = Ledger::default();
    let mut old = ledger.document();
    old.as_object_mut().expect("object").remove("gaps");
    assert_eq!(Ledger::decode(&serde_json::to_vec(&old).expect("JSON")).expect("old ledger"), ledger);
    landing(&mut ledger, 1);
    let session = session(&mut ledger, 1);
    ledger.triage_gap(session, "qa", "no line", "qa", 1).expect("gap");
    assert_eq!(
        ledger.answer_none(session, "internal").expect_err("gap answered").to_string(),
        "landing 1 has gaps; no QA is refused"
    );
    let mut document = ledger.document();
    document["gaps"][0]["future"] = serde_json::json!(true);
    assert_eq!(
        Ledger::decode(&serde_json::to_vec(&document).expect("JSON")).expect_err("unknown").to_string(),
        "ledger.json: unknown field future"
    );
}

#[test]
fn configured_gap_owners_follow_the_kind_and_default_to_owner() {
    use std::path::Path;
    use temper_qa_tool::Configuration;
    let mut document = serde_json::json!({"owner":"qa","git":{"directory":"repo"},"agent":{"command":["agent"],"triage":"low","pass":"high"},"request":{"command":["file"]}});
    let defaults =
        Configuration::decode(&serde_json::to_vec(&document).expect("JSON"), Path::new("home")).expect("config");
    for kind in ["ctl", "fake", "qa"] {
        assert_eq!(defaults.gap_owner(kind), "qa");
    }
    document["areas"] = serde_json::json!({"ctl":"tools"});
    document["gaps"] = serde_json::json!({"fake":"fakes"});
    let config =
        Configuration::decode(&serde_json::to_vec(&document).expect("JSON"), Path::new("home")).expect("config");
    for (kind, owner) in [("ctl", "tools"), ("fake", "fakes"), ("qa", "qa")] {
        assert_eq!(config.gap_owner(kind), owner);
    }
    document["gaps"]["other"] = serde_json::json!("area");
    assert!(
        Configuration::decode(&serde_json::to_vec(&document).expect("JSON"), Path::new("home"))
            .expect_err("unknown")
            .to_string()
            .contains("unknown field other")
    );
}

#[test]
fn gap_listing_cuts_at_two_hundred_bytes_on_character_boundaries() {
    const LIMIT: usize = 200; // gaps.md, 5's per-line byte bound.
    const PREFIX: &str = "1 qa qa open landing 1: ";
    let mut ledger = Ledger::default();
    landing(&mut ledger, 1);
    let session = session(&mut ledger, 1);
    ledger.triage_gap(session, "qa", "missing", "qa", 1).expect("gap");
    for (why, expected) in [
        ("a".repeat(LIMIT - PREFIX.len()), format!("{PREFIX}{}\n", "a".repeat(LIMIT - PREFIX.len()))),
        ("a".repeat(LIMIT - PREFIX.len() + 1), format!("{PREFIX}{}\n", "a".repeat(LIMIT - PREFIX.len()))),
        (
            format!("{}é", "a".repeat(LIMIT - PREFIX.len() - 1)),
            format!("{PREFIX}{}\n", "a".repeat(LIMIT - PREFIX.len() - 1)),
        ),
        ("first\nsecond\rthird".into(), format!("{PREFIX}first second third\n")),
    ] {
        ledger.gaps[0].why = why;
        assert_eq!(ledger.gaps_text(false), expected);
    }
}
