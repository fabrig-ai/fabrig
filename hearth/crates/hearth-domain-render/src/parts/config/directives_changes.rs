use super::{Config, LineKeyed, PresetKeyed, ScriptsChange};
use alloc::boxed::Box;

pub(super) fn presets(before: Option<&Config>, after: Option<&Config>) -> Box<[PresetKeyed]> {
    let before = match before {
        Some(config) => config.presets.as_ref(),
        None => &[],
    };
    let after = match after {
        Some(config) => config.presets.as_ref(),
        None => &[],
    };
    crate::diff::keyed(before, after, |row| &row.unit, |change| change.map(|row| row.action))
}
pub(super) fn tmpfiles(before: Option<&Config>, after: Option<&Config>) -> Box<[LineKeyed]> {
    let before = match before {
        Some(config) => config.tmpfiles.as_ref(),
        None => &[],
    };
    let after = match after {
        Some(config) => config.tmpfiles.as_ref(),
        None => &[],
    };
    crate::diff::keyed(before, after, |row| &row.path, |change| change.map(|row| row.line.clone()))
}
pub(super) fn scripts(before: Option<&Config>, after: Option<&Config>) -> Option<ScriptsChange> {
    let before = match before {
        Some(config) => config.scripts.as_ref(),
        None => &[],
    };
    let after = match after {
        Some(config) => config.scripts.as_ref(),
        None => &[],
    };
    if before == after { None } else { Some(ScriptsChange { before: Box::from(before), after: Box::from(after) }) }
}
