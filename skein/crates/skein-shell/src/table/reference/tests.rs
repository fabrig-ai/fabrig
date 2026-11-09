//! Handwritten documents from skein/docs/design/shell.md, 15.3.

use crate::Table;
use crate::table::demo::{DEMO, DEMO_TESTS, OTHER};

const ARGUMENTS: &str = "\
# demo-box: arguments

Generated from demo-box's table (`skein/docs/design/shell.md`, 15.3): a test checks it byte for byte, and the tree's rewrite switch rewrites it.

## Public

### `demo-box`

- **Set:** public
- **Forms:** `[--config DIR] [--data DIR] [--cache DIR] [--echo none|steps]`
- Run the box's steps.
- **Flags:** `--config DIR`, `--data DIR`, `--cache DIR`, `--echo none|steps`
- **Notes:** Uses the deployment's directories.

### `demo-box init`

- **Set:** public
- **Forms:** `[DIRECTORIES]`
- Write a deployment that starts with demo's defaults.
- **Flags:** `--config DIR`, `--data DIR`, `--cache DIR`
- **Notes:** Creates the initial configuration.

### `demo-box serve`

- **Set:** public
- **Forms:** `[DIRECTORIES]`
- Run a deployment in one process, until stopped.
- **Flags:** `--config DIR`, `--data DIR`, `--cache DIR`
- **Notes:** Stop with a termination signal.

### `demo-box program-login`

- **Set:** public
- **Forms:** `[DIRECTORIES] NAME`
- Sign a program source in on this host.
- **Arguments:** `NAME`: The program source to sign in.
- **Flags:** `--config DIR`, `--data DIR`, `--cache DIR`
- **Notes:** The source must be configured.

### `demo-box help`

- **Set:** public
- **Forms:** `[NAME | pages | --search WORDS...]`
- Print this, a page, the pages, or a search.
- **Flags:** `--config DIR`, `--data DIR`, `--cache DIR`

### `--config DIR`

- **Set:** public
- **Default:** $XDG_CONFIG_HOME/demo
- **Twin:** `DEMO_CONFIG`
- Configuration and charters.
- **Notes:** Keep the deployment's configuration here.

### `--data DIR`

- **Set:** public
- **Default:** $XDG_DATA_HOME/demo
- **Twin:** `DEMO_DATA`
- Stored deployment data.

### `--cache DIR`

- **Set:** public
- **Default:** $XDG_CACHE_HOME/demo
- **Twin:** `DEMO_CACHE`
- Files that can be fetched again.

### `--echo none|steps`

- **Set:** public
- **Default:** none
- Show errors, or every step's lines too.
- **Notes:** Choose for this invocation.

## Internal

### `demo-box forge`

- **Set:** internal
- **Forms:** `COMMAND...`
- Manage the deployment's forge.
- **Notes:** For the deployment's own processes and developers.

### run as `git`

- **Set:** internal
- **Forms:** `ARGUMENT...`
- Run the delegated git grammar.
- **Notes:** Arguments and exit codes follow git's grammar.

### `--front`

- **Set:** internal
- Open the box's front end.
- **Notes:** Used by the box's launcher.
";

const ENVIRONMENT: &str = "\
# demo-box: environment

Generated from demo-box's table (`skein/docs/design/shell.md`, 15.3): a test checks it byte for byte, and the tree's rewrite switch rewrites it.

## Public

### `DEMO_JOB`

- **Set:** public
- **Sets**
- **Form:** NAME
- **Absent:** No job name is passed.
- The job name passed to programs the deployment runs.
- **Notes:** Set for each job's programs.

## Internal

### `DEMO_INTERNAL_DUMP`

- **Set:** internal
- **Reads**
- **Form:** 1
- **Absent:** Nothing is dumped.
- Dump each model request and reply to standard error.
- **Notes:** For a developer.
";

const COMMON: &str = "\
# demo: common environment

Generated from the tree's tables (`skein/docs/design/shell.md`, 15.3): a test checks it byte for byte, and the tree's rewrite switch rewrites it.

## Public

### `DEMO_CONFIG`

- **Set:** public
- **Reads**
- **Form:** DIR
- **Absent:** default $XDG_CONFIG_HOME/demo
- **Twin of:** `--config`
- Configuration and charters.
- **Notes:** Keep the deployment's configuration here.
- **Read by:** demo-box, demo-engine

