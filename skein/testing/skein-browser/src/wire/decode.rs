//! Bounded JSON document access for CDP replies and events.
//!
//! The document owns one message and its value index (skein/docs/design/testing/browser.md,
//! 4 and 8). Lookups skip indexed subtrees. Only requested strings unescape.

#![expect(clippy::disallowed_types, reason = "requested JSON strings are copied to owned byte vectors")]

use alloc::boxed::Box;
use alloc::vec::Vec;

/// A malformed or overlong CDP document.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum DecodeError {
    TooLong,
    TooDeep,
    Syntax,
}

/// One complete JSON document, without the trailing NUL, and the index of its values.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Document {
    bytes: Box<[u8]>,
    entries: Box<[Entry]>,
}

impl Document {
    /// Validate and hold a CDP message under `limit` bytes.
    pub fn parse(bytes: &[u8], limit: u32) -> Result<Document, DecodeError> {
        let Ok(limit) = usize::try_from(limit) else {
            return Err(DecodeError::TooLong);
        };
        if bytes.len() > limit {
            return Err(DecodeError::TooLong);
        }
        let count = Parser { bytes, at: 0, count: 0, entries: None }.document()?;
        let mut entries = Entry::allocate(usize::try_from(count).expect("entry count fits the bounded message"));
        Parser { bytes, at: 0, count: 0, entries: Some(&mut entries) }.document()?;
        Ok(Document { bytes: Box::from(bytes), entries })
    }

    /// The root JSON value.
    #[must_use]
    pub fn root(&self) -> Value<'_> {
        Value { document: self, at: 0 }
    }
}

/// A value's or a member key's place, in document order.
/// A member's key entry is followed by its value entry.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
struct Entry {
    start: u32,
    end: u32,
    after: u32,
}

impl Entry {
    #[expect(clippy::disallowed_methods, reason = "validation counted the exact index length before allocation")]
    fn allocate(count: usize) -> Box<[Entry]> {
        core::iter::repeat_n(Entry { start: 0, end: 0, after: 0 }, count).collect()
    }
}

/// A valid JSON value of a validated document: an entry of its index.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub struct Value<'a> {
    document: &'a Document,
    at: u32,
}

