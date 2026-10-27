//! Resolve review values serially, retaining their answers in the call's existing pages.
use super::reads::Reading;
use jig_client::{Readout, contract as c};

pub(crate) const FIRST: u32 = 8;
pub(crate) fn review(page: &Readout) -> Option<(c::ProposalKey, &c::SettingsReview)> {
    if let Readout::Jig(c::ReadResult::Value(c::ReadValue::OwnerProposal(proposal))) = page
        && let c::OwnerProposalAction::Settings(settings) = &proposal.action
        && let Some(review) = &settings.review
    {
        return Some((proposal.key, review));
    }
    None
}
pub(crate) fn slot(index: usize, proposed: bool) -> u32 {
    FIRST
        .checked_add(u32::try_from(index).expect("bounded review").checked_mul(2).expect("two values"))
        .expect("review slots")
        .checked_add(u32::from(proposed))
        .expect("review slots")
}
pub(super) fn next(reading: &Reading) -> Option<(u32, c::Read)> {
    let (key, review) = review(reading.pages.get(&1)?)?;
    for (index, item) in review.items.iter().enumerate() {
        for (proposed, value) in [(false, &item.now), (true, &item.would_be)] {
            let local = slot(index, proposed);
            let large = match value {
                c::ReviewValue::TooLarge(_) => true,
                c::ReviewValue::Value(_) | c::ReviewValue::None => false,
            };
            if large && !reading.pages.contains_key(&local) {
                return Some((
                    local,
                    c::Read::Setting(c::ReadSetting {
                        setting: item.setting,
                        revision: if proposed { 0 } else { review.newest },
                        proposal: if proposed { Some(key) } else { None },
                    }),
                ));
            }
        }
    }
    None
}
