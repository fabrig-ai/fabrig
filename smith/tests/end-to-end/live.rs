//! Shipped smith on its own live accounts (smith/docs/design/testing.md, section 2.3).
//! The live profile selects these stories serially, without retries.

pub mod support;

pub use support::{live, live_report};

use live_report::{Leg, Settled, Step, exec_report, file_report};
use smith_events::{Block, Event};
use smith_local_process_world::referee::CheckoutRead;
use smith_real_world::Scratch;
use support::live_run::Expectation;

const CHANGE: &str = "In the working directory, read original.txt, replace its content with exactly edited followed by one newline, create result.txt containing exactly new followed by one newline, and use the command tool to create command.txt containing exactly ran followed by one newline. Run the configured .smith-test/check checks without modifying that script. Finish with a change result, a short title and body, so the host delivers the checked commit. Do not run git commit or push yourself.";

const STDOUT_BYTES: usize = 20_000_000;

fn changed_files() -> Vec<(Vec<u8>, Vec<u8>)> {
    [("original.txt", "edited\n"), ("result.txt", "new\n"), ("command.txt", "ran\n"), ("checks-ran.txt", "checked\n")]
        .into_iter()
        .map(|(path, content)| (path.as_bytes().to_vec(), content.as_bytes().to_vec()))
        .collect()
}

#[test]
fn chatgpt_delivers_a_commit_in_place() {
    commit("chatgpt");
}

#[test]
fn a_run_refreshes_its_grant_when_the_issuer_allows() {
    let mut environment = live::Environment::load(&["chatgpt"]);
    environment.refresh_lead = Some(live::REFRESH_LEAD_SECONDS);
    let clock = skein_shell::Clock::new();
    let mut before = environment.refresh_facts();
    let mut case = live::refresh_case(&before, clock.now().wall).unwrap_or_else(|message| panic!("{message}"));
    let waited = match case {
        live::RefreshCase::Wait { seconds } => {
            std::thread::sleep(std::time::Duration::from_secs(seconds));
            let waiting = before;
            before = environment.refresh_facts();
            let now = clock.now().wall;
            assert_eq!(
                live::refresh_case(&before, now),
                Ok(live::RefreshCase::Refreshed),
                "case=wait waited={seconds}s: the earliest refresh time must have passed; {}",
                live::refresh_report(&waiting, &before, now)
            );
            case = live::RefreshCase::Refreshed;
            format!(" waited={seconds}s")
        }
        live::RefreshCase::Refreshed | live::RefreshCase::Kept => String::new(),
    };
    let scratch = Scratch::new();
    let completed = change(
        &environment,
        Leg {
            story: "a_run_refreshes_its_grant_when_the_issuer_allows",
            account: "chatgpt",
            invocation: 1,
            step: Step::Exec,
        },
        &scratch,
        CHANGE,
        None,
        false,
    );
    let after = environment.refresh_facts();
    let refresh_report = live::refresh_report(&before, &after, clock.now().wall);
    match case {
        live::RefreshCase::Refreshed => assert!(
            after.generation > before.generation,
            "case=refreshed{waited}: the issuer refresh was kept; {refresh_report}\n{completed}"
        ),
        live::RefreshCase::Kept => assert_eq!(
            after.generation, before.generation,
            "case=kept: no refresh before the issuer's earliest refresh time; {refresh_report}\n{completed}"
        ),
        live::RefreshCase::Wait { .. } => unreachable!("the refresh case was resolved before the run"),
    }
}

#[test]
fn claude_delivers_a_commit_in_place() {
    commit("claude");
}

#[test]
#[ignore = "live: needs an Anthropic API key; run by the owner (smith/docs/design/testing.md, 2.3)"]
fn an_anthropic_key_delivers_a_commit_in_place() {
    commit("anthropic-key");
}

fn commit(account: &str) {
    let environment = live::Environment::load(&[account]);
    let scratch = Scratch::new();
    let story = match account {
        "chatgpt" => "chatgpt_delivers_a_commit_in_place",
        "claude" => "claude_delivers_a_commit_in_place",
        "anthropic-key" => "an_anthropic_key_delivers_a_commit_in_place",
        _ => unreachable!("live account"),
    };
    change(&environment, Leg { story, account, invocation: 1, step: Step::Exec }, &scratch, CHANGE, None, false);
}

