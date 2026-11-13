//! A path's drawing and textual fallback (temper/docs/design/client/pages/chat/conversation.md, 3).

use skein_lib::{List, Reader, Writer, usize_of};
use skein_ui::{Binding, Builder, DrawingKind, Role, Run, State};

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
    pub needed: bool,
    pub over: &'a [u8],
}

/// A path whose stops are pressed (temper/docs/design/client/pages/chat/conversation.md, 3).
#[derive(Debug)]
pub struct Pressed<'a> {
    /// Reports a drawing act whose data is the stop's index as one byte.
    pub drawing: Binding,
    /// One fallback binding per stop, with None for a bar.
    pub stops: &'a [Option<Binding>],
    /// The stop whose card is open.
    pub open: Option<u32>,
}

/// The stop a drawing act names, encoded as exactly one byte.
#[must_use]
pub fn pressed_stop(data: &[u8]) -> Option<u32> {
    if data.len() != 1 {
        return None;
    }
    Some(u32::from(*data.first()?))
}

/// temper's drawing kind for a path.
pub const KIND: DrawingKind = DrawingKind { owner: crate::look::owner(), kind: 0 };
pub const STOPS: u32 = 32;
pub const WORDS: u32 = 64;
pub const OVER: u32 = 16;
/// Count, flags and open stop (3), then 32 × (mark, needed, words length,
/// 64 words bytes, over length, 16 over bytes): 3 + 32 × 84 = 2691.
pub const DATA: u32 = 2691;
/// Boundary, base and two rises (4), five marks per pressed stop (32 × 5),
/// one halo and one caret: 4 + 160 + 2 = 166.
pub const MARKS: u32 = 166;
/// 31 joins × (two whole cubics (18 each) + moved base segment (10)),
/// 32 × (star (42), words (64), over (16), two acts (1 each)), caret (12)
/// and two fades (`scene::fade_bytes(3)` = 20 each): 1426 + 3968 + 12 + 40 = 5446.
pub const SCENE_BYTES: u32 = 5406_u32
    .checked_add(skein_ui::scene::fade_bytes(3).expect("three fade stops").checked_mul(2).expect("two rises"))
    .expect("bounded path scene");

/// Refused before changing the builder; builder capacities remain its caller's contract.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Problem {
    Stops,
    Words,
    Over,
}

