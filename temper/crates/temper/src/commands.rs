//! The binary's table (protocol/README.md, section 4): each command a row,
//! whence the parser's words, `temper --help`, `temper COMMAND --help` and
//! the reference page `commands`.
use crate::{box_command, directories, fake};
pub use temper_shell::commands::{Argument, DIRECTORIES, HELP, Row, Table, Who};
use temper_shell::rows;

use temper_engine_shell::forge::{FETCH, FORGE, FORGE_CHILD, PREPARE, REMOVE, RUN, SIGN_IN, SOURCE};

/// Every row, in `temper --help`'s order: by `Who`, then as listed in protocol/README.md, section 4.
pub const COMMANDS: [Row; 24] = [
    HELP,
    Row {
        words: &["init"],
        who: Who::Deployment,
        forms: &["[DIRECTORIES] [--box]"],
        line: "Write a deployment that starts, with temper's default configuration, into the three directories.",
        arguments: &[DIRECTORIES, Argument { shown: "--box", line: "Also write the box's parts: the welcome, the owned Forgejo and the playground." }],
        settings: &[&directories::SETTINGS],
        notes: "Alone, init chooses two ports on `127.0.0.1`, preferring 8367 for clients and 8371 for webhooks. With `--box`, it adds the welcome, owned Forgejo and playground: five ports, preferring 8367 for clients, 8368 for owned Forgejo, 8369 for the fake forge, 8370 for the demo LLM and 8371 for webhooks. Taken ports fall back to the kernel. It writes matching addresses in `temper.json` and `agent.json`.

To choose ports for another box, leave the first running and give init
separate empty configuration, data and cache directories. No port flag
or coordinated edits are needed. See temper help box, step 1.

The saved addresses are fixed: later starts reuse them and refuse an
occupied address. A separate `temper init` closes its reserved listeners
on exit. On empty directories, `temper box` runs init itself and keeps
all five reservations through startup; use that first-run path to avoid
a gap between selection and startup.

Refused, changing nothing, when the configuration directory already holds temper.json.",
    },
    Row {
        words: &["serve"],
        who: Who::Deployment,
        forms: &["[DIRECTORIES]"],
        line: "Run the deployment's services, as its configuration's shape names them, until stopped.",
        arguments: &[DIRECTORIES],
        settings: &[&directories::SETTINGS],
        notes: "Its stdout carries only sign-in links, one a line; its stderr, its startup notes and the reason of a failure. A first termination signal settles the engine's forge writes and drains it; a second aborts. Exit 0 after a drained stop, 1 on a refused start or any failure.",
    },
    Row {
        words: &["box"],
        who: Who::Deployment,
        forms: &["[DIRECTORIES] [--echo none|steps] [--no-tui]", "[DIRECTORIES] --fake-backends [--bundle BUNDLE] [--no-tui]"],
        line: "Run a deployment as one box: the setup steps of each forge it sets up, then temper serve, each a child process.",
        arguments: &[
            DIRECTORIES,
            Argument {
                shown: "--echo",
                line: "none prints errors and a failed step's log; steps prints every step's lines too, prefixed by its name.",
            },
            Argument {
                shown: "--fake-backends",
                line: "Run against temper's fake LLM provider, forge and OAuth issuer, as a deployment of its own under DATA/box/fake-backends that shares the cache.",
            },
            Argument {
                shown: "BUNDLE",
                line: "The web client's bundle directory, in place of the configured one; a relative path is taken from the working directory.",
            },
            Argument { shown: "--no-tui", line: "Run the box with no TUI in front, as on a pipe." },
        ],
        settings: &[&directories::SETTINGS, &box_command::ECHO_SETTINGS],
        notes: "In a terminal, with --echo none, the box runs the TUI in front, signed in, with its link shown there. The block, --fake-backends lines and errors go to the box log while it runs; quitting the TUI stops the box. Steps run in order, each restarted when it exits until it has exited too often within its window, when the box gives up, stops the others and exits 1; they stop in reverse order. Its stdout is its sign-in links, each line whole; its stderr, the block's other lines and errors, and what --echo chooses; the box and each step log under the data directory.",
    },
    temper_worker_shell::PROGRAM_LOGIN,
    FETCH,
    PREPARE,
    RUN,
    REMOVE,
    SOURCE,
    SIGN_IN,
    rows::TUI,
    rows::CTL,
    Row {
        words: &["open"],
        who: Who::Clients,
        forms: &["DIR LINK_OR_ADDRESS"],
        line: "Open a person's terminal client on a deployment.",
        arguments: &[
            Argument { shown: "DIR", line: "The client's private directory, never one that ctl uses." },
            Argument {
                shown: "LINK_OR_ADDRESS",
                line: "A sign-in link temper serve printed, or the address of an engine the directory is signed in to.",
            },
        ],
        settings: &[],
        notes: "For people; an agent works through temper ctl.",
    },
    Row {
        words: &["internal", "agent"],
        who: Who::Spawned,
        forms: &["AGENT.json"],
        line: "Run one agent for a worker's run, on smith's agent configuration.",
        arguments: &[Argument {
            shown: "AGENT.json",
            line: "The agent's configuration, as the worker's agent.configuration names it.",
        }],
        settings: &[],
        notes: "",
    },
    Row {
        words: &["internal", "agent-list"],
        who: Who::Spawned,
        forms: &["AGENT.json"],
        line: "Read one endpoint's model list, on smith's agent configuration.",
        arguments: &[Argument {
            shown: "AGENT.json",
            line: "The agent's configuration, as the worker's agent.configuration names it.",
        }],
        settings: &[],
        notes: "",
    },
    Row {
        words: &["internal", "serve"],
        who: Who::Spawned,
        forms: &["[DIRECTORIES] --box"],
        line: "Run temper serve as the box's step, on the listeners and the pipe the box gives it.",
        arguments: &[DIRECTORIES],
        settings: &[&directories::SETTINGS],
        notes: "",
    },
    FORGE_CHILD,
    Row {
        words: &["internal", "tui"],
        who: Who::Spawned,
        forms: &["--box ADDRESS"],
        line: "Run the TUI in front of the box that started it, signed in by the link the box hands it.",
        arguments: &[Argument { shown: "ADDRESS", line: "The box's engine, http://HOST:PORT, on loopback." }],
        settings: &[],
        notes: "",
    },
    Row {
        words: &["fake", "llm"],
        who: Who::TestDoubles,
        forms: &[
            "[DIRECTORIES] [--ready PREFIX] [--seed N] [--observations FILE]",
            "--document FILE [--ready PREFIX] [--seed N] [--observations FILE]",
        ],
        line: "Serve a fake LLM provider: the demo's, from the configuration, or one from a JSON document.",
        arguments: &[DIRECTORIES],
        settings: &[&fake::LLM_SETTINGS, &directories::SETTINGS],
        notes: "",
    },
    Row {
        words: &["fake", "forge"],
        who: Who::TestDoubles,
        forms: &[
            "[DIRECTORIES] FORGE [--ready PREFIX] [--seed N] [--observations FILE]",
            "--document FILE [--ready PREFIX] [--seed N] [--observations FILE]",
        ],
        line: "Serve a fake forge: forge FORGE of the configuration, or one from a JSON document.",
        arguments: &[DIRECTORIES, FORGE],
        settings: &[&fake::FORGE_SETTINGS, &directories::SETTINGS],
        notes: "",
    },
    Row {
        words: &["fake", "forge", "check"],
        who: Who::TestDoubles,
        forms: &["[DIRECTORIES] FORGE OBSERVATIONS STORE"],
        line: "Judge a fake forge's observations against a stopped deployment's store.",
        arguments: &[
            DIRECTORIES,
            FORGE,
            Argument { shown: "OBSERVATIONS", line: "The observations file the fake forge wrote." },
            Argument { shown: "STORE", line: "A copy of the deployment's store." },
        ],
        settings: &[&directories::SETTINGS],
        notes: "",
    },
    Row {
        words: &["fake", "oauth"],
        who: Who::TestDoubles,
        forms: &["--document FILE [--ready PREFIX] [--seed N]"],
        line: "Serve a fake OAuth issuer from a JSON document.",
        arguments: &[],
        settings: &[&fake::OAUTH_SETTINGS],
        notes: "",
    },
    Row {
        words: &["fake", "git"],
        who: Who::TestDoubles,
        forms: &["ARGS..."],
        line: "Run temper's fake git with git's arguments, as the demo's git does.",
        arguments: &[],
        settings: &[],
        notes: "",
    },
];

pub const TABLE: Table = Table { binary: "temper", rows: &COMMANDS };

/// The row whose words begin `words`, the longest such.
#[must_use]
pub fn find(words: &[Box<[u8]>]) -> Option<&'static Row> {
    TABLE.find(words)
}
pub(crate) fn unknown(words: &[Box<[u8]>]) -> String {
    TABLE.unknown(words)
}
#[expect(dead_code, reason = "the box keeps its forms wrapper over the shared table")]
pub(crate) fn forms(row: &Row) -> String {
    TABLE.forms(row)
}
pub(crate) fn refused(row: &Row, why: &str) -> String {
    TABLE.refused(row, why)
}
/// `temper --help`.
#[must_use]
pub fn help() -> String {
    TABLE.help()
}
/// `temper COMMAND --help`.
#[must_use]
pub fn row(row: &Row) -> String {
    TABLE.row(row)
}
/// The reference page `commands`, whole, as `docs/help/reference/commands.md` holds it.
#[must_use]
pub fn reference() -> String {
    TABLE.reference()
}
