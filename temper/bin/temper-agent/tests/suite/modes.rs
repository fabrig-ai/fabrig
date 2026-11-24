//! Each mode through main (temper/docs/design/protocol/binaries.md, 3).
use smith_protocol_program::{Status, status};
use std::{
    ffi::OsString,
    fs,
    os::unix::{
        ffi::{OsStrExt, OsStringExt},
        fs::{PermissionsExt, symlink},
    },
    path::{Path, PathBuf},
};
use temper_agent_shell::AGENT;

const DOCUMENT: &[u8] = br#"{"profile":{"name":"standard","declared":{"memory":1099511627776,"tool_payload":256,"read_window":256}},"endpoints":[{"name":"fake","number":0,"dialect":3,"provider":"openai","address":"127.0.0.1:8080","transport":"plaintext","connect":1000,"handshake":1000,"limits":{"window":65536,"output":512,"reasoning_item":64,"head":1000,"idle":1000}}],"environment":[]}"#;

fn agent(arguments: &[OsString]) -> std::process::Output {
    super::output(&skein_scratch::paths::binary("temper-agent"), arguments)
}

fn loopback_document(scratch: &skein_scratch::Scratch) -> OsString {
    let document = scratch.path().join("agent.json");
    fs::write(&document, DOCUMENT).expect("plaintext loopback document");
    document.into_os_string()
}

#[test]
fn the_binary_prints_its_overview() {
    let output = agent(&["--help".into()]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, AGENT.help().as_bytes());
    assert_eq!(output.stderr, b"");
}

#[test]
fn the_binary_prints_its_whole_version_line() {
    let output = agent(&["--version".into()]);
    let expected = format!("temper-agent {} ({})\n", temper_version::VERSION, temper_version::COMMIT);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, expected.as_bytes());
    assert_eq!(output.stderr, b"");
}

#[test]
fn no_document_is_a_no_command_refusal() {
    let output = agent(&[]);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(output.stdout, b"");
    assert_eq!(output.stderr, b"temper-agent: no command; temper-agent --help lists them\n");
}

#[test]
fn the_relay_reports_its_own_usage() {
    let output = agent(&["--mcp-relay".into()]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert_eq!(
        output.stderr,
        b"smith-agent: usage: smith-agent --mcp-relay 127.0.0.1:PORT, its secret in SMITH_MCP_SECRET\n"
    );
}

#[test]
fn the_program_child_forwards_auth_status_and_its_exit() {
    let scratch = skein_scratch::Scratch::new("agent-child");
    let script = scratch.path().join("claude");
    fs::write(&script, b"#!/bin/sh\nif [ \"$#\" -ne 2 ] || [ \"$1\" != auth ] || [ \"$2\" != status ]; then\nprintf '%s\\n' 'script: expected auth status' >&2\nexit 9\nfi\nprintf '%s\\n' '{\"loggedIn\":false}'\nexit 1\n").expect("status program");
    fs::set_permissions(&script, fs::Permissions::from_mode(0o700)).expect("executable status");
    let output = agent(&[
        "--program-child".into(),
        std::process::id().to_string().into(),
        script.into_os_string(),
        "auth".into(),
        "status".into(),
    ]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"{\"loggedIn\":false}\n");
    assert_eq!(output.stderr, b"");
    assert_eq!(status(1, &output.stdout), Status::NotSignedIn);
}

#[test]
fn a_run_starts_and_reports_the_closed_channel() {
    let scratch = skein_scratch::Scratch::new("agent-run");
    let output = agent(&[loopback_document(&scratch)]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert_eq!(output.stderr, b"smith: commands run in process groups and are not contained\nsmith: agent started\nsmith: the run could not answer: the channel ended before the answer\n");
}

#[test]
fn a_list_starts_and_returns_its_channel_answer() {
    let scratch = skein_scratch::Scratch::new("agent-list");
    let output = agent(&["--list".into(), loopback_document(&scratch)]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, [1, 2]);
    assert_eq!(output.stderr, b"");
}

fn launcher(directory: &Path) -> PathBuf {
    use skein_forge::forgejo::pin::Platform;
    use temper_fetch::claude_code::{self, Libc};
    let build = claude_code::build(
        if cfg!(target_os = "macos") { Platform::Macos } else { Platform::Linux },
        std::env::consts::ARCH,
        if cfg!(target_env = "musl") { Libc::Musl } else { Libc::Gnu },
    )
    .expect("pinned host build");
    let cache = directory.join("cache");
    let file = PathBuf::from(std::ffi::OsStr::from_bytes(&claude_code::file(cache.as_os_str().as_bytes())));
    fs::create_dir_all(file.parent().expect("build directory")).expect("cache directory");
    fs::write(&file, format!(
        "#!/bin/sh\nprintf '%s\\000' \"$@\" > '{}/arguments'\nprintf '%s %s' \"$DISABLE_AUTOUPDATER\" \"$DISABLE_UPDATES\" > '{}/updater'\nprintf 'command stdout\\n'\nexit 0\n",
        directory.display(), directory.display(),
    )).expect("recording build");
    fs::OpenOptions::new().write(true).open(&file).expect("build").set_len(build.bytes).expect("pinned size");
    fs::set_permissions(&file, fs::Permissions::from_mode(0o700)).expect("executable build");
    let launchers = directory.join("claude-code");
    fs::create_dir(&launchers).expect("launcher directory");
    let launcher = launchers.join("claude");
    symlink(skein_scratch::paths::binary("temper-agent"), &launcher).expect("binary link");
    symlink(cache, launchers.join("cache")).expect("cache link");
    launcher
}

#[test]
fn the_claude_link_keeps_argument_bytes_and_disables_updates() {
    let scratch = skein_scratch::Scratch::new("agent-launcher");
    let directory = scratch.path();
    let output = super::output(&launcher(directory), &[OsString::from_vec(b"one \xff\n'".to_vec()), "--two=$x".into()]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, b"command stdout\n");
    assert_eq!(output.stderr, b"");
    assert_eq!(fs::read(directory.join("arguments")).expect("recorded arguments"), b"one \xff\n'\0--two=$x\0");
    assert_eq!(fs::read(directory.join("updater")).expect("recorded updater"), b"1 1");
}
