//! Pure recording redaction (temper/docs/design/client/web/journeys.md, 3.6).
use crate::Originals;
use skein_lib::bytes::find_from;
use std::{cmp::Reverse, collections::BTreeMap};

/// Whether a length prefix counts a field's bytes (client/web/journeys.md, 3.6).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Framing {
    Framed,
    Text,
}

struct Replacement {
    end: usize,
    bytes: Box<[u8]>,
}

/// Replace every original's forms and digests in one field; return the places replaced.
pub fn redact_field(field: &mut Box<[u8]>, framing: Framing, originals: &Originals) -> u32 {
    let mut replacements = BTreeMap::new();
    for original in originals.sought() {
        for (from, to) in original.digests() {
            seek(field, from, to, &mut replacements);
        }
        // An escaped form can contain its bytes as a prefix: a& in a&amp;.
        let mut forms: Vec<_> = original.forms().iter().collect();
        forms.sort_by_key(|form| Reverse(form.len()));
        for form in forms {
            let length = match framing {
                Framing::Framed => form.len(),
                Framing::Text => original.len(),
            };
            seek(field, form, &vec![b'x'; length], &mut replacements);
        }
    }
    let places = u32::try_from(replacements.len()).expect("field replacement count fits u32");
    if places == 0 {
        return 0;
    }
    // All searches see the original field; emitted digests are never searched again.
    let mut output = Vec::with_capacity(field.len());
    let mut start = 0;
    for (at, replacement) in replacements {
        output.extend_from_slice(&field[start..at]);
        output.extend_from_slice(&replacement.bytes);
        start = replacement.end;
    }
    output.extend_from_slice(&field[start..]);
    *field = output.into();
    places
}

fn seek(field: &[u8], from: &[u8], to: &[u8], replacements: &mut BTreeMap<usize, Replacement>) {
    let mut start = 0;
    while let Some(at) = find_from(field, from, start) {
        let end = at + from.len();
        let overlaps = replacements.range(..end).next_back().is_some_and(|(_, prior)| prior.end > at);
        if overlaps {
            start = at + 1;
        } else {
            replacements.insert(at, Replacement { end, bytes: to.into() });
            start = end;
        }
    }
}

#[cfg(test)]
mod tests;
