//! Callback commands (skein/docs/foundation/testing-strategy.md, 2.10).
use super::parent;
use skein_lib::Duration;
use skein_world::{Deadline, Expired};
use std::{
    fs,
    io::Write,
    os::unix::fs::PermissionsExt,
    process::{Command, ExitStatus, Output, Stdio},
};

/// These commands run in story callbacks; the enclosing world keeps the verdict.
pub fn output(command: &mut Command, what: &'static str, within: Duration, input: Option<&[u8]>) -> Output {
    capture(input, |input, stdout, stderr| complete(command, what, within, input, true, stdout, stderr))
}

/// A command before any world starts must end and report its own starvation.
pub fn standalone(command: &mut Command, what: &'static str, within: Duration) -> Output {
    capture(None, |input, stdout, stderr| complete(command, what, within, input, false, stdout, stderr))
}

fn capture(
    input: Option<&[u8]>,
    complete: impl FnOnce(Option<&[u8]>, &mut Vec<u8>, &mut Vec<u8>) -> ExitStatus,
) -> Output {
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let status = complete(input, &mut stdout, &mut stderr);
    Output { status, stdout, stderr }
}

fn complete(
    command: &mut Command,
    what: &'static str,
    within: Duration,
    input: Option<&[u8]>,
    defer: bool,
    captured_stdout: &mut Vec<u8>,
    captured_stderr: &mut Vec<u8>,
) -> ExitStatus {
    let base = skein_scratch::paths::target_tmp();
    fs::create_dir_all(&base).expect("story command scratch parent");
    let scratch = skein_scratch::Scratch::new_in(&base, "story-command");
    fs::set_permissions(scratch.path(), fs::Permissions::from_mode(0o700)).expect("private command scratch");
    let stdout = scratch.path().join("stdout");
    let stderr = scratch.path().join("stderr");
    super::write_private(&stdout, b"");
    super::write_private(&stderr, b"");
    let mut deadline = Some(Deadline::arm(what, within));
    let child = command
        .stdin(if input.is_some() { Stdio::piped() } else { Stdio::null() })
        .stdout(fs::File::create(&stdout).expect("command output"))
        .stderr(fs::File::create(&stderr).expect("command errors"))
        .spawn()
        .expect("story command starts");
    let mut owner = parent::Guard::new(child);
    let child = owner.child.as_mut().expect("owned command");
    deadline.as_mut().expect("armed command").watch_process(child.id());
    if let Some(input) = input {
        child.stdin.take().expect("command input").write_all(input).expect("command input written");
    }
    let status = loop {
        if let Some(status) = child.try_wait().expect("story command status") {
            break status;
        }
        if let Some(bound) = &deadline
            && bound.passed()
        {
            match bound.expired() {
                Expired::Starved(_) if defer => deadline = None,
                Expired::Starved(starved) => {
                    drop(owner);
                    starved.pass();
                }
                Expired::Failed(figures) => panic!("story command exceeded its bound: {figures}"),
            }
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    };
    *captured_stdout = fs::read(stdout).expect("command output");
    *captured_stderr = fs::read(stderr).expect("command errors");
    // The status has been reaped; no identity or pipe handle remains in the owner.
    drop(owner.child.take());
    status
}

pub mod tests {
    use super::capture;
    use std::{os::unix::process::ExitStatusExt, process::ExitStatus};

    pub fn a_callback_command_captures_output_and_keeps_a_bad_exit() {
        let output = capture(Some(b"request"), |input, stdout, stderr| {
            stdout.extend_from_slice(input.expect("callback stdin"));
            stderr.extend_from_slice(b"diagnostic");
            ExitStatus::from_raw(7 << 8)
        });
        assert_eq!(output.status.code(), Some(7));
        assert_eq!(output.stdout, b"request");
        assert_eq!(output.stderr, b"diagnostic");
    }
}