impl<'a> Value<'a> {
    fn entry(self) -> &'a Entry {
        self.document
            .entries
            .get(usize::try_from(self.at).expect("index fits the document"))
            .expect("validated value has an index entry")
    }

    /// The JSON representation of this value.
    #[must_use]
    pub fn raw(self) -> &'a [u8] {
        let entry = self.entry();
        let start = usize::try_from(entry.start).expect("offset fits the document");
        let end = usize::try_from(entry.end).expect("offset fits the document");
        self.document.bytes.get(start..end).expect("validated span is inside the document")
    }

    /// A member of an object, if it exists. Duplicate keys select the first.
    #[must_use]
    pub fn get(self, key: &[u8]) -> Option<Value<'a>> {
        if self.raw().first() != Some(&b'{') {
            return None;
        }
        let end = self.entry().after;
        let mut at = self.at.checked_add(1)?;
        while at < end {
            let name = Value { document: self.document, at };
            let raw = name.raw();
            let inner = raw.get(1..raw.len().checked_sub(1)?)?;
            let equal = if inner.contains(&b'\\') { name.text()?.as_ref() == key } else { inner == key };
            let value = Value { document: self.document, at: at.checked_add(1)? };
            if equal {
                return Some(value);
            }
            at = value.entry().after;
        }
        None
    }

    /// Iterate an array's elements.
    #[must_use]
    pub fn array(self) -> Option<Array<'a>> {
        if self.raw().first() != Some(&b'[') {
            return None;
        }
        Some(Array { document: self.document, next: self.at.checked_add(1)?, end: self.entry().after })
    }

    /// The unescaped UTF-8 bytes of a JSON string.
    #[must_use]
    pub fn text(self) -> Option<Box<[u8]>> {
        let bytes = self.raw();
        if bytes.first() != Some(&b'"') || bytes.last() != Some(&b'"') {
            return None;
        }
        let inner = bytes.get(1..bytes.len().checked_sub(1)?)?;
        let mut out = Vec::with_capacity(inner.len());
        let mut at = 0_usize;
        while at < inner.len() {
            let byte = *inner.get(at)?;
            if byte != b'\\' {
                out.push(byte);
                at = at.checked_add(1)?;
                continue;
            }
            at = at.checked_add(1)?;
            let escaped = *inner.get(at)?;
            match escaped {
                b'"' | b'\\' | b'/' => out.push(escaped),
                b'b' => out.push(8),
                b'f' => out.push(12),
                b'n' => out.push(b'\n'),
                b'r' => out.push(b'\r'),
                b't' => out.push(b'\t'),
                b'u' => {
                    let first = hex4(inner, at.checked_add(1)?)?;
                    at = at.checked_add(4)?;
                    let scalar = if (0xD800..=0xDBFF).contains(&first) {
                        if inner.get(at.checked_add(1)?..at.checked_add(3)?)? != b"\\u" {
                            return None;
                        }
                        let second = hex4(inner, at.checked_add(3)?)?;
                        if !(0xDC00..=0xDFFF).contains(&second) {
                            return None;
                        }
                        at = at.checked_add(6)?;
                        0x10000_u32
                            .checked_add((u32::from(first).checked_sub(0xD800)?).checked_mul(1024)?)?
                            .checked_add(u32::from(second).checked_sub(0xDC00)?)?
                    } else {
                        u32::from(first)
                    };
                    let ch = char::from_u32(scalar)?;
                    let mut encoded = [0_u8; 4];
                    out.extend_from_slice(ch.encode_utf8(&mut encoded).as_bytes());
                }
                _ => return None,
            }
            at = at.checked_add(1)?;
        }
        Some(out.into_boxed_slice())
    }

    /// An unsigned JSON integer.
    #[must_use]
    pub fn u64(self) -> Option<u64> {
        unsigned(self.raw())
    }

    /// A signed JSON integer.
    #[must_use]
    pub fn i64(self) -> Option<i64> {
        signed(self.raw())
    }

    /// A decimal CSS coordinate with its fractional part dropped toward zero.
    /// Chromium's coordinate fields are decimals rather than exponents.
    #[must_use]
    pub fn whole_pixels(self) -> Option<i64> {
        let bytes = self.raw();
        let mut end = 0_usize;
        for byte in bytes {
            if *byte == b'.' {
                break;
            }
            if *byte == b'e' || *byte == b'E' {
                return None;
            }
            end = end.checked_add(1)?;
        }
        signed(bytes.get(..end)?)
    }

    /// A JSON boolean.
    #[must_use]
    pub fn bool(self) -> Option<bool> {
        let bytes = self.raw();
        match bytes {
            b"true" => Some(true),
            b"false" => Some(false),
            _ => None,
        }
    }

    /// Whether this is JSON null.
    #[must_use]
    pub fn is_null(self) -> bool {
        self.raw() == b"null"
    }
}

/// A borrowed iterator through a validated JSON array's entries.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Array<'a> {
    document: &'a Document,
    next: u32,
    end: u32,
}

impl<'a> Iterator for Array<'a> {
    type Item = Value<'a>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.next >= self.end {
            return None;
        }
        let value = Value { document: self.document, at: self.next };
        self.next = value.entry().after;
        Some(value)
    }
}

#[derive(Debug)]
struct Parser<'a> {
    bytes: &'a [u8],
    at: usize,
    count: u32,
    entries: Option<&'a mut [Entry]>,
}

