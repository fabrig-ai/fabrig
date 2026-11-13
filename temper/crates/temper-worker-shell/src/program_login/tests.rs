use super::*;
use std::os::unix::fs::PermissionsExt;
use std::{ffi::OsString, os::unix::ffi::OsStringExt};

#[test]
fn only_terminal_input_takes_the_foreground_and_a_refusal_fails() {
    for (terminal, result, expected) in [
        (false, Err(Error::Permission), Ok(())),
        (true, Ok(()), Ok(())),
        (true, Err(Error::Permission), Err("cannot take the terminal: Permission".to_owned())),
    ] {
        let mut called = false;
        let outcome = take_terminal(terminal, |descriptor| {
            assert_eq!(descriptor, Fd::new(0));
            called = true;
            result
        });
        assert_eq!(outcome, expected);
        assert_eq!(called, terminal);
    }
}

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
    std::fs::write(directory.join("root.der"), skein_tls_kit::pki::ROOT).expect("worker-alone fixture");
    std::fs::write(directory.join("temper.json"), serde_json::to_vec(&input).expect("worker-alone fixture"))
        .expect("worker-alone fixture");
    let command = command(&crate::tests::configuration::directories(&directory), OsStr::new("source"), [], None)
        .expect("worker-alone fixture");
    assert_eq!(command.get_program(), OsStr::new("/missing/claude"));
    assert_eq!(command.get_args().collect::<Vec<_>>(), ["auth", "login"]);
    assert!(!directory.join("state").exists());
    assert!(!directory.join("worker.secret").exists(), "no link secret needed for program login");
    std::fs::remove_dir_all(directory).expect("worker-alone fixture");
}

#[test]
fn a_document_of_another_shape_needs_a_deployment_reader() {
    let scratch = skein_scratch::Scratch::new("temper-login-shape");
    let directory = scratch.path();
    let (agent, input) = crate::tests::configuration::documents(directory, "/missing/temper");
    let mut input: serde_json::Value = serde_json::from_slice(&input).expect("engine-alone fixture");
    input["engine"]["hosts"]["listener"] = serde_json::json!([{"socket":"127.0.0.1:7100"}]);
    input["engine"]["hosts"]["workers"] = serde_json::json!([{"name":"worker-a","digest":"worker.digest"}]);
    input["workers"] = serde_json::json!([]);
    input["application"]["workers"] = serde_json::json!([]);
    let programs = directory.join("programs");
    let mut agent: serde_json::Value = serde_json::from_slice(&agent).expect("agent fixture");
    agent["programs"] = serde_json::json!({"claude_code":{"path":"/missing/claude","directory":programs}});
    std::fs::write(directory.join("agent.json"), serde_json::to_vec(&agent).expect("agent fixture"))
        .expect("agent fixture");
    std::fs::write(directory.join("temper.json"), serde_json::to_vec(&input).expect("engine-alone fixture"))
        .expect("engine-alone fixture");
    let directories = crate::tests::configuration::directories(directory);
    assert_eq!(command(&directories, OsStr::new("work"), [], None).unwrap_err(), "temper.json holds no worker alone");
    assert!(!programs.exists(), "a refused shape makes no programs directory");
}
