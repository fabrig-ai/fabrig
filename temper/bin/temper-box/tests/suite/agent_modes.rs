//! temper/docs/design/protocol/binaries.md: smith's agent modes precede command parsing.

use crate::support;
use smith_protocol_program::{Status, status};
use std::{
    fs,
    os::unix::fs::PermissionsExt,
    process::{Command, Stdio},
};

#[test]
fn the_binary_serves_smiths_agent_modes() {
    let mut failures = Vec::new();
    for (case, flag, code, stdout, stderr, expected_status) in [
        (
            "relay usage",
            "--mcp-relay",
            1_u8,
            "",
            "smith-agent: usage: smith-agent --mcp-relay 127.0.0.1:PORT, its secret in SMITH_MCP_SECRET\n",
            None,
        ),
        ("child refused", "--program-child", 1, "", "smith-agent: program child: missing parent\n", None),
        (
            "signed in",
            "--program-child",
            0,
            "{\"loggedIn\":true,\"authMethod\":\"claude.ai\"}\n",
            "",
            Some(Status::Usable),
        ),
        ("not signed in", "--program-child", 1, "{\"loggedIn\":false}\n", "", Some(Status::NotSignedIn)),
    ] {
        let scratch = support::Scratch::new(case);
        let mut command = Command::new(skein_scratch::paths::binary("temper-box"));
        command.arg(flag).env_clear().stdin(Stdio::piped()).stdout(Stdio::piped()).stderr(Stdio::piped());
        if expected_status.is_some() {
            let program = scratch.path().join("claude");
            fs::write(
                &program,
                format!(
                    "#!/bin/sh\n\
                     if [ \"$#\" -ne 2 ] || [ \"$1\" != auth ] || [ \"$2\" != status ]; then\n\
                     printf '%s\\n' 'script: expected auth status' >&2\n\
                     exit 9\n\
                     fi\n\
                     printf '%s' '{stdout}'\n\
                     exit {code}\n"
                ),
            )
            .expect("scripted program");
            fs::set_permissions(&program, fs::Permissions::from_mode(0o700)).expect("executable program");
            command.arg(std::process::id().to_string()).arg(program).args(["auth", "status"]);
        }
        let mut child = command.spawn().expect("temper agent mode");
        drop(child.stdin.take());
        let output = child.wait_with_output().expect("agent mode output");
        let program_status = expected_status.map(|_| {
            status(u8::try_from(output.status.code().expect("program exit code")).expect("byte exit"), &output.stdout)
        });
        if output.status.code() != Some(i32::from(code))
            || output.stdout != stdout.as_bytes()
            || output.stderr != stderr.as_bytes()
            || program_status != expected_status
        {
            failures.push(format!(
                "{case}: expected exit {code}, stdout {stdout:?}, stderr {stderr:?}, status {expected_status:?}; \
                 got exit {:?}, stdout {:?}, stderr {:?}, status {program_status:?}",
                output.status.code(),
                String::from_utf8_lossy(&output.stdout),
                String::from_utf8_lossy(&output.stderr),
            ));
        } else {
            scratch.remove();
        }
    }
    assert!(failures.is_empty(), "{}", failures.join("\n"));
}
