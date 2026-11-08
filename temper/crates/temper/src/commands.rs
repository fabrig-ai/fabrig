//! The binary's table (protocol/README.md, section 4): each command a row,
//! whence the parser's words, `temper --help`, `temper COMMAND --help` and
//! the reference page `commands`.
use crate::{box_command, directories, fake};
pub use temper_shell::commands::{Argument, DIRECTORIES, HELP, Row, Table, Who};

const FORGE: Argument = Argument { shown: "FORGE", line: "A forge's number in the configuration." };

/// Every row, in `temper --help`'s order: by `Who`, then as listed in protocol/README.md, section 4.
pub const COMMANDS: [Row; 23] = [
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
        forms: &["[DIRECTORIES] [--echo none|steps]", "[DIRECTORIES] --fake-backends [--bundle BUNDLE]"],
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
        ],
        settings: &[&directories::SETTINGS, &box_command::ECHO_SETTINGS],
        notes: "Steps run in order, each restarted when it exits until it has exited too often within its window, when the box gives up, stops the others and exits 1; they stop in reverse order. Its stdout is its sign-in links, each line whole; its stderr, the block's other lines and errors, and what --echo chooses; the box and each step log under the data directory.",
    },
    Row {
        words: &["program-login"],
        who: Who::Deployment,
        forms: &["[DIRECTORIES] NAME"],
        line: "Sign a program source in on this host, through the program's own sign-in.",
        arguments: &[
            DIRECTORIES,
            Argument { shown: "NAME", line: "The program source's name: 1 to 32 of a to z, 0 to 9 and -." },
        ],
        settings: &[&directories::SETTINGS],
        notes: "Checks the configuration and the agent's, makes the source's directory, then replaces itself by the program's sign-in, whose exit status is then its own. The engine need not run.",
    },
    Row {
        words: &["forge", "fetch"],
        who: Who::Deployment,
        forms: &["[DIRECTORIES] FORGE"],
        line: "Download an owned forge's Forgejo and runner binaries into the cache.",
        arguments: &[DIRECTORIES, FORGE],
        settings: &[&directories::SETTINGS],
        notes: "Fetches only the binaries the forge's setup gives as \"fetched\", at temper's pinned versions, for amd64 or arm64; another machine is refused, naming its architecture. It says how far each binary has gone on its stdout, a line at each percent. A forge with no group's file takes `application.machine.owned`, in `<data>/forge-N`, and writes nothing under `box/`.",
    },
    Row {
        words: &["forge", "prepare"],
        who: Who::Deployment,
        forms: &["[DIRECTORIES] FORGE"],
        line: "Prepare an owned forge before its Forgejo first runs.",
        arguments: &[DIRECTORIES, FORGE],
        settings: &[&directories::SETTINGS],
        notes: "A forge with no group's file takes `application.machine.owned`, in `<data>/forge-N`, and writes nothing under `box/`.",
    },
    Row {
        words: &["forge", "run"],
        who: Who::Deployment,
        forms: &["[DIRECTORIES] FORGE forgejo|runner"],
        line: "Run an owned forge's Forgejo or its runner.",
        arguments: &[DIRECTORIES, FORGE],
        settings: &[&directories::SETTINGS],
        notes: "Checks the binary and the preparation, and takes the forge's lock, first. Run alone, it replaces itself by the program, whose exit status is then its own; given the forge's listener, as the box gives it, it runs the program as its child. A forge with no group's file takes `application.machine.owned`, in `<data>/forge-N`, and writes nothing under `box/`.",
    },
    Row {
        words: &["forge", "remove"],
        who: Who::Deployment,
        forms: &["[DIRECTORIES] FORGE"],
        line: "Remove a stopped box slot's forge directory, keeping the cache.",
        arguments: &[DIRECTORIES, FORGE],
        settings: &[&directories::SETTINGS],
        notes: "Reads the slot's kind from box/groups/FORGE and refuses a directory outside data or in the cache.",
    },
    Row {
        words: &["forge", "source"],
        who: Who::Deployment,
        forms: &["[DIRECTORIES] FORGE"],
        line: "Add the box’s public temper sign-in source to its owned Forgejo.",
        arguments: &[DIRECTORIES, FORGE],
        settings: &[&directories::SETTINGS],
        notes: "Lists the sources and adds temper when absent, under the setup deadline. Prints source added or source failed WHY and exits 0 either way: a failure keeps the box running with plain links.",
    },
    Row {
        words: &["forge", "sign-in"],
        who: Who::Deployment,
        forms: &["[DIRECTORIES] FORGE"],
        line: "Make what people need to sign in to temper through a Forgejo, and print the Forgejo command that registers temper.",
        arguments: &[DIRECTORIES, FORGE],
        settings: &[&directories::SETTINGS],
        notes: "For a Forgejo of application.sign_in.forges, on temper's host, before temper serve. Writes the provider's key and the forge's client secret where they are missing, private, and leaves any already there; prints Forgejo's add-oauth command, never running it.",
    },
    Row {
        words: &[],
        who: Who::Clients,
        forms: &["[ADDRESS]"],
        line: "Run temper's full-screen terminal client, the TUI, on an engine; temper help tui explains closing and signing out.",
        arguments: &[Argument {
            shown: "ADDRESS",
            line: "Engine listener, http://HOST:PORT on loopback; defaults to the engine link file's address for this environment, else the TUI's last sign-in address, else http://127.0.0.1:8367.",
        }],
        settings: &[],
        notes: "Needs a terminal on stdin and stdout; without one, bare temper says it has no command. Its sign-in link is pasted on its screen, never given. Its directory is $XDG_STATE_HOME/temper/tui, else ~/.local/state/temper/tui. `/exit` or Ctrl+C closes it without signing out; on the sign-in screen, use Ctrl+C. `q` types a character on every screen.",
    },
    Row {
        words: &["ctl"],
        who: Who::Clients,
        forms: &[
            "DIR GROUP VERB [ARGUMENT...] [--json]",
            "DIR me|inbox|sign-in|sign-out|pending [ARGUMENT...] [--json]",
            "DIR --call FILE [--json]",
            "help [GROUP [VERB]]",
        ],
        line: "Act in a deployment as one party, one call a process: the agents' command line.",
        arguments: &[
            Argument {
                shown: "DIR",
                line: "ctl's private directory for one deployment and one party: its sign-in, its kept requests and the engine's address.",
            },
            Argument {
                shown: "GROUP",
                line: "A group of tools, a noun of temper's or one of the client's own names; temper ctl help lists them.",
            },
            Argument { shown: "VERB", line: "One of the group's tools; temper ctl help GROUP lists them." },
            Argument {
                shown: "FILE",
                line: "One call as JSON, {\"tool\": \"GROUP VERB\", \"arguments\": {...}}, or - for stdin.",
            },
        ],
        settings: &[],
        notes: "temper ctl help prints how to work through it, then the groups. A call holds one of the engine's streams while it runs, a wait for its whole span; past the engine's bound a call says so.",
    },
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
    Row {
        words: &["internal", "forge-child"],
        who: Who::Spawned,
        forms: &["--parent PID --signal kill|terminate -- PROGRAM ARGS..."],
        line: "Run an owned forge's program, sent SIGNAL when its parent PID ends.",
        arguments: &[
            Argument { shown: "PID", line: "The process whose end ends the program." },
            Argument { shown: "PROGRAM", line: "The program to run, with its ARGS." },
        ],
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
