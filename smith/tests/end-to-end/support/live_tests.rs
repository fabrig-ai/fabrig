//! Offline checks of live configuration (smith/docs/design/testing.md, 2.3).
use super::live::{root, setup};
use smith_real_world::Scratch;
use std::path::{Path, PathBuf};

#[test]
fn missing_live_root_names_the_variable() {
    assert_eq!(root(None).expect_err("unset root"), "$SMITH_LIVE unset");
    assert_eq!(root(Some(std::ffi::OsStr::new(""))).expect_err("empty root"), "$SMITH_LIVE unset");
}

#[test]
fn missing_chatgpt_names_the_sign_in() {
    assert_eq!(
        setup(Path::new("/dev/null"), "chatgpt", false),
        Err("no record in `$SMITH_LIVE/tokens/chatgpt`: run `XDG_CONFIG_HOME=$SMITH_LIVE/config smith login chatgpt`")
    );
}

#[test]
fn missing_claude_names_the_sign_in() {
    assert_eq!(
        setup(Path::new("/dev/null"), "claude", false),
        Err("no record in `$SMITH_LIVE/programs/claude`: run `XDG_CONFIG_HOME=$SMITH_LIVE/config smith login claude`")
    );
}

#[test]
fn missing_key_names_the_variable() {
    assert_eq!(setup(Path::new("/dev/null"), "anthropic-key", false), Err("`ANTHROPIC_API_KEY` is not set"));
    assert_eq!(setup(Path::new("/dev/null"), "anthropic-key", true), Ok(()));
}

fn synthetic_login(root: &Path) -> PathBuf {
    use std::os::unix::fs::{DirBuilderExt, PermissionsExt};
    let login = root.join("synthetic-login");
    std::fs::DirBuilder::new()
        .recursive(true)
        .mode(0o700)
        .create(login.join("tokens"))
        .expect("private synthetic login");
    let record = skein_oauth::SavedToken {
        key: 0,
        generation: 1,
        access_token: b"unissued-access".as_slice().into(),
        refresh_token: None,
        expires_at: skein_lib::Wall::from_nanos(u64::MAX),
        issued_client_id: None,
        granted_scopes: Some(
            b"openid profile email offline_access resource.invoke chatgpt.tokens.use.direct".as_slice().into(),
        ),
        subject: None,
        id_token: None,
        earliest_refresh_at: None,
        metadata: None,
    };
    let bytes = skein_oauth::encode_record(&record, &smith_local_shell::credentials::token_limits())
        .expect("synthetic native record");
    let file = login.join("tokens/chatgpt");
    std::fs::write(&file, &bytes).expect("synthetic record file");
    std::fs::set_permissions(file, std::fs::Permissions::from_mode(0o600)).expect("private record");
    login
}

