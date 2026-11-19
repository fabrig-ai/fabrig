//! The worker's file reaches its part and program-login without an engine (configuration/file.md, 6).
use std::{ffi::OsStr, fs, os::unix::ffi::OsStrExt};
use temper_config_shell::{FILE, FORMER, Shape};

#[test]
fn toml_worker_prepares_its_part_from_the_file() {
    let scratch = skein_scratch::Scratch::new("worker-config-file");
    let directories = crate::tests::configuration::directories(scratch.path());
    fs::write(scratch.path().join(FILE), "engine = \"127.0.0.1:7200\"\nname = \"w1\"\n").expect("worker file");
    let read = temper_service_shell::configuration::read(&directories, Shape::WorkerAlone).expect("worker file");
    let mut part = temper_worker_service::startup::worker_part(
        &read.document,
        &temper_service_shell::jig_directories(&directories),
        None,
        &[],
    )
    .expect("worker part");
    let executable = scratch.path().join("temper-worker");
    let agent = scratch.path().join("temper-agent");
    fs::copy("/bin/sh", &agent).expect("inert agent executable");
    crate::worker::inputs(&mut part, &directories, &executable).expect("derived worker program");
    assert_eq!(part.named.program.as_ref(), agent.as_os_str().as_bytes());
}

#[test]
fn toml_worker_program_login_reads_the_file() {
    let scratch = skein_scratch::Scratch::new("worker-login-file");
    let directories = crate::tests::configuration::directories(scratch.path());
    fs::write(scratch.path().join(FILE), "engine = \"127.0.0.1:7200\"\nname = \"w1\"\n").expect("worker file");
    let program = scratch.path().join("claude");
    fs::copy("/bin/sh", &program).expect("inert login executable");
    let command = crate::program_login::command(
        &directories,
        OsStr::new("source"),
        [("PATH".into(), scratch.path().as_os_str().to_owned())],
        None,
    )
    .expect("worker login without link secret");
    assert_eq!(command.get_program(), program.as_os_str());
    assert_eq!(command.get_args().collect::<Vec<_>>(), ["auth", "login"]);
    assert!(!scratch.path().join("state").exists());
    assert!(!scratch.path().join("secrets/worker.secret").exists());
}

#[test]
fn toml_worker_login_refuses_both_files() {
    let scratch = skein_scratch::Scratch::new("worker-both-files");
    let directories = crate::tests::configuration::directories(scratch.path());
    fs::write(scratch.path().join(FILE), "engine = \"127.0.0.1:7200\"\n").expect("worker file");
    fs::write(scratch.path().join(FORMER), b"never decoded").expect("former file");
    let why = crate::program_login::command(&directories, OsStr::new("source"), [], None).unwrap_err();
    assert_eq!(
        why,
        format!(
            "{FORMER}: moved: the configuration is {FILE}; write it from temper-worker help config, then remove {FORMER}"
        )
    );
}
