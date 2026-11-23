use super::*;
use crate::credentials::tests::Files;
use smith_host_domain::{FailureReason, ProgramCheck, ProgramFailure, ProgramMethod};
use std::{os::unix::fs::PermissionsExt, path::Path};

fn check_settings(files: &Files, model: &str, source: &str, extra: &str) -> Settings {
    let (endpoint, _) = model.split_once('/').expect("fixture model");
    let certificate = crate::fixture::trust();
    let path = crate::fixture::commands_path();
    let base = format!(
        "model='{model}'\ncredentials='{}'\n[accounts.{endpoint}]\nsource='{source}'\n\
         [endpoints.{endpoint}]\naccount='{endpoint}'\naddress='127.0.0.1:443'\ntrust.system=false\ntrust.der=['{}']\n\
         [environment]\ninherit=[]\n[environment.set]\nPATH='{path}'\n",
        files.path().join("unopened").display(),
        certificate.display()
    );
    let mut settings = crate::fixture::ready(settings::merge(
        &command::Shared::default(),
        files.path().to_path_buf(),
        &[
            settings::Source { layer: settings::Layer::User, path: files.path().join("base.toml"), text: base },
            settings::Source {
                layer: settings::Layer::User,
                path: files.path().join("extra.toml"),
                text: extra.into(),
            },
        ],
    ));
    settings.paths.path = Some(crate::fixture::commands_path().into());
    settings.paths.state_home = Some(files.path().join("state").into());
    settings
}

fn checked(settings: &Settings) -> Box<Report> {
    match report(settings, skein_lib::Wall::EPOCH, &skein_shell::Clock::new()) {
        Verdict::Ready(report) => report,
        Verdict::Refused(why) => panic!("{}", why.line()),
    }
}

#[test]
fn check_names_the_agents_rg_and_refuses_a_path_without_it() {
    let files = Files::new("smith-check-rg");
    for (name, target) in [("sh", "/bin/sh"), ("git", "/usr/bin/git")] {
        std::os::unix::fs::symlink(target, files.path().join(name)).expect("fixture tool");
    }
    let rg = files.path().join("rg");
    std::fs::write(&rg, b"unexecuted rg stand-in\n").expect("rg stand-in");
    std::fs::set_permissions(&rg, std::fs::Permissions::from_mode(0o700)).expect("executable mode");
    let extra = format!("[environment.set]\nPATH='{}'\n", files.path().display());
    let settings = check_settings(&files, "openai/gpt-6-luna", "none", &extra);
    let report = checked(&settings);
    assert_eq!(report.tools.get("rg"), Some(&rg), "report names commands' rg");
    let resolved = crate::fixture::ready(settings::resolve::resolve(&settings));
    assert_eq!(resolved.agent.service.rg.as_ref(), rg.as_os_str().as_encoded_bytes(), "agent uses the reported rg");
    std::fs::remove_file(&rg).expect("remove only rg");
    let Verdict::Refused(why) = super::report(&settings, skein_lib::Wall::EPOCH, &skein_shell::Clock::new()) else {
        panic!("check must refuse an rg absent from PATH, not use the run's fallback");
    };
    assert_eq!(why.key, "tools.rg");
    assert_eq!(why.problem, Problem::MissingTool);
}

#[test]
fn zero_output_names_its_key_and_positive_requirement() {
    let files = Files::new("smith-credentials");
    let settings = check_settings(&files, "openai/gpt-6-luna", "none", "[models.'openai/gpt-6-luna']\noutput=0\n");
    let Verdict::Refused(why) = report(&settings, skein_lib::Wall::EPOCH, &skein_shell::Clock::new()) else {
        panic!("zero model output must refuse check");
    };
    assert_eq!(why.key, "models.openai/gpt-6-luna.output", "output declaration");
    assert_eq!(why.problem, Problem::Input("output must be positive".into()), "positive output requirement");
    assert!(
        !files.path().join("state").exists() && !files.path().join("unopened").exists(),
        "refusal creates no resources"
    );
}