fn pinned_product_check(conversations: i64) -> smith_bench::process::ProcessResult {
    use skein_lib::Duration;
    let scratch = Scratch::new();
    let root = scratch.path();
    let guard = smith_bench::guard::Guard::new(&root.join("user"), &root.join("config"), &root.join("state"), &[])
        .expect("guard");
    let private = smith_bench::isolate::empty(&guard, &root.join("attempt")).expect("private paths");
    let project = skein_scratch::paths::package().join("benchmarks");
    let pin_file = project.join("agents/smith/everything.pin.toml");
    let pin = smith_bench::read_configuration(&pin_file).expect("pin");
    let mut task = smith_bench::read_task(&project.join("tasks/probes/first-request/task.toml")).expect("task");
    task.smith.insert("deployment.conversations".into(), toml::Value::Integer(conversations));
    let model = smith_bench::ModelChoice { model: Some("gpt-6-luna".into()), effort: Some("low".into()) };
    let arm = smith_bench::arm::ArmRef::Installed { program: "/unused".into() };
    let environment = smith_bench::isolate::environment(
        &smith_bench::isolate::Machine { path: "/usr/bin:/bin".into(), rustup_home: None },
        &private,
        &std::collections::BTreeMap::new(),
    )
    .expect("environment");
    let clock = skein_shell::Clock::new();
    let login = synthetic_login(root);
    let context = smith_bench::agent::Context {
        guard: &guard,
        private: &private,
        pin: &pin,
        pin_file: &pin_file,
        task: &task,
        variant: None,
        model: &model,
        arm: &arm,
        installed_version: "unused",
        home: root,
        wall: clock.now().wall,
        lifetime: Duration::from_secs(20),
        environment: &environment,
    };
    let layer: toml::Value =
        toml::from_str(include_str!("../../../benchmarks/agents/smith/everything.toml")).expect("product layer");
    let base = serde_json::to_value(layer).expect("layer values");
    let mut rendered = smith_bench::agent::smith::product_settings(&context, &base, &login).expect("product rendering");
    let invocation = rendered.as_object_mut().expect("layer").remove("invocation").expect("product flags");
    let directory = private.config.join("smith");
    std::fs::create_dir(&directory).expect("user layer directory");
    let document: toml::Value = serde_json::from_value(rendered).expect("TOML document");
    std::fs::write(directory.join("config.toml"), toml::to_string(&document).expect("user layer"))
        .expect("scratch settings");
    let result = smith_bench::process::execute(
        &clock,
        skein_world::end_to_end::Command {
            program: skein_scratch::paths::binary("smith"),
            arguments: product_check_arguments(&invocation, &private.workspace, root),
            environment,
            directory: private.workspace.clone(),
        },
        Box::default(),
        smith_bench::process::Capture::Raw,
        smith_bench::process::Bounds {
            hard: Duration::from_secs(1),
            teardown: Duration::ZERO,
            grace: Duration::from_millis(20),
            settlement: Duration::from_secs(1),
            stdout: 1_048_576,
            stderr: 65_536,
            force_walk: true,
        },
    )
    .expect("check settles");
    assert!(!root.join("unopened").exists(), "check never creates execution output");
    assert_eq!(std::fs::read_dir(&private.state).expect("scratch state").count(), 0, "check writes no execution state");
    result
}

fn product_check_arguments(invocation: &serde_json::Value, workspace: &Path, root: &Path) -> Vec<std::ffi::OsString> {
    let mut arguments = vec![
        "check".into(),
        "--json".into(),
        "exec".into(),
        "--json".into(),
        "-C".into(),
        workspace.into(),
        "-o".into(),
        root.join("unopened/result").into(),
        "--ephemeral".into(),
    ];
    arguments
        .extend(invocation.as_array().expect("product flags").iter().map(|value| value.as_str().expect("flag").into()));
    arguments
}

#[test]
fn the_product_adapter_user_layer_passes_the_shipped_check() {
    let result = pinned_product_check(5);
    assert_eq!(result.exit, smith_bench::Exit::Code(0), "pinned product settings pass check");
    assert!(
        smith_bench::agent::smith::teardown_from_check(&result.stdout)
            .is_ok_and(|bound| bound == skein_lib::Duration::from_secs(100)),
        "check reports its teardown bound"
    );
}

use super::live_report::{Leg, Settled, Step, exec_report, file_report, stderr_lines};

fn failed_record() -> smith_events::Record {
    use smith_events::{
        AnswerClass, AnswerFailure, CompletionFailure, Event, Evidence, FailureClass, Record, RunCompleted, Status,
        Usage,
    };
    Record {
        t_ms: 0,
        event: Event::RunCompleted(RunCompleted {
            run: 1,
            wind_down: false,
            status: Status::Failed,
            failure: Some(AnswerFailure {
                class: AnswerClass::Model,
                endpoint: None,
                model: None,
                attempts: Some(3),
                completion: Some(CompletionFailure {
                    field: None,
                    code: Some(b"private-code".as_slice().into()),
                    request_id: Some(b"private-request".as_slice().into()),
                    class: FailureClass::Unauthorized,
                    evidence: Evidence::Response,
                    retry_after_ms: None,
                    phase: None,
                    limit: None,
                    status: Some(401),
                    detail: Some(b"private-provider-detail".as_slice().into()),
                }),
                account: None,
                reset_ms: None,
                state: None,
                code: None,
                program: None,
                exit: None,
                detail: Some(b"private-answer-detail".as_slice().into()),
                used: None,
                bound: None,
                which: None,
                reason: None,
                dropped: None,
                ended: None,
                turn: None,
            }),
            result: None,
            turns: 2,
            spent: 0,
            usage: Usage {
                input_tokens: None,
                cache_read_tokens: None,
                cache_write_tokens: None,
                output_tokens: None,
                reasoning_tokens: None,
            },
            read: None,
            unread: 0,
            duration_ms: 0,
        }),
    }
}

