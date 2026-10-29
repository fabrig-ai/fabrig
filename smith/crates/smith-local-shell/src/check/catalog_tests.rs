use super::*;
use crate::credentials::tests::Files;
use std::os::unix::fs::PermissionsExt;

fn check_settings(files: &Files, model: &str, source: &str, extra: &str) -> Settings {
    let (endpoint, _) = model.split_once('/').expect("fixture model");
    let certificate = skein_scratch::paths::tree().join("tests/protocol-llm/fixtures/root.der");
    let base = format!(
        "model='{model}'\ncredentials='{}'\n[accounts.{endpoint}]\nsource='{source}'\n\
         [endpoints.{endpoint}]\naddress='127.0.0.1:443'\ntrust.system=false\ntrust.der=['{}']\n\
         [environment]\ninherit=[]\n[environment.set]\nPATH='/usr/bin:/bin'\n",
        files.0.join("unopened").display(),
        certificate.display()
    );
    let mut settings = settings::merge(
        &command::Shared::default(),
        files.0.clone(),
        &[
            settings::Source { layer: settings::Layer::User, path: files.0.join("base.toml"), text: base },
            settings::Source { layer: settings::Layer::User, path: files.0.join("extra.toml"), text: extra.into() },
        ],
    )
    .unwrap_or_else(|why| panic!("{}", why.line()));
    settings.paths.path = Some("/usr/bin:/bin".into());
    settings.paths.state_home = Some(files.0.join("state").into());
    settings
}

fn checked(settings: &Settings) -> Box<Report> {
    match report(settings, skein_lib::Wall::EPOCH, &skein_shell::Clock::new()) {
        Verdict::Ready(report) => report,
        Verdict::Refused(why) => panic!("{}", why.line()),
    }
}

/// Lower test of smith's end_to_end::local_bad_setting_exits_2_and_names_output_before_a_run (top tiers test wiring only).
#[test]
fn zero_output_names_its_key_and_positive_requirement() {
    let files = Files::new();
    let settings = check_settings(&files, "codex/gpt-6-luna", "none", "[models.'codex/gpt-6-luna']\noutput=0\n");
    let Verdict::Refused(why) = report(&settings, skein_lib::Wall::EPOCH, &skein_shell::Clock::new()) else {
        panic!("zero model output must refuse check");
    };
    assert_eq!(why.key, "models.codex/gpt-6-luna.output", "output declaration");
    assert_eq!(why.problem, Problem::Input("output must be positive".into()), "positive output requirement");
    assert!(!files.0.join("state").exists() && !files.0.join("unopened").exists(), "refusal creates no resources");
}

/// Lower test of smith's end_to_end::smith_check_and_startup_refuse_missing_sign_in_registration_keys_by_name (top tiers test wiring only).
#[test]
fn missing_fixed_registration_keys_keep_their_source() {
    let files = Files::new();
    let base = check_settings(&files, "codex/gpt-6-luna", "sign_in", "");
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
                format!("accounts.codex.{field}"),
                settings::Sourced {
                    value: toml::Value::String(value.into()),
                    origin: Origin::File { layer: settings::Layer::User, path: "registration.toml".into(), line: 8 },
                },
            );
        }
        settings.values.remove(&format!("accounts.codex.{missing}"));
        let refusal = settings::resolve::resolve_startup(&settings).err().expect("required registration field");
        assert_eq!(refusal.key, format!("accounts.codex.{missing}"), "missing field");
        assert_eq!(
            refusal.origin, base.values["accounts.codex.source"].origin,
            "missing field uses the source declaration's origin"
        );
        assert!(refusal.line().contains("base.toml"), "{}", refusal.line());
    }
    assert!(!files.0.join("unopened").exists(), "registration validation opens no credential store");
}

