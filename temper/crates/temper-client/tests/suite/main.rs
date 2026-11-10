//! The client's binary wiring (temper/docs/design/protocol/binaries.md).
#![forbid(unsafe_code)]

mod help;
mod version;

fn output(arguments: &[&str]) -> std::process::Output {
    std::process::Command::new(skein_scratch::paths::binary("temper-client"))
        .args(arguments)
        .stdin(std::process::Stdio::null())
        .stdout(std::process::Stdio::piped())
        .output()
        .expect("run temper-client")
}
