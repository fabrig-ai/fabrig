//! Each operation holds the ledger's rules (qa/ledger.md, 4).
use super::{case, landing, session};
use serde_json::json;
use temper_qa_tool::{Answer, BuildState, CaseInput, Exit, Ledger};

#[test]
fn triage_starts_only_the_next_landing_and_never_overlaps() {
    let mut ledger = Ledger::default();
    landing(&mut ledger, 1);
    landing(&mut ledger, 2);
    assert_eq!(ledger.next_landing().expect("next").number, 1);
    assert!(ledger.start_session(2, "low", super::pages(), vec!["fresh".into()], 0).is_err());
    let first = session(&mut ledger, 1);
    assert!(ledger.start_session(1, "low", super::pages(), vec!["fresh".into()], 0).is_err());
    ledger.add_case(first, &case(), 1).expect("first case");
    assert_eq!(ledger.next_landing().expect("not triaged until end").number, 1);
    ledger.end_session(first, Exit::Code(0), None, 2, "qa").expect("end");
    assert_eq!(ledger.next_landing().expect("second").number, 2);
    session(&mut ledger, 2);
}

#[test]
fn decoded_out_of_order_and_ended_sessions_refuse_both_commands() {
    let mut ledger = Ledger::default();
    landing(&mut ledger, 1);
    landing(&mut ledger, 2);
    let number = session(&mut ledger, 1);
    let mut value = ledger.document();
    value["sessions"][0]["landing"] = json!(2);
    let mut ledger = Ledger::decode(&serde_json::to_vec(&value).expect("JSON")).expect("decoded session");
    let before = ledger.clone();
    let words = "session 1 triages landing 2, which is not next: landing 1 is not triaged";
    assert_eq!(ledger.answer_none(number, "nothing").expect_err("out of order").to_string(), words);
    assert_eq!(ledger.add_case(number, &case(), 1).expect_err("out of order").to_string(), words);
    assert_eq!(ledger, before);
    ledger.sessions[0].ended = Some(1);
    assert_eq!(ledger.answer_none(number, "nothing").expect_err("ended").to_string(), "session 1 is not running");
    assert_eq!(ledger.add_case(999, &case(), 1).expect_err("unknown").to_string(), "session 999 is not running");
}

#[test]
fn cases_fit_the_session_and_refuse_unknown_fields_without_mutation() {
    let mut ledger = Ledger::default();
    landing(&mut ledger, 1);
    let number = session(&mut ledger, 1);
    let good = json!({"surface":"web","page":"work","setup":"fresh","steps":[{"do":"open Work","expect":"Drafts lists the draft"}]});
    for (pointer, replacement, words) in [
        ("/surface", json!(null), "no surface"),
        ("/surface", json!("native"), "surface native is neither web nor tui"),
        ("/page", json!(""), "no page"),
        ("/page", json!("chat"), "chat is no web page; pages: work, home"),
        ("/setup", json!(""), "no setup"),
        ("/setup", json!("busy"), "setup busy is not in the catalogue; setups: fresh"),
        ("/steps", json!([]), "no step"),
        ("/steps/0/do", json!(""), "step 1 says nothing to do"),
        ("/steps/0/expect", json!(null), "step 1 has no expectation"),
    ] {
        let mut value = good.clone();
        *value.pointer_mut(pointer).expect("field") = replacement;
        let before = ledger.clone();
        let result = CaseInput::decode(&serde_json::to_vec(&value).expect("JSON"), "case.json")
            .and_then(|input| ledger.add_case(number, &input, 1));
        assert_eq!(result.expect_err("invalid case").to_string(), format!("case refused: {words}"), "{pointer}");
        assert_eq!(ledger, before);
    }
    for pointer in ["", "/steps/0"] {
        let mut value = good.clone();
        value
            .pointer_mut(pointer)
            .expect("object")
            .as_object_mut()
            .expect("object")
            .insert("surprise".into(), json!(true));
        assert_eq!(
            CaseInput::decode(&serde_json::to_vec(&value).expect("JSON"), "-").expect_err("unknown field").to_string(),
            "case refused: unknown field surprise"
        );
    }
    let mut input = case();
    input.steps[0].expect.clear();
    assert_eq!(
        ledger.add_case(number, &input, 1).expect_err("in-memory check").to_string(),
        "case refused: step 1 has no expectation"
    );
}