#[test]
fn a_command_reads_the_whole_configured_environment() {
    let environment = live::Environment::load(&["chatgpt"]);
    let scratch = Scratch::new();
    run(
        &environment,
        Leg {
            story: "a_command_reads_the_whole_configured_environment",
            account: "chatgpt",
            invocation: 1,
            step: Step::Exec,
        },
        &scratch,
        "Use shell to run `printf '%s\\n' \"$SMITH_TEST_ENVIRONMENT\" > configured.txt`. Do not use a file-writing tool or substitute a literal value. Finish with a report.",
        Expectation { before: None, files: vec![(b"configured.txt".to_vec(), b"configured\n".to_vec())] },
        false,
        false,
    );
}

#[test]
fn a_second_run_resumes_the_chat_from_its_files() {
    let environment = live::Environment::load(&["chatgpt"]);
    let scratch = Scratch::new();
    let marker = format!("smith-memory-{}", std::process::id());
    let first = run(
        &environment,
        Leg {
            story: "a_second_run_resumes_the_chat_from_its_files",
            account: "chatgpt",
            invocation: 1,
            step: Step::Exec,
        },
        &scratch,
        &format!(
            "Remember this code for our next invocation: {marker}. Do not write files. Call finish with a report."
        ),
        Expectation { before: None, files: vec![] },
        false,
        false,
    );
    let before = turns(&scratch, &first);
    assert!(!before.is_empty(), "first invocation wrote durable turns\n{first}");
    assert!(!scratch.path().join("repo/remembered.txt").exists(), "code is kept only in the conversation\n{first}");
    let second = change(
        &environment,
        Leg {
            story: "a_second_run_resumes_the_chat_from_its_files",
            account: "chatgpt",
            invocation: 2,
            step: Step::Exec,
        },
        &scratch,
        &format!(
            "{CHANGE} Also create remembered.txt containing exactly the code I asked you to remember in the previous invocation, with no newline. Obtain it from the saved conversation."
        ),
        Some((b"remembered.txt".to_vec(), marker.into_bytes())),
        true,
    );
    let after = turns(&scratch, &second);
    assert!(after.len() > before.len(), "resume appended durable history\n{second}");
    assert!(before.iter().all(|entry| after.contains(entry)), "earlier turns remain unchanged\n{second}");
}

#[test]
fn a_chat_crosses_from_chatgpt_to_claude_then_back() {
    let environment = live::Environment::load(&["chatgpt", "claude"]);
    let scratch = Scratch::new();
    let mut completed = String::new();
    for (invocation, (account, filename, content, resumed)) in [
        ("chatgpt", "first.txt", "first\n", false),
        ("claude", "second.txt", "second\n", true),
        ("chatgpt", "third.txt", "third\n", true),
    ]
    .into_iter()
    .enumerate()
    {
        completed = run(
            &environment,
            Leg {
                story: "a_chat_crosses_from_chatgpt_to_claude_then_back",
                account,
                invocation: invocation.checked_add(1).expect("three legs"),
                step: Step::Exec,
            },
            &scratch,
            &format!(
                "Create {filename} in the working directory containing exactly {} followed by one newline. Finish with a report.",
                content.trim()
            ),
            Expectation { before: None, files: vec![(filename.as_bytes().to_vec(), content.as_bytes().to_vec())] },
            false,
            resumed,
        );
    }
    for (file, content) in
        [("first.txt", b"first\n".as_slice()), ("second.txt", b"second\n"), ("third.txt", b"third\n")]
    {
        let found = std::fs::read(scratch.path().join("repo").join(file)).ok();
        let misplaced = if found.is_none() { found_beneath(&scratch, file.as_bytes()) } else { None };
        assert!(
            found.as_deref() == Some(content),
            "each completed run keeps its file\n{completed}\n{}{}",
            file_report(file.as_bytes(), content, found.as_deref()),
            misplaced.as_deref().unwrap_or_default()
        );
    }
}

