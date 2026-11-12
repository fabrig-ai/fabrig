//! A pending proposal's values are reviewed against the newest settings (domain/engine/settings.md, 6).

use crate::{Limits, settings as s};
use jig_domain_tasks as t;
use skein_lib::List;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ReviewItem {
    pub setting: s::Setting,
    pub edit: crate::reads::RevisionEdit,
    pub now: Option<s::Value>,
    /// true is a change, false a default; absent is not offered.
    pub origin: Option<(bool, u32)>,
    pub would_be: Option<s::Value>,
    pub application: Option<ApplicationReview>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct ApplicationReview {
    pub edit: s::application::SizedEdit,
    pub now: Option<s::application::State>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SettingsReview {
    pub key: Option<u64>,
    pub newest: u32,
    pub items: List<ReviewItem>,
    pub value_bytes: u64,
}
#[must_use]
pub fn settings_heads(items: &[t::SettingHead]) -> List<crate::reads::RevisionItemHead> {
    let mut result = List::with_capacity(t::SETTINGS_ITEMS);
    for item in items {
        result
            .push(crate::reads::RevisionItemHead {
                setting: match item.setting {
                    t::Setting::Charter(n) => s::Setting::Charter(n),
                    t::Setting::Template(n) => s::Setting::Template(n),
                    t::Setting::Policy(n) => s::Setting::Policy(n),
                    t::Setting::Rules => s::Setting::Rules,
                    t::Setting::Model(n) => s::Setting::Model(n),
                },
                edit: match item.edit {
                    t::SettingEdit::Set => crate::reads::RevisionEdit::Set,
                    t::SettingEdit::Removed => crate::reads::RevisionEdit::Removed,
                    t::SettingEdit::Reset => crate::reads::RevisionEdit::Reset,
                },
            })
            .expect("bounded item heads");
    }
    result
}
pub fn document_bound(limits: &Limits) -> Option<u64> {
    s::set_bound(limits)?
        .checked_mul(2)?
        .checked_add(List::<ReviewItem>::price(t::SETTINGS_ITEMS)?)?
        .checked_add(u64::from(t::SETTINGS_ITEMS).checked_mul(List::<u32>::price(u32::from(u8::MAX))?)?)?
        .checked_add(List::<crate::reads::RevisionItemHead>::price(t::SETTINGS_ITEMS)?)?
        .checked_add(u64::try_from(size_of::<SettingsReview>()).ok()?)
}
