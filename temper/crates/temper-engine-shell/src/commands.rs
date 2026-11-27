//! The engine's table and the box's shared forge commands (temper/docs/design/cli/README.md, 7).
use skein_shell::{Argument, Command, Reach, Set, Switch, Table};
use temper_shell::{commands::Row, directories, environment, table::HELP};

pub const ENGINE: Table = Table {
    binary: "temper-engine",
    line: "the deployment's engine, with the owned forge's commands.",
    // The identity is main's (temper/docs/design/cli/README.md, 3).
    version: "",
    commit: "",
    commands: &[
        SERVE,
        VERSIONS,
        UPGRADE_CHECK,
        UPGRADE_WORKERS,
        UPGRADE_BACK,
        CHECKPOINT,
        RESTORE,
        HELP,
        FETCH,
        PREPARE,
        RUN,
        SIGN_IN,
        SOURCE,
        REMOVE,
        FORGE_CHILD,
    ],
    settings: &[&directories::SETTINGS],
    variables: &[
        &temper_service_shell::WAIT_CHECK,
        &temper_service_shell::CUT_AT_LISTEN,
        &super::forge_sign_in::NO_REGISTRATION,
        &temper_shell::environment::INTERNAL_COMMIT,
        &temper_client_bundle::INTERNAL_BUNDLE,
    ],
    more: "temper-engine COMMAND --help prints one command; temper-engine help pages lists the pages.",
};

const DIRECTORIES: Argument =
    Argument { shown: temper_shell::commands::DIRECTORIES.shown, line: temper_shell::commands::DIRECTORIES.line };
const FORGE: Argument = Argument { shown: super::forge::FORGE.shown, line: super::forge::FORGE.line };

pub const RECORD: Switch = Switch {
    name: "record",
    set: Set::Public,
    line: "Have every page served record a journey in its browser, saved by Alt+Shift+S.",
    notes: "",
};

pub const SERVE: Command = Command {
    reach: Reach::Words(&["serve"]),
    set: Set::Public,
    forms: &["[DIRECTORIES] [--record]"],
    line: "Run an engine alone's document until stopped; temper-worker serve runs a worker alone, and temper-box serve runs an engine and its worker in one process.",
    arguments: &[DIRECTORIES],
    settings: &[&directories::SETTINGS],
    switches: &[RECORD],
    variables: &[&environment::PATH, &environment::TERM, &environment::LISTEN_FDS, &environment::XDG_RUNTIME_DIR],
    notes: "Its stdout carries only sign-in links, one a line; its stderr, its startup notes and the reason of a failure. A first termination signal settles the engine's forge writes and drains it; a second aborts. Exit 0 after a drained stop, 1 on a refused start or any failure. Refuses a worker alone's document, naming temper-worker serve, or an engine and its worker in one process, naming temper-box serve.",
};

pub const VERSIONS: Command = Command {
    reach: Reach::Words(&["versions"]),
    set: Set::Public,
    forms: &[""],
    line: "Print levels, store and family versions, wire fingerprints, the host tools' contract floor and version, and bounds.",
    arguments: &[],
    settings: &[],
    switches: &[],
    variables: &[],
    notes: "Each entry takes one line. Reads no configuration and no store.",
};

pub const UPGRADE_CHECK: Command = Command {
    reach: Reach::Words(&["upgrade", "--check"]),
    set: Set::Public,
    forms: &["[DIRECTORIES] DIRECTORY"],
    line: "Rehearses an upgrade over a copy; switches nothing.",
    arguments: &[
        DIRECTORIES,
        Argument {
            shown: "DIRECTORY",
            line: "A stopped deployment's data directory, a checkpoint of a serving deployment, or a copy of one; never written.",
        },
    ],
    settings: &[&directories::SETTINGS],
    switches: &[],
    variables: &[],
    notes: "Uses the configuration serve would read and a scratch copy under the cache directory; removes a rehearsal's scratch left behind there. Reports the jobs it ran, restores, elapsed time and whether workers may stay. Names a start it refuses, including each setting out of its numbers. What follows the switch is not rehearsed. Exit 0 when its jobs and restores pass, 1 when a start would be refused.",
};
pub const UPGRADE_WORKERS: Command = Command {
    reach: Reach::Words(&["upgrade", "--workers"]),
    line: "Judges the workers alone over links and small copies; switches nothing.",
    notes: "Uses the configuration serve would read and a scratch of links and small copies under the cache directory; removes a rehearsal's scratch left behind there. Recovers the maps without task logs and prints only the workers' verdict, with no record decoded, level checked, plan, job or restore run. Exit 0 for yes, no or unknown; 1 on an opening refusal. Takes DIRECTORY's lock, never the serving deployment's when reading a checkpoint; writes nothing under --data before the rehearsal.",
    ..UPGRADE_CHECK
};
pub const UPGRADE_BACK: Command = Command {
    reach: Reach::Words(&["upgrade", "--back"]),
    forms: &["[DIRECTORIES]"],
    line: "Makes the previous generation active again, before the upgraded one's first commit.",
    arguments: &[DIRECTORIES],
    notes: "Stop the deployment first, then start the old binary after going back. Past the upgraded generation's first commit, going back is restoring a backup.",
    ..UPGRADE_CHECK
};

