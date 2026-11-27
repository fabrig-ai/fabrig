//! Bounds shared by parsing, running and rendering.
use skein_lib::Duration;
/// The generic tools' bounds, from the host's preset.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Limits {
    pub rows: u32,
    pub settings: jig_client_protocol_wire::Limits,
    pub program_login: &'static [u8],
    pub application: &'static [u8],
    pub ask_bytes: u64,
    pub result_bytes: u64,
    pub text: u32,
    /// Most bytes of an act's words and a model's or effort's name, checked before sending (client/ctl/bounds.md, 1).
    pub words: u32,
    pub part: u32,
    pub wait: Duration,
    pub wait_most: Duration,
}
/// A tool's notes are at most this many bytes (client/ctl/tools.md, 1).
pub const NOTES_BYTES: usize = 1_024;
/// A tool and each of its argument forms, tried in order.
#[derive(Debug)]
pub struct Tool {
    pub name: &'static [u8],
    pub line: &'static [u8],
    /// What it prints, one line.
    pub prints: &'static [u8],
    pub result: &'static [u8],
    pub forms: &'static [&'static [Argument]],
    pub keyed: Keyed,
    pub area: u8,
    /// What this tool alone does beyond its line; lines end with `\n`.
    pub notes: &'static [u8],
}
#[derive(Debug)]
pub struct Argument {
    pub name: &'static [u8],
    pub shown: &'static [u8],
    pub place: Place,
    pub kind: Kind,
    pub occurs: Occurs,
    pub line: &'static [u8],
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Place {
    Positional,
    Flag,
    Switch,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Occurs {
    Required,
    Optional,
    Repeated { most: u32 },
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Keyed {
    Never,
    Optional,
    Required,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Kind {
    Number,
    /// A text amount in hundreds of spend units; JSON keeps spend units.
    Amount,
    Count,
    /// Inclusive numeric bounds, checked in argv and JSON.
    NumberRange {
        min: u64,
        max: u64,
    },
    /// An item name, or decimal digits within the inclusive numeric bounds.
    NameOrNumber {
        min: u64,
        max: u64,
    },
    /// A closed word or a prefix followed by a bounded number.
    NamedNumber {
        words: &'static [&'static [u8]],
        prefixes: &'static [&'static [u8]],
        min: u64,
        max: u64,
    },
    Word {
        set: &'static [&'static [u8]],
    },
    Text,
    /// A literal name: a dash is a name, never stdin.
    Literal,
    File,
    Duration,
    Proposal,
    /// A project name or number and a finding number, P/N.
    Finding,
    Digest,
    /// A worker's stable name (client/ctl/workers.md, 2).
    Worker,
    Places,
    /// A worker secret's SHA-256, not a watch digest.
    Sha256,
    Condition,
    Project {
        empty: bool,
    },
    Turn,
    Cursor,
    Group,
    Order,
    Tool,
}
/// What the parser accepts of an argument beyond its kind.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Bound {
    None,
    Numbers { least: u64, most: u64 },
    Bytes { most: u64 },
    Seconds { least: u64, most: u64 },
}
/// One source for the parser's checks and the help.
#[must_use]
pub fn bound(argument: &Argument, limits: &Limits) -> Bound {
    match argument.kind {
        Kind::NumberRange { min, max } | Kind::NameOrNumber { min, max } | Kind::NamedNumber { min, max, .. } => {
            Bound::Numbers { least: min, most: max }
        }
        Kind::Count => Bound::Numbers { least: 1, most: u64::from(limits.rows) },
        Kind::Places => Bound::Numbers { least: 1, most: u64::from(u32::MAX) },
        Kind::Worker => Bound::Bytes { most: u64::from(limits.settings.worker_setting_name) },
        Kind::Number if argument.name == b"forge" => Bound::Numbers { least: 0, most: 65_535 },
        Kind::Text | Kind::Literal => Bound::Bytes { most: limits.ask_bytes },
        Kind::File if argument.name == b"items" || argument.name == b"instructions" || argument.name == b"spec" => {
            Bound::Bytes { most: limits.ask_bytes }
        }
        Kind::File if argument.name == b"body" => Bound::Bytes { most: 4096 },
        Kind::Duration if argument.name == b"timeout" => Bound::Seconds {
            least: 0,
            most: limits.wait_most.as_nanos().checked_div(1_000_000_000).expect("nonzero divisor"),
        },
        Kind::Duration if argument.name == b"due" => Bound::Seconds { least: 1, most: 2_592_000 },
        Kind::Number
        | Kind::Amount
        | Kind::File
        | Kind::Duration
        | Kind::Word { .. }
        | Kind::Proposal
        | Kind::Finding
        | Kind::Digest
        | Kind::Sha256
        | Kind::Condition
        | Kind::Project { .. }
        | Kind::Turn
        | Kind::Cursor
        | Kind::Group
        | Kind::Order
        | Kind::Tool => Bound::None,
    }
}
#[must_use]
pub const fn positional(
    name: &'static [u8],
    shown: &'static [u8],
    kind: Kind,
    occurs: Occurs,
    line: &'static [u8],
) -> Argument {
    Argument { name, shown, kind, occurs, line, place: Place::Positional }
}
#[must_use]
pub const fn flag(
    name: &'static [u8],
    shown: &'static [u8],
    kind: Kind,
    occurs: Occurs,
    line: &'static [u8],
) -> Argument {
    Argument { name, shown, kind, occurs, line, place: Place::Flag }
}
#[must_use]
pub const fn switch(name: &'static [u8], line: &'static [u8]) -> Argument {
    Argument { name, shown: b"", kind: Kind::Word { set: &[] }, occurs: Occurs::Optional, line, place: Place::Switch }
}
#[must_use]
pub const fn repeated(
    name: &'static [u8],
    shown: &'static [u8],
    kind: Kind,
    most: u32,
    line: &'static [u8],
) -> Argument {
    flag(name, shown, kind, Occurs::Repeated { most }, line)
}

