//! Tuning is a locked read through the binary, over ledgers the test writes (qa/ledger.md, 6).
use super::commands::{home, run, success};
use temper_qa_tool::{
    CaseInput, Exit, Ledger, PlaceWait, SessionKind,
    faces::{ClockFace, RealClock},
};

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

#[test]
fn tune_counts_open_waits_to_now_and_clips_them_to_seven_days() {
    let (_scratch, home) = home();
    let before = RealClock::default().now().wall;
    home.transact(|ledger| {
        let case = CaseInput::decode(
            br#"{"surface":"web","page":"work","setup":"fresh","steps":[{"do":"open","expect":"ready"}]}"#,
            "case",
        )?;
        ledger.add_case(1, &case, before - 1200)?;
        ledger.place_waits = vec![
            PlaceWait { from: before - 8 * 86400, to: None },
            PlaceWait { from: before - 10 * 86400, to: Some(before - 9 * 86400) },
        ];
        Ok(())
    })
    .expect("open waits");
    let bytes = std::fs::read(&home.ledger).expect("before");
    let output = run(&home, &["tune"], None, None);
    let after = RealClock::default().now().wall;
    assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
    let text = String::from_utf8(output.stdout).expect("text");
    assert!(
        text.contains("places: 1; 1 waits for a place in the last 7 days, the longest 604800 s, 604800 s in all"),
        "{text}"
    );
    let build = text.lines().find(|line| line.starts_with("build_wait:")).expect("build figures");
    let wait: u64 = build.split_whitespace().rev().nth(1).expect("longest").parse().expect("seconds");
    assert!((1200..=1200 + after - before).contains(&wait), "{build}");
    assert_eq!(std::fs::read(&home.ledger).expect("after"), bytes);
}

#[test]
fn tune_suggests_a_place_at_an_hour_but_not_a_second_before_and_only_recent_bounds() {
    let (_scratch, home) = home();
    let now = RealClock::default().now().wall;
    home.transact(|ledger| {
        ledger.end_session(1, Exit::Bound, None, 900, "qa")?;
        let template = ledger.sessions[0].clone();
        for number in 2..=21 {
            let mut session = template.clone();
            session.number = number;
            session.exit = Some(Exit::Code(0));
            ledger.sessions.push(session);
        }
        ledger.place_waits = vec![
            PlaceWait { from: now - 10000, to: Some(now - 8200) },
            PlaceWait { from: now - 8000, to: Some(now - 6200) },
            PlaceWait { from: now - 9 * 86400, to: Some(now - 8 * 86400) },
        ];
        Ok(())
    })
    .expect("ledger");
    for (seconds, suggest) in [(3600, true), (3599, false)] {
        home.transact(|ledger| {
            ledger.place_waits[1].to = Some(now - 8000 + seconds - 1800);
            Ok(())
        })
        .expect("wait boundary");
        let before = std::fs::read(&home.ledger).expect("before");
        let output = run(&home, &["tune"], None, None);
        assert_eq!(output.status.code(), Some(0), "{}", String::from_utf8_lossy(&output.stderr));
        let text = String::from_utf8(output.stdout).expect("text");
        assert!(
            text.contains(&format!(
                "places: 1; 2 waits for a place in the last 7 days, the longest 1800 s, {seconds} s in all\n"
            )),
            "{text}"
        );
        assert_eq!(text.contains("suggest: another place:"), suggest, "{text}");
        assert!(!text.contains("suggest: raise"), "bound is not in the last 20 sessions: {text}");
        assert_eq!(text.lines().count(), if suggest { 5 } else { 4 });
        assert_eq!(std::fs::read(&home.ledger).expect("after"), before);
        Ledger::decode(&before).expect("readable ledger");
    }
}