/// Add a path to the tree: the drawing, with its fallback children.
pub fn path(
    builder: &mut Builder,
    label: &[u8],
    stops: &[Stop<'_>],
    pressed: Option<&Pressed<'_>>,
) -> Result<(), Problem> {
    let data = encode(stops, pressed)?;
    builder.drawing(KIND, label, data.as_slice(), DATA);
    builder.style(crate::look::WAYS_PATH);
    if let Some(pressed) = pressed {
        assert_eq!(pressed.stops.len(), stops.len(), "one fallback binding per stop");
        builder.bind(pressed.drawing);
    }
    builder.open(Role::List);
    builder.name(label);
    for (index, stop) in stops.iter().enumerate() {
        let actor: &[u8] = match stop.mark {
            Mark::Person => b", a person",
            Mark::Agent => b", an agent",
            Mark::Code => b", code",
            Mark::Bar => b", a branch",
        };
        let needed: &[u8] = if stop.needed { b", when needed" } else { b"" };
        let over_room = if stop.over.is_empty() { 0 } else { stop.over.len().checked_add(3).expect("bounded over") };
        let size = stop
            .words
            .len()
            .checked_add(actor.len())
            .expect("bounded actor")
            .checked_add(needed.len())
            .expect("bounded needed")
            .checked_add(over_room)
            .expect("bounded fallback words");
        let mut words = Writer::new(size);
        words.put(stop.words).expect("counted stop words");
        words.put(actor).expect("counted actor words");
        words.put(needed).expect("counted needed words");
        if !stop.over.is_empty() {
            words.put(b" (").expect("counted opening bracket");
            words.put(stop.over).expect("counted over words");
            words.put(b")").expect("counted closing bracket");
        }
        let words = words.finish();
        builder.open(Role::Item);
        builder.name(&words);
        let button = pressed.is_some() && stop.mark != Mark::Bar;
        if button {
            let pressed = pressed.expect("pressed fallback");
            builder.open(Role::Button);
            builder.name(&words);
            builder.bind(pressed.stops.get(index).copied().flatten().expect("a non-bar stop has a binding"));
            if pressed.open == Some(u32::try_from(index).expect("bounded stop index")) {
                builder.state(State::Expanded);
            }
        }
        builder.open(Role::Text);
        builder.text(Run::Plain, &words);
        builder.close();
        if button {
            builder.close();
        }
        builder.close();
    }
    builder.close();
    builder.close();
    Ok(())
}

fn encode(stops: &[Stop<'_>], pressed: Option<&Pressed<'_>>) -> Result<List<u8>, Problem> {
    if stops.is_empty() || stops.len() > usize_of(STOPS) {
        return Err(Problem::Stops);
    }
    let mut size = 3_usize;
    for stop in stops {
        if stop.words.is_empty() || stop.words.len() > usize_of(WORDS) {
            return Err(Problem::Words);
        }
        if stop.over.len() > usize_of(OVER) || (stop.mark == Mark::Bar && !stop.over.is_empty()) {
            return Err(Problem::Over);
        }
        if stop.mark == Mark::Bar && stop.needed {
            return Err(Problem::Stops);
        }
        size = size
            .checked_add(4)
            .expect("bounded stop header")
            .checked_add(stop.words.len())
            .expect("bounded words")
            .checked_add(stop.over.len())
            .expect("bounded path");
    }
    let mut writer = Writer::new(size);
    writer.put(&[u8::try_from(stops.len()).expect("bounded stop count")]).expect("counted stop count");
    let (flags, open) = match pressed {
        Some(pressed) => (1, open_stop(stops, pressed.open)),
        None => (0, None),
    };
    writer
        .put(&[
            flags,
            match open {
                Some(index) => u8::try_from(index).expect("bounded open stop"),
                None => 255,
            },
        ])
        .expect("counted state");
    for stop in stops {
        let mark = match stop.mark {
            Mark::Person => 0,
            Mark::Agent => 1,
            Mark::Code => 2,
            Mark::Bar => 3,
        };
        writer
            .put(&[mark, u8::from(stop.needed), u8::try_from(stop.words.len()).expect("bounded words")])
            .expect("counted stop header");
        writer.put(stop.words).expect("counted stop words");
        writer.put(&[u8::try_from(stop.over.len()).expect("bounded over")]).expect("counted over length");
        writer.put(stop.over).expect("counted over bytes");
    }
    Ok(List::from_boxed(writer.finish()))
}

fn open_stop(stops: &[Stop<'_>], open: Option<u32>) -> Option<u32> {
    let index = open?;
    let stop = stops.get(usize_of(index))?;
    (stop.mark != Mark::Bar).then_some(index)
}

struct Decoded<'a> {
    stops: List<Stop<'a>>,
    pressed: bool,
    open: Option<u32>,
}

fn decode(data: &[u8]) -> Option<Decoded<'_>> {
    if data.len() > usize_of(DATA) {
        return None;
    }
    let mut reader = Reader::new(data);
    let count = u32::from(reader.u8()?);
    if count == 0 || count > STOPS {
        return None;
    }
    let flags = reader.u8()?;
    let open = reader.u8()?;
    if flags > 1 || (open != 255 && (flags == 0 || u32::from(open) >= count)) {
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
        let needed = reader.u8()?;
        if needed > 1 || (mark == Mark::Bar && needed != 0) {
            return None;
        }
        let length = u32::from(reader.u8()?);
        if length == 0 || length > WORDS {
            return None;
        }
        let words = reader.bytes(length)?;
        let length = u32::from(reader.u8()?);
        if length > OVER || (mark == Mark::Bar && length != 0) {
            return None;
        }
        stops.push(Stop { mark, words, needed: needed == 1, over: reader.bytes(length)? }).ok()?;
    }
    let open = if open == 255 { None } else { Some(u32::from(open)) };
    if !reader.is_empty() || open_stop(stops.as_slice(), open) != open {
        return None;
    }
    Some(Decoded { stops, pressed: flags == 1, open })
}

/// temper's draw: its own kind, else skein's flows.
pub fn draw_any(kind: DrawingKind, data: &[u8], width: u16, height: u16, scene: &mut skein_ui::scene::Scene) -> bool {
    if kind.owner == crate::look::owner() {
        draw(kind, data, width, height, scene)
    } else {
        skein_ui::flow::draw(kind, data, width, height, scene)
    }
}
