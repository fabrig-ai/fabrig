use super::*;
use std::os::unix::fs::PermissionsExt;
use std::{ffi::OsString, os::unix::ffi::OsStringExt};

#[test]
fn source_names_accept_smiths_alphabet_within_the_length_bound() {
    let max = smith_host_domain::SourceName::BYTES;
    for (bytes, valid) in [(b"a".to_vec(), true), (vec![b'a'; max], true), (b"az09-".to_vec(), true)] {
        assert_eq!(source_name(&OsString::from_vec(bytes.clone())).is_ok(), valid, "{bytes:?}");
    }
}
#[test]
fn source_names_refuse_values_outside_smiths_alphabet_or_length() {
    let max = smith_host_domain::SourceName::BYTES;
    for (bytes, valid) in [
        (vec![], false),
        (vec![b'a'; max + 1], false),
        (b"A".to_vec(), false),
        (b"a_b".to_vec(), false),
        (b"../a".to_vec(), false),
        (vec![0xff], false),
        (vec![0], false),
    ] {
        assert_eq!(source_name(&OsString::from_vec(bytes.clone())).is_ok(), valid, "{bytes:?}");
    }
}

#[test]
fn smiths_terminal_login_command_preserves_its_configured_process_contract() {
    let directory = std::env::temp_dir().join(format!("temper-login-command-{}", std::process::id()));
    std::fs::create_dir(&directory).expect("login command scratch");
    std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700)).expect("private scratch");
    let executable = std::path::Path::new("/nonexistent/configured-claude");
    let programs = directory.join("programs");
    let source = programs.join("source");
    let (agent, deployment) = crate::tests::configuration::documents(&directory, "/fixture/temper");
    let mut agent: serde_json::Value = serde_json::from_slice(&agent).expect("agent JSON");
    agent["programs"] = serde_json::json!({"claude_code":{"path":executable,"directory":programs}});
    std::fs::write(directory.join("agent.json"), serde_json::to_vec(&agent).expect("agent JSON")).expect("agent");
    std::fs::write(directory.join("root.der"), skein_tls_world::pki::ROOT).expect("public trust");
    let configuration = directory.join("temper.json");
    std::fs::write(&configuration, deployment).expect("non-UTF8 configuration path");
    let benign = OsString::from_vec(b"value-\xff".to_vec());
    let command = command(
        &crate::tests::configuration::directories(&directory),
        OsStr::new("source"),
        [
            ("HOME".into(), "/home/person".into()),
            ("PATH".into(), "/bin".into()),
            ("TEMPER_BENIGN".into(), benign.clone()),
            ("CLAUDE_CONFIG_DIR".into(), "/wrong".into()),
            ("ANTHROPIC_API_KEY".into(), "private".into()),
            ("CLAUDE_CODE_OAUTH_TOKEN".into(), "private".into()),
            ("AWS_BEARER_TOKEN_BEDROCK".into(), "private".into()),
            ("CLAUDE_CODE_USE_VERTEX".into(), "private".into()),
        ],
    )
    .expect("checked terminal command");
    assert_eq!(command.get_program(), executable.as_os_str());
    assert_eq!(command.get_args().collect::<Vec<_>>(), ["auth", "login"]);
    let environment = command
        .get_envs()
        .map(|(name, value)| (name, value.expect("set")))
        .collect::<std::collections::BTreeMap<_, _>>();
    assert_eq!(environment.len(), 4);
    for (name, value) in [
        ("HOME", OsStr::new("/home/person")),
        ("PATH", OsStr::new("/bin")),
        ("TEMPER_BENIGN", benign.as_os_str()),
        ("CLAUDE_CONFIG_DIR", source.as_os_str()),
    ] {
        assert_eq!(environment[OsStr::new(name)], value);
    }
    std::fs::remove_dir_all(directory).expect("remove command scratch");
}

#[test]
fn a_worker_alone_login_reads_only_its_worker_part() {
    let directory = std::env::temp_dir().join(format!("temper-worker-login-{}", std::process::id()));
    std::fs::create_dir(&directory).expect("worker-alone fixture");
    std::fs::set_permissions(&directory, std::fs::Permissions::from_mode(0o700)).expect("worker-alone fixture");
    let (agent, input) = crate::tests::worker_document(&directory);
    let mut agent: serde_json::Value = serde_json::from_slice(&agent).expect("worker-alone fixture");
    agent["programs"] =
        serde_json::json!({"claude_code":{"path":"/missing/claude","directory":directory.join("programs")}});
    std::fs::write(directory.join("agent.json"), serde_json::to_vec(&agent).expect("worker-alone fixture"))
        .expect("worker-alone fixture");
    std::fs::write(directory.join("root.der"), skein_tls_world::pki::ROOT).expect("worker-alone fixture");
    std::fs::write(directory.join("temper.json"), serde_json::to_vec(&input).expect("worker-alone fixture"))
        .expect("worker-alone fixture");
    let command = command(&crate::tests::configuration::directories(&directory), OsStr::new("source"), [])
        .expect("worker-alone fixture");
    assert_eq!(command.get_program(), OsStr::new("/missing/claude"));
    assert_eq!(command.get_args().collect::<Vec<_>>(), ["auth", "login"]);
    assert!(!directory.join("state").exists());
    assert!(!directory.join("worker.secret").exists(), "no link secret needed for program login");
    std::fs::remove_dir_all(directory).expect("worker-alone fixture");
}
