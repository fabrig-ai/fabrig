//! The engine's table and the box's shared forge commands (temper/docs/design/cli/README.md, 7).
use skein_shell::{Argument, Command, Reach, Set, Table};
use temper_shell::{commands::Row, directories, environment, table::HELP};

pub const ENGINE: Table = Table {
    binary: "temper-engine",
    line: "the deployment's engine, with the owned forge's commands.",
    version: temper_version::VERSION,
    commit: temper_version::COMMIT,
    commands: &[SERVE, VERSIONS, UPGRADE_CHECK, UPGRADE_BACK, HELP, FETCH, PREPARE, RUN, SIGN_IN, SOURCE, REMOVE, FORGE_CHILD],
    settings: &[&directories::SETTINGS],
    variables: &[
        &temper_service_shell::WAIT_CHECK,
        &temper_service_shell::CUT_AT_LISTEN,
        &super::forge_sign_in::NO_REGISTRATION,
        &temper_version::INTERNAL_COMMIT,
        &temper_client_bundle::INTERNAL_BUNDLE,
    ],
    more: "temper-engine COMMAND --help prints one command; temper-engine help pages lists the pages.",
};

const DIRECTORIES: Argument =
    Argument { shown: temper_shell::commands::DIRECTORIES.shown, line: temper_shell::commands::DIRECTORIES.line };
const FORGE: Argument = Argument { shown: super::forge::FORGE.shown, line: super::forge::FORGE.line };

pub const SERVE: Command = Command {
    reach: Reach::Words(&["serve"]),
    set: Set::Public,
    forms: &["[DIRECTORIES]"],
    line: "Run an engine alone's document until stopped; temper-worker serve runs a worker alone, and temper-box serve runs an engine and its worker in one process.",
    arguments: &[DIRECTORIES],
    settings: &[&directories::SETTINGS],
    switches: &[],
    variables: &[&environment::PATH, &environment::TERM, &environment::LISTEN_FDS, &environment::XDG_RUNTIME_DIR],
    notes: "Its stdout carries only sign-in links, one a line; its stderr, its startup notes and the reason of a failure. A first termination signal settles the engine's forge writes and drains it; a second aborts. Exit 0 after a drained stop, 1 on a refused start or any failure. Refuses a worker alone's document, naming temper-worker serve, or an engine and its worker in one process, naming temper-box serve.",
};

pub const VERSIONS: Command = Command {
    reach: Reach::Words(&["versions"]),
    set: Set::Public,
    forms: &[""],
    line: "Print this binary's patch levels, its store's and families' versions, and its wires, one line each.",
    arguments: &[],
    settings: &[],
    switches: &[],
    variables: &[],
    notes: "Reads no configuration and no store.",
};

pub const UPGRADE_CHECK: Command = Command {
    reach: Reach::Words(&["upgrade", "--check"]), set: Set::Public,
    forms: &["[DIRECTORIES] DIRECTORY"],
    line: "Rehearses an upgrade over a copy; switches nothing.",
    arguments: &[DIRECTORIES, Argument { shown: "DIRECTORY", line: "A stopped deployment's data directory or a copy of one; never written." }],
    settings: &[&directories::SETTINGS], switches: &[], variables: &[],
    notes: "Uses the configuration serve would read and a scratch copy under the cache directory. Reports the patches, restores, elapsed time and whether workers may stay. What follows the switch is not rehearsed. Exit 0 when patches and restores pass, 1 when a start would be refused.",
};
pub const UPGRADE_BACK: Command = Command {
    reach: Reach::Words(&["upgrade", "--back"]), forms: &["[DIRECTORIES]"],
    line: "Makes the previous generation active again, before the patched one's first commit.",
    arguments: &[DIRECTORIES],
    notes: "Stop the deployment first, then start the old binary after going back. Past the patched generation's first commit, going back is restoring a backup.",
    ..UPGRADE_CHECK
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