#[test]
fn failed_exec_reports_the_typed_cause() {
    let record = failed_record();
    let end =
        smith_bench::End::Failed { reason: smith_bench::FailureReason::Provider, detail: "private-end-detail".into() };
    let report = exec_report(
        &Leg { story: "synthetic_failure", account: "chatgpt", invocation: 2, step: Step::Exec },
        "live exec succeeds",
        &Settled {
            exit: smith_bench::Exit::Code(1),
            forced: smith_bench::Forced::No,
            error: None,
            live_pids: 0,
            stderr: b"smith: agent started\nprivate-stderr\n",
        },
        &[&record],
        Some(&end),
    );
    for fact in [
        "synthetic_failure",
        "chatgpt",
        "invocation=2",
        "step=Exec",
        "live exec succeeds",
        "exit=Code(1)",
        "end=Failed",
        "reason=Provider",
        "status=Failed",
        "class=Model",
        "completion.class=Unauthorized",
        "completion.evidence=Response",
        "completion.status=401",
    ] {
        assert!(report.contains(fact), "missing {fact}: {report}");
    }
    assert!(!report.contains("private-"), "private failure and stderr bytes stay out: {report}");
    println!("{report}");
}

#[test]
fn reports_exclude_captured_content_and_bound_many_records() {
    use smith_events::{
        Block, Effect, Event, Message, Model, Prompt, Record, ResponseStarted, Role, Source, ToolChoice, ToolStarted,
    };
    let prompt = Record {
        t_ms: 0,
        event: Event::ResponseStarted(ResponseStarted {
            run: 1,
            conversation: 1,
            response: 1,
            turn: 1,
            attempt: 1,
            window: 1,
            model: Model {
                endpoint: b"private-endpoint".as_slice().into(),
                model: b"private-model".as_slice().into(),
                window_tokens: 0,
                output_tokens: 0,
                effort: None,
                account: None,
                backend: smith_events::Backend::Unknown(b"private-backend".as_slice().into()),
            },
            messages: 1,
            output_tokens: 0,
            tool_choice: ToolChoice::Auto,
            prompt: Some(Prompt {
                system: None,
                tools: None,
                messages: skein_lib::List::from_boxed(Box::new([Message {
                    role: Role::User,
                    blocks: skein_lib::List::from_boxed(Box::new([Block::Text {
                        text: b"private-prompt".as_slice().into(),
                    }])),
                }])),
            }),
        }),
    };
    let tool = Record {
        t_ms: 0,
        event: Event::ToolStarted(ToolStarted {
            run: 1,
            conversation: 1,
            call: b"private-call".as_slice().into(),
            tool: b"private-tool".as_slice().into(),
            source: Source::Run,
            effect: Effect::Write,
            deadline_ms: None,
            input_bytes: 13,
            input: Some(b"private-input".as_slice().into()),
        }),
    };
    let mut failure = failed_record();
    if let Event::RunCompleted(completed) = &mut failure.event {
        completed.status = smith_events::Status::Unknown(b"private-status".as_slice().into());
    }
    let leg = Leg { story: "content_filter", account: "claude", invocation: 1, step: Step::Check };
    let settled = Settled {
        exit: smith_bench::Exit::Signal(9),
        forced: smith_bench::Forced::Killed,
        error: Some("synthetic harness fault"),
        live_pids: 2,
        stderr: b"private-stderr\n",
    };
    let short = exec_report(&leg, "captured content is absent", &settled, &[&prompt, &tool, &failure], None);
    assert!(short.contains("status=unknown"), "unknown enum gets a fixed name: {short}");
    assert!(!short.contains("private-"), "captured content stays out: {short}");
    let records = vec![&failure; 1000];
    let report = exec_report(&leg, "many records", &settled, &records, None);
    assert!(report.len() <= 4096, "report is bounded: {}", report.len());
    assert!(!report.contains("private-"), "bounded reports keep the same privacy rule");
    let long = exec_report(&leg, &"é".repeat(5000), &settled, &[], None);
    assert!(long.len() <= 4096, "UTF-8 report is bounded");
}

