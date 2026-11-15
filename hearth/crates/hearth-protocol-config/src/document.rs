//! Configuration layers (hearth/docs/design/protocol/format/README.md, 3.1 to 3.6).
use crate::{
    Decode, Error, Fault, Field, Position,
    json::{Entry, Kind, Value},
    names,
};
use alloc::boxed::Box;
pub use hearth_domain_render_vocabulary::revision::{Firmware, Runs};
use skein_lib::List;
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Document {
    pub at: Position,
    pub layer: Layer,
    pub members: Option<Members>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Layer {
    pub tasks: Box<[Task]>,
    pub debian: Option<Value>,
    pub placement: Option<Placement>,
}
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Task {
    pub name: Box<[u8]>,
    pub at: Position,
    pub value: Value,
}
pub fn document(value: Value) -> Result<Document, Error> {
    let entries = match value.kind {
        Kind::Map(entries) => entries,
        Kind::Null | Kind::Bool(_) | Kind::Int(_) | Kind::List(_) | Kind::String(_) => {
            return Err(Error { at: value.at, fault: Fault::Decode(Decode::NotObject) });
        }
    };
    refuse_list_null(&entries)?;
    let mut members = None;
    let mut layer = empty_layer();
    let mut tasks = List::with_capacity(u32::try_from(entries.len()).expect("document bounded by u32"));
    for entry in entries {
        if &*entry.key == b"members" {
            members = Some(read_members(entry.value)?);
        } else {
            read_field(entry, &mut layer, &mut tasks)?;
        }
    }
    layer.tasks = tasks.into_boxed();
    Ok(Document { at: value.at, layer, members })
}
#[derive(Clone, Debug)]
enum Unread<'a> {
    List(core::slice::Iter<'a, Value>),
    Map(core::slice::Iter<'a, Entry>, bool),
}
fn refuse_list_null(entries: &[Entry]) -> Result<(), Error> {
    /// Steps of the walk: the decoder's length is a u32, each node takes a byte of it, and each is read once and
    /// closed once.
    const WALK_STEPS: u64 = 1 << 33;

    let mut pending = List::uncapped();
    pending.push(Unread::Map(entries.iter(), false)).expect("an uncapped list");
    let mut first: Option<Position> = None;
    for _ in 0..WALK_STEPS {
        let Some(index) = pending.len().checked_sub(1) else { break };
        let (value, inside) = match pending.get_mut(index).expect("top frame") {
            Unread::List(values) => {
                let Some(value) = values.next() else {
                    pending.truncate(index);
                    continue;
                };
                (value, true)
            }
            Unread::Map(entries, inside) => {
                let Some(entry) = entries.next() else {
                    pending.truncate(index);
                    continue;
                };
                (&entry.value, *inside)
            }
        };
        let unread = match &value.kind {
            Kind::Null => {
                let earlier = match first {
                    Some(at) => value.at.offset < at.offset,
                    None => true,
                };
                if inside && earlier {
                    first = Some(value.at);
                }
                continue;
            }
            Kind::List(values) => Unread::List(values.iter()),
            Kind::Map(entries) => Unread::Map(entries.iter(), inside),
            Kind::Bool(_) | Kind::Int(_) | Kind::String(_) => continue,
        };
        pending.push(unread).expect("an uncapped list");
    }
    // Maps are held in name order, so retain the earliest source position.
    match first {
        Some(at) => Err(decode(at, Decode::ListNull)),
        None => Ok(()),
    }
}
fn empty_layer() -> Layer {
    Layer { tasks: Box::new([]), debian: None, placement: None }
}
fn read_layer(entries: Box<[Entry]>) -> Result<Layer, Error> {
    let mut layer = empty_layer();
    let mut tasks = List::with_capacity(u32::try_from(entries.len()).expect("document bounded by u32"));
    for entry in entries {
        read_field(entry, &mut layer, &mut tasks)?;
    }
    layer.tasks = tasks.into_boxed();
    Ok(layer)
}
fn read_field(entry: Entry, layer: &mut Layer, tasks: &mut List<Task>) -> Result<(), Error> {
    match &*entry.key {
        b"debian" => layer.debian = Some(entry.value),
        b"placement" => layer.placement = Some(read_placement(entry.value)?),
        b"members" => return Err(decode(entry.at, Decode::Member)),
        _ => {
            if !names::identifier(&entry.key) {
                return Err(decode(entry.at, Decode::Name));
            }
            match &entry.value.kind {
                Kind::Map(_) | Kind::Null => {}
                Kind::Bool(_) | Kind::Int(_) | Kind::List(_) | Kind::String(_) => {
                    return Err(decode(entry.value.at, Decode::Settings { task: entry.key }));
                }
            }
            tasks.push(Task { name: entry.key, at: entry.at, value: entry.value }).expect("one task per entry");
        }
    }
    Ok(())
}
/// Members in name order, at the map's opening brace (hearth/docs/design/protocol/format/README.md, 3.6).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Members {
    pub at: Position,
    pub each: Box<[Member]>,
}
/// One member's layer, or a removal, at its name.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Member {
    pub name: Box<[u8]>,
    pub at: Position,
    pub layer: Option<Layer>,
}
fn read_members(value: Value) -> Result<Members, Error> {
    let Kind::Map(entries) = value.kind else {
        return Err(decode(value.at, Decode::Members));
    };
    let mut each = List::with_capacity(u32::try_from(entries.len()).expect("members bounded by u32"));
    for entry in entries {
        if !names::member(&entry.key) {
            return Err(decode(entry.at, Decode::Member));
        }
        let layer = match entry.value.kind {
            Kind::Null => None,
            Kind::Map(entries) => Some(read_layer(entries)?),
            Kind::Bool(_) | Kind::Int(_) | Kind::List(_) | Kind::String(_) => {
                return Err(decode(entry.value.at, Decode::Members));
            }
        };
        each.push(Member { name: entry.key, at: entry.at, layer }).expect("one member per entry");
    }
    Ok(Members { at: value.at, each: each.into_boxed() })
}

