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
pub(crate) const CHROMIUM: Variable = Variable {
    name: "SKEIN_TEST_CHROMIUM",
    set: Set::Test,
    form: "FILE",
    line: "The Chromium binary used by the drive session.",
    absent: "A full drive session is refused before starting children.",
    notes: "",
    way: Way::Reads,
};
pub(crate) const CAUTION: &str = "These aren't all a secret could be: any other private text stands unless --redact names it, so read the pages before you commit them.";
const ADD_NOTES: &str = concat!(
    "add takes out the link secret, stored credentials, typed secrets and each --redact TEXT, replacing their bytes with x's. ",
    "It refuses surviving originals and shapes of a key, JWT, bearer, code or refresh token in records, drawn pages or printed pages. ",
    "It proves the redacted replay, then normalises and replays it again before scanning and writing.\n",
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
const BASE: Command = Command {
    reach: Reach::Words(&[]),
    set: Set::Internal,
    forms: &[""],
    line: "",
    arguments: &[],
    settings: &[],
    switches: &[],
    variables: &[],
    notes: "",
};
pub(crate) const NORMALISE: Command = Command {
    reach: Reach::Words(&["normalise"]),
    line: "Normalise every browser and TUI recording, then reprint both directories.",
    notes: "Write each changed recording only after its normalised replay holds, naming each written. A stale recording is named with its turn and line kind and left as it was; after writing the others and reprinting, exit 1. Reads no configuration, environment, clock or network.",
    ..BASE
};
pub(crate) const ADD: Command = Command {
    reach: Reach::Words(&["add"]),
    forms: &["[--redact TEXT]... NAME FILE"],
    line: "Redact, prove, normalise, scan and print a saved recording as NAME.",
    arguments: &[
        Argument { shown: "NAME", line: "The journey's name, [a-z][a-z0-9-]*, unique across both clients." },
        Argument { shown: "FILE", line: "The saved recording; replaced in memory before anything is written." },
    ],
    settings: &[&REDACTION],
    notes: ADD_NOTES,
    ..BASE
};
const SESSION: Settings = Settings {
    tool: "temper-client-journeys drive",
    about: "The given build and private scratch directory; nothing is built.",
    table: &[
        Setting {
            name: "bin",
            value: "DIR",
            about: "Directory holding executable temper-box, fake-claude and temper; with --deployment, also temper-engine, temper-worker and temper-agent.",
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
            notes: "Required without --deployment.",
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
            about: "A fake model script; repeats in order.",
            fallback: Fallback::Text(""),
            set: Set::Internal,
            twin: None,
            notes: "Repeats.",
        },
        Setting {
            name: "ports",
            value: "LIST",
            about: "Nine distinct nonzero ports, joined by commas.",
            fallback: Fallback::Text(""),
            set: Set::Internal,
            twin: None,
            notes: "Required with --deployment. On a box, given as TEMPER_INTERNAL_FAKE_BOX_PORTS; a retained sandbox keeps its own ports.",
        },
        Setting {
            name: "files",
            value: "DIR",
            about: "Read-only directory of the files forge commit, forge push, deployment patch and ctl settings apply/propose lines name.",
            fallback: Fallback::Text(""),
            set: Set::Internal,
            twin: None,
            notes: "Optional. Must be an existing directory.",
        },
        Setting {
            name: "deployment",
            value: "DIR",
            about: "Deployment documents, in place of a box; browser and TUI use the engine’s origin.",
            fallback: Fallback::Text(""),
            set: Set::Internal,
            twin: None,
            notes: "Needs --ports. --ctl selects a ctl session without Chromium; otherwise a full session. --bundle is optional unless a document names {place.bundle}.",
        },
        WIDTHS,
    ],
};
const WIDTHS: Setting = Setting {
    name: "widths",
    value: "LIST",
    about: "Distinct screenshot widths from 320 to 3840, joined by commas, in the given order.",
    fallback: Fallback::Text(""),
    set: Set::Internal,
    twin: None,
    notes: "Requires --journey. With none, web mark answers text alone.",
};
pub(crate) const DRIVE: Command = Command {
    reach: Reach::Words(&["drive"]),
    forms: &[
        "--bin DIR [--bundle DIR] --scratch DIR [--scripts FILE]... [--strict] [--ports LIST] [--files DIR] [--ctl] [--deployment DIR] [--journey [--widths LIST]]",
    ],
    line: "Drive a signed-in box or deployment from a given build, one stdin line at a time.",
    settings: &[&SESSION],
    switches: &[
        Switch { name: "strict", set: Set::Internal, line: "Give --strict to the session's fake model.", notes: "" },
        Switch {
            name: "ctl",
            set: Set::Internal,
            line: "Take ctl and forge lines alone, without a browser or TUI.",
            notes: "Needs no SKEIN_TEST_CHROMIUM; the box still takes its bundle.",
        },
        Switch {
            name: "journey",
            set: Set::Internal,
            line: "Take journey lines without an implied root load or TUI approval.",
            notes: "Refuses --ctl. tui approve signs the TUI in; web mark and tui mark answer looks and are not kept.",
        },
    ],
    variables: &[&CHROMIUM, &PATH],
    notes: "Answers ready, then each web, tui, ctl, forge or deployment line with shown lines prefixed | and ok or failed. forge commit forgejo REPOSITORY MESSAGE PATH [PATH]... commits files from --files DIR; forge checks forgejo REPOSITORY waits on its checks. deployment patch DOCUMENT PATH merges a JSON object into engine or worker, then restarts the deployment; deployment restart keeps the documents unchanged. A box refuses deployment lines. ctl settings apply/propose resolve input files from --files DIR before ctl starts. ctl fields {forgejo.api}, {forgejo.git}, {forgejo.admin}, {git} and {model.key} read the box’s values when run, or the deployment’s rendered documents. end or stdin's end stops the session. Exit 0: ended; 3: store refused; 4: box or deployment failed; 1: session failure. See temper/docs/design/client/drive.md.",
    ..BASE
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
    ..BASE
};
pub(crate) const RECIPE_LIST: Command = Command {
    reach: Reach::Words(&["recipe", "list"]),
    forms: &["[--setups DIR]"],
    line: "List recipes, parents and each README's first sentence, in name order.",
    settings: &[&RECIPES],
    notes: "An unreadable recipe is listed as refused; the catalogue goes on.",
    ..BASE
};
pub(crate) const RECIPE_WALK: Command = Command {
    reach: Reach::Words(&["recipe", "walk"]),
    forms: &["NAME --bin DIR [--bundle DIR] --scratch DIR [--from ANCESTOR] [--strict] [--ports LIST] [--setups DIR]"],
    line: "Walk a recipe's chain through one drive session, stopping at its first failed line.",
    arguments: RECIPE_HASH.arguments,
    settings: &[&RECIPES, &RECIPE_SESSION],
    switches: &[DRIVE.switches[0]],
    variables: DRIVE.variables,
    notes: "A box needs --bundle. A deployment uses its root’s config/ and needs --ports; with web or tui lines it also needs --bundle and runs a full session with resets. With none it runs --ctl --deployment without resets, passing --bundle where given. Chain scripts go to the session root first. Answers carry recipe and file line, or walk: for resets. Exit 0: walked NAME HASH; 5: stopped at NAME:N: WHY. Session statuses 3, 4 and 1 pass through. No build. See temper/docs/design/client/recipe.md, 5.",
    ..BASE
};

const JOURNEY_SESSION: Settings = Settings {
    tool: "temper-client-journeys journey walk",
    about: "One journey session on a fresh scratch directory; nothing is built.",
    table: &[
        SESSION.table[0],
        SESSION.table[1],
        SESSION.table[2],
        SESSION.table[4],
        SESSION.table[6],
        Setting { notes: "With none, web mark answers text alone.", ..WIDTHS },
    ],
};
pub(crate) const JOURNEY_WALK: Command = Command {
    reach: Reach::Words(&["journey", "walk"]),
    forms: &[
        "NAME --bin DIR [--bundle DIR] --scratch DIR [--strict] [--ports LIST] [--widths LIST] [--deployment DIR]",
    ],
    line: "Walk a journey's chain through one journey session, stopping at its first failed line.",
    arguments: &[Argument {
        shown: "NAME",
        line: "Journey name, [a-z][a-z0-9-]*, under the checkout's web or TUI roots.",
    }],
    settings: &[&JOURNEY_SESSION],
    switches: &[DRIVE.switches[0]],
    variables: DRIVE.variables,
    notes: "Refuses config/ or data/ in scratch. Chain scripts and lines go root first, without ancestors' marks or added resets. Answers carry journey and file line. Exit 0: walked NAME; 5: stopped at NAME:N: WHY. Session statuses 3, 4 and 1 pass through. No build.",
    ..BASE
};

/// The internal journeys binary's table (temper/docs/design/testing/tools.md, 4.1).
pub const TABLE: Table = Table {
    binary: "temper-client-journeys",
    line: "recorded walks, replayed through the shipped hosts and printed.",
    version: "",
    commit: "",
    commands: &[ADD, NORMALISE, DRIVE, JOURNEY_WALK, RECIPE_WALK, RECIPE_HASH, RECIPE_LIST],
    settings: &[],
    variables: &[&CHROMIUM, &PATH],
    more: "temper-client-journeys COMMAND --help prints a command; read temper/docs/design/client/web/journeys.md, section 5.1.",
};
