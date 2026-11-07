//! Flow owners, data and scene bounds (skein/docs/design/ui.md, 3.3, 10.1 and 11.5).

use crate::look::Names;
use crate::{DrawingKind, Owner, Style};
use skein_lib::{List, Reader, Writer, usize_of};

mod layout;
pub(crate) mod read;

pub const OWNER: Owner = Owner { number: u32::MAX };
pub const KIND: DrawingKind = DrawingKind { owner: OWNER, kind: 0 };
pub const BOX: Style = Style { owner: OWNER, token: 0 };
pub const NAME: Style = Style { owner: OWNER, token: 1 };
pub const CAPTION: Style = Style { owner: OWNER, token: 2 };
pub const ARROW: Style = Style { owner: OWNER, token: 3 };
pub const STRONG: Style = Style { owner: OWNER, token: 4 };
pub const NAMES: Names<'static> = Names {
    owner: OWNER,
    prefix: b"skein",
    styles: &[
        b"skein-ui-flow-box",
        b"skein-ui-flow-name",
        b"skein-ui-flow-caption",
        b"skein-ui-flow-arrow",
        b"skein-ui-flow-strong",
    ],
    regions: &[],
};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Direction {
    LeftToRight,
    TopToBottom,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Node<'a> {
    pub name: &'a [u8],
    pub caption: Option<&'a [u8]>,
    pub strong: bool,
}

/// None past `bound`, or for an empty path, name or caption, or a name or caption past a `u16`'s bytes.
#[must_use]
pub fn encode(direction: Direction, nodes: &[Node<'_>], bound: u32) -> Option<List<u8>> {
    if nodes.is_empty() {
        return None;
    }
    let mut size = 1_u32;
    for node in nodes {
        size = size.checked_add(3)?.checked_add(u32::from(text_length(node.name)?))?;
        if let Some(caption) = node.caption {
            size = size.checked_add(2)?.checked_add(u32::from(text_length(caption)?))?;
        }
        if size > bound {
            return None;
        }
    }
    let mut writer = Writer::new(usize_of(size));
    let direction = match direction {
        Direction::LeftToRight => 0,
        Direction::TopToBottom => 1,
    };
    writer.put(&[direction]).expect("counted direction byte");
    for node in nodes {
        let flags = u8::from(node.caption.is_some()) | (u8::from(node.strong) << 1_u8);
        writer.put(&[flags]).expect("counted flags byte");
        write_text(&mut writer, node.name);
        if let Some(caption) = node.caption {
            write_text(&mut writer, caption);
        }
    }
    Some(List::from_boxed(writer.finish()))
}

fn text_length(bytes: &[u8]) -> Option<u16> {
    if bytes.is_empty() {
        return None;
    }
    u16::try_from(bytes.len()).ok()
}

fn write_text(writer: &mut Writer, bytes: &[u8]) {
    let length = text_length(bytes).expect("checked flow text length");
    writer.put(&length.to_le_bytes()).expect("counted text length bytes");
    writer.put(bytes).expect("counted text bytes");
}

/// The whole data checked first: None for anything `encode` can't write.
#[must_use]
pub fn decode(data: &[u8]) -> Option<(Direction, Nodes<'_>)> {
    u32::try_from(data.len()).ok()?;
    let mut reader = Reader::new(data);
    let byte = reader.u8()?;
    let direction = if byte == 0 {
        Direction::LeftToRight
    } else if byte == 1 {
        Direction::TopToBottom
    } else {
        return None;
    };
    if reader.is_empty() {
        return None;
    }
    let nodes = Nodes { reader: reader.clone() };
    // Each node consumes at least four bytes, so this byte count bounds the scan.
    for _ in 0..reader.remaining() {
        if reader.is_empty() {
            break;
        }
        read_node(&mut reader)?;
    }
    Some((direction, nodes))
}

/// Iterates a decoded flow's nodes in the path's order.
#[derive(Clone, Debug)]
pub struct Nodes<'a> {
    reader: Reader<'a>,
}

impl<'a> Iterator for Nodes<'a> {
    type Item = Node<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        read_node(&mut self.reader)
    }
}

fn read_node<'a>(reader: &mut Reader<'a>) -> Option<Node<'a>> {
    let flags = reader.u8()?;
    if flags & !3 != 0 {
        return None;
    }
    let name = read_text(reader)?;
    let caption = if flags & 1 != 0 { Some(read_text(reader)?) } else { None };
    Some(Node { name, caption, strong: flags & 2 != 0 })
}

fn read_text<'a>(reader: &mut Reader<'a>) -> Option<&'a [u8]> {
    let low = reader.u8()?;
    let high = reader.u8()?;
    let length = u16::from_le_bytes([low, high]);
    if length == 0 {
        return None;
    }
    reader.bytes(u32::from(length))
}

/// The marks a flow of `nodes` nodes takes at most: four each.
#[must_use]
pub const fn marks(nodes: u32) -> Option<u32> {
    nodes.checked_mul(4)
}

/// The scene's bytes it takes at most: `data`, then 25 for each node's arrow.
#[must_use]
pub const fn scene_bytes(nodes: u32, data: u32) -> Option<u32> {
    match nodes.checked_mul(25) {
        Some(arrows) => data.checked_add(arrows),
        None => None,
    }
}
