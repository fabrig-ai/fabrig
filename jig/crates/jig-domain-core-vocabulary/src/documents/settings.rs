//! Settings inventory without values (client/domain/contract/documents.md, 1, and client/domain/contract/README.md, 5.1).

use super::Text;
use crate::{
    Limits,
    settings::{Head, Setting},
};
use jig_domain_people as people;
use skein_lib::List;

#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Follows {
    Default,
    Change,
    Overlay,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SettingLine {
    pub setting: Setting,
    pub name: Option<Text>,
    pub follows: Follows,
    pub last: Option<u32>,
    pub application: Option<crate::settings::application::State>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct HeldRevision {
    pub head: Head,
    pub holders: u32,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct SettingsProposal {
    pub proposer: people::Proposer,
    pub number: u32,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Settings {
    pub newest: u32,
    pub settings: List<SettingLine>,
    pub held: List<HeldRevision>,
    pub pending: List<SettingsProposal>,
}

pub fn document_bound(limits: &Limits, kinds: &[crate::settings::Kind]) -> Option<u64> {
    let count = crate::settings::capacity(limits)?
        .checked_mul(2)?
        .checked_sub(jig_domain_tasks::MODELS)?
        .checked_add(crate::settings::application::capacity(kinds, limits)?)?;
    let mut charters = 0_u64;
    for kind in kinds {
        let lines = crate::settings::application::capacity(core::slice::from_ref(kind), limits)?;
        charters =
            charters.checked_add(u64::from(lines).checked_mul(List::<u32>::price(u32::from(kind.charters))?)?)?;
    }
    List::<SettingLine>::price(count)?
        .checked_add(charters)?
        .checked_add(u64::from(limits.templates).checked_mul(2)?.checked_mul(u64::from(limits.documents.text_bytes))?)?
        .checked_add(List::<HeldRevision>::price(limits.settings.revisions.checked_add(1)?)?)?
        .checked_add(List::<SettingsProposal>::price(limits.settings.proposals)?)?
        .checked_add(u64::try_from(size_of::<Settings>()).ok()?)
}
