//! Recorded versions and the upgrade's backup name (protocol/setup.md, 6.3).
use crate::check::joined;
use alloc::boxed::Box;
use core::cmp::Ordering;
/// How a recorded version stands against its pin; a later variant is worse.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum Course {
    /// The same version: a later preparation.
    Same,
    /// Older: migrate Forgejo or prepare the runner again.
    Older,
    /// Newer: migrations only go forward.
    Newer,
    /// Not a dotted decimal within the reader's bounds.
    Unreadable,
}
fn components(input: &[u8]) -> Option<[u32; 4]> {
    let mut parts = [0_u32; 4];
    let mut index = 0_usize;
    let mut digits = 0_u8;
    for byte in input {
        if *byte == b'.' {
            if digits == 0 {
                return None;
            }
            index = index.checked_add(1)?;
            digits = 0;
        } else {
            if !byte.is_ascii_digit() || digits == 9 {
                return None;
            }
            let value = parts.get_mut(index)?;
            *value = value.checked_mul(10)?.checked_add(u32::from(byte.checked_sub(b'0')?))?;
            digits = digits.checked_add(1)?;
        }
    }
    if digits == 0 {
        return None;
    }
    Some(parts)
}
/// Compares at most four decimal components, each at most nine digits; missing components are zero.
#[must_use]
pub fn course(recorded: &[u8], pinned: &[u8]) -> Course {
    let (Some(recorded), Some(pinned)) = (components(recorded), components(pinned)) else {
        return Course::Unreadable;
    };
    match recorded.cmp(&pinned) {
        Ordering::Equal => Course::Same,
        Ordering::Less => Course::Older,
        Ordering::Greater => Course::Newer,
    }
}
/// The dump's name under backups, naming both prepared versions.
#[must_use]
pub fn backup_name(recorded: &[u8], pinned: &[u8]) -> Box<[u8]> {
    joined(&[b"forgejo-", recorded, b"-to-", pinned, b".zip"])
}
