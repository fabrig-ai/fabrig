use std::process::{Command, Stdio};

#[test]
fn the_binary_prints_its_version_and_refuses_the_rest() {
    for (arguments, code, stdout, stderr) in [
        (vec!["--version"], 0, temper_client::version(), ""),
        (
            vec!["--version", "extra"],
            1,
            String::new(),
            "temper-client: no command --version extra; temper-client --help lists them\n",
        ),
        (vec!["nope"], 1, String::new(), "temper-client: no command nope; temper-client --help lists them\n"),
        (vec![], 1, String::new(), "temper-client: no command; temper-client --help lists them\n"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_temper-client"))
            .args(&arguments)
            .stdin(Stdio::null())
            .output()
            .expect("run temper-client");
        assert_eq!(output.status.code(), Some(code), "{arguments:?}");
        assert_eq!(output.stdout, stdout.as_bytes(), "{arguments:?}");
        assert_eq!(output.stderr, stderr.as_bytes(), "{arguments:?}");
    }
}
