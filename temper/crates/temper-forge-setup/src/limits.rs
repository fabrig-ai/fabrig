//! Reconcile's deadline and wait (temper/docs/design/protocol/configuration/limits.md;
//! temper/docs/design/protocol/documentation.md, 8).
use skein_forge_setup::limits::Limits;

/// temper gives no org (protocol/setup/reconcile.md, 3): the least bound the kit takes.
pub const ORGS: u32 = 1;

/// A fetched binary's cap; largest pin: Forgejo 17.0.1 amd64, 122,057,320 bytes.
pub const FETCH_BYTES: u64 = 268_435_456;
/// Reads `setup.fetch_bytes`, which must be above zero.
pub fn fetch_bytes(value: u64) -> Result<u64, Refused> {
    if value == 0 { Err(Refused::Range) } else { Ok(value) }
}
/// Small-profile host admission, beside the pure plan profile returned by `small()`.
pub mod small {
    /// A copy may wait thirty minutes for its source (protocol/setup/owned.md, 9).
    pub const COPY_DEADLINE: skein_lib::Duration = skein_lib::Duration::from_secs(1800);
    // Four forges, each with one writing run and one Reachable beside it.
    pub const RUNS: u32 = 8;
    // A manifest return and an installation return for each of four forges.
    pub const STATES: u32 = 8;
}
/// The engine host's setup limits (protocol/configuration/limits.md).
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Host {
    pub runs: u32,
    pub states: u32,
    pub state_life: Duration,
    pub reachable: u32,
    pub repositories: u32,
    pub deadline: Duration,
    pub backoff: Duration,
    pub backoff_max: Duration,
}
/// The host reuses the existing plan profile.
#[must_use]
pub fn host() -> Host {
    let plan = small();
    Host {
        runs: small::RUNS,
        states: small::STATES,
        state_life: Duration::from_secs(3600),
        reachable: plan.reachable,
        repositories: plan.repositories,
        deadline: plan.deadline,
        backoff: plan.backoff,
        backoff_max: plan.backoff_max,
    }
}
impl Host {
    /// The same pure-plan limits used by the command's existing profile.
    #[must_use]
    pub fn plan(&self) -> Limits {
        Limits {
            orgs: ORGS,
            reachable: self.reachable,
            files: 2,
            file_bytes: 4096,
            repositories: self.repositories,
            deadline: self.deadline,
            copy_deadline: small::COPY_DEADLINE,
            backoff: self.backoff,
            backoff_max: self.backoff_max,
        }
    }
}
/// Applies host overrides, including the pure plan's dotted names.
pub fn apply_host(limits: &mut Host, name: &[u8], value: u64) -> Result<(), Refused> {
    let mut known = false;
    for row in &HOST_NAMES {
        known |= row.name.as_bytes() == name;
    }
    if !known {
        return Err(Refused::Unknown);
    }
    match name {
        b"setup.runs" | b"setup.states" => {
            let count = u32::try_from(value).or(Err(Refused::Range))?;
            if count == 0 {
                return Err(Refused::Range);
            }
            if name == b"setup.runs" {
                limits.runs = count;
            } else {
                limits.states = count;
            }
        }
        b"setup.state_life_ms" => {
            if value == 0 || value > 3_600_000 {
                return Err(Refused::Range);
            }
            limits.state_life = duration(value)?;
        }
        _ => {
            let mut plan = limits.plan();
            apply(&mut plan, name, value)?;
            limits.repositories = plan.repositories;
            limits.deadline = plan.deadline;
            limits.backoff = plan.backoff;
            limits.backoff_max = plan.backoff_max;
        }
    }
    Ok(())
}