impl Parser<'_> {
    fn document(&mut self) -> Result<u32, DecodeError> {
        self.space();
        self.value(0)?;
        self.space();
        if self.at != self.bytes.len() {
            return Err(DecodeError::Syntax);
        }
        Ok(self.count)
    }

    fn reserve(&mut self) -> Result<u32, DecodeError> {
        let at = self.count;
        self.count = self.count.checked_add(1).ok_or(DecodeError::TooLong)?;
        Ok(at)
    }

    fn finish(&mut self, at: u32, start: usize) {
        if let Some(entries) = &mut self.entries {
            let entry = entries
                .get_mut(usize::try_from(at).expect("entry offset fits the bounded message"))
                .expect("counted index has room for every entry");
            *entry = Entry {
                start: u32::try_from(start).expect("start fits the bounded message"),
                end: u32::try_from(self.at).expect("end fits the bounded message"),
                after: self.count,
            };
        }
    }

    fn peek(&self) -> Option<u8> {
        self.bytes.get(self.at).copied()
    }

    fn space(&mut self) {
        while whitespace(self.peek()) {
            self.at = self.at.saturating_add(1);
        }
    }

    fn take(&mut self, expected: u8) -> Result<(), DecodeError> {
        if self.peek() != Some(expected) {
            return Err(DecodeError::Syntax);
        }
        self.at = self.at.checked_add(1).ok_or(DecodeError::Syntax)?;
        Ok(())
    }

    fn value(&mut self, depth: u32) -> Result<(), DecodeError> {
        let start = self.at;
        let at = self.reserve()?;
        match self.peek() {
            Some(b'{') => self.object(depth),
            Some(b'[') => self.array(depth),
            Some(b'"') => self.string(),
            Some(b't') => self.literal(b"true"),
            Some(b'f') => self.literal(b"false"),
            Some(b'n') => self.literal(b"null"),
            Some(b'-' | b'0'..=b'9') => self.number(),
            _ => Err(DecodeError::Syntax),
        }?;
        self.finish(at, start);
        Ok(())
    }

    fn object(&mut self, depth: u32) -> Result<(), DecodeError> {
        if depth >= 64 {
            return Err(DecodeError::TooDeep);
        }
        self.take(b'{')?;
        self.space();
        if self.peek() == Some(b'}') {
            return self.take(b'}');
        }
        loop {
            let start = self.at;
            let at = self.reserve()?;
            self.string()?;
            self.finish(at, start);
            self.space();
            self.take(b':')?;
            self.space();
            self.value(depth.checked_add(1).ok_or(DecodeError::TooDeep)?)?;
            self.space();
            if self.peek() != Some(b',') {
                return self.take(b'}');
            }
            self.take(b',')?;
            self.space();
        }
    }

    fn array(&mut self, depth: u32) -> Result<(), DecodeError> {
        if depth >= 64 {
            return Err(DecodeError::TooDeep);
        }
        self.take(b'[')?;
        self.space();
        if self.peek() == Some(b']') {
            return self.take(b']');
        }
        loop {
            self.value(depth.checked_add(1).ok_or(DecodeError::TooDeep)?)?;
            self.space();
            if self.peek() != Some(b',') {
                return self.take(b']');
            }
            self.take(b',')?;
            self.space();
        }
    }

    fn string(&mut self) -> Result<(), DecodeError> {
        self.take(b'"')?;
        let mut segment = self.at;
        loop {
            let byte = self.peek().ok_or(DecodeError::Syntax)?;
            if byte == b'"' {
                valid_text(self.bytes.get(segment..self.at).ok_or(DecodeError::Syntax)?)?;
                return self.take(b'"');
            }
            if byte == b'\\' {
                valid_text(self.bytes.get(segment..self.at).ok_or(DecodeError::Syntax)?)?;
                self.take(b'\\')?;
                let escaped = self.peek().ok_or(DecodeError::Syntax)?;
                match escaped {
                    b'"' | b'\\' | b'/' | b'b' | b'f' | b'n' | b'r' | b't' => {
                        self.at = self.at.checked_add(1).ok_or(DecodeError::Syntax)?;
                    }
                    b'u' => {
                        let first = hex4(self.bytes, self.at.checked_add(1).ok_or(DecodeError::Syntax)?)
                            .ok_or(DecodeError::Syntax)?;
                        self.at = self.at.checked_add(5).ok_or(DecodeError::Syntax)?;
                        if (0xD800..=0xDBFF).contains(&first) {
                            self.take(b'\\')?;
                            self.take(b'u')?;
                            let second = hex4(self.bytes, self.at).ok_or(DecodeError::Syntax)?;
                            if !(0xDC00..=0xDFFF).contains(&second) {
                                return Err(DecodeError::Syntax);
                            }
                            self.at = self.at.checked_add(4).ok_or(DecodeError::Syntax)?;
                        } else if (0xDC00..=0xDFFF).contains(&first) {
                            return Err(DecodeError::Syntax);
                        }
                    }
                    _ => return Err(DecodeError::Syntax),
                }
                segment = self.at;
            } else if byte < 0x20 {
                return Err(DecodeError::Syntax);
            } else {
                self.at = self.at.checked_add(1).ok_or(DecodeError::Syntax)?;
            }
        }
    }

    fn literal(&mut self, literal: &[u8]) -> Result<(), DecodeError> {
        let end = self.at.checked_add(literal.len()).ok_or(DecodeError::Syntax)?;
        if self.bytes.get(self.at..end) != Some(literal) {
            return Err(DecodeError::Syntax);
        }
        self.at = end;
        Ok(())
    }

    fn number(&mut self) -> Result<(), DecodeError> {
        if self.peek() == Some(b'-') {
            self.at = self.at.checked_add(1).ok_or(DecodeError::Syntax)?;
        }
        match self.peek() {
            Some(b'0') => self.at = self.at.checked_add(1).ok_or(DecodeError::Syntax)?,
            Some(b'1'..=b'9') => {
                self.at = self.at.checked_add(1).ok_or(DecodeError::Syntax)?;
                while digit(self.peek()) {
                    self.at = self.at.checked_add(1).ok_or(DecodeError::Syntax)?;
                }
            }
            _ => return Err(DecodeError::Syntax),
        }
        if self.peek() == Some(b'.') {
            self.at = self.at.checked_add(1).ok_or(DecodeError::Syntax)?;
            self.digits()?;
        }
        if exponent(self.peek()) {
            self.at = self.at.checked_add(1).ok_or(DecodeError::Syntax)?;
            if sign(self.peek()) {
                self.at = self.at.checked_add(1).ok_or(DecodeError::Syntax)?;
            }
            self.digits()?;
        }
        Ok(())
    }

    fn digits(&mut self) -> Result<(), DecodeError> {
        if !digit(self.peek()) {
            return Err(DecodeError::Syntax);
        }
        while digit(self.peek()) {
            self.at = self.at.checked_add(1).ok_or(DecodeError::Syntax)?;
        }
        Ok(())
    }
}

