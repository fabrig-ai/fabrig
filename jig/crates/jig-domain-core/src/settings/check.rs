//! One whole-set check, also used after restoring stored changes (domain/engine/settings.md, 1).

pub use crate::below::settings::check::*;
use crate::{
    Limits, Settings,
    settings::{Current, Failure, Item, Setting},
};
use alloc::boxed::Box;
use jig_domain_tasks as tasks;

/// Check the maps in place: no third materialised set of values is constructed.
pub fn check(current: &Current, settings: &Settings, connectors: &[u16], limits: &Limits) -> Result<(), Failure> {
    check_set(current, true, &[], settings, connectors, limits)?;
    check_set(current, false, &[], settings, connectors, limits)
}

/// Read templates from the same overlay that checks the decision, without publishing it.
pub(crate) fn templates(current: &Current, items: &[Item]) -> Box<[crate::Template]> {
    let names = offered(current, false, items);
    let mut result = skein_lib::List::with_capacity(names.len());
    for &setting in &names {
        if let Some(value) = overlay_value(current, false, items, setting)
            && let Some(template) = value.as_template()
        {
            result.push(template.clone()).expect("one template per offered name");
        }
    }
    result.into_boxed()
}

/// Check a streamed change before retaining its value under a per-kind heap bound.
pub(super) fn admits_change(current: &Current, setting: Setting, limits: &Limits) -> Result<(), Failure> {
    let mut counts = [0_u32; 3];
    for (&held, _) in &current.changes {
        count(held, &mut counts, limits)?;
    }
    count(setting, &mut counts, limits)
}

/// Only the tasks child's bounded model table is materialised for the live-task check.
pub(crate) fn changed_charters(
    current: &Current,
    items: &[Item],
    _settings: &Settings,
    limits: &Limits,
) -> Box<[tasks::CharterModels]> {
    let catalogue = super::catalogue::overlay(current, false, items);
    let models = super::catalogue::rows(&catalogue);
    let mut result = skein_lib::List::with_capacity(limits.tasks.charters);
    for &setting in &offered(current, false, items) {
        let value = overlay_value(current, false, items, setting);
        if let Some(value) = value
            && let Some(charter) = current.made_charter(setting, value)
        {
            result
                .push(super::charters::one_models_rows(
                    &super::charters::resolved(&charter, &catalogue, limits).expect("checked resolution"),
                    current.newest(),
                    &models,
                ))
                .expect("checked charter count");
        }
    }
    result.into_boxed()
}
