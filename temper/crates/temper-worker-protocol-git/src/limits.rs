//! Limits and counted payloads (protocol/git.md, section 8).
use crate::{capped::Capped, operation::Operation, ring::ByteRing};
use alloc::boxed::Box;
use skein_io::kernel;
use skein_lib::{Id, List, Map, Slab, Token};
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
    worst_case(limits, checkout).ok_or(Unfit::Memory)?;
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

/// Argument/environment/message/output/path payloads and one sweep page.
#[must_use]
pub fn worst_case(limits: &Limits, checkout: &checkout::Limits) -> Option<u64> {
    let name = u64::from(checkout.name_bytes);
    // Forty argument boxes; settings/options/ids have fewer than 1024 bytes,
    // and the longest leased push owns its remote and branch twice.
    let arguments = List::<Box<[u8]>>::price(40)?.checked_add(1024)?.checked_add(name.checked_mul(3)?)?;
    // The grant ceiling of jig's protocol/channel.md, section 11, is 8192
    // bytes (the pinned jig schema).
    // Four identity fields, their base64 header and author copies, fixed
    // variable names, and HOME fit this envelope for a filesystem path.
    let environment = List::<Box<[u8]>>::price(14)?.checked_add(32768)?;
    let paths = List::<Box<[u8]>>::price(checkout.conflicts)?
        .checked_add(u64::from(checkout.conflicts).checked_mul(u64::from(checkout.path_bytes))?)?;
    let mut each = arguments
        .checked_add(environment)?
        .checked_add(u64::from(checkout.message_bytes).checked_add(3)?)?
        .checked_add(paths)?;
    // The scan retains its input paths while collecting marked paths.
    each = each.checked_add(paths)?;
    each = each
        .checked_add(Capped::worst_case(limits.stdout_bytes)?)?
        .checked_add(ByteRing::worst_case(limits.stderr_bytes)?)?
        // Growth or a tail copy can coexist with the invocation's held bytes.
        .checked_add(u64::from(limits.stdout_bytes))?
        .checked_add(u64::from(limits.stderr_bytes))?
        .checked_add(u64::from(limits.read_bytes))?
        // The operation, command, and emitted working directory own names too.
        .checked_add(name.checked_mul(6)?)?
        .checked_add(u64::from(checkout.message_bytes).checked_add(3)?)?
        .checked_add(8192)?
        // The repository program retained by the operation and its spawn.
        .checked_add(u64::from(checkout::PROGRAM_BYTES).checked_mul(2)?)?;
    let capacity = limits.operations.checked_add(1)?;
    let table = Slab::<Operation>::worst_case(capacity.checked_mul(2)?)?
        .checked_add(Map::<Token, Id<Operation>>::price(capacity)?)?;
    let page = List::<kernel::Entry>::price(limits.sweep_entries)?.checked_add(u64::from(limits.sweep_bytes))?;
    let background = u64::try_from(size_of::<crate::files::Task>())
        .ok()?
        .checked_add(u64::try_from(kernel::LONGEST_NAME).ok()?.checked_mul(2)?)?;
    u64::from(limits.operations.max(1))
        .checked_mul(each)?
        .checked_add(page)?
        .checked_add(table)?
        .checked_add(background)?
        // One last checked item program and its answer.
        .checked_add(u64::from(checkout::PROGRAM_BYTES))
}

/// The most records one entry point emits, except abort.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct MaxOut {
    pub events: u32,
    pub io: u32,
    pub kernel: u32,
}
/// One terminal, a spawn or kill and closes, and one file operation.
pub const MAX_OUT: MaxOut = MaxOut { events: 1, io: 5, kernel: 1 };
