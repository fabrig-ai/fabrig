//! Reads a fence into a flow (skein/docs/design/ui/markdown.md, 1).

use super::{Direction, Node, decode, encode};
use crate::Limits;
use skein_lib::{List, usize_of};

/// One node as read: `next` the node its edge goes to, `entered` whether one goes to it.
#[derive(Debug)]
pub(crate) struct Entry<'a> {
    id: &'a [u8],
    name: &'a [u8],
    caption: Option<&'a [u8]>,
    labelled: bool,
    strong: bool,
    next: Option<u32>,
    entered: bool,
}

pub(crate) struct Read<'a> {
    pub name: &'a [u8],
    pub data: List<u8>,
    pub fallback: List<u8>,
}

pub(crate) fn read<'a>(info: &[u8], body: &'a [u8], limits: &Limits) -> Option<Read<'a>> {
    let info = info.strip_suffix(b"\n").unwrap_or(info);
    let info = info.strip_suffix(b"\r").unwrap_or(info);
    if trim(info.strip_prefix(b"```")?) != b"mermaid" {
        return None;
    }
    let bytes = u32::try_from(body.len()).ok()?;
    let capacity = limits.drawing_bytes.checked_div(4)?.min(bytes.checked_div(2)?.checked_add(1)?);
    if capacity == 0 {
        return None;
    }
    let mut entries = List::with_capacity(capacity);
    let mut direction = None;
    let mut name = None;
    let mut start = 0_usize;
    for (end, byte) in body.iter().enumerate() {
        if *byte == b'\n' {
            body_line(body.get(start..end)?, &mut direction, &mut name, &mut entries)?;
            start = end.checked_add(1)?;
        }
    }
    body_line(body.get(start..)?, &mut direction, &mut name, &mut entries)?;
    let direction = direction?;
    let name = name.unwrap_or(b"");
    if name.len() > usize_of(limits.text_per_node) {
        return None;
    }
    let mut first = None;
    for (index, entry) in entries.iter().enumerate() {
        if !entry.entered {
            if first.is_some() {
                return None;
            }
            first = Some(u32::try_from(index).ok()?);
        }
    }
    let mut next = Some(first?);
    if entries.len() < 2 {
        return None;
    }
    // A path uses at least four body bytes per node, less its final arrow.
    // Compact the entries before projecting nodes: the two lists together fit
    // the Entry price for body bytes / 2 + 1 in markdown_bound (ui/README.md, 13).
    let entries = entries.into_boxed();
    let mut nodes = List::with_capacity(u32::try_from(entries.len()).ok()?);
    for _node in entries.as_ref() {
        let entry = entries.get(usize_of(next?))?;
        nodes.push(Node { name: entry.name, caption: entry.caption, strong: entry.strong }).ok()?;
        next = entry.next;
    }
    if next.is_some() {
        return None;
    }
    let data = encode(direction, nodes.as_slice(), limits.drawing_bytes)?;
    let (_, nodes) = decode(data.as_slice())?;
    let mut size = 1_u32;
    for (at, node) in nodes.clone().enumerate() {
        if at != 0 {
            size = size.checked_add(7)?;
        }
        size = size.checked_add(u32::try_from(node.name.len()).ok()?)?;
        if let Some(caption) = node.caption {
            size = size.checked_add(2)?.checked_add(u32::try_from(caption.len()).ok()?)?;
        }
    }
    if size > limits.text_per_node {
        return None;
    }
    let mut fallback = List::with_capacity(size);
    for (at, node) in nodes.enumerate() {
        if at != 0 {
            append(&mut fallback, b"; then ")?;
        }
        append(&mut fallback, node.name)?;
        if let Some(caption) = node.caption {
            append(&mut fallback, b", ")?;
            append(&mut fallback, caption)?;
        }
    }
    append(&mut fallback, b".")?;
    Some(Read { name, data, fallback })
}

fn append(out: &mut List<u8>, bytes: &[u8]) -> Option<()> {
    for byte in bytes {
        out.push(*byte).ok()?;
    }
    Some(())
}

fn trim(mut bytes: &[u8]) -> &[u8] {
    for _byte in 0..bytes.len() {
        if bytes.first() != Some(&b' ') && bytes.first() != Some(&b'\t') {
            break;
        }
        bytes = bytes.get(1..).expect("trim removes an existing byte");
    }
    for _byte in 0..bytes.len() {
        if bytes.last() != Some(&b' ') && bytes.last() != Some(&b'\t') {
            break;
        }
        bytes =
            bytes.get(..bytes.len().checked_sub(1).expect("trim has a byte")).expect("trim removes an existing byte");
    }
    bytes
}

