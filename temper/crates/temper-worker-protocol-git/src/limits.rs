//! Limits and counted payloads (protocol/git.md, section 8).
use skein_io::kernel;
use temper_worker_domain_checkout::{self as checkout, git};

/// The component's configured bounds.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Limits {
    pub operations: u32,
    pub stdout_bytes: u32,
    pub stderr_bytes: u32,
    pub read_bytes: u32,
    pub sweep_entries: u32,
    pub sweep_bytes: u32,
}
/// Foreground operations and the background filesystem task.
#[must_use]
pub fn operations(limits: &Limits) -> Option<u32> {
    limits.operations.checked_add(1)
}

/// The rule that refuses startup, named under `git_component`.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Unfit {
    Operations,
    StdoutBytes,
    StderrBytes,
    ReadBytes,
    SweepEntries,
    SweepBytes,
    Memory,
}

/// Validate every rule before allocating the component.
pub fn check(limits: &Limits, checkout: &checkout::Limits) -> Result<(), Unfit> {
    if limits.operations < checkout.workspaces {
        return Err(Unfit::Operations);
    }
    let output = largest_output(checkout).ok_or(Unfit::StdoutBytes)?;
    if limits.stdout_bytes < output {
        return Err(Unfit::StdoutBytes);
    }
    if u64::from(limits.stderr_bytes) < u64::try_from(git::PushDiagnostic::CAPACITY).expect("diagnostic bound fits u64")
    {
        return Err(Unfit::StderrBytes);
    }
    if limits.read_bytes == 0 {
        return Err(Unfit::ReadBytes);
    }
    if limits.sweep_entries == 0 {
        return Err(Unfit::SweepEntries);
    }
    if u64::from(limits.sweep_bytes) < u64::try_from(kernel::LONGEST_NAME).expect("name bound fits u64") {
        return Err(Unfit::SweepBytes);
    }
    Ok(())
}

/// A NUL path list, or push's framing, URL, refspec and status summary.
#[must_use]
pub fn largest_output(checkout: &checkout::Limits) -> Option<u32> {
    let paths = checkout.conflicts.checked_mul(checkout.path_bytes.checked_add(1)?)?;
    // To <URL> LF, flag TAB <64 digit id>:refs/heads/<branch> TAB summary LF,
    // Done LF. The longest accepted summary and fixed framing fit 256 bytes.
    let push = checkout.name_bytes.checked_mul(2)?.checked_add(256)?;
    Some(paths.max(push).max(65))
}

/// Kernel submissions one entry point makes: one file operation at the
/// queue's edge toward skein-io's kernel.
pub const KERNEL_SUBMISSIONS: u32 = 1;