fn turns(scratch: &Scratch, report: &str) -> Vec<(std::path::PathBuf, Vec<u8>)> {
    let mut workspaces = std::fs::read_dir(scratch.path().join("state/smith/chats"))
        .unwrap_or_else(|_| panic!("workspace store\n{report}"));
    let directory = workspaces
        .next()
        .unwrap_or_else(|| panic!("one workspace\n{report}"))
        .unwrap_or_else(|_| panic!("workspace entry\n{report}"))
        .path()
        .join("chat");
    assert!(workspaces.next().is_none(), "only this story's workspace\n{report}");
    let mut files: Vec<_> = std::fs::read_dir(directory)
        .unwrap_or_else(|_| panic!("durable chat\n{report}"))
        .map(|entry| entry.unwrap_or_else(|_| panic!("turn entry\n{report}")).path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "turn"))
        .map(|path| {
            let bytes = std::fs::read(&path).unwrap_or_else(|_| panic!("durable turn\n{report}"));
            (path, bytes)
        })
        .collect();
    files.sort_by(|left, right| left.0.cmp(&right.0));
    files
}

fn change(
    environment: &live::Environment,
    leg: Leg<'_>,
    scratch: &Scratch,
    prompt: &str,
    extra: Option<(Vec<u8>, Vec<u8>)>,
    resumed: bool,
) -> String {
    let mut files = changed_files();
    files.extend(extra);
    run(environment, leg, scratch, prompt, Expectation { before: scratch.checkout().head(), files }, false, resumed)
}

#[expect(clippy::disallowed_types, reason = "independent live checkout and remote observations")]
fn git(path: &std::path::Path, arguments: &[&str]) -> std::process::Output {
    std::process::Command::new("/usr/bin/git")
        .env_clear()
        .env("HOME", path)
        .env("PATH", std::env::var_os("PATH").expect("PATH"))
        .arg("-C")
        .arg(path)
        .args(arguments)
        .output()
        .expect("live git command")
}

#[test]
fn a_configured_push_lands() {
    let environment = live::Environment::load(&["chatgpt"]);
    let Some(remote) = std::env::var_os("SMITH_TEST_LIVE_GIT_REMOTE").filter(|value| !value.is_empty()) else {
        eprintln!("configured push not run: SMITH_TEST_LIVE_GIT_REMOTE is absent");
        return;
    };
    let remote = remote.to_str().expect("remote is UTF-8");
    let scratch = Scratch::new();
    let repo = scratch.path().join("repo");
    let branch = format!("smith-live-{}-{}", std::process::id(), skein_shell::Clock::new().now().now.as_nanos());
    assert!(git(&repo, &["checkout", "-q", "-b", &branch]).status.success(), "unique live branch");
    assert!(git(&repo, &["remote", "add", "origin", remote]).status.success(), "selected remote");
    let mut completed = String::new();
    let outcome = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        completed = run(
            &environment,
            Leg { story: "a_configured_push_lands", account: "chatgpt", invocation: 1, step: Step::Exec },
            &scratch,
            CHANGE,
            Expectation { before: scratch.checkout().head(), files: changed_files() },
            true,
            false,
        );
        let seen = git(&repo, &["ls-remote", "origin", &format!("refs/heads/{branch}")]);
        assert!(seen.status.success(), "inspect remote branch; exit={:?}\n{completed}", seen.status.code());
        let head = scratch.checkout().head().unwrap_or_else(|| panic!("delivered head\n{completed}"));
        assert!(
            seen.stdout.split(u8::is_ascii_whitespace).next() == Some(head.as_slice()),
            "push landed at the delivered commit\n{completed}"
        );
    }));
    if completed.is_empty()
        && let Err(failure) = &outcome
        && let Some(report) = failure.downcast_ref::<String>()
    {
        completed.clone_from(report);
    }
    let cleanup = git(&repo, &["push", "origin", &format!(":refs/heads/{branch}")]);
    assert!(
        cleanup.status.success(),
        "remove remote branch even on story failure; exit={:?}\n{completed}",
        cleanup.status.code()
    );
    if let Err(failure) = outcome {
        std::panic::resume_unwind(failure);
    }
}