use skein_lib::Duration;
/// Why an override cannot be applied.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Refused {
    Unknown,
    Range,
}
/// The shipped profile (forge-setup decision 5).
#[must_use]
pub fn small() -> Limits {
    // Legacy command default; checked startup supplies forge.client.repositories.
    let repositories = 2;
    Limits {
        orgs: ORGS,
        reachable: 8, // Legacy command default; checked startup supplies picker.page.
        files: 2,
        file_bytes: 4096,
        repositories,
        deadline: Duration::from_secs(120),
        copy_deadline: small::COPY_DEADLINE,
        backoff: Duration::from_millis(100),
        backoff_max: Duration::from_secs(5),
    }
}
/// Applies a dotted setup override, refusing zero and overflow.
pub fn apply(limits: &mut Limits, name: &[u8], value: u64) -> Result<(), Refused> {
    let mut known = false;
    for row in &NAMES {
        known |= row.name.as_bytes() == name;
    }
    if !known {
        return Err(Refused::Unknown);
    }
    if value == 0 {
        return Err(Refused::Range);
    }
    match name {
        b"forge.client.repositories" => limits.repositories = u32::try_from(value).or(Err(Refused::Range))?,
        b"setup.deadline_ms" => limits.deadline = duration(value)?,
        b"setup.backoff_ms" => limits.backoff = duration(value)?,
        b"setup.backoff_max_ms" => limits.backoff_max = duration(value)?,
        _ => return Err(Refused::Unknown),
    }
    Ok(())
}
fn duration(value: u64) -> Result<Duration, Refused> {
    match value.checked_mul(1_000_000) {
        Some(n) => Ok(Duration::from_nanos(n)),
        None => Err(Refused::Range),
    }
}

/// What a named limit counts.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Unit {
    Count,
    Bytes,
    Milliseconds,
}

/// One override name and its value in its owner's limits.
#[derive(Clone, Copy, Debug)]
pub struct Name<L> {
    pub name: &'static str,
    pub unit: Unit,
    /// What it bounds, one sentence of at most 120 bytes.
    pub line: &'static str,
    pub value: fn(&L) -> u64,
}

/// Every name the pure-plan override parser takes, in parser order.
pub static NAMES: [Name<Limits>; 4] = [
    Name {
        name: "forge.client.repositories",
        unit: Unit::Count,
        line: "Bounds repositories retained by the forge client and read by a setup plan.",
        value: |limits| u64::from(limits.repositories),
    },
    Name {
        name: "setup.deadline_ms",
        unit: Unit::Milliseconds,
        line: "Bounds a setup plan's run, including its wait for the forge.",
        value: |limits| limits.deadline.as_millis(),
    },
    Name {
        name: "setup.backoff_ms",
        unit: Unit::Milliseconds,
        line: "Sets the first delay before a setup request is tried again.",
        value: |limits| limits.backoff.as_millis(),
    },
    Name {
        name: "setup.backoff_max_ms",
        unit: Unit::Milliseconds,
        line: "Bounds the doubling delay before a setup request is tried again.",
        value: |limits| limits.backoff_max.as_millis(),
    },
];

/// Every name the setup host override parser takes, in parser order.
pub static HOST_NAMES: [Name<Host>; 7] = [
    Name {
        name: "setup.runs",
        unit: Unit::Count,
        line: "Bounds simultaneous setup host runs, counting a writing run and a reachable read separately.",
        value: |limits| u64::from(limits.runs),
    },
    Name {
        name: "setup.states",
        unit: Unit::Count,
        line: "Bounds states held for manifest and installation returns.",
        value: |limits| u64::from(limits.states),
    },
    Name {
        name: "setup.state_life_ms",
        unit: Unit::Milliseconds,
        line: "Bounds a return state's life, at most the manifest conversion code's hour.",
        value: |limits| limits.state_life.as_millis(),
    },
    Name {
        name: "forge.client.repositories",
        unit: Unit::Count,
        line: "Bounds repositories retained by the forge client and read by a setup plan.",
        value: |limits| u64::from(limits.repositories),
    },
    Name {
        name: "setup.deadline_ms",
        unit: Unit::Milliseconds,
        line: "Bounds a setup plan's run, including its wait for the forge.",
        value: |limits| limits.deadline.as_millis(),
    },
    Name {
        name: "setup.backoff_ms",
        unit: Unit::Milliseconds,
        line: "Sets the first delay before a setup request is tried again.",
        value: |limits| limits.backoff.as_millis(),
    },
    Name {
        name: "setup.backoff_max_ms",
        unit: Unit::Milliseconds,
        line: "Bounds the doubling delay before a setup request is tried again.",
        value: |limits| limits.backoff_max.as_millis(),
    },
];

/// The fetched binary's scalar bound, outside the plan and host limits.
pub static FETCH_NAMES: [Name<u64>; 1] = [Name {
    name: "setup.fetch_bytes",
    unit: Unit::Bytes,
    line: "Bounds a fetched binary's streamed body, written to its part file without holding it in memory.",
    value: |limit| *limit,
}];
