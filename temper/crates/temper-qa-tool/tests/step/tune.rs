//! Tuning over ledger values and supplied time (temper/docs/design/testing/qa/ledger.md, 6).
use std::path::Path;
use temper_qa_tool::{
    Before, Configuration, Exit, Ledger, PlaceWait, SessionKind,
    testing::{case, landing, session},
};

const DAY: u64 = 86_400;
const NOW: u64 = 14 * DAY;
const LANDING: u64 = 1;

fn configuration() -> Configuration {
    Configuration::decode(
        br#"{"owner":"qa","git":{"directory":"repo"},"agent":{"command":["agent"],"triage":"low","pass":"high"},"request":{"command":["file"]}}"#,
        Path::new("/qa"),
    )
    .expect("configuration")
}

fn triage() -> (Ledger, u64) {
    let mut ledger = Ledger::default();
    landing(&mut ledger, LANDING);
    let triage = session(&mut ledger, LANDING);
    (ledger, triage)
}

fn tune(ledger: &Ledger, configuration: &Configuration) -> String {
    let before = ledger.encode().expect("ledger before tune");
    let text = ledger.tune(configuration, NOW);
    assert_eq!(ledger.encode().expect("ledger after tune"), before);
    text
}

#[test]
fn tune_prints_all_four_figures_excludes_lost_and_unstarted_and_records_nothing() {
    let mut configuration = configuration();
    configuration.bounds.build_wait = 800;
    let (mut ledger, triage) = triage();
    for (page, queued) in [("work", 100), ("home", 200)] {
        let mut input = case();
        input.page = page.into();
        ledger.add_case(triage, &input, queued).expect("case");
    }
    ledger.end_session(triage, Exit::Code(0), None, 100, "qa").expect("triage end");
    let template = ledger.sessions.pop().expect("triage session");
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
            session.before = Some(Before { state: None, place: None, build: None, cases: Vec::new() });
        }
        session.ended = Some(session.started + duration);
        session.exit = Some(exit);
        session.tokens = tokens;
        ledger.sessions.push(session);
    }
    // The earliest build need not be first in the ledger.
    for (commit, kept, order) in [("later", 2000, 2), ("first", 1000, 1)] {
        ledger.keep_build(commit, kept).expect("build");
        ledger.set_build_holds(commit, vec![LANDING], order).expect("holds");
    }
    assert_eq!(
        tune(&ledger, &configuration),
        concat!(
            "triage: 3 sessions; 1 ended by the bound of 900 s; time median 200 s, longest 900 s; tokens median 2000, most 9000\n",
            "pass: 2 sessions; 0 ended by the bound of 7200 s; time median 50 s, longest 60 s; tokens none\n",
            "build_wait: 1 cases waited past 800 s; the longest wait for a kept build 900 s\n",
            "places: 1; 0 waits for a place in the last 7 days, the longest 0 s, 0 s in all\n",
            "suggest: raise bounds.triage past 900 s: 1 of its last 3 sessions ended by it\n",
        ),
    );
}

#[test]
fn tune_counts_open_waits_to_now_and_clips_them_to_seven_days() {
    let (mut ledger, triage) = triage();
    ledger.add_case(triage, &case(), NOW - 1200).expect("case");
    ledger.place_waits =
        vec![PlaceWait { from: NOW - 8 * DAY, to: None }, PlaceWait { from: NOW - 10 * DAY, to: Some(NOW - 9 * DAY) }];

    let text = tune(&ledger, &configuration());
    assert!(
        text.contains("places: 1; 1 waits for a place in the last 7 days, the longest 604800 s, 604800 s in all\n"),
        "{text}"
    );
    assert!(
        text.contains("build_wait: 0 cases waited past 21600 s; the longest wait for a kept build 1200 s\n"),
        "{text}"
    );
}

#[test]
fn tune_suggests_a_place_at_an_hour_but_not_a_second_before_and_only_recent_bounds() {
    let (mut ledger, triage) = triage();
    ledger.end_session(triage, Exit::Bound, None, 900, "qa").expect("triage end");
    let template = ledger.sessions.first().expect("triage session").clone();
    // Twenty newer sessions leave the bound outside the suggestion's window.
    for number in 2..=21 {
        ledger.sessions.push(temper_qa_tool::Session { number, exit: Some(Exit::Code(0)), ..template.clone() });
    }
    let configuration = configuration();
    for (seconds, suggest, lines) in [(3600, true, 5), (3599, false, 4)] {
        ledger.place_waits = vec![
            PlaceWait { from: NOW - 10000, to: Some(NOW - 8200) },
            PlaceWait { from: NOW - 8000, to: Some(NOW - 8000 + seconds - 1800) },
            PlaceWait { from: NOW - 9 * DAY, to: Some(NOW - 8 * DAY) },
        ];
        let text = tune(&ledger, &configuration);
        assert!(
            text.contains(&format!(
                "places: 1; 2 waits for a place in the last 7 days, the longest 1800 s, {seconds} s in all\n"
            )),
            "{text}"
        );
        assert_eq!(text.contains("suggest: another place:"), suggest, "{text}");
        assert!(!text.contains("suggest: raise"), "bound is not in the last 20 sessions: {text}");
        assert_eq!(text.lines().count(), lines);
        Ledger::decode(&ledger.encode().expect("ledger bytes")).expect("readable ledger");
    }
}
