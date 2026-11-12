//! The cluster's two one-slot workers each run a chat to its end.
use skein_lib::Duration;
use temper_real_loop_world::{
    End, Exit, argv,
    calls::{D1, D2, file, join, same_party, start, task, watching},
    fixtures, llm, run_cluster, scripts,
};
use temper_real_loop_world::{calls, cluster::run_cluster_with_grace};

const CONTACT_HOLD: &[u8] = b"engine-contact";
const BEFORE: &[u8] = b"Before the engine restart";

fn interrupted_script() -> llm::Recipe {
    use skein_fake_llm_domain::api::{Finish, Line, Piece, Script, Turn};
    llm::scripted(Box::new([Script {
        cue: Box::default(),
        turns: Box::new([
            Turn {
                lines: Box::new([
                    Line::Text { text: BEFORE.into() },
                    Line::Call {
                        name: b"read_docs".as_slice().into(),
                        arguments: br#"{"topic":"chat"}"#.as_slice().into(),
                    },
                ]),
                finish: Finish::ToolCalls,
                tokens: 1,
            },
            Turn {
                lines: Box::new([
                    Line::Pieces {
                        pieces: Box::new([Piece { text: scripts::REPLY.into(), hold: Some(CONTACT_HOLD.into()) }]),
                    },
                    Line::Call { name: b"wait".as_slice().into(), arguments: b"{}".as_slice().into() },
                ]),
                finish: Finish::ToolCalls,
                tokens: 1,
            },
        ]),
    }]))
}

fn restart_chat_steps(past: bool, failures: u32) -> Vec<calls::Step> {
    const CHAT: usize = 1;
    let mut steps = vec![
        calls::sign_in(D1),
        calls::call(
            D1,
            argv!["chat", "start", "--project", fixtures::PROJECT, "--key", "restart-chat", scripts::WORDS],
        ),
        calls::until(D1, argv!["chat", "show", task(CHAT)], BEFORE, scripts::EARLY),
        calls::held(CONTACT_HOLD),
        calls::stop_engine(),
    ];
    if past {
        steps.push(calls::worker_cancelled("w1"));
    }
    steps.push(calls::start_engine_after_failures(failures));
    if past {
        steps.push(calls::held(CONTACT_HOLD));
    }
    steps.extend([
        calls::release_hold(CONTACT_HOLD),
        calls::until(D1, argv!["chat", "show", task(CHAT)], scripts::REPLY, scripts::EARLY),
        calls::call(D1, argv!["chat", "close", task(CHAT), "--key", "close-restart"]),
        calls::call(D1, argv!["task", "show", task(CHAT), "--until", "ended"]),
    ]);
    steps
}

#[test]
fn past_the_engine_away_bound_the_attempt_is_interrupted_and_resumes_without_a_try() {
    let cluster = run_cluster_with_grace(&["w1"], Duration::from_millis(1_000), interrupted_script, || {
        restart_chat_steps(true, 0)
    });
    for (at, called) in cluster.calls().iter().enumerate() {
        assert_eq!(called.exit, Some(Exit::Code(0)), "call {at}: {}", String::from_utf8_lossy(&called.stderr));
    }
    let worker = cluster.worker("w1");
    let [first, second] = worker.assignments() else { panic!("two attempts: {}", worker.assignments().len()) };
    assert_eq!(first.task(), second.task());
    assert_eq!(cluster.tries(first.task()), jig_domain_tasks::Tries::NONE, "contact costs no try");
    assert_ne!(first.attempt(), second.attempt());
    assert_eq!(first.identity(), second.identity(), "the new attempt keeps the first conversation");
    assert_eq!((first.transcript()).len(), 0);
    assert_eq!(calls::occurrences(second.transcript(), BEFORE), 1, "the assignment carries the committed first turn");
    let queries: Vec<_> =
        cluster
            .llm()
            .iter()
            .filter_map(|seen| {
                if let skein_fake_peers::llm::Observation::Query { query, .. } = seen { Some(query) } else { None }
            })
            .collect();
    let [_, before, resumed] = queries.as_slice() else {
        panic!("first turn, held second turn, resumed second turn: {}", queries.len())
    };
    assert!(resumed.messages.len() >= before.messages.len());
    for (before, resumed) in before.messages.iter().zip(&resumed.messages) {
        assert_eq!(before.role, resumed.role);
        assert!(resumed.parts.starts_with(&before.parts), "resumed conversation retains each interrupted message");
    }
    assert_eq!(worker.linked(), 2);
    assert!(cluster.killed().is_empty());
}

#[test]
fn an_engine_restarted_within_the_bound_keeps_the_same_attempt() {
    same_attempt_after_starts(0);
}

#[test]
fn two_refused_engine_starts_within_the_bound_keep_the_same_attempt() {
    same_attempt_after_starts(2);
}