fn run(
    environment: &live::Environment,
    leg: Leg<'_>,
    scratch: &Scratch,
    prompt: &str,
    expectation: Expectation,
    push: bool,
    resumed: bool,
) -> String {
    use skein_lib::Duration;
    use skein_world::end_to_end::Command;
    use smith_bench::process::{Bounds, Capture, execute};
    let account = leg.account;
    let clock = skein_shell::Clock::new();
    let mut process_environment = vec![
        ("HOME".into(), scratch.path().join("home").into()),
        ("XDG_CONFIG_HOME".into(), environment.configuration().into()),
        ("XDG_STATE_HOME".into(), scratch.path().join("state").into()),
        ("PATH".into(), std::env::var_os("PATH").expect("live PATH")),
    ];
    std::fs::create_dir_all(scratch.path().join("home")).expect("private home");
    if account == "anthropic-key" {
        let key = std::env::var_os("ANTHROPIC_API_KEY").filter(|value| !value.is_empty()).expect(live::KEY);
        process_environment.push(("ANTHROPIC_API_KEY".into(), key));
    }
    let command = |arguments| Command {
        program: skein_scratch::paths::binary("smith"),
        arguments,
        environment: process_environment.clone(),
        directory: scratch.path().into(),
    };
    let bounds = |hard, teardown| Bounds {
        hard,
        teardown,
        grace: Duration::from_secs(2),
        settlement: Duration::from_secs(10),
        stdout: STDOUT_BYTES,
        stderr: 65_536,
        force_walk: true,
    };
    let change = expectation.before.is_some();
    let mut arguments = live::arguments(account, scratch.path(), change, push);
    let trace = scratch.path().join(format!("trace-{}.jsonl", leg.invocation));
    live::with_trace(&mut arguments, &trace);
    if let Some(lead) = environment.refresh_lead {
        live::with_refresh_lead(&mut arguments, lead);
    }
    let mut check_arguments = vec!["check".into(), "--json".into()];
    check_arguments.extend(arguments.clone());
    let check = execute(
        &clock,
        command(check_arguments),
        Box::default(),
        Capture::Raw,
        bounds(Duration::from_secs(live::CHECK_SECONDS), Duration::ZERO),
    )
    .expect("live check settles");
    let check_leg = Leg { step: Step::Check, ..leg };
    let state = check_state(&check.stdout, account);
    let checked = |failed: &str| {
        exec_report(
            &check_leg,
            &format!("{failed}; credentials.{account}.state={state}"),
            &Settled::of(&check),
            &[],
            None,
        )
    };
    assert!(check.exit == smith_bench::Exit::Code(0), "{}", checked("live check must pass"));
    let teardown = smith_bench::agent::smith::teardown_from_check(&check.stdout)
        .unwrap_or_else(|_| panic!("{}", checked("effective teardown bound")));
    let _configuration = environment.configuration();
    let mut result = execute(
        &clock,
        command(arguments),
        format!("{prompt}\n").into_bytes().into(),
        Capture::Agent(Box::new(smith_bench::agent::Observing::Smith(observer(
            environment,
            scratch,
            teardown,
            account,
            trace,
        )))),
        bounds(Duration::from_secs(live::EXEC_SECONDS), teardown),
    )
    .unwrap_or_else(|_| {
        panic!("shipped live exec settles\n{}", checked("check completed before exec settlement failed"))
    });
    assert_exec(&leg, &mut result, scratch, prompt, expectation, resumed)
}

fn check_state(stdout: &[u8], account: &str) -> &'static str {
    let parsed: Option<serde_json::Value> = serde_json::from_slice(stdout).ok();
    let state = parsed.as_ref().and_then(|value| value.get("credentials")?.get(account)?.get("state")?.as_str());
    match state {
        Some("usable") => "usable",
        Some("not_signed_in") => "not_signed_in",
        Some("sign_in_again") => "sign_in_again",
        Some("without_scope") => "without_scope",
        Some("program_not_signed_in") => "program_not_signed_in",
        Some("program_signed_in_otherwise") => "program_signed_in_otherwise",
        Some("program_failed") => "program_failed",
        Some("unreadable") => "unreadable",
        Some("key_unreadable") => "key_unreadable",
        Some("unset") => "unset",
        Some("expired_variable") => "expired_variable",
        Some(_) => "unknown",
        None => "absent",
    }
}

