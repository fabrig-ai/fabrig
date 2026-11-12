//! A pending proposal's values are reviewed against the newest settings (domain/engine/settings.md, 6).

pub use crate::below::documents::settings_review::*;
use crate::{Core, Limits, settings as s};
use alloc::boxed::Box;
use jig_domain_tasks as t;
use skein_lib::List;

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
        let (now, origin) = match item.setting {
            s::Setting::ProjectPolicy { project } => {
                (Some(s::Value::Policy(Box::new(core.configured_policy(project)?))), None)
            }
            s::Setting::Charter(_)
            | s::Setting::Template(_)
            | s::Setting::Policy(_)
            | s::Setting::Model(_)
            | s::Setting::Rules
            | s::Setting::Application { .. } => {
                let (now, origin) = core.current.lookup(item.setting);
                (now.cloned(), if now.is_some() { origin } else { None })
            }
        };
        let (edit, would_be) = match &item.edit {
            s::Edit::Set(value) => (crate::reads::RevisionEdit::Set, Some(value)),
            s::Edit::Removed => (crate::reads::RevisionEdit::Removed, None),
            s::Edit::Reset => (crate::reads::RevisionEdit::Reset, core.current.default_value(item.setting)),
        };
        items
            .push(ReviewItem {
                setting: item.setting,
                edit,
                now,
                origin,
                would_be: match would_be {
                    Some(value @ s::Value::Overlay(_)) => match core.current.made_charter(item.setting, value) {
                        Some(charter) => Some(s::Value::Charter(Box::new(charter))),
                        None => None,
                    },
                    Some(value) => Some(value.clone()),
                    None => None,
                },
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
        key: set.key,
        newest: core.current.newest(),
        items,
        value_bytes: u64::from(limits.documents.text_bytes),
    }))
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
                        key: None,
                        applications: Box::default(),
                        against: 2,
                        items: Box::new([s::Item { setting, edit }]),
                    },
                )
                .unwrap();
            let review = review(&core, &limits, jig_domain_people::Proposer::Party(1), 7).unwrap();
            let item = review.items.get(0).unwrap();
            assert_eq!(review.newest, 2);
            assert_eq!(review.key, None);
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
                    key: None,
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
                        key: Some(23),
                        against: 1,
                        items: Box::default(),
                        applications: Box::new([Recorded { kind: 7, scope: 0, number: 3, edit: edit.clone() }]),
                    },
                )
                .unwrap();
            let review = review(&core, &limits, jig_domain_people::Proposer::Party(1), 1).unwrap();
            assert_eq!(review.items.len(), 1);
            assert_eq!(review.key, Some(23));
            let item = review.items.get(0).unwrap();
            assert_eq!(item.setting, setting);
            assert_eq!((item.now.as_ref(), item.would_be.as_ref(), item.origin), (None, None, None));
            assert_eq!(item.application, Some(ApplicationReview { edit, now: Some(state.clone()) }));
        }
    }
}