#[test]
fn one_answer_kind_and_nonempty_why_and_repeated_none_replaces_why() {
    let mut ledger = Ledger::default();
    landing(&mut ledger, 1);
    let number = session(&mut ledger, 1);
    assert_eq!(ledger.answer_none(number, " \n").expect_err("empty why").to_string(), "no QA needs a line saying why");
    ledger.answer_none(number, "first").expect("none");
    ledger.answer_none(number, "second").expect("replace");
    assert_eq!(ledger.landings[0].answer, Some(Answer::None { why: "second".into() }));
    let before = ledger.clone();
    assert_eq!(
        ledger.add_case(number, &case(), 1).expect_err("none already").to_string(),
        "landing 1 is answered no QA; a case is refused"
    );
    assert_eq!(ledger, before);
    ledger.end_session(number, Exit::Lost, None, 2, "qa").expect("end");
    landing(&mut ledger, 2);
    let number = session(&mut ledger, 2);
    ledger.add_case(number, &case(), 3).expect("case");
    let before = ledger.clone();
    assert_eq!(
        ledger.answer_none(number, "nothing").expect_err("case already").to_string(),
        "landing 2 has cases; no QA is refused"
    );
    assert_eq!(ledger, before);
}

#[test]
fn a_queued_page_joins_asks_and_keeps_unique_landings_in_order() {
    let mut ledger = Ledger::default();
    landing(&mut ledger, 1);
    landing(&mut ledger, 2);
    let first = session(&mut ledger, 1);
    assert_eq!(ledger.add_case(first, &case(), 10).expect("queue").to_string(), "case 1 queued");
    ledger.end_session(first, Exit::Signal(15), Some(20), 12, "qa").expect("end");
    let second = session(&mut ledger, 2);
    assert_eq!(ledger.add_case(second, &case(), 13).expect("join").to_string(), "case 1 joined: landings 1, 2");
    assert_eq!(ledger.cases.len(), 1);
    assert_eq!(ledger.cases[0].asks.len(), 2);
    assert_eq!(ledger.cases[0].landings(), [1, 2]);
    assert_eq!(ledger.cases[0].queued, 10);
    assert_eq!(ledger.add_case(second, &case(), 14).expect("repeat ask").to_string(), "case 1 joined: landings 1, 2");
    assert_eq!(ledger.cases[0].asks.len(), 3);
    assert_eq!(ledger.cases[0].landings(), [1, 2]);
    let mut other = case();
    other.surface = "tui".into();
    other.page = "chat".into();
    assert_eq!(ledger.add_case(second, &other, 14).expect("other page").to_string(), "case 2 queued");
}

#[test]
fn ending_counts_unanswered_twice_but_not_unstarted_and_answers_stand() {
    let mut ledger = Ledger::default();
    landing(&mut ledger, 1);
    let number = session(&mut ledger, 1);
    ledger.end_session(number, Exit::Unstarted, None, 1, "owner").expect("unstarted");
    assert_eq!(ledger.landings[0].sessions, 0);
    assert_eq!(ledger.landings[0].triaged, None);
    let number = session(&mut ledger, 1);
    ledger.end_session(number, Exit::Bound, Some(17), 2, "owner").expect("first unanswered");
    assert_eq!(ledger.landings[0].sessions, 1);
    assert_eq!(ledger.landings[0].triaged, None);
    assert!(ledger.lines.is_empty());
    let number = session(&mut ledger, 1);
    ledger.end_session(number, Exit::Lost, None, 3, "owner").expect("second unanswered");
    assert_eq!(ledger.landings[0].triaged, Some(3));
    assert!(ledger.landings[0].unanswered);
    assert_eq!(
        ledger.lines[0].words,
        "landing 1 (goal 11, 000000000000): triage recorded nothing in two sessions; triaged as unanswered"
    );
    assert_eq!(ledger.lines[0].area, "owner");
    assert_eq!(ledger.lines[0].time, 3);
    landing(&mut ledger, 2);
    let number = session(&mut ledger, 2);
    ledger.answer_none(number, "no surface changed").expect("answer");
    ledger.end_session(number, Exit::Code(1), Some(40), 4, "owner").expect("answered even on failure");
    assert_eq!(ledger.landings[1].triaged, Some(4));
    assert_eq!(ledger.landings[1].sessions, 0);
    assert_eq!(ledger.sessions.last().expect("session").tokens, Some(40));
    assert!(ledger.running_sessions().is_empty());
}

