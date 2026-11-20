//! Provider silence after a worker's shell result (testing/real-loop.md, hung requests).
use serde_json::{Value, json};
use skein_fake_llm_domain::api::{Finish, Line, Part, Piece, Script, Turn};
use skein_fake_peers::llm::Observation;
use skein_lib::Duration;
use temper_real_loop_world::{
    Cluster, Exit, argv,
    calls::{D1, call, file, release_hold, release_hold_after_queries, task, until},
    fixtures,
};

const HEAD_MS: u64 = 250;
const IDLE_MS: u64 = 300;
const HOLD: &[u8] = b"hung-after-shell";
const MARKER: &[u8] = b"hung-shell-result";
const REPORT: &str = "The shell result reached the model after its retry.";
const CHAT: usize = 3;
const READ: usize = 4;

fn configure(
    deployment: &mut temper_playground::documents::Configuration,
    agent: &mut Value,
    forge: std::net::SocketAddr,
) {
    temper_real_loop_world::landing::configure(deployment, agent, forge);
    // A held response also exercises transport teardown. Keep its close tail
    // below the endpoint's idle bound instead of paying the stock five seconds.
    agent["profile"]["policy"]["close"] = 20.into();
    agent["endpoints"][0]["limits"]["head"] = HEAD_MS.into();
    agent["endpoints"][0]["limits"]["idle"] = IDLE_MS.into();
    // Observe one failed run, not the engine's separate retries of whole runs.
    deployment.jig_over("tasks.retries.run.retries", 0);
}

fn scripts(_forge: std::net::SocketAddr) -> Box<[Script]> {
    Box::new([Script {
        cue: Box::default(),
        turns: Box::new([
            Turn {
                lines: Box::new([Line::Call {
                    name: b"shell".as_slice().into(),
                    arguments: br#"{"command":"printf 'hung-shell-result\\n'"}"#.as_slice().into(),
                }]),
                finish: Finish::ToolCalls,
                tokens: 1,
            },
            Turn {
                lines: Box::new([
                    Line::Pieces {
                        pieces: Box::new([Piece { text: b"Recovered.".as_slice().into(), hold: Some(HOLD.into()) }]),
                    },
                    Line::Call {
                        name: b"finish".as_slice().into(),
                        arguments: serde_json::to_vec(&json!({"form":"report","text":REPORT,"fields":{}}))
                            .expect("finish arguments")
                            .into(),
                    },
                ]),
                finish: Finish::ToolCalls,
                tokens: 1,
            },
        ]),
    }])
}

fn run(answer_retry: bool) -> Cluster {
    let engine = std::cell::OnceCell::new();
    temper_real_loop_world::cluster::run_cluster_forge_prepared(
        &["w1"],
        &[],
        temper_real_loop_world::forge::Fixture { configure, ..temper_real_loop_world::forge::fixture() },
        scripts,
        || Box::new([]),
        |scratch| {
            engine.set(scratch[0].path().join("data")).expect("one engine");
        },
        || {
            let mut steps = fixtures::signed_chat("hung-model");
            let chat = steps.pop().expect("chat opener");
            let mut items = fixtures::setup_items(engine.get().expect("prepared engine")).expect("settings");
            items[0]["set"]["shell"] = true.into();
            std::fs::write(
                engine.get().expect("prepared engine").join("shell-settings.json"),
                serde_json::to_vec(&json!({"items":items})).expect("shell settings JSON"),
            )
            .expect("shell settings");
            steps.extend([
                // Membership is revision two; the charter is the next transaction.
                call(
                    D1,
                    argv!["settings", "apply", file("shell-settings.json"), "--against", "2", "--key", "shell-chat"],
                ),
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
                        "hung-home"
                    ],
                ),
                chat,
            ]);
            if answer_retry {
                // The script counts assistant messages, so a retry selects the same
                // held turn. Release only after shell, held request, and retry arrived.
                steps.push(release_hold_after_queries(HOLD, 3));
            }
            steps.push(until(
                D1,
                argv!["task", "show", task(CHAT), "--activity", "32", "--json"],
                b"\"run_ended\"",
                b"\"task\"",
            ));
            if !answer_retry {
                steps.extend([
                    // The run has failed; settle the fake's detached responses too.
                    release_hold(HOLD),
                    call(
                        D1,
                        argv!["task", "cancel", task(CHAT), "The failed run was observed", "--key", "end-hung-model"],
                    ),
                    call(D1, argv!["task", "show", task(CHAT), "--until", "ended"]),
                ]);
            }
            steps
        },
    )
}

