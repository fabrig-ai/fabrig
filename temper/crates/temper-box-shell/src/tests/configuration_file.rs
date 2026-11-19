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
    let mut service = service;
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
    assert_eq!(why, format!("temper-box serve: {FILE}:4: public: invalid form\n"));
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
fn toml_box_program_login_uses_the_named_claude() {
    let (scratch, directories) = fixture("listen = \"127.0.0.1:7100\"\n");
    let program = scratch.path().join("named-claude");
    fs::copy("/bin/sh", &program).expect("named executable");
    fs::copy("/bin/sh", scratch.path().join("claude")).expect("PATH executable");
    fs::write(scratch.path().join(FILE), format!("listen = \"127.0.0.1:7100\"\n[programs]\nclaude = {program:?}\n"))
        .expect("box file");
    let command = crate::program_login::command(
        &directories,
        OsStr::new("source"),
        [("PATH".into(), scratch.path().as_os_str().to_owned())],
    )
    .expect("named login");
    assert_eq!(command.get_program(), program.as_os_str());
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
fn toml_box_git_wins_over_path_and_absent_key_uses_path() {
    use skein_io::kernel::{Complete, Done, Fd, Op};
    use skein_lib::{Time, Wall};
    for (explicit, on_path) in [(true, true), (true, false), (false, true)] {
        let scratch = skein_scratch::Scratch::new("box-program-git");
        let root = scratch.path();
        let directories = super::configuration::directories(root);
        let named = root.join("named-git");
        let found = root.join("git");
        fs::copy("/bin/sh", &named).expect("named executable");
        fs::copy("/bin/sh", &found).expect("PATH executable");
        fs::write(root.join("worker.secret"), b"fixture-worker-secret").expect("fixture secret");
        fs::set_permissions(root.join("worker.secret"), fs::Permissions::from_mode(0o600)).expect("private");
        let text = if explicit {
            format!("listen = \"127.0.0.1:7100\"\n[programs]\ngit = {named:?}\n")
        } else {
            "listen = \"127.0.0.1:7100\"\n".into()
        };
        fs::write(root.join(FILE), text).expect("box file");
        let environment = if on_path {
            vec![(Box::from(b"PATH".as_slice()), Box::from(root.as_os_str().as_bytes()))]
        } else {
            Vec::new()
        };
        let mut service =
            crate::serve::prepare_in("temper-box", &environment, b"/", &directories).expect("prepared box");
        let mut program = None;
        let mut descriptor = 1000_i32;
        for _ in 0..32 {
            temper_service::iterate(&mut service, Time::ZERO, Wall::EPOCH);
            for _ in 0..service.submissions().len() {
                let submit = service.submissions().pop().expect("counted submissions");
                let result = match &submit.kind {
                    Op::Socket { .. } => {
                        descriptor += 1;
                        Some(Done::Fd(Fd::new(descriptor)))
                    }
                    Op::Bind { addr, .. } => Some(Done::Bound(*addr)),
                    Op::Listen { .. } => Some(Done::Nothing),
                    Op::Spawn { spawn } => {
                        assert_eq!(spawn.args.last().expect("version argument").as_ref(), b"version");
                        program = Some(spawn.program.clone());
                        None
                    }
                    Op::Accept { .. }
                    | Op::Connect { .. }
                    | Op::Recv { .. }
                    | Op::Send { .. }
                    | Op::Shutdown { .. }
                    | Op::Close { .. }
                    | Op::Open { .. }
                    | Op::Read { .. }
                    | Op::Write { .. }
                    | Op::Append { .. }
                    | Op::Truncate { .. }
                    | Op::Sync { .. }
                    | Op::Lock { .. }
                    | Op::Stat { .. }
                    | Op::Rename { .. }
                    | Op::HardLink { .. }
                    | Op::Remove { .. }
                    | Op::MakeDirectory { .. }
                    | Op::List { .. }
                    | Op::Wait { .. }
                    | Op::Signal { .. }
                    | Op::Usage
                    | Op::Window { .. }
                    | Op::ReadSignal { .. }
                    | Op::PipeRead { .. }
                    | Op::PipeWrite { .. }
                    | Op::Cancel { .. } => None,
                };
                if let Some(result) = result {
                    service.completions().push(Complete { op: submit.op, kind: submit.kind, result: Ok(result) });
                }
            }
            if program.is_some() {
                break;
            }
        }
        let expected = if explicit { &named } else { &found };
        assert_eq!(program.as_deref(), Some(expected.as_os_str().as_bytes()));
        temper_engine_shell::serve::close_limits_store(service.engine_component().store()).expect("close store");
    }
}

#[test]
fn toml_box_claude_wins_then_path_then_fetched_in_pinned_configuration() {
    let scratch = skein_scratch::Scratch::new("box-program-claude");
    let root = scratch.path();
    let directories = super::configuration::directories(root);
    let named = root.join("named-claude");
    let found = root.join("claude");
    fs::copy("/bin/sh", &named).expect("named executable");
    fs::copy("/bin/sh", &found).expect("PATH executable");
    fs::write(root.join("worker.secret"), b"fixture-worker-secret").expect("fixture secret");
    fs::set_permissions(root.join("worker.secret"), fs::Permissions::from_mode(0o600)).expect("private");
    for (explicit, on_path) in [(true, true), (true, false), (false, true), (false, false)] {
        let text = if explicit {
            format!("listen = \"127.0.0.1:7100\"\n[programs]\nclaude = {named:?}\n")
        } else {
            "listen = \"127.0.0.1:7100\"\n".into()
        };
        fs::write(root.join(FILE), text).expect("box file");
        let environment = if on_path {
            vec![(Box::from(b"PATH".as_slice()), Box::from(root.as_os_str().as_bytes()))]
        } else {
            Vec::new()
        };
        let mut owned = Vec::new();
        let read = temper_service_shell::configuration::read(&directories, temper_config_shell::Shape::Standalone)
            .expect("box read");
        let programs = read.programs().expect("checked programs");
        let (service, _) = crate::serve::prepare_with_store(
            "temper-box",
            &environment,
            b"/",
            &directories,
            &read.document,
            &mut owned,
            false,
            &programs,
        )
        .expect("prepared box");
        let mut service = service;
        let bytes = fs::read(root.join("workspaces/agent/agent.json")).expect("pinned agent");
        let written: serde_json::Value = serde_json::from_slice(&bytes).expect("agent JSON");
        let expected = if explicit {
            named.to_str().expect("named path")
        } else if on_path {
            found.to_str().expect("PATH path")
        } else {
            "fetched"
        };
        assert_eq!(written["programs"]["claude_code"]["path"], expected);
        temper_engine_shell::serve::close_limits_store(service.engine_component().store()).expect("close store");
        drop(service);
        crate::close(owned);
    }
}

#[test]
fn toml_box_program_paths_refuse_at_the_keys_line_before_preparation() {
    let (scratch, directories) = fixture("listen = \"127.0.0.1:7100\"\n");
    let missing = scratch.path().join("missing");
    let file = scratch.path().join("not-executable");
    fs::write(&file, b"inert").expect("nonexecutable file");
    fs::set_permissions(&file, fs::Permissions::from_mode(0o600)).expect("not executable");
    for key in ["git", "claude", "forgejo", "runner"] {
        for path in [missing.as_path(), file.as_path(), scratch.path()] {
            fs::write(
                scratch.path().join(FILE),
                format!("listen = \"127.0.0.1:7100\"\n[programs]\n{key} = {path:?}\n"),
            )
            .expect("box file");
            let Err(why) = crate::serve::prepare_in("temper-box", &[], b"/", &directories) else {
                panic!("bad program refused")
            };
            assert_eq!(
                why,
                format!(
                    "temper-box serve: {FILE}:3: programs.{key}: {}: must be an executable regular file\n",
                    path.display()
                )
            );
        }
        fs::write(
            scratch.path().join(FILE),
            format!("listen = \"127.0.0.1:7100\"\n[programs]\n{key} = \"relative/tool\"\n"),
        )
        .expect("box file");
        let Err(why) = crate::serve::prepare_in("temper-box", &[], b"/", &directories) else {
            panic!("relative program refused")
        };
        assert!(why.starts_with(&format!("temper-box serve: {FILE}:3: programs.{key}: ")), "{why}");
        assert!(why.contains("absolute"), "{why}");
    }
}

#[test]
fn toml_box_commands_refuse_the_former_file_alone_or_beside_the_current_file() {
    for present in [false, true] {
        let (scratch, directories) = fixture("listen = \"127.0.0.1:7100\"\n");
        fs::create_dir(scratch.path().join(FORMER)).expect("former directory");
        if !present {
            fs::remove_file(scratch.path().join(FILE)).expect("current absent");
        }
        let expected = format!(
            "{FORMER}: moved: the configuration is {FILE}; write it from temper-box help config, then remove {FORMER}"
        );
        let Err(serve) = crate::serve::prepare_in("temper-box", &[], b"/", &directories) else {
            panic!("both refused")
        };
        assert_eq!(serve, format!("temper-box serve: {expected}\n"));
        assert_eq!(crate::program_login::command(&directories, OsStr::new("source"), []).unwrap_err(), expected);
        let Err(upgrade) = crate::upgrade::prepare(&directories) else { panic!("both refused") };
        assert_eq!(upgrade, expected);
        assert_eq!(crate::box_command::configuration(&directories, &[]).unwrap_err(), expected);
        let jig = temper_service_shell::jig_directories(&directories);
        let setup = temper_forge_setup::setup_file::chosen(
            temper_forge_setup::machine::BoxKind::Owned,
            3007,
            b"sam",
            false,
            &jig,
        )
        .expect("owned setup");
        temper_forge_setup_shell::setup_file::write(&jig, 7, &setup).expect("setup file");
        for words in [
            &["fetch", "7"][..],
            &["prepare", "7"][..],
            &["run", "7", "forgejo"][..],
            &["run", "7", "runner"][..],
            &["source", "7"][..],
            &["sign-in", "7"][..],
        ] {
            let arguments: Vec<_> = words.iter().map(std::ffi::OsString::from).collect();
            let why =
                crate::forge::start(&directories, &arguments, None, &[]).unwrap_err().line_with_binary("temper-box");
            assert_eq!(why, format!("temper-box forge {}: forge 7: {expected}\n", words.first().expect("verb")));
        }
    }
}