fn read_direction(line: &[u8]) -> Option<Direction> {
    let rest = if let Some(rest) = line.strip_prefix(b"flowchart") { rest } else { line.strip_prefix(b"graph")? };
    if rest.first() != Some(&b' ') && rest.first() != Some(&b'\t') {
        return None;
    }
    let rest = trim(rest);
    if rest == b"LR" {
        Some(Direction::LeftToRight)
    } else if rest == b"TD" || rest == b"TB" {
        Some(Direction::TopToBottom)
    } else {
        None
    }
}

fn read_line<'a>(line: &'a [u8], entries: &mut List<Entry<'a>>) -> Option<()> {
    let mut rest = line;
    let mut previous = None;
    // Every node consumes at least one byte of this line.
    for _byte in 0..line.len() {
        let (index, tail) = read_node(rest, entries)?;
        if let Some(previous) = previous {
            if entries.get(previous)?.next.is_some() || entries.get(index)?.entered {
                return None;
            }
            entries.get_mut(previous)?.next = Some(index);
            entries.get_mut(index)?.entered = true;
        }
        rest = trim(tail);
        if rest.is_empty() {
            return Some(());
        }
        rest = trim(rest.strip_prefix(b"-->")?);
        previous = Some(index);
    }
    None
}

fn read_node<'a>(line: &'a [u8], entries: &mut List<Entry<'a>>) -> Option<(u32, &'a [u8])> {
    let mut end = 0_usize;
    for (at, byte) in line.iter().enumerate() {
        if line.get(at..)?.starts_with(b"-->") {
            break;
        }
        if !byte.is_ascii_alphanumeric() && *byte != b'_' && *byte != b'-' {
            break;
        }
        end = at.checked_add(1)?;
    }
    if end == 0 {
        return None;
    }
    let id = line.get(..end)?;
    let mut rest = line.get(end..)?;
    let mut label = None;
    if let Some(words) = rest.strip_prefix(b"[") {
        let (words, closing) =
            if let Some(quoted) = words.strip_prefix(b"\"") { (quoted, b'"') } else { (words, b']') };
        let mut end = None;
        for (at, byte) in words.iter().enumerate() {
            if *byte == closing {
                end = Some(at);
                break;
            }
        }
        let end = end?;
        label = Some(read_label(words.get(..end)?)?);
        rest = words.get(end.checked_add(1)?..)?;
        if closing == b'"' {
            rest = rest.strip_prefix(b"]")?;
        }
    }
    let strong = rest.starts_with(b":::strong");
    if strong {
        rest = rest.strip_prefix(b":::strong")?;
    }
    let mut found = None;
    for (index, entry) in entries.iter().enumerate() {
        if entry.id == id {
            found = Some(u32::try_from(index).ok()?);
            break;
        }
    }
    let index = if let Some(index) = found {
        index
    } else {
        let index = entries.len();
        entries
            .push(Entry { id, name: id, caption: None, labelled: false, strong: false, next: None, entered: false })
            .ok()?;
        index
    };
    let entry = entries.get_mut(index)?;
    if let Some((name, caption)) = label {
        if entry.labelled {
            return None;
        }
        entry.name = name;
        entry.caption = caption;
        entry.labelled = true;
    }
    entry.strong |= strong;
    Some((index, rest))
}

fn read_label(words: &[u8]) -> Option<(&[u8], Option<&[u8]>)> {
    let mut split = None;
    let mut skip = 0;
    for (at, byte) in words.iter().enumerate() {
        if at < skip {
            continue;
        }
        if *byte == b'<' {
            if split.is_some() {
                return None;
            }
            let rest = words.get(at..)?;
            let length = if rest.starts_with(b"<br>") {
                4
            } else if rest.starts_with(b"<br/>") {
                5
            } else {
                return None;
            };
            skip = at.checked_add(length)?;
            split = Some((at, skip));
        } else if byte.is_ascii_control() {
            return None;
        }
    }
    let (name, caption) = if let Some((end, start)) = split {
        let caption = trim(words.get(start..)?);
        if caption.is_empty() {
            return None;
        }
        (trim(words.get(..end)?), Some(caption))
    } else {
        (trim(words), None)
    };
    if name.is_empty() {
        return None;
    }
    Some((name, caption))
}

fn body_line<'a>(
    line: &'a [u8],
    direction: &mut Option<Direction>,
    name: &mut Option<&'a [u8]>,
    entries: &mut List<Entry<'a>>,
) -> Option<()> {
    let line = trim(line.strip_suffix(b"\r").unwrap_or(line));
    if line.is_empty() || line.starts_with(b"%%") || line.starts_with(b"classDef ") || line.starts_with(b"accDescr:") {
        return Some(());
    }
    if direction.is_none() {
        *direction = Some(read_direction(line)?);
    } else if let Some(words) = line.strip_prefix(b"accTitle:") {
        let words = trim(words);
        if name.is_some() || words.is_empty() {
            return None;
        }
        *name = Some(words);
    } else {
        read_line(line, entries)?;
    }
    Some(())
}
