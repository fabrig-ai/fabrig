mod credentials;
mod program_check;
mod program_commands;
mod program_rechecks;
mod program_runs;
mod relay;
pub mod support;

use smith_local_process_world::{Placement, referee::CheckoutRead};
use smith_real_world::{Scratch, World};

#[test]
fn the_terminal_modes_are_unchanged_after_smith_exits() {
    use smith_local_process_world::terminal::Action;
    let scratch = Scratch::new();
    let _file = support::user_layer(&scratch, &support::fake_provider_settings(&scratch));
    let (ran, queries) = support::chat_modes(
        &scratch,
        vec![Action::Wait(b"> ".as_slice().into()), Action::Send(b"Ask first\n".as_slice().into()), Action::Eof],
    );
    assert_eq!(ran.code, 0, "{}", String::from_utf8_lossy(&ran.stderr));
    assert_eq!(queries.len(), 1);
}

#[test]
fn end_of_input_at_the_terminal_lets_the_run_answer_then_exits_0() {
    use smith_local_process_world::terminal::Action;
    use std::os::unix::fs::PermissionsExt;
    let scratch = Scratch::new();
    let mut settings: toml::Value =
        toml::from_str(std::str::from_utf8(&support::fake_provider_settings(&scratch)).unwrap()).unwrap();
    settings["accounts"]["codex"] = toml::Value::try_from(serde_json::json!({"source":"key"})).unwrap();
    settings["endpoints"]["codex"].as_table_mut().unwrap().insert("preset".into(), "openai".into());
    let _file = support::user_layer(&scratch, toml::to_string(&settings).unwrap().as_bytes());
    let token: serde_json::Value =
        serde_json::from_slice(&std::fs::read(scratch.path().join("auth.json")).unwrap()).unwrap();
    let key = scratch.path().join("credentials/keys/codex");
    std::fs::create_dir_all(key.parent().unwrap()).unwrap();
    std::fs::write(&key, token["tokens"]["access_token"].as_str().unwrap()).unwrap();
    std::fs::set_permissions(&key, std::fs::Permissions::from_mode(0o600)).unwrap();
    let ran = support::key_chat(
        &scratch,
        vec![Action::Wait(b"> ".as_slice().into()), Action::Send(b"Ask first\n".as_slice().into()), Action::Eof],
    );
    assert_eq!(ran.code, 0, "{}", String::from_utf8_lossy(&ran.stderr));
    assert!(String::from_utf8_lossy(&ran.stdout).contains("API key"));
    assert!(skein_lib::bytes::find(&ran.stdout, b"First answer").is_some());
}

#[test]
fn a_window_size_change_reaches_the_status_line() {
    use smith_local_process_world::terminal::Action;
    let scratch = Scratch::new();
    let _file = support::user_layer(&scratch, &support::fake_provider_settings(&scratch));
    let (ran, queries) = support::chat_resized(
        &scratch,
        vec![
            Action::Wait(b"> ".as_slice().into()),
            Action::Send(b"Ask first\n".as_slice().into()),
            Action::Wait(b"codex/gpt-6-luna; API key: in ".as_slice().into()),
            Action::Wait(b"\r\nFinishing now\r\n".as_slice().into()),
            Action::Send(b"Again\n".as_slice().into()),
            Action::Eof,
        ],
    );
    assert_eq!(ran.code, 0, "{}", String::from_utf8_lossy(&ran.stderr));
    assert_eq!(queries.len(), 2);
    let widths: Vec<_> = ran
        .stdout
        .split(|byte| *byte == b'\n')
        .map(|line| line.strip_suffix(b"\r").unwrap_or(line))
        .filter(|line| line.starts_with(b"codex/gpt-6-luna;"))
        .map(<[u8]>::len)
        .collect();
    assert_eq!(widths, [80, 20]);
}

#[test]
fn the_shipped_local_and_agent_answer_on_a_terminal() {
    let scratch = Scratch::new();
    let scenario = World::new(31, &scratch, Placement::Spawned, b"First answer").scenario;
    let seen = support::run(&scratch, scenario);
    assert_eq!(seen.queries.len(), 1);
}

