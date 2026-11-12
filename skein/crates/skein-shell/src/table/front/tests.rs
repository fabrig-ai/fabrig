//! The front's ordered rules and refusals (runtime/shell.md, 14.5).

use crate::table::demo::{DEMO, FRONT_BARE, FRONT_RUN_AS, FRONT_WORDS, OTHER};
use crate::{Command, Front, Reach, Table};

fn arguments(bytes: &[&[u8]]) -> Box<[Box<[u8]>]> {
    bytes.iter().map(|word| Box::from(*word)).collect()
}

fn command(table: &Table, reach: Reach) -> &Command {
    table.commands.iter().find(|command| command.reach == reach).expect("the demo table has this command")
}

#[test]
fn front_answers_the_first_rule_that_holds() {
    let bare = command(&DEMO, Reach::Words(&[]));
    let init = command(&DEMO, Reach::Words(&["init"]));
    let help = command(&DEMO, Reach::Words(&["help"]));
    let serve = command(&OTHER, Reach::Words(&["serve"]));
    let internal = command(&FRONT_WORDS, Reach::Words(&["internal"]));
    let list = command(&FRONT_WORDS, Reach::Words(&["internal", "list"]));
    let out = command(&FRONT_BARE, Reach::Words(&[]));
    let a_help = arguments(&[b"a", b"--help"]);
    let x = arguments(&[b"x"]);
    let other = arguments(&[b"other"]);
    let x_y = arguments(&[b"x", b"--y"]);
    let pages = arguments(&[b"pages"]);
    let help_x = arguments(&[b"help", b"x"]);
    let version = arguments(&[b"--version"]);
    let dash_help = arguments(&[b"--", b"--help"]);
    let cases: &[(&Table, &[&[u8]], Front<'_>)] = &[
        (&OTHER, &[b"--help", b"x"], Front::Print(OTHER.help())),
        (&OTHER, &[b"-h"], Front::Print(OTHER.help())),
        (&OTHER, &[b"help"], Front::Print(OTHER.help())),
        (&OTHER, &[b"--version", b"x"], Front::Print(OTHER.version_line())),
        (&OTHER, &[b"serve", b"--help", b"x"], Front::Print(OTHER.command_help(serve))),
        (&OTHER, &[b"serve", b"-h", b"x"], Front::Print(OTHER.command_help(serve))),
        (&OTHER, &[b"serve", b"a", b"--help"], Front::Run { command: serve, rest: &a_help }),
        (
            &OTHER,
            &[b"nope", b"x"],
            Front::Refuse("demo-engine: no command nope; demo-engine --help says more\n".into()),
        ),
        (&OTHER, &[], Front::Refuse("demo-engine: no command; demo-engine --help says more\n".into())),
        (
            &OTHER,
            &[b"\xffx"],
            Front::Refuse("demo-engine: no command \u{fffd}x; demo-engine --help says more\n".into()),
        ),
        (&FRONT_WORDS, &[b"internal", b"list", b"x"], Front::Run { command: list, rest: &x }),
        (&FRONT_WORDS, &[b"internal", b"other"], Front::Run { command: internal, rest: &other }),
        (&FRONT_WORDS, &[b"internal"], Front::Run { command: internal, rest: &[] }),
        (&FRONT_WORDS, &[b"internal", b"list", b"--help"], Front::Print(FRONT_WORDS.command_help(list))),
        (&DEMO, &[], Front::Run { command: bare, rest: &[] }),
        (&DEMO, &[b"x", b"--y"], Front::Run { command: bare, rest: &x_y }),
        (&DEMO, &[b"init"], Front::Run { command: init, rest: &[] }),
        (&DEMO, &[b"-h"], Front::Print(DEMO.help())),
        (&DEMO, &[b"help"], Front::Run { command: help, rest: &[] }),
        (&DEMO, &[b"help", b"pages"], Front::Run { command: help, rest: &pages }),
        (&DEMO, &[b"--help"], Front::Print(DEMO.help())),
        (&DEMO, &[b"--version"], Front::Print(DEMO.version_line())),
        (&FRONT_BARE, &[b"help"], Front::Print(FRONT_BARE.help())),
        (&FRONT_BARE, &[b"help", b"x"], Front::Run { command: out, rest: &help_x }),
        (&FRONT_RUN_AS, &[b"git"], Front::Refuse("demo-engine: no command git; demo-engine --help says more\n".into())),
        (&OTHER, &[b"serve", b"--version"], Front::Run { command: serve, rest: &version }),
        (&OTHER, &[b"serve", b"--", b"--help"], Front::Run { command: serve, rest: &dash_help }),
    ];
    for (table, bytes, expected) in cases {
        let arguments = arguments(bytes);
        let actual = table.front(&arguments);
        assert_eq!(&actual, expected, "{} with {bytes:?}", table.binary);
        if let Front::Run { command: reached, rest } = actual {
            let start = arguments.len().checked_sub(rest.len()).expect("rest fits within the arguments");
            assert!(
                std::ptr::eq(rest, arguments.get(start..).expect("rest is a suffix")),
                "rest borrows the input suffix"
            );
            assert!(
                table.commands.iter().any(|row| std::ptr::eq(row, reached)),
                "the command is one of the table's rows"
            );
        }
    }
}

#[test]
fn refusal_gives_the_reason_then_the_commands_usage() {
    let bare = command(&FRONT_BARE, Reach::Words(&[]));
    assert_eq!(
        FRONT_BARE.refusal(bare, "OUT is not given"),
        "demo: OUT is not given\nusage:\n  demo OUT\n",
        "bare refusal"
    );
    let list = command(&FRONT_WORDS, Reach::Words(&["internal", "list"]));
    let page = FRONT_WORDS.command_help(list);
    let (_, usage_and_page) = page.split_once("usage:\n").expect("the command's page has usage");
    let (forms, _) = usage_and_page.split_once("\n\n").expect("the command's line follows usage");
    assert_eq!(
        FRONT_WORDS.refusal(list, "OUT is not given"),
        format!("demo-engine internal list: OUT is not given\nusage:\n{forms}\n"),
        "word commands use every word and the same usage as their page, without the internal warning"
    );
}
