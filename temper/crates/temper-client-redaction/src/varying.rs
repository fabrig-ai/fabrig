//! Recording placeholders (temper/docs/design/client/journey-placeholders.md, 2, 3 and 6).
use crate::{
    Framing,
    originals::{debug, html, percent},
};
use jig_client_protocol_wire::{ApprovalCode, CODE_ALPHABET};
use skein_lib::bytes::find_from;
use std::{cmp::Reverse, collections::BTreeMap};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Varying {
    Build,
    Port,
    Code,
}

pub struct Normalised {
    pub recording: Box<[u8]>,
    pub build: u32,
    pub port: u32,
    pub code: u32,
    pub places: u32,
    pub stood: u32,
}

const BUILD: &[u8] = b"0.0.0 (000000000000)";
const HOSTS: [&[u8]; 3] = [b"127.0.0.1:", b"localhost:", b"[::1]:"];

struct Form {
    from: Box<[u8]>,
    to: Box<[u8]>,
}
struct Original {
    kind: Varying,
    key: Box<[u8]>,
    forms: Vec<Form>,
    stood: bool,
    replaced: bool,
}
struct Port {
    number: u16,
    placeholder: Option<Box<[u8]>>,
    stood: bool,
    replaced: bool,
}
struct Hit {
    at: usize,
    end: usize,
    number: u16,
    decimal: bool,
}

/// A recording's two passes: collect every field, then replace every field.
/// Both shells enable decimal byte elements for addresses in their events' Debug prints.
pub struct Varied {
    originals: Vec<Original>,
    ports: Vec<Port>,
    prefixes: Vec<(Box<[u8]>, bool)>,
    places: u32,
}
impl Default for Varied {
    fn default() -> Self {
        Self::new()
    }
}
impl Varied {
    #[must_use]
    pub fn new() -> Self {
        let mut prefixes = Vec::new();
        for host in HOSTS {
            for form in forms(host, host) {
                if !prefixes.contains(&(form.from.clone(), false)) {
                    prefixes.push((form.from, false));
                }
            }
        }
        Self { originals: Vec::new(), ports: Vec::new(), prefixes, places: 0 }
    }

    pub fn decimal_ports(&mut self) {
        for host in HOSTS {
            let prefix = [decimal(host).as_ref(), b", "].concat().into_boxed_slice();
            if !self.prefixes.contains(&(prefix.clone(), true)) {
                self.prefixes.push((prefix, true));
            }
        }
    }

    pub fn build(&mut self, bytes: &[u8]) {
        if bytes.is_empty() || bytes.iter().all(|byte| *byte == b'x') {
            return;
        }
        self.original(Varying::Build, bytes, forms(bytes, BUILD), false);
    }

    pub fn code(&mut self, bytes: &[u8]) {
        if bytes.iter().all(|byte| *byte == b'x') {
            return;
        }
        let Some(code) = ApprovalCode::parse(bytes) else {
            return;
        };
        if self.originals.iter().any(|one| one.kind == Varying::Code && one.key.as_ref() == code.letters()) {
            return;
        }
        let index = self.originals.iter().filter(|one| one.kind == Varying::Code).count();
        let letter = CODE_ALPHABET.get(index);
        let mut pairs = Vec::new();
        if let Some(letter) = letter {
            let placeholder = ApprovalCode::parse(&[*letter; 8]).expect("alphabet letters make a code");
            pairs.extend(forms(code.letters(), placeholder.letters()));
            pairs.extend(forms(&code.shown(), &placeholder.shown()));
        }
        self.original(Varying::Code, code.letters(), pairs, letter.is_none());
    }

    fn original(&mut self, kind: Varying, key: &[u8], forms: Vec<Form>, stood: bool) {
        if !self.originals.iter().any(|one| one.kind == kind && one.key.as_ref() == key) {
            self.originals.push(Original { kind, key: key.into(), forms, stood, replaced: false });
        }
    }

    /// Discover ports in byte order and remember every incompatible framed form.
    pub fn seen(&mut self, field: &[u8], framing: Framing) {
        for hit in self.hits(field) {
            let index = if let Some(index) = self.ports.iter().position(|port| port.number == hit.number) {
                index
            } else {
                let ordinal = self.ports.len().checked_add(1).expect("port count fits usize");
                let placeholder = if ordinal <= 99 {
                    Some((10000 + ordinal).to_string().into_bytes().into_boxed_slice())
                } else {
                    None
                };
                self.ports.push(Port {
                    number: hit.number,
                    stood: placeholder.is_none(),
                    placeholder,
                    replaced: false,
                });
                self.ports.len() - 1
            };
            let port = &mut self.ports[index];
            if let Some(placeholder) = &port.placeholder {
                let length = if hit.decimal { decimal(placeholder).len() } else { placeholder.len() };
                if framing == Framing::Framed && hit.end - hit.at != length {
                    port.stood = true;
                }
            }
        }
        for one in &mut self.originals {
            if framing == Framing::Framed
                && one
                    .forms
                    .iter()
                    .any(|form| form.from.len() != form.to.len() && find_from(field, &form.from, 0).is_some())
            {
                one.stood = true;
            }
        }
    }

