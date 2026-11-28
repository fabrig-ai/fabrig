use super::*;
use serde_json::{Value, json};
use std::ffi::OsStr;

fn place() -> Place {
    Place::new([4101, 4102, 4103, 4104, 4105, 4106, 4107, 4108, 4109], "/scratch".into(), "/build/bin".into(), None)
        .expect("absolute place")
}
fn json(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).expect("JSON output")
}

#[test]
fn place_names_render_only_string_values() {
    let place = place();
    let input = br#"{
        "{place.unknown}": ["{place.clients}", {"nested": "{place.workers}"}],
        "ports": "{place.webhooks} {place.model} {place.forgejo}",
        "paths": "{place.config} {place.data} {place.cache} {place.bin} {place.git}",
        "cue": "changes/{pull}.md {other} {place.clients}/{place.clients}",
        "plain": [null, true, false, 1.234567890123456789e+42]
    }"#;
    let rendered = render("model.json", input, &place).expect("place names");
    assert_eq!(
        json(&rendered),
        json!({
            "{place.unknown}": ["127.0.0.1:4101", {"nested": "127.0.0.1:4102"}],
            "ports": "127.0.0.1:4105 127.0.0.1:4108 127.0.0.1:4109",
            "paths": "/scratch/config /scratch/data /scratch/cache /build/bin /scratch/data/bin/git",
            "cue": "changes/{pull}.md {other} 127.0.0.1:4101/127.0.0.1:4101",
            "plain": [null, true, false, 1.234_567_890_123_456_7e42]
        })
    );
    assert!(rendered.windows(b"1.234567890123456789e+42".len()).any(|bytes| bytes == b"1.234567890123456789e+42"));
    let why = render("model.json", br#"{"value":"{place.missing}"}"#, &place).expect_err("unknown name");
    assert!(why.contains("model.json") && why.contains("missing"), "{why}");
    assert!(Place::new([1; 9], "/scratch".into(), "/bin".into(), None).is_err());
    assert!(Place::new([0, 1, 2, 3, 4, 5, 6, 7, 8], "/scratch".into(), "/bin".into(), None).is_err());
    assert!(Place::new(place.ports.array(), "relative".into(), "/bin".into(), None).is_err());
    assert!(Place::new(place.ports.array(), "/scratch".into(), "relative".into(), None).is_err());
}

#[test]
fn bundle_name_renders_the_absolute_bundle_and_requires_it_by_document_name() {
    let mut place = place();
    let input = b"workers = \"{place.workers}\"\n[internal]\nbundle = \"{place.bundle}\"\n";
    let why = render("engine.toml", input, &place).expect_err("bundle absent");
    assert!(why.contains("engine.toml") && why.contains("place name bundle needs --bundle"), "{why}");
    place.bundle = Some("/p/bundle".into());
    let file =
        configuration("engine", &render("engine.toml", input, &place).expect("bundle given"), Shape::EngineAlone)
            .expect("file");
    assert_eq!(file.internal.bundle, Some("/p/bundle".into()));
    assert!(Place::new(place.ports.array(), "/scratch".into(), "/bin".into(), Some("relative".into())).is_err());
}

#[test]
fn documents_are_exact_bounded_files_at_prepared_paths() {
    let place = place();
    let files = [
        ("engine.toml", b"workers = \"{place.workers}\"\n".as_slice()),
        ("worker.toml", b"engine = \"{place.workers}\"\nname = \"worker\"\n".as_slice()),
        ("forgejo.json", b"{}".as_slice()),
        ("model.json", b"{}".as_slice()),
        ("providers.json", b"{}".as_slice()),
    ];
    let rendered = documents(&files, &place).expect("deployment documents and kept providers");
    assert_eq!(rendered[0].0, PathBuf::from("/scratch").join(Document::Engine.path()));
    assert_eq!(rendered[1].0, PathBuf::from("/scratch").join(Document::Worker.path()));
    assert_eq!(
        rendered.iter().skip(2).map(|(path, _)| path.to_str().expect("fixture path")).collect::<Vec<_>>(),
        [
            "/scratch/config/forgejo.json",
            "/scratch/config/model.json",
            "/scratch/data/engine/providers.setting",
            "/scratch/config/providers.json"
        ]
    );
    assert_eq!(
        configuration("engine", &rendered[0].1, Shape::EngineAlone).expect("engine").workers,
        Some("127.0.0.1:4102".parse().expect("socket"))
    );
    assert_eq!(
        configuration("worker", &rendered[1].1, Shape::WorkerAlone).expect("worker").name.as_deref(),
        Some("worker")
    );
    assert!(rendered.iter().skip(2).all(|(_, bytes)| json(bytes) == json!({})));
    for (name, _) in files {
        let missing: Vec<_> = files.iter().copied().filter(|(given, _)| *given != name).collect();
        let why = documents(&missing, &place).expect_err("required document");
        assert!(why.contains(name) && why.contains("missing"), "{why}");
    }
    let mut extra = files.to_vec();
    extra.push(("agent.json", b"{}"));
    assert!(documents(&extra, &place).expect_err("extra file").contains("agent.json"));
    let limit = usize::try_from(DOCUMENT_BYTES).expect("byte bound");
    let mut full = b"workers = \"127.0.0.1:4102\"\n".to_vec();
    full.resize(limit, b' ');
    render("engine.toml", &full, &place).expect("at byte bound");
    full.push(b' ');
    for bytes in [full.as_slice(), b"not TOML", b""] {
        let why = render("engine.toml", bytes, &place).expect_err("invalid document");
        assert!(why.contains("engine.toml"), "{why}");
    }
}

