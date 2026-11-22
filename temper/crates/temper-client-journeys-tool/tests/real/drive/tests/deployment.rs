use sha2::{Digest, Sha256};
use std::path::Path;
use temper_client_journeys_tool::{
    line::Document,
    testing::{deployment_plan, io},
};

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "one preparation, reuse and patch fixture also stages the kept provider inputs"
)]
fn preparation_and_patch_files_are_private_held_and_atomic() {
    use std::{fs, os::unix::fs::PermissionsExt};
    let root = skein_scratch::Scratch::new("drive-deployment-files");
    let mut plan = deployment_plan();
    plan.options.scratch = root.path().into();
    plan.options.deployment = Some(root.path().join("recipe"));
    plan.options.scripts.clear();
    let source = plan.options.deployment.as_ref().expect("source");
    fs::create_dir(source).expect("fixture source");
    for (name, bytes) in [
        (
            "engine.toml",
            "workers = \"{place.workers}\"\nowner = \"kept@example.invalid\"\npublic = \"https://old.example.com\"\n",
        ),
        ("worker.toml", "engine = \"{place.workers}\"\nname = \"worker\"\n"),
        ("forgejo.json", r#"{"admin_token":"{place.data}/forgejo/admin"}"#),
        ("model.json", r#"{"key":"{place.data}/model/key"}"#),
        ("providers.json", r#"{"providers":[]}"#),
    ] {
        fs::write(source.join(name), bytes).expect("fixture document");
    }
    let values = io::deployment::prepare_with(&plan, Some([42; 32])).expect("fresh");
    assert!(values.forgejo_admin.ends_with(b"/data/forgejo/admin"));
    assert_eq!(fs::read_link(root.path().join("data/bin/git")).expect("git link"), Path::new("/build/bin/temper-box"));
    for relative in [
        "config",
        "config/engine",
        "config/worker",
        "data",
        "data/bin",
        "data/engine",
        "data/engine/secrets",
        "data/worker",
        "data/worker/secrets",
        "data/forgejo",
        "data/model",
        "cache",
        "cache/engine",
        "cache/worker",
    ] {
        assert_eq!(fs::metadata(root.path().join(relative)).expect("directory").permissions().mode() & 0o777, 0o700);
    }
    for relative in [
        Document::Engine.path().to_str().expect("engine path"),
        Document::Worker.path().to_str().expect("worker path"),
        "config/forgejo.json",
        "config/model.json",
        "data/worker/secrets/worker.secret",
        "data/engine/secrets/worker.digest",
        "config/providers.json",
        "data/engine/providers.setting",
    ] {
        assert_eq!(fs::metadata(root.path().join(relative)).expect("file").permissions().mode() & 0o777, 0o600);
    }
    let secret_path = root.path().join("data/worker/secrets/worker.secret");
    let digest_path = root.path().join("data/engine/secrets/worker.digest");
    let pair = crate::drive::deployment::secret_pair(&[42; 32]);
    let secret_hash = Sha256::digest(fs::read(&secret_path).expect("synthetic secret"));
    let digest_hash = Sha256::digest(fs::read(&digest_path).expect("synthetic digest"));
    assert_eq!(secret_hash, Sha256::digest(&pair.0));
    assert_eq!(digest_hash, Sha256::digest(&pair.1));
    fs::write(root.path().join("data/engine/providers.setting"), b"store value").expect("store kept another value");
    fs::remove_dir_all(source).expect("held reads no recipe");
    plan.options.bin = "/new/bin".into();
    io::deployment::prepare_with(&plan, Some([13; 32])).expect("held");
    assert_eq!(
        fs::read(root.path().join("data/engine/providers.setting")).expect("kept inputs"),
        br#"{"providers":[]}"#
    );
    assert_eq!(fs::read_link(root.path().join("data/bin/git")).expect("remade git"), Path::new("/new/bin/temper-box"));
    assert_eq!(Sha256::digest(fs::read(&secret_path).expect("held secret")), secret_hash);
    assert_eq!(Sha256::digest(fs::read(&digest_path).expect("held digest")), digest_hash);
    let engine = root.path().join(Document::Engine.path());
    let before = fs::read(&engine).expect("document");
    let patch_source = root.path().join("files");
    fs::create_dir(&patch_source).expect("files");
    plan.options.files = Some(patch_source.clone());
    for (name, content) in
        [("not-object", "[]"), ("invalid", "{"), ("public.toml", "public = \"http://{place.clients}\"\n")]
    {
        fs::write(patch_source.join(name), content).expect("patch fixture");
    }
    for name in ["absent", "not-object", "invalid"] {
        assert!(io::deployment::patch(&plan, Document::Engine, name.as_bytes()).is_err());
        assert_eq!(fs::read(&engine).expect("unchanged"), before);
    }
    let (destination, bytes) = io::deployment::patch(&plan, Document::Engine, b"public.toml").expect("valid patch");
    assert_eq!(fs::read(&engine).expect("not written yet"), before);
    io::deployment::replace(&destination, &bytes).expect("atomic replacement");
    let patched = crate::drive::deployment::configuration(
        "engine",
        &fs::read(&engine).expect("patched"),
        temper_config_shell::Shape::EngineAlone,
    )
    .expect("patched file");
    assert_eq!(patched.owner.as_deref(), Some("kept@example.invalid"));
    assert_eq!(patched.public.as_deref(), Some("http://127.0.0.1:4101"));
    assert_eq!(fs::metadata(&engine).expect("private replacement").permissions().mode() & 0o777, 0o600);
    plan.options.files = None;
    assert!(
        io::deployment::patch(&plan, Document::Engine, b"public.toml")
            .expect_err("files missing")
            .contains("no files directory")
    );
    let link = root.path().join("session/link");
    io::deployment::directory(&root.path().join("session")).expect("session");
    io::deployment::write(&link, b"synthetic link\n").expect("link file");
    assert_eq!(fs::metadata(&link).expect("private link").permissions().mode() & 0o777, 0o600);
    io::deployment::remove_link(root.path()).expect("link removed");
    assert!(!link.exists());
    fs::remove_dir_all(root.path().join("data")).expect("remove data");
    assert!(io::deployment::prepare_with(&plan, Some([0; 32])).expect_err("config alone").contains("config/"));
    fs::rename(root.path().join("config"), root.path().join("data")).expect("data alone");
    assert!(io::deployment::prepare_with(&plan, Some([0; 32])).expect_err("data alone").contains("data/"));
}

#[test]
fn script_files_use_the_shared_joiner_and_name_a_bad_source() {
    use std::{fs, os::unix::fs::PermissionsExt};
    let root = skein_scratch::Scratch::new("drive-script-files");
    let mut plan = deployment_plan();
    plan.options.scratch = root.path().into();
    for relative in [
        "config",
        "config/engine",
        "config/worker",
        "data",
        "data/engine",
        "data/worker",
        "data/forgejo",
        "data/model",
        "session",
    ] {
        fs::create_dir(root.path().join(relative)).expect("held fixture");
    }
    fs::write(root.path().join("config/providers.json"), br#"{"providers":[]}"#).expect("provider inputs");
    fs::write(root.path().join("config/forgejo.json"), br#"{"admin_token":"/unused/admin"}"#).expect("forge document");
    fs::write(root.path().join("config/model.json"), br#"{"listen":{"socket":"127.0.0.1:0"},"key":"/unused/key","tls":null,"models":[{"name":"qa","window":4096,"output":1024}],"cued":[],"asked":[],"otherwise":["fallback"]}"#).expect("model document");
    let first = root.path().join("first.json");
    let second = root.path().join("second.json");
    fs::write(&first, br#"{"asked":[{"cue":"one","turns":["a"]}],"otherwise":["old"]}"#).expect("first scripts");
    fs::write(&second, br#"{"asked":[{"cue":"two","turns":["b"]}],"otherwise":["new"]}"#).expect("second scripts");
    plan.options.scripts = vec![first, second.clone()];
    io::deployment::prepare_with(&plan, None).expect("scripts prepared");
    let output = root.path().join("session/scripts.json");
    let value: serde_json::Value = serde_json::from_slice(&fs::read(&output).expect("joined file")).expect("JSON");
    assert_eq!(value["asked"], serde_json::json!([{"cue":"one","turns":["a"]},{"cue":"two","turns":["b"]}]));
    assert_eq!(value["otherwise"], serde_json::json!(["new"]));
    assert_eq!(fs::metadata(&output).expect("private scripts").permissions().mode() & 0o777, 0o600);
    fs::remove_file(&output).expect("next session");
    fs::write(&second, br#"{"unknown":[]}"#).expect("bad source");
    let refusal = io::deployment::prepare_with(&plan, None).expect_err("invalid scripts refused before processes");
    assert!(refusal.contains(&format!("{}: unknown: unknown member", second.display())), "{refusal}");
    assert!(!output.exists());
}
