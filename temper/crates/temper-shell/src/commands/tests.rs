use super::{Row, find, help, refused, row, unknown};
use crate::rows;

const COMMANDS: [Row; 3] = [rows::HELP, rows::TUI, rows::CTL];

fn bytes(words: &[&str]) -> Vec<Box<[u8]>> {
    words.iter().map(|word| Box::from(word.as_bytes())).collect()
}

#[test]
fn a_table_prints_for_its_binarys_name() {
    let text = help("temper-client", &COMMANDS);
    assert!(text.starts_with("usage: temper-client COMMAND [ARGUMENT...]\ntemper-client help prints the index of temper's pages; temper-client COMMAND --help prints one command.\n"), "{text}");
    assert!(text.contains("  temper-client ctl DIR GROUP VERB [ARGUMENT...] [--json]\n"), "{text}");
    assert!(row("temper-client", &rows::TUI).starts_with("usage: temper-client [ADDRESS]\n"));
    for (words, expected) in [
        (vec![], "temper-client: no command; temper-client --help lists them\n"),
        (vec!["box", "serve"], "temper-client: no command box; temper-client --help lists them\n"),
    ] {
        assert_eq!(unknown("temper-client", &COMMANDS, &bytes(&words)), expected);
    }
    assert_eq!(
        refused("temper-client", &rows::TUI, "invalid arguments"),
        "temper-client: invalid arguments\nusage: temper-client [ADDRESS]\n"
    );
    for (words, expected) in [
        (vec![], Some(&rows::TUI)),
        (vec!["http://127.0.0.1:8367"], Some(&rows::TUI)),
        (vec!["ctl", "DIR", "me"], Some(&rows::CTL)),
        (vec!["box"], None),
    ] {
        assert_eq!(find(&COMMANDS, &bytes(&words)), expected, "{words:?}");
    }
}
