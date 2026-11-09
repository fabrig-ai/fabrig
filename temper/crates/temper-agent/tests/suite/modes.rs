//! Each mode through main (temper/docs/design/protocol/README.md, 4.1).
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
use temper_agent::{COMMANDS, LIST, NAME};
use temper_shell::{
    commands,
    help::{self, Request},
};

const RELAY_USAGE: &str =
    "smith-agent: usage: smith-agent --mcp-relay 127.0.0.1:PORT, its secret in SMITH_MCP_SECRET\n";
const DOCUMENT: &[u8] = br#"{"profile":{"name":"standard","declared":{"memory":1099511627776,"tool_payload":256,"read_window":256}},"endpoints":[{"name":"fake","number":0,"dialect":3,"provider":"openai","address":"127.0.0.1:8080","transport":"plaintext","connect":1000,"handshake":1000,"limits":{"window":65536,"output":512,"reasoning_item":64,"head":1000,"idle":1000}}],"environment":[]}"#;

#[derive(Clone, Copy)]
enum Invocation {
    Binary,
    Program(Status),
    Launcher { right_size: bool },
}

enum Errors {
    Exact(String),
    Prefix(&'static str),
}

fn arguments(words: &[&str]) -> Vec<OsString> {
    words.iter().map(OsString::from).collect()
}

fn launcher(directory: &Path, right_size: bool) -> PathBuf {
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
        "#!/bin/sh\nprintf '%s\\000' \"$@\" > '{}/arguments'\nprintf '%s' \"$DISABLE_AUTOUPDATER\" > '{}/updater'\nprintf 'command stdout\\n'\nexit 0\n",
        directory.display(), directory.display(),
    )).expect("recording build");
    let length = if right_size { build.bytes } else { build.bytes.checked_sub(1).expect("nonempty build") };
    fs::OpenOptions::new().write(true).open(&file).expect("build").set_len(length).expect("pinned size");
    fs::set_permissions(&file, fs::Permissions::from_mode(0o700)).expect("executable build");
    let launchers = directory.join("claude-code");
    fs::create_dir(&launchers).expect("launcher directory");
    let launcher = launchers.join("claude");
    symlink(skein_scratch::paths::binary("temper-agent"), &launcher).expect("binary link");
    symlink(cache, launchers.join("cache")).expect("cache link");
    launcher
}

