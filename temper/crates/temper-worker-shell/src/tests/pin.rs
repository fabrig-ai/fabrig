use crate::{agent, worker};
use std::{
    fs,
    os::unix::{ffi::OsStrExt, fs::PermissionsExt},
    path::Path,
};

fn launch(program: &Path, configuration: &Path, words: &[&[u8]]) -> jig_protocol_config::Agent {
    jig_protocol_config::Agent {
        program: program.as_os_str().as_bytes().into(),
        configuration: configuration.as_os_str().as_bytes().into(),
        arguments: words.iter().map(|word| Box::from(*word)).collect(),
    }
}

#[test]
fn pin_replaces_the_previous_agent_with_private_byte_copies() {
    let scratch = skein_scratch::Scratch::new("worker-pin");
    let root = scratch.path().join("worker");
    fs::create_dir_all(root.join("agent")).expect("pin fixture");
    fs::write(root.join("agent/old"), b"stale").expect("pin fixture");
    let program = scratch.path().join("temper-agent");
    let configuration = scratch.path().join("agent.json");
    fs::write(&program, b"a").expect("pin fixture");
    let pinned = worker::pin(&root, &launch(&program, &configuration, &[]), b"checked").expect("pin fixture");
    let mut names: Vec<_> = fs::read_dir(root.join("agent"))
        .expect("pin fixture")
        .map(|entry| entry.expect("pin fixture").file_name())
        .collect();
    names.sort();
    assert_eq!(names, ["agent.json", "temper-agent"]);
    for (path, bytes, mode) in [
        (crate::path(&pinned.program), b"a".as_slice(), 0o700),
        (crate::path(&pinned.configuration), b"checked".as_slice(), 0o600),
    ] {
        assert_eq!(fs::read(path).expect("pin fixture"), bytes);
        assert_eq!(fs::metadata(path).expect("pin fixture").permissions().mode() & 0o777, mode);
    }
    assert_eq!(fs::metadata(root.join("agent")).expect("pin fixture").permissions().mode() & 0o777, 0o700);
}

#[test]
fn pinned_launch_survives_edits_and_renames_of_its_sources() {
    let scratch = skein_scratch::Scratch::new("worker-redeploy");
    let root = scratch.path();
    let program = root.join("temper-agent");
    let configuration = root.join("agent.json");
    let configured = configuration.as_os_str().as_bytes();
    for words in [
        vec![b"internal".as_slice(), b"agent", configured],
        vec![configured],
        vec![b"/other.json", configured, configured],
    ] {
        fs::write(&program, b"program at start").expect("pin fixture");
        fs::write(&configuration, b"configuration at start").expect("pin fixture");
        let pinned = worker::pin(root, &launch(&program, &configuration, &words), b"configuration at start")
            .expect("pin fixture");
        assert_eq!(crate::path(&pinned.program), root.join("agent/temper-agent"));
        assert!(crate::path(&pinned.program).is_absolute());
        for (before, after) in words.iter().zip(&pinned.arguments) {
            assert_eq!(after.as_ref(), if *before == configured { pinned.configuration.as_ref() } else { before });
        }
        for (path, changed, replacement, original, copy) in [
            (
                &program,
                b"edited program".as_slice(),
                root.join("new-program"),
                b"program at start".as_slice(),
                &pinned.program,
            ),
            (
                &configuration,
                b"edited configuration".as_slice(),
                root.join("new-configuration"),
                b"configuration at start".as_slice(),
                &pinned.configuration,
            ),
        ] {
            fs::write(path, changed).expect("pin fixture");
            assert_eq!(fs::read(crate::path(copy)).expect("pin fixture"), original, "in-place overwrite");
            fs::write(&replacement, changed).expect("pin fixture");
            fs::rename(replacement, path).expect("pin fixture");
            assert_eq!(fs::read(crate::path(copy)).expect("pin fixture"), original, "rename replacement");
        }
    }
}

