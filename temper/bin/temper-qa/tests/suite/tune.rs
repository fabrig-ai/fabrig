//! Tuning is a locked read through the binary, over ledgers the test writes (qa/ledger.md, 6).
use super::commands::{home, run, success};
use temper_qa_tool::{CaseInput, Exit, SessionKind};

#[test]
fn tune_prints_all_four_figures_excludes_lost_and_unstarted_and_records_nothing() {
    let (_scratch, home) = home();
    let mut config: serde_json::Value =
        serde_json::from_slice(&std::fs::read(home.directory.join("qa.json")).expect("config")).expect("JSON");
    config["bounds"] = serde_json::json!({"build_wait":800});
    std::fs::write(home.directory.join("qa.json"), serde_json::to_vec(&config).expect("JSON")).expect("config");
    home.transact(|ledger| {
        for (page, queued) in [("work", 100), ("home", 200)] {
            let case = CaseInput::decode(
                format!(
                    r#"{{"surface":"web","page":"{page}","setup":"fresh","steps":[{{"do":"open","expect":"ready"}}]}}"#
                )
                .as_bytes(),
                "case",
            )?;
            ledger.add_case(1, &case, queued)?;
        }
        ledger.end_session(1, Exit::Code(0), None, 100, "qa")?;
        let template = ledger.sessions[0].clone();
        ledger.sessions.clear();
        for (kind, duration, exit, tokens) in [
            (SessionKind::Triage, 100, Exit::Code(0), Some(1000)),
            (SessionKind::Triage, 200, Exit::Signal(15), Some(2000)),
            (SessionKind::Triage, 900, Exit::Bound, Some(9000)),
            (SessionKind::Triage, 9999, Exit::Lost, Some(99999)),
            (SessionKind::Triage, 9999, Exit::Unstarted, Some(99999)),
            (SessionKind::Pass, 40, Exit::Code(0), None),
            (SessionKind::Pass, 60, Exit::Code(0), None),
            (SessionKind::Pass, 9999, Exit::Lost, Some(99999)),
            (SessionKind::Pass, 9999, Exit::Unstarted, Some(99999)),
        ] {
            let mut session = template.clone();
            session.number = u64::try_from(ledger.sessions.len()).expect("length") + 1;
            session.kind = kind;
            if kind == SessionKind::Pass {
                session.landing = None;
                session.pass = Some(1);
                session.before =
                    Some(temper_qa_tool::Before { state: None, place: None, build: None, cases: Vec::new() });
            }
            session.ended = Some(duration);
            session.exit = Some(exit);
            session.tokens = tokens;
            ledger.sessions.push(session);
        }
        ledger.keep_build("first", 1000)?;
        ledger.set_build_holds("first", vec![1], 1)?;
        ledger.keep_build("later", 2000)?;
        ledger.set_build_holds("later", vec![1], 2)?;
        Ok(())
    })
    .expect("ledger");
    let before = std::fs::read(&home.ledger).expect("before");
    success(&run(&home, &["tune"], None, None), b"triage: 3 sessions; 1 ended by the bound of 900 s; time median 200 s, longest 900 s; tokens median 2000, most 9000\npass: 2 sessions; 0 ended by the bound of 7200 s; time median 50 s, longest 60 s; tokens none\nbuild_wait: 1 cases waited past 800 s; the longest wait for a kept build 900 s\nplaces: 1; 0 waits for a place in the last 7 days, the longest 0 s, 0 s in all\nsuggest: raise bounds.triage past 900 s: 1 of its last 3 sessions ended by it\n");
    assert_eq!(std::fs::read(&home.ledger).expect("after"), before);
}