fn hex4(bytes: &[u8], start: usize) -> Option<u16> {
    let mut value = 0_u16;
    for offset in 0..4_usize {
        let digit = *bytes.get(start.checked_add(offset)?)?;
        let nibble = match digit {
            b'0'..=b'9' => u16::from(digit.checked_sub(b'0')?),
            b'a'..=b'f' => u16::from(digit.checked_sub(b'a')?).checked_add(10)?,
            b'A'..=b'F' => u16::from(digit.checked_sub(b'A')?).checked_add(10)?,
            _ => return None,
        };
        value = value.checked_mul(16)?.checked_add(nibble)?;
    }
    Some(value)
}

fn unsigned(bytes: &[u8]) -> Option<u64> {
    if bytes.is_empty() {
        return None;
    }
    let mut value = 0_u64;
    for byte in bytes {
        if !digit(Some(*byte)) {
            return None;
        }
        value = value.checked_mul(10)?.checked_add(u64::from(byte.checked_sub(b'0')?))?;
    }
    Some(value)
}

fn signed(bytes: &[u8]) -> Option<i64> {
    if bytes.first() == Some(&b'-') {
        let magnitude = unsigned(bytes.get(1..)?)?;
        if magnitude == i64::MIN.unsigned_abs() { Some(i64::MIN) } else { i64::try_from(magnitude).ok()?.checked_neg() }
    } else {
        i64::try_from(unsigned(bytes)?).ok()
    }
}