#[test]
fn a_first_run_signs_in_and_ends_with_a_commit_in_place() {
    let scratch = Scratch::new();
    let mut scenario = World::changed(32, &scratch, Placement::Spawned).scenario;
    scenario.launch.authenticated = true;
    scenario.authentication = Some(smith_local_process_world::world::Authentication::SignIn);
    std::fs::remove_file(scenario.launch.token_directory.join("0.json")).expect("first sign-in has no token");
    let seen = support::run(&scratch, scenario);
    let oauth = seen.oauth.expect("issuer and browser observations");
    assert_eq!(oauth.posts, 1);
    assert_eq!(oauth.pages, 1);
    assert!(oauth.browser_replied && oauth.saved_before_query);
    let checkout = scratch.checkout();
    let head = checkout.head().expect("committed head");
    let files = checkout.files(&head);
    assert_eq!(files[b"original.txt".as_slice()], b"edited\n");
    assert_eq!(files[b"result.txt".as_slice()], b"new\n");
    assert_eq!(files[b"command.txt".as_slice()], b"ran\n");
    assert_eq!(files[b"checks-ran.txt".as_slice()], b"checked\n");
}

#[test]
fn an_interrupt_cancels_the_run() {
    use smith_local_process_world::terminal::Action;
    let scratch = Scratch::new();
    let _file = support::user_layer(&scratch, &support::fake_provider_settings(&scratch));
    let ran = support::chat_waiting(
        &scratch,
        vec![
            Action::Send(b"hello\n".as_slice().into()),
            Action::Wait(b"Starting now".as_slice().into()),
            Action::Send(Box::new([3])),
            Action::Wait(b"interrupted: the run was cancelled".as_slice().into()),
            Action::Eof,
        ],
    );
    assert_eq!(ran.code, 0, "{}", String::from_utf8_lossy(&ran.stderr));
    assert!(skein_lib::bytes::find(&ran.stdout, b"interrupted: the run was cancelled").is_some());
}

#[test]
fn smith_on_a_terminal_signs_in_and_commits_in_place() {
    use smith_local_process_world::terminal::Action;
    let scratch = Scratch::new();
    let mut settings: toml::Value =
        toml::from_str(std::str::from_utf8(&support::fake_provider_settings(&scratch)).unwrap()).unwrap();
    let host = smith_real_world::loopback();
    let redirect = smith_real_world::redirect_port();
    settings["accounts"]["codex"] = toml::Value::try_from(serde_json::json!({
        "source":"sign_in", "client_id":"client", "authorization_url":format!("http://{host}:34444/authorize"),
        "token_url":format!("http://{host}:34444/token"), "redirect_uri":format!("http://127.0.0.1:{redirect}/callback"),
        "scope":"read", "token_format":"form"
    }))
    .unwrap();
    settings.as_table_mut().unwrap().insert(
        "workspace".into(),
        toml::Value::try_from(serde_json::json!({
            "directories":[{"name":"repo","path":scratch.path().join("repo"),"writable":true,"git":true}],
            "conventions":{"guide":".smith-test/guide","checks":".smith-test/check"},
            "contract":{"form":"change","checks_must_pass":true,
                "fields":[{"name":"title","max":256},{"name":"body","max":1024}]}
        }))
        .unwrap(),
    );
    let _file = support::user_layer(&scratch, toml::to_string(&settings).unwrap().as_bytes());
    let before = scratch.checkout().head().unwrap();
    let ran = support::chat_signed_in(
        &scratch,
        vec![
            Action::Send(b"change it\n".as_slice().into()),
            Action::Wait(b"Change delivered".as_slice().into()),
            Action::Eof,
        ],
    );
    assert_eq!(ran.code, 0, "{}", String::from_utf8_lossy(&ran.stderr));
    assert!(String::from_utf8_lossy(&ran.stdout).contains(&format!("Sign in: http://{host}:34444/authorize")));
    assert!(scratch.path().join("credentials/tokens/codex").is_file());
    let checkout = scratch.checkout();
    let after = checkout.head().unwrap();
    assert_ne!(before, after);
    let files = checkout.files(&after);
    assert_eq!(files[b"original.txt".as_slice()], b"edited\n");
    assert_eq!(files[b"result.txt".as_slice()], b"new\n");
    assert_eq!(files[b"command.txt".as_slice()], b"ran\n");
    assert_eq!(files[b"checks-ran.txt".as_slice()], b"checked\n");
}

