#[test]
fn the_binary_prints_its_whole_version_line() {
    let output = super::output(&["--version"]);
    let expected = format!("{}\n", temper_version::line(temper_client_shell::NAME));
    assert_eq!(output.status.code(), Some(0));
    assert_eq!(output.stdout, expected.as_bytes());
    assert_eq!(output.stderr, b"");
}
