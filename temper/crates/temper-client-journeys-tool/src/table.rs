//! The journeys tool's forms and environment (temper/docs/design/testing/tools.md, 4).
use skein_shell::{Argument, Command, Fallback, Reach, Set, Setting, Settings, Table, Variable, Way};

pub(crate) const PATH: Variable = Variable {
    name: "PATH",
    set: Set::Public,
    form: "PATHS",
    line: "Directories searched for the build and child programs.",
    absent: "Child programs cannot be found by the recording harness.",
    notes: "The recording harness searches absolute directories only.",
    way: Way::Reads,
};
pub(crate) const TARGET: Variable = Variable {
    name: "CARGO_TARGET_DIR",
    set: Set::Public,
    form: "DIR",
    line: "Cargo's target directory, holding the built binaries to copy.",
    absent: "Use the workspace's target directory.",
    notes: "A relative path is relative to temper's root.",
    way: Way::Reads,
};
pub(crate) const CHROMIUM: Variable = Variable {
    name: "SKEIN_TEST_CHROMIUM",
    set: Set::Test,
    form: "FILE",
    line: "The Chromium binary used to record a journey again.",
    absent: "Recording again is refused before building or starting children.",
    notes: "",
    way: Way::Reads,
};
const BUNDLE: Variable = Variable {
    name: "TEMPER_INTERNAL_BUNDLE",
    set: Set::Internal,
    form: "none",
    line: "Leave the browser bundle out of the child Cargo build.",
    absent: "The tool always sets none for its builds and builds the bundle separately.",
    notes: "",
    way: Way::Sets,
};

pub(crate) const CAUTION: &str = "These aren't all a secret could be: any other private text stands unless --redact names it, so read the pages before you commit them.";
const ADD_NOTES: &str = concat!(
    "add takes out the link secret, stored credentials, typed secrets and each --redact TEXT, replacing their bytes with x's. ",
    "It refuses surviving originals and shapes of a key, JWT, bearer, code or refresh token in records, drawn pages or printed pages. ",
    "It proves the redacted replay before writing.\n",
    "A secret split across run deliveries or writing-call input is refused; --redact cannot replace it, so walk it again. ",
    "Text split across other records is seen only where a page draws it whole. Partial secrets and other encodings may remain unseen. ",
    "Values the client reshapes can stale the replay. Reload a settled page: stored requests in flight refuse. ",
    "--redact TEXT remains in shell history.\n",
    "These aren't all a secret could be: any other private text stands unless --redact names it, so read the pages before you commit them.\n",
    "FILE keeps every original: keep it out of the repository and remove it once added."
);
const REDACTION: Settings = Settings {
    tool: "temper-client-journeys",
    about: "Text to redact beside the secrets the recorder identifies.",
    table: &[Setting {
        name: "redact",
        value: "TEXT",
        about: "Replace these bytes with x's; repeats for each text.",
        fallback: Fallback::Text(""),
        set: Set::Internal,
        twin: None,
        notes: "Repeats. With no --redact, only the recorder's identified secrets are replaced.",
    }],
};
pub(crate) const REPRINT: Command = Command {
    reach: Reach::Words(&[]),
    set: Set::Internal,
    forms: &[""],
    line: "Reprint every browser and TUI journey from its recording.",
    arguments: &[],
    settings: &[],
    switches: &[],
    variables: &[],
    notes: "A stale journey is named and left as it was; after printing the others, exit 1.",
};
pub(crate) const ADD: Command = Command {
    reach: Reach::Words(&["add"]),
    forms: &["[--redact TEXT]... NAME FILE"],
    line: "Redact, prove, scan and print a saved recording as NAME.",
    arguments: &[
        Argument { shown: "NAME", line: "The journey's name, [a-z][a-z0-9-]*, unique across both clients." },
        Argument { shown: "FILE", line: "The saved recording; replaced in memory before anything is written." },
    ],
    settings: &[&REDACTION],
    notes: ADD_NOTES,
    ..REPRINT
};
pub(crate) const AGAIN: Command = Command {
    reach: Reach::Words(&["again"]),
    forms: &["NAME"],
    line: "Record journey NAME again while its acts hold, then add it.",
    arguments: &[Argument { shown: "NAME", line: "The journey to record again." }],
    variables: &[&CHROMIUM, &BUNDLE, &PATH, &TARGET],
    notes: "Unwalkable journeys and invalid setups are refused before any build or child starts.",
    ..REPRINT
};

/// The internal journeys binary's table (temper/docs/design/testing/tools.md, 4.1).
pub const TABLE: Table = Table {
    binary: "temper-client-journeys",
    line: "recorded walks, replayed through the shipped hosts and printed.",
    version: "",
    commit: "",
    commands: &[REPRINT, ADD, AGAIN],
    settings: &[],
    variables: &[&CHROMIUM, &BUNDLE, &PATH, &TARGET],
    more: "temper-client-journeys COMMAND --help prints a command; read temper/docs/design/client/web/journeys.md, section 5.1.",
};