#[test]
fn missing_fixed_registration_keys_keep_their_source() {
    let files = Files::new("smith-credentials");
    let base = check_settings(&files, "openai/gpt-6-luna", "sign_in", "");
    let fields = [
        ("client_id", "client"),
        ("authorization_url", "http://127.0.0.1:444/authorize"),
        ("token_url", "http://127.0.0.1:444/token"),
        ("redirect_uri", "http://127.0.0.1:2345/callback"),
        ("scope", "read"),
        ("token_format", "form"),
    ];
    for (missing, _) in fields {
        let mut settings = base.clone();
        for (field, value) in fields {
            settings.values.insert(
                format!("accounts.openai.{field}"),
                settings::Sourced {
                    value: toml::Value::String(value.into()),
                    origin: Origin::File { layer: settings::Layer::User, path: "registration.toml".into(), line: 8 },
                },
            );
        }
        settings.values.remove(&format!("accounts.openai.{missing}"));
        let refusal = settings::resolve::resolve_startup(&settings).err().expect("required registration field");
        assert_eq!(refusal.key, format!("accounts.openai.{missing}"), "missing field");
        assert_eq!(
            refusal.origin, base.values["accounts.openai.source"].origin,
            "missing field uses the source declaration's origin"
        );
        assert!(refusal.line().contains("base.toml"), "{}", refusal.line());
    }
    assert!(!files.path().join("unopened").exists(), "registration validation opens no credential store");
}

#[test]
fn default_checks_declare_memory_without_opening_resources() {
    // The standard profile's shipped memory declaration.
    let memory_bytes = smith_agent_service::profile::standard().declared.memory;
    let files = Files::new("smith-credentials");
    for model in ["openai/gpt-6-astra", "anthropic/claude-opus-5-5"] {
        let settings = check_settings(&files, model, "none", "");
        let report = checked(&settings);
        let memory = report.named_limits.iter().find(|limit| limit["name"] == "declared.memory").expect("memory");
        assert_eq!(
            memory,
            &serde_json::json!({"name":"declared.memory","value":memory_bytes,"unit":"bytes"}),
            "{model}"
        );
        assert!(!files.path().join("state").exists(), "check starts no chat store");
        assert!(!files.path().join("unopened").exists(), "check starts no credential store");
    }
}

#[test]
fn effective_limits_keep_their_names_values_and_units() {
    let files = Files::new("smith-credentials");
    let report = checked(&check_settings(&files, "openai/gpt-6-luna", "none", ""));
    let json: serde_json::Value = serde_json::from_str(&render_json(&report)).expect("check JSON");
    let limits = json["limits"].as_array().expect("limits inventory");
    // Shipped standard tool payload, model head deadline and product teardown.
    for (name, value, unit) in [
        ("declared.tool_payload", 40_960, "bytes"),
        ("declared.head[endpoint=openai]", 60_000, "ms"),
        ("service.teardown", 40_000, "ms"),
    ] {
        let expected = serde_json::json!({"name":name,"value":value,"unit":unit});
        assert!(limits.contains(&expected), "missing {expected}");
        assert!(render_text(&report).contains(&format!("{name} = {value} {unit}")), "text names {name}");
    }
    assert!(
        limits.iter().all(|limit| !limit["name"].as_str().expect("name").contains("endpoint=0")),
        "symbolic endpoints"
    );
}

#[test]
fn an_unlisted_wrapper_cache_warns_without_creating_it() {
    let files = Files::new("smith-credentials");
    let cargo = files.path().join("cargo");
    for cache in ["registry", "git"] {
        std::fs::create_dir_all(cargo.join(cache)).expect("existing Rust cache");
    }
    let cache = files.path().join("wrapper-cache");
    let extra = format!(
        "[build]\npreset='rust'\n[environment.set]\nCARGO_HOME='{}'\nRUSTC_WRAPPER='sccache'\nSCCACHE_DIR='{}'\n",
        cargo.display(),
        cache.display()
    );
    let report = checked(&check_settings(&files, "openai/gpt-6-luna", "none", &extra));
    let warning = report
        .warnings
        .iter()
        .find(|warning| matches!(warning, Warning::WrapperCache { .. }))
        .expect("wrapper warning");
    let json = serde_json::to_value(warning).expect("warning JSON");
    assert_eq!(json["configured_by"], "RUSTC_WRAPPER", "wrapper configuration name");
    assert_eq!(json["cache"], cache.to_str().expect("cache path"), "unlisted cache");
    assert!(warnings::line(warning).contains("has no listed writable cache"), "actionable warning");
    assert!(!cache.exists(), "check never creates the wrapper cache");
}

