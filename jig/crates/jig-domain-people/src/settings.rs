//! Setting names and check faults carried by keyed answers (jig/docs/design/domain/engine/settings.md, 1, jig/docs/design/domain/engine/settings-application.md, 1).

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

/// A core or application setting (jig/docs/design/domain/engine/settings.md, 1, jig/docs/design/domain/engine/settings-application.md, 1).
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Setting {
    Charter(u32),
    Template(u32),
    Policy(u32),
    Rules,
    /// Scope 0 is the deployment; otherwise it is a project number.
    Application {
        kind: u32,
        scope: u32,
        number: u32,
    },
    /// A project's own policy, an item of its scope (jig/docs/design/domain/engine/settings.md, 2).
    ProjectPolicy {
        project: u32,
    },
    Model(u32),
    Worker(u32),
    Provider(u32),
    Limits,
}
/// A check fault, including a connector's judgment (jig/docs/design/domain/engine/settings.md, 1, jig/docs/design/domain/engine/settings-application.md, 1).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SettingFault {
    Shape,
    Count,
    Reference,
    Rules,
    Budget,
    Grace,
    Owner,
    Judged,
    Linked,
    Accounts,
    /// A charter, template, policy or model numbered out of its range.
    Range,
    /// A reserved or already offered charter, template or policy name.
    Name,
}

/// Whether an accepted settings revision moves running work (domain/engine/settings.md, 2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Running {
    Stays,
    Moves,
}

/// The charter member rejected by the settings check (domain/engine/settings.md, 9).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SettingMember {
    /// The charter's document count.
    Documents,
    /// A document resource name's segment count.
    NameSegments,
    /// One segment's byte length in a document resource name.
    NameSegmentBytes,
    /// The document path's byte length.
    Path,
    /// The optional heading's byte length.
    Heading,
    /// The document connector's registration.
    Connector,
    /// The charter's stored policy and parts byte length.
    StoredPolicy,
}
/// The failed setting rule; scalar bounds are carried beside it.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum SettingRule {
    /// The supplied count or length exceeds the configured maximum.
    Maximum,
    /// The supplied length falls below the required minimum.
    Minimum,
    /// The supplied connector number is not registered.
    Registered,
}
/// Bounded refusal facts, without copying any of the rejected charter's bytes.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct SettingDetail {
    /// The member relative to the charter.
    pub member: SettingMember,
    /// Document index; zero for documents or stored policy.
    pub document: u32,
    /// Name segment index; zero for members other than `NameSegmentBytes`.
    pub segment: u32,
    /// The failed check, interpreted with the member and bound.
    pub rule: SettingRule,
    /// The supplied count, byte length or connector number.
    pub supplied: u32,
    /// The minimum or maximum; zero for `Registered`.
    pub bound: u32,
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
