use super::*;

#[test]
fn a_small_declared_memory_checks() {
    let files = crate::credentials::tests::Files::new("smith-credentials");
    crate::credentials::tests::private_key(&files);
    let certificate = crate::fixture::trust();
    let text = format!(
        "credentials='{}'\nmodel='openai/gpt-6-luna'\n\
         [deployment]\nmemory=1\n[environment]\ninherit=['PATH']\n\
         [accounts.codex]\nprovider='openai'\nsource='key'\n[endpoints.openai]\n\
         trust.system=false\ntrust.der=['{}']\n[trace]\npath='unopened-events'\n",
        files.path().display(),
        certificate.display()
    );
    let mut settings = crate::fixture::ready(settings::merge(
        &command::Shared::default(),
        files.path().to_path_buf(),
        &[settings::Source { layer: settings::Layer::User, path: files.path().join("check.toml"), text }],
    ));
    settings.paths.path = Some("/usr/bin:/bin".into());
    let execution = command::Exec {
        shared: command::Shared::default(),
        prompt: command::Prompt::Argument(b"hello".as_slice().into()),
        output: Some(files.path().join("unopened-result")),
        ephemeral: true,
        deliver: false,
        json: true,
    };
    for invocation in [Some(&execution), None] {
        let report = match report_for(&settings, skein_lib::Wall::EPOCH, &skein_shell::Clock::new(), invocation) {
            Verdict::Ready(report) => report,
            Verdict::Refused(why) => panic!("{}", why.line()),
        };
        assert!(report.credentials_ready(), "check succeeds with the synthetic key");
        let declaration = report
            .named_limits
            .iter()
            .find(|limit| limit["name"] == "declared.memory")
            .expect("emitted memory declaration");
        assert_eq!(declaration["value"], 1);
    }
    assert!(!files.path().join("unopened-result").exists());
    assert!(!files.path().join("unopened-events").exists());
}

#[test]
fn interactive_checks_enable_the_live_front() {
    let mut resolved = execution_fixture();
    let (limits, _) = interactive_limits(&mut resolved).expect("interactive admission");
    let events = resolved.agent.service.limits.events.expect("interactive sink");
    assert!(events.front);
    assert!(smith_local_service::valid(&limits));
}

fn resolved(trace: bool) -> settings::resolve::Resolved {
    let certificate = crate::fixture::trust().display().to_string();
    let trace = if trace { "[trace]\npath='/unopened-events'\n" } else { "" };
    let text = format!(
        "model='openai/gpt-6-luna'\n[deployment]\nmemory=8589934592\n[accounts.openai]\nprovider='openai'\nsource='env'\nvariable='SMITH_FIXTURE_TOKEN'\n[endpoints.openai]\ntrust.system=false\ntrust.der=['{certificate}']\n{trace}"
    );
    let settings = crate::fixture::settings(&text);
    crate::fixture::ready(settings::resolve::resolve(&settings))
}

#[test]
fn check_execs_teardown_includes_only_its_result_file_deadlines() {
    let mut resolved = execution_fixture();
    for (output, milliseconds) in [(false, 10_000), (true, 12_000)] {
        let mut arguments = vec!["check", "--json", "exec"];
        if output {
            arguments.extend(["-o", "/unopened-output-parent/result"]);
        }
        let Command::Check { execution: Some(execution), .. } =
            command::parse(arguments.into_iter().map(OsString::from).collect()).expect("check exec arguments")
        else {
            panic!("invocation-aware check");
        };
        let inventory = invocation_limits(&mut resolved, Some(&execution), skein_lib::Wall::from_nanos(0))
            .expect("pure invocation derivation")
            .expect("effective inventory");
        let document = named_limits(&inventory, &resolved.agent).expect("check's named JSON limits");
        assert!(document.iter().any(|limit| limit["name"] == "domain.content_edge_records"
            && limit["value"] == 1024
            && limit["unit"] == "count"));
        for removed in ["run.facts", "session.facts", "tools.facts", "inline.facts"] {
            assert!(document.iter().all(|limit| limit["name"] != removed), "{removed}");
        }
        let teardown = document.iter().find(|limit| limit["name"] == "service.teardown").expect("teardown bound");
        assert_eq!(teardown["unit"], "ms");
        assert_eq!(teardown["value"], milliseconds);
    }
}

fn execution_fixture() -> settings::resolve::Resolved {
    let mut resolved = crate::fixture::resolved("/workspace".into());
    let profile = smith_agent_process_world::profile();
    resolved.limits.agent = smith_local_domain::AgentLimits::inline(
        resolved.agent.service.limits.domain,
        2,
        skein_lib::Duration::from_secs(3600),
        profile.policy.cancel_grace,
    )
    .expect("product inline fixture");
    resolved.agent.memory = 8_589_934_592;
    resolved.agent.profile = profile;
    resolved
}

#[test]
fn check_derives_execs_event_destinations_through_its_stream_owner() {
    for (trace, json, destinations) in [(false, false, 0), (false, true, 1), (true, false, 1), (true, true, 2)] {
        let mut resolved = resolved(trace);
        let invocation = command::Exec {
            shared: command::Shared::default(),
            prompt: command::Prompt::Stdin,
            output: Some("/unopened-output-parent/result".into()),
            ephemeral: false,
            deliver: false,
            json,
        };
        let (limits, result_file) =
            derive_invocation(&mut resolved, &invocation, skein_lib::Wall::from_nanos(0), false)
                .expect("pure invocation derivation");
        let agent = &resolved.agent;
        let events = agent.service.limits.events.as_ref();
        assert_eq!(events.map_or(0, |events| events.sinks.destinations), destinations);
        let inventory = smith_local_service::profile::effective(
            &agent.profile,
            &agent.declarations,
            &agent.service,
            &limits,
            result_file,
        )
        .expect("one effective limits owner");
        assert!(inventory.iter().any(|limit| limit.name.field == "teardown"));
    }
}

#[test]
fn interactive_chat_checks_derive_accounts_endpoints_and_the_live_sink() {
    // The smoke-tools Chat arm: standard profile, openai/gpt-6-luna
    // selected at low effort, an env token and an everything trace.
    // Invocation derivation checks the selected sources without reading the variable.
    let certificate = crate::fixture::trust().display().to_string();
    let text = format!(
        "model='openai/gpt-6-luna'\neffort='low'\n\
         [accounts.openai]\nprovider='openai'\nsource='env'\nvariable='SMITH_FIXTURE_TOKEN'\n\
         [endpoints.openai]\ntrust.system=false\ntrust.der=['{certificate}']\n\
         [trace]\npath='/unopened-events'\ncapture='everything'\n"
    );
    let settings = crate::fixture::ready(settings::merge(
        &command::Shared::default(),
        "/workspace".into(),
        &[settings::Source { layer: settings::Layer::User, path: "chat-start.toml".into(), text }],
    ));
    let mut resolved = crate::fixture::ready(settings::resolve::resolve(&settings));
    assert_eq!(resolved.accounts.len(), 1);
    assert_eq!(resolved.limits.endpoints.len(), 1);

    // This is check's derive_invocation -> interactive_header path; its
    // derivation includes the trace header's event limits as well.
    interactive_limits(&mut resolved).expect("standard Chat startup checks");
    let events = resolved.agent.service.limits.events.expect("interactive trace and view");
    assert!(events.front);
    assert_eq!(events.sinks.destinations, 1);
}
