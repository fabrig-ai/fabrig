//! Worker settings and stable names (jig/docs/design/domain/engine/workers.md, 1–3).
use super::{Current, Edit, Failure, Fault, Item, Setting, Value};
use alloc::boxed::Box;

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct WorkerSetting {
    pub number: u32,
    pub name: Box<[u8]>,
    pub places: u32,
    pub digest: Option<[u8; 32]>,
    pub withdrawn: bool,
}
#[must_use]
pub fn valid_name(name: &[u8], bound: u32) -> bool {
    let Some(first) = name.first() else { return false };
    if !first.is_ascii_alphanumeric() || name.len() > skein_lib::usize_of(bound.min(jig_domain_fleet::WORKER_NAME)) {
        return false;
    }
    for byte in name {
        if !(byte.is_ascii_alphanumeric() || *byte == b'-' || *byte == b'_' || *byte == b'.') {
            return false;
        }
    }
    true
}
#[must_use]
pub fn fits(value: &WorkerSetting, limits: &crate::Limits) -> bool {
    value.number != 0
        && valid_name(&value.name, limits.fleet.worker_name_bytes)
        && value.places != 0
        && value.places <= limits.fleet.slots
}
fn failure(setting: Setting) -> Failure {
    Failure { setting, fault: Fault::Shape, detail: None }
}
pub fn check_change(current: &Current, items: &[Item]) -> Result<(), Failure> {
    for item in items {
        let Setting::Worker(_) = item.setting else { continue };
        match &item.edit {
            Edit::Reset => {
                if current.default_value(item.setting).is_none() {
                    return Err(failure(item.setting));
                }
            }
            Edit::Removed => {
                if let Some(value) = worker(current.value(item.setting))
                    && value.digest.is_none()
                {
                    return Err(failure(item.setting));
                }
            }
            Edit::Set(value) => {
                let value = value.as_worker().ok_or(failure(item.setting))?;
                if value.withdrawn {
                    return Err(failure(item.setting));
                }
                identity(current, item.setting, value)?;
                digest(current, item.setting, value)?;
            }
        }
    }
    Ok(())
}
fn identity(current: &Current, setting: Setting, value: &WorkerSetting) -> Result<(), Failure> {
    for (&held, default) in &current.defaults {
        name_pair(setting, value, held, &default.value)?;
    }
    for (&held, changed) in &current.changes {
        if let Some(previous) = &changed.value {
            name_pair(setting, value, held, previous)?;
        }
    }
    Ok(())
}
fn name_pair(setting: Setting, value: &WorkerSetting, held: Setting, previous: &Value) -> Result<(), Failure> {
    if let Some(previous) = previous.as_worker()
        && ((held == setting) != (previous.name == value.name))
    {
        return Err(failure(setting));
    }
    Ok(())
}
fn digest(current: &Current, setting: Setting, value: &WorkerSetting) -> Result<(), Failure> {
    let local = match worker(current.default_value(setting)) {
        Some(value) => value.digest.is_none(),
        None => false,
    };
    if value.digest.is_none() != local {
        return Err(failure(setting));
    }
    Ok(())
}
pub fn check_set(current: &Current, defaults: bool, items: &[Item]) -> Result<(), Failure> {
    let names = super::check::offered(current, defaults, items);
    for &setting in &names {
        let Some(value) = worker(super::check::overlay_value(current, defaults, items, setting)) else { continue };
        identity(current, setting, value)?;
        digest(current, setting, value)?;
        for item in items {
            if let Edit::Set(previous) = &item.edit {
                name_pair(setting, value, item.setting, previous)?;
            }
        }
    }
    Ok(())
}
pub fn check_defaults(current: &Current, in_process: bool) -> Result<(), Failure> {
    for (&setting, default) in &current.defaults {
        if let Some(value) = default.value.as_worker()
            && value.digest.is_none()
            && !in_process
        {
            return Err(failure(setting));
        }
    }
    Ok(())
}

#[must_use]
pub fn worker(value: Option<&Value>) -> Option<&WorkerSetting> {
    value?.as_worker()
}
