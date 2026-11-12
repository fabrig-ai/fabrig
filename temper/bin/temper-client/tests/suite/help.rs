use temper_client_shell::{COMMANDS, NAME};
use temper_shell::{
    commands,
    help::{Request, answer},
    rows,
};

#[test]
fn the_binary_answers_its_help_and_refuses_what_it_hasnt() {
    for (arguments, code, stdout, stderr) in [
        (vec!["--help"], 0, commands::help(NAME, &COMMANDS), ""),
        (vec!["ctl", "--help"], 0, commands::row(NAME, &rows::CTL), ""),
        (vec!["help"], 0, answer(NAME, &Request::Index).expect("index page"), ""),
        (vec!["box"], 1, String::new(), "temper-client: no command box; temper-client --help lists them\n"),
        (vec![], 1, String::new(), "temper-client: no command; temper-client --help lists them\n"),
    ] {
        let output = super::output(&arguments);
        assert_eq!(output.status.code(), Some(code), "{arguments:?}");
        assert_eq!(output.stdout, stdout.as_bytes(), "{arguments:?}");
        assert_eq!(output.stderr, stderr.as_bytes(), "{arguments:?}");
    }
}
