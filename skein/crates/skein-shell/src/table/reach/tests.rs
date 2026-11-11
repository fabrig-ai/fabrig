//! Invocation precedence and no-command words (runtime/shell.md, 14.4).

use crate::{Command, Reach, Reached, Set, Table, table::demo::DEMO};

const SERVE: Command = Command {
    reach: Reach::Words(&["serve"]),
    set: Set::Public,
    forms: &[],
    line: "Serve the demo.",
    arguments: &[],
    settings: &[],
    switches: &[],
    variables: &[],
    notes: "",
};
const BARE: Command = Command { reach: Reach::Words(&[]), ..SERVE };
const GIT: Command = Command { reach: Reach::RunAs("git"), set: Set::Internal, ..SERVE };
const HELP: Command = Command { reach: Reach::Words(&["help"]), ..SERVE };
const FAKE_FORGE: Command = Command { reach: Reach::Words(&["fake", "forge"]), ..SERVE };
const FAKE_CHECK: Command = Command { reach: Reach::Words(&["fake", "forge", "check"]), ..SERVE };
const FORGE_CHECK: Command = Command { reach: Reach::Words(&["forge", "check"]), set: Set::Test, ..SERVE };
const FORGE_LIST: Command = Command { reach: Reach::Words(&["forge", "list"]), ..SERVE };
const NAMED: Table =
    Table { binary: "demo", commands: &[GIT, FAKE_CHECK, SERVE, FAKE_FORGE, HELP, FORGE_CHECK, FORGE_LIST], ..DEMO };
const WITH_BARE: Table =
    Table { commands: &[BARE, GIT, FAKE_FORGE, SERVE, FAKE_CHECK, HELP, FORGE_CHECK, FORGE_LIST], ..NAMED };
const NO_HELP: Table = Table { commands: &[SERVE], ..NAMED };

fn bytes(words: &[&[u8]]) -> Box<[Box<[u8]>]> {
    words.iter().map(|word| Box::from(*word)).collect()
}

type ReachCase<'a> = (Table, &'a [u8], &'a [&'a [u8]], Reached<'a>);

#[test]
fn invocation_precedence_reaches_pages_or_longest_command() {
    let cases: &[ReachCase<'_>] = &[
        (NAMED, b"/usr/bin/git", &[b"--help"], Reached::Command(&GIT, &bytes(&[b"--help"]))),
        (NAMED, b"git", &[b"--version"], Reached::Command(&GIT, &bytes(&[b"--version"]))),
        (WITH_BARE, b"git", &[], Reached::Command(&GIT, &[])),
        (WITH_BARE, b"git", &[b"serve", b"-h"], Reached::Command(&GIT, &bytes(&[b"serve", b"-h"]))),
        (NAMED, b"gitx", &[b"--help"], Reached::Help),
        (NAMED, b"/usr/bin/git/", &[b"--help"], Reached::Help),
        (NAMED, b"", &[b"-h"], Reached::Help),
        (NAMED, b"demo", &[b"--help"], Reached::Help),
        (NAMED, b"demo", &[b"help"], Reached::Help),
        (NO_HELP, b"demo", &[b"-h"], Reached::Help),
        (NO_HELP, b"demo", &[b"--help"], Reached::Help),
        (NO_HELP, b"demo", &[b"help"], Reached::Help),
        (WITH_BARE, b"demo", &[b"--help"], Reached::Help),
        (NAMED, b"demo", &[b"--version"], Reached::Version),
        (WITH_BARE, b"demo", &[b"--version"], Reached::Version),
        (
            NAMED,
            b"demo",
            &[b"--version", b"extra"],
            Reached::Unknown("demo: no command --version; demo --help lists them\n".into()),
        ),
        (WITH_BARE, b"demo", &[b"--version", b"extra"], Reached::Command(&BARE, &bytes(&[b"--version", b"extra"]))),
        (NAMED, b"demo", &[b"fake", b"forge", b"check", b"x"], Reached::Command(&FAKE_CHECK, &bytes(&[b"x"]))),
        (WITH_BARE, b"demo", &[b"fake", b"forge", b"check", b"x"], Reached::Command(&FAKE_CHECK, &bytes(&[b"x"]))),
        (NAMED, b"demo", &[b"fake", b"forge", b"7"], Reached::Command(&FAKE_FORGE, &bytes(&[b"7"]))),
        (NAMED, b"demo", &[b"fake", b"forge"], Reached::Command(&FAKE_FORGE, &[])),
        (NAMED, b"demo", &[b"help", b"pages"], Reached::Command(&HELP, &bytes(&[b"pages"]))),
        (NAMED, b"demo", &[b"serve"], Reached::Command(&SERVE, &[])),
        (NAMED, b"demo", &[b"serve", b"--help"], Reached::CommandHelp(&SERVE)),
        (NAMED, b"demo", &[b"serve", b"-h"], Reached::CommandHelp(&SERVE)),
        (NAMED, b"demo", &[b"fake", b"forge", b"check", b"--help"], Reached::CommandHelp(&FAKE_CHECK)),
        (NAMED, b"demo", &[b"serve", b"--help", b"x"], Reached::Command(&SERVE, &bytes(&[b"--help", b"x"]))),
        (NAMED, b"demo", &[b"serve", b"help"], Reached::Command(&SERVE, &bytes(&[b"help"]))),
        (NAMED, b"demo", &[b"serve", b"--version"], Reached::Command(&SERVE, &bytes(&[b"--version"]))),
        (NAMED, b"demo", &[b"serve", b"\xff"], Reached::Command(&SERVE, &bytes(&[b"\xff"]))),
        (NAMED, b"demo", &[b"forge", b"check"], Reached::Command(&FORGE_CHECK, &[])),
        (WITH_BARE, b"demo", &[], Reached::Command(&BARE, &[])),
        (WITH_BARE, b"demo", &[b"--data", b"D"], Reached::Command(&BARE, &bytes(&[b"--data", b"D"]))),
        (WITH_BARE, b"demo", &[b"serve", b"-h"], Reached::CommandHelp(&SERVE)),
        (NAMED, b"demo", &[], Reached::Unknown("demo: no command; demo --help lists them\n".into())),
        (
            NAMED,
            b"demo",
            &[b"forge", b"bogus"],
            Reached::Unknown("demo: no command forge bogus; demo --help lists them\n".into()),
        ),
        (
            NAMED,
            b"demo",
            &[b"bogus", b"x"],
            Reached::Unknown("demo: no command bogus; demo --help lists them\n".into()),
        ),
        (
            NAMED,
            b"demo",
            &[b"servex", b"x"],
            Reached::Unknown("demo: no command servex; demo --help lists them\n".into()),
        ),
        (NAMED, b"demo", &[b"fake"], Reached::Unknown("demo: no command fake; demo --help lists them\n".into())),
    ];
    for (table, run_as, arguments, expected) in cases {
        let arguments = bytes(arguments);
        assert_eq!(&table.reach(run_as, &arguments), expected, "run as {run_as:?}, arguments {arguments:?}");
    }
}