#[test]
fn an_absent_cache_warns_without_being_created() {
    let files = Files::new("smith-credentials");
    let cache = files.path().join("absent-cache");
    let extra = format!("[build]\ncaches=['{}']\n", cache.display());
    let report = checked(&check_settings(&files, "openai/gpt-6-luna", "none", &extra));
    assert!(report.warnings.iter().any(|warning| matches!(warning, Warning::CacheAccess { cache: path, reason } if path == &cache && reason == "absent")), "absent cache report");
    assert!(!cache.exists(), "check creates nothing");
}

#[test]
fn a_file_cache_warns_and_keeps_its_bytes() {
    let files = Files::new("smith-credentials");
    let cache = files.path().join("file-cache");
    std::fs::write(&cache, b"kept").expect("file instead of a cache");
    let extra = format!("[build]\ncaches=['{}']\n", cache.display());
    let report = checked(&check_settings(&files, "openai/gpt-6-luna", "none", &extra));
    assert!(report.warnings.iter().any(|warning| matches!(warning, Warning::CacheAccess { cache: path, reason } if path == &cache && reason == "not a directory")), "file cache report");
    assert_eq!(std::fs::read(cache).expect("retained cache file"), b"kept", "check preserves file bytes");
}

const SPARE_ACCOUNT: &str = "[accounts.spare]\nprovider='anthropic'\nsource='program'\n";

fn assert_spare_warns(json: &serde_json::Value, directory: &Path) {
    // SHA-256 of spare starts cf2d9706; the first four bytes name the account.
    const SPARE: u32 = 0xcf2d_9706;
    assert_eq!(
        json["credentials"]["spare"],
        serde_json::json!({
            "required":false,"provider":"anthropic","source":"program","number":SPARE,
            "kept":{"kind":"directory","at":directory},
            "state":"program_not_signed_in","program":"claude_code"
        }),
        "unused account retains its metadata and checked state"
    );
    assert!(
        json["warnings"].as_array().expect("warnings").contains(&serde_json::json!({
            "kind":"credential","account":"spare","credential":{"state":"program_not_signed_in","program":"claude_code"}
        })),
        "unused account warning"
    );
}

#[test]
fn an_unused_program_account_warns_without_refusing_check() {
    let files = Files::new("smith-credentials");
    let report = checked(&check_settings(&files, "openai/gpt-6-luna", "none", SPARE_ACCOUNT));
    let json: serde_json::Value = serde_json::from_str(&render_json(&report)).expect("check JSON");
    assert_spare_warns(&json, &files.path().join("unopened/programs/spare"));
    assert!(report.credentials_ready(), "only needed credentials decide readiness");
    assert!(
        render_text(&report).contains("warning: account spare: Claude Code is not signed in"),
        "unused account warning text"
    );
}

