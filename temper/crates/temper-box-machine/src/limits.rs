//! Restart and shutdown bounds (temper/docs/design/protocol/configuration/limits.md;
//! temper/docs/design/protocol/documentation.md, 8).
use skein_lib::Duration;
/// The box limits, durations in milliseconds at configuration boundaries.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Limits {
    pub backoff: Duration,
    pub backoff_max: Duration,
    pub restarts: u32,
    pub window: Duration,
    pub grace: Duration,
    pub line_bytes: u32,
    pub log_bytes: u32,
    pub tail_lines: u32,
    pub groups: u32,
}
/// The most restarts whose exit times can be retained.
pub const MAX_RESTARTS: u32 = 64;
/// One timestamp, the longest diagnostic piece, and its added LF.
pub const RECORD_BYTES: u32 = 21_u32
    .checked_add(crate::host::LINE_BYTES)
    .expect("record piece fits u32")
    .checked_add(1)
    .expect("record LF fits u32");
/// The largest failure tail retained per step.
pub const MAX_TAIL_LINES: u32 = 100;
/// The failed cross-check or restart bound.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Unfit {
    Backoff,
    Window,
    Restarts,
    LineBytes,
    LogBytes,
    TailLines,
    Groups,
}
/// An unknown name or a value beyond u32.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Refused {
    Unknown,
    Range,
}
impl Unfit {
    /// The names to report for this failure.
    #[must_use]
    pub fn names(self) -> &'static [&'static [u8]] {
        match self {
            Self::Backoff => &[b"box.backoff_ms", b"box.backoff_max_ms"],
            Self::Window => &[b"box.backoff_max_ms", b"box.window_ms"],
            Self::Restarts => &[b"box.restarts"],
            Self::LineBytes => &[b"box.line_bytes"],
            Self::LogBytes => &[b"box.log_bytes"],
            Self::TailLines => &[b"box.tail_lines"],
            Self::Groups => &[b"box.groups"],
        }
    }
}
/// The shipped profile (forge-setup decision 5).
#[must_use]
pub fn small() -> Limits {
    Limits {
        backoff: Duration::from_millis(500),
        backoff_max: Duration::from_secs(30),
        restarts: 5,
        window: Duration::from_secs(300),
        grace: Duration::from_secs(10),
        line_bytes: 256,
        log_bytes: 4_194_304,
        tail_lines: 10,
        groups: 8,
    }
}
/// Applies one whole dotted name.
pub fn apply(limits: &mut Limits, name: &[u8], value: u64) -> Result<(), Refused> {
    let mut known = false;
    for row in &NAMES {
        known |= row.name.as_bytes() == name;
    }
    if !known {
        return Err(Refused::Unknown);
    }
    let value = u32::try_from(value).or(Err(Refused::Range))?;
    let duration = Duration::from_millis(u64::from(value));
    match name {
        b"box.backoff_ms" => limits.backoff = duration,
        b"box.backoff_max_ms" => limits.backoff_max = duration,
        b"box.restarts" => limits.restarts = value,
        b"box.window_ms" => limits.window = duration,
        b"box.grace_ms" => limits.grace = duration,
        b"box.line_bytes" => limits.line_bytes = value,
        b"box.log_bytes" => limits.log_bytes = value,
        b"box.tail_lines" => limits.tail_lines = value,
        b"box.groups" => limits.groups = value,
        _ => return Err(Refused::Unknown),
    }
    Ok(())
}
/// Checks relationships in their documented order.
pub fn check(limits: &Limits) -> Result<(), Unfit> {
    if limits.backoff > limits.backoff_max {
        return Err(Unfit::Backoff);
    }
    if limits.backoff_max >= limits.window {
        return Err(Unfit::Window);
    }
    if limits.restarts > MAX_RESTARTS {
        return Err(Unfit::Restarts);
    }
    if limits.line_bytes < 64 {
        return Err(Unfit::LineBytes);
    }
    if limits.log_bytes < RECORD_BYTES {
        return Err(Unfit::LogBytes);
    }
    if limits.tail_lines == 0 || limits.tail_lines > MAX_TAIL_LINES {
        return Err(Unfit::TailLines);
    }
    Ok(())
}

/// What a named limit counts.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Unit {
    Count,
    Bytes,
    Milliseconds,
}

/// One override name and its small-profile value reader.
#[derive(Clone, Copy, Debug)]
pub struct Name {
    pub name: &'static str,
    pub unit: Unit,
    /// What it bounds, one sentence of at most 120 bytes.
    pub line: &'static str,
    pub value: fn(&Limits) -> u64,
}

/// Every name the box override parser takes, in parser order.
pub static NAMES: [Name; 9] = [
    Name {
        name: "box.backoff_ms",
        unit: Unit::Milliseconds,
        line: "Sets the first delay before restarting a long step.",
        value: |limits| limits.backoff.as_millis(),
    },
    Name {
        name: "box.backoff_max_ms",
        unit: Unit::Milliseconds,
        line: "Bounds the doubling delay before restarting a long step.",
        value: |limits| limits.backoff_max.as_millis(),
    },
    Name {
        name: "box.restarts",
        unit: Unit::Count,
        line: "Bounds a long step's exits within the restart window before the box gives up.",
        value: |limits| u64::from(limits.restarts),
    },
    Name {
        name: "box.window_ms",
        unit: Unit::Milliseconds,
        line: "Sets the window in which a long step's exits count toward its restart limit.",
        value: |limits| limits.window.as_millis(),
    },
    Name {
        name: "box.grace_ms",
        unit: Unit::Milliseconds,
        line: "Bounds the grace after each of the two stop signals sent to a long step.",
        value: |limits| limits.grace.as_millis(),
    },
    Name {
        name: "box.line_bytes",
        unit: Unit::Bytes,
        line: "Bounds a box pipe line in either direction, including its line feed.",
        value: |limits| u64::from(limits.line_bytes),
    },
    Name {
        name: "box.log_bytes",
        unit: Unit::Bytes,
        line: "Bounds a step's log file before it moves aside and a fresh log starts.",
        value: |limits| u64::from(limits.log_bytes),
    },
    Name {
        name: "box.tail_lines",
        unit: Unit::Count,
        line: "Bounds retained log lines printed after a step's failure.",
        value: |limits| u64::from(limits.tail_lines),
    },
    Name {
        name: "box.groups",
        unit: Unit::Count,
        line: "Bounds group files the box lists and runs at a later start.",
        value: |limits| u64::from(limits.groups),
    },
];
