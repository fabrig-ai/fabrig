#[test]
fn the_binary_prints_its_version_and_refuses_the_rest() {
    for (arguments, code, stdout, stderr) in [
        (vec!["--version"], 0, format!("{}\n", temper_version::line(temper_client_shell::NAME)), ""),
        (vec!["--version", "extra"], 2, String::new(), "temper: invalid arguments\nusage: temper [ADDRESS]\n"),
        (vec!["nope"], 2, String::new(), "temper: not an address: nope\nusage: temper [ADDRESS]\n"),
        (vec![], 1, String::new(), "temper: no command; temper --help lists them\n"),
    ] {
        let output = super::output(&arguments);
        assert_eq!(output.status.code(), Some(code), "{arguments:?}");
        assert_eq!(output.stdout, stdout.as_bytes(), "{arguments:?}");
        assert_eq!(output.stderr, stderr.as_bytes(), "{arguments:?}");
    }
}