/// A whole placement, replaced by a later layer (hearth/docs/design/protocol/format/README.md, 3.4).
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Placement {
    pub at: Position,
    pub kind: Runs,
    pub firmware: Option<Firmware>,
    pub hardware: Option<Box<[u8]>>,
    pub on: Option<On>,
    pub mutable: bool,
}
/// A container's host, as written, at the target value.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct On {
    pub target: Box<[u8]>,
    pub at: Position,
}
fn decode(at: Position, fault: Decode) -> Error {
    Error { at, fault: Fault::Decode(fault) }
}
fn text(value: Value, field: Field) -> Result<Box<[u8]>, Error> {
    match value.kind {
        Kind::String(text) => Ok(text),
        Kind::Null | Kind::Bool(_) | Kind::Int(_) | Kind::List(_) | Kind::Map(_) => {
            Err(decode(value.at, Decode::Wrong { field }))
        }
    }
}
fn read_kind(value: &Value) -> Result<Runs, Error> {
    if let Kind::String(name) = &value.kind {
        match &**name {
            b"metal" => return Ok(Runs::Metal),
            b"vm" => return Ok(Runs::Vm),
            b"container" => return Ok(Runs::Container),
            _ => {}
        }
    }
    Err(decode(value.at, Decode::Wrong { field: Field::Kind }))
}
fn read_placement(value: Value) -> Result<Placement, Error> {
    let Kind::Map(entries) = value.kind else {
        return Err(decode(value.at, Decode::Placement));
    };
    let mut kind = None;
    for entry in &entries {
        if &*entry.key == b"kind" {
            kind = Some(read_kind(&entry.value)?);
        }
    }
    let kind = kind.ok_or(decode(value.at, Decode::Missing { field: Field::Kind }))?;
    let mut placement = Placement { at: value.at, kind, firmware: None, hardware: None, on: None, mutable: false };
    for entry in entries {
        let field = match &*entry.key {
            b"kind" => continue,
            b"firmware" => Field::Firmware,
            b"hardware" => Field::Hardware,
            b"on" => Field::On,
            b"mutable" if placement.kind != Runs::Container => Field::Mutable,
            _ => return Err(decode(entry.at, Decode::Unknown { field: entry.key })),
        };
        if (placement.kind == Runs::Container && (field == Field::Firmware || field == Field::Hardware))
            || (placement.kind != Runs::Container && field == Field::On)
        {
            return Err(decode(entry.at, Decode::Misplaced { field }));
        }
        match field {
            Field::Firmware => {
                let at = entry.value.at;
                placement.firmware = Some(match &*text(entry.value, field)? {
                    b"efi" => Firmware::Efi,
                    b"bios" => Firmware::Bios,
                    _ => return Err(decode(at, Decode::Wrong { field })),
                });
            }
            Field::Hardware => placement.hardware = Some(text(entry.value, field)?),
            Field::On => placement.on = Some(On { at: entry.value.at, target: text(entry.value, field)? }),
            Field::Mutable => {
                let Kind::Bool(mutable) = entry.value.kind else {
                    return Err(decode(entry.value.at, Decode::Wrong { field }));
                };
                placement.mutable = mutable;
            }
            Field::Kind => unreachable!("kind decoded before fields"),
        }
    }
    for (field, missing) in [
        (Field::Firmware, placement.kind != Runs::Container && placement.firmware.is_none()),
        (Field::Hardware, placement.kind != Runs::Container && placement.hardware.is_none()),
        (Field::On, placement.kind == Runs::Container && placement.on.is_none()),
    ] {
        if missing {
            return Err(decode(value.at, Decode::Missing { field }));
        }
    }
    Ok(placement)
}
