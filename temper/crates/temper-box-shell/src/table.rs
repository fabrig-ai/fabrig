//! The box's table and the tree's tables (temper/docs/design/cli/README.md, 6 and 9).
use crate::{box_command, directories, fake};
use skein_shell::{Argument, Command, Reach, Set, Switch, Table, Variable};
use temper_engine_shell::commands::{FETCH, FORGE_CHILD, PREPARE, REMOVE, RUN, SIGN_IN, SOURCE};
use temper_shell::{environment, table::HELP};

pub const TABLE: Table = Table {
    binary: "temper-box",
    line: "temper on this machine, as one box.",
    version: temper_version::VERSION,
    commit: temper_version::COMMIT,
    commands: &COMMANDS,
    settings: &[],
    variables: &[
        &temper_service_shell::WAIT_CHECK,
        &temper_service_shell::CUT_AT_LISTEN,
        &temper_engine_shell::forge_sign_in::NO_REGISTRATION,
        &super::fake_backends::FAKE_BOX_PORTS,
        &temper_version::INTERNAL_COMMIT,
        &temper_client_bundle::INTERNAL_BUNDLE,
        &environment::INTERNAL_DUMP_LLM,
        &temper_worker_shell::INTERNAL_WORKSPACE_REUSE,
    ],
    more: "temper-box COMMAND --help prints one command; temper-box help pages lists the pages.",
};
pub const TREE: [&Table; 4] =
    [&TABLE, &temper_engine_shell::ENGINE, &temper_worker_shell::WORKER, &temper_agent_shell::AGENT];

const DIRECTORIES: Argument =
    Argument { shown: temper_shell::commands::DIRECTORIES.shown, line: temper_shell::commands::DIRECTORIES.line };
const FORGE: Argument =
    Argument { shown: temper_engine_shell::forge::FORGE.shown, line: temper_engine_shell::forge::FORGE.line };
const TUI_VARIABLES: &[&Variable] = &[
    &environment::INTERNAL_RECORD,
    &environment::PATH,
    &environment::TERM,
    &environment::COLORTERM,
    &environment::COLORFGBG,
    &environment::DISPLAY,
    &environment::WAYLAND_DISPLAY,
    &environment::SSH_CONNECTION,
    &environment::SSH_TTY,
    &environment::XDG_RUNTIME_DIR,
    &skein_shell::HOME,
    skein_shell::Base::State.variable(),
];
const BUNDLE_SETTINGS: skein_shell::Settings = skein_shell::Settings {
    tool: "temper",
    about: "The fake backends' web client bundle.",
    table: &[
        skein_shell::Setting {
            name: "bundle",
            value: "BUNDLE",
            set: Set::Internal,
            twin: None,
            about: "The web client's bundle directory; a relative path is taken from the working directory.",
            fallback: skein_shell::Fallback::Text(""),
            notes: "Requires --fake-backends; absent, use the configured bundle.",
        },
        skein_shell::Setting {
            name: "scripts",
            value: "FILE",
            set: Set::Internal,
            twin: None,
            about: "A JSON script file; repeat to join in order, relative to the working directory.",
            fallback: skein_shell::Fallback::Text(""),
            notes: "Requires --fake-backends; replaces generated model scripts for this start.",
        },
    ],
};
const INTERNAL: Command = Command {
    reach: Reach::Words(&[]),
    set: Set::Internal,
    forms: &[],
    line: "",
    arguments: &[],
    settings: &[],
    switches: &[],
    variables: &[],
    notes: "",
};
const FAKE_GIT: Command = Command {
    reach: Reach::Words(&["fake", "git"]),
    forms: &["ARGS..."],
    line: "Run temper's fake git with git's arguments, as the demo's git does.",
    ..INTERNAL
};
pub(crate) const BOX: Command = Command {
    reach: Reach::Words(&[]),
    set: Set::Public,
    forms: &["[--config DIR] [--data DIR] [--cache DIR] [--echo none|steps]"],
    line: "Run a deployment as one box: the setup steps of each forge it sets up, then temper-box serve, each a child process.",
    arguments: &[DIRECTORIES],
    settings: &[&directories::SETTINGS, &box_command::ECHO_SETTINGS, &BUNDLE_SETTINGS],
    switches: &[
        Switch {
            name: "fake-backends",
            set: Set::Internal,
            line: "Run against temper's fake LLM provider, forge and OAuth issuer.",
            notes: "An isolated deployment under DATA/box/fake-backends, sharing the cache.",
        },
        Switch {
            name: "strict",
            set: Set::Internal,
            line: "Fail a model turn no script answers.",
            notes: "Requires --fake-backends.",
        },
        Switch {
            name: "no-tui",
            set: Set::Internal,
            line: "Run the box with no TUI in front, as on a pipe.",
            notes: "",
        },
    ],
    variables: &[
        &environment::PATH,
        &environment::TERM,
        &environment::LISTEN_FDS,
        &environment::SSL_CERT_FILE,
        &environment::XDG_RUNTIME_DIR,
        &environment::JOB,
    ],
    notes: "In a terminal, with --echo none, the box runs the TUI in front, signed in, with its link shown there. The block, --fake-backends lines and errors go to the box log while it runs; quitting the TUI stops the box. Steps run in order, each restarted when it exits until it has exited too often within its window, when the box gives up, stops the others and exits 1; they stop in reverse order. Its stdout is its sign-in links, each line whole; its stderr, the block's other lines and errors, and what --echo chooses; the box and each step log under the data directory.",
};

