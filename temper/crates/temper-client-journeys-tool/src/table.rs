//! The journeys tool's forms and environment (temper/docs/design/testing/tools.md, 4).
use skein_shell::{Argument, Command, Fallback, Reach, Set, Setting, Settings, Switch, Table, Variable, Way};

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

const RECORD: Variable = Variable {
    way: Way::Sets,
    absent: "The tool always sets 1 for the TUI and its box when recording again.",
    ..temper_shell::environment::INTERNAL_RECORD
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
    variables: &[&CHROMIUM, &BUNDLE, &RECORD, &PATH, &TARGET],
    notes: "Unwalkable journeys and invalid setups are refused before any build or child starts.",
    ..REPRINT
};

const SESSION: Settings = Settings {
    tool: "temper-client-journeys drive",
    about: "The given build and private scratch directory; nothing is built.",
    table: &[
        Setting {
            name: "bin",
            value: "DIR",
            about: "Directory holding executable temper-box, fake-claude and temper-client.",
            fallback: Fallback::Text(""),
            set: Set::Internal,
            twin: None,
            notes: "Required.",
        },
        Setting {
            name: "bundle",
            value: "DIR",
            about: "The loopback browser bundle, holding index.html.",
            fallback: Fallback::Text(""),
            set: Set::Internal,
            twin: None,
            notes: "Required.",
        },
        Setting {
            name: "scratch",
            value: "DIR",
            about: "Existing, owned private directory for every session write.",
            fallback: Fallback::Text(""),
            set: Set::Internal,
            twin: None,
            notes: "Required. Only session/ is made afresh; config/, data/ and cache/ stay.",
        },
        Setting {
            name: "scripts",
            value: "FILE",
            about: "A fake model script given to the box; repeats in order.",
            fallback: Fallback::Text(""),
            set: Set::Internal,
            twin: None,
            notes: "Repeats.",
        },
        Setting {
            name: "ports",
            value: "LIST",
            about: "Nine distinct nonzero ports, joined by commas, for a new sandbox.",
            fallback: Fallback::Text(""),
            set: Set::Internal,
            twin: None,
            notes: "Given only to the box as TEMPER_INTERNAL_FAKE_BOX_PORTS; a retained sandbox keeps its own ports.",
        },
    ],
};
pub(crate) const DRIVE: Command = Command {
    reach: Reach::Words(&["drive"]),
    forms: &["--bin DIR --bundle DIR --scratch DIR [--scripts FILE]... [--strict] [--ports LIST]"],
    line: "Drive a signed-in fake box from a given build, one stdin line at a time.",
    settings: &[&SESSION],
    switches: &[Switch {
        name: "strict",
        set: Set::Internal,
        line: "Give --strict to the box's fake model.",
        notes: "",
    }],
    variables: &[&CHROMIUM, &PATH],
    notes: "Answers ready, then each web, tui, ctl or forge line with shown lines prefixed | and ok or failed. forge commit FORGE REPOSITORY MESSAGE PATH CONTENT [PATH CONTENT]... makes one commit; forge checks FORGE REPOSITORY waits on its checks. end or stdin's end stops the box. Exit 0: ended; 3: store refused; 4: box failed; 1: session failure. See temper/docs/design/client/drive.md.",
    ..REPRINT
};

const RECIPES: Settings = Settings {
    tool: "temper-client-journeys recipe",
    about: "The recipes' directory.",
    table: &[Setting {
        name: "setups",
        value: "DIR",
        about: "Directory of named recipe directories.",
        fallback: Fallback::Text(crate::recipe::SETUPS),
        set: Set::Internal,
        twin: None,
        notes: "Relative to the working directory.",
    }],
};
const RECIPE_SESSION: Settings = Settings {
    tool: "temper-client-journeys recipe walk",
    about: "A given build and scratch directory, fresh or restored from an ancestor.",
    table: &[
        SESSION.table[0],
        SESSION.table[1],
        SESSION.table[2],
        SESSION.table[4],
        Setting {
            name: "from",
            value: "ANCESTOR",
            about: "Start with this member's check on its restored box.",
            fallback: Fallback::Text(""),
            set: Set::Internal,
            twin: None,
            notes: "With --from, config/ and data/ must be directories; without it, both must be absent.",
        },
    ],
};
pub(crate) const RECIPE_HASH: Command = Command {
    reach: Reach::Words(&["recipe", "hash"]),
    forms: &["NAME [--setups DIR]"],
    line: "Read a recipe's chain and print its SHA-256 hash.",
    arguments: &[Argument { shown: "NAME", line: "Recipe name, 1–64 bytes of [a-z][a-z0-9-]*." }],
    settings: &[&RECIPES],
    notes: "Refusals name the recipe and its line or file, exit 1.",
    ..REPRINT
};
pub(crate) const RECIPE_LIST: Command = Command {
    reach: Reach::Words(&["recipe", "list"]),
    forms: &["[--setups DIR]"],
    line: "List recipes, parents and each README's first sentence, in name order.",
    settings: &[&RECIPES],
    notes: "An unreadable recipe is listed as refused; the catalogue goes on.",
    ..REPRINT
};
pub(crate) const RECIPE_WALK: Command = Command {
    reach: Reach::Words(&["recipe", "walk"]),
    forms: &["NAME --bin DIR --bundle DIR --scratch DIR [--from ANCESTOR] [--strict] [--ports LIST] [--setups DIR]"],
    line: "Walk a recipe's chain through one drive session, stopping at its first failed line.",
    arguments: RECIPE_HASH.arguments,
    settings: &[&RECIPES, &RECIPE_SESSION],
    switches: DRIVE.switches,
    variables: DRIVE.variables,
    notes: "Chain scripts go to the box root first. Answers carry recipe and file line, or walk: for resets. Exit 0: walked NAME HASH; 5: stopped at NAME:N: WHY. Session statuses 3, 4 and 1 pass through. No build. See temper/docs/design/client/recipe.md, 5.",
    ..REPRINT
};

/// The internal journeys binary's table (temper/docs/design/testing/tools.md, 4.1).
pub const TABLE: Table = Table {
    binary: "temper-client-journeys",
    line: "recorded walks, replayed through the shipped hosts and printed.",
    version: "",
    commit: "",
    commands: &[REPRINT, ADD, AGAIN, DRIVE, RECIPE_WALK, RECIPE_HASH, RECIPE_LIST],
    settings: &[],
    variables: &[&CHROMIUM, &BUNDLE, &RECORD, &PATH, &TARGET],
    more: "temper-client-journeys COMMAND --help prints a command; read temper/docs/design/client/web/journeys.md, section 5.1.",
};