fn check(cluster: &Cluster, answer_retry: bool) {
    for called in cluster.calls() {
        assert_eq!(called.exit, Some(Exit::Code(0)), "{}", String::from_utf8_lossy(&called.stderr));
    }
    assert_eq!(cluster.worker("w1").ran(), &[cluster.number(CHAT)], "one run on the worker alone");
    assert!(
        cluster.worker("w1").notes().iter().all(|(_, note)| !matches!(
            note,
            temper_service::Note::Worker(temper_service::worker_alone::Note::AgentStopped { .. })
        )),
        "no agent stop, including NoProgress"
    );
    let queries: Vec<_> = cluster
        .llm()
        .iter()
        .filter_map(|event| if let Observation::Query { query, .. } = event { Some(query) } else { None })
        .collect();
    let agent: Value =
        serde_json::from_slice(&std::fs::read(cluster.path().join("data/world-agent.json")).expect("engine figures"))
            .expect("agent JSON");
    assert_eq!(agent["endpoints"][0]["limits"]["head"], HEAD_MS);
    assert_eq!(agent["endpoints"][0]["limits"]["idle"], IDLE_MS);
    let configured = smith_agent_shell::config::from_value(agent).expect("configured agent");
    let limits = &configured.service.limits.domain.session;
    assert_eq!(limits.retries, 3, "stock session retries");
    let attempts = if answer_retry { 2 } else { usize::try_from(limits.retries + 1).expect("session attempts") };
    assert_eq!(queries.len(), attempts + 1, "one shell turn, then held attempts");
    let attempts = &queries[1..];
    for query in attempts {
        assert_eq!(query, &attempts[0], "retry input is unchanged");
        let outputs: Vec<_> = query
            .messages
            .iter()
            .flat_map(|message| message.parts.iter())
            .filter_map(|part| {
                if let Part::ToolOutput { output, is_error, .. } = part { Some((output, is_error)) } else { None }
            })
            .collect();
        assert_eq!(outputs.len(), 1, "the shell's result in every attempt");
        let (output, is_error) = outputs[0];
        assert!(
            !is_error
                && output.starts_with(b"exit code 0\n")
                && output.windows(MARKER.len()).any(|bytes| bytes == MARKER),
            "{}",
            String::from_utf8_lossy(output)
        );
    }
    let host = smith_agent_profile::host_limits(
        &configured.profile,
        &configured.service.limits,
        &configured.service.domain.limits,
    )
    .expect("host supervision");
    let bound = Duration::from_millis(HEAD_MS + IDLE_MS)
        .saturating_mul(u64::from(limits.retries) + 1)
        .saturating_add(limits.backoff_max.saturating_mul(u64::from(limits.retries)))
        .saturating_add(Duration::from_secs(3));
    assert!(bound.saturating_mul(10) < host.no_progress && host.no_progress >= limits.tools.tool_deadline);
    let times = cluster.llm_query_times();
    assert_eq!(times.len(), queries.len());
    for pair in times[1..].windows(2) {
        let gap = pair[1].saturating_since(pair[0]);
        assert!(
            gap >= Duration::from_millis(IDLE_MS)
                .saturating_add(Duration::from_nanos(limits.backoff_base.as_nanos() / 2)),
            "idle cut and backoff before retry: {gap:?}"
        );
        assert!(
            gap < Duration::from_millis(HEAD_MS + IDLE_MS)
                .saturating_add(limits.backoff_max)
                .saturating_add(Duration::from_secs(3)),
            "retry within endpoint and backoff bounds: {gap:?}"
        );
    }
    let elapsed = cluster.calls()[READ].ended.saturating_since(times[1]);
    assert!(elapsed < bound, "held request ended in {elapsed:?}, bound {bound:?}");
    assert!(cluster.killed().is_empty(), "no agent was killed");
}

#[test]
fn a_model_request_held_once_retries_and_finishes() {
    let cluster = run(true);
    check(&cluster, true);
    let read: Value = serde_json::from_slice(&cluster.calls()[READ].stdout).expect("task JSON");
    // Until-ended can observe the live terminal row or its reclaimed result.
    let report = read
        .pointer("/task/result/value/report/bytes")
        .or_else(|| read.pointer("/task/result/ending/done/report/bytes"))
        .expect("accepted report in the live ending or reclaimed result");
    assert_eq!(report, REPORT, "accepted result: {read}");
}

#[test]
fn a_model_request_held_on_every_attempt_fails_before_no_progress() {
    let cluster = run(false);
    check(&cluster, false);
    let read: Value = serde_json::from_slice(&cluster.calls()[READ].stdout).expect("task JSON");
    let held = &read["task"]["hold"];
    assert_eq!(held["reason"], json!({"failures":{"class":"run"}}), "a run failure, not an agent failure: {read}");
    assert_eq!(
        held["words"]["bytes"], "fake fake: timed out, status 200, response",
        "the engine's named endpoint/model timeout"
    );
    assert_eq!(held["tries"]["run"], 1);
    assert_eq!(held["tries"]["agent"], 0);
    let ended: Vec<_> = read["activity"]["rows"]
        .as_array()
        .expect("activity rows")
        .iter()
        .filter_map(|row| row.pointer("/event/owner/source/line/run_ended"))
        .collect();
    assert_eq!(ended.len(), 1, "one failed run");
    assert_eq!(ended[0]["end"], json!({"failed":{"class":"run"}}));
}