#[test]
fn file_mismatches_report_relations_without_content() {
    let expected = b"private-file\n";
    for (found, relation) in [
        (None, "missing"),
        (Some(b"".as_slice()), "empty"),
        (Some(b"private-file"), "expected without its final newline"),
        (Some(b"private-file\nextra"), "expected followed by 5 more bytes"),
        (Some(b"priv"), "a prefix of expected (4 of 13 bytes)"),
        (Some(b"prXvate-file\n"), "differs at byte 2 (expected 13 bytes, found 13)"),
    ] {
        let report = file_report(b"odd\n\xff.txt", expected, found);
        assert!(report.contains(relation), "{relation}: {report}");
        assert!(report.contains("odd\\n\\xff.txt"), "escaped path: {report}");
        for private in ["private-file", "extra", "prXvate"] {
            assert!(!report.contains(private), "file bytes stay out: {report}");
        }
    }
}

#[test]
fn stderr_keeps_only_complete_allow_listed_forms() {
    use smith_local_shell::credentials::Credential;
    let credentials = [
        Credential::NotSignedIn,
        Credential::SignInAgain,
        Credential::ProgramNotSignedIn { program: smith_agent_shell::config::ProgramName::ClaudeCode },
    ];
    let mut allowed: Vec<String> = [
        "smith: local host started",
        "smith: agent started",
        "smith: commands run in process groups and are not contained",
        "smith: chat chat",
        "smith: trust roots: 12 taken, 0 skipped, 1 malformed blocks, 2 other blocks",
        "smith: run failed: out of turns: used 2 of 3",
        "smith: run failed: out of spend: used 21 of 34",
        "smith: run failed: out of time: used 55 of 89",
    ]
    .into_iter()
    .map(str::to_owned)
    .collect();
    for credential in credentials {
        allowed.push(format!("smith: {}", credential.line("claude")));
        allowed.push(format!("smith: warning: {}", credential.line("claude")));
    }
    let rejected = [
        "private-stderr",
        "smith: agent started private-tail",
        "smith: trust roots: private taken, 0 skipped, 0 malformed blocks, 0 other blocks",
        "smith: run failed: out of turns: used private of 2",
        "smith: run failed: out of memory: used 1 of 2",
        "smith: run failed: out of time: used 1 of 2 private",
        "smith: account other has no credential: run `smith login other`",
        "smith: run failed: the agent private words (signal 9): private tail",
        "smith: run failed: the agent ended without answering (exit private): private tail",
        "smith: run failed: the agent ended without answering (exit 256): private tail",
        "smith: run failed: the agent ended without answering (signal 4294967296): private tail",
        "smith: run failed: the agent ended without answering (signal +9): private tail",
        "smith: run failed: the agent ended without answering (signal -9): private tail",
        "smith: run failed: the agent ended without answering (signal 09): private tail",
        "smith: run failed: the agent ended without answering (signal ): private tail",
        "smith: run failed: the agent ended without answering (signal 9 ); killed: private tail",
        "smith: run failed: the agent ended without answering (signal 9); stopped: private tail",
        "smith: run failed: the agent ended without answering (signal 9); killed; terminated: private tail",
        "smith: run failed: the agent ended without answering (signal 9) private tail",
        "smith: run failed: the agent ended without answering (signal 9):private tail",
        "smith: run failed: the agent ended without answering: private tail (signal 9)",
        "smith: run failed: the agent ended without answering (unknown)",
        "smith: run failed: the agent ended without answering",
    ];
    let stderr = format!("{}\n{}\n", allowed.join("\n"), rejected.join("\n"));
    let (bytes, lines, kept) = stderr_lines(stderr.as_bytes(), "claude");
    assert_eq!(bytes, stderr.len(), "all bytes counted");
    assert_eq!(lines, allowed.len().checked_add(rejected.len()).expect("small fixture"), "all lines counted");
    assert_eq!(kept, allowed, "only complete allow-listed lines survive");
    assert_eq!(stderr_lines(b"", "claude"), (0, 0, vec![]), "empty stderr has no lines");
    assert_eq!(stderr_lines(b"private\n\xff", "claude"), (9, 2, vec![]), "invalid UTF-8 counts without rendering");
}

