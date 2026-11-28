//! Whole provider settings, read at one inventory revision (configuration/providers.md, 6).
use super::reads::Reading;
use crate::jig::{call::Call, rows};
use jig_client::{Readout, contract as c};

pub(crate) fn all(call: &Call) -> bool {
    call.tool == rows::SETTINGS_AT && call.form == 1
}
pub(super) fn next(reading: &Reading) -> Option<(u32, c::Read)> {
    if !all(&reading.called.call) {
        return None;
    }
    let Readout::Jig(c::ReadResult::Value(c::ReadValue::Settings(inventory))) = reading.pages.get(&1)? else {
        return None;
    };
    for line in &inventory.settings {
        let c::Setting::Provider(number) = line.setting else { continue };
        let slot = number.number.checked_add(8)?;
        match reading.pages.get(&slot) {
            Some(Readout::Jig(c::ReadResult::Value(c::ReadValue::Setting(_)))) => {}
            Some(Readout::Jig(_) | Readout::Family { .. } | Readout::Limit | Readout::NotServed { .. }) => return None,
            None => {
                return Some((
                    slot,
                    c::Read::Setting(c::ReadSetting {
                        project: None,
                        setting: line.setting,
                        revision: inventory.newest,
                        proposal: None,
                    }),
                ));
            }
        }
    }
    None
}
pub(super) fn defaults(reading: &mut Reading, inventory: &c::SettingsDocument) {
    for line in &inventory.settings {
        if let c::Setting::Provider(number) = line.setting
            && line.follows == c::SettingFollows::Default
        {
            reading.provider_defaults.insert(number.number);
        }
    }
}
