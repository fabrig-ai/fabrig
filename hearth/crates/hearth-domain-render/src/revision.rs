//! The listing and typed documents (hearth/docs/design/protocol/format/README.md, 2.1 and 3.4).
use crate::errors::Repository;
use alloc::boxed::Box;
pub use hearth_domain_render_vocabulary::revision::{Firmware, Mode, Oid, Position, Runs};
pub mod declaration;
pub(crate) mod held;
use skein_lib::{Bisection, bytes};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Entry {
    pub path: Box<[u8]>,
    pub mode: Mode,
    pub oid: Oid,
    pub size: u64,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Class<'a> {
    Fleet,
    Environment { env: &'a [u8] },
    Host { base: &'a [u8], name: &'a [u8] },
    Declaration { task: &'a [u8] },
    Tree { task: &'a [u8], path: &'a [u8] },
    Template { task: &'a [u8], path: &'a [u8] },
    Script { task: &'a [u8], name: &'a [u8] },
    Readme { task: &'a [u8] },
    Refused { task: Option<&'a [u8]>, why: Repository },
    Other,
}
#[must_use]
pub fn class(path: &[u8]) -> Class<'_> {
    for chunk in path.utf8_chunks() {
        if !chunk.invalid().is_empty() {
            return Class::Refused { task: task(path), why: Repository::Name };
        }
    }
    if path == b"fleet.json" {
        return Class::Fleet;
    }
    if let Some(rest) = path.strip_prefix(b"envs/") {
        if let Some(env) = rest.strip_suffix(b".json")
            && name(env)
        {
            return Class::Environment { env };
        }
        return Class::Refused { task: None, why: Repository::Name };
    }
    if let Some(rest) = path.strip_prefix(b"hosts/") {
        let Some((base, file)) = split(rest) else {
            return Class::Refused { task: None, why: Repository::Stray };
        };
        if !name(base) {
            return Class::Refused { task: None, why: Repository::Name };
        }
        if !file.contains(&b'/')
            && let Some(host) = file.strip_suffix(b".json")
        {
            if host == base || environment(host, base) {
                return Class::Host { base, name: host };
            }
            return Class::Refused { task: None, why: Repository::Stray };
        }
        return Class::Other;
    }
    if let Some(rest) = path.strip_prefix(b"tasks/") {
        let Some((task, file)) = split(rest) else {
            return Class::Refused { task: Some(rest), why: Repository::Declaration };
        };
        if !identifier(task) {
            return Class::Refused { task: Some(task), why: Repository::Name };
        }
        if file == b"task.json" {
            return Class::Declaration { task };
        }
        if file == b"README.md" {
            return Class::Readme { task };
        }
        if let Some(path) = file.strip_prefix(b"tree/")
            && !path.is_empty()
        {
            if path == b".j2" || path.ends_with(b"/.j2") {
                return Class::Refused { task: Some(task), why: Repository::Name };
            }
            return Class::Tree { task, path };
        }
        if let Some(path) = file.strip_prefix(b"templates/")
            && path.ends_with(b".j2")
            && path != b".j2"
            && !path.ends_with(b"/.j2")
        {
            return Class::Template { task, path };
        }
        if [b"customize.sh".as_slice(), b"customize.sh.j2", b"software.sh", b"software.sh.j2"].contains(&file) {
            return Class::Script { task, name: file };
        }
        return Class::Refused { task: Some(task), why: Repository::Stray };
    }
    Class::Other
}
fn split(path: &[u8]) -> Option<(&[u8], &[u8])> {
    let (head, tail) = path.split_at_checked(bytes::find(path, b"/")?)?;
    Some((head, tail.strip_prefix(b"/")?))
}
fn environment(host: &[u8], base: &[u8]) -> bool {
    let Some(suffix) = host.strip_prefix(base) else {
        return false;
    };
    let Some(env) = suffix.strip_prefix(b"-") else {
        return false;
    };
    name(env)
}
fn name(name: &[u8]) -> bool {
    let Some(first) = name.first() else {
        return false;
    };
    if (!first.is_ascii_lowercase() && !first.is_ascii_digit()) || name.ends_with(b"-") {
        return false;
    }
    for byte in name {
        if !byte.is_ascii_lowercase() && !byte.is_ascii_digit() && *byte != b'-' {
            return false;
        }
    }
    true
}
fn identifier(name: &[u8]) -> bool {
    let Some(first) = name.first() else {
        return false;
    };
    if !first.is_ascii_lowercase() {
        return false;
    }
    for byte in name {
        if !byte.is_ascii_lowercase() && !byte.is_ascii_digit() && *byte != b'_' {
            return false;
        }
    }
    true
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Document {
    pub at: Position,
    pub layer: Layer,
    pub members: Option<Members>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Members {
    pub at: Position,
    pub each: Box<[Member]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Member {
    pub name: Box<[u8]>,
    pub at: Position,
    pub layer: Option<Layer>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Layer {
    pub debian: Option<Value>,
    pub placement: Option<Placement>,
    pub tasks: Box<[Task]>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Task {
    pub name: Box<[u8]>,
    pub at: Position,
    pub value: Value,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Placement {
    pub at: Position,
    pub kind: Runs,
    pub firmware: Option<Firmware>,
    pub hardware: Option<Box<[u8]>>,
    pub on: Option<On>,
    pub mutable: bool,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct On {
    pub target: Box<[u8]>,
    pub at: Position,
}

/// Decoded configuration values with positions (hearth/docs/design/domain/render/README.md, 3.1).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Value {
    pub at: Position,
    pub kind: Kind,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub enum Kind {
    Null,
    Bool(bool),
    Int(i64),
    String(Box<[u8]>),
    List(Box<[Value]>),
    Map(Box<[Pair]>),
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Pair {
    pub key: Box<[u8]>,
    pub at: Position,
    pub value: Value,
}
/// Documents in listing path order, including refused documents.
#[derive(Debug, Hash)]
pub struct Documents {
    entries: Box<[Entry]>,
    decoded: Box<[Option<Document>]>,
    declared: Box<[Option<declaration::Declaration>]>,
}
impl Documents {
    #[must_use]
    pub fn new(entries: &[Entry]) -> Self {
        let capacity = u32::try_from(entries.len()).expect("bounded listing");
        let mut listed = skein_lib::Map::with_capacity(capacity);
        let mut decoded = skein_lib::List::with_capacity(capacity);
        for entry in entries {
            if entry.mode == Mode::Link || entry.mode == Mode::Submodule {
                continue;
            }
            match class(&entry.path) {
                Class::Fleet | Class::Environment { .. } | Class::Host { .. } | Class::Declaration { .. } => {
                    listed.insert(entry.path.clone(), entry.clone()).expect("listing capacity");
                    decoded.push(None).expect("listing capacity");
                }
                Class::Tree { .. }
                | Class::Template { .. }
                | Class::Script { .. }
                | Class::Readme { .. }
                | Class::Refused { .. }
                | Class::Other => {}
            }
        }
        let mut entries = skein_lib::List::with_capacity(listed.len());
        for (_, entry) in &listed {
            entries.push(entry.clone()).expect("listing capacity");
        }
        let mut declared = skein_lib::List::with_capacity(listed.len());
        for _ in 0..listed.len() {
            declared.push(None).expect("listing capacity");
        }
        Self { entries: entries.into_boxed(), decoded: decoded.into_boxed(), declared: declared.into_boxed() }
    }
    #[must_use]
    pub fn path(&self, document: u32) -> &[u8] {
        &self.entries.get(usize::try_from(document).expect("u32 fits")).expect("document index").path
    }
    #[must_use]
    pub fn get(&self, document: u32) -> Option<&Document> {
        self.decoded.get(usize::try_from(document).ok()?)?.as_ref()
    }
    pub fn set_declaration(&mut self, oid: &Oid, declaration: declaration::Declaration) {
        self.store(oid, crate::vocabulary::Decoded::Declaration(declaration));
    }
    #[must_use]
    pub fn declaration(&self, name: &[u8]) -> Option<(u32, Option<&declaration::Declaration>)> {
        // new's Map keeps paths sorted and unique. Only declarations remain under
        // tasks/, whose identifier bytes (lowercase, digits, _) all sort above /,
        // so their path order is also their task order.
        let mut search = Bisection::over(self.entries.len());
        for _ in 0..Bisection::PROBES {
            let Some(probe) = search.probe() else { break };
            let entry = self.entries.get(probe).expect("a probe within the slice");
            search.narrow(entry.path.as_ref().cmp(b"tasks/"));
        }
        let first = search.end();
        let entries = self.entries.get(first..)?;
        let mut search = Bisection::over(entries.len());
        for _ in 0..Bisection::PROBES {
            let Some(probe) = search.probe() else { break };
            let entry = entries.get(probe).expect("a probe within the slice");
            search.narrow(task(&entry.path).unwrap_or(b"").cmp(name));
        }
        if task(&entries.get(search.end())?.path) != Some(name) {
            return None;
        }
        let index = first.checked_add(search.end()).expect("bounded listing");
        Some((u32::try_from(index).expect("bounded listing"), self.declared.get(index)?.as_ref()))
    }
    pub fn set(&mut self, oid: &Oid, document: Document) {
        self.store(oid, crate::vocabulary::Decoded::Document(document));
    }
    fn store(&mut self, oid: &Oid, value: crate::vocabulary::Decoded) {
        use crate::vocabulary::Decoded;
        let declared = match &value {
            Decoded::Document(_) => false,
            Decoded::Declaration(_) => true,
            Decoded::Link(_) | Decoded::Template(_) | Decoded::Refused { .. } => unreachable!("document input"),
        };
        let last = self.last_copy(oid, declared);
        let mut source = Some(value);
        for (index, entry) in self.entries.iter().enumerate() {
            if &entry.oid != oid || task(&entry.path).is_some() != declared {
                continue;
            }
            let value = if Some(index) == last {
                source.take().expect("last copy remains")
            } else {
                source.as_ref().expect("last copy remains").clone()
            };
            match value {
                Decoded::Document(value) => *self.decoded.get_mut(index).expect("document index") = Some(value),
                Decoded::Declaration(value) => *self.declared.get_mut(index).expect("document index") = Some(value),
                Decoded::Link(_) | Decoded::Template(_) | Decoded::Refused { .. } => unreachable!("document input"),
            }
        }
    }
    pub(crate) fn take(&mut self, index: usize) -> Option<crate::vocabulary::Decoded> {
        if let Some(document) = self.decoded.get_mut(index)?.take() {
            return Some(crate::vocabulary::Decoded::Document(document));
        }
        Some(crate::vocabulary::Decoded::Declaration(self.declared.get_mut(index)?.take()?))
    }
    fn last_copy(&self, oid: &Oid, declared: bool) -> Option<usize> {
        let mut last = None;
        for (index, entry) in self.entries.iter().enumerate() {
            if &entry.oid == oid && task(&entry.path).is_some() == declared {
                last = Some(index);
            }
        }
        last
    }
    #[must_use]
    pub fn entries(&self) -> &[Entry] {
        &self.entries
    }
    #[must_use]
    pub fn find(&self, path: &[u8]) -> Option<u32> {
        let mut search = Bisection::over(self.entries.len());
        for _ in 0..Bisection::PROBES {
            let Some(probe) = search.probe() else { break };
            let entry = self.entries.get(probe).expect("a probe within the slice");
            search.narrow(entry.path.as_ref().cmp(path));
        }
        if self.entries.get(search.end())?.path.as_ref() == path { u32::try_from(search.end()).ok() } else { None }
    }
}
/// Retained decoded bytes admitted on arrival (hearth/docs/design/domain/render/README.md, 2.3).
#[derive(Debug, Hash)]
pub(crate) struct Budget {
    held: u64,
    max: u64,
    exceeded: bool,
}
impl Budget {
    #[must_use]
    pub const fn new(limits: &crate::limits::Limits) -> Self {
        Self { held: 0, max: limits.decoded, exceeded: false }
    }
    #[must_use]
    pub const fn exceeded(&self) -> bool {
        self.exceeded
    }
    #[must_use]
    pub(crate) const fn remaining(&self) -> u64 {
        self.max.saturating_sub(self.held)
    }
    #[must_use]
    pub fn take(&mut self, size: u64) -> bool {
        if self.exceeded {
            return false;
        }
        let Some(held) = self.held.checked_add(size) else {
            self.exceeded = true;
            return false;
        };
        if held > self.max {
            self.exceeded = true;
            return false;
        }
        self.held = held;
        true
    }
}

#[must_use]
pub(crate) fn task(path: &[u8]) -> Option<&[u8]> {
    let path = path.strip_prefix(b"tasks/")?;
    Some(match split(path) {
        Some((task, _)) => task,
        None => path,
    })
}

#[must_use]
pub(crate) fn base(path: &[u8]) -> Option<&[u8]> {
    Some(split(path.strip_prefix(b"hosts/")?)?.0)
}

/// The logical bytes retained from one decoded input (hearth/docs/design/domain/render/README.md, 2.3).
#[must_use]
pub fn decoded_size(read: &crate::vocabulary::Decoded, limits: &crate::limits::Limits) -> u64 {
    match read {
        crate::vocabulary::Decoded::Document(document) => held::document(document, limits),
        crate::vocabulary::Decoded::Declaration(declaration) => held::declaration::declaration(declaration, limits),
        crate::vocabulary::Decoded::Template(template) => held::template::template(template),
        crate::vocabulary::Decoded::Link(_) | crate::vocabulary::Decoded::Refused { .. } => 0,
    }
}
