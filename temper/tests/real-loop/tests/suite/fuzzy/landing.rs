//! A real forge landing precedes the kept delegate's first and only run.
//! Dependency policy is held by the engine domain's `forge_routes::kept_delegate`.
use serde_json::Value;
use temper_real_loop_world::landing;

#[test]
fn a_chat_keeps_its_live_delegate_until_the_change_lands() {
    use landing::kept::{A3, E1};
    use temper_real_loop_world::{
        argv,
        calls::{D1, call, number, sign_in, task},
        forge, run_forge,
    };
    const ONE_RUN: usize = 1;
    const CHAT: usize = 2;
    const E1_DOCUMENT: usize = 4;
    const A3_DOCUMENT: usize = 5;
    let ran = run_forge(
        landing::kept::fixture(),
        |at| landing::kept::scripts(at, u64::from(forge::ID)),
        || {
            vec![
                sign_in(D1),
                call(
                    D1,
                    argv![
                        "repository",
                        "adopt",
                        "--project",
                        "1",
                        "--owner",
                        "acme",
                        "--name",
                        "app",
                        "--landing",
                        "main",
                        "--key",
                        "adopt-app"
                    ],
                ),
                call(D1, argv!["chat", "start", "--project", "1", "--key", "kept", "@kept_chat Add a landing wait"]),
                call(D1, argv!["task", "show", task(CHAT), "--until", "replied"]),
                call(D1, argv!["task", "show", number(E1), "--until", "ended", "--json"]),
                call(D1, argv!["task", "show", number(A3), "--until", "ended", "--json"]),
                // Drain the chat before teardown; its scripted reply is not an oracle.
                call(D1, argv!["task", "show", task(CHAT), "--until", "ended"]),
            ]
        },
    );
    assert!(ran.forge().merged_green() && ran.forge().main_file(b"README.md").ends_with(landing::EDIT));
    let e1: Value =
        serde_json::from_slice::<Value>(&ran.calls()[E1_DOCUMENT].stdout).expect("e1 document")["task"].take();
    let a3: Value =
        serde_json::from_slice::<Value>(&ran.calls()[A3_DOCUMENT].stdout).expect("a3 document")["task"].take();
    let ended = |value: &Value| value["chip"]["details"]["owner"]["details"]["ended_at"].as_u64().expect("ended wall");
    let started = a3["chip"]["details"]["owner"]["details"]["run_since"].as_u64().expect("a3 run start");
    assert!(started >= ended(&e1), "a3 started after e1 ended");
    let mut a3_runs = 0;
    // a3's script has one finish call per run, hence one query per run.
    for seen in ran.llm() {
        let skein_fake_peers::llm::Observation::Query { query, .. } = seen else { continue };
        if query.system.windows(b"@kept_a3".len()).any(|word| word == b"@kept_a3") {
            a3_runs += 1;
        }
    }
    assert_eq!(a3_runs, ONE_RUN);
}