/// Repeated files and findings take at least one positional value; flag repeats may be absent.
#[must_use]
pub fn required(argument: &Argument) -> bool {
    match argument.occurs {
        Occurs::Required => true,
        Occurs::Optional => false,
        Occurs::Repeated { .. } => {
            argument.place == Place::Positional && (argument.kind == Kind::File || argument.kind == Kind::Finding)
        }
    }
}

/// The published vocabulary of argument placeholders.
pub const PLACEHOLDERS: &[&[u8]] = &[
    b"HOLD",
    b"FINDING",
    b"AREA",
    b"PARTY",
    b"FOLLOW",
    b"TIER",
    b"TASK",
    b"PROJECT",
    b"FORGE",
    b"INSTALLATION",
    b"ACCOUNT",
    b"PROVIDER",
    b"ADAPTER",
    b"SOURCE",
    b"POSITION",
    b"NAME",
    b"HOST:PORT",
    b"URL",
    b"REVISION",
    b"REV",
    b"PROPOSAL",
    b"QUESTION",
    b"CHOICE",
    b"STATE",
    b"NUMBER",
    b"N",
    b"TEMPLATE",
    b"CHARTER",
    b"CURSOR",
    b"COUNT",
    b"PART",
    b"OFFSET",
    b"PAGE",
    b"TURN",
    b"DIGEST",
    b"WORKER",
    b"PLACES",
    b"SHA256",
    b"CONDITION",
    b"DURATION",
    b"TIME",
    b"PRIORITY",
    b"MODEL",
    b"EFFORT",
    b"MODEL[:EFFORT]",
    b"AMOUNT",
    b"ADDRESS",
    b"ORG/NAME",
    b"OWNER",
    b"ORG",
    b"REPOSITORY",
    b"BRANCH",
    b"COMMIT",
    b"PREFIX",
    b"GATE",
    b"LABEL",
    b"IDS",
    b"APP",
    b"ROLE",
    b"FILE",
    b"PATH",
    b"TEXT",
    b"REASON",
    b"KEY",
    b"GROUP",
    b"VERB",
];

/// A published name has at most one separating space.
#[must_use]
pub fn name_words(name: &[u8]) -> (&[u8], Option<&[u8]>) {
    for (at, byte) in name.iter().enumerate() {
        if *byte == b' ' {
            let (group, rest) = name.split_at_checked(at).expect("name position");
            return (group, rest.strip_prefix(b" "));
        }
    }
    (name, None)
}