#[test]
fn unknown_stops_at_first_word_without_a_continuation() {
    for (table, arguments, expected) in [
        (NAMED, bytes(&[]), "demo: no command; demo --help lists them\n"),
        (WITH_BARE, bytes(&[]), "demo: no command; demo --help lists them\n"),
        (NAMED, bytes(&[b"forge", b"bogus", b"x"]), "demo: no command forge bogus; demo --help lists them\n"),
        (WITH_BARE, bytes(&[b"forge", b"bogus", b"x"]), "demo: no command forge bogus; demo --help lists them\n"),
        (NAMED, bytes(&[b"bogus", b"x"]), "demo: no command bogus; demo --help lists them\n"),
        (WITH_BARE, bytes(&[b"bogus", b"x"]), "demo: no command bogus; demo --help lists them\n"),
        (NAMED, bytes(&[b"forge"]), "demo: no command forge; demo --help lists them\n"),
        (NAMED, bytes(&[b"fake", b"forge", b"check"]), "demo: no command fake forge check; demo --help lists them\n"),
        (
            NAMED,
            bytes(&[b"fake", b"forge", b"check", b"x", b"y"]),
            "demo: no command fake forge check x; demo --help lists them\n",
        ),
        (NAMED, bytes(&[b"serve", b"x", b"y"]), "demo: no command serve x; demo --help lists them\n"),
        (NAMED, bytes(&[b"git", b"x"]), "demo: no command git; demo --help lists them\n"),
        (NAMED, bytes(&[b"forge", b"\xff", b"x"]), "demo: no command forge �; demo --help lists them\n"),
        (NAMED, bytes(&[b"", b"x"]), "demo: no command ; demo --help lists them\n"),
        (NAMED, bytes(&[b"for", b"x"]), "demo: no command for; demo --help lists them\n"),
        (
            Table { binary: "other", commands: &[], ..NAMED },
            bytes(&[b"forge", b"x"]),
            "other: no command forge; other --help lists them\n",
        ),
    ] {
        assert_eq!(table.unknown(&arguments), expected, "arguments {arguments:?}");
    }
}
