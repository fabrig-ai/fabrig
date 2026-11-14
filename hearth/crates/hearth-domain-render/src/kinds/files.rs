//! One source per destination and directory governance (hearth/docs/design/protocol/format/tasks.md, 5).
use crate::{
    entries::{Concrete, Fault, Located},
    errors::{Format, Kind, Repository, Schema, TypeName},
    revision::{
        Position,
        declaration::{Declaration, Interpolated, Map, Name},
    },
    schema::forms,
};
use alloc::boxed::Box;
use hearth_lib::encode::Digest;
use skein_lib::{List, Map as Ordered, bytes::copy_of};
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct Plan {
    pub task: Box<[u8]>,
    pub installs: Box<[Install]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct Install {
    pub path: Box<[u8]>,
    pub at: Option<Position>,
    pub make: Make,
    pub owner: Box<[u8]>,
    pub group: Box<[u8]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) enum Make {
    Static { sha256: Digest, mode: u16 },
    Link { target: Box<[u8]> },
    Directory { mode: u16 },
    Rendered { template: Box<[u8]>, via: Option<Via>, mode: u16 },
    Copied { source: Box<[u8]>, mode: u16 },
}
/// The concrete entry and element retained by expansion.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub(crate) struct Via {
    pub entry: u32,
    pub element: Option<u32>,
}
#[must_use]
pub(crate) fn link(target: &[u8]) -> bool {
    !target.is_empty() && target.len() <= 4095 && !target.contains(&0)
}
#[must_use]
pub(crate) fn dropped(declaration: &Declaration, kept: &[Located<'_>]) -> Box<[Box<[u8]>]> {
    let mut capacity = 0_usize;
    for group in &declaration.entries {
        capacity = capacity.saturating_add(group.entries.len());
    }
    let mut paths = List::with_capacity(u32::try_from(capacity).expect("declaration bound"));
    for group in &declaration.entries {
        if group.map != Map::Files {
            continue;
        }
        for entry in &group.entries {
            if entry.for_each.is_some() {
                continue;
            }
            match &entry.key {
                Interpolated::Plain { text, .. } if !contains(kept, text) => {
                    paths.push(text.clone()).expect("declaration bound");
                }
                Interpolated::Plain { .. } | Interpolated::Parts { .. } => {}
            }
        }
    }
    paths.into_boxed()
}
pub(crate) fn plan(
    task: &[u8],
    tree: &[Install],
    files: &[Located<'_>],
    dropped: &[Box<[u8]>],
    faults: &mut List<Fault>,
) -> Plan {
    let capacity = u32::try_from(tree.len().saturating_add(files.len())).expect("listing and expansion bound");
    let mut installs = Ordered::with_capacity(capacity);
    for file in tree {
        if excluded(&file.path, dropped) {
            continue;
        }
        installs.insert(file.path.clone(), file.clone()).expect("plan capacity");
    }
    for (index, row) in files.iter().enumerate() {
        if row.task != task || excluded(&row.entry.key, dropped) {
            continue;
        }
        match install(row.entry, tree, u32::try_from(index).expect("expansion bound")) {
            Ok(install) => {
                installs.insert(install.path.clone(), install).expect("plan capacity");
            }
            Err(fault) => faults.push(fault).expect("fault capacity"),
        }
    }
    let mut result = List::with_capacity(capacity);
    for (_, install) in &installs {
        let mut install = install.clone();
        install.owner = owner(&install.path, files, Name::Owner);
        install.group = owner(&install.path, files, Name::Group);
        result.push(install).expect("plan capacity");
    }
    Plan { task: copy_of(task), installs: result.into_boxed() }
}
fn excluded(path: &[u8], dropped: &[Box<[u8]>]) -> bool {
    for key in dropped {
        if key.as_ref() == path || (key.ends_with(b"/") && path.starts_with(key)) {
            return true;
        }
    }
    false
}
fn owner(path: &[u8], rows: &[Located<'_>], field: Name) -> Box<[u8]> {
    let mut length = 0;
    let mut value = b"root".as_slice();
    for row in rows {
        let key = row.entry.key.as_ref();
        if key.len() >= length
            && (key.strip_suffix(b"/").unwrap_or(key) == path || (key.ends_with(b"/") && path.starts_with(key)))
            && let Some(given) = row.entry.text(field)
        {
            length = key.len();
            value = given;
        }
    }
    copy_of(value)
}
fn install(entry: &Concrete, tree: &[Install], index: u32) -> Result<Install, Fault> {
    let path = entry.key.as_ref();
    let directory = path.ends_with(b"/");
    let checked = if directory { path.strip_suffix(b"/").expect("suffix") } else { path };
    if !forms::check(Format::Path, checked) {
        return Err(schema(Schema::Malformed { format: Format::Path }, entry.at));
    }
    let source = source(entry, directory)?;
    let mut tree_file = None;
    let mut subtree = false;
    for file in tree {
        if file.path.as_ref() == path {
            tree_file = Some(file);
        }
        subtree |= file.path.starts_with(path);
    }
    let make = match source {
        Some(Name::Directory) => {
            if entry.bool(Name::Directory) != Some(true) {
                return Err(schema(Schema::Wrong { expected: TypeName::Bool }, field_at(entry, Name::Directory)));
            }
            Make::Directory { mode: mode(entry, 0o755)? }
        }
        Some(Name::Link) => {
            if let Some(field) = field(entry, Name::Mode) {
                return Err(undeclared(Name::Mode, field.at));
            }
            let target = entry.text(Name::Link).expect("typed link");
            if !link(target) {
                return Err(schema(Schema::Malformed { format: Format::Link }, field_at(entry, Name::Link)));
            }
            Make::Link { target: copy_of(target) }
        }
        Some(Name::Template) => Make::Rendered {
            template: copy_of(entry.text(Name::Template).expect("typed template")),
            via: Some(Via { entry: index, element: entry.element }),
            mode: mode(entry, 0o644)?,
        },
        Some(Name::Copy) => {
            Make::Copied { source: copy_of(entry.text(Name::Copy).expect("typed copy")), mode: mode(entry, 0o644)? }
        }
        Some(
            Name::Action
            | Name::Age
            | Name::Algorithm
            | Name::Argument
            | Name::Base
            | Name::Class
            | Name::Component
            | Name::Count
            | Name::From
            | Name::Gecos
            | Name::Gid
            | Name::Group
            | Name::Groups
            | Name::Home
            | Name::Kind
            | Name::Mode
            | Name::Order
            | Name::Owner
            | Name::Param
            | Name::Password
            | Name::Path
            | Name::Paths
            | Name::PublicPath
            | Name::Sha256
            | Name::Shell
            | Name::Strip
            | Name::Subordinate
            | Name::Type
            | Name::Uid
            | Name::Unit
            | Name::Unpack
            | Name::Url
            | Name::Value,
        ) => unreachable!("source names"),
        None if directory && subtree => Make::Directory { mode: mode(entry, 0o755)? },
        None => {
            let Some(file) = tree_file else { return Err(repository(Repository::NoTree, entry.at)) };
            let mut make = file.make.clone();
            match &mut make {
                Make::Static { mode: default, .. }
                | Make::Rendered { mode: default, .. }
                | Make::Copied { mode: default, .. }
                | Make::Directory { mode: default } => *default = mode(entry, *default)?,
                Make::Link { .. } => {
                    if let Some(field) = field(entry, Name::Mode) {
                        return Err(undeclared(Name::Mode, field.at));
                    }
                }
            }
            make
        }
    };
    if source.is_some() && tree_file.is_some() {
        return Err(repository(Repository::Destination, entry.at));
    }
    Ok(Install { path: copy_of(checked), at: Some(entry.at), make, owner: copy_of(b"root"), group: copy_of(b"root") })
}
fn mode(entry: &Concrete, default: u16) -> Result<u16, Fault> {
    let Some(text) = entry.text(Name::Mode) else {
        return Ok(default);
    };
    if !forms::check(Format::Mode, text) {
        return Err(schema(Schema::Malformed { format: Format::Mode }, field_at(entry, Name::Mode)));
    }
    Ok(forms::octal(text))
}

fn schema(kind: Schema, at: Position) -> Fault {
    Fault { kind: Kind::Schema(kind), at, also: None }
}
fn repository(kind: Repository, at: Position) -> Fault {
    Fault { kind: Kind::Repository(kind), at, also: None }
}
fn undeclared(name: Name, at: Position) -> Fault {
    schema(Schema::UndeclaredField { field: copy_of(name.key()), declared: Box::new([]) }, at)
}

fn contains(rows: &[Located<'_>], key: &[u8]) -> bool {
    for row in rows {
        if row.entry.key.as_ref() == key {
            return true;
        }
    }
    false
}
fn field(entry: &Concrete, name: Name) -> Option<&crate::entries::Made> {
    let mut found = None;
    for field in &entry.fields {
        if field.name == name {
            found = Some(field);
        }
    }
    found
}

fn field_at(entry: &Concrete, name: Name) -> Position {
    field(entry, name).expect("present field").at
}

fn source(entry: &Concrete, directory: bool) -> Result<Option<Name>, Fault> {
    let mut source = None;
    for field in &entry.fields {
        if [Name::Template, Name::Copy, Name::Link, Name::Directory].contains(&field.name) {
            if directory {
                return Err(undeclared(field.name, field.at));
            }
            if let Some(first) = source {
                return Err(schema(Schema::Source { first, second: field.name }, field.at));
            }
            source = Some(field.name);
        }
    }
    Ok(source)
}
