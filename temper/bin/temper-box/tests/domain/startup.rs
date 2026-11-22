//! Harness parsers, configuration naming and embedded bundle values.
use crate::support;
use support::bundle_fixture::{BUNDLE_BYTES, BUNDLE_COUNT, BUNDLE_FILES, BUNDLE_LIMITS, PAGE, PATH_STEM_BOUND};

#[test]
fn a_rung_reads_created_chats_waited_changes_and_both_child_sets_in_ctls_forms() {
    use serde_json::json;
    use support::forge_land::rung::{child_rows, created_chat, plan_question, task_digest, waited};
    // The chat-start document supplies the created range; the top-level task
    // names an operation target and is null when creating a chat.
    let created = json!({"operation":"chat","task":null,"to":null,
        "outcome":{"done":{"document":{"created":{"first":7,"last":7,"given":{"task":7,"amount":207_552}}}}}});
    assert_eq!(created_chat(&created), Ok(7));
    assert!(created_chat(&json!({"task":7})).is_err(), "an operation target is not creation evidence");
    let panel = json!({"task":3,"pull":{"number":2},"head":"abc","gates":[]});
    let change = json!({"change":panel,"met":true});
    assert_eq!(waited(&["change", "show", "3"], change), Ok(panel));
    assert!(waited(&["change", "show", "3"], json!({"met":true})).is_err());
    assert!(waited(&["change", "show", "3"], json!({"met":false})).is_err());
    let task = json!({"task":{"chip":{"task":2},"digest":"abc"},"met":true});
    assert_eq!(waited(&["task", "show", "2"], task.clone()), Ok(task));
    assert_eq!(task_digest(&json!({"chip":{"task":2},"digest":"plain"})), Ok("plain"));
    assert_eq!(task_digest(&json!({"task":{"digest":"compound"},"tree":{}})), Ok("compound"));
    assert!(task_digest(&json!({"chip":{"task":2}})).is_err());
    let accounts = json!({"accounts":{"party":[]},"met":true});
    assert_eq!(waited(&["account", "list"], accounts.clone()), Ok(accounts));
    let live = json!({"tasks":{"rows":[{"chip":{"task":3}}],"more":false,"digest":"abc"},"read_on":null});
    let ended = json!({"live":[],"live_after":null,
        "ended":{"rows":{"rows":[{"chip":{"task":4}}],"after":null,"cut":false}}});
    let rows = child_rows(&live, &ended).expect("both sets");
    assert_eq!(rows, vec![json!({"chip":{"task":3}}), json!({"chip":{"task":4}})]);
    assert_eq!(child_rows(&live, &json!({"ended":"none"})).expect("no ended children").len(), 1);
    let mut cut = ended.clone();
    cut["ended"]["rows"]["cut"] = true.into();
    assert!(child_rows(&live, &cut).is_err(), "a partial set cannot prove no change");
    let mut more = live;
    more["tasks"]["more"] = true.into();
    assert!(child_rows(&more, &ended).is_err(), "an unread live child cannot disappear");
    let question = json!({"event":{"owner":{"source":{"line":{"words":{
        "kind":"question","by":{"task":2},"message":7,"words":"May I go ahead with this plan?"
    }}}}}});
    assert_eq!(plan_question(std::slice::from_ref(&question), 2, 7), Ok("May I go ahead with this plan?"));
    assert!(plan_question(std::slice::from_ref(&question), 3, 7).is_err(), "actual author, not just any question");
    assert!(plan_question(std::slice::from_ref(&question), 2, 8).is_err(), "the exact question in the person's inbox");
}

#[test]
fn a_rung_recognizes_the_exact_delivered_approval_in_explicit_message_activity() {
    use serde_json::json;
    use support::forge_land::rung::approval_at;
    let mut delivered = json!({"position":9,"at":42,"event":{"owner":{"source":{"line":{"words":{
        "message":3,"by":{"task":1},"kind":{"answer":{"question":1}},
        "words":"Go ahead. The CSV's first line must be the header fruit,count."
    }}}}}});
    assert_eq!(approval_at(std::slice::from_ref(&delivered), 1), Some(42));
    assert_eq!(approval_at(std::slice::from_ref(&delivered), 2), None, "the chat must send it");
    delivered["event"]["owner"]["source"]["line"]["words"]["words"] = "Go ahead.".into();
    assert_eq!(approval_at(&[delivered], 1), None, "the header condition must reach the work too");
}

#[test]
fn a_rungs_setup_keeps_the_final_json_answer_after_progress_and_refuses_a_broken_stream() {
    use support::forge_land::rung::ctl_json;
    assert_eq!(ctl_json(b"{\"setting_up\":3}\n{\"project\":2}\n"), Ok(serde_json::json!({"project":2})));
    assert!(ctl_json(b"{\"setting_up\":3}\nnot JSON\n{\"project\":2}").is_err());
    assert!(ctl_json(b"").is_err());
}

