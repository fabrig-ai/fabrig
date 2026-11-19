//! The box reads its own configuration at each command boundary (configuration/file.md, 6).
use std::{
    ffi::OsStr,
    fs,
    os::unix::{ffi::OsStrExt, fs::PermissionsExt},
};
use temper_config_shell::{FILE, FORMER};

fn fixture(text: &str) -> (skein_scratch::Scratch, crate::Directories) {
    let scratch = skein_scratch::Scratch::new("box-config-file");
    let directories = super::configuration::directories(scratch.path());
    fs::write(scratch.path().join(FILE), text).expect("configuration file");
    (scratch, directories)
}

#[test]
fn toml_box_prepares_from_its_one_line_file() {
    let (scratch, directories) = fixture("listen = \"127.0.0.1:7100\"\n");
    fs::write(scratch.path().join("worker.secret"), b"fixture-worker-secret").expect("fixture secret");
    fs::set_permissions(scratch.path().join("worker.secret"), fs::Permissions::from_mode(0o600)).expect("private");
    let service = crate::serve::prepare_in("temper-box", &[], b"/", &directories).unwrap_or_else(|why| panic!("{why}"));
    let crate::Service::Joined(mut service) = service else { panic!("standalone service") };
    assert!(service.operations() > 0);
    temper_engine_shell::serve::close_limits_store(service.engine_component().store()).expect("close store");
}

#[test]
fn toml_box_problem_names_public_key_and_line_on_serve_output() {
    let (_scratch, directories) =
        fixture("# configuration\n\nlisten = \"127.0.0.1:7100\"\npublic = \"http://example.com\"\n");
    let mut arguments = vec![Box::from(b"serve".as_slice())];
    arguments.extend(directories.flags());
    let command = crate::configure_with(&arguments, &[], b"/").expect("serve command");
    let why = crate::run(command, &[], b"/", b"temper-box", "test", "test").expect_err("refused before io");
    assert_eq!(why, format!("temper serve: {FILE}:4: public: invalid form\n"));
}

#[test]
fn toml_box_refuses_a_workers_key_at_its_line() {
    let (_scratch, directories) = fixture("# worker\nengine = \"127.0.0.1:7200\"\n");
    let Err(why) =
        crate::serve::prepare_in_with("temper-box", &[], b"/", &directories, &temper_service::stock::Given::Stock)
    else {
        panic!("another binary's file refused")
    };
    assert_eq!(why, format!("temper-box serve: {FILE}:2: engine: a key of temper-worker's file, not temper-box's\n"));
}

#[test]
fn toml_box_program_login_reads_the_file() {
    let (scratch, directories) = fixture("listen = \"127.0.0.1:7100\"\n");
    let program = scratch.path().join("claude");
    fs::copy("/bin/sh", &program).expect("inert login executable");
    let command = crate::program_login::command(
        &directories,
        OsStr::new("source"),
        [("PATH".into(), scratch.path().as_os_str().to_owned())],
    )
    .expect("login needs neither engine nor link secret");
    assert_eq!(command.get_program(), program.as_os_str());
    assert_eq!(command.get_args().collect::<Vec<_>>(), ["auth", "login"]);
    assert!(!scratch.path().join("state").exists());
}

#[test]
fn toml_box_upgrade_reads_the_file() {
    let (scratch, directories) = fixture("listen = \"127.0.0.1:7100\"\n");
    fs::write(scratch.path().join("worker.secret"), b"fixture-worker-secret").expect("fixture secret");
    fs::set_permissions(scratch.path().join("worker.secret"), fs::Permissions::from_mode(0o600)).expect("private");
    let (service, deployment) = crate::upgrade::prepare(&directories).unwrap_or_else(|why| panic!("{why}"));
    assert!(service.operations() > 0);
    assert_eq!(deployment.as_ref(), scratch.path().join("state").as_os_str().as_bytes());
}

#[test]
fn toml_box_commands_refuse_both_files() {
    let (scratch, directories) = fixture("listen = \"127.0.0.1:7100\"\n");
    fs::write(scratch.path().join(FORMER), b"never decoded").expect("former file");
    let expected = format!(
        "{FORMER}: moved: the configuration is {FILE}; write it from temper-box help config, then remove {FORMER}"
    );
    let Err(serve) = crate::serve::prepare_in("temper-box", &[], b"/", &directories) else { panic!("both refused") };
    assert_eq!(serve, format!("temper-box serve: {expected}\n"));
    assert_eq!(crate::program_login::command(&directories, OsStr::new("source"), []).unwrap_err(), expected);
    let Err(upgrade) = crate::upgrade::prepare(&directories) else { panic!("both refused") };
    assert_eq!(upgrade, expected);
    assert_eq!(crate::box_command::configuration(&directories, &[]).unwrap_err(), expected);
}