const COMMANDS: [Command; 29] = [
    BOX,
    Command {
        reach: Reach::Words(&["init"]),
        set: Set::Public,
        forms: &["[DIRECTORIES]"],
        line: "Write a deployment that starts, with temper's default configuration, into the three directories.",
        arguments: &[DIRECTORIES],
        settings: &[&directories::SETTINGS],
        switches: &[Switch { name: "box", set: Set::Internal, line: "Also write the box's parts: the welcome, the owned Forgejo and the playground.", notes: "" }],
        variables: &[&environment::PATH],
        notes: "Alone, init chooses two ports on `127.0.0.1`, preferring 8367 for clients and 8371 for webhooks. With `--box`, it adds the welcome, owned Forgejo and playground: five ports, preferring 8367 for clients, 8368 for owned Forgejo, 8369 for the fake forge, 8370 for the demo LLM and 8371 for webhooks. Taken ports fall back to the kernel. It writes matching addresses in `temper.json` and `agent.json`.

To choose ports for another box, leave the first running and give init
separate empty configuration, data and cache directories. No port flag
or coordinated edits are needed. See temper help box, step 1.

The saved addresses are fixed: later starts reuse them and refuse an
occupied address. A separate `temper-box init` closes its reserved listeners
on exit. On empty directories, `temper-box` runs init itself and keeps
all five reservations through startup; use that first-run path to avoid
a gap between selection and startup.

Refused, changing nothing, when the configuration directory already holds temper.json.",
    },
    Command {
        reach: Reach::Words(&["serve"]),
        set: Set::Public,
        forms: &["[DIRECTORIES]"],
        line: "Run the deployment's services, as its configuration's shape names them, until stopped.",
        arguments: &[DIRECTORIES],
        settings: &[&directories::SETTINGS],
        switches: &[],
        variables: &[&environment::PATH, &environment::TERM, &environment::LISTEN_FDS, &environment::XDG_RUNTIME_DIR, &environment::JOB],
        notes: "Its stdout carries only sign-in links, one a line; its stderr, its startup notes and the reason of a failure. A first termination signal settles the engine's forge writes and drains it; a second aborts. Exit 0 after a drained stop, 1 on a refused start or any failure.",
    },
    temper_worker_shell::PROGRAM_LOGIN,
    HELP,
    temper_engine_shell::commands::VERSIONS,
    FETCH,
    PREPARE,
    RUN,
    REMOVE,
    SOURCE,
    SIGN_IN,
    temper_shell::table::CTL,
    Command {
        reach: Reach::Words(&["open"]),
        set: Set::Internal,
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
        switches: &[],
        variables: &[],
        notes: "For people; an agent works through temper ctl.",
    },
    Command {
        reach: Reach::Words(&["internal", "agent"]),
        set: Set::Internal,
        forms: &["AGENT.json"],
        line: "Run one agent for a worker's run, on smith's agent configuration.",
        arguments: &[Argument {
            shown: "AGENT.json",
            line: "The agent's configuration, as the worker's agent.configuration names it.",
        }],
        settings: &[],
        switches: &[],
        variables: &[],
        notes: "",
    },
    Command {
        reach: Reach::Words(&["internal", "agent-list"]),
        set: Set::Internal,
        forms: &["AGENT.json"],
        line: "Read one endpoint's model list, on smith's agent configuration.",
        arguments: &[Argument {
            shown: "AGENT.json",
            line: "The agent's configuration, as the worker's agent.configuration names it.",
        }],
        settings: &[],
        switches: &[],
        variables: &[],
        notes: "",
    },
    Command {
        reach: Reach::Words(&["internal", "serve"]),
        set: Set::Internal,
        forms: &["[DIRECTORIES] --box"],
        line: "Run temper-box serve as the box's step, on the listeners and the pipe the box gives it.",
        arguments: &[DIRECTORIES],
        settings: &[&directories::SETTINGS],
        switches: &[Switch { name: "box", set: Set::Internal, line: "Take the box's listeners and pipe.", notes: "" }],
        variables: &[&environment::PATH, &environment::TERM, &environment::LISTEN_FDS, &environment::XDG_RUNTIME_DIR, &environment::JOB],
        notes: "",
    },
    FORGE_CHILD,
    Command {
        reach: Reach::Words(&["internal", "tui"]),
        set: Set::Internal,
        forms: &["--box ADDRESS"],
        line: "Run the TUI in front of the box that started it, signed in by the link the box hands it.",
        arguments: &[Argument { shown: "ADDRESS", line: "The box's engine, http://HOST:PORT, on loopback." }],
        settings: &[],
        switches: &[],
        variables: TUI_VARIABLES,
        notes: "",
    },
    Command {
        reach: Reach::Words(&["fake", "llm"]),
        set: Set::Internal,
        forms: &[
            "[DIRECTORIES] [--ready PREFIX] [--seed N] [--observations FILE] [--strict]",
            "--document FILE [--ready PREFIX] [--seed N] [--observations FILE] [--scripts SCRIPTS] [--strict]",
        ],
        line: "Serve a fake LLM provider: the demo's, from the configuration, or one from a JSON document.",
        arguments: &[DIRECTORIES],
        settings: &[&fake::LLM_SETTINGS, &directories::SETTINGS],
        switches: &[Switch {
            name: "strict",
            set: Set::Internal,
            line: "Fail a turn no script answers.",
            notes: "",
        }],
        variables: &[&environment::LISTEN_FDS],
        notes: "",
    },
    Command {
        reach: Reach::Words(&["fake", "forge"]),
        set: Set::Internal,
        forms: &[
            "[DIRECTORIES] FORGE [--ready PREFIX] [--seed N] [--observations FILE]",
            "--document FILE [--ready PREFIX] [--seed N] [--observations FILE]",
        ],
        line: "Serve a fake forge: forge FORGE of the configuration, or one from a JSON document.",
        arguments: &[DIRECTORIES, FORGE],
        settings: &[&fake::FORGE_SETTINGS, &directories::SETTINGS],
        switches: &[],
        variables: &[&environment::LISTEN_FDS],
        notes: "",
    },
    Command {
        reach: Reach::Words(&["fake", "forge", "check"]),
        set: Set::Internal,
        forms: &["[DIRECTORIES] FORGE OBSERVATIONS STORE"],
        line: "Judge a fake forge's observations against a stopped deployment's store.",
        arguments: &[
            DIRECTORIES,
            FORGE,
            Argument { shown: "OBSERVATIONS", line: "The observations file the fake forge wrote." },
            Argument { shown: "STORE", line: "A copy of the deployment's store." },
        ],
        settings: &[&directories::SETTINGS],
        switches: &[],
        variables: &[],
        notes: "",
    },
    Command {
        reach: Reach::Words(&["fake", "oauth"]),
        set: Set::Internal,
        forms: &["--document FILE [--ready PREFIX] [--seed N]"],
        line: "Serve a fake OAuth issuer from a JSON document.",
        arguments: &[],
        settings: &[&fake::OAUTH_SETTINGS],
        switches: &[],
        variables: &[&environment::LISTEN_FDS],
        notes: "",
    },
    FAKE_GIT,
    Command { reach: Reach::RunAs("git"), ..FAKE_GIT },
    temper_agent_shell::CLAUDE,
    temper_agent_shell::RELAY,
    temper_agent_shell::PROGRAM_CHILD,
    Command { reach: Reach::Words(&["--list", "internal", "agent-list"]), forms: &["AGENT.json"], line: "Read one endpoint's model list, on smith's agent configuration.", ..INTERNAL },
];