fn observer(
    environment: &live::Environment,
    scratch: &Scratch,
    teardown: skein_lib::Duration,
    account: &str,
    trace: std::path::PathBuf,
) -> smith_bench::agent::smith::Observing {
    let registry = smith_bench::agent::smith::FaceSince {
        commit: String::new(),
        shape: smith_bench::agent::smith::events::Shape::Exec,
        receiving: smith_bench::agent::smith::Receiving {
            string: 256_000,
            content: 13_056_000,
            items: 52_778,
            encoded_line: 19_817_472,
        },
    };
    let prepared = smith_bench::agent::smith::Prepared {
        face: registry.shape,
        registry,
        settings: environment.root.join("config/smith/config.toml"),
        serial: None,
        trace: Some(trace),
        rendered: live::rendered(account),
        program: skein_scratch::paths::binary("smith"),
        state: scratch.path().join("state"),
        teardown,
    };
    smith_bench::agent::smith::Observing::new(prepared).expect("outside observer")
}

#[expect(clippy::disallowed_types, reason = "bounded outside observation of the settled agent trace")]
fn agent_trace(path: Option<&std::path::Path>) -> Result<Vec<u8>, String> {
    use std::io::Read;
    let mut path = path.ok_or("program account requires a trace path")?.as_os_str().to_os_string();
    path.push(".agent");
    let mut bytes = Vec::new();
    std::fs::File::open(path)
        .map_err(|error| format!("agent trace open: {error}"))?
        .take(u64::try_from(STDOUT_BYTES).expect("stdout bound fits u64").saturating_add(1))
        .read_to_end(&mut bytes)
        .map_err(|error| format!("agent trace read: {error}"))?;
    if bytes.len() > STDOUT_BYTES {
        return Err("agent trace exceeds the stdout bound".into());
    }
    Ok(bytes)
}

fn assert_exec(
    leg: &Leg<'_>,
    result: &mut smith_bench::process::ProcessResult,
    scratch: &Scratch,
    prompt: &str,
    expectation: Expectation,
    resumed: bool,
) -> String {
    let change = expectation.before.is_some();
    let mut observing = result.observing.take();
    let Some(smith_bench::agent::Observing::Smith(observing)) = observing.as_deref_mut() else {
        panic!("{}", exec_report(leg, "exec observer", &Settled::of(result), &[], None))
    };
    let finish = observing.observer.finish();
    let trace = observing.prepared.program_source().then(|| agent_trace(observing.prepared.trace.as_deref()));
    if let Some(trace) = &trace {
        // agent_trace feeds and finishes the spawned agent's observer.
        observing.agent_trace(trace.as_deref().map_err(Clone::clone));
    }
    let end = observing.observer.classify(result.exit, result.forced).end;
    let detail_end = observing.events().classify(result.exit, result.forced).end;
    let records: Vec<_> = observing.events().sessions().iter().flat_map(|session| &session.records).collect();
    let mut reported = records.clone();
    if observing.prepared.program_source() {
        reported.extend(observing.observer.sessions().iter().flat_map(|session| &session.records));
    }
    let mut errors = Vec::new();
    errors.extend(result.error.iter().cloned());
    errors.extend(finish.as_ref().err().map(|fault| fault.what.clone()));
    errors.extend(trace.as_ref().and_then(|trace| trace.as_ref().err()).cloned());
    let error = (!errors.is_empty()).then(|| errors.join("; "));
    let settled = Settled { error: error.as_deref(), ..Settled::of(result) };
    let report = |failed: &str| exec_report(leg, failed, &settled, &reported, Some(&end));
    assert!(finish.is_ok(), "{}", report("complete typed event stream"));
    assert!(trace.as_ref().is_none_or(Result::is_ok), "{}", report("read the bounded agent trace"));
    assert!(
        result.error.is_none() && result.forced == smith_bench::Forced::No && result.tree.live_pids.is_empty(),
        "{}",
        report("live process tree settles")
    );
    assert!(result.exit == smith_bench::Exit::Code(0), "{}", report("live exec succeeds"));
    assert!(matches!(end, smith_bench::End::Completed), "{}", report("typed answer and exit agree"));
    assert!(
        matches!(detail_end, smith_bench::End::Completed),
        "{}",
        exec_report(leg, "the run's typed detail stream completes", &settled, &reported, Some(&detail_end))
    );
    assert!(
        records.iter().any(|record| matches!(&record.event, Event::RunStarted(start) if start.resumed == resumed)),
        "{}",
        report("run reports the expected resume state")
    );
    assert!(records.iter().any(|record| matches!(&record.event, Event::ResponseStarted(start) if start.prompt.as_ref().is_some_and(|captured| captured.messages.iter().any(|message| message.blocks.iter().any(|block| matches!(block, Block::Text { text } if skein_lib::bytes::find(text, prompt.as_bytes()).is_some())))))), "{}", report("the task appears in the typed prompt"));
    if leg.account == "claude" {
        program_contract(&records, leg, &settled, &end);
    }
    if change {
        assert!(records.iter().any(|record| matches!(&record.event, Event::ToolCompleted(tool) if tool.tool.as_ref() == b"finish" && tool.delivery == Some(smith_events::Delivered::Delivered))), "{}", report("finish delivered the change"));
        assert!(
            records.iter().any(|record| matches!(&record.event, Event::CheckCompleted(check) if check.passed)),
            "{}",
            report("the configured checks passed")
        );
        let head = scratch.checkout().head().unwrap_or_else(|| panic!("{}", report("delivered commit")));
        assert!(Some(&head) != expectation.before.as_ref(), "{}", report("a new commit was delivered"));
        assert!(
            skein_lib::bytes::find(&scratch.checkout().message(&head), b"Smith-Delivery:").is_some(),
            "{}",
            report("commit carries its delivery identity")
        );
        let files = scratch.checkout().files(&head);
        let original = scratch
            .checkout()
            .files(expectation.before.as_ref().unwrap_or_else(|| panic!("{}", report("initial commit"))));
        assert!(
            files.get(b".smith-test/check".as_slice()) == original.get(b".smith-test/check".as_slice()),
            "{}\n{}",
            report("the prescribed check stays unchanged"),
            file_report(
                b".smith-test/check",
                original.get(b".smith-test/check".as_slice()).map_or(&[], Vec::as_slice),
                files.get(b".smith-test/check".as_slice()).map(Vec::as_slice)
            )
        );
        for (path, content) in &expectation.files {
            assert!(
                files.get(path) == Some(content),
                "{}\n{}",
                report("committed file matches its prescribed content"),
                file_report(path, content, files.get(path).map(Vec::as_slice))
            );
        }
    }
    for (path, content) in expectation.files {
        let path_text = std::str::from_utf8(&path).unwrap_or_else(|_| panic!("{}", report("test path")));
        let found = std::fs::read(scratch.path().join("repo").join(path_text)).ok();
        let misplaced = if found.is_none() { found_beneath(scratch, &path) } else { None };
        assert!(
            found.as_deref() == Some(content.as_slice()),
            "{}\n{}{}",
            report("checkout file matches its prescribed content"),
            file_report(&path, &content, found.as_deref()),
            misplaced.as_deref().unwrap_or_default()
        );
    }
    report("completed")
}