#[test]
fn stderr_keeps_an_agent_failure_without_its_detail() {
    for words in [
        "could not start",
        "ended without answering",
        "broke the channel rules",
        "exceeded its receiving bound",
        "made no progress",
        "reached its wall time bound",
    ] {
        for end in [
            "exit 0",
            "exit 255",
            "signal 9",
            "signal 4294967295",
            "exit not observed",
            "not spawned",
            "inline run settled",
        ] {
            for forced in ["", "; terminated", "; killed"] {
                let expected = format!("smith: run failed: the agent {words} ({end}){forced}");
                for detail in ["", ": ", ": private tail: more private bytes"] {
                    let stderr = format!("{expected}{detail}\n");
                    let (bytes, lines, kept) = stderr_lines(stderr.as_bytes(), "claude");
                    assert_eq!(bytes, stderr.len(), "detail bytes are counted");
                    assert_eq!(lines, 1, "the failure is one line");
                    assert_eq!(kept, vec![expected.clone()], "only the fixed failure survives: {stderr}");
                }
            }
        }
    }
}

#[test]
fn an_unknown_notice_is_named_by_its_kind() {
    use smith_events::{Event, Level, Notice, NoticeKind, Record};
    let leg = Leg { story: "notice_kinds", account: "claude", invocation: 1, step: Step::Exec };
    let settled = Settled {
        exit: smith_bench::Exit::Code(1),
        forced: smith_bench::Forced::No,
        error: None,
        live_pids: 0,
        stderr: b"",
    };
    for (kind, expected) in [
        (b"not_contained".as_slice(), "not_contained"),
        (b"a0_", "a0_"),
        (b"abcdefghijklmnopqrstuvwxyz012345", "abcdefghijklmnopqrstuvwxyz012345"),
        (b"abcdefghijklmnopqrstuvwxyz0123456", "unknown"),
        (b"abcdefghijklmnopqrstuvwxyz01234567890123", "unknown"),
        (b"not-contained", "unknown"),
        (b"not contained", "unknown"),
        (b"Not_contained", "unknown"),
        (b"not_contained\n", "unknown"),
        (b"\xff", "unknown"),
        (b"", "unknown"),
    ] {
        let record = Record {
            t_ms: 0,
            event: Event::Notice(Notice {
                level: Level::Warning,
                kind: NoticeKind::Unknown(kind.into()),
                run: None,
                account: None,
                reset_ms: None,
                code: Some(b"private-notice-code".as_slice().into()),
                bytes: None,
                exit: None,
                forced: None,
            }),
        };
        let report = exec_report(&leg, "notice kind", &settled, &[&record], None);
        let notice = report.lines().find(|line| line.starts_with("session=1 notice"));
        assert_eq!(notice, Some(format!("session=1 notice kind={expected}").as_str()), "bounded kind: {kind:?}");
        assert!(!report.contains("private-"), "notice metadata stays out: {report}");
    }
}

#[test]
fn end_reports_keep_terminals_and_omit_private_metadata() {
    use smith_bench::{BudgetLimit, End, FailureReason};
    let leg = Leg { story: "terminals", account: "chatgpt", invocation: 1, step: Step::Exec };
    let settled = Settled {
        exit: smith_bench::Exit::Code(1),
        forced: smith_bench::Forced::Terminated,
        error: Some("synthetic finish fault"),
        live_pids: 0,
        stderr: b"",
    };
    for (end, facts) in [
        (End::Completed, vec!["end=Completed"]),
        (
            End::Failed { reason: FailureReason::Host, detail: "private-end-detail".into() },
            vec!["end=Failed", "reason=Host"],
        ),
        (End::Budget { which: BudgetLimit::Time }, vec!["end=Budget", "which=Time"]),
        (End::Refused { setup: "private-setup".into() }, vec!["end=Refused"]),
        (End::Timeout, vec!["end=Timeout"]),
        (
            End::HarnessError { what: "synthetic classification fault".into() },
            vec!["end=HarnessError", "what=synthetic classification fault"],
        ),
        (
            End::HarnessError { what: "unknown run status private-status".into() },
            vec!["what=unknown run status unknown"],
        ),
        (
            End::HarnessError { what: "unknown answer failure class private-class".into() },
            vec!["what=unknown answer failure class unknown"],
        ),
        (
            End::HarnessError { what: "unknown session mode private-mode".into() },
            vec!["what=unknown session mode unknown"],
        ),
    ] {
        let report = exec_report(&leg, "typed terminal", &settled, &[], Some(&end));
        assert!(report.contains("error=synthetic finish fault"), "finish fault reaches the error line: {report}");
        for fact in facts {
            assert!(report.contains(fact), "{fact}: {report}");
        }
        assert!(!report.contains("private-"), "terminal content stays out: {report}");
    }
}

