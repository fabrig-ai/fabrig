//! A pending proposal's values are reviewed against the newest settings (engine.md, 17.6).
use crate::{Core, Limits, settings as s};
use alloc::boxed::Box;
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
pub(super) fn review(
    core: &Core,
    limits: &Limits,
    proposer: jig_domain_people::Proposer,
    number: u64,
) -> Option<Box<SettingsReview>> {
    let number = u32::try_from(number).ok()?;
    let set = core.settings_proposals.get(&(proposer, number))?;
    let mut items = List::with_capacity(t::SETTINGS_ITEMS);
    for item in &set.items {
        let (now, origin) = core.current.lookup(item.setting);
        let origin = if now.is_some() { origin } else { None };
        let (edit, would_be) = match &item.edit {
            s::Edit::Set(value) => (crate::reads::RevisionEdit::Set, Some(value)),
            s::Edit::Removed => (crate::reads::RevisionEdit::Removed, None),
            s::Edit::Reset => (crate::reads::RevisionEdit::Reset, core.current.default_value(item.setting)),
        };
        items
            .push(ReviewItem {
                setting: item.setting,
                edit,
                now: now.cloned(),
                origin,
                would_be: would_be.cloned(),
                application: None,
            })
            .expect("bounded review");
    }
    for item in &set.applications {
        let setting = s::Setting::Application { kind: item.kind, scope: item.scope, number: item.number };
        let now = core.current.applications.get(&setting).cloned();
        let edit = match item.edit {
            s::application::SizedEdit::Set { .. } => crate::reads::RevisionEdit::Set,
            s::application::SizedEdit::Remove => crate::reads::RevisionEdit::Removed,
            s::application::SizedEdit::Reset => crate::reads::RevisionEdit::Reset,
        };
        items
            .push(ReviewItem {
                setting,
                edit,
                now: None,
                origin: None,
                would_be: None,
                application: Some(ApplicationReview { edit: item.edit.clone(), now }),
            })
            .expect("bounded application review");
    }
    Some(Box::new(SettingsReview {
        newest: core.current.newest(),
        items,
        value_bytes: u64::from(limits.documents.text_bytes),
    }))
}
pub(super) fn document_bound(limits: &Limits) -> Option<u64> {
    s::set_bound(limits)?
        .checked_mul(2)?
        .checked_add(List::<ReviewItem>::price(t::SETTINGS_ITEMS)?)?
        .checked_add(u64::from(t::SETTINGS_ITEMS).checked_mul(List::<u32>::price(u32::from(u8::MAX))?)?)?
        .checked_add(List::<crate::reads::RevisionItemHead>::price(t::SETTINGS_ITEMS)?)?
        .checked_add(u64::try_from(size_of::<SettingsReview>()).ok()?)
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn review_set_removal_and_reset_keep_now_and_the_would_be_default() {
        let (mut core, limits) = crate::tests::core();
        let setting = s::Setting::Charter(1);
        let default = core.current.value(setting).unwrap().clone();
        let mut changed = core.current.charter(1).unwrap().clone();
        changed.policy.instructions = b"changed".as_slice().into();
        let changed = s::Value::Charter(Box::new(changed));
        core.current.changes.insert(setting, s::Changed { value: Some(changed.clone()), revision: 2 }).unwrap();
        core.current.head.number = 2;
        for (edit, expected) in [
            (s::Edit::Set(default.clone()), Some(default.clone())),
            (s::Edit::Removed, None),
            (s::Edit::Reset, Some(default.clone())),
        ] {
            core.settings_proposals
                .insert(
                    (jig_domain_people::Proposer::Party(1), 7),
                    s::Proposal {
                        applications: Box::default(),
                        against: 2,
                        items: Box::new([s::Item { setting, edit }]),
                    },
                )
                .unwrap();
            let review = review(&core, &limits, jig_domain_people::Proposer::Party(1), 7).unwrap();
            let item = review.items.get(0).unwrap();
            assert_eq!(review.newest, 2);
            assert_eq!(item.now, Some(changed.clone()));
            assert_eq!(item.origin, Some((true, 2)));
            assert_eq!(item.would_be, expected);
        }
        core.current.changes.insert(setting, s::Changed { value: None, revision: 3 }).unwrap();
        core.current.head.number = 3;
        core.settings_proposals
            .insert(
                (jig_domain_people::Proposer::Party(1), 7),
                s::Proposal {
                    applications: Box::default(),
                    against: 3,
                    items: Box::new([s::Item { setting, edit: s::Edit::Reset }]),
                },
            )
            .unwrap();
        let review = review(&core, &limits, jig_domain_people::Proposer::Party(1), 7).unwrap();
        let item = review.items.get(0).unwrap();
        assert_eq!(item.now, None);
        assert_eq!(item.origin, None);
        assert_eq!(item.would_be, Some(default));
    }
    #[test]
    fn application_reviews_keep_the_sized_edit_and_current_state_without_values() {
        use s::application::{Follows, Recorded, SizedEdit, State};
        let (mut core, limits) = crate::tests::core();
        core.current.applications = skein_lib::Map::with_capacity(1);
        let setting = s::Setting::Application { kind: 7, scope: 0, number: 3 };
        let state = State { follows: Follows::Changed { revision: 2 }, size: 11, charters: Box::new([1]) };
        core.current.applications.insert(setting, state.clone()).unwrap();
        for edit in [SizedEdit::Set { size: 23 }, SizedEdit::Remove, SizedEdit::Reset] {
            core.settings_proposals
                .insert(
                    (jig_domain_people::Proposer::Party(1), 1),
                    s::Proposal {
                        against: 1,
                        items: Box::default(),
                        applications: Box::new([Recorded { kind: 7, scope: 0, number: 3, edit: edit.clone() }]),
                    },
                )
                .unwrap();
            let review = review(&core, &limits, jig_domain_people::Proposer::Party(1), 1).unwrap();
            assert_eq!(review.items.len(), 1);
            let item = review.items.get(0).unwrap();
            assert_eq!(item.setting, setting);
            assert_eq!((item.now.as_ref(), item.would_be.as_ref(), item.origin), (None, None, None));
            assert_eq!(item.application, Some(ApplicationReview { edit, now: Some(state.clone()) }));
        }
    }
}
