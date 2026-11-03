//! Strict reading of the versioned log text (playground.md, 6.1).
use super::{End, Entry, Read, Refused, Said};
use crate::{bridge, fetch, transport::StreamDuty};
use skein_lib::{Time, Wall, base64};

pub(super) fn read(text: &[u8]) -> Result<Read, Refused> {
    let mut lines = text.split_inclusive(|byte| *byte == b'\n');
    let mut first = Fields::new(lines.next().ok_or(Refused::Line { line: 1 })?, 1)?;
    if first.word()? != "skein-playground-log" {
        return Err(Refused::Format);
    }
    let version = first.word()?;
    if version != "2" {
        return Err(Refused::Version { found: version.as_bytes().into() });
    }
    let build = first.field("build=")?;
    if build.is_empty() {
        return Err(first.error());
    }
    let build = if build == "-" { None } else { Some(build.as_bytes().into()) };
    let seed = first.number("seed=")?;
    let start = Wall::from_nanos(first.number("start=")?);
    first.end()?;
    let mut entries = Vec::new();
    let mut end = None;
    let mut number = 1_u32;
    for line in lines {
        number = number.saturating_add(1);
        if end.is_some() {
            return Err(Refused::Line { line: number });
        }
        let mut fields = Fields::new(line, number)?;
        match fields.word()? {
            "input" => {
                let stamp = fields.stamp()?;
                let said = fields.said()?;
                let input = fields.input()?;
                fields.end()?;
                entries.push(Entry { stamp, input, said });
            }
            "end" => {
                let stamp = fields.stamp()?;
                let said = fields.said()?;
                fields.end()?;
                end = Some(End::At { step: stamp.step, instant: stamp.instant, said });
            }
            "cut" => {
                fields.end()?;
                end = Some(End::Cut);
            }
            _ => return Err(fields.error()),
        }
    }
    let end = end.ok_or(Refused::Line { line: number.saturating_add(1) })?;
    Ok(Read { build, seed, start, entries, end })
}

struct Fields<'a> {
    fields: std::str::Split<'a, char>,
    line: u32,
}
impl<'a> Fields<'a> {
    fn new(line: &'a [u8], number: u32) -> Result<Self, Refused> {
        let error = Refused::Line { line: number };
        let line = line.strip_suffix(b"\n").ok_or_else(|| error.clone())?;
        if !line.iter().all(|byte| (b' '..=b'~').contains(byte)) {
            return Err(error);
        }
        let text = std::str::from_utf8(line).map_err(|_| error)?;
        Ok(Self { fields: text.split(' '), line: number })
    }
    fn error(&self) -> Refused {
        Refused::Line { line: self.line }
    }
    fn word(&mut self) -> Result<&'a str, Refused> {
        self.fields.next().filter(|word| !word.is_empty()).ok_or_else(|| self.error())
    }
    fn field(&mut self, name: &str) -> Result<&'a str, Refused> {
        self.word()?.strip_prefix(name).ok_or_else(|| self.error())
    }
    fn number<T: std::str::FromStr>(&mut self, name: &str) -> Result<T, Refused> {
        let digits = self.field(name)?;
        if digits.is_empty()
            || !digits.bytes().all(|byte| byte.is_ascii_digit())
            || (digits.len() > 1 && digits.starts_with('0'))
        {
            return Err(self.error());
        }
        digits.parse().map_err(|_| self.error())
    }
    fn decode(&self, text: &str) -> Result<Box<[u8]>, Refused> {
        base64::decode(base64::Alphabet::Standard, text.as_bytes(), u32::MAX).map_err(|_| self.error())
    }
    fn bytes(&mut self, name: &str) -> Result<Box<[u8]>, Refused> {
        let text = self.field(name)?;
        self.decode(text)
    }
    fn text(&mut self, name: &str) -> Result<String, Refused> {
        String::from_utf8(self.bytes(name)?.into_vec()).map_err(|_| self.error())
    }
    fn stamp(&mut self) -> Result<bridge::Stamp, Refused> {
        Ok(bridge::Stamp { step: self.number("step=")?, instant: Time::from_nanos(self.number("instant=")?) })
    }
    fn said(&mut self) -> Result<Said, Refused> {
        let text = self.field("said=")?;
        if text.len() != 64 {
            return Err(self.error());
        }
        let mut digest = [0; 32];
        for (slot, pair) in digest.iter_mut().zip(text.as_bytes().as_chunks::<2>().0) {
            let [high, low] = *pair;
            *slot = self.nibble(high)? << 4_u32 | self.nibble(low)?;
        }
        Ok(Said(digest))
    }
    fn nibble(&self, byte: u8) -> Result<u8, Refused> {
        match byte {
            b'0'..=b'9' => Ok(byte.checked_sub(b'0').expect("decimal digit")),
            b'a'..=b'f' => Ok(byte.checked_sub(b'a').expect("hex digit").saturating_add(10)),
            _ => Err(self.error()),
        }
    }
    fn headers(&mut self) -> Result<Vec<crate::transport::Header>, Refused> {
        let mut headers = Vec::new();
        while let Some(field) = self.fields.next() {
            let field = field.strip_prefix("header=").ok_or_else(|| self.error())?;
            let (name, value) = field.split_once(':').ok_or_else(|| self.error())?;
            headers.push((self.decode(name)?, self.decode(value)?));
        }
        Ok(headers)
    }
    fn input(&mut self) -> Result<bridge::Input, Refused> {
        Ok(match self.word()? {
            "exchange" => {
                let id = self.number("id=")?;
                let method = self.text("method=")?;
                let path = self.bytes("path=")?;
                let bound = self.number("bound=")?;
                let body = self.bytes("body=")?;
                let cache = match self.field("cache=")? {
                    "browser" => fetch::Cache::Browser,
                    "no-store" => fetch::Cache::NoStore,
                    _ => return Err(self.error()),
                };
                let count: usize = self.number("keep=")?;
                let mut keep = Vec::new();
                for _ in 0..count {
                    keep.push(self.text("name=")?);
                }
                let mut headers = Vec::new();
                for (name, value) in self.headers()? {
                    let name = String::from_utf8(name.into_vec()).map_err(|_| self.error())?;
                    let value = String::from_utf8(value.into_vec()).map_err(|_| self.error())?;
                    headers.push((name, value));
                }
                bridge::Input::Exchange(bridge::Exchange { id, path, method, headers, body, bound, cache, keep })
            }
            "abandon" => bridge::Input::Abandon { exchange: self.number("id=")? },
            "open" => {
                let path = self.bytes("path=")?;
                let bound = self.number("bound=")?;
                bridge::Input::Stream(StreamDuty::Open { path, headers: self.headers()?, bound })
            }
            "close" => bridge::Input::Stream(StreamDuty::Close),
            "read-on" => bridge::Input::ReadOn,
            _ => return Err(self.error()),
        })
    }
    fn end(&mut self) -> Result<(), Refused> {
        if self.fields.next().is_none() { Ok(()) } else { Err(self.error()) }
    }
}
