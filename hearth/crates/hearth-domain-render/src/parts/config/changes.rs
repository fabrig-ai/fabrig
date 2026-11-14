use super::{Config, GroupKeyed, MemberChange, UserKeyed};
use alloc::boxed::Box;
use skein_lib::{Joined, List, Map, bytes};
pub(super) fn groups(before: &Config, after: &Config) -> Box<[GroupKeyed]> {
    let mut old = Map::uncapped();
    for row in &before.groups {
        old.insert(row.name.as_ref(), row).expect("an uncapped map");
    }
    let mut new = Map::uncapped();
    for row in &after.groups {
        new.insert(row.name.as_ref(), row).expect("an uncapped map");
    }
    let mut changes = List::uncapped();
    for (key, joined) in old.join(&new) {
        let change = match joined {
            Joined::First(row) => GroupKeyed::Removed { key: bytes::copy_of(key), value: row.kind.clone() },
            Joined::Second(row) => GroupKeyed::Added { key: bytes::copy_of(key), value: row.kind.clone() },
            Joined::Both(was, now) if was == now => continue,
            Joined::Both(was, now) => {
                GroupKeyed::Changed { key: bytes::copy_of(key), before: was.kind.clone(), after: now.kind.clone() }
            }
        };
        changes.push(change).expect("an uncapped list");
    }
    changes.into_boxed()
}
pub(super) fn users(before: &Config, after: &Config) -> Box<[UserKeyed]> {
    let mut old = Map::uncapped();
    for row in &before.users {
        old.insert(row.name.as_ref(), row).expect("an uncapped map");
    }
    let mut new = Map::uncapped();
    for row in &after.users {
        new.insert(row.name.as_ref(), row).expect("an uncapped map");
    }
    let mut changes = List::uncapped();
    for (key, joined) in old.join(&new) {
        let change = match joined {
            Joined::First(row) => UserKeyed::Removed { key: bytes::copy_of(key), value: row.kind.clone() },
            Joined::Second(row) => UserKeyed::Added { key: bytes::copy_of(key), value: row.kind.clone() },
            Joined::Both(was, now) if was == now => continue,
            Joined::Both(was, now) => {
                UserKeyed::Changed { key: bytes::copy_of(key), before: was.kind.clone(), after: now.kind.clone() }
            }
        };
        changes.push(change).expect("an uncapped list");
    }
    changes.into_boxed()
}
pub(super) fn members(before: Option<&Config>, after: Option<&Config>) -> Box<[MemberChange]> {
    let before = match before {
        Some(config) => config.members.as_ref(),
        None => &[],
    };
    let after = match after {
        Some(config) => config.members.as_ref(),
        None => &[],
    };
    let mut changes =
        List::with_capacity(u32::try_from(before.len().saturating_add(after.len())).expect("member count"));
    for member in before {
        if !after.contains(member) {
            changes.push(MemberChange::Removed(member.clone())).expect("member count");
        }
    }
    for member in after {
        if !before.contains(member) {
            changes.push(MemberChange::Added(member.clone())).expect("member count");
        }
    }
    changes.into_boxed()
}
