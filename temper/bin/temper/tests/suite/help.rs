#[test]
fn the_binary_prints_its_help_page() {
    let output = super::output(&["--help"]);
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, temper_client_shell::CLIENT.help().as_bytes());
    assert_eq!(output.stderr, b"");
}
