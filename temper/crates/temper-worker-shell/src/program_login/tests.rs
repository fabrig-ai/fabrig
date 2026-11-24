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
fn login_selects_and_places_only_the_fetched_launcher() {
    let directory = Path::new(ROOT);
    let directories = crate::tests::configuration::directories(directory);
    let (_, input) = crate::tests::worker_document(directory);
    let part = temper_worker_service::startup::worker_part(
        &crate::tests::worker_start(&input),
        &jig_directories(&directories),
        None,
        &[],
    )
    .expect("worker fixture");
    let bin = directory.join("program-bin");
    let on_path = bin.join("claude");
    let launcher = directory.join("claude-code/claude");
    let binary = directory.join("temper-box");
    let mut placements = Vec::new();
    for (found_on_path, selected) in [(true, &on_path), (false, &launcher)] {
        let command = from_machine_with(
            &part,
            &directories,
            "source",
            vec![("PATH".into(), bin.as_os_str().into())],
            |candidate| {
                assert_eq!(candidate, on_path);
                found_on_path
            },
            |programs, program, name| {
                assert_eq!(programs, directory.join("programs"));
                place_launcher_with(
                    programs,
                    program,
                    super::super::path(&directories.cache),
                    &binary,
                    |candidate| {
                        assert_eq!(candidate, binary);
                        true
                    },
                    |place, cache, target| {
                        placements.push((place.to_path_buf(), cache.to_path_buf(), target.to_path_buf()));
                        Ok(())
                    },
                )?;
                Ok(programs.join(name))
            },
        )
        .expect("prepared login");
        assert_eq!(command.get_program(), selected.as_os_str());
        assert_eq!(command.get_args().collect::<Vec<_>>(), ["auth", "login"]);
        if found_on_path {
            assert!(placements.is_empty(), "the PATH row places neither link");
            // The resolved PATH program is `false`: its completion, not a launcher refusal.
            let mut streams = None;
            let status = run_command_with(
                command,
                false,
                |_| panic!("no terminal"),
                |exec| {
                    assert_eq!(exec.get_program(), on_path.as_os_str());
                    streams = Some((b"".as_slice(), b"".as_slice()));
                    Ok(1)
                },
            );
            assert_eq!(status, Ok(1));
            let (stdout, stderr) = streams.expect("the direct login ran on the fake");
            assert_eq!(stdout, b"");
            assert!(!String::from_utf8_lossy(stderr).contains("fetching"));
        } else {
            assert_eq!(placements, [(directory.join("claude-code"), directory.to_path_buf(), binary.clone())]);
        }
    }
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