#[test]
fn preparation_requires_both_directories_or_neither() {
    assert_eq!(preparation(false, false), Ok(Preparation::Fresh));
    assert_eq!(preparation(true, true), Ok(Preparation::Held));
    assert_eq!(preparation(true, false), Err("config/ is present without data/".into()));
    assert_eq!(preparation(false, true), Err("data/ is present without config/".into()));
}

#[test]
fn secret_pair_matches_temper_init_vectors() {
    // temper-init/src/tests/files.rs: hash the digits alone, before their newline.
    let (secret, digest) = secret_pair(&std::array::from_fn(|index| u8::try_from(index).expect("32 bytes")));
    assert_eq!(secret.as_ref(), b"000102030405060708090a0b0c0d0e0f101112131415161718191a1b1c1d1e1f\n");
    assert_eq!(digest.as_ref(), b"6c86c6aac5fb24bcf5d9939cb7d7d5645ce39418f449e03b262dd4fa14b4b92b\n");
}

#[test]
fn values_take_output_paths_from_rendered_documents() {
    let place = place();
    let forgejo = render("forgejo.json", br#"{"admin_token":"{place.data}/forgejo/admin"}"#, &place).expect("forge");
    let model =
        render("model.json", br#"{"key":"{place.data}/model/key","model":{"key":"wrong"}}"#, &place).expect("model");
    let values = values(&place, &forgejo, &model).expect("deployment values");
    for (name, expected) in [
        ("{forgejo.api}", "127.0.0.1:4109"),
        ("{forgejo.git}", "http://127.0.0.1:4109"),
        ("{forgejo.admin}", "/scratch/data/forgejo/admin"),
        ("{git}", "/scratch/data/bin/git"),
        ("{model.key}", "/scratch/data/model/key"),
    ] {
        assert_eq!(values.value(BoxValue::parse(name.as_bytes()).expect("value name")), expected.as_bytes());
    }
    for (forgejo, model, document, member) in [
        (b"{}".as_slice(), model.as_ref(), "forgejo.json", "admin_token"),
        (forgejo.as_ref(), b"{}".as_slice(), "model.json", "key"),
        (forgejo.as_ref(), br#"{"model":{"key":"nested"}}"#, "model.json", "key"),
        (forgejo.as_ref(), br#"{"key":null}"#, "model.json", "key"),
    ] {
        let why = super::values(&place, forgejo, model).expect_err("missing top-level string");
        assert!(why.contains(document) && why.contains(member), "{why}");
    }
}

#[test]
fn merge_overlays_toml_and_renders_only_the_patch() {
    let place = place();
    let source = b"engine = \"127.0.0.1:4102\"\nname = \"{place.clients}\"\n[environment]\nOLD = \"old\"\n";
    let patch = b"[environment]\nNEW = \"{place.clients}\"\n";
    let merged = merge(source, patch, &place, Shape::WorkerAlone).expect("merge");
    let file = configuration("worker", &merged, Shape::WorkerAlone).expect("worker file");
    assert_eq!(file.name.as_deref(), Some("{place.clients}"));
    assert_eq!(file.environment, [("NEW".into(), "127.0.0.1:4101".into())]);
    for patch in [b"[]".as_slice(), b"name = null", b"listen = '127.0.0.1:7100'", b"bad"] {
        assert!(merge(source, patch, &place, Shape::WorkerAlone).expect_err("valid patch required").contains("patch"));
    }
    let why = merge(source, b"name = '{place.nope}'", &place, Shape::WorkerAlone).expect_err("patch name");
    assert!(why.contains("patch") && why.contains("nope"), "{why}");
    let limit = usize::try_from(PATCH_BYTES).expect("patch bound");
    let mut full = vec![b' '; limit];
    merge(source, &full, &place, Shape::WorkerAlone).expect("empty patch at bound");
    full.push(b' ');
    assert!(merge(source, &full, &place, Shape::WorkerAlone).expect_err("patch past bound").contains("patch"));
}

#[test]
fn toml_engine_patches_are_checked_as_the_binary_checks_them() {
    let place = place();
    let source = b"workers = '127.0.0.1:4102'\n";
    let why =
        merge(source, b"public = 'not an origin'", &place, Shape::EngineAlone).expect_err("invalid public origin");
    assert!(why.contains(temper_config_shell::FILE) && why.contains("public"), "{why}");
    let why = render("engine.toml", b"workers = '127.0.0.1:4102'\npublic = 'not an origin'", &place)
        .expect_err("invalid template origin");
    assert!(why.contains("engine.toml") && why.contains("public"), "{why}");
}

#[test]
fn plans_start_in_order_and_sign_in_with_closed_stdin() {
    let mut plan = super::super::tests::plan();
    plan.options.bundle = None;
    plan.options.deployment = Some("/recipe/config".into());
    plan.options.ports = Some("4101,4102,4103,4104,4105,4106,4107,4108,4109".into());
    assert_eq!(plan.place().expect("options place").value("clients").as_deref(), Some(b"127.0.0.1:4101".as_slice()));
    let programs = plan.deployment_programs();
    let expected = [
        (
            "temper-box",
            vec![
                "fake",
                "forge",
                "--document",
                "/scratch/config/forgejo.json",
                "--ready",
                "listening ",
                "--state",
                "/scratch/data/forgejo/state",
            ],
        ),
        (
            "temper-box",
            vec![
                "fake",
                "llm",
                "--document",
                "/scratch/config/model.json",
                "--ready",
                "listening ",
                "--scripts",
                "/scratch/session/scripts.json",
                "--strict",
            ],
        ),
        (
            "temper-engine",
            vec![
                "serve",
                "--config",
                "/scratch/config/engine",
                "--data",
                "/scratch/data/engine",
                "--cache",
                "/scratch/cache/engine",
            ],
        ),
        (
            "temper-worker",
            vec![
                "serve",
                "--config",
                "/scratch/config/worker",
                "--data",
                "/scratch/data/worker",
                "--cache",
                "/scratch/cache/worker",
            ],
        ),
    ];
    for (program, (name, args)) in programs.iter().zip(expected) {
        let (binary, args) = deployment_command(name, &args);
        assert_eq!(program.program, binary);
        assert_eq!(program.args, args);
        assert_eq!(program.input, Input::Null);
        assert_eq!(program.directory, PathBuf::from("/scratch"));
        assert_eq!(program.environment, plan.environment);
        for (_, value) in program.environment.iter().filter(|(key, _)| key != OsStr::new("PATH")) {
            assert!(PathBuf::from(value).starts_with("/scratch"));
        }
        let text = format!("{program:?}");
        assert!(!text.contains("cargo") && !text.contains("scripts/bundle.sh"));
    }
    let sign_in = plan.deployment_sign_in();
    assert_eq!(sign_in.program, PathBuf::from("/build/bin/temper"));
    assert_eq!(sign_in.args, ["ctl", "/scratch/session/ctl", "sign-in", "/scratch/session/link"]);
    assert_eq!(sign_in.input, Input::Null);
    plan.options.scripts.clear();
    plan.options.strict = false;
    let (_, args) = deployment_command(
        "temper-box",
        &["fake", "llm", "--document", "/scratch/config/model.json", "--ready", "listening "],
    );
    assert_eq!(plan.deployment_programs()[1].args, args);
}

fn deployment_command(name: &str, args: &[&str]) -> (PathBuf, Vec<OsString>) {
    let child = PathBuf::from("/build/bin").join(name);
    let mut words = vec![];
    let binary = if cfg!(target_os = "linux") {
        words.extend(["internal", "forge-child", "--parent", "42", "--signal", "kill", "--"].map(OsString::from));
        words.push(child.into_os_string());
        PathBuf::from("/build/bin/temper-box")
    } else {
        child
    };
    words.extend(args.iter().map(OsString::from));
    (binary, words)
}
