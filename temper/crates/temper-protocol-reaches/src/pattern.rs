//! Segment patterns (temper/docs/design/domain/forge.md, 5.10).
use crate::PathSet;
use alloc::boxed::Box;
use skein_lib::List;

// Segments are walked by byte offsets, without callbacks or stored borrows.
fn segment(bytes: &[u8], start: usize) -> &[u8] {
    let rest = bytes.get(start..).unwrap_or_default();
    for (index, byte) in rest.iter().enumerate() {
        if *byte == b'/' {
            return rest.get(..index).expect("position within rest");
        }
    }
    rest
}
#[must_use]
pub fn well_formed(pattern: &[u8]) -> bool {
    if pattern.is_empty() {
        return false;
    }
    let mut start = 0;
    for end in 0..=pattern.len() {
        if end == pattern.len() || pattern.get(end) == Some(&b'/') {
            let part = pattern.get(start..end).expect("segment boundaries");
            if part.is_empty() {
                return false;
            }
            if part != b"**" {
                let mut star = false;
                for byte in part {
                    if star && *byte == b'*' {
                        return false;
                    }
                    star = *byte == b'*';
                }
            }
            start = end.saturating_add(1);
        }
    }
    true
}
// A segment's glob has a bounded greedy walk: the last star consumes one
// more byte on each retry, and a retry only revisits the suffix after it.
fn within(pattern: &[u8], path: &[u8]) -> bool {
    let mut p = 0_usize;
    let mut t = 0_usize;
    let mut star = None;
    let mut consumed = 0_usize;
    let bound = pattern.len().saturating_add(1).saturating_mul(path.len().saturating_add(1));
    for _attempt in 0..bound {
        if pattern.get(p) == Some(&b'*') {
            p = p.saturating_add(1);
            star = Some(p);
            consumed = t;
        } else if t == path.len() {
            return p == pattern.len();
        } else if pattern.get(p) == path.get(t) {
            p = p.saturating_add(1);
            t = t.saturating_add(1);
        } else if let Some(after) = star {
            consumed = consumed.saturating_add(1);
            if consumed > path.len() {
                return false;
            }
            t = consumed;
            p = after;
        } else {
            return false;
        }
    }
    false
}
#[must_use]
pub fn matches(pattern: &[u8], path: &[u8]) -> bool {
    if !well_formed(pattern) {
        return false;
    }
    let capacity = match u32::try_from(path.len().saturating_add(1)) {
        Ok(value) => value,
        Err(_error) => return false,
    };
    let mut previous = List::with_capacity(capacity);
    previous.push(true).expect("initial boundary");
    for _byte in path {
        previous.push(false).expect("one state per byte");
    }
    let mut p = 0;
    for _part in 0..=pattern.len() {
        if p > pattern.len() {
            break;
        }
        let part = segment(pattern, p);
        let mut current = List::with_capacity(capacity);
        for _boundary in 0..capacity {
            current.push(false).expect("one state per byte and end");
        }
        let mut t = 0;
        for _path_part in 0..=path.len() {
            let active = *previous.get(u32::try_from(t).expect("bounded path offset")).expect("boundary state");
            if part == b"**" && active {
                *current.get_mut(u32::try_from(t).expect("bounded offset")).expect("boundary") = true;
            }
            if t == path.len() {
                break;
            }
            let text = segment(path, t);
            let end = t.saturating_add(text.len());
            let next = if end == path.len() { end } else { end.saturating_add(1) };
            let carry = part == b"**" && *current.get(u32::try_from(t).expect("bounded offset")).expect("boundary");
            if carry || (active && within(part, text)) {
                *current.get_mut(u32::try_from(next).expect("bounded offset")).expect("boundary") = true;
            }
            t = next;
        }
        previous = current;
        p = p.saturating_add(part.len()).saturating_add(1);
    }
    *previous.get(u32::try_from(path.len()).expect("bounded length")).expect("end state")
}
impl PathSet {
    #[must_use]
    pub fn matched(&self, paths: &[Box<[u8]>]) -> u8 {
        let mut bits = 0;
        for (index, pattern) in self.patterns.iter().enumerate() {
            for path in paths {
                if matches(pattern, path) {
                    bits |= pattern_bit(index);
                    break;
                }
            }
        }
        bits
    }
}
pub(crate) fn pattern_bit(index: usize) -> u8 {
    match u32::try_from(index) {
        Ok(shift) => 1_u8.checked_shl(shift).unwrap_or(0),
        Err(_error) => 0,
    }
}
pub(crate) fn set_bit(number: u8) -> u8 {
    match number.checked_sub(1) {
        Some(index) => pattern_bit(usize::from(index)),
        None => 0,
    }
}