fn synthetic_session() -> smith_events::Record {
    use smith_events::{Agent, Capture, Containment, Delivery, Event, Mode, Record, SessionStarted, Versions};
    Record {
        t_ms: 0,
        event: Event::SessionStarted(SessionStarted {
            wall_ms: 0,
            agent: Agent { name: b"private-agent".as_slice().into(), version: b"private-version".as_slice().into() },
            mode: Mode::Exec,
            chat: None,
            containment: Containment::Groups,
            pid: 0,
            profile: b"private-profile".as_slice().into(),
            capture: Capture::Everything,
            delivery: Delivery::Complete,
            versions: Versions {
                events: smith_events::VERSION,
                channel: 0,
                charter: 0,
                transcript: 0,
                chat_store: None,
            },
            limits: skein_lib::List::uncapped(),
        }),
    }
}

fn synthetic_run_start() -> smith_events::Record {
    use smith_events::{Event, Record};
    Record {
        t_ms: 0,
        event: Event::RunStarted(smith_events::RunStarted {
            run: 1,
            resumed: true,
            messages: 7,
            main: smith_events::Model {
                endpoint: b"private-endpoint".as_slice().into(),
                model: b"private-model".as_slice().into(),
                window_tokens: 0,
                output_tokens: 0,
                effort: None,
                account: None,
                backend: smith_events::Backend::ClaudeCode,
            },
            budget: smith_events::Budget {
                turns: 8,
                spend: 9,
                time_ms: 10,
                reserve: smith_events::Reserve { turns: 0, time_ms: 0, spend: 0 },
            },
            tools: smith_events::Tools {
                families: skein_lib::List::uncapped(),
                withheld: skein_lib::List::uncapped(),
                wait: false,
                deliver: false,
                host: skein_lib::List::uncapped(),
            },
            contract: skein_lib::List::uncapped(),
        }),
    }
}

fn synthetic_session_end() -> smith_events::Record {
    use smith_events::{Event, Loss, Record, SessionEnded};
    Record {
        t_ms: 0,
        event: Event::SessionEnded(SessionEnded {
            exit: 3,
            answered: false,
            aborted: false,
            teardown_ms: None,
            emitted: skein_lib::List::uncapped(),
            loss: Loss { events: 0, channel: None },
            cpu_ms: None,
            peak_rss_bytes: None,
        }),
    }
}

