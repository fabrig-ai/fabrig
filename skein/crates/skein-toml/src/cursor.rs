//! Byte grammar and line ends (skein/docs/design/data/toml.md, 2 and 5).

use crate::{Refusal, Why};

pub(crate) struct Cursor {
    pub(crate) at: usize,
    pub(crate) line: u64,
    invalid_utf8: bool,
}

impl Cursor {
    /// Grammar reads only the valid prefix. Its end is a fault in its own right
    /// if there are bad bytes, but an earlier grammar fault still wins.
    pub(crate) fn new(input: &[u8]) -> (Cursor, &[u8]) {
        let input = input.strip_prefix(b"\xef\xbb\xbf").unwrap_or(input);
        let (input, invalid_utf8) = valid_prefix(input);
        (Cursor { at: 0, line: 1, invalid_utf8 }, input)
    }

    pub(crate) fn peek(&self, input: &[u8]) -> Option<u8> {
        self.ahead(input, 0)
    }

    pub(crate) fn ahead(&self, input: &[u8], steps: usize) -> Option<u8> {
        input.get(self.at.checked_add(steps)?).copied()
    }

    pub(crate) fn take(&mut self, input: &[u8]) -> Result<u8, Refusal> {
        let Some(byte) = self.peek(input) else { return Err(self.grammar(input)) };
        self.at = self.at.saturating_add(1);
        if byte == b'\n' {
            // An admitted input's length fits u32; its line count can be one more.
            self.line = self.line.saturating_add(1);
        }
        Ok(byte)
    }

    pub(crate) fn eat(&mut self, input: &[u8], byte: u8) -> bool {
        if self.peek(input) != Some(byte) {
            return false;
        }
        self.take(input).expect("the byte was just seen");
        true
    }

    pub(crate) fn need(&mut self, input: &[u8], byte: u8) -> Result<(), Refusal> {
        if self.eat(input, byte) { Ok(()) } else { Err(self.grammar(input)) }
    }

    pub(crate) fn horizontal(&mut self, input: &[u8]) {
        for &byte in input.get(self.at..).expect("cursor stays within input") {
            match byte {
                b' ' | b'\t' => self.at = self.at.saturating_add(1),
                _ => break,
            }
        }
    }

    pub(crate) fn newline(&mut self, input: &[u8]) -> Result<(), Refusal> {
        if self.peek(input) == Some(b'\r') && self.ahead(input, 1) != Some(b'\n') {
            return Err(self.grammar(input));
        }
        if self.eat(input, b'\r') {
            self.need(input, b'\n')?;
            return Ok(());
        }
        self.need(input, b'\n')
    }

    pub(crate) fn comment(&mut self, input: &[u8]) -> Result<(), Refusal> {
        if !self.eat(input, b'#') {
            return Ok(());
        }
        for &byte in input.get(self.at..).expect("cursor stays within input") {
            match byte {
                b'\r' | b'\n' => return Ok(()),
                b'\x00'..=b'\x08' | b'\x0b'..=b'\x0c' | b'\x0e'..=b'\x1f' | b'\x7f' => return Err(self.grammar(input)),
                _ => self.at = self.at.saturating_add(1),
            }
        }
        self.end(input)
    }

    /// Whitespace and comments between statements and between array tokens.
    pub(crate) fn trivia(&mut self, input: &[u8]) -> Result<(), Refusal> {
        for _ in self.at..input.len() {
            match self.peek(input) {
                Some(b' ' | b'\t') => self.horizontal(input),
                Some(b'\r' | b'\n') => self.newline(input)?,
                Some(b'#') => self.comment(input)?,
                Some(_) | None => return Ok(()),
            }
        }
        Ok(())
    }

    pub(crate) fn statement_end(&mut self, input: &[u8]) -> Result<(), Refusal> {
        self.horizontal(input);
        self.comment(input)?;
        if self.at == input.len() { self.end(input) } else { self.newline(input) }
    }

    pub(crate) fn end(&self, input: &[u8]) -> Result<(), Refusal> {
        if self.at == input.len() && !self.invalid_utf8 { Ok(()) } else { Err(self.grammar(input)) }
    }

    pub(crate) const fn grammar(&self, input: &[u8]) -> Refusal {
        let why = if self.at == input.len() && self.invalid_utf8 { Why::InvalidUtf8 } else { Why::Grammar };
        self.refuse(why)
    }

    pub(crate) const fn refuse(&self, why: Why) -> Refusal {
        Refusal { line: self.line, why }
    }
}

#[expect(clippy::disallowed_methods, reason = "the TOML grammar checks UTF-8 and needs the offset of its first fault")]
fn valid_prefix(input: &[u8]) -> (&[u8], bool) {
    match core::str::from_utf8(input) {
        Ok(_) => (input, false),
        Err(error) => (input.get(..error.valid_up_to()).expect("UTF-8 validation reports a prefix of the input"), true),
    }
}