    /// All searches see the input field, so swapped originals cannot chain.
    pub fn replace(&mut self, field: &mut Box<[u8]>) {
        let mut replacements = BTreeMap::new();
        for hit in self.hits(field) {
            let port =
                self.ports.iter_mut().find(|port| port.number == hit.number).expect("port collected in first pass");
            if port.stood {
                continue;
            }
            let placeholder = port.placeholder.as_ref().expect("replaceable port has a placeholder");
            let to = if hit.decimal { decimal(placeholder) } else { placeholder.clone() };
            if field[hit.at..hit.end] != *to {
                replacements.insert(hit.at, (hit.end, to));
                port.replaced = true;
            }
        }
        for one in &mut self.originals {
            if one.stood {
                continue;
            }
            for form in &one.forms {
                if form.from == form.to {
                    continue;
                }
                let mut start = 0;
                while let Some(at) = find_from(field, &form.from, start) {
                    let end = at + form.from.len();
                    if replacements.range(..end).next_back().is_none_or(|(_, (prior, _))| *prior <= at) {
                        replacements.insert(at, (end, form.to.clone()));
                        one.replaced = true;
                    }
                    start = end;
                }
            }
        }
        self.places = self.places.checked_add(count(replacements.len())).expect("places fit u32");
        if replacements.is_empty() {
            return;
        }
        let mut output = Vec::with_capacity(field.len());
        let mut from = 0;
        for (at, (end, to)) in replacements {
            output.extend_from_slice(&field[from..at]);
            output.extend_from_slice(&to);
            from = end;
        }
        output.extend_from_slice(&field[from..]);
        *field = output.into();
    }

    #[must_use]
    pub fn counts(&self, recording: Box<[u8]>) -> Normalised {
        Normalised {
            recording,
            build: count(self.originals.iter().filter(|one| one.kind == Varying::Build && one.replaced).count()),
            code: count(self.originals.iter().filter(|one| one.kind == Varying::Code && one.replaced).count()),
            port: count(self.ports.iter().filter(|port| port.replaced).count()),
            places: self.places,
            stood: count(
                self.originals.iter().filter(|one| one.stood).count()
                    + self.ports.iter().filter(|port| port.stood).count(),
            ),
        }
    }

    fn hits(&self, field: &[u8]) -> Vec<Hit> {
        let mut hits = Vec::new();
        for (prefix, decimal) in &self.prefixes {
            let mut start = 0;
            while let Some(at) = find_from(field, prefix, start) {
                let digits = at + prefix.len();
                if (!*decimal || field[..at].ends_with(b"[") || field[..at].ends_with(b", "))
                    && let Some((end, number)) = port_number(field, digits, *decimal)
                {
                    hits.push(Hit { at: digits, end, number, decimal: *decimal });
                }
                start = digits;
            }
        }
        hits.sort_by_key(|hit| hit.at);
        hits.dedup_by_key(|hit| hit.at);
        hits
    }
}

fn count(value: usize) -> u32 {
    u32::try_from(value).expect("recording count fits u32")
}

fn forms(from: &[u8], to: &[u8]) -> Vec<Form> {
    let mut result = Vec::new();
    for (from, to) in [
        (Box::from(from), Box::from(to)),
        (percent(from, false), percent(to, false)),
        (percent(from, true), percent(to, true)),
        (html(from), html(to)),
    ] {
        if !result.iter().any(|form: &Form| form.from == from) {
            result.push(Form { from, to });
        }
    }
    if let (Some(from), Some(to)) = (debug(from), debug(to))
        && !result.iter().any(|form| form.from == from)
    {
        result.push(Form { from, to });
    }
    result.sort_by_key(|form| Reverse(form.from.len()));
    result
}
fn decimal(bytes: &[u8]) -> Box<[u8]> {
    bytes.iter().map(u8::to_string).collect::<Vec<_>>().join(", ").into_bytes().into()
}
fn port_number(field: &[u8], start: usize, decimal: bool) -> Option<(usize, u16)> {
    let mut cursor = start;
    let mut end = start;
    let mut number = 0_u32;
    let mut width = 0;
    loop {
        let (byte, next) = if decimal {
            decimal_element(field, cursor)?
        } else {
            let Some(byte) = field.get(cursor) else {
                break;
            };
            (*byte, cursor + 1)
        };
        if !byte.is_ascii_digit() {
            break;
        }
        width += 1;
        if width > 5 {
            return None;
        }
        number = number * 10 + u32::from(byte - b'0');
        end = next;
        if decimal {
            if field[end..].starts_with(b", ") {
                cursor = end + 2;
                let (byte, _) = decimal_element(field, cursor)?;
                if byte.is_ascii_digit() {
                    continue;
                }
            }
            break;
        }
        cursor = end;
    }
    if width == 0 || number == 0 {
        return None;
    }
    Some((end, u16::try_from(number).ok()?))
}

fn decimal_element(field: &[u8], at: usize) -> Option<(u8, usize)> {
    let tail = field.get(at..)?;
    let length = tail.iter().position(|byte| *byte == b',' || *byte == b']')?;
    let element = &tail[..length];
    let byte = std::str::from_utf8(element).ok()?.parse::<u8>().ok()?;
    if byte.to_string().as_bytes() != element {
        return None;
    }
    Some((byte, at + length))
}

#[cfg(test)]
mod tests;