pub const CHECKPOINT: Command = Command {
    reach: Reach::Words(&["checkpoint"]),
    set: Set::Public,
    forms: &["[DIRECTORIES]", "--hold NAME [DIRECTORIES] -- COMMAND [ARGUMENT...]", "--remove NAME [DIRECTORIES]"],
    line: "takes a checkpoint of the stopped deployment; --hold runs a command under one's lock, --remove removes one",
    arguments: &[
        DIRECTORIES,
        Argument { shown: "NAME", line: "A checkpoint name: 20 decimal digits, a hyphen and 16 hexadecimal digits." },
        Argument {
            shown: "COMMAND [ARGUMENT...]",
            line: "The child command and its arguments, only with --hold and after --.",
        },
    ],
    settings: &[&directories::SETTINGS],
    switches: &[],
    variables: &[&environment::PATH, &crate::checkpoint::DIRECTORY],
    notes: "--hold and --remove exclude each other. Uses the configuration serve reads. Taking one requires a stopped deployment; holding or removing one also works while it serves. --hold inherits the working directory and environment, sets TEMPER_CHECKPOINT to the checkpoint's absolute directory, and holds its lock until the child is closed. Exit 0 on success, 1 on refusal, 2 for invalid arguments; --hold exits as its child did, 128 plus its terminating signal, or 127 when the command cannot run.",
};

pub const RESTORE: Command = Command {
    reach: Reach::Words(&["restore"]),
    set: Set::Public,
    forms: &["NAME [DIRECTORIES]"],
    line: "restores a checkpoint over the stopped deployment, after a safety checkpoint of it",
    arguments: &[
        Argument { shown: "NAME", line: "A checkpoint name: 20 decimal digits, a hyphen and 16 hexadecimal digits." },
        DIRECTORIES,
    ],
    settings: &[&directories::SETTINGS],
    switches: &[],
    variables: &[],
    notes: "Uses the configuration serve reads. Stop the deployment first. The state before restoring is kept as a safety checkpoint; restore that checkpoint to bring it back. Exit 0 on success, 1 on refusal, 2 for invalid arguments.",
};

pub const FETCH: Command =
    Command { variables: &[&environment::SSL_CERT_FILE], ..from_row(&super::forge::FETCH, &[DIRECTORIES, FORGE]) };
pub const PREPARE: Command = from_row(&super::forge::PREPARE, &[DIRECTORIES, FORGE]);
pub const RUN: Command =
    Command { variables: &[&environment::LISTEN_FDS], ..from_row(&super::forge::RUN, &[DIRECTORIES, FORGE]) };
pub const REMOVE: Command = from_row(&super::forge::REMOVE, &[DIRECTORIES, FORGE]);
pub const SOURCE: Command = from_row(&super::forge::SOURCE, &[DIRECTORIES, FORGE]);
pub const SIGN_IN: Command = from_row(&super::forge::SIGN_IN, &[DIRECTORIES, FORGE]);
pub const FORGE_CHILD: Command = from_row(
    &super::forge::FORGE_CHILD,
    &[
        Argument { shown: "PID", line: "The process whose end ends the program." },
        Argument { shown: "PROGRAM", line: "The program to run, with its ARGS." },
    ],
);

// The legacy rows remain the box's until its table moves; both tables use their wording.
const fn from_row(row: &Row, arguments: &'static [Argument]) -> Command {
    Command {
        reach: Reach::Words(row.words),
        set: Set::Internal,
        forms: row.forms,
        line: row.line,
        arguments,
        settings: row.settings,
        switches: &[],
        variables: &[],
        notes: row.notes,
    }
}

#[cfg(test)]
mod tests;