fn check_given_program_answers(
    arguments: &[&str],
    directory: &Path,
    extra: &str,
    mut replies: accounts::ProgramAnswers,
    expected_accounts: &[&str],
) -> Output {
    let Command::Check { shared, json, execution } =
        command::parse(arguments.iter().map(OsString::from).collect()).expect("check arguments")
    else {
        panic!("check command");
    };
    let commands_path = crate::fixture::commands_path();
    let certificate = crate::fixture::trust();
    let text = format!(
        "model='anthropic/claude-haiku-5-5'\ncredentials='{}'\n[programs]\nclaude_code='/bin/true'\n\
         [accounts.work]\nprovider='anthropic'\nsource='program'\n[accounts.anthropic]\nsource='none'\n\
         [endpoints.anthropic]\naccount='work'\naddress='127.0.0.1:443'\nprogram='claude_code'\ntrust.system=false\ntrust.der=['{}']\n\
         [environment]\ninherit=[]\n[environment.set]\nPATH='{commands_path}'\n{extra}",
        directory.display(),
        certificate.display(),
    );
    let mut settings = crate::fixture::ready(settings::from_sources(
        &shared,
        "/workspace".into(),
        &[crate::fixture::source("check.toml".into(), text)],
    ));
    settings.paths.path = Some(commands_path.into());
    let wall = skein_lib::Wall::EPOCH;
    let prepared = crate::fixture::ready(prepare_report(&settings, wall, execution.as_ref()));
    let mut asked = Vec::new();
    let mut answers = accounts::ProgramAnswers::new();
    for account in accounts::program_accounts(&prepared.resolved) {
        asked.push(account.clone());
        let reply = replies.remove(&account).unwrap_or_else(|| panic!("unexpected inspection of {account}"));
        answers.insert(account, reply);
    }
    assert_eq!(asked, expected_accounts, "each listed program account is inspected once");
    assert!(replies.is_empty(), "all supplied answers were requested");
    render_verdict(prepared.finish(wall, answers), json)
}

#[test]
fn check_exits_3_for_a_program_account_not_signed_in() {
    let output = check_given_program_answers(
        &["check"],
        Path::new("/credentials"),
        "",
        accounts::ProgramAnswers::from([("work".into(), Ok(ProgramCheck::NotSignedIn))]),
        &["work"],
    );
    assert_eq!(output.code, 3);
    assert_eq!(output.stderr, "");
    assert!(
        output.stdout.lines().any(|line| line == "account work: Claude Code is not signed in: run `smith login work`")
    );
}

#[test]
fn check_says_a_program_signed_in_otherwise() {
    let method = ProgramMethod::new(b"api").expect("method");
    let output = check_given_program_answers(
        &["check", "--json", "exec"],
        Path::new("/credentials"),
        "",
        accounts::ProgramAnswers::from([("work".into(), Ok(ProgramCheck::SignedInOtherwise(method)))]),
        &["work"],
    );
    assert_eq!(output.code, 3);
    let json: serde_json::Value = serde_json::from_str(&output.stdout).expect("check JSON");
    assert_eq!(json["credentials"]["work"]["state"], "program_signed_in_otherwise");
    assert_eq!(json["credentials"]["work"]["method"], "api");
}

#[test]
fn a_failed_program_check_is_exit_3() {
    let unreadable = ProgramCheck::Failed(ProgramFailure { reason: FailureReason::Unreadable, detail: Box::new([]) });
    let output = check_given_program_answers(
        &["check", "--json"],
        Path::new("/credentials"),
        "",
        accounts::ProgramAnswers::from([("work".into(), Ok(unreadable))]),
        &["work"],
    );
    assert_eq!(output.code, 3);
    let json: serde_json::Value = serde_json::from_str(&output.stdout).expect("check JSON");
    assert_eq!(json["credentials"]["work"]["state"], "program_failed");
}

#[test]
fn check_inspects_an_unused_program_account_without_refusing_startup() {
    let directory = Path::new("/credentials");
    let output = check_given_program_answers(
        &["check", "--json"],
        directory,
        SPARE_ACCOUNT,
        accounts::ProgramAnswers::from([
            ("work".into(), Ok(ProgramCheck::Usable { email: None })),
            ("spare".into(), Ok(ProgramCheck::NotSignedIn)),
        ]),
        &["spare", "work"],
    );
    assert_eq!(output.code, 0, "only needed credentials decide the exit");
    let json: serde_json::Value = serde_json::from_str(&output.stdout).expect("check JSON");
    assert_eq!(json["credentials"]["work"]["required"], true);
    assert_eq!(json["credentials"]["work"]["state"], "usable");
    assert_spare_warns(&json, &directory.join("programs/spare"));
}

fn linked_directory(path: &Path, target: &Path) {
    std::fs::rename(path, target).expect("move source directory");
    std::os::unix::fs::symlink(target, path).expect("linked source directory");
}

fn public_directory(path: &Path, _target: &Path) {
    std::fs::set_permissions(path, std::fs::Permissions::from_mode(0o755)).expect("public source directory");
}