#[test]
fn the_binary_runs_every_mode_as_temper_does() {
    let scratch = skein_scratch::Scratch::new("agent-modes");
    let missing = scratch.path().join("missing.json");
    let document = scratch.path().join("agent.json");
    fs::write(&document, DOCUMENT).expect("plaintext loopback document");
    let absent = format!("configuration metadata: {}: file absent\n", missing.display());
    for (case, invocation, mut arguments, code, stdout, stderr) in [
        ("version", Invocation::Binary, arguments(&["--version"]), 0, temper_agent::version().into_bytes(), Errors::Exact(String::new())),
        ("overview", Invocation::Binary, arguments(&["--help"]), 0, commands::help(NAME, &COMMANDS).into_bytes(), Errors::Exact(String::new())),
        ("list help", Invocation::Binary, arguments(&["--list", "--help"]), 0, commands::row(NAME, &LIST).into_bytes(), Errors::Exact(String::new())),
        ("help index", Invocation::Binary, arguments(&["help"]), 0, help::answer(NAME, &Request::Index).expect("index").into_bytes(), Errors::Exact(String::new())),
        ("no command", Invocation::Binary, vec![], 1, vec![], Errors::Exact("temper-agent: no command; temper-agent --help lists them\n".into())),
        ("unknown", Invocation::Binary, arguments(&["-x"]), 1, vec![], Errors::Exact("temper-agent: no command -x; temper-agent --help lists them\n".into())),
        ("version extra", Invocation::Binary, arguments(&["--version", "extra"]), 1, vec![], Errors::Exact("temper-agent: no command --version; temper-agent --help lists them\n".into())),
        ("run extra", Invocation::Binary, arguments(&["a.json", "b.json"]), 1, vec![], Errors::Exact("temper-agent: invalid arguments\nusage: temper-agent AGENT.json\n".into())),
        ("list missing argument", Invocation::Binary, arguments(&["--list"]), 1, vec![], Errors::Exact("temper-agent --list: invalid arguments\nusage: temper-agent --list AGENT.json\n".into())),
        ("relay usage", Invocation::Binary, arguments(&["--mcp-relay"]), 1, vec![], Errors::Exact(RELAY_USAGE.into())),
        ("relay help", Invocation::Binary, arguments(&["--mcp-relay", "--help"]), 1, vec![], Errors::Exact(RELAY_USAGE.into())),
        ("child refused", Invocation::Binary, arguments(&["--program-child"]), 1, vec![], Errors::Exact("smith-agent: program child: missing parent\n".into())),
        ("signed in", Invocation::Program(Status::Usable), vec![], 0, b"{\"loggedIn\":true,\"authMethod\":\"claude.ai\"}\n".to_vec(), Errors::Exact(String::new())),
        ("not signed in", Invocation::Program(Status::NotSignedIn), vec![], 1, b"{\"loggedIn\":false}\n".to_vec(), Errors::Exact(String::new())),
        ("run missing", Invocation::Binary, vec![missing.clone().into_os_string()], 1, vec![], Errors::Exact(format!("smith: {absent}temper-agent: {absent}"))),
        ("list missing", Invocation::Binary, vec!["--list".into(), missing.into_os_string()], 1, vec![], Errors::Exact(format!("temper-agent --list: {absent}"))),
        ("run", Invocation::Binary, vec![document.clone().into_os_string()], 1, vec![], Errors::Exact("smith: commands run in process groups and are not contained\nsmith: agent started\nsmith: the run could not answer: the channel ended before the answer\n".into())),
        ("list", Invocation::Binary, vec!["--list".into(), document.into_os_string()], 0, vec![1, 2], Errors::Exact(String::new())),
        ("cached launcher", Invocation::Launcher { right_size: true }, vec![OsString::from_vec(b"one \xff\n'".to_vec()), "--two=$x".into()], 0, b"command stdout\n".to_vec(), Errors::Exact(String::new())),
        ("wrong sized launcher", Invocation::Launcher { right_size: false }, vec![], 1, vec![], Errors::Prefix("temper-agent: claude: ")),
    ] {
        let directory = scratch.path().join(case);
        fs::create_dir(&directory).expect("row directory");
        let program = match invocation {
            Invocation::Binary => skein_scratch::paths::binary("temper-agent"),
            Invocation::Program(_) => {
                let program = directory.join("claude");
                fs::write(&program, format!(
                    "#!/bin/sh\nif [ \"$#\" -ne 2 ] || [ \"$1\" != auth ] || [ \"$2\" != status ]; then\nprintf '%s\\n' 'script: expected auth status' >&2\nexit 9\nfi\nprintf '%s' '{}'\nexit {code}\n",
                    String::from_utf8_lossy(&stdout),
                )).expect("status program");
                fs::set_permissions(&program, fs::Permissions::from_mode(0o700)).expect("executable status");
                arguments = vec!["--program-child".into(), std::process::id().to_string().into(), program.into_os_string(), "auth".into(), "status".into()];
                skein_scratch::paths::binary("temper-agent")
            }
            Invocation::Launcher { right_size } => launcher(&directory, right_size),
        };
        let output = super::output(&program, &arguments);
        assert_eq!(output.status.code(), Some(code), "{case}: {}", String::from_utf8_lossy(&output.stderr));
        assert_eq!(output.stdout, stdout, "{case}");
        match stderr {
            Errors::Exact(expected) => assert_eq!(output.stderr, expected.as_bytes(), "{case}"),
            Errors::Prefix(expected) => assert!(output.stderr.starts_with(expected.as_bytes()), "{case}: {}", String::from_utf8_lossy(&output.stderr)),
        }
        match invocation {
            Invocation::Program(expected) => assert_eq!(status(u8::try_from(code).expect("byte status"), &output.stdout), expected, "{case}"),
            Invocation::Launcher { right_size: true } => {
                assert_eq!(fs::read(directory.join("arguments")).expect("recorded arguments"), b"one \xff\n'\0--two=$x\0");
                assert_eq!(fs::read(directory.join("updater")).expect("recorded updater"), b"1");
            }
            Invocation::Binary | Invocation::Launcher { right_size: false } => {}
        }
    }
}
