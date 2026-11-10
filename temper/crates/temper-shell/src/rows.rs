//! The rows shared by binaries (temper/docs/design/protocol/binaries.md).
use crate::commands::{Argument, Row, Who};

pub const HELP: Row = Row {
    words: &["help"],
    who: Who::Documentation,
    forms: &["[NAME | --search WORDS...]"],
    line: "Print the index of temper's pages, one page whole, or the sections that hold every word.",
    arguments: &[
        Argument { shown: "NAME", line: "A page's name, as the index and the pages give it." },
        Argument {
            shown: "WORDS",
            line: "Words that a section must hold every one of, case folded: at most 8, each at most 32 bytes.",
        },
    ],
    settings: &[],
    notes: "Reads no directory and no configuration, so it works before temper init. A name that is no page, or a search that matches nothing, exits 1. A search prints at most 20 hits, a line each, NAME: HEADING: LINE, then \"and more; narrow the words\" when more matched.",
};

pub const TUI: Row = Row {
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
};

pub const CTL: Row = Row {
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
};