#[test]
fn the_owned_example_is_named_without_a_problem() {
    // Only naming a fixture document; no deployment or files are opened.
    let named = temper_service::named(support::owned::EXAMPLE, &jig_protocol_config::Directories::working())
        .expect("owned example named");
    assert_eq!(named.worker.as_ref().expect("standalone worker").agent.as_ref(), b"agent.json");
    assert_eq!(named.worker.as_ref().expect("standalone worker").root.as_ref(), b"workspaces");
    assert_eq!(named.deployment.as_ref(), b"state");
    for path in [b"forge-1/api_token".as_slice(), b"forge-1/user", b"forge-1/webhook_secret", b"forge-1/git_token"] {
        assert_eq!(
            named.files.iter().find(|file| file.path.as_ref() == path).map(|file| file.private),
            None,
            "records-only owned input names no file: {path:?}"
        );
    }
}

#[test]
fn embedded_files_refuse_each_spoiled_bundle_and_limit() {
    use temper_service::limits::Clients;
    let long = format!("0123456789abcdef/{}.js", "a".repeat(PATH_STEM_BOUND + 1)); // 1025 bytes below the digest.
    let rows = [
        ("no page", BUNDLE_FILES[1..].to_vec(), BUNDLE_LIMITS, "index.html: absent".to_owned()),
        ("no digest", BUNDLE_FILES[..1].to_vec(), BUNDLE_LIMITS, "no digest directory".to_owned()),
        (
            "two digests",
            vec![("index.html", PAGE), ("0123456789abcdef/a.js", b""), ("fedcba9876543210/a.js", b"")],
            BUNDLE_LIMITS,
            "fedcba9876543210: more than one digest directory".to_owned(),
        ),
        (
            "bad digest",
            vec![("index.html", PAGE), ("0123456789abcdeF/a.js", b"")],
            BUNDLE_LIMITS,
            "0123456789abcdeF: digest must be 16 lowercase hex digits".to_owned(),
        ),
        (
            "short digest",
            vec![("index.html", PAGE), ("abc/a.js", b"")],
            BUNDLE_LIMITS,
            "abc: digest must be 16 lowercase hex digits".to_owned(),
        ),
        (
            "parent path",
            vec![("index.html", PAGE), ("0123456789abcdef/../a.js", b"")],
            BUNDLE_LIMITS,
            "0123456789abcdef/../a.js: path must be plain and at most 1024 bytes".to_owned(),
        ),
        (
            "dot path",
            vec![("index.html", PAGE), ("0123456789abcdef/./a.js", b"")],
            BUNDLE_LIMITS,
            "0123456789abcdef/./a.js: path must be plain and at most 1024 bytes".to_owned(),
        ),
        (
            "empty component",
            vec![("index.html", PAGE), ("0123456789abcdef//a.js", b"")],
            BUNDLE_LIMITS,
            "0123456789abcdef//a.js: path must be plain and at most 1024 bytes".to_owned(),
        ),
        (
            "long path",
            vec![("index.html", PAGE), (long.as_str(), b"")],
            BUNDLE_LIMITS,
            format!("{long}: path must be plain and at most 1024 bytes"),
        ),
        (
            "closed list",
            vec![("index.html", PAGE), ("0123456789abcdef/data.bin", b"")],
            BUNDLE_LIMITS,
            "0123456789abcdef/data.bin: file type is not on the closed list".to_owned(),
        ),
        (
            "files",
            BUNDLE_FILES.to_vec(),
            Clients { bundle_files: BUNDLE_COUNT - 1, ..BUNDLE_LIMITS },
            "0123456789abcdef/files/robots.txt: past clients.bundle_files".to_owned(),
        ),
        (
            "bytes",
            BUNDLE_FILES.to_vec(),
            Clients { bundle_bytes: BUNDLE_BYTES - 1, ..BUNDLE_LIMITS },
            "0123456789abcdef/files/robots.txt: past clients.bundle_bytes".to_owned(),
        ),
    ];
    for (name, files, limits, why) in rows {
        assert_eq!(
            temper_box_shell::bundle::embedded(&files, limits).expect_err("refused"),
            format!("application.clients.bundle: the binary's bundle: {why}"),
            "{name}"
        );
    }
}

#[test]
fn embedded_files_at_each_bound_are_admitted() {
    let bound = format!("0123456789abcdef/{}.js", "a".repeat(PATH_STEM_BOUND));
    for (name, files, limits) in [
        (
            "files and bytes",
            BUNDLE_FILES.to_vec(),
            temper_service::limits::Clients { bundle_files: BUNDLE_COUNT, bundle_bytes: BUNDLE_BYTES },
        ),
        ("path", vec![("index.html", PAGE), (bound.as_str(), b"")], BUNDLE_LIMITS),
    ] {
        temper_box_shell::bundle::embedded(&files, limits).unwrap_or_else(|why| panic!("{name}: {why}"));
    }
}

#[test]
fn a_story_purse_is_priced_from_the_init_template() {
    assert!(support::first_repository::run_spend(5).is_power_of_two());
}
