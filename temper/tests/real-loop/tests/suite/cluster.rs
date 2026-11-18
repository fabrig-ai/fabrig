//! The cluster's two one-slot workers each run a chat to its end.
use temper_real_loop_world::{
    End, Exit, argv,
    calls::{D1, D2, file, join, same_party, start, task, watching},
    fixtures, llm, run_cluster, scripts,
};

const PROBE: &str = "temper-worker-path-probe";
const MARKER: &[u8] = b"worker-path-reached-agent";
const PROBE_CHAT: usize = 3;
fn path_probe_scripts(_forge: std::net::SocketAddr) -> Box<[skein_fake_llm_domain::api::Script]> {
    use skein_fake_llm_domain::api::{Finish, Line, Turn};
    let mut scripts = scripts::chat_scripts(1);
    let script = scripts.first_mut().expect("one probe chat");
    let mut turns = core::mem::take(&mut script.turns).into_vec();
    turns.insert(
        0,
        Turn {
            lines: Box::new([Line::Call {
                name: b"shell".as_slice().into(),
                arguments: serde_json::to_vec(&serde_json::json!({"command":PROBE}))
                    .expect("probe tool arguments")
                    .into(),
            }]),
            finish: Finish::ToolCalls,
            tokens: 1,
        },
    );
    script.turns = turns.into_boxed_slice();
    scripts
}
fn path_probe_steps(engine: &std::path::Path) -> Vec<temper_real_loop_world::calls::Step> {
    use temper_real_loop_world::calls::call;
    let mut steps = fixtures::signed_chat("path-probe");
    let chat = steps.pop().expect("chat opener");
    let mut items = fixtures::setup_items(engine).expect("chat settings");
    items[0]["set"]["shell"] = true.into();
    std::fs::write(
        engine.join("shell-settings.json"),
        serde_json::to_vec(&serde_json::json!({"items":items})).expect("shell charter settings JSON"),
    )
    .expect("shell charter settings");
    steps.extend([
        // The cluster's membership transaction is revision 2, before this story's charter.
        call(D1, argv!["settings", "apply", file("shell-settings.json"), "--against", "2", "--key", "shell-chat"]),
        call(
            D1,
            argv![
                "repository",
                "adopt",
                "--project",
                "1",
                "--forge",
                "1",
                "--owner",
                "acme",
                "--name",
                "app",
                "--landing",
                "main",
                "--key",
                "probe-home"
            ],
        ),
        chat,
        call(D1, argv!["task", "show", task(PROBE_CHAT), "--until", "replied"]),
        call(D1, argv!["chat", "close", task(PROBE_CHAT), "--key", "close-path-probe"]),
        call(D1, argv!["task", "show", task(PROBE_CHAT), "--until", "ended"]),
    ]);
    steps
}
#[test]
fn an_agents_shell_finds_a_program_only_on_its_workers_path() {
    use skein_fake_llm_domain::api::Part;
    use std::os::unix::fs::PermissionsExt;
    let engine = std::cell::OnceCell::new();
    let cluster = temper_real_loop_world::cluster::run_cluster_forge_prepared(
        &["w1"],
        &[],
        temper_real_loop_world::forge::Fixture {
            configure: |deployment, agent, forge| {
                temper_real_loop_world::landing::configure(deployment, agent, forge);
                // The probe's chat parks at wait; its person ends it after reading.
                fixtures::requester(deployment);
            },
            ..temper_real_loop_world::forge::fixture()
        },
        path_probe_scripts,
        || Box::new([]),
        |scratch| {
            engine.set(scratch[0].path().join("data")).expect("one engine scratch");
            let bin = scratch[1].path().join("data/bin");
            std::fs::create_dir_all(&bin).expect("worker's private PATH directory");
            let probe = bin.join(PROBE);
            std::fs::write(&probe, b"#!/bin/sh\nprintf 'worker-path-reached-agent\\n'\n").expect("PATH probe");
            std::fs::set_permissions(probe, std::fs::Permissions::from_mode(0o700)).expect("executable probe");
        },
        || path_probe_steps(engine.get().expect("prepared engine")),
    );
    for called in cluster.calls() {
        assert_eq!(called.exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(&called.stderr));
    }
    let chat = cluster.number(PROBE_CHAT);
    assert_eq!(cluster.worker("w1").ran(), &[chat], "the probe runs once, without a retried wait");
    assert_eq!(cluster.tries(chat).run, 0, "waiting is a chat's successful park");
    let mut found = false;
    let mut outputs = Vec::new();
    for observed in cluster.llm() {
        if let skein_fake_peers::llm::Observation::Query { query, .. } = observed {
            for message in &query.messages {
                for part in &message.parts {
                    if let Part::ToolOutput { output, is_error, .. } = part {
                        outputs.push(String::from_utf8_lossy(output).into_owned());
                        if output.windows(MARKER.len()).any(|bytes| bytes == MARKER) {
                            assert!(
                                !is_error && output.starts_with(b"exit code 0\n"),
                                "shell failed: {}",
                                String::from_utf8_lossy(output)
                            );
                            found = true;
                        }
                    }
                }
            }
        }
    }
    assert!(found, "the next LLM query holds the shell's PATH probe output: {outputs:?}");
}

