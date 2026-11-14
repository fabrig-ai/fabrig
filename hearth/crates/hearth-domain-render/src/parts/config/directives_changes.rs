use super::{Config, LineKeyed, PresetKeyed, ScriptsChange};
use alloc::boxed::Box;
use skein_lib::{Joined, List, Map, bytes};

pub(super) fn presets(before: &Config, after: &Config) -> Box<[PresetKeyed]> {
    let mut old = Map::uncapped();
    for row in &before.presets {
        old.insert(row.unit.as_ref(), row).expect("an uncapped map");
    }
    let mut new = Map::uncapped();
    for row in &after.presets {
        new.insert(row.unit.as_ref(), row).expect("an uncapped map");
    }
    let mut changes = List::uncapped();
    for (key, joined) in old.join(&new) {
        let change = match joined {
            Joined::First(row) => PresetKeyed::Removed { key: bytes::copy_of(key), value: row.action },
            Joined::Second(row) => PresetKeyed::Added { key: bytes::copy_of(key), value: row.action },
            Joined::Both(was, now) if was == now => continue,
            Joined::Both(was, now) => {
                PresetKeyed::Changed { key: bytes::copy_of(key), before: was.action, after: now.action }
            }
        };
        changes.push(change).expect("an uncapped list");
    }
    changes.into_boxed()
}
pub(super) fn tmpfiles(before: &Config, after: &Config) -> Box<[LineKeyed]> {
    let mut old = Map::uncapped();
    for row in &before.tmpfiles {
        old.insert(row.path.as_ref(), row).expect("an uncapped map");
    }
    let mut new = Map::uncapped();
    for row in &after.tmpfiles {
        new.insert(row.path.as_ref(), row).expect("an uncapped map");
    }
    let mut changes = List::uncapped();
    for (key, joined) in old.join(&new) {
        let change = match joined {
            Joined::First(row) => LineKeyed::Removed { key: bytes::copy_of(key), value: row.line.clone() },
            Joined::Second(row) => LineKeyed::Added { key: bytes::copy_of(key), value: row.line.clone() },
            Joined::Both(was, now) if was == now => continue,
            Joined::Both(was, now) => {
                LineKeyed::Changed { key: bytes::copy_of(key), before: was.line.clone(), after: now.line.clone() }
            }
        };
        changes.push(change).expect("an uncapped list");
    }
    changes.into_boxed()
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