fn found_beneath(scratch: &Scratch, path: &[u8]) -> Option<String> {
    let path = std::str::from_utf8(path).ok()?;
    scratch
        .path()
        .join("repo/repo")
        .join(path)
        .exists()
        .then(|| format!("\nfile {path:?}: found at repo/{path} beneath the working directory instead"))
}

fn program_contract(records: &[&smith_events::Record], leg: &Leg<'_>, settled: &Settled<'_>, end: &smith_bench::End) {
    let report = |failed: &str| exec_report(leg, failed, settled, records, Some(end));
    let started = records
        .iter()
        .find_map(|record| if let Event::RunStarted(start) = &record.event { Some(start) } else { None })
        .unwrap_or_else(|| panic!("{}", report("program admission event")));
    assert!(
        started.main.backend == smith_events::Backend::ClaudeCode,
        "{}",
        report("the product spawned the program backend")
    );
    assert!(
        started.tools.families.as_slice()
            == [smith_events::Family::Inspect, smith_events::Family::Modify, smith_events::Family::Shell],
        "{}",
        report("the program has exactly the granted capabilities")
    );
    let tools = records
        .iter()
        .find_map(|record| {
            if let Event::ResponseStarted(start) = &record.event {
                start.prompt.as_ref().and_then(|prompt| prompt.tools.as_ref()).filter(|tools| !tools.is_empty())
            } else {
                None
            }
        })
        .unwrap_or_else(|| panic!("{}", report("the first prompt's offered tools")));
    let tools: Vec<&[u8]> = tools.iter().map(AsRef::as_ref).collect();
    if let Some(fault) = support::program_contract::offered_tools_fault(&tools) {
        panic!("{}", report(&fault));
    }
}
