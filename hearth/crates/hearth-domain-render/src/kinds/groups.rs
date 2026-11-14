//! Group declarations and bounded account ids (hearth/docs/design/protocol/format/tasks.md, 6).
use super::secrets::fault;
use crate::{
    entries::{Claimant, Concrete, Located},
    errors::{Error, Schema},
    parts::config::{self, GroupKind},
    revision::declaration::Name,
};
use alloc::boxed::Box;
use skein_lib::bytes::copy_of;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct Group {
    pub value: config::Group,
    pub source: Claimant,
    pub package_gid: Option<u32>,
}
impl Group {
    pub(crate) const fn gid(&self) -> Option<u32> {
        match self.value.kind {
            GroupKind::Sysusers { gid } => gid,
            GroupKind::Package => self.package_gid,
        }
    }
}
pub(crate) fn entry(located: &Located<'_>) -> Result<Group, Box<Error>> {
    match checked(located) {
        Ok(group) => Ok(group),
        Err((name, schema)) => Err(Box::new(fault(located, Some(name), schema))),
    }
}
fn checked(located: &Located<'_>) -> Result<Group, (Name, Schema)> {
    let package = package(located.entry)?;
    let gid = id(located.entry, Name::Gid)?;
    Ok(Group {
        value: config::Group {
            name: located.entry.key.clone(),
            kind: if package { GroupKind::Package } else { GroupKind::Sysusers { gid } },
        },
        source: Claimant::new(located.task, located.entry.at),
        package_gid: if package { gid } else { None },
    })
}

pub(super) fn package(entry: &Concrete) -> Result<bool, (Name, Schema)> {
    match entry.text(Name::From).unwrap_or(b"sysusers") {
        b"sysusers" => Ok(false),
        b"package" => Ok(true),
        value => Err((
            Name::From,
            Schema::Enum { value: copy_of(value), listed: Box::new([copy_of(b"sysusers"), copy_of(b"package")]) },
        )),
    }
}
pub(super) fn bounded(value: i64, min: i64, max: i64) -> Result<u32, Schema> {
    if value < min {
        Err(Schema::Below { min })
    } else if value > max {
        Err(Schema::Above { max })
    } else {
        Ok(u32::try_from(value).expect("bounded id"))
    }
}
pub(super) fn id(entry: &Concrete, name: Name) -> Result<Option<u32>, (Name, Schema)> {
    let Some(value) = entry.int(name) else { return Ok(None) };
    match bounded(value, 1, 65533) {
        Ok(value) => Ok(Some(value)),
        Err(schema) => Err((name, schema)),
    }
}