/// Lower test of smith's end_to_end::the_interactive_default_check_emits_its_declaration_without_starting_resources (top tiers test wiring only).
#[test]
fn default_checks_declare_memory_without_opening_resources() {
    // The standard profile's shipped memory declaration.
    const MEMORY: u64 = 2_281_701_376;
    let files = Files::new();
    for model in ["codex/gpt-6-astra", "anthropic/claude-opus-5-5"] {
        let settings = check_settings(&files, model, "none", "");
        let report = checked(&settings);
        let memory = report.named_limits.iter().find(|limit| limit["name"] == "declared.memory").expect("memory");
        assert_eq!(memory, &serde_json::json!({"name":"declared.memory","value":MEMORY,"unit":"bytes"}), "{model}");
        assert!(!files.0.join("state").exists(), "check starts no chat store");
        assert!(!files.0.join("unopened").exists(), "check starts no credential store");
    }
}

/// Lower test of smith's end_to_end::smith_check_json_lists_the_effective_limits_by_name (top tiers test wiring only).
#[test]
fn effective_limits_keep_their_names_values_and_units() {
    let files = Files::new();
    let report = checked(&check_settings(&files, "codex/gpt-6-luna", "none", ""));
    let json: serde_json::Value = serde_json::from_str(&render_json(&report)).expect("check JSON");
    let limits = json["limits"].as_array().expect("limits inventory");
    // Shipped standard tool payload, model head deadline and product teardown.
    for (name, value, unit) in [
        ("declared.tool_payload", 40_960, "bytes"),
        ("declared.head[endpoint=codex,model=gpt-6-luna]", 60_000, "ms"),
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

/// Lower test of smith's end_to_end::smith_check_json_warns_of_a_wrapper_cache_outside_the_caches (top tiers test wiring only).
#[test]
fn an_unlisted_wrapper_cache_warns_without_creating_it() {
    let files = Files::new();
    let cargo = files.0.join("cargo");
    for cache in ["registry", "git"] {
        std::fs::create_dir_all(cargo.join(cache)).expect("existing Rust cache");
    }
    let cache = files.0.join("wrapper-cache");
    let extra = format!(
        "[build]\npreset='rust'\n[environment.set]\nCARGO_HOME='{}'\nRUSTC_WRAPPER='sccache'\nSCCACHE_DIR='{}'\n",
        cargo.display(),
        cache.display()
    );
    let report = checked(&check_settings(&files, "codex/gpt-6-luna", "none", &extra));
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

/// Lower test of smith's end_to_end::smith_check_warns_about_an_absent_cache_and_creates_nothing (top tiers test wiring only).
#[test]
fn an_absent_cache_warns_without_being_created() {
    let files = Files::new();
    let cache = files.0.join("absent-cache");
    let extra = format!("[build]\ncaches=['{}']\n", cache.display());
    let report = checked(&check_settings(&files, "codex/gpt-6-luna", "none", &extra));
    assert!(report.warnings.iter().any(|warning| matches!(warning, Warning::CacheAccess { cache: path, reason } if path == &cache && reason == "absent")), "absent cache report");
    assert!(!cache.exists(), "check creates nothing");
}

/// Lower test of smith's end_to_end::smith_check_warns_about_a_file_named_as_a_cache_and_keeps_it (top tiers test wiring only).
#[test]
fn a_file_cache_warns_and_keeps_its_bytes() {
    let files = Files::new();
    let cache = files.0.join("file-cache");
    std::fs::write(&cache, b"kept").expect("file instead of a cache");
    let extra = format!("[build]\ncaches=['{}']\n", cache.display());
    let report = checked(&check_settings(&files, "codex/gpt-6-luna", "none", &extra));
    assert!(report.warnings.iter().any(|warning| matches!(warning, Warning::CacheAccess { cache: path, reason } if path == &cache && reason == "not a directory")), "file cache report");
    assert_eq!(std::fs::read(cache).expect("retained cache file"), b"kept", "check preserves file bytes");
}

/// Lower test of smith's end_to_end::check_inspects_an_unused_program_account_without_refusing_startup (top tiers test wiring only).
#[test]
fn an_unused_program_account_warns_without_refusing_check() {
    let files = Files::new();
    let report = checked(&check_settings(
        &files,
        "codex/gpt-6-luna",
        "none",
        "[accounts.spare]\nprovider='anthropic'\nsource='program'\n",
    ));
    let json: serde_json::Value = serde_json::from_str(&render_json(&report)).expect("check JSON");
    // SHA-256 of spare starts cf2d9706; the first four bytes name the account.
    const SPARE: u32 = 0xcf2d_9706;
    assert_eq!(
        json["credentials"]["spare"],
        serde_json::json!({
            "required":false,"provider":"anthropic","source":"program","number":SPARE,
            "kept":{"kind":"directory","at":files.0.join("unopened/programs/spare")},
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
    assert!(report.credentials_ready(), "only needed credentials decide readiness");
    assert!(
        render_text(&report).contains("warning: account spare: Claude Code is not signed in"),
        "unused account warning text"
    );
}

/// Lower tests of smith's end_to_end::check_refuses_a_linked_program_account_directory and end_to_end::check_refuses_a_public_program_account_directory (top tiers test wiring only).
/// Lower tests of smith's end_to_end::check_refuses_a_linked_programs_directory and end_to_end::check_refuses_a_public_programs_directory (top tiers test wiring only).
#[test]
fn unsafe_program_directories_map_to_failed_account_reports() {
    use std::os::unix::fs::{DirBuilderExt, symlink};
    // Path components and faults are row data; every row follows the same inspection.
    for (relative, linked, reason) in [
        ("programs/codex", true, "symbolic link"),
        ("programs/codex", false, "nonprivate mode bits 0o55"),
        ("programs", true, "symbolic link"),
        ("programs", false, "nonprivate mode bits 0o55"),
    ] {
        let files = Files::new();
        let executable = files.0.join("unrun-program");
        std::fs::write(&executable, b"").expect("scratch executable");
        std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).expect("executable mode");
        let extra =
            format!("[programs]\nclaude_code='{}'\n[endpoints.codex]\nprogram='claude_code'\n", executable.display());
        let settings = check_settings(&files, "codex/gpt-6-luna", "program", &extra);
        let base = files.0.join("unopened");
        let path = base.join(relative);
        std::fs::DirBuilder::new()
            .recursive(true)
            .mode(0o700)
            .create(base.join("programs/codex"))
            .expect("private source tree");
        if linked {
            let target = files.0.join("linked-target");
            std::fs::rename(&path, &target).expect("move source directory");
            symlink(target, &path).expect("linked source directory");
        } else {
            std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o755)).expect("public source directory");
        }
        let report = checked(&settings);
        let json: serde_json::Value = serde_json::from_str(&render_json(&report)).expect("check JSON");
        assert_eq!(json["credentials"]["codex"]["state"], "program_failed", "{relative}: {reason}");
        assert_eq!(
            json["credentials"]["codex"]["detail"],
            format!("{}: {reason}", path.display()),
            "exact unsafe-directory detail"
        );
        assert!(!report.credentials_ready(), "the needed unsafe account refuses readiness");
    }
}

#[test]
fn the_report_lists_each_program_a_run_may_use() {
    let files = Files::new();
    let executable = files.0.join("claude");
    std::fs::write(&executable, b"").expect("scratch executable");
    std::fs::set_permissions(&executable, std::fs::Permissions::from_mode(0o700)).expect("executable mode");
    let certificate = skein_scratch::paths::tree().join("tests/protocol-llm/fixtures/root.der").display().to_string();
    let text = format!(
        "model='codex/gpt-6-luna'\ncredentials='{}'\n[environment]\ninherit=['PATH']\n[programs]\nclaude_code='{}'\n[accounts.codex]\nsource='key'\n[endpoints.codex]\naddress='127.0.0.1:443'\nprogram='claude_code'\ntrust.system=false\ntrust.der=['{certificate}']\n",
        files.0.display(),
        executable.display()
    );
    let mut settings = settings::merge(
        &command::Shared::default(),
        files.0.clone(),
        &[settings::Source { layer: settings::Layer::User, path: files.0.join("config.toml"), text }],
    )
    .unwrap_or_else(|why| panic!("{}", why.line()));
    settings.paths.path = Some("/usr/bin:/bin".into());
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
    assert!(!files.0.join("programs").exists(), "building the report never prepares the program source");
}