#[test]
fn unsafe_program_directories_map_to_failed_account_reports() {
    use std::os::unix::fs::DirBuilderExt;

    // Only credentials are inspected; serde validates but does not materialize the other fields.
    #[derive(serde::Deserialize)]
    struct CredentialDocument {
        credentials: serde_json::Value,
    }

    // Path components and faults are row data; every row follows the same inspection.
    type Fault = fn(&Path, &Path);
    let rows: &[(&str, Fault, &str)] = &[
        ("programs/openai", linked_directory, "symbolic link"),
        ("programs/openai", public_directory, "nonprivate mode bits 0o55"),
        ("programs", linked_directory, "symbolic link"),
        ("programs", public_directory, "nonprivate mode bits 0o55"),
    ];
    let files = Files::new("smith-credentials");
    let executable = files.path().join("unrun-program");
    std::fs::write(&executable, b"").expect("scratch executable");
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).expect("executable mode");
    let extra =
        format!("[programs]\nclaude_code='{}'\n[endpoints.openai]\nprogram='claude_code'\n", executable.display());
    let settings = check_settings(&files, "openai/gpt-6-luna", "program", &extra);
    let base = files.path().join("unopened");
    for &(relative, fault, reason) in rows {
        let path = base.join(relative);
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(base.join("programs/openai"))
            .expect("private source tree");
        // The link target stays inside the tree removed after each row.
        fault(&path, &base.join("linked-target"));
        let report = checked(&settings);
        let json: CredentialDocument = serde_json::from_str(&render_json(&report)).expect("check JSON");
        assert_eq!(json.credentials["openai"]["state"], "program_failed", "{relative}: {reason}");
        assert_eq!(
            json.credentials["openai"]["detail"],
            format!("{}: {reason}", path.display()),
            "exact unsafe-directory detail"
        );
        assert!(!report.credentials_ready(), "the needed unsafe account refuses readiness");
        std::fs::remove_dir_all(&base).expect("remove the row's source tree and fault");
    }
}

#[test]
fn the_report_lists_each_program_a_run_may_use() {
    let files = Files::new("smith-credentials");
    let executable = files.path().join("claude");
    std::fs::write(&executable, b"").expect("scratch executable");
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).expect("executable mode");
    let certificate = crate::fixture::trust().display().to_string();
    let text = format!(
        "model='openai/gpt-6-luna'\ncredentials='{}'\n[environment]\ninherit=['PATH']\n[programs]\nclaude_code='{}'\n[accounts.openai]\nprovider='openai'\nsource='key'\n[endpoints.openai]\naddress='127.0.0.1:443'\nprogram='claude_code'\ntrust.system=false\ntrust.der=['{certificate}']\n",
        files.path().display(),
        executable.display()
    );
    let mut settings = crate::fixture::ready(settings::merge(
        &command::Shared::default(),
        files.path().to_path_buf(),
        &[settings::Source { layer: settings::Layer::User, path: files.path().join("config.toml"), text }],
    ));
    settings.paths.path = Some(crate::fixture::commands_path().into());
    let report = match report(&settings, skein_lib::Wall::from_nanos(0), &skein_shell::Clock::new()) {
        Verdict::Ready(report) => report,
        Verdict::Refused(why) => panic!("{}", why.line()),
    };
    let json: serde_json::Value = serde_json::from_str(&render_json(&report)).expect("check document");
    assert_eq!(
        json["programs"],
        serde_json::json!({"claude_code":executable}),
        "the builder resolves each declared program path"
    );
    assert_eq!(json["version"], 2, "check emits version 2");
    assert_eq!(json["formats"]["check"], 2, "formats names the same version");
    let text = render_text(&report);
    let program_line = format!("program claude_code = {}", executable.display());
    assert_eq!(
        text.lines().skip_while(|line| !line.starts_with("sh = ")).nth(1),
        Some(program_line.as_str()),
        "programs immediately follow the tools"
    );
    assert!(text.contains(&format!("{program_line}\nenvironment:")), "programs precede the environment");
    assert!(!files.path().join("programs").exists(), "building the report never prepares the program source");
}
