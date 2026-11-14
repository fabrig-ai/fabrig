use super::{Config, GroupKeyed, MemberChange, UserKeyed};
use alloc::boxed::Box;
use skein_lib::List;
pub(super) fn groups(before: Option<&Config>, after: Option<&Config>) -> Box<[GroupKeyed]> {
    let before = match before {
        Some(config) => config.groups.as_ref(),
        None => &[],
    };
    let after = match after {
        Some(config) => config.groups.as_ref(),
        None => &[],
    };
    crate::diff::keyed(before, after, |row| &row.name, |change| change.map(|row| row.kind.clone()))
}
pub(super) fn users(before: Option<&Config>, after: Option<&Config>) -> Box<[UserKeyed]> {
    let before = match before {
        Some(config) => config.users.as_ref(),
        None => &[],
    };
    let after = match after {
        Some(config) => config.users.as_ref(),
        None => &[],
    };
    crate::diff::keyed(before, after, |row| &row.name, |change| change.map(|row| row.kind.clone()))
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
