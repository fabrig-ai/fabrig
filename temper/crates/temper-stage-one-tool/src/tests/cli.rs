//! The stage-one table's answers (temper/docs/design/testing/tools.md, 6).
use crate::TABLE;
use skein_shell::{Front, Table};

const TEST_TABLE: Table = Table { version: "0.0.0", commit: "0123456789ab", ..TABLE };

fn help(arguments: &[&str]) -> String {
    let arguments: Vec<_> = arguments.iter().map(|word| Box::from(word.as_bytes())).collect();
    let Front::Print(page) = TEST_TABLE.front(&arguments) else { panic!("help must print") };
    page
}

fn paragraph(text: &str) -> String {
    text.lines().take_while(|line| !line.is_empty()).collect::<Vec<_>>().join(" ")
}

#[test]
fn help_aliases_share_the_public_page() {
    let page = help(&["--help"]);
    assert_eq!(paragraph(&page), "temper-stage-one: configuring's stage 1: its golden, check, judge and evidence.");
    assert!(!page.contains("\ncommands:\n"));
    assert!(!page.contains("\nusage:\n"));
    for alias in ["-h", "help"] {
        assert_eq!(help(&[alias]), page, "{alias}");
    }
}

#[test]
fn each_command_help_page_says_it_is_internal() {
    for command in ["obligations", "collect", "check", "judge", "verdict"] {
        assert_eq!(
            paragraph(&help(&[command, "--help"])),
            "internal: not part of temper-stage-one's public command line; it may change without notice.",
            "{command}"
        );
    }
}
