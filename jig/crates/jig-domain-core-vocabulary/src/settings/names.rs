//! Names are reserved per kind, independently of what the settings offer (domain/engine/settings.md, 1).
use super::{Current, Edit, Failure, Fault, Item, Setting, Value, check::edit};
use alloc::boxed::Box;
use skein_lib::List;

#[derive(Clone, Copy, PartialEq, Eq)]
enum Kind {
    Charter,
    Template,
    Policy,
}

fn kind(setting: Setting) -> Option<Kind> {
    match setting {
        Setting::Charter(_) => Some(Kind::Charter),
        Setting::Template(_) => Some(Kind::Template),
        Setting::Policy(_) => Some(Kind::Policy),
        Setting::Rules
        | Setting::Limits
        | Setting::Worker(_)
        | Setting::Model(_)
        | Setting::Provider(_)
        | Setting::Application { .. }
        | Setting::ProjectPolicy { .. } => None,
    }
}

fn reserved(current: &Current, wanted: Kind, name: &[u8]) -> bool {
    for (&setting, default) in &current.defaults {
        if kind(setting) == Some(wanted) && default.value.name() == Some(name) {
            return true;
        }
    }
    false
}

pub(super) fn check_defaults(current: &Current) -> Result<(), Failure> {
    for (&setting, default) in &current.defaults {
        let (Some(wanted), Some(name)) = (kind(setting), default.value.name()) else { continue };
        for (&earlier, other) in &current.defaults {
            if earlier == setting {
                break;
            }
            if kind(earlier) == Some(wanted) && other.value.name() == Some(name) {
                return Err(Failure { setting, fault: Fault::Shape, detail: None });
            }
        }
    }
    Ok(())
}

pub(super) fn check_edits(current: &Current, items: &[Item]) -> Result<(), Failure> {
    for (at, item) in items.iter().enumerate() {
        let (Some(wanted), Edit::Set(value)) = (kind(item.setting), &item.edit) else { continue };
        let Some(name) = value.name() else { continue };
        let failure = Failure { setting: item.setting, fault: Fault::Name, detail: None };
        if let Some(default) = current.default_value(item.setting) {
            if default.name() != Some(name) {
                return Err(failure);
            }
            continue;
        }
        if reserved(current, wanted, name) {
            return Err(failure);
        }
        for (&other, change) in &current.changes {
            if kind(other) == Some(wanted)
                && edit(items, other).is_none()
                && current.default_value(other).is_none()
                && let Some(value) = &change.value
                && value.name() == Some(name)
            {
                return Err(failure);
            }
        }
        for earlier in items.iter().take(at) {
            if kind(earlier.setting) == Some(wanted)
                && let Edit::Set(value) = &earlier.edit
                && value.name() == Some(name)
            {
                return Err(failure);
            }
        }
    }
    Ok(())
}

/// At a default's number, even a whole change follows the default's name.
pub fn fill(current: &Current, setting: Setting, value: &mut Value) {
    if kind(setting).is_some()
        && let Some(default) = current.default_value(setting)
    {
        value.set_name(default.name());
    }
}

/// Install the supplied defaults' names on the current changes, withholding a name
/// the release now reserves. The stored own value is not rewritten at start.
#[must_use]
pub fn restore(current: &mut Current) -> Box<[Setting]> {
    let mut fill = List::with_capacity(current.changes.len());
    let mut withheld = List::with_capacity(current.changes.len());
    for (&setting, change) in &current.changes {
        let (Some(wanted), Some(value)) = (kind(setting), change.value.as_ref()) else { continue };
        if let Some(default) = current.default_value(setting) {
            if default.name() != value.name() {
                fill.push(setting).expect("one per change");
            }
        } else if let Some(name) = value.name()
            && reserved(current, wanted, name)
        {
            withheld.push(setting).expect("one per change");
        }
    }
    for &setting in &fill {
        let name = current.defaults.get(&setting).expect("default name").value.name();
        current.changes.get_mut(&setting).expect("whole change").value.as_mut().expect("offered").set_name(name);
    }
    for &setting in &withheld {
        current.changes.get_mut(&setting).expect("own change").value.as_mut().expect("offered").set_name(None);
    }
    withheld.into_boxed()
}
