//! Help's text and layout rules (skein/docs/design/runtime/shell.md, 14.2).

use super::{one_column, two_columns};
use crate::table::{Command, Reach, Set, Switch, Table, Variable, Way, demo::DEMO};
use crate::{Fallback, Setting, Settings};

fn assert_page(actual: &str, expected: &str) {
    assert_eq!(actual, expected, "the page follows runtime/shell.md, 14.2");
    for text in [actual, expected] {
        for line in text.lines() {
            assert!(line.chars().count() <= 80, "line exceeds 80 columns: {line:?}");
        }
    }
}

#[test]
fn layout_wraps_at_spaces_and_keeps_long_words_whole() {
    for (first, second, expected) in [
        ("123456789012345678901234567890123", "fits", "  123456789012345678901234567890123  fits\n"),
        (
            "1234567890123456789012345678901234",
            "next row",
            "  1234567890123456789012345678901234\n                                     next row\n",
        ),
        (
            "short",
            "12345678901234567890 1234567890123456789012 end",
            "  short                              12345678901234567890 1234567890123456789012\n                                     end\n",
        ),
        (
            "short",
            "1234567890123456789012345678901234567890123 next 12345678901234567890123456789012345678 end",
            "  short                              1234567890123456789012345678901234567890123\n                                     next 12345678901234567890123456789012345678\n                                     end\n",
        ),
        (
            "short",
            "before abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMN after",
            "  short                              before\n                                     abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMN\n                                     after\n",
        ),
        ("ééééééééééééééééééééééééééééééééé", "chars", "  ééééééééééééééééééééééééééééééééé  chars\n"),
    ] {
        let mut actual = String::new();
        two_columns(&mut actual, first, second);
        assert_page(&actual, expected);
    }
    let mut actual = String::new();
    one_column(&mut actual, "1234567890123456789012345678901234567890 1234567890123456789012345678901234567 end", 2);
    assert_page(&actual, "  1234567890123456789012345678901234567890 1234567890123456789012345678901234567\n  end\n");
}

const HOST_ENVIRONMENT: &str = concat!(
    "  XDG_CONFIG_HOME                    Base for a setting's configuration\n",
    "                                     directory, before the tool's name.\n",
    "  HOME                               Base for a setting's directory when its XDG\n",
    "                                     base is unusable.\n",
    "  XDG_DATA_HOME                      Base for a setting's data directory, before\n",
    "                                     the tool's name.\n",
    "  XDG_CACHE_HOME                     Base for a setting's cache directory,\n",
    "                                     before the tool's name.\n",
);
const TWINS: &str = concat!(
    "  DEMO_CONFIG, DEMO_DATA, DEMO_CACHE\n",
    "                                     As --config, --data and --cache.\n",
);

const DIRECTORY_FLAGS: &str = concat!(
    "flags:\n",
    "  --config DIR                       Configuration and charters. Default:\n",
    "                                     $XDG_CONFIG_HOME/demo. Environment:\n",
    "                                     DEMO_CONFIG.\n",
    "  --data DIR                         Stored deployment data. Default:\n",
    "                                     $XDG_DATA_HOME/demo. Environment:\n",
    "                                     DEMO_DATA.\n",
    "  --cache DIR                        Files that can be fetched again. Default:\n",
    "                                     $XDG_CACHE_HOME/demo. Environment:\n",
    "                                     DEMO_CACHE.\n",
);

#[test]
fn pages_list_their_entries_in_the_shared_format() {
    let environment = format!("environment:\n{TWINS}{HOST_ENVIRONMENT}");
    let internal_environment = format!(
        "environment:\n{TWINS}  DEMO_INTERNAL_DUMP                 Dump each model request and reply to\n                                     standard error.\n{HOST_ENVIRONMENT}"
    );
    let expected = concat!(
        "demo-box: demo on this machine, as one box.\n",
        "\n",
        "usage:\n",
        "  demo-box [--config DIR] [--data DIR] [--cache DIR] [--echo none|steps]\n",
        "  demo-box COMMAND [ARGUMENT...]\n",
        "\n",
        "commands:\n",
        "  init [DIRECTORIES]                 Write a deployment that starts with demo's\n",
        "                                     defaults.\n",
        "  serve [DIRECTORIES]                Run a deployment in one process, until\n",
        "                                     stopped.\n",
        "  program-login [DIRECTORIES] NAME   Sign a program source in on this host.\n",
        "  help [NAME | pages | --search WORDS...]\n",
        "                                     Print this, a page, the pages, or a search.\n",
        "\n",
        "flags:\n",
        "  --config DIR, --data DIR, --cache DIR\n",
        "                                     The deployment's three directories.\n",
        "  -h, --help                         Print this.\n",
        "  --version                          Print the name, version and commit.\n",
        "\n",
    );
    let expected = format!(
        "{expected}{environment}\ndemo-box COMMAND --help prints one command; demo-box help pages lists the pages.\n"
    );
    assert_page(&DEMO.help(), &expected);
    assert_page(&DEMO.version_line(), "demo-box 0.1.0 (0123456789ab)\n");

    command_pages(&environment, &internal_environment);
    other_pages();
}