const WORKERS: &[&str] = &["w1", "w2"];
const CHATS: usize = 2;
const RUNS_PER_WORKER: usize = 1;
const LINKS_PER_WORKER: usize = 1;
const FIRST_SIGN_IN: usize = 0;
const FIRST_CHAT: usize = 2;
const SECOND_CHAT: usize = 3;
const FIRST_REPLY: usize = 4;
const SECOND_REPLY: usize = 5;
const FIRST_WAIT: usize = 6;
const FIRST_CLOSE: usize = 7;
const SECOND_WAIT: usize = 8;
const SECOND_CLOSE: usize = 9;
#[test]
fn two_workers_take_figures_and_each_run_a_chat_to_its_end() {
    let cluster = run_cluster(
        WORKERS,
        || {
            use skein_fake_llm_domain::api::{Line, Piece};
            let mut scripts = scripts::chat_scripts(CHATS);
            for script in &mut scripts {
                *script.turns.first_mut().expect("chat turn").lines.first_mut().expect("chat text") = Line::Pieces {
                    pieces: Box::new([Piece {
                        text: scripts::REPLY.into(),
                        hold: Some(temper_real_loop_world::cluster::PLACED.into()),
                    }]),
                };
            }
            llm::scripted(scripts)
        },
        || {
            vec![
                start(D1, argv!["sign-in", file("link")]),
                same_party(D2),
                join(FIRST_SIGN_IN),
                start(D1, argv!["chat", "start", "--project", fixtures::PROJECT, "--key", "first", scripts::WORDS]),
                start(D2, argv!["chat", "start", "--project", fixtures::PROJECT, "--key", "second", scripts::WORDS]),
                join(FIRST_CHAT),
                join(SECOND_CHAT),
                start(D1, argv!["task", "show", task(FIRST_CHAT), "--until", "replied"]),
                start(D2, argv!["task", "show", task(SECOND_CHAT), "--until", "replied"]),
                join(FIRST_REPLY),
                join(SECOND_REPLY),
                start(D1, argv!["task", "show", task(FIRST_CHAT), "--until", "ended"]),
                watching(FIRST_WAIT),
                start(D1, argv!["chat", "close", task(FIRST_CHAT), "--key", "close-first"]),
                join(FIRST_CLOSE),
                join(FIRST_WAIT),
                start(D2, argv!["task", "show", task(SECOND_CHAT), "--until", "ended"]),
                watching(SECOND_WAIT),
                start(D2, argv!["chat", "close", task(SECOND_CHAT), "--key", "close-second"]),
                join(SECOND_CLOSE),
                join(SECOND_WAIT),
            ]
        },
    );
    for (at, called) in cluster.calls().iter().enumerate() {
        assert_eq!(called.exit, Some(Exit::Code(0)), "call {at}: {}", String::from_utf8_lossy(&called.stderr));
    }
    assert_eq!(cluster.served(), Some(&End::Engine(temper_service::EngineEnd::Drained)));
    let mut tasks = [FIRST_CHAT, SECOND_CHAT].map(|call| cluster.number(call));
    let mut placed = Vec::new();
    for name in WORKERS {
        let worker = cluster.worker(name);
        assert_eq!(worker.figures_lines().len(), 1, "{name} took figures once");
        let line = &worker.figures_lines()[0];
        assert!(line.starts_with("figures taken from 127.0.0.1:"), "{line}");
        assert!(line.contains(" places, ") && line.ends_with(" limits, 1 endpoints\n"), "{line}");
        assert_eq!(worker.linked(), LINKS_PER_WORKER, "{name} linked once");
        assert_eq!(worker.end(), Some(&End::Worker(temper_service::worker_alone::End::Drained)), "{name} drained");
        assert_eq!(worker.ran().len(), RUNS_PER_WORKER, "{name} ran one chat");
        placed.extend_from_slice(worker.ran());
    }
    placed.sort_unstable();
    tasks.sort_unstable();
    assert_eq!(placed, tasks, "each worker first ran a different chat");
}
