//! Absolute host path normalization before root attachment (smith/docs/design/protocol/agent.md, section 4).
use alloc::boxed::Box;
use skein_lib::{List, Writer};

/// Normalize an absolute host path before its root is attached; no filesystem lookup.
/// The shell attests its absolute spelling, and the run retains only bounded bytes.
#[must_use]
pub fn host_path(path: &[u8], maximum: u32) -> Option<Box<[u8]>> {
    if path.first() != Some(&b'/') || path.len() > usize::try_from(maximum).ok()? {
        return None;
    }
    let capacity = maximum.checked_add(1)? / 2;
    let mut names = List::<&[u8]>::with_capacity(capacity);
    let mut kept = 0_u32;
    let mut start = 1_usize;
    for end in 1..=path.len() {
        if end == path.len() || path.get(end) == Some(&b'/') {
            let part = path.get(start..end)?;
            if part == b".." {
                kept = kept.saturating_sub(1);
            } else if !part.is_empty() && part != b"." {
                for byte in part {
                    if *byte == 0 {
                        return None;
                    }
                }
                match names.get_mut(kept) {
                    Some(previous) => *previous = part,
                    None => names.push(part).ok()?,
                }
                kept = kept.checked_add(1)?;
            }
            start = end.checked_add(1)?;
        }
    }
    let names = names.as_slice().get(..usize::try_from(kept).ok()?)?;
    let mut bytes = 1_usize;
    for (index, name) in names.iter().enumerate() {
        bytes = bytes.checked_add(name.len())?.checked_add(usize::from(index != 0))?;
    }
    let mut writer = Writer::new(bytes);
    writer.put(b"/").ok()?;
    for (index, name) in names.iter().enumerate() {
        if index != 0 {
            writer.put(b"/").ok()?;
        }
        writer.put(name).ok()?;
    }
    Some(writer.finish())
}
