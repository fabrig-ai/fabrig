//! Callback commands (skein/docs/foundation/testing-strategy.md, 2.10).
use super::{parent, patience::Patience};
use skein_lib::Duration;
use std::{
    fs,
    io::Write,
    os::unix::fs::PermissionsExt,
    process::{Command, Output, Stdio},
};

/// These commands run in story callbacks; the enclosing world keeps the verdict.
pub fn output(command: &mut Command, what: &'static str, within: Duration, input: Option<&[u8]>) -> Output {
    let base = skein_scratch::paths::target_tmp();
    fs::create_dir_all(&base).expect("story command scratch parent");
    let scratch = skein_scratch::Scratch::new_in(&base, "story-command");
    fs::set_permissions(scratch.path(), fs::Permissions::from_mode(0o700)).expect("private command scratch");
    let stdout = scratch.path().join("stdout");
    let stderr = scratch.path().join("stderr");
    super::write_private(&stdout, b"");
    super::write_private(&stderr, b"");
    let mut patience = Patience::arm(what, within);
    let child = command
        .stdin(if input.is_some() { Stdio::piped() } else { Stdio::null() })
        .stdout(fs::File::create(&stdout).expect("command output"))
        .stderr(fs::File::create(&stderr).expect("command errors"))
        .spawn()
        .expect("story command starts");
    let mut owner = parent::Guard::new(child);
    let child = owner.child.as_mut().expect("owned command");
    patience.watch_process(child.id());
    if let Some(input) = input {
        child.stdin.take().expect("command input").write_all(input).expect("command input written");
    }
    let status = loop {
        if let Some(status) = child.try_wait().expect("story command status") {
            break status;
        }
        if let Some(figures) = patience.failed() {
            panic!("story command exceeded its bound: {figures}");
        }
        std::thread::sleep(std::time::Duration::from_millis(5));
    };
    let output = Output {
        status,
        stdout: fs::read(stdout).expect("command output"),
        stderr: fs::read(stderr).expect("command errors"),
    };
    // The status has been reaped; no identity or pipe handle remains in the owner.
    drop(owner.child.take());
    output
}

#[cfg(test)]
mod tests {
    use super::{Duration, output};
    use std::process::Command;

    #[test]
    fn a_callback_command_captures_output_and_keeps_a_bad_exit() {
        let output = output(
            Command::new("/bin/sh").args(["-c", "cat; printf diagnostic >&2; exit 7"]),
            "the callback command fixture",
            Duration::from_secs(1),
            Some(b"request"),
        );
        assert_eq!(output.status.code(), Some(7));
        assert_eq!(output.stdout, b"request");
        assert_eq!(output.stderr, b"diagnostic");
    }
}
