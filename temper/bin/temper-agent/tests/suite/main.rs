//! The agent's binary wiring (temper/docs/design/protocol/binaries.md, 3).
#![forbid(unsafe_code)]

mod modes;

fn output(program: &std::path::Path, arguments: &[std::ffi::OsString]) -> std::process::Output {
    std::process::Command::new(program)
        .args(arguments)
        .env_clear()
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .stderr(std::process::Stdio::piped())
        .output()
        .expect("run temper-agent")
}