fn same_attempt_after_starts(failures: u32) {
    let cluster = run_cluster_with_grace(&["w1"], Duration::from_secs(60), interrupted_script, || {
        restart_chat_steps(false, failures)
    });
    assert_eq!(cluster.failed_starts().len(), usize::try_from(failures).expect("small start count"));
    for (at, called) in cluster.calls().iter().enumerate() {
        assert_eq!(called.exit, Some(Exit::Code(0)), "call {at}: {}", String::from_utf8_lossy(&called.stderr));
    }
    let worker = cluster.worker("w1");
    let [assignment] = worker.assignments() else { panic!("one attempt: {}", worker.assignments().len()) };
    assert_eq!(worker.ran(), &[assignment.task()]);
    assert_eq!(worker.linked(), 2);
    assert_eq!(cluster.tries(assignment.task()), jig_domain_tasks::Tries::NONE);
    assert_eq!(
        cluster.llm().iter().filter(|seen| matches!(seen, skein_fake_peers::llm::Observation::Query { .. })).count(),
        2
    );
    assert_eq!(calls::occurrences(&cluster.calls()[3].stdout, scripts::REPLY), 1);
    assert!(cluster.killed().is_empty());
}

#[test]
fn stopping_a_worker_before_the_first_turn_keeps_the_first_message_once() {
    let cluster = run_cluster_with_grace(
        WORKERS,
        Duration::from_secs(60),
        || {
            use skein_fake_llm_domain::api::{Line, Piece};
            let mut scripts = scripts::chat_scripts(1);
            scripts[0].turns[0].lines[0] = Line::Pieces {
                pieces: Box::new([Piece { text: scripts::REPLY.into(), hold: Some(CONTACT_HOLD.into()) }]),
            };
            llm::scripted(scripts)
        },
        || {
            vec![
                calls::sign_in(D1),
                calls::call(
                    D1,
                    argv!["chat", "start", "--project", fixtures::PROJECT, "--key", "worker-stop", scripts::WORDS],
                ),
                calls::held(CONTACT_HOLD),
                calls::stop_worker("w1"),
                calls::held(CONTACT_HOLD),
                calls::release_hold(CONTACT_HOLD),
                calls::until(D1, argv!["chat", "show", task(1)], scripts::REPLY, scripts::EARLY),
                calls::start_worker("w1"),
                calls::call(D1, argv!["chat", "close", task(1), "--key", "close-worker-stop"]),
                calls::call(D1, argv!["task", "show", task(1), "--until", "ended"]),
            ]
        },
    );
    for (at, called) in cluster.calls().iter().enumerate() {
        assert_eq!(called.exit, Some(Exit::Code(0)), "call {at}: {}", String::from_utf8_lossy(&called.stderr));
    }
    let [first] = cluster.worker("w1").assignments() else { panic!("first attempt on w1") };
    let [second] = cluster.worker("w2").assignments() else { panic!("second attempt on w2") };
    assert_eq!(first.task(), second.task());
    assert_ne!(first.attempt(), second.attempt());
    assert!(second.transcript().is_empty(), "no first turn committed");
    let queries: Vec<_> =
        cluster
            .llm()
            .iter()
            .filter_map(|seen| {
                if let skein_fake_peers::llm::Observation::Query { query, .. } = seen { Some(query) } else { None }
            })
            .collect();
    let [first, second] = queries.as_slice() else { panic!("two model queries") };
    for query in [first, second] {
        let messages: usize = query
            .messages
            .iter()
            .flat_map(|message| &message.parts)
            .map(|part| {
                if let skein_fake_llm_domain::api::Part::Text { text } = part {
                    calls::occurrences(text, scripts::WORDS.as_bytes())
                } else {
                    0
                }
            })
            .sum();
        assert_eq!(messages, 1, "the first message reaches each attempt once");
    }
    assert_eq!(calls::occurrences(&cluster.calls()[2].stdout, scripts::REPLY), 1);
    assert!(cluster.killed().is_empty());
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
fn two_workers_each_run_a_chat_to_its_end() {
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
    assert_eq!(cluster.served(), Some(&End::Drained));
    let mut tasks = [FIRST_CHAT, SECOND_CHAT].map(|call| cluster.number(call));
    let mut placed = Vec::new();
    for name in WORKERS {
        let worker = cluster.worker(name);
        assert_eq!(worker.linked(), LINKS_PER_WORKER, "{name} linked once");
        assert_eq!(worker.end(), Some(&End::Drained), "{name} drained");
        assert_eq!(worker.ran().len(), RUNS_PER_WORKER, "{name} ran one chat");
        placed.extend_from_slice(worker.ran());
    }
    placed.sort_unstable();
    tasks.sort_unstable();
    assert_eq!(placed, tasks, "each worker first ran a different chat");
}
