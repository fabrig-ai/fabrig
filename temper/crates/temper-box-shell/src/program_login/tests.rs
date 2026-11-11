use super::*;
use std::ffi::{OsStr, OsString};
use std::os::unix::{
    ffi::{OsStrExt, OsStringExt},
    fs::PermissionsExt,
};

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
    std::fs::write(directory.join("root.der"), skein_tls_kit::pki::ROOT).expect("public trust");
    let configuration = directory.join("temper.json");
    std::fs::write(&configuration, deployment).expect("non-UTF8 configuration path");
    let benign = OsString::from_vec(b"value-\xff".to_vec());
    let command = temper_worker_shell::program_login::command(
        &crate::tests::configuration::directories(&directory),
        OsStr::new("source"),
        [
            ("HOME".into(), "/home/person".into()),
            ("PATH".into(), "/bin".into()),
            ("BENIGN_SETTING".into(), benign.clone()),
            ("CLAUDE_CONFIG_DIR".into(), "/wrong".into()),
            ("ANTHROPIC_API_KEY".into(), "private".into()),
            ("CLAUDE_CODE_OAUTH_TOKEN".into(), "private".into()),
            ("AWS_BEARER_TOKEN_BEDROCK".into(), "private".into()),
            ("CLAUDE_CODE_USE_VERTEX".into(), "private".into()),
        ],
        Some(super::deployment),
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
        ("BENIGN_SETTING", benign.as_os_str()),
        ("CLAUDE_CONFIG_DIR", source.as_os_str()),
    ] {
        assert_eq!(environment[OsStr::new(name)], value);
    }
    std::fs::remove_dir_all(directory).expect("remove command scratch");
}

#[test]
fn the_printed_line_preserves_shell_arguments_from_another_directory() {
    let directory = std::env::temp_dir().join(format!("temper-printed-login-{}", std::process::id()));
    std::fs::create_dir(&directory).expect("printed command scratch");
    let binary = directory.join("temper ' binary");
    std::fs::write(&binary, b"#!/bin/sh\nprintf '%s\\n' \"$@\"\n").expect("stub program");
    std::fs::set_permissions(&binary, std::fs::Permissions::from_mode(0o700)).expect("executable stub");
    let mut directories = crate::tests::configuration::directories(&directory);
    directories.config = directory.join("config ' one").as_os_str().as_bytes().into();
    directories.data = directory.join("data ' two").as_os_str().as_bytes().into();
    directories.cache = directory.join("cache ' three").as_os_str().as_bytes().into();
    let home = directory.as_os_str().as_bytes();
    let word = crate::printed::path_word(binary.as_os_str().as_bytes(), Some(home));
    let printed = line(&directories, &word, Some(home)).expect("portable command");
    assert!(printed.starts_with("~/'temper"));
    assert!(printed.contains(" --data ~/'data"));
    let output = std::process::Command::new("/bin/sh")
        .args(["-c", &format!("{printed} source")])
        .current_dir("/")
        .env_clear()
        .env("PATH", "/missing")
        .env("HOME", &directory)
        .output()
        .expect("paste command into shell");
    assert!(output.status.success(), "{:?}", output.stderr);
    let expected = format!(
        "program-login\n--config\n{}\n--data\n{}\n--cache\n{}\nsource\n",
        crate::path(&directories.config).display(),
        crate::path(&directories.data).display(),
        crate::path(&directories.cache).display()
    );
    assert_eq!(output.stdout, expected.as_bytes());
    std::fs::remove_dir_all(directory).expect("remove printed command scratch");
}

#[test]
fn the_printed_line_fits_its_wire_bound_including_shell_quoting() {
    let directories = crate::tests::configuration::directories(std::path::Path::new("/fixture"));
    let base = line(&directories, b"/b", None).expect("short command");
    let binary = format!("/b{}", "x".repeat(8192 - base.len()));
    assert_eq!(line(&directories, binary.as_bytes(), None).unwrap().len(), 8192);
    for extra in ["x", "'"] {
        assert!(
            line(&directories, crate::printed::path_word(format!("{binary}{extra}").as_bytes(), None).as_ref(), None)
                .is_none()
        );
    }
}

#[test]
fn the_printed_line_refuses_non_utf8_in_each_path() {
    let original = crate::tests::configuration::directories(std::path::Path::new("/fixture"));
    let binary = std::path::PathBuf::from(OsString::from_vec(b"/binary-\xff".to_vec()));
    assert!(line(&original, binary.as_os_str().as_bytes(), None).is_none());
    for field in ["config", "data", "cache"] {
        let mut directories = original.clone();
        let path = Box::from(b"/directory-\xff".as_slice());
        match field {
            "config" => directories.config = path,
            "data" => directories.data = path,
            "cache" => directories.cache = path,
            _ => unreachable!(),
        }
        assert!(line(&directories, b"/temper", None).is_none(), "{field}");
    }
}