#[test]
fn the_startup_refusals_of_both_commands_are_observed_outside() {
    let scratch = Scratch::new();
    let agent = skein_scratch::paths::binary("smith-agent");
    let errors = support::refusal_program(&agent, &scratch, vec!["".into()]);
    assert!(String::from_utf8_lossy(&errors).contains("configuration metadata"));
    let config = scratch.path().join("config/smith/config.toml");
    std::fs::create_dir_all(&config).expect("settings metadata refusal");
    let errors = support::refusal_code(&scratch, Vec::new(), 2);
    assert!(String::from_utf8_lossy(&errors).contains("regular file"));
    let path = scratch.path().join("agent-config.json");
    std::fs::write(
        &path,
        br#"{"profile":{"name":"standard","declared":{"memory":1099511627776,"conversations":1,"calls_per_response":2,"tool_payload":256,"read_window":256},"policy":{"group_stop":10}},"endpoints":[{"name":"fake","number":0,"dialect":1,"account":0,"provider":"codex","address":"127.0.0.1:8080","transport":"plaintext","connect":1000,"handshake":1000,"models":[{"name":"fake","window":65536,"output":512,"reasoning_item":64,"prices":{"input":1,"cached":1,"output":1,"unit":1},"head":1000,"idle":1000}]}],"environment":[]}"#,
    )
    .expect("valid agent fixture");
    let errors = support::refusal_program(&agent, &scratch, vec![path.clone().into()]);
    assert_eq!(
        String::from_utf8_lossy(&errors).lines().last(),
        Some("smith: the run could not answer: the channel ended before the answer")
    );
    std::fs::write(&path, br#"{"profile":{"name":"standard","declared":{"memory":1099511627776,"conversations":1,"calls_per_response":2,"tool_payload":256,"read_window":256},"policy":{"group_stop":10}},"endpoints":[{"name":"invalid","number":1,"dialect":1,"account":0,"provider":"codex","connect":10000,"handshake":10000,"models":[{"name":"fake","window":65536,"output":512,"reasoning_item":64,"prices":{"input":1,"cached":1,"output":1,"unit":1},"head":60000,"idle":30000}],"address":"192.0.2.1:8080","transport":"plaintext"}],"environment":[]}"#).expect("invalid transport fixture");
    let errors = support::refusal_program(&agent, &scratch, vec![path.into()]);
    assert_eq!(
        String::from_utf8_lossy(&errors).lines().last(),
        Some("smith: plaintext endpoint address must be loopback")
    );
    assert!(!scratch.path().join("chat").exists(), "refusals leave no chat");
}

#[test]
fn the_live_outcome_referee_runs_on_the_shared_fake_binary_harness() {
    let report = Scratch::new();
    support::live_run::run_fake(&report, &World::new(35, &report, Placement::Spawned, b"First answer").scenario);
    let change = Scratch::new();
    support::live_run::run_fake(&change, &World::changed(36, &change, Placement::Spawned).scenario);
}

#[test]
fn standard_profile_runs_a_large_coding_write_and_the_following_checks() {
    use skein_fake_llm_domain::api;
    use smith_local_process_world::referee::CheckoutRead;

    let scratch = Scratch::new();
    let scenario = World::changed(37, &scratch, Placement::Spawned).scenario;
    let large = serde_json::to_vec(&serde_json::json!({
        "path": "result.txt", "content": "// coding regression\n".repeat(160)
    }))
    .expect("large coding tool arguments");
    assert!(large.len() > 2048, "coding write exceeds 2,048 bytes");
    let mut turns = vec![(b"write".as_slice(), large)];
    for (name, arguments) in [
        ("read", r#"{"path":"result.txt"}"#),
        ("read", r#"{"path":"original.txt"}"#),
        ("edit", r#"{"path":"original.txt","edits":[{"old":"original","new":"edited"}]}"#),
        ("write", r#"{"path":"result.txt","content":"new\n"}"#),
        ("shell", r#"{"command":"printf 'ran\n' > command.txt"}"#),
        (
            "finish",
            r#"{"form":"change","text":"","fields":{"title":"Updated result","body":"Created the result file"}}"#,
        ),
    ] {
        turns.push((name.as_bytes(), arguments.as_bytes().to_vec()));
    }
    let token = support::settings(&scratch, &scenario);
    let peer = smith_agent_process_world::fake::configured_transport(
        Box::new([api::Script {
            cue: b"@local-shell".as_slice().into(),
            turns: turns
                .into_iter()
                .map(|(name, arguments)| api::Turn {
                    lines: Box::new([api::Line::Call { name: name.into(), arguments: arguments.into() }]),
                    finish: api::Finish::ToolCalls,
                    tokens: 1,
                })
                .collect(),
        }]),
        skein_llm::Credential { value: token, account_id: b"acc".as_slice().into(), scheme: skein_llm::Scheme::Bearer },
        skein_lib::Duration::ZERO,
        skein_fake_peers::Transport::Tls,
    );
    let seen = support::run_with_provider(&scratch, scenario, Some(peer));
    assert_eq!(seen.queries.len(), 7, "large write and every subsequent call completed");
    assert!(
        seen.queries.iter().skip(1).any(|query| query
            .messages
            .iter()
            .any(|message| { message.parts.iter().any(|part| matches!(part, api::Part::ToolOutput { .. })) })),
        "follow-up provider prompts contain workspace answers"
    );
    let checkout = scratch.checkout();
    let files = checkout.files(&checkout.head().expect("committed head"));
    assert_eq!(files[b"result.txt".as_slice()], b"new\n");
    assert_eq!(files[b"original.txt".as_slice()], b"edited\n");
    assert_eq!(files[b"checks-ran.txt".as_slice()], b"checked\n");
}

#[test]
fn local_host_io_failure_keeps_exit_1() {
    let scratch = Scratch::new();
    let _file = support::user_layer(&scratch, &support::fake_provider_settings(&scratch));
    let state = scratch.path().join("state");
    std::fs::write(&state, b"a file cannot be the state directory").expect("block state directory");
    let ran = support::smith(&scratch, &["exec", "hello", "--chat", "refused"], b"");
    assert_eq!(ran.code, 1);
    assert!(String::from_utf8_lossy(&ran.stderr).contains("Not a directory"));
    assert_eq!(std::fs::read(state).expect("unchanged state file"), b"a file cannot be the state directory");
}

#[test]
fn exec_in_a_repository_without_a_commit_exits_two_with_its_line() {
    let scratch = Scratch::new();
    let _file = support::user_layer(&scratch, &support::fake_provider_settings(&scratch));
    let (ran, requests) = support::exec_without_head(&scratch);
    assert_eq!(ran.code, 2);
    let stderr = std::str::from_utf8(&ran.stderr).expect("operator lines are UTF-8");
    assert_eq!(stderr.lines().last(), Some("smith: run refused: repository workspace: its head cannot be read"));
    assert!(requests.is_empty(), "a refused start sends no provider request");
}

#[test]
fn local_chat_store_io_failure_keeps_exit_1() {
    use std::fmt::Write as _;
    let scratch = Scratch::new();
    let _file = support::user_layer(&scratch, &support::fake_provider_settings(&scratch));
    let workspace = scratch.path().canonicalize().unwrap();
    let digest = skein_io::digest::digest(workspace.as_os_str().as_encoded_bytes());
    let mut key = String::new();
    for part in digest.0 {
        write!(&mut key, "{part:016x}").expect("fixture workspace key");
    }
    let state = scratch.path().join("state/smith/chats").join(key);
    std::fs::create_dir_all(&state).expect("state root");
    let chat = state.join("refused");
    std::fs::write(&chat, b"a file cannot be the chat directory").expect("block chat directory");
    let ran = support::smith(&scratch, &["exec", "hello", "--chat", "refused"], b"");
    assert_eq!(ran.code, 1);
    assert!(String::from_utf8_lossy(&ran.stderr).contains("File exists"), "{}", String::from_utf8_lossy(&ran.stderr));
    assert_eq!(std::fs::read(chat).expect("unchanged chat file"), b"a file cannot be the chat directory");
}

#[test]
fn smith_agent_spawned_by_the_sample_host_answers() {
    support::sample_host::run();
}

#[test]
fn smith_check_json_shows_each_value_with_its_layer() {
    let scratch = Scratch::new();
    let _file = support::user_layer(&scratch, b"model='codex/gpt-6-luna'\neffort='low'\n[accounts.codex]\nsource='env'\nvariable='SMITH_FIXTURE_TOKEN'\n[endpoints.codex]\naddress='127.0.0.1:443'\n");
    let directory = scratch.path().join(".smith");
    std::fs::create_dir(&directory).expect("repository layer");
    std::fs::write(directory.join("config.toml"), b"[environment.set]\nREPO='repository-value'\n")
        .expect("repository environment");
    let ran = support::smith(&scratch, &["check", "--json", "-c", "effort='high'"], b"");
    assert_eq!(ran.code, 3, "{}", String::from_utf8_lossy(&ran.stderr));
    let report: serde_json::Value = serde_json::from_slice(&ran.stdout).expect("one check document");
    assert_eq!(report["version"], 2);
    assert_eq!(report["outside_models"]["anthropic/claude-opus-5-5"], "sub_agents is off");
    assert_eq!(report["settings"]["values"]["model"]["origin"]["layer"], "user");
    assert_eq!(report["settings"]["values"]["effort"]["value"], "high");
    assert_eq!(report["settings"]["values"]["effort"]["origin"]["kind"], "override");
    assert_eq!(report["settings"]["values"]["environment.set.REPO"]["origin"]["layer"], "repository");
    assert!(!String::from_utf8_lossy(&ran.stdout).contains("repository-value"));
    assert_eq!(
        report["formats"],
        serde_json::json!({"channel":5,"charter":4,"transcript":6,"events":4,"chat_store":1,"check":2})
    );
    for tool in ["sh", "rg", "git"] {
        assert!(report["tools"][tool].as_str().expect("tool path").starts_with('/'));
    }
    assert_eq!(report["agent"], serde_json::json!({"name":"smith", "version":env!("CARGO_PKG_VERSION")}));
    assert!(report.get("build").is_none(), "check reports the agent name and version");
}

#[test]
fn smith_check_names_both_sides_of_a_budget_past_the_profile_ceiling() {
    let scratch = Scratch::new();
    let _file = support::user_layer(&scratch, b"model='codex/gpt-6-luna'\n[accounts.codex]\nsource='env'\nvariable='SMITH_FIXTURE_TOKEN'\n[endpoints.codex]\naddress='cannot-resolve.invalid:443'\n");
    let ran = support::smith(&scratch, &["check", "-c", "budget.turns=5000"], b"");
    assert_eq!(ran.code, 2);
    assert_eq!(
        String::from_utf8(ran.stderr).expect("UTF-8"),
        "smith: budget.turns = 5000 (from -c) exceeds the standard profile's ceiling of 4096 turns\n"
    );
    assert!(ran.stdout.is_empty());
}

#[test]
fn a_headless_run_that_would_wait_exits_5_naming_its_chat() {
    let scratch = Scratch::new();
    let _file = support::user_layer(&scratch, &support::fake_provider_settings(&scratch));
    let ran =
        support::exec_with_provider(&scratch, &["exec", "--chat", "plan", "-"], b"Ask first\n", support::Reply::Wait);
    assert_eq!(ran.code, 5, "{}", String::from_utf8_lossy(&ran.stderr));
    assert!(ran.stderr.ends_with(b"smith: needs input: answer with `smith exec --chat plan`\n"));
}

#[test]
fn smith_exec_json_ends_accepted_with_its_events_and_exit_0() {
    let scratch = Scratch::new();
    let _file = support::user_layer(&scratch, &support::fake_provider_settings(&scratch));
    let ran = support::exec_with_provider(&scratch, &["exec", "--json", "-"], b"Ask first\n", support::Reply::Accepted);
    assert_eq!(ran.code, 0, "{}", String::from_utf8_lossy(&ran.stderr));
    let records: Vec<serde_json::Value> = ran
        .stdout
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_slice(line).expect("event record"))
        .collect();
    assert_eq!(records.first().unwrap()["type"], "session.started");
    assert_eq!(records.first().unwrap()["mode"], "exec");
    assert_eq!(records.first().unwrap()["versions"]["chat_store"], 1);
    assert_eq!(records.last().unwrap()["type"], "session.ended");
    assert_eq!(records.last().unwrap()["exit"], 0);
    assert!(records.iter().any(|r| r["type"] == "run.completed"));
    assert!(skein_lib::bytes::find(&ran.stderr, b"smith: accepted in").is_some());
}

#[test]
fn an_account_without_a_credential_exits_3_before_any_request() {
    let scratch = Scratch::new();
    let _file = support::user_layer(&scratch, &support::fake_provider_settings(&scratch));
    std::fs::remove_file(scratch.path().join("auth.json")).unwrap();
    let ran = support::smith(&scratch, &["exec", "Ask first"], b"");
    assert_eq!(ran.code, 3);
    assert!(ran.stdout.is_empty());
    assert!(!scratch.path().join("state/smith/chats").exists());
}

#[test]
fn an_empty_prompt_is_a_usage_error_exiting_2() {
    let scratch = Scratch::new();
    let _file = support::user_layer(&scratch, &support::fake_provider_settings(&scratch));
    let ran = support::smith(&scratch, &["exec", "-"], b" \n");
    assert_eq!(ran.code, 2);
    assert!(ran.stderr.ends_with(b"smith: usage: the prompt is empty\n"));
}

#[test]
fn the_trace_file_is_read_as_it_grows() {
    let scratch = Scratch::new();
    let _file = support::user_layer(&scratch, &support::fake_provider_settings(&scratch));
    let ran = support::exec_with_provider(
        &scratch,
        &["exec", "--trace", "events.jsonl", "-"],
        b"Ask first\n",
        support::Reply::Accepted,
    );
    assert_eq!(ran.code, 0, "{}", String::from_utf8_lossy(&ran.stderr));
    assert_eq!(ran.stdout, b"First answer");
    let bytes = std::fs::read(scratch.path().join("events.jsonl")).unwrap();
    assert!(bytes.ends_with(b"\n"));
    for line in bytes.split(|byte| *byte == b'\n').filter(|line| !line.is_empty()) {
        let _record: serde_json::Value = serde_json::from_slice(line).unwrap();
    }
}

#[test]
fn smith_exec_replaces_the_result_file_with_only_its_text() {
    let scratch = Scratch::new();
    let _file = support::user_layer(&scratch, &support::fake_provider_settings(&scratch));
    let path = scratch.path().join("answer.txt");
    std::fs::write(&path, b"an older and longer result\n").unwrap();
    let ran = support::exec_with_provider(
        &scratch,
        &["exec", "-o", "answer.txt", "-"],
        b"Ask first\n",
        support::Reply::Accepted,
    );
    assert_eq!(ran.code, 0, "{}", String::from_utf8_lossy(&ran.stderr));
    assert_eq!(std::fs::read(path).unwrap(), b"First answer");
    assert_eq!(ran.stdout, b"First answer");
    assert!(
        ran.stderr
            .split(|byte| *byte == b'\n')
            .rfind(|line| !line.is_empty())
            .unwrap()
            .starts_with(b"smith: accepted in ")
    );
}

#[test]
fn smith_exec_json_writes_its_result_file_beside_the_event_stream() {
    let scratch = Scratch::new();
    let _file = support::user_layer(&scratch, &support::fake_provider_settings(&scratch));
    let ran = support::exec_with_provider(
        &scratch,
        &["exec", "--json", "-o", "answer.txt", "-"],
        b"Ask first\n",
        support::Reply::Accepted,
    );
    assert_eq!(ran.code, 0, "{}", String::from_utf8_lossy(&ran.stderr));
    assert_eq!(std::fs::read(scratch.path().join("answer.txt")).unwrap(), b"First answer");
    let records = output_records(&ran.stdout);
    assert_eq!(records.last().unwrap()["type"], "session.ended");
    assert_eq!(records.last().unwrap()["exit"], 0);
    assert_eq!(records.iter().find(|record| record["type"] == "run.completed").unwrap()["status"], "accepted");
}

#[test]
fn smith_exec_result_file_failure_exits_1_after_an_accepted_run() {
    let scratch = Scratch::new();
    let _file = support::user_layer(&scratch, &support::fake_provider_settings(&scratch));
    std::fs::create_dir(scratch.path().join("answer.txt")).unwrap();
    let ran = support::exec_with_provider(
        &scratch,
        &["exec", "--json", "-o", "answer.txt", "-"],
        b"Ask first\n",
        support::Reply::Accepted,
    );
    assert_eq!(ran.code, 1, "{}", String::from_utf8_lossy(&ran.stderr));
    assert!(
        ran.stderr.ends_with(b"smith: run accepted, but its result could not be written to answer.txt: IsADirectory\n")
    );
    let records = output_records(&ran.stdout);
    assert_eq!(records.last().unwrap()["type"], "session.ended");
    assert_eq!(records.last().unwrap()["exit"], 1);
    assert_eq!(records.iter().find(|record| record["type"] == "run.completed").unwrap()["status"], "accepted");
    assert!(scratch.path().join("answer.txt").is_dir());
}

#[test]
fn smith_exec_result_file_failure_warns_before_a_parked_answers_last_line() {
    let scratch = Scratch::new();
    let _file = support::user_layer(&scratch, &support::fake_provider_settings(&scratch));
    std::fs::create_dir(scratch.path().join("answer.txt")).unwrap();
    let ran = support::exec_with_provider(
        &scratch,
        &["exec", "--chat", "plan", "--json", "-o", "answer.txt", "-"],
        b"Ask first\n",
        support::Reply::Wait,
    );
    assert_eq!(ran.code, 5, "{}", String::from_utf8_lossy(&ran.stderr));
    assert!(ran.stderr.ends_with(b"smith: warning: its result could not be written to answer.txt: IsADirectory\nsmith: needs input: answer with `smith exec --chat plan`\n"));
    let records = output_records(&ran.stdout);
    assert_eq!(records.last().unwrap()["exit"], 5);
    assert_eq!(records.iter().find(|record| record["type"] == "run.completed").unwrap()["status"], "parked");
}

fn output_records(bytes: &[u8]) -> Vec<serde_json::Value> {
    bytes
        .split(|byte| *byte == b'\n')
        .filter(|line| !line.is_empty())
        .map(|line| serde_json::from_slice(line).expect("event record"))
        .collect()
}

#[test]
fn smith_exec_ephemeral_removes_its_private_files_after_acceptance() {
    let scratch = Scratch::new();
    let _file = support::user_layer(&scratch, &support::fake_provider_settings(&scratch));
    let ran = support::exec_with_provider(
        &scratch,
        &["exec", "--ephemeral", "-o", "answer.txt", "-"],
        b"Ask first\n",
        support::Reply::Accepted,
    );
    assert_eq!(ran.code, 0, "{}", String::from_utf8_lossy(&ran.stderr));
    assert_eq!(ran.stdout, b"First answer");
    assert_eq!(std::fs::read(scratch.path().join("answer.txt")).unwrap(), b"First answer");
    assert!(ephemeral_directories(&scratch).is_empty());
    assert!(!String::from_utf8_lossy(&ran.stderr).contains("smith: warning: ephemeral directory "));
}

#[test]
fn smith_exec_ephemeral_removes_its_private_files_after_a_failed_answer() {
    let scratch = Scratch::new();
    let _file = support::user_layer(&scratch, &support::fake_provider_settings(&scratch));
    let ran = support::exec_with_provider(
        &scratch,
        &["exec", "--ephemeral", "--json", "-"],
        b"Ask first\n",
        support::Reply::Filtered,
    );
    assert_eq!(ran.code, 1, "{}", String::from_utf8_lossy(&ran.stderr));
    let records = output_records(&ran.stdout);
    assert_eq!(records.last().unwrap()["type"], "session.ended");
    assert_eq!(records.last().unwrap()["exit"], 1);
    assert_eq!(records.iter().find(|record| record["type"] == "run.completed").unwrap()["status"], "failed");
    assert!(ephemeral_directories(&scratch).is_empty());
}

#[test]
fn smith_exec_ephemeral_keeps_a_foreign_file_and_warns_before_the_last_line() {
    use std::os::unix::fs::PermissionsExt;

    let scratch = Scratch::new();
    let _file = support::user_layer(&scratch, &support::fake_provider_settings(&scratch));
    let ran = support::exec_with_foreign_file(&scratch, &["exec", "--ephemeral", "-"], b"Ask first\n");
    assert_eq!(ran.code, 0, "{}", String::from_utf8_lossy(&ran.stderr));
    let entries = ephemeral_directories(&scratch);
    assert_eq!(entries.len(), 1);
    let directory = entries[0].clone();
    assert_eq!(std::fs::metadata(&directory).unwrap().permissions().mode() & 0o777, 0o700);
    assert_eq!(std::fs::read(directory.join("foreign.txt")).unwrap(), b"kept foreign content");
    assert_eq!(std::fs::read_dir(&directory).unwrap().count(), 1);
    let errors = std::str::from_utf8(&ran.stderr).unwrap();
    let mut last = errors.lines().rev();
    assert!(last.next().unwrap().starts_with("smith: accepted in "));
    assert_eq!(last.next().unwrap(), format!("smith: warning: ephemeral directory {}: NotEmpty", directory.display()));
}

#[test]
fn an_ephemeral_directory_is_removed_when_product_startup_refuses() {
    let scratch = Scratch::new();
    let _file = support::user_layer(&scratch, &support::fake_provider_settings(&scratch));
    let ran = support::smith(&scratch, &["exec", "--ephemeral", "--trace", "absent/events.jsonl", "message"], b"");
    assert_eq!(ran.code, 1, "{}", String::from_utf8_lossy(&ran.stderr));
    assert!(ran.stdout.is_empty());
    assert!(ephemeral_directories(&scratch).is_empty());
}

/// What an ephemeral run left in the scratch runtime directory, where its
/// private directory is made; none is ever made under the state directory
/// (smith/docs/design/shell.md 4.2).
fn ephemeral_directories(scratch: &Scratch) -> Vec<std::path::PathBuf> {
    let state = std::fs::read_dir(scratch.path().join("state")).into_iter().flatten();
    assert!(
        !state
            .map(|entry| entry.expect("scratch state entry").file_name())
            .any(|name| name.as_encoded_bytes().starts_with(b".smith-exec-")),
        "no ephemeral directory under the state directory"
    );
    std::fs::read_dir(scratch.path().join("runtime"))
        .expect("the scratch runtime directory holds the private directory")
        .map(|entry| entry.expect("scratch runtime entry").path())
        .collect()
}

fn login_settings(scratch: &Scratch, registration: &str) {
    let directory = scratch.path().join("config/smith");
    std::fs::create_dir_all(&directory).expect("scratch user layer");
    std::fs::write(
        directory.join("config.toml"),
        format!(
            "credentials='{}'\n[accounts.test]\nsource='sign_in'\n{registration}",
            scratch.path().join("credentials").display()
        ),
    )
    .expect("scratch registration");
}

#[test]
fn smith_login_signs_in_against_the_fake_issuer_and_exits_0() {
    let scratch = Scratch::new();
    login_settings(
        &scratch,
        &format!(
            "client_id='client'\nauthorization_url='http://{host}:34444/authorize'\ntoken_url='http://{host}:34444/token'\nredirect_uri='http://127.0.0.1:{redirect}/callback'\nscope='read'\ntoken_format='form'\n",
            host = smith_real_world::loopback(),
            redirect = smith_real_world::redirect_port()
        ),
    );
    let ran = support::login::sign_in(&scratch);
    assert_eq!(ran.code, 0);
    assert!(String::from_utf8_lossy(&ran.stderr).contains("account test: signed in"));
    let file = scratch.path().join("credentials/tokens/test");
    let secret = skein_shell::read_private_file(
        &file,
        smith_local_shell::local_host::token_limits().record_bytes,
        skein_shell::effective_user(),
    )
    .expect("kept private record");
    let record = skein_oauth::decode_record(secret.expose(), &smith_local_shell::local_host::token_limits())
        .expect("native record");
    assert!(record.access_token.as_ref() == b"access-new", "access credential retained");
    assert!(record.refresh_token.as_deref() == Some(b"refresh-new".as_slice()), "refresh credential retained");
    assert!(!scratch.path().join("state/smith/tokens").exists(), "credentials kept apart from chat state");
}
