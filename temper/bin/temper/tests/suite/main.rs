//! The client's binary wiring (temper/docs/design/cli/README.md, 7.4 and 10).
#![forbid(unsafe_code)]

mod help;
mod version;

#[test]
fn a_non_address_is_refused_with_the_bare_usage() {
    let output = output(&["frob"]);
    assert_eq!(output.status.code(), Some(2));
    assert_eq!(output.stdout, b"");
    assert_eq!(output.stderr, b"temper: not an address: frob\nusage: temper [ADDRESS]\n");
}

#[test]
fn a_tui_address_needs_a_terminal() {
    let output = output(&["http://127.0.0.1:8367"]);
    assert_eq!(output.status.code(), Some(1));
    assert_eq!(output.stdout, b"");
    assert_eq!(output.stderr, b"temper: the TUI needs a terminal on stdin and stdout\n");
}

fn output(arguments: &[&str]) -> std::process::Output {
    let scratch = skein_scratch::Scratch::new("client-binary");
    std::process::Command::new(skein_scratch::paths::binary("temper"))
        .args(arguments)
        .env_clear()
        .env("TEMPER_CONFIG", scratch.path().join("config"))
        .env("TEMPER_DATA", scratch.path().join("data"))
        .env("TEMPER_CACHE", scratch.path().join("cache"))
        .current_dir(scratch.path())
        .stdin(std::process::Stdio::null())
        .output()
        .expect("run temper")
}
