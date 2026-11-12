#[test]
fn the_binary_prints_its_version_and_refuses_the_rest() {
    for (arguments, code, stdout, stderr) in [
        (vec!["--version"], 0, format!("{}\n", temper_version::line(temper_client_shell::NAME)), ""),
        (
            vec!["--version", "extra"],
            1,
            String::new(),
            "temper-client: no command --version; temper-client --help lists them\n",
        ),
        (vec!["nope"], 1, String::new(), "temper-client: no command nope; temper-client --help lists them\n"),
        (vec![], 1, String::new(), "temper-client: no command; temper-client --help lists them\n"),
    ] {
        let output = super::output(&arguments);
        assert_eq!(output.status.code(), Some(code), "{arguments:?}");
        assert_eq!(output.stdout, stdout.as_bytes(), "{arguments:?}");
        assert_eq!(output.stderr, stderr.as_bytes(), "{arguments:?}");
    }
}
