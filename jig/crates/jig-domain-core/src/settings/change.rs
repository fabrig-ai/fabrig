//! The edits a settings decision leaves (domain/engine/settings.md, 1–2).

use crate::{
    CoreKey, CoreRecord, Write,
    settings::{Changed, Current, Edit, Item, Key, Record, Setting},
};
pub use jig_domain_core_vocabulary::settings::change::*;
use skein_lib::List;
#[must_use]
pub(crate) fn changes_current(current: &Current, items: &[Item]) -> bool {
    for item in items {
        if super::application::is_application(item.setting) || super::is_project_policy(item.setting) {
            continue;
        }
        let after = match &item.edit {
            Edit::Set(value) => Some(value),
            Edit::Removed => {
                if let Some(model) = match current.value(item.setting) {
                    Some(value) => value.as_model(),
                    None => None,
                } {
                    if !model.withdrawn || model.source == super::ModelSource::List {
                        return true;
                    }
                    continue;
                }
                None
            }
            Edit::Reset => match current.defaults.get(&item.setting) {
                Some(default) => Some(&default.value),
                None => None,
            },
        };
        if after != current.value(item.setting) {
            return true;
        }
    }
    false
}
/// Apply only a checked, nonempty set. Both persisted and held changes get the same stamp.
pub(crate) fn apply(current: &mut Current, items: &[Item], revision: u32) -> List<Write> {
    let mut writes = List::with_capacity(
        u32::try_from(items.len())
            .expect("bounded items")
            .checked_mul(2)
            .expect("reset writes")
            .checked_add(current.resolved.capacity())
            .expect("resolution stamps"),
    );
    // Erases precede puts in memory so replacing a whole bounded kind needs no spare slot.
    for item in items {
        if super::is_project_policy(item.setting) {
            continue;
        }
        let erase = match item.edit {
            Edit::Reset => true,
            Edit::Removed => match item.setting {
                Setting::Model(_) => false,
                Setting::Charter(_)
                | Setting::Template(_)
                | Setting::Policy(_)
                | Setting::Rules
                | Setting::Application { .. }
                | Setting::ProjectPolicy { .. } => current.defaults.get(&item.setting).is_none(),
            },
            Edit::Set(_) => false,
        };
        if erase {
            drop(current.changes.remove(&item.setting));
        }
    }
    for item in items {
        if super::is_project_policy(item.setting) {
            continue;
        }
        let value = match &item.edit {
            Edit::Set(value) => Some(Some(value.clone())),
            Edit::Removed => match item.setting {
                Setting::Model(_) => match current.value(item.setting) {
                    Some(super::Value::Model(value)) => {
                        let mut value = value.clone();
                        value.withdrawn = true;
                        if value.source == super::ModelSource::List {
                            value.source = super::ModelSource::Person;
                        }
                        Some(Some(super::Value::Model(value)))
                    }
                    Some(
                        super::Value::Overlay(_)
                        | super::Value::Charter(_)
                        | super::Value::Template(_)
                        | super::Value::Policy(_)
                        | super::Value::Rules(_)
                        | super::Value::Application(_),
                    )
                    | None => None,
                },
                Setting::Charter(_)
                | Setting::Template(_)
                | Setting::Policy(_)
                | Setting::Rules
                | Setting::Application { .. }
                | Setting::ProjectPolicy { .. } => {
                    if current.defaults.get(&item.setting).is_some() {
                        Some(None)
                    } else {
                        None
                    }
                }
            },
            Edit::Reset => None,
        };
        let write = match value {
            Some(value) => {
                let changed = Changed { value, revision };
                drop(current.changes.insert(item.setting, changed.clone()).expect("checked changed count"));
                Write::Save(crate::Record::Core(CoreRecord::Settings(Record::Changed {
                    setting: item.setting,
                    changed,
                })))
            }
            None => Write::Erase(crate::Key::Core(CoreKey::Settings(Key::Changed(item.setting)))),
        };
        writes.push(write).expect("one change write per item");
        if item.edit == Edit::Reset
            && let Some(default) = current.defaults.get_mut(&item.setting)
        {
            default.revision = revision;
            writes
                .push(Write::Save(crate::Record::Core(CoreRecord::Settings(Record::Default {
                    setting: item.setting,
                    default: default.clone(),
                }))))
                .expect("reset default write");
        }
    }
    writes
}
