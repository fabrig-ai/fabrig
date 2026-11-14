//! The existing typed rows plus the password reference and implicit own group.
use super::{Claimant, Declared, Rows};
use crate::{
    errors::{Collector, Subject},
    kinds::{self, cmdline::Parameter, groups::Group, users::User},
    parts::{
        config::{self, GroupKind, Preset, Tmpfile, UserKind},
        packages::Preseed,
    },
    revision::declaration::{Map, Name},
};
use alloc::boxed::Box;
use skein_lib::{List, bytes::copy_of};
#[derive(Debug)]
pub(super) enum Row {
    User(User, Option<Box<[u8]>>),
    Group(Group, bool),
    Preset(Preset, Claimant),
    Tmpfile(Tmpfile, Claimant),
    Preseed(Preseed, Claimant),
    Parameter(Parameter, Claimant),
}
impl Row {
    pub(super) fn source(&self) -> &Claimant {
        match self {
            Self::User(user, _) => &user.source,
            Self::Group(group, _) => &group.source,
            Self::Preset(_, source)
            | Self::Tmpfile(_, source)
            | Self::Preseed(_, source)
            | Self::Parameter(_, source) => source,
        }
    }
    pub(super) const fn subject(&self) -> Subject {
        match self {
            Self::User(..) => Subject::User,
            Self::Group(..) => Subject::Group,
            Self::Preset(..) => Subject::Preset,
            Self::Tmpfile(..) => Subject::Tmpfiles,
            Self::Preseed(..) => Subject::Preseed,
            Self::Parameter(..) => Subject::Parameter,
        }
    }
    pub(super) const fn explicit(&self) -> bool {
        !matches_group(self)
    }
    pub(super) fn same(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::User(a, ap), Self::User(b, bp)) => {
                a.value.kind == b.value.kind
                    && a.subordinate == b.subordinate
                    && a.member_of == b.member_of
                    && a.package_ids == b.package_ids
                    && ap == bp
            }
            (Self::Group(_, false), Self::Group(_, false)) => true,
            (Self::Group(a, _), Self::Group(b, _)) => a.value.kind == b.value.kind && a.package_gid == b.package_gid,
            (Self::Preset(a, _), Self::Preset(b, _)) => a == b,
            (Self::Tmpfile(a, _), Self::Tmpfile(b, _)) => a == b,
            (Self::Preseed(a, _), Self::Preseed(b, _)) => a == b,
            (Self::Parameter(a, _), Self::Parameter(b, _)) => a.order == b.order && a.param == b.param,
            (
                Self::User(..)
                | Self::Group(..)
                | Self::Preset(..)
                | Self::Tmpfile(..)
                | Self::Preseed(..)
                | Self::Parameter(..),
                _,
            ) => false,
        }
    }
}
const fn matches_group(row: &Row) -> bool {
    match row {
        Row::Group(_, explicit) => !*explicit,
        Row::User(..) | Row::Preset(..) | Row::Tmpfile(..) | Row::Preseed(..) | Row::Parameter(..) => false,
    }
}
pub(super) fn gather(rows: &mut Rows, declared: &Declared<'_>, errors: &mut Collector) {
    accounts(rows, declared, errors);
    for map in [Map::Presets, Map::Tmpfiles, Map::Preseeds, Map::Cmdline] {
        for located in &declared.expansion.of(map) {
            let row = match map {
                Map::Presets => {
                    let row = match kinds::presets::entry(located) {
                        Ok(row) => row,
                        Err(error) => {
                            fault(rows, *error, declared, errors);
                            continue;
                        }
                    };
                    Row::Preset(row, Claimant::new(located.task, located.entry.at))
                }
                Map::Tmpfiles => {
                    let row = match kinds::tmpfiles::entry(located) {
                        Ok(row) => row,
                        Err(error) => {
                            fault(rows, *error, declared, errors);
                            continue;
                        }
                    };
                    Row::Tmpfile(row, Claimant::new(located.task, located.entry.at))
                }
                Map::Preseeds => {
                    let Ok(row) = kinds::preseeds::entry(located.entry) else { continue };
                    Row::Preseed(row, Claimant::new(located.task, located.entry.at))
                }
                Map::Cmdline => {
                    let Ok(row) = kinds::cmdline::entry(located.entry) else { continue };
                    Row::Parameter(row, Claimant::new(located.task, located.entry.at))
                }
                Map::Packages
                | Map::Initramfs
                | Map::Software
                | Map::Files
                | Map::Users
                | Map::Groups
                | Map::Data
                | Map::Secrets
                | Map::Identity
                | Map::Live => unreachable!("declaration kinds"),
            };
            rows.insert(&located.entry.key, row, declared.target, errors);
        }
    }
    let located = declared.expansion.of(Map::Data);
    let mut data = List::with_capacity(u32::try_from(located.len()).expect("expansion bound"));
    for located in &located {
        if let Ok(entry) = kinds::data::entry(located) {
            data.push(entry).expect("expansion bound");
        }
    }
    for line in crate::additions::data_lines(data.as_slice()) {
        rows.insert(&line.tmpfile.path.clone(), Row::Tmpfile(line.tmpfile, line.source), declared.target, errors);
    }
}
fn fault(rows: &mut Rows, mut error: crate::errors::Error, declared: &Declared<'_>, errors: &mut Collector) {
    rows.valid = false;
    error.target = Some(copy_of(declared.target));
    errors.push(error);
}

fn accounts(rows: &mut Rows, declared: &Declared<'_>, errors: &mut Collector) {
    let users = declared.expansion.of(Map::Users);
    let mut passwords = skein_lib::Map::with_capacity(rows.rows.capacity());
    for located in &users {
        if let Some(password) = located.entry.text(Name::Password)
            && passwords.insert((located.task, located.entry.key.as_ref()), password).is_err()
        {
            rows.valid = false;
            errors.push(super::error(
                declared.target,
                crate::errors::Kind::Evaluation(crate::evaluate::FaultKind::Limit),
                &Claimant::new(located.task, located.entry.at),
                None,
            ));
        }
    }
    for user in &declared.accounts.users {
        let mut password = None;
        if let Some(secret) = passwords.get(&(user.source.task.as_ref(), user.value.name.as_ref())) {
            password = Some(copy_of(secret));
        }
        rows.insert(&user.value.name, Row::User(user.clone(), password), declared.target, errors);
        if let UserKind::Sysusers { uid, group, .. } = &user.value.kind
            && *group == user.value.name
        {
            rows.insert(
                &user.value.name,
                Row::Group(
                    Group {
                        value: config::Group { name: user.value.name.clone(), kind: GroupKind::Sysusers { gid: *uid } },
                        source: user.source.clone(),
                        package_gid: None,
                    },
                    false,
                ),
                declared.target,
                errors,
            );
        }
    }
    for group in &declared.accounts.groups {
        rows.insert(&group.value.name, Row::Group(group.clone(), true), declared.target, errors);
    }
}
