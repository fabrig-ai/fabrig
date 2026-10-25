pub mod support;

use serde_json::{Value, json};
use support::forge_land::{last_readme_line, require_chat_progress};

#[test]
fn a_readme_ignores_trailing_empty_lines() {
    for (readme, expected) in [
        ("starter\ntag\n\n", Some("tag")),
        ("tag", Some("tag")),
        ("tag\n", Some("tag")),
        ("tag\n\n\n", Some("tag")),
        ("tag\r\n", Some("tag")),
        ("tag\r\n\r\n", Some("tag")),
        ("", None),
        ("\n", None),
        ("\n\n", None),
        ("\r\n", None),
    ] {
        assert_eq!(last_readme_line(readme.as_bytes()), expected, "README {readme:?}");
    }
}

#[test]
fn a_readme_keeps_its_last_nonempty_line_exactly() {
    for (readme, expected) in [
        ("tag\nother\n\n", "other"),
        ("tag \n\n", "tag "),
        (" tag\n", " tag"),
        ("tag\n  \n", "  "),
        ("tag\n\t\n\n", "\t"),
    ] {
        assert_eq!(last_readme_line(readme.as_bytes()), Some(expected), "README {readme:?}");
    }
}

// The facts read by discovery, in task --json's owner shape. The encoders live
// in temper-client-tools/src/jig/render/document/{chips,owner_chips,
// owner_authority,tasks,parts,llm}.rs.
fn empty_park() -> Value {
    json!({
        "chip": {"details": {"owner": {
            "phase": {"active": "idle"},
            "details": {"attempt": 1, "live_delegates": 0}
        }}},
        "waits_on": "news",
        "run_wait": null,
        "subscriptions": {"entries": []},
        "cut": false,
        "wake": {
            "words": "immediate", "notices": "immediate", "news": "immediate",
            "results": "last_or_failure", "questions": true, "answers": true, "timers": true
        }
    })
}

#[test]
fn an_empty_park_stops_the_story() {
    assert_eq!(require_chat_progress(&empty_park()), Err("chat parked without delegating a change".to_owned()));
}

fn continues_with(pointer: &str, replacement: Value) {
    let mut task = empty_park();
    *task.pointer_mut(pointer).expect("fixture field") = replacement;
    assert_eq!(require_chat_progress(&task), Ok(()), "{pointer}: {task}");
}

#[test]
fn active_work_and_intentional_waits_continue() {
    continues_with("/chip/details/owner/details/attempt", Value::Null);
    continues_with("/chip/details/owner/details/live_delegates", json!(1));
    for stage in [
        json!("due"),
        json!("preparing"),
        json!({"claimed": {"attempt": 1}}),
        json!({"running": {"attempt": 1}}),
        json!({"backing_off": {"until": 10}}),
    ] {
        continues_with("/chip/details/owner/phase/active", stage);
    }
    let mut running = empty_park();
    running["chip"]["details"]["owner"]["phase"] = json!({"active": {"running": {"attempt": 1}}});
    for work in [
        json!("preparing"),
        json!("starting"),
        json!("active"),
        json!("writing"),
        json!({"tool": {"call": "01", "tool": "02"}}),
        json!("waiting"),
        json!("ending"),
    ] {
        running["chip"]["details"]["owner"]["details"]["observation"] =
            json!({"channel": 1, "work": work, "since": 1, "reported_at": 2, "started": 1});
        assert_eq!(require_chat_progress(&running), Ok(()), "{running}");
    }
    continues_with(
        "/chip/details/owner/phase",
        json!({"waiting": {"dependencies": [2], "more": false, "holds": false}}),
    );
    continues_with(
        "/chip/details/owner/phase",
        json!({"person": {"address": {"party": 1}, "taken_by": null, "since": null}}),
    );
    for waits in [
        json!("nothing"),
        json!("delegates"),
        json!({"dependencies": {"pending": [2], "count": 1}}),
        json!({"party": {"whom": {"person": 1}, "entries": 1, "task": 2, "kind": "question"}}),
    ] {
        continues_with("/waits_on", waits);
    }
    for wait in
        [json!("credential"), json!("host"), json!({"account": {"account": 1, "source": null, "why": "refreshing"}})]
    {
        continues_with("/run_wait", wait);
    }
    for kind in [
        json!({"timer": {"at": 10, "period": null}}),
        json!({"task": {"target": 2, "held": true, "result": true}}),
        json!({"topic": {"connector": 1, "topic": 2}}),
    ] {
        continues_with("/subscriptions/entries", json!([{"number": 1, "kind": kind}]));
    }
    continues_with("/cut", json!(true));
    for key in ["words", "notices", "news"] {
        for rule in [json!("never"), json!({"batch": {"count": 2, "age": 10}}), json!("unknown")] {
            continues_with(&format!("/wake/{key}"), rule);
        }
    }
    for rule in ["never", "each", "unknown"] {
        continues_with("/wake/results", json!(rule));
    }
    for key in ["questions", "answers", "timers"] {
        continues_with(&format!("/wake/{key}"), json!(false));
    }
    for key in ["words", "notices", "news", "results", "questions", "answers", "timers"] {
        let mut task = empty_park();
        task["wake"].as_object_mut().expect("wake object").remove(key);
        assert_eq!(require_chat_progress(&task), Ok(()), "missing wake {key}");
    }
    let mut task = empty_park();
    task["wake"]["future"] = json!(true);
    assert_eq!(require_chat_progress(&task), Ok(()), "unknown wake field");
    continues_with("/wake", Value::Null);
}

#[test]
fn a_terminal_chat_is_not_an_empty_park() {
    for phase in [
        json!({"held": {"reason": "drift", "revision": 1}}),
        json!({"closing": {"run": {"attempt": 1}}}),
        json!({"closing": "delegates"}),
        json!({"closing": "effects"}),
        json!({"closing": "releases"}),
        json!({"closing": "settled"}),
        json!("ended"),
    ] {
        continues_with("/chip/details/owner/phase", phase);
    }
}

#[test]
fn missing_chat_facts_report_the_shape_error() {
    for (parent, key) in [
        ("", "chip"),
        ("/chip", "details"),
        ("/chip/details", "owner"),
        ("/chip/details/owner", "phase"),
        ("/chip/details/owner", "details"),
        ("/chip/details/owner/details", "attempt"),
        ("/chip/details/owner/details", "live_delegates"),
        ("", "waits_on"),
        ("", "run_wait"),
        ("", "subscriptions"),
        ("/subscriptions", "entries"),
        ("", "cut"),
        ("", "wake"),
    ] {
        let mut task = empty_park();
        task.pointer_mut(parent).expect("fixture parent").as_object_mut().expect("object").remove(key);
        assert_eq!(require_chat_progress(&task), Err("chat progress facts missing".to_owned()), "{parent}/{key}");
    }
    for (pointer, malformed) in [
        ("/chip/details/owner/details/attempt", json!("1")),
        ("/chip/details/owner/details/attempt", json!(-1)),
        ("/chip/details/owner/details/live_delegates", Value::Null),
        ("/chip/details/owner/details/live_delegates", json!("0")),
        ("/subscriptions/entries", Value::Null),
        ("/cut", Value::Null),
    ] {
        let mut task = empty_park();
        *task.pointer_mut(pointer).expect("fixture field") = malformed;
        assert_eq!(require_chat_progress(&task), Err("chat progress facts missing".to_owned()), "{pointer}");
    }
}
