//! Setting names and check faults carried by keyed answers (jig/docs/design/domain/engine/settings.md, 1, jig/docs/design/domain/engine/settings-application.md, 1).

pub use jig_domain_contract::ops::party::{Setting, SettingDetail, SettingFault, SettingMember, SettingRule};

/// The last number of a charter, template, policy or model (jig/docs/design/domain/engine/settings.md, 1).
pub const NUMBERS: u32 = 64;

/// The defaults' last number; higher numbers through `NUMBERS` are a deployment's own
/// (jig/docs/design/domain/engine/settings.md, 1).
pub const STOCK_NUMBERS: u32 = 32;

/// The byte bound for a charter, template or policy's name (jig/docs/design/domain/engine/settings.md, 1).
pub const NAME_BYTES: u32 = 32;

/// A name starts with a lowercase ASCII letter, followed by letters, digits or hyphens.
#[must_use]
pub fn is_name(bytes: &[u8]) -> bool {
    let Some((first, rest)) = bytes.split_first() else { return false };
    if !first.is_ascii_lowercase() || bytes.len() > skein_lib::usize_of(NAME_BYTES) {
        return false;
    }
    for byte in rest {
        if !byte.is_ascii_lowercase() && !byte.is_ascii_digit() && *byte != b'-' {
            return false;
        }
    }
    true
}

/// Whether an accepted settings revision moves running work (domain/engine/settings.md, 2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Running {
    Stays,
    Moves,
}

#[cfg(test)]
mod tests {
    #[test]
    fn setting_name_form() {
        for valid in [b"goal".as_slice(), b"draft-goal", b"a1", &[b'a'; 32]] {
            assert!(super::is_name(valid), "{valid:?}");
        }
        for invalid in [b"".as_slice(), b"Goal", b"1goal", b"-goal", b"goal_x", b"12", &[b'a'; 33]] {
            assert!(!super::is_name(invalid), "{invalid:?}");
        }
    }

    #[test]
    fn settings_number_constants_bound_stock_and_deployment() {
        let numbers: u32 = crate::NUMBERS;
        let stock_numbers: u32 = crate::STOCK_NUMBERS;
        assert_eq!(numbers, 64);
        assert_eq!(stock_numbers, 32);
        assert!(stock_numbers < numbers);
    }
}