#[test]
fn session_reports_keep_start_and_terminal_facts_without_tool_names_from_peers() {
    use smith_events::{
        AccountState, AgentExit, CheckCompleted, CheckExit, Delivered, Event, FailurePhase, Level, Notice, NoticeKind,
        Record, ToolCompleted, Verdict,
    };
    let mut failure = failed_record();
    if let Event::RunCompleted(completed) = &mut failure.event {
        let answer = completed.failure.as_mut().expect("synthetic failure");
        answer.state = Some(AccountState::Unauthorized);
        answer.exit = Some(AgentExit::Code(3));
        answer.ended = Some(AgentExit::Signal(9));
        answer.used = Some(4);
        answer.bound = Some(5);
        answer.turn = Some(6);
        answer.which = Some(b"turns".as_slice().into());
        answer.reason = Some(b"store".as_slice().into());
        answer.completion.as_mut().expect("synthetic completion").phase = Some(FailurePhase::Head);
    }
    let records = [
        synthetic_session(),
        synthetic_run_start(),
        failure,
        Record {
            t_ms: 0,
            event: Event::CheckCompleted(CheckCompleted {
                run: 1,
                exit: CheckExit::Code { code: 1 },
                passed: false,
                duration_ms: 0,
            }),
        },
        Record {
            t_ms: 0,
            event: Event::ToolCompleted(ToolCompleted {
                run: 1,
                conversation: 1,
                call: b"private-call".as_slice().into(),
                tool: b"finish".as_slice().into(),
                verdict: Verdict::Failed,
                exit: None,
                job: None,
                bytes: 0,
                omitted_bytes: 0,
                clamped: false,
                duration_ms: 0,
                child: None,
                delivery: Some(Delivered::Refused),
            }),
        },
        Record {
            t_ms: 0,
            event: Event::Notice(Notice {
                level: Level::Warning,
                kind: NoticeKind::CredentialUnauthorized,
                run: Some(1),
                account: None,
                reset_ms: None,
                code: Some(b"private-notice".as_slice().into()),
                bytes: None,
                exit: None,
                forced: None,
            }),
        },
        synthetic_session(),
        synthetic_session_end(),
    ];
    let references: Vec<_> = records.iter().collect();
    let leg = Leg { story: "session_facts", account: "claude", invocation: 3, step: Step::Exec };
    let settled = Settled {
        exit: smith_bench::Exit::Code(3),
        forced: smith_bench::Forced::No,
        error: None,
        live_pids: 0,
        stderr: b"",
    };
    let report = exec_report(&leg, "session facts", &settled, &references, None);
    for fact in [
        "session=1 run.started resumed=true messages=7 backend=ClaudeCode",
        "session=1 run.completed",
        "state=Unauthorized",
        "exit=Code(3)",
        "ended=Signal(9)",
        "used=4",
        "bound=5",
        "turn=6",
        "which=turns",
        "reason=store",
        "completion.phase=Head",
        "check.completed exit=Code(1) passed=false",
        "tool=finish verdict=Failed delivery=Refused",
        "kind=CredentialUnauthorized",
        "session=2 session.ended exit=3 answered=false",
    ] {
        assert!(report.contains(fact), "{fact}: {report}");
    }
    assert!(!report.contains("private-"), "only session facts survive: {report}");
}

#[test]
fn unknown_tool_fields_keep_only_fixed_names() {
    use smith_events::{ConversationClosed, ConversationEnd, Delivered, Event, Record, ToolCompleted, Verdict};
    let leg = Leg { story: "unknown_tools", account: "claude", invocation: 1, step: Step::Exec };
    let settled = Settled {
        exit: smith_bench::Exit::Code(1),
        forced: smith_bench::Forced::No,
        error: None,
        live_pids: 0,
        stderr: b"",
    };
    let failure = failed_record();
    let Event::RunCompleted(completed) = &failure.event else { panic!("synthetic answer") };
    let closed = Record {
        t_ms: 0,
        event: Event::ConversationClosed(ConversationClosed {
            run: 1,
            conversation: 1,
            end: ConversationEnd::Failed,
            which: None,
            failure: None,
            turns: 2,
            usage: completed.usage.clone(),
            spent: 0,
            duration_ms: 0,
        }),
    };
    let tool = Record {
        t_ms: 0,
        event: Event::ToolCompleted(ToolCompleted {
            run: 1,
            conversation: 1,
            call: b"private-call".as_slice().into(),
            tool: b"private-tool-name".as_slice().into(),
            verdict: Verdict::Unknown(b"private-verdict".as_slice().into()),
            exit: None,
            job: None,
            bytes: 0,
            omitted_bytes: 0,
            clamped: false,
            duration_ms: 0,
            child: None,
            delivery: Some(Delivered::Unknown(b"private-delivery".as_slice().into())),
        }),
    };
    let report = exec_report(&leg, "unknown tool facts", &settled, &[&closed, &tool], None);
    assert!(report.contains("conversation.closed end=Failed"), "conversation terminal: {report}");
    assert!(report.contains("tool=unknown verdict=unknown delivery=unknown"), "unknown tool fields: {report}");
    assert!(!report.contains("private-"), "unknown payloads stay out: {report}");
}
