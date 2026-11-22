use super::*;
use crate::tests::machine::{Disk, ROOT};
use skein_fake_machine::{Item, Refusal};
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
    let directory = Path::new(ROOT);
    let (_, input) = crate::tests::worker_document(directory);
    let program = directory.join("claude");
    let disk = Disk::new(&[
        Item::file(
            temper_config_shell::FILE.as_bytes(),
            &temper_config_shell::render(&input, temper_config_shell::Shape::WorkerAlone),
        ),
        Item::file(b"claude", b"claude").mode(0o700),
    ]);
    let command = disk.login().expect("worker-alone fixture");
    assert_eq!(command.get_program(), program.as_os_str());
    assert_eq!(command.get_args().collect::<Vec<_>>(), ["auth", "login"]);
    assert_eq!(disk.metadata(&directory.join("state")), Err(Refusal::NotFound));
    assert_eq!(
        disk.metadata(&directory.join("secrets/worker.secret")),
        Err(Refusal::NotFound),
        "no link secret needed for program login"
    );
}

#[test]
fn an_engine_file_is_refused_at_its_key_before_program_setup() {
    let scratch = skein_scratch::Scratch::new("temper-login-shape");
    let directory = scratch.path();
    let (agent, _) = crate::tests::configuration::documents(directory, "/missing/temper");
    let input = temper_config_shell::File {
        workers: Some("127.0.0.1:7100".parse().expect("workers address")),
        ..temper_config_shell::File::default()
    };

    let programs = directory.join("programs");
    let mut agent: serde_json::Value = serde_json::from_slice(&agent).expect("agent fixture");
    agent["programs"] = serde_json::json!({"claude_code":{"path":"/missing/claude","directory":programs}});
    std::fs::write(directory.join("agent.json"), serde_json::to_vec(&agent).expect("agent fixture"))
        .expect("agent fixture");
    std::fs::write(
        directory.join(temper_config_shell::FILE),
        temper_config_shell::render(&input, temper_config_shell::Shape::EngineAlone),
    )
    .expect("engine-alone fixture");
    let directories = crate::tests::configuration::directories(directory);
    assert_eq!(
        command(&directories, OsStr::new("work"), []).unwrap_err(),
        format!(
            "{file}:4: workers: a key of temper-engine's file, not temper-worker's\n{file}: engine: required in temper-worker's file",
            file = temper_config_shell::FILE
        )
    );
    assert!(!programs.exists(), "a refused shape makes no programs directory");
}
