//! The worker's file reaches its part and program-login without an engine (configuration/file.md, 6).
use crate::tests::machine::{Disk, ROOT};
use skein_fake_machine::{Item, Refusal};
use std::{ffi::OsStr, fs, os::unix::ffi::OsStrExt, path::Path};
use temper_config_shell::{FILE, FORMER, Shape};

const WORKER_TOML: &[u8] = b"engine = \"127.0.0.1:7200\"\nname = \"w1\"\n";

#[test]
fn toml_worker_prepares_its_part_from_the_file() {
    let root = Path::new(ROOT);
    let directories = crate::tests::configuration::directories(root);
    let disk =
        Disk::new(&[Item::file(FILE.as_bytes(), WORKER_TOML), Item::file(b"temper-agent", b"agent").mode(0o700)]);
    let read = disk.configuration(Shape::WorkerAlone);
    let mut part = temper_worker_service::startup::worker_part(
        &read.document,
        &temper_service_shell::jig_directories(&directories),
        None,
        &[],
    )
    .expect("worker part");
    let executable = root.join("temper-worker");
    let agent = root.join("temper-agent");
    crate::worker::inputs_with(&mut part, &executable, disk.former_agent(), |path| disk.executable(path))
        .expect("derived worker program");
    assert_eq!(part.named.program.as_ref(), agent.as_os_str().as_bytes());
}

#[test]
fn toml_worker_program_login_reads_the_file() {
    let root = Path::new(ROOT);
    let disk = Disk::new(&[Item::file(FILE.as_bytes(), WORKER_TOML), Item::file(b"claude", b"claude").mode(0o700)]);
    let program = root.join("claude");
    let command = disk.login().expect("worker login without link secret");
    assert_eq!(command.get_program(), program.as_os_str());
    assert_eq!(command.get_args().collect::<Vec<_>>(), ["auth", "login"]);
    assert_eq!(disk.metadata(&root.join("state")), Err(Refusal::NotFound));
    assert_eq!(disk.metadata(&root.join("secrets/worker.secret")), Err(Refusal::NotFound));
}

#[test]
fn toml_worker_program_login_uses_the_named_claude() {
    let program = Path::new(ROOT).join("named-claude");
    let text = format!("engine = \"127.0.0.1:7200\"\n[programs]\nclaude = {program:?}\n");
    let disk = Disk::new(&[
        Item::file(FILE.as_bytes(), text.as_bytes()),
        Item::file(b"named-claude", b"named").mode(0o700),
        Item::file(b"claude", b"claude").mode(0o700),
    ]);
    let command = disk.login().expect("named login");
    assert_eq!(command.get_program(), program.as_os_str());
}

#[test]
fn toml_worker_login_refuses_the_former_file_alone_or_beside_the_current_file() {
    for present in [false, true] {
        let scratch = skein_scratch::Scratch::new("worker-both-files");
        let directories = crate::tests::configuration::directories(scratch.path());
        fs::write(scratch.path().join(FILE), "engine = \"127.0.0.1:7200\"\n").expect("worker file");
        fs::create_dir(scratch.path().join(FORMER)).expect("former directory");
        if !present {
            fs::remove_file(scratch.path().join(FILE)).expect("current absent");
        }
        let why = crate::program_login::command(&directories, OsStr::new("source"), []).unwrap_err();
        assert_eq!(
            why,
            format!(
                "{FORMER}: moved: the configuration is {FILE}; write it from temper-worker help config, then remove {FORMER}"
            )
        );
    }
}
