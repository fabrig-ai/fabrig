//! Setting names and check faults carried by keyed answers (jig/docs/design/domain/engine/settings.md, 1, jig/docs/design/domain/engine/settings-application.md, 1).

pub use jig_domain_contract::ops::party::{Setting, SettingDetail, SettingFault, SettingMember, SettingRule};

/// The last number of a charter, template, policy or model (jig/docs/design/domain/engine/settings.md, 1).
pub const NUMBERS: u32 = 64;

/// The defaults' last number; higher numbers through `NUMBERS` are a deployment's own
/// (jig/docs/design/domain/engine/settings.md, 1).
pub const STOCK_NUMBERS: u32 = 32;

/// Whether an accepted settings revision moves running work (domain/engine/settings.md, 2).
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Running {
    Stays,
    Moves,
}

#[cfg(test)]
mod tests {
    #[test]
    fn settings_number_constants_bound_stock_and_deployment() {
        let numbers: u32 = crate::NUMBERS;
        let stock_numbers: u32 = crate::STOCK_NUMBERS;
        assert_eq!(numbers, 64);
        assert_eq!(stock_numbers, 32);
        assert!(stock_numbers < numbers);
    }
}
