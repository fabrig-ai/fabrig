//! Shared variables read by temper (temper/docs/design/cli/README.md, 8).
use skein_shell::{Set, Variable, Way};

/// The compile-time commit variable (temper/docs/design/cli/README.md, 8).
pub const INTERNAL_COMMIT: Variable = Variable {
    name: "TEMPER_INTERNAL_COMMIT",
    set: Set::Internal,
    form: "COMMIT",
    way: Way::Reads,
    line: "The build's commit and tracked-file dirty mark.",
    absent: "The build script supplies unknown when git identity is unavailable.",
    notes: "Read at compile time from temper-version's build script alone; not a runtime override.",
};

pub const INTERNAL_DUMP_LLM: Variable = Variable {
    name: "TEMPER_INTERNAL_DUMP_LLM",
    set: Set::Internal,
    form: "DIR",
    way: Way::Reads,
    line: "Directory for agents' model request dumps.",
    absent: "Do not dump model requests.",
    notes: "The box's worker reads it and sets it for its agents. An empty value disables dumping; a relative path is taken from the working directory.",
};

pub const INTERNAL_RECORD: Variable = Variable {
    name: "TEMPER_INTERNAL_RECORD",
    set: Set::Internal,
    form: "VALUE",
    way: Way::Reads,
    line: "Record the TUI into its directory.",
    absent: "Record nothing.",
    notes: "Any nonempty value enables recording; an empty value is unset.",
};

pub const PATH: Variable = Variable {
    name: "PATH",
    set: Set::Public,
    form: "DIRS",
    line: "Search path for optional executables and child programs.",
    absent: "Optional executables are not discovered.",
    notes: "",
    way: Way::Reads,
};
pub const TERM: Variable = Variable {
    name: "TERM",
    set: Set::Public,
    form: "NAME",
    line: "Terminal type for links and terminal output.",
    absent: "Use terminal output without terminal-specific links.",
    notes: "",
    way: Way::Reads,
};
pub const COLORTERM: Variable = Variable {
    name: "COLORTERM",
    set: Set::Public,
    form: "NAME",
    line: "Terminal true-colour capability.",
    absent: "Use the terminal's indexed colours.",
    notes: "",
    way: Way::Reads,
};
pub const COLORFGBG: Variable = Variable {
    name: "COLORFGBG",
    set: Set::Public,
    form: "COLOURS",
    line: "Terminal foreground and background colours.",
    absent: "Use the default terminal background.",
    notes: "",
    way: Way::Reads,
};
pub const DISPLAY: Variable = Variable {
    name: "DISPLAY",
    set: Set::Public,
    form: "ADDRESS",
    line: "The X display, when opening a browser on this host.",
    absent: "On Linux, use WAYLAND_DISPLAY to identify a graphical session.",
    notes: "",
    way: Way::Reads,
};
pub const WAYLAND_DISPLAY: Variable = Variable {
    name: "WAYLAND_DISPLAY",
    set: Set::Public,
    form: "NAME",
    line: "The Wayland display, when opening a browser on this host.",
    absent: "On Linux, use DISPLAY to identify a graphical session.",
    notes: "",
    way: Way::Reads,
};
pub const SSH_CONNECTION: Variable = Variable {
    name: "SSH_CONNECTION",
    set: Set::Public,
    form: "CONNECTION",
    line: "An SSH session prevents opening a browser on the remote host.",
    absent: "Check SSH_TTY and the display variables.",
    notes: "",
    way: Way::Reads,
};
pub const SSH_TTY: Variable = Variable {
    name: "SSH_TTY",
    set: Set::Public,
    form: "TTY",
    line: "An SSH terminal prevents opening a browser on the remote host.",
    absent: "Check SSH_CONNECTION and the display variables.",
    notes: "",
    way: Way::Reads,
};
pub const SSL_CERT_FILE: Variable = Variable {
    name: "SSL_CERT_FILE",
    set: Set::Public,
    form: "FILE",
    line: "Certificate trust file for HTTPS connections.",
    absent: "Use the machine's certificate trust.",
    notes: "",
    way: Way::Reads,
};
pub const LISTEN_FDS: Variable = Variable {
    name: "LISTEN_FDS",
    set: Set::Public,
    form: "N",
    line: "Inherited listeners beginning at file descriptor 3.",
    absent: "Open the configured listeners.",
    notes: "",
    way: Way::Reads,
};
pub const XDG_RUNTIME_DIR: Variable = Variable {
    name: "XDG_RUNTIME_DIR",
    set: Set::Public,
    form: "DIR",
    line: "Runtime directory for the engine link file.",
    absent: "Use the deployment data directory's link file.",
    notes: "",
    way: Way::Reads,
};

pub const JOB: Variable = Variable {
    name: "TEMPER_JOB",
    set: Set::Public,
    form: "DIR",
    way: Way::Sets,
    line: "The job's directory, set for its scripts.",
    absent: "Set when a worker runs a job's scripts.",
    notes: "",
};
