//! The two box commands' process wiring, using a sibling script rather than a download.
use std::{
    fs,
    io::Write,
    os::unix::{
        ffi::{OsStrExt, OsStringExt},
        fs::PermissionsExt,
    },
    process::{Command, Stdio},
};
use temper_client_shell::CLIENT;

#[test]
fn the_client_finds_its_sibling_and_execs_it_with_unchanged_arguments_and_descriptors() {
    let scratch = skein_scratch::Scratch::new("client-box-sibling");
    let root = scratch.path();
    let client = root.join("temper");
    fs::copy(skein_scratch::paths::binary("temper"), &client).expect("copy client");
    let box_file = root.join("temper-box");
    let identity = CLIENT.version_line().replacen("temper ", "temper-box ", 1);
    fs::write(&box_file, format!("#!/bin/sh\nif [ \"$1\" = --version ]; then printf '{identity}'; exit 0; fi\nprintf 'pid=%s env=%s\\n' \"$$\" \"$BOX_TEST\"\nprintf '<%s>\\n' \"$@\"\nIFS= read -r input\nprintf 'stdin=%s\\n' \"$input\"\nprintf 'box diagnostic\\n' >&2\nexit 23\n")).expect("script");
    fs::set_permissions(&box_file, fs::Permissions::from_mode(0o755)).expect("executable sibling");
    let configure = |command: &mut Command| {
        command
            .env_clear()
            .env("TEMPER_CONFIG", root.join("config"))
            .env("TEMPER_DATA", root.join("data"))
            .env("TEMPER_CACHE", root.join("cache"))
            .env("BOX_TEST", "kept")
            .current_dir(root);
    };
    let mut fetch = Command::new(&client);
    configure(&mut fetch);
    let output = fetch.args(["internal", "fetch-box"]).stdin(Stdio::null()).output().expect("fetch sibling");
    assert_eq!(output.status.code(), Some(0));
    let mut expected = b"found ".to_vec();
    expected.extend_from_slice(box_file.as_os_str().as_bytes());
    expected.push(b'\n');
    assert_eq!(output.stdout, expected);
    assert!(output.stderr.is_empty());
    let mut command = Command::new(&client);
    configure(&mut command);
    let raw = std::ffi::OsString::from_vec(b"bytes-\xff".to_vec());
    let mut child = command
        .args(["box", "--echo", "steps", "--cache", "relative-cache", "serve", "a b", ""])
        .arg(raw)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .expect("box exec");
    let pid = child.id();
    child.stdin.take().expect("stdin").write_all(b"kept input\n").expect("input");
    let output = child.wait_with_output().expect("box output");
    assert_eq!(output.status.code(), Some(23), "the exec'd box owns its exit");
    let mut expected =
        format!("pid={pid} env=kept\n<--echo>\n<steps>\n<--cache>\n<relative-cache>\n<serve>\n<a b>\n<>\n")
            .into_bytes();
    expected.extend_from_slice(b"<bytes-\xff>\nstdin=kept input\n");
    assert_eq!(output.stdout, expected, "exec keeps the PID, arguments, environment and input");
    assert_eq!(output.stderr, b"box diagnostic\n", "a sibling lookup says nothing");
    assert!(!root.join("cache").exists());
    assert!(!root.join("relative-cache").exists());
}
