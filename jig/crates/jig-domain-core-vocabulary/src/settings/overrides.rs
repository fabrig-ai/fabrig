//! Shape alone of limit overrides (domain/engine/settings.md, 1).
use alloc::boxed::Box;

pub const OVERRIDE_NAME_BYTES: u32 = 64;
pub const OVERRIDE_REASON_BYTES: u32 = 120;
pub const OVERRIDES: u32 = 256;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Override {
    pub name: Box<[u8]>,
    pub value: u64,
    pub reason: Box<[u8]>,
}
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum OverrideFault {
    Count,
    Name { at: u32 },
    Order { at: u32 },
    Reason { at: u32 },
}
#[expect(clippy::disallowed_methods, reason = "the settings shape check owns the reason's UTF-8 invariant")]
pub fn check_overrides(overrides: &[Override]) -> Result<(), OverrideFault> {
    if overrides.len() > skein_lib::usize_of(OVERRIDES) {
        return Err(OverrideFault::Count);
    }
    let mut previous: Option<&[u8]> = None;
    for (at, value) in overrides.iter().enumerate() {
        let at = u32::try_from(at).expect("bounded overrides");
        if value.name.is_empty() || value.name.len() > skein_lib::usize_of(OVERRIDE_NAME_BYTES) {
            return Err(OverrideFault::Name { at });
        }
        for &byte in &value.name {
            if !byte.is_ascii_lowercase() && !byte.is_ascii_digit() && byte != b'_' && byte != b'.' {
                return Err(OverrideFault::Name { at });
            }
        }
        if let Some(previous) = previous
            && previous >= value.name.as_ref()
        {
            return Err(OverrideFault::Order { at });
        }
        if value.reason.is_empty()
            || value.reason.len() > skein_lib::usize_of(OVERRIDE_REASON_BYTES)
            || core::str::from_utf8(&value.reason).is_err()
        {
            return Err(OverrideFault::Reason { at });
        }
        previous = Some(&value.name);
    }
    Ok(())
}
