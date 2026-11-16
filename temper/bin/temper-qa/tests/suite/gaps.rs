//! Gap commands and their read-only views (qa/gaps.md, 8).
use super::commands::{home, run, success};
use std::fs;
use temper_qa_tool::{CaseInput, CaseState, Exit, GapClosed, GapState};

#[test]
fn triage_gap_records_owners_and_lists_newest_first() {
    let (_scratch, home) = home();
    let mut config: serde_json::Value =
        serde_json::from_slice(&fs::read(home.directory.join("qa.json")).expect("config")).expect("JSON");
    config["areas"] = serde_json::json!({"ctl":"tools"});
    config["gaps"] = serde_json::json!({"fake":"fakes"});
    fs::write(home.directory.join("qa.json"), serde_json::to_vec(&config).expect("JSON")).expect("config");
    for (kind, number) in [("ctl", 1), ("fake", 2), ("qa", 3)] {
        success(
            &run(&home, &["triage", "gap", kind, "cannot judge"], Some("1"), None),
            format!("gap {number}\n").as_bytes(),
        );
    }
    success(&run(&home, &["gaps"], None, None), b"3 qa qa open landing 1: cannot judge\n2 fake fakes open landing 1: cannot judge\n1 ctl tools open landing 1: cannot judge\n");
    home.transact(|ledger| {
        ledger.gaps[0].state = GapState::Closed;
        ledger.gaps[0].closed = Some(GapClosed { landing: 1, time: 2 });
        ledger.gaps[1].state = GapState::Filed;
        ledger.gaps[1].filed = Some(2);
        Ok(())
    })
    .expect("states for views");
    success(
        &run(&home, &["gaps", "--open"], None, None),
        b"3 qa qa open landing 1: cannot judge\n2 fake fakes filed landing 1: cannot judge\n",
    );
    let show = run(&home, &["show"], None, None);
    assert_eq!(show.status.code(), Some(0));
    assert!(String::from_utf8(show.stdout).expect("show").contains("gaps: 1 open; 1 filed; 1 closed"));
}

#[test]
fn gap_commands_refuse_bad_kind_empty_why_and_wrong_session() {
    let (_scratch, home) = home();
    for (arguments, session, words) in [
        (vec!["triage", "gap", "web", "missing tool"], Some("1"), "gap refused: web is none of ctl, fake, qa"),
        (vec!["triage", "gap", "ctl", "  "], Some("1"), "gap refused: no why"),
        (
            vec!["triage", "gap", "ctl", "missing tool"],
            None,
            "TEMPER_INTERNAL_QA_SESSION is not set: triage commands run in a triage session",
        ),
        (vec!["pass", "gap", "1", "ctl", "missing tool"], Some("1"), "session 1 is a triage session"),
    ] {
        let before = fs::read(&home.ledger).expect("before");
        let output = run(&home, &arguments, session, None);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stderr, format!("temper-qa: {words}\n").as_bytes());
        assert_eq!(fs::read(&home.ledger).expect("after"), before);
    }
}

fn pass(home: &temper_qa_tool::Home) {
    home.transact(|ledger| {
        let input = CaseInput::decode(
            br#"{"surface":"web","page":"work","setup":"fresh","steps":[{"do":"open Work","expect":"Drafts"}]}"#,
            "-",
        )?;
        ledger.add_case(1, &input, 1)?;
        ledger.end_session(1, Exit::Code(0), None, 2, "qa")?;
        ledger.keep_build("build", 3)?;
        ledger.set_build_holds("build", vec![1], 1)?;
        ledger.start_pass("places/1", &["web".into()], "high", 4).expect("pass");
        Ok(())
    })
    .expect("assigned case");
}

#[test]
fn pass_gap_refuses_without_mutation_and_records_its_case() {
    let (_scratch, home) = home();
    pass(&home);
    for (case, kind, why, words) in [
        ("1", "web", "missing", "gap refused: web is none of ctl, fake, qa"),
        ("1", "ctl", "", "gap refused: no why"),
        ("2", "ctl", "missing", "case 2 is not assigned to pass 1"),
    ] {
        let before = fs::read(&home.ledger).expect("before");
        let output = run(&home, &["pass", "gap", case, kind, why], Some("2"), None);
        assert_eq!(output.status.code(), Some(1));
        assert_eq!(output.stderr, format!("temper-qa: {words}\n").as_bytes());
        assert_eq!(fs::read(&home.ledger).expect("after"), before);
    }
    success(&run(&home, &["pass", "gap", "1", "qa", "no setup reaches the branch"], Some("2"), None), b"gap 1\n");
    home.read(|ledger| {
        assert_eq!(ledger.cases[0].state, CaseState::Gap);
        assert_eq!(ledger.gaps[0].case, Some(1));
        assert_eq!(ledger.gaps[0].owner, "qa");
        Ok(())
    })
    .expect("recorded");
    success(&run(&home, &["gaps"], None, None), b"1 qa qa open landing 1 case 1: no setup reaches the branch\n");
}