fn command_pages(environment: &str, internal_environment: &str) {
    for (reach, before_flags, after_flags) in [
        (
            Reach::Words(&["program-login"]),
            concat!(
                "usage:\n",
                "  demo-box program-login [DIRECTORIES] NAME\n\n",
                "Sign a program source in on this host.\n\n",
                "arguments:\n",
                "  NAME                               The program source to sign in.\n\n",
            ),
            format!("\n{environment}\nThe source must be configured.\n"),
        ),
        (
            Reach::Words(&[]),
            concat!(
                "usage:\n",
                "  demo-box [--config DIR] [--data DIR] [--cache DIR] [--echo none|steps]\n\n",
                "Run the box's steps.\n\n",
            ),
            format!(
                "  --echo none|steps                  Show errors, or every step's lines too.\n                                     Default: none.\n\n{environment}\nUses the deployment's directories.\n"
            ),
        ),
        (
            Reach::Words(&["forge"]),
            concat!(
                "internal: not part of demo-box's public command line; it may change without\n",
                "notice.\n\n",
                "usage:\n",
                "  demo-box forge COMMAND...\n\n",
                "Manage the deployment's forge.\n\n",
            ),
            format!("\n{internal_environment}\nFor the deployment's own processes and developers.\n"),
        ),
        (
            Reach::RunAs("git"),
            concat!(
                "internal: not part of demo-box's public command line; it may change without\n",
                "notice.\n\n",
                "usage:\n",
                "  git ARGUMENT...\n\n",
                "Run the delegated git grammar.\n\n",
            ),
            format!("\n{internal_environment}\nArguments and exit codes follow git's grammar.\n"),
        ),
    ] {
        let command = DEMO.commands.iter().find(|command| command.reach == reach).expect("demo command");
        assert_page(&DEMO.command_help(command), &format!("{before_flags}{DIRECTORY_FLAGS}{after_flags}"));
    }
}

const FIRST_SETTING: Setting = Setting {
    name: "one",
    value: "VALUE",
    about: "Choose one.",
    fallback: Fallback::Refuse,
    set: Set::Public,
    twin: Some("DEMO_ONE"),
    notes: "",
};
const GROUP: Settings = Settings {
    tool: "demo",
    about: "Choose values.",
    table: &[
        FIRST_SETTING,
        Setting { name: "two", twin: Some("DEMO_TWO"), fallback: Fallback::Text("two"), ..FIRST_SETTING },
        Setting { name: "hidden", twin: Some("DEMO_HIDDEN"), set: Set::Internal, ..FIRST_SETTING },
        Setting { name: "test", twin: Some("DEMO_TEST"), set: Set::Test, ..FIRST_SETTING },
    ],
};
const VARIABLE: Variable = Variable {
    name: "DEMO_READ",
    set: Set::Public,
    form: "VALUE",
    line: "Read a value.",
    absent: "",
    notes: "",
    way: Way::Reads,
};
const COMMAND: Command = Command {
    reach: Reach::Words(&["do", "work"]),
    set: Set::Public,
    forms: &["ONE", "TWO"],
    line: "Do work.",
    arguments: &[],
    settings: &[&GROUP],
    switches: &[
        Switch { name: "public", set: Set::Public, line: "Public switch.", notes: "" },
        Switch { name: "hidden", set: Set::Internal, line: "Internal switch.", notes: "" },
        Switch { name: "test", set: Set::Test, line: "Test switch.", notes: "" },
    ],
    variables: &[
        &VARIABLE,
        &Variable { set: Set::Internal, name: "DEMO_PRIVATE", ..VARIABLE },
        &Variable { set: Set::Test, name: "DEMO_TEST_READ", ..VARIABLE },
        &Variable { way: Way::Sets, name: "DEMO_SET", ..VARIABLE },
    ],
    notes: "",
};

const ONE_GROUP: Settings = Settings { table: &[FIRST_SETTING], ..GROUP };

fn other_pages() {
    let empty = Table { binary: "demo", line: "", commands: &[], settings: &[], variables: &[], more: "", ..DEMO };
    assert_page(
        &empty.help(),
        "demo:\n\nflags:\n  -h, --help                         Print this.\n  --version                          Print the name, version and commit.\n",
    );
    let no_parts = Command { forms: &[], line: "", settings: &[], switches: &[], variables: &[], ..COMMAND };
    assert_page(&empty.command_help(&no_parts), "");

    let table = Table { commands: &[COMMAND], variables: &[&VARIABLE, &VARIABLE], ..empty };
    assert_page(
        &table.help(),
        concat!(
            "demo:\n\nusage:\n  demo COMMAND [ARGUMENT...]\n\ncommands:\n",
            "  do work ONE                        Do work.\n  do work TWO\n\n",
            "flags:\n  -h, --help                         Print this.\n",
            "  --version                          Print the name, version and commit.\n\n",
            "environment:\n  DEMO_ONE, DEMO_TWO                 As --one and --two.\n",
            "  DEMO_READ                          Read a value.\n",
        ),
    );
    assert_page(
        &table.command_help(&COMMAND),
        concat!(
            "usage:\n  demo do work ONE\n  demo do work TWO\n\nDo work.\n\nflags:\n",
            "  --one VALUE                        Choose one. Required. Environment:\n",
            "                                     DEMO_ONE.\n",
            "  --two VALUE                        Choose one. Default: two. Environment:\n",
            "                                     DEMO_TWO.\n",
            "  --public                           Public switch.\n\n",
            "environment:\n  DEMO_ONE, DEMO_TWO                 As --one and --two.\n",
            "  DEMO_READ                          Read a value.\n",
        ),
    );
    let table = Table { settings: &[&ONE_GROUP, &ONE_GROUP], commands: &[], variables: &[], ..empty };
    // Repeated groups contribute their environment twins only once.
    assert_page(&table.help_environment(None), "  DEMO_ONE                           As --one.\n");
    let internal = Command { set: Set::Internal, forms: &[], line: "", settings: &[], variables: &[], ..COMMAND };
    assert_page(
        &empty.command_help(&internal),
        concat!(
            "internal: not part of demo's public command line; it may change without notice.\n\n",
            "flags:\n  --public                           Public switch.\n",
            "  --hidden                           Internal switch.\n",
            "  --test                             Test switch.\n",
        ),
    );
}