### `DEMO_DATA`

- **Set:** public
- **Reads**
- **Form:** DIR
- **Absent:** default $XDG_DATA_HOME/demo
- **Twin of:** `--data`
- Stored deployment data.
- **Read by:** demo-box, demo-engine

### `DEMO_CACHE`

- **Set:** public
- **Reads**
- **Form:** DIR
- **Absent:** default $XDG_CACHE_HOME/demo
- **Twin of:** `--cache`
- Files that can be fetched again.
- **Read by:** demo-box, demo-engine

### `XDG_CONFIG_HOME`

- **Set:** public
- **Reads**
- **Form:** DIR
- **Absent:** Use $HOME/.config when HOME is absolute.
- Base for a setting's configuration directory, before the tool's name.
- **Notes:** Only absolute paths are used.
- **Read by:** demo-box, demo-engine

### `HOME`

- **Set:** public
- **Reads**
- **Form:** DIR
- **Absent:** A directory setting needs an absolute XDG base or an explicit value.
- Base for a setting's directory when its XDG base is unusable.
- **Notes:** Only absolute paths are used.
- **Read by:** demo-box, demo-engine

### `XDG_DATA_HOME`

- **Set:** public
- **Reads**
- **Form:** DIR
- **Absent:** Use $HOME/.local/share when HOME is absolute.
- Base for a setting's data directory, before the tool's name.
- **Notes:** Only absolute paths are used.
- **Read by:** demo-box, demo-engine

### `XDG_CACHE_HOME`

- **Set:** public
- **Reads**
- **Form:** DIR
- **Absent:** Use $HOME/.cache when HOME is absolute.
- Base for a setting's cache directory, before the tool's name.
- **Notes:** Only absolute paths are used.
- **Read by:** demo-box, demo-engine

## Internal
";

const TESTS: &str = "\
# demo: test environment

Generated from the tree's test table (`skein/docs/design/shell.md`, 15.3): a test checks it byte for byte, and the tree's rewrite switch rewrites it.

## Test

### `DEMO_TEST_RATE`

- **Set:** test
- **Reads**
- **Form:** N
- **Absent:** Use the test's default rate.
- The test world's iteration rate.
- **Notes:** Use a small rate to replay a failure.

### `DEMO_TEST_WRAPPER`

- **Set:** test
- **Reads**
- **Form:** PROGRAM
- **Absent:** Run programs directly.
- Wrap programs started by the test harness.
";

#[test]
fn the_four_documents_follow_the_reference_form_byte_for_byte() {
    let tree = &[&DEMO, &OTHER];
    for (name, got, expected) in [
        ("arguments", DEMO.arguments_document(), ARGUMENTS),
        ("environment", DEMO.environment_document(tree), ENVIRONMENT),
        ("common", Table::common_document("demo", tree), COMMON),
        ("test", DEMO_TESTS.document(), TESTS),
    ] {
        assert_eq!(got, expected, "{name}: handwritten from shell.md, 15.3");
    }
}

/// Only headings and the first Set item belong to contracts.md, 5.
fn environment_entries(document: &str) -> Vec<(&str, &str)> {
    let mut entries = Vec::new();
    let mut name = None;
    for line in document.lines() {
        if let Some(heading) = line.strip_prefix("### `").and_then(|heading| heading.strip_suffix('`')) {
            assert!(name.is_none(), "the previous variable needs its Set item");
            name = Some(heading);
        } else if let Some(set) = line.strip_prefix("- **Set:** ") {
            let name = name.take().expect("Set follows a variable heading");
            assert!(["public", "internal", "test"].contains(&set), "contract set spelling");
            entries.push((name, set));
        }
    }
    assert!(name.is_none(), "the last variable needs its Set item");
    entries
}

#[test]
fn environment_headings_and_sets_are_the_names_check_contract() {
    let documents = [
        DEMO.environment_document(&[&DEMO, &OTHER]),
        Table::common_document("demo", &[&DEMO, &OTHER]),
        DEMO_TESTS.document(),
    ];
    let entries: Vec<_> = documents.iter().flat_map(|document| environment_entries(document)).collect();
    assert_eq!(
        entries,
        [
            ("DEMO_JOB", "public"),
            ("DEMO_INTERNAL_DUMP", "internal"),
            ("DEMO_CONFIG", "public"),
            ("DEMO_DATA", "public"),
            ("DEMO_CACHE", "public"),
            ("XDG_CONFIG_HOME", "public"),
            ("HOME", "public"),
            ("XDG_DATA_HOME", "public"),
            ("XDG_CACHE_HOME", "public"),
            ("DEMO_TEST_RATE", "test"),
            ("DEMO_TEST_WRAPPER", "test"),
        ],
        "the names check needs no knowledge of the remaining Markdown"
    );
}

