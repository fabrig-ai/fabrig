//! A path's drawing and textual fallback (temper/docs/design/client/README.md, 5.16).

use skein_lib::{List, Reader, Writer, usize_of};
use skein_ui::{Builder, DrawingKind, Role, Run};

mod layout;
pub use layout::draw;

/// Who acts at a stop, for the drawing alone.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Mark {
    Person,
    Agent,
    Code,
    Bar,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Stop<'a> {
    pub mark: Mark,
    pub words: &'a [u8],
}

/// temper's drawing kind for a path.
pub const KIND: DrawingKind = DrawingKind { owner: crate::frame::owner(), kind: 0 };
pub const STOPS: u32 = 32;
pub const WORDS: u32 = 64;
/// Count byte, then each stop's mark, byte length and words.
pub const DATA: u32 = 2113;
/// A line and boundary, then a mark and its words per stop.
pub const MARKS: u32 = 66;
/// At most 31 connecting bytes, a 42-byte star and 64 words bytes per stop, plus a move.
pub const SCENE_BYTES: u32 = 4389;

/// Refused before changing the builder; builder capacities remain its caller's contract.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Problem {
    Stops,
    Words,
}

/// Add a path to the tree: the drawing, with its fallback children.
pub fn path(builder: &mut Builder, label: &[u8], stops: &[Stop<'_>]) -> Result<(), Problem> {
    let data = encode(stops)?;
    builder.drawing(KIND, label, data.as_slice(), DATA);
    builder.open(Role::List);
    builder.name(label);
    for stop in stops {
        let actor: &[u8] = match stop.mark {
            Mark::Person => b", a person",
            Mark::Agent => b", an agent",
            Mark::Code => b", code",
            Mark::Bar => b", a branch",
        };
        let mut words = Writer::new(stop.words.len().checked_add(actor.len()).expect("bounded fallback words"));
        words.put(stop.words).expect("counted stop words");
        words.put(actor).expect("counted actor words");
        let words = words.finish();
        builder.open(Role::Item);
        builder.name(&words);
        builder.open(Role::Text);
        builder.text(Run::Plain, &words);
        builder.close();
        builder.close();
    }
    builder.close();
    builder.close();
    Ok(())
}

fn encode(stops: &[Stop<'_>]) -> Result<List<u8>, Problem> {
    if stops.is_empty() || stops.len() > usize_of(STOPS) {
        return Err(Problem::Stops);
    }
    let mut size = 1_usize;
    for stop in stops {
        if stop.words.is_empty() || stop.words.len() > usize_of(WORDS) {
            return Err(Problem::Words);
        }
        size = size.checked_add(2).expect("bounded stop header").checked_add(stop.words.len()).expect("bounded path");
    }
    let mut writer = Writer::new(size);
    writer.put(&[u8::try_from(stops.len()).expect("bounded stop count")]).expect("counted stop count");
    for stop in stops {
        let mark = match stop.mark {
            Mark::Person => 0,
            Mark::Agent => 1,
            Mark::Code => 2,
            Mark::Bar => 3,
        };
        writer.put(&[mark, u8::try_from(stop.words.len()).expect("bounded words")]).expect("counted stop header");
        writer.put(stop.words).expect("counted stop words");
    }
    Ok(List::from_boxed(writer.finish()))
}

fn decode(data: &[u8]) -> Option<List<Stop<'_>>> {
    if data.len() > usize_of(DATA) {
        return None;
    }
    let mut reader = Reader::new(data);
    let count = u32::from(reader.u8()?);
    if count == 0 || count > STOPS {
        return None;
    }
    let mut stops = List::with_capacity(count);
    for _ in 0..count {
        let tag = reader.u8()?;
        let mark = if tag == 0 {
            Mark::Person
        } else if tag == 1 {
            Mark::Agent
        } else if tag == 2 {
            Mark::Code
        } else if tag == 3 {
            Mark::Bar
        } else {
            return None;
        };
        let length = u32::from(reader.u8()?);
        if length == 0 || length > WORDS {
            return None;
        }
        stops.push(Stop { mark, words: reader.bytes(length)? }).ok()?;
    }
    reader.is_empty().then_some(stops)
}

/// temper's draw: its own kind, else skein's flows.
pub fn draw_any(kind: DrawingKind, data: &[u8], width: u16, height: u16, scene: &mut skein_ui::scene::Scene) -> bool {
    if kind.owner == crate::frame::owner() {
        draw(kind, data, width, height, scene)
    } else {
        skein_ui::flow::draw(kind, data, width, height, scene)
    }
}
