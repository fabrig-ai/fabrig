//! Demo tables shared by the render tests (runtime/shell.md, 14).

use super::{Argument, Command, Reach, Set, Switch, Table, TestTable, Variable, Way};
use crate::{Base, Fallback, Setting, Settings};

pub(crate) const DIRECTORIES: Settings = Settings {
    tool: "demo",
    about: "The deployment's three directories.",
    table: &[
        Setting {
            name: "config",
            value: "DIR",
            about: "Configuration and charters.",
            fallback: Fallback::Base(Base::Config),
            set: Set::Public,
            twin: Some("DEMO_CONFIG"),
            notes: "Keep the deployment's configuration here.",
        },
        Setting {
            name: "data",
            value: "DIR",
            about: "Stored deployment data.",
            fallback: Fallback::Base(Base::Data),
            set: Set::Public,
            twin: Some("DEMO_DATA"),
            notes: "",
        },
        Setting {
            name: "cache",
            value: "DIR",
            about: "Files that can be fetched again.",
            fallback: Fallback::Base(Base::Cache),
            set: Set::Public,
            twin: Some("DEMO_CACHE"),
            notes: "",
        },
    ],
};
pub(crate) const ECHO: Settings = Settings {
    tool: "demo",
    about: "How the box shows its steps.",
    table: &[Setting {
        name: "echo",
        value: "none|steps",
        about: "Show errors, or every step's lines too.",
        fallback: Fallback::Text("none"),
        set: Set::Public,
        twin: None,
        notes: "Choose for this invocation.",
    }],
};

const JOB: Variable = Variable {
    name: "DEMO_JOB",
    set: Set::Public,
    form: "NAME",
    line: "The job name passed to programs the deployment runs.",
    absent: "No job name is passed.",
    notes: "Set for each job's programs.",
    way: Way::Sets,
};
const INTERNAL_DUMP: Variable = Variable {
    name: "DEMO_INTERNAL_DUMP",
    set: Set::Internal,
    form: "1",
    line: "Dump each model request and reply to standard error.",
    absent: "Nothing is dumped.",
    notes: "For a developer.",
    way: Way::Reads,
};
const SERVE: Command = Command {
    reach: Reach::Words(&["serve"]),
    set: Set::Public,
    forms: &["[DIRECTORIES]"],
    line: "Run a deployment in one process, until stopped.",
    arguments: &[],
    settings: &[],
    switches: &[],
    variables: &[&JOB],
    notes: "Stop with a termination signal.",
};
pub(crate) const DEMO: Table = Table {
    binary: "demo-box",
    line: "demo on this machine, as one box.",
    version: "0.1.0",
    commit: "0123456789ab",
    commands: &[
        Command {
            reach: Reach::Words(&[]),
            set: Set::Public,
            forms: &["[--config DIR] [--data DIR] [--cache DIR] [--echo none|steps]"],
            line: "Run the box's steps.",
            arguments: &[],
            settings: &[&ECHO],
            switches: &[Switch {
                name: "front",
                set: Set::Internal,
                line: "Open the box's front end.",
                notes: "Used by the box's launcher.",
            }],
            variables: &[],
            notes: "Uses the deployment's directories.",
        },
        Command {
            reach: Reach::Words(&["init"]),
            forms: &["[DIRECTORIES]"],
            line: "Write a deployment that starts with demo's defaults.",
            variables: &[],
            notes: "Creates the initial configuration.",
            ..SERVE
        },
        SERVE,
        Command {
            reach: Reach::Words(&["program-login"]),
            forms: &["[DIRECTORIES] NAME"],
            line: "Sign a program source in on this host.",
            arguments: &[Argument { shown: "NAME", line: "The program source to sign in." }],
            variables: &[],
            notes: "The source must be configured.",
            ..SERVE
        },
        Command {
            reach: Reach::Words(&["help"]),
            forms: &["[NAME | pages | --search WORDS...]"],
            line: "Print this, a page, the pages, or a search.",
            variables: &[],
            notes: "",
            ..SERVE
        },
        Command {
            reach: Reach::Words(&["forge"]),
            set: Set::Internal,
            forms: &["COMMAND..."],
            line: "Manage the deployment's forge.",
            variables: &[],
            notes: "For the deployment's own processes and developers.",
            ..SERVE
        },
        Command {
            reach: Reach::RunAs("git"),
            set: Set::Internal,
            forms: &["ARGUMENT..."],
            line: "Run the delegated git grammar.",
            variables: &[],
            notes: "Arguments and exit codes follow git's grammar.",
            ..SERVE
        },
    ],
    settings: &[&DIRECTORIES],
    variables: &[&INTERNAL_DUMP],
    more: "demo-box COMMAND --help prints one command; demo-box help pages lists the pages.",
};
pub(crate) const DEMO_TESTS: TestTable = TestTable {
    tree: "demo",
    variables: &[
        Variable {
            name: "DEMO_TEST_RATE",
            set: Set::Test,
            form: "N",
            line: "The test world's iteration rate.",
            absent: "Use the test's default rate.",
            notes: "Use a small rate to replay a failure.",
            way: Way::Reads,
        },
        Variable {
            name: "DEMO_TEST_WRAPPER",
            set: Set::Test,
            form: "PROGRAM",
            line: "Wrap programs started by the test harness.",
            absent: "Run programs directly.",
            notes: "",
            way: Way::Reads,
        },
    ],
};
pub(crate) const OTHER: Table = Table {
    binary: "demo-engine",
    line: "demo's deployment engine.",
    commands: &[Command { variables: &[], ..SERVE }],
    variables: &[],
    more: "demo-engine serve --help prints the command.",
    ..DEMO
};

pub(crate) const FRONT_BARE: Table =
    Table { binary: "demo", commands: &[Command { reach: Reach::Words(&[]), forms: &["OUT"], ..SERVE }], ..OTHER };
pub(crate) const FRONT_WORDS: Table = Table {
    commands: &[
        Command { reach: Reach::Words(&["internal"]), set: Set::Internal, ..SERVE },
        Command {
            reach: Reach::Words(&["internal", "list"]),
            set: Set::Internal,
            forms: &["OUT", "--check OUT"],
            ..SERVE
        },
    ],
    ..OTHER
};
pub(crate) const FRONT_RUN_AS: Table = Table { commands: &[Command { reach: Reach::RunAs("git"), ..SERVE }], ..OTHER };

// Compile the shared fixtures together.
const _: (Table, TestTable, Table) = (DEMO, DEMO_TESTS, OTHER);