#[test]
fn entries_keep_order_deduplicate_and_preserve_their_own_set_and_way() {
    use crate::{Base, Command, Fallback, HOME, Reach, Set, Setting, Settings, Switch, Variable, Way};

    const GROUP: Settings = Settings {
        tool: "demo",
        about: "Mixed settings.",
        table: &[
            Setting {
                name: "required",
                value: "VALUE",
                about: "Supply a value.",
                fallback: Fallback::Refuse,
                set: Set::Internal,
                twin: Some("DEMO_REQUIRED"),
                notes: "",
            },
            Setting {
                name: "mode",
                value: "WORD",
                about: "Choose a mode.",
                fallback: Fallback::Text("quiet"),
                set: Set::Test,
                twin: Some("DEMO_MODE"),
                notes: "",
            },
            Setting {
                name: "state",
                value: "DIR",
                about: "State directory.",
                fallback: Fallback::Base(Base::State),
                set: Set::Public,
                twin: None,
                notes: "",
            },
        ],
    };
    const READ: Variable = Variable { name: "DEMO_READ", ..HOME };
    const SET: Variable = Variable { name: "DEMO_SET", way: Way::Sets, ..HOME };
    const COMMAND: Command = Command {
        reach: Reach::Words(&["nested", "command"]),
        set: Set::Test,
        forms: &["", "NAME"],
        line: "",
        arguments: &[],
        settings: &[&GROUP],
        switches: &[Switch { name: "again", set: Set::Test, line: "Again.", notes: "" }],
        variables: &[&READ, &SET],
        notes: "",
    };
    const TABLE: Table = Table { settings: &[], variables: &[&READ, &SET], commands: &[COMMAND, COMMAND], ..DEMO };
    const SECOND: Table =
        Table { binary: "demo-second", settings: &[], variables: &[&READ, &SET], commands: &[], ..TABLE };
    let environment = TABLE.environment_document(&[&TABLE, &TABLE]);
    assert_eq!(
        environment_entries(&environment),
        [
            ("DEMO_READ", "public"),
            ("DEMO_SET", "public"),
            ("XDG_STATE_HOME", "public"),
            ("HOME", "public"),
            ("DEMO_REQUIRED", "internal"),
            ("DEMO_MODE", "test"),
        ],
        "table variables precede command twins and implicit host reads; duplicate binaries are one reader"
    );
    assert!(
        environment.contains("- **Absent:** required\n- **Twin of:** `--required`\n- Supply a value.\n"),
        "a command twin inherits its setting's set, form, default and line"
    );
    assert!(
        environment.contains("- **Absent:** default quiet\n- **Twin of:** `--mode`\n"),
        "a text default uses help's words"
    );
    let arguments = TABLE.arguments_document();
    assert_eq!(arguments.matches("### `--required VALUE`").count(), 1, "repeated command flags appear once");
    assert_eq!(arguments.matches("### `--again`").count(), 1, "repeated switches appear once");
    assert!(arguments.contains("- **Required.**\n"), "a refused default is required");
    assert!(
        arguments.contains("## Test\n\n### `demo-box nested command`\n\n- **Set:** test\n- **Forms:** ``, `NAME`\n- **Flags:** `--mode WORD`, `--again`\n"),
        "test commands follow internal entries and name only flags of their own set"
    );
    assert!(arguments.contains("- **Default:** quiet\n"), "text flag defaults omit the default label");
    let solo = TABLE.environment_document(&[&TABLE, &SECOND]);
    assert!(!solo.contains("### `DEMO_READ`"), "two readers move a variable to common");
    assert!(solo.contains("### `DEMO_SET`"), "a variable set by both binaries stays with its setters");
    let common = Table::common_document("demo", &[&TABLE, &SECOND, &TABLE]);
    assert_eq!(environment_entries(&common), [("DEMO_READ", "public")], "only shared reads are common");
    assert!(common.contains("- **Read by:** demo-box, demo-second\n"), "readers retain order without duplicates");
}