fn whitespace(byte: Option<u8>) -> bool {
    match byte {
        Some(b' ' | b'\n' | b'\r' | b'\t') => true,
        Some(_) | None => false,
    }
}

fn digit(byte: Option<u8>) -> bool {
    match byte {
        Some(b'0'..=b'9') => true,
        Some(_) | None => false,
    }
}

fn exponent(byte: Option<u8>) -> bool {
    match byte {
        Some(b'e' | b'E') => true,
        Some(_) | None => false,
    }
}

fn sign(byte: Option<u8>) -> bool {
    match byte {
        Some(b'+' | b'-') => true,
        Some(_) | None => false,
    }
}

#[expect(clippy::disallowed_methods, reason = "a JSON string must contain valid UTF-8")]
fn valid_text(bytes: &[u8]) -> Result<(), DecodeError> {
    match core::str::from_utf8(bytes) {
        Ok(_) => Ok(()),
        Err(_) => Err(DecodeError::Syntax),
    }
}

#[cfg(test)]
mod tests {
    use super::{DecodeError, Document, Vec};

    #[test]
    fn nested_reply_and_escaped_text() {
        let doc = Document::parse(br#"{"sessionId":"s","result":{"nodes":[{"role":{"value":"button"},"name":{"value":"a\u00e9"},"backendDOMNodeId":42}]},"id":7}"#, 512)
            .expect("valid CDP reply");
        let root = doc.root();
        assert_eq!(root.get(b"id").expect("id").u64(), Some(7));
        assert_eq!(root.get(b"sessionId").expect("session").text().expect("text").as_ref(), b"s");
        let mut nodes =
            root.get(b"result").expect("result").get(b"nodes").expect("nodes").array().expect("nodes array");
        let node = nodes.next().expect("one node");
        assert_eq!(node.get(b"backendDOMNodeId").expect("backend node").u64(), Some(42));
        assert_eq!(
            node.get(b"name").expect("name").get(b"value").expect("value").text().expect("text").as_ref(),
            "aé".as_bytes()
        );
        assert!(nodes.next().is_none());
    }

    #[test]
    fn malformed_and_over_limit() {
        assert_eq!(Document::parse(br#"{"x":01}"#, 100), Err(DecodeError::Syntax));
        assert_eq!(Document::parse(br#"{"x":1}"#, 2), Err(DecodeError::TooLong));
        assert_eq!(Document::parse(br#"{"x":"\uD800"}"#, 100), Err(DecodeError::Syntax));
    }

    #[test]
    fn fractional_css_pixels_drop_toward_zero() {
        let doc = Document::parse(br"[67.4375,-12.75,-0.5]", 100).expect("valid numbers");
        let mut values = doc.root().array().expect("an array");
        assert_eq!(values.next().expect("first").whole_pixels(), Some(67));
        assert_eq!(values.next().expect("second").whole_pixels(), Some(-12));
        assert_eq!(values.next().expect("third").whole_pixels(), Some(0));
    }

    #[test]
    fn members_skip_subtrees_at_their_own_depth() {
        let doc =
            Document::parse(br#"{"a":{"b":1,"c":[1,[2,{"b":3}]]},"b":{"x":"y"},"d":4}"#, 512).expect("nested members");
        let root = doc.root();
        assert_eq!(root.get(b"b").expect("root b").raw(), br#"{"x":"y"}"#);
        assert_eq!(root.get(b"d").expect("past subtrees").u64(), Some(4));
        assert!(root.get(b"c").is_none());
        assert_eq!(root.get(b"a").expect("a").get(b"c").expect("nested c").raw(), br#"[1,[2,{"b":3}]]"#);
    }

    #[test]
    fn duplicate_members_select_the_first() {
        let doc = Document::parse(br#"{"k":1,"k":2,"a":{"k":3,"k":4}}"#, 100).expect("duplicates");
        assert_eq!(doc.root().get(b"k").expect("first root member").u64(), Some(1));
        assert_eq!(doc.root().get(b"a").expect("a").get(b"k").expect("first nested member").u64(), Some(3));
    }

    #[test]
    fn escaped_members_match_unescaped_bytes() {
        let doc =
            Document::parse(br#"{"ab":1,"q\"":2,"ab":3,"a\u0062":4,"\uD83D\uDE00":5}"#, 100).expect("escaped keys");
        assert_eq!(doc.root().get(b"ab").expect("plain key").u64(), Some(1));
        assert_eq!(doc.root().get(b"q\"").expect("escaped quote").u64(), Some(2));
        assert_eq!(doc.root().get("😀".as_bytes()).expect("surrogate pair key").u64(), Some(5));
    }

    #[test]
    fn arrays_yield_whole_elements_then_nothing() {
        let doc = Document::parse(br#"[ 1 , "x" , {"k":[2]} , [] , null ]"#, 100).expect("array");
        let mut elements = doc.root().array().expect("array view");
        for expected in [b"1".as_slice(), br#""x""#, br#"{"k":[2]}"#, b"[]", b"null"] {
            assert_eq!(elements.next().expect("whole element").raw(), expected);
        }
        assert!(elements.next().is_none());
        assert!(elements.next().is_none());
        let empty = Document::parse(b"[]", 100).expect("empty array");
        assert!(empty.root().array().expect("empty view").next().is_none());
    }

    #[test]
    fn lookups_on_the_wrong_kind_find_nothing() {
        for bytes in [b"[]".as_slice(), br#""x""#, b"1", b"{}"] {
            let doc = Document::parse(bytes, 100).expect("value");
            assert!(doc.root().get(b"k").is_none());
        }
        for bytes in [b"{}".as_slice(), br#""x""#] {
            let doc = Document::parse(bytes, 100).expect("not an array");
            assert!(doc.root().array().is_none());
        }
    }

    #[test]
    fn raw_keeps_internal_whitespace_exact() {
        let doc = Document::parse(br#" { "a" : [ 1 , 2 ] , "b" : "c" } "#, 100).expect("spaces");
        assert_eq!(doc.root().raw(), br#"{ "a" : [ 1 , 2 ] , "b" : "c" }"#);
        assert_eq!(doc.root().get(b"a").expect("a").raw(), b"[ 1 , 2 ]");
    }

    #[test]
    fn deepest_subtree_leaves_later_members_visible() {
        for arrays in [63_usize, 64] {
            let mut bytes = Vec::from(br#"{"a":"#.as_slice());
            bytes.extend(core::iter::repeat_n(b'[', arrays));
            bytes.push(b'0');
            bytes.extend(core::iter::repeat_n(b']', arrays));
            bytes.extend_from_slice(br#", "b":1}"#);
            let parsed = Document::parse(&bytes, 512);
            if arrays == 63 {
                let doc = parsed.expect("64 containers are accepted");
                assert_eq!(doc.root().get(b"b").expect("past deepest subtree").u64(), Some(1));
            } else {
                assert_eq!(parsed, Err(DecodeError::TooDeep));
            }
        }
    }

    #[test]
    fn index_has_one_entry_per_value_and_key() {
        let doc = Document::parse(br#"{"a":[1,2],"b":{}}"#, 100).expect("indexed members");
        assert_eq!(doc.entries.len(), 7);
        assert_eq!(size_of::<super::Entry>(), 12);
        let mut entries = doc.entries.iter();
        for expected in [(0, 18, 7), (1, 4, 2), (5, 10, 5), (6, 7, 4), (8, 9, 5), (11, 14, 6), (15, 17, 7)] {
            let entry = entries.next().expect("entry in document order");
            assert_eq!((entry.start, entry.end, entry.after), expected);
        }
        assert!(entries.next().is_none());
    }
}
