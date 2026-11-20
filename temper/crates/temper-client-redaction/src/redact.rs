//! Pure recording redaction (temper/docs/design/client/web/journey-redaction.md).
use crate::Originals;
use skein_lib::bytes::find_from;
use std::collections::BTreeMap;

/// Whether a length prefix counts a field's bytes (client/web/journey-redaction.md).
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Framing {
    Framed,
    Text,
}

#[derive(Clone, Copy)]
enum Output {
    Digest([u8; 32]),
    Xs(usize),
}
struct Replacement {
    end: usize,
    output: Output,
}

/// Replace every original's forms and digests in one field; return the places replaced.
pub fn redact_field(field: &mut Box<[u8]>, framing: Framing, originals: &Originals) -> u32 {
    let mut replacements = BTreeMap::new();
    for original in originals.sought() {
        for (from, to) in original.digests() {
            seek(field, from, &Output::Digest(*to), &mut replacements);
        }
        for &index in original.form_order() {
            let form = &original.forms()[index];
            let length = match framing {
                Framing::Framed => form.len(),
                Framing::Text => original.len(),
            };
            seek(field, form, &Output::Xs(length), &mut replacements);
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
        match replacement.output {
            Output::Digest(bytes) => output.extend_from_slice(&bytes),
            Output::Xs(length) => output.resize(output.len() + length, b'x'),
        }
        start = replacement.end;
    }
    output.extend_from_slice(&field[start..]);
    *field = output.into();
    places
}

fn seek(field: &[u8], from: &[u8], to: &Output, replacements: &mut BTreeMap<usize, Replacement>) {
    let mut start = 0;
    while let Some(at) = find_from(field, from, start) {
        let end = at + from.len();
        let overlaps = replacements.range(..end).next_back().is_some_and(|(_, prior)| prior.end > at);
        if overlaps {
            start = at + 1;
        } else {
            replacements.insert(at, Replacement { end, output: *to });
            start = end;
        }
    }
}

#[cfg(test)]
mod tests;