fn deletable(ledger: &Ledger) -> Vec<&str> {
    ledger.deletable_builds().iter().map(|build| build.commit.as_str()).collect()
}
#[test]
fn deletion_keeps_unreached_held_and_newest_reached_case_builds() {
    let mut ledger = Ledger::default();
    landing(&mut ledger, 1);
    ledger.keep_build("B1", 0).expect("build");
    ledger.set_build_holds("B1", vec![1], 1).expect("holds");
    assert!(deletable(&ledger).is_empty());
    let number = session(&mut ledger, 1);
    ledger.add_case(number, &case(), 1).expect("case");
    ledger.end_session(number, Exit::Code(0), None, 2, "qa").expect("triaged");
    assert!(deletable(&ledger).is_empty());
    landing(&mut ledger, 2);
    ledger.keep_build("B2", 3).expect("newer");
    ledger.set_build_holds("B2", vec![1, 2], 2).expect("holds");
    assert!(deletable(&ledger).is_empty(), "unreached B2 cannot replace B1");
    let number = session(&mut ledger, 2);
    ledger.answer_none(number, "no change").expect("none");
    ledger.end_session(number, Exit::Code(0), None, 4, "qa").expect("triaged");
    assert_eq!(deletable(&ledger), ["B1"]);
    let mut document = ledger.document();
    document["builds"][0]["held"] = json!(42);
    let held = Ledger::decode(&serde_json::to_vec(&document).expect("JSON")).expect("held decoded");
    assert!(deletable(&held).is_empty());
    ledger.keep_build("unknown", 4).expect("unknown holds");
    assert_eq!(ledger.builds_without_holds().len(), 1);
    assert_eq!(deletable(&ledger), ["B1"]);
    assert!(ledger.mark_build_deleted("B2", 5).is_err());
    ledger.mark_build_deleted("B1", 5).expect("delete");
    assert_eq!(ledger.builds[0].state, BuildState::Deleted);
    assert_eq!(ledger.builds[0].deleted, Some(5));
    let mut no_cases = Ledger::default();
    landing(&mut no_cases, 1);
    no_cases.keep_build("B", 0).expect("build");
    no_cases.set_build_holds("B", vec![1], 1).expect("holds");
    let number = session(&mut no_cases, 1);
    no_cases.answer_none(number, "only internals").expect("none");
    no_cases.end_session(number, Exit::Code(0), None, 1, "qa").expect("end");
    assert_eq!(deletable(&no_cases), ["B"]);
}

#[test]
fn a_waiting_case_lines_at_the_bound_once_and_not_when_a_reached_build_holds_it() {
    let mut ledger = Ledger::default();
    landing(&mut ledger, 1);
    let number = session(&mut ledger, 1);
    ledger.add_case(number, &case(), 10).expect("case");
    ledger.end_session(number, Exit::Code(0), None, 11, "qa").expect("end");
    assert!(ledger.waiting_lines(20, 29, "owner").is_empty());
    let mut held = ledger.clone();
    held.keep_build("B", 15).expect("build");
    held.set_build_holds("B", vec![1], 1).expect("holds");
    assert!(held.waiting_lines(20, 30, "owner").is_empty());
    assert_eq!(ledger.waiting_lines(20, 30, "owner"), [1]);
    assert_eq!(
        ledger.lines[0].words,
        "case 1 (work on web; landings 1) has waited past 20 seconds for a kept build holding its landings"
    );
    assert_eq!(ledger.ungiven_lines().len(), 1);
    ledger.mark_line_given(1).expect("given");
    assert!(ledger.ungiven_lines().is_empty());
    assert!(ledger.waiting_lines(20, 31, "owner").is_empty());
    assert!(ledger.cases[0].line);
}

#[test]
fn documents_round_trip_all_fields_and_duplicate_commits_record_nothing() {
    let mut ledger = Ledger::default();
    ledger.record_head("main");
    landing(&mut ledger, 1);
    assert_eq!(ledger.record_landing(&format!("{:040x}", 1), 99, Some(33), "other", 5), None);
    let number = session(&mut ledger, 1);
    let mut input = case();
    input.adds = "a draft".into();
    input.steps[0].see = Some("Drafts".into());
    ledger.add_case(number, &input, 1).expect("case");
    ledger.end_session(number, Exit::Signal(15), Some(30), 2, "qa").expect("end");
    ledger.keep_build("B", 3).expect("build");
    ledger.set_build_holds("B", vec![1], 1).expect("holds");
    ledger.add_line("qa", "a line", 4);
    ledger.mark_line_given(1).expect("given");
    assert_eq!(Ledger::decode(&ledger.encode().expect("encode")).expect("decode"), ledger);
    let mut value = ledger.document();
    value["future"] = json!(true);
    assert_eq!(
        Ledger::decode(&serde_json::to_vec(&value).expect("JSON")).expect_err("unknown document").to_string(),
        "ledger.json: unknown field future"
    );
    assert!(Ledger::decode(b"broken").expect_err("bad JSON").to_string().starts_with("ledger.json: "));
}