#[test]
fn the_checked_agent_and_source_environment_need_no_second_configuration_read() {
    let scratch = skein_scratch::Scratch::new("worker-read-once");
    let directory = scratch.path();
    let (bytes, document) = super::worker_document(directory);
    let mut bytes: serde_json::Value = serde_json::from_slice(&bytes).expect("pin fixture");
    bytes["environment"] = serde_json::json!([{"name":"PINNED_ENV","value":"at-start"}]);
    bytes["programs"] =
        serde_json::json!({"claude_code":{"path":"/missing/claude","directory":directory.join("missing/programs")}});
    bytes["endpoints"][0]["program"] = "claude_code".into();
    let bytes = serde_json::to_vec(&bytes).expect("pin fixture");
    fs::write(directory.join("agent.json"), &bytes).expect("pin fixture");
    fs::write(directory.join("root.der"), skein_tls_kit::pki::ROOT).expect("pin fixture");
    let mut part = temper_worker_service::startup::worker_part(
        &serde_json::to_vec(&document).expect("pin fixture"),
        &temper_service_shell::jig_directories(&super::configuration::directories(directory)),
    )
    .expect("pin fixture");
    let program = directory.join("temper-agent");
    fs::write(&program, b"a").expect("pin fixture");
    part.named.program = program.as_os_str().as_bytes().into();
    let prepared = agent::prepare_worker(&part.named).expect("pin fixture");
    assert_eq!(prepared.bytes.as_ref(), bytes);
    assert_eq!(prepared.agent.bounds.program_endpoints.as_ref(), [1]);
    assert_eq!(prepared.agent.bounds.turns, 800);
    assert_eq!(prepared.agent.tool_payload, 2048);
    assert_eq!(prepared.environment.as_ref(), [Box::from(b"PINNED_ENV=at-start".as_slice())]);
    fs::remove_file(directory.join("agent.json")).expect("pin fixture");
    fs::write(directory.join("worker.secret"), b"worker-secret").expect("pin fixture");
    let files = temper_service_shell::read_files(
        agent::without_configuration(part.files, &part.named.agent),
        skein_shell::effective_user(),
    )
    .expect("pin fixture");
    let files = agent::with_configuration(files, &part.named.agent, &prepared.bytes);
    assert_eq!(files.contents(&part.named.agent), Some(bytes.as_slice()));
    let mut owned = Vec::new();
    let runtime =
        worker::root(&part.named, &prepared.bytes, skein_shell::effective_user(), &mut owned).expect("pin fixture");
    assert_eq!(fs::read(crate::path(&runtime.agent.configuration)).expect("pin fixture"), bytes);
    let sources =
        worker::sources(&prepared.agent, prepared.environment, &program, directory, skein_shell::effective_user(), &[])
            .expect("pin fixture");
    assert!(sources.resources.is_some(), "program resources from the same document");
    temper_service_shell::close(owned);
}

#[test]
fn worker_serve_refuses_programs_it_cannot_pin_and_names_the_file() {
    let scratch = skein_scratch::Scratch::new("worker-pin-refusal");
    let directory = scratch.path();
    let (agent, mut document) = super::worker_document(directory);
    fs::write(directory.join("agent.json"), agent).expect("pin fixture");
    fs::write(directory.join("root.der"), skein_tls_kit::pki::ROOT).expect("pin fixture");
    fs::write(directory.join("worker.secret"), b"worker-secret").expect("pin fixture");
    for program in [directory.join("absent-agent"), directory.join("directory-not-program")] {
        if program.ends_with("directory-not-program") {
            fs::create_dir(&program).expect("pin fixture");
        }
        document["workers"][0]["agent"]["program"] = serde_json::json!(program);
        fs::write(directory.join("temper.json"), serde_json::to_vec(&document).expect("pin fixture"))
            .expect("pin fixture");
        let why = temper_service_shell::run(
            "temper-worker",
            &super::configuration::directories(directory),
            &[],
            crate::served::SERVED,
            |_| Ok(()),
            |bytes, owned, ()| {
                worker::alone(&super::configuration::directories(directory), bytes, owned, &[])
                    .map(|service| temper_service_shell::Prepared { service, store: Box::new([]), adoption: () })
            },
            |_, _, ()| Ok(()),
        )
        .expect_err("pin refused");
        assert!(why.starts_with("temper-worker serve: "), "{why}");
        assert!(why.contains(program.to_str().expect("pin fixture")), "{why}");
    }
}
